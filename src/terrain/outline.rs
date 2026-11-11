use std::collections::HashMap;

use super::grid::Dem;

pub fn geojson(dem: &Dem, mask: &[bool], area_km2: f64) -> String {
    let ring = boundary(mask, dem.rows, dem.cols);
    let coordinates: Vec<String> = ring.iter().map(|(col, row)| {
        let x = dem.x_min + *col as f64 * dem.dx;
        let y = dem.y_max - *row as f64 * dem.dy;
        format!("[{x:.6}, {y:.6}]")
    }).collect();
    let body = if coordinates.len() >= 4 {
        format!("[{}]", coordinates.join(", "))
    } else {
        "[]".into()
    };
    format!("{{\"type\":\"Feature\",\"properties\":{{\"area_km2\":{area_km2:.6},\"cells\":{}}},\"geometry\":{{\"type\":\"Polygon\",\"coordinates\":[{body}]}}}}", mask.iter().filter(|cell| **cell).count())
}

fn boundary(mask: &[bool], rows: usize, cols: usize) -> Vec<(i32, i32)> {
    let mut next: HashMap<(i32, i32), Vec<(i32, i32)>> = HashMap::new();
    for row in 0..rows {
        for col in 0..cols {
            if !mask[row * cols + col] {
                continue;
            }
            let (row, col) = (row as i32, col as i32);
            let outside = |rr: i32, cc: i32| rr < 0 || cc < 0 || rr >= rows as i32 || cc >= cols as i32 || !mask[rr as usize * cols + cc as usize];
            if outside(row - 1, col) { push(&mut next, (col + 1, row), (col, row)); }
            if outside(row + 1, col) { push(&mut next, (col, row + 1), (col + 1, row + 1)); }
            if outside(row, col - 1) { push(&mut next, (col, row), (col, row + 1)); }
            if outside(row, col + 1) { push(&mut next, (col + 1, row + 1), (col + 1, row)); }
        }
    }
    let mut best = Vec::new();
    while let Some(start) = next.keys().next().copied() {
        let mut ring = vec![start];
        let mut cursor = start;
        loop {
            let Some(target) = next.get_mut(&cursor).and_then(|items| items.pop()) else {
                next.remove(&cursor);
                break;
            };
            if next.get(&cursor).is_some_and(|items| items.is_empty()) { next.remove(&cursor); }
            ring.push(target);
            if target == start || ring.len() > mask.len() * 4 { break; }
            cursor = target;
        }
        if ring.len() > best.len() { best = ring; }
    }
    best
}

fn push(next: &mut HashMap<(i32, i32), Vec<(i32, i32)>>, from: (i32, i32), to: (i32, i32)) {
    next.entry(from).or_default().push(to);
}
