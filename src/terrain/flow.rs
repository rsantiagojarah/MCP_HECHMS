use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};

use super::grid::{is_gap, Dem};

const STEP: [(i32, i32); 8] = [(-1, 0), (-1, 1), (0, 1), (1, 1), (1, 0), (1, -1), (0, -1), (-1, -1)];

pub fn watershed(dem: &Dem, row: usize, col: usize, snap: u32) -> (Vec<bool>, usize, usize, bool) {
    let filled = fill(dem);
    let mut direction = steepest(dem, &filled);
    resolve_flats(&filled, dem, &mut direction);
    let accumulation = accumulate(&direction, dem.rows, dem.cols);
    let (row, col, snapped) = snap_outlet(dem, &accumulation, row, col, snap);
    let mask = upstream(&direction, dem.rows, dem.cols, row, col);
    (mask, row, col, snapped)
}

fn fill(dem: &Dem) -> Vec<f32> {
    let mut height = dem.values.clone();
    let n = dem.rows * dem.cols;
    let mut closed = vec![false; n];
    let mut heap = BinaryHeap::new();
    for row in 0..dem.rows {
        for col in 0..dem.cols {
            let index = row * dem.cols + col;
            if is_gap(height[index], dem.nodata) {
                closed[index] = true;
                continue;
            }
            if row == 0 || col == 0 || row + 1 == dem.rows || col + 1 == dem.cols {
                closed[index] = true;
                heap.push(Cell { height: height[index], index });
            }
        }
    }
    while let Some(cell) = heap.pop() {
        let (row, col) = split(cell.index, dem.cols);
        for (dr, dc) in STEP {
            let Some(next) = shift(row, col, dr, dc, dem.rows, dem.cols) else { continue };
            if closed[next] {
                continue;
            }
            closed[next] = true;
            if height[next] < cell.height {
                height[next] = cell.height;
            }
            heap.push(Cell { height: height[next], index: next });
        }
    }
    height
}

fn steepest(dem: &Dem, height: &[f32]) -> Vec<u8> {
    let mut direction = vec![u8::MAX; height.len()];
    for index in 0..height.len() {
        if is_gap(dem.values[index], dem.nodata) {
            continue;
        }
        let (row, col) = split(index, dem.cols);
        let mut best = 0.0f32;
        let mut choice = u8::MAX;
        for (step, (dr, dc)) in STEP.iter().enumerate() {
            let Some(next) = shift(row, col, *dr, *dc, dem.rows, dem.cols) else { continue };
            if is_gap(dem.values[next], dem.nodata) {
                continue;
            }
            let drop = height[index] - height[next];
            let slope = drop / distance(*dr, *dc, dem.dx, dem.dy);
            if slope > best {
                best = slope;
                choice = step as u8;
            }
        }
        direction[index] = choice;
    }
    direction
}

fn resolve_flats(height: &[f32], dem: &Dem, direction: &mut [u8]) {
    let mut queue = VecDeque::new();
    for (index, step) in direction.iter().enumerate() {
        if *step != u8::MAX {
            queue.push_back(index);
        }
    }
    while let Some(index) = queue.pop_front() {
        let (row, col) = split(index, dem.cols);
        for (step, (dr, dc)) in STEP.iter().enumerate() {
            let Some(next) = shift(row, col, *dr, *dc, dem.rows, dem.cols) else { continue };
            if direction[next] != u8::MAX || is_gap(dem.values[next], dem.nodata) || height[next] != height[index] {
                continue;
            }
            let back = (step + 4) % 8;
            direction[next] = back as u8;
            queue.push_back(next);
        }
    }
}

fn accumulate(direction: &[u8], rows: usize, cols: usize) -> Vec<u32> {
    let mut incoming = vec![0u32; direction.len()];
    for (index, step) in direction.iter().enumerate() {
        if let Some(down) = downstream(index, *step, rows, cols) {
            incoming[down] += 1;
        }
    }
    let mut queue: VecDeque<usize> = incoming.iter().enumerate().filter_map(|(index, count)| (*count == 0).then_some(index)).collect();
    let mut area = vec![1u32; direction.len()];
    while let Some(index) = queue.pop_front() {
        let Some(down) = downstream(index, direction[index], rows, cols) else { continue };
        area[down] = area[down].saturating_add(area[index]);
        incoming[down] -= 1;
        if incoming[down] == 0 {
            queue.push_back(down);
        }
    }
    area
}

fn snap_outlet(dem: &Dem, accumulation: &[u32], row: usize, col: usize, snap: u32) -> (usize, usize, bool) {
    if snap == 0 {
        return (row, col, false);
    }
    let mut best = (row, col, accumulation[row * dem.cols + col]);
    let snap = snap as i32;
    for dr in -snap..=snap {
        for dc in -snap..=snap {
            let Some(index) = shift(row, col, dr, dc, dem.rows, dem.cols) else { continue };
            if is_gap(dem.values[index], dem.nodata) {
                continue;
            }
            if accumulation[index] > best.2 {
                best = (index / dem.cols, index % dem.cols, accumulation[index]);
            }
        }
    }
    (best.0, best.1, best.0 != row || best.1 != col)
}

fn upstream(direction: &[u8], rows: usize, cols: usize, row: usize, col: usize) -> Vec<bool> {
    let mut mask = vec![false; direction.len()];
    let mut stack = vec![row * cols + col];
    mask[row * cols + col] = true;
    while let Some(index) = stack.pop() {
        let (row, col) = split(index, cols);
        for (step, (dr, dc)) in STEP.iter().enumerate() {
            let Some(next) = shift(row, col, *dr, *dc, rows, cols) else { continue };
            let toward = (step + 4) % 8;
            if !mask[next] && direction[next] == toward as u8 {
                mask[next] = true;
                stack.push(next);
            }
        }
    }
    mask
}

fn downstream(index: usize, step: u8, rows: usize, cols: usize) -> Option<usize> {
    let step = usize::from(step);
    if step >= STEP.len() {
        return None;
    }
    let (row, col) = split(index, cols);
    shift(row, col, STEP[step].0, STEP[step].1, rows, cols)
}

fn shift(row: usize, col: usize, dr: i32, dc: i32, rows: usize, cols: usize) -> Option<usize> {
    let row = row as i32 + dr;
    let col = col as i32 + dc;
    if row < 0 || col < 0 || row as usize >= rows || col as usize >= cols {
        None
    } else {
        Some(row as usize * cols + col as usize)
    }
}

fn split(index: usize, cols: usize) -> (usize, usize) {
    (index / cols, index % cols)
}

fn distance(dr: i32, dc: i32, dx: f64, dy: f64) -> f32 {
    let x = dc as f64 * dx;
    let y = dr as f64 * dy;
    (x * x + y * y).sqrt().max(1e-9) as f32
}

#[derive(Clone, Copy)]
struct Cell {
    height: f32,
    index: usize,
}

impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool { self.height.to_bits() == other.height.to_bits() && self.index == other.index }
}
impl Eq for Cell {}
impl PartialOrd for Cell {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}
impl Ord for Cell {
    fn cmp(&self, other: &Self) -> Ordering {
        other.height.total_cmp(&self.height).then(other.index.cmp(&self.index))
    }
}
