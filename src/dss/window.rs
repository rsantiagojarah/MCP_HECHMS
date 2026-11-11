use serde::Serialize;

use super::ffi_series::Retrieved;
use super::time::{Point, format_offset, is_missing};

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SeriesWindow {
    pub pathname: String,
    pub units: String,
    pub data_type: String,
    pub point_count: usize,
    pub returned: usize,
    pub downsampled: bool,
    pub points: Vec<Point>,
}

pub fn build(pathname: &str, series: &Retrieved, julian_base: i32, max_points: usize) -> SeriesWindow {
    let valid: Vec<_> = series
        .samples
        .iter()
        .filter(|sample| !is_missing(sample.value))
        .collect();
    let cap = max_points.clamp(1, 500);
    let downsampled = valid.len() > cap;
    let picked = if downsampled { pick(&valid, cap) } else { valid };
    let points = picked
        .into_iter()
        .map(|sample| Point {
            time: format_offset(julian_base, sample.offset_seconds),
            value: sample.value,
        })
        .collect::<Vec<_>>();
    let returned = points.len();
    SeriesWindow {
        pathname: pathname.to_string(),
        units: series.units.clone(),
        data_type: series.data_type.clone(),
        point_count: series.samples.len(),
        returned,
        downsampled,
        points,
    }
}

fn pick<'a>(samples: &[&'a super::time::Sample], cap: usize) -> Vec<&'a super::time::Sample> {
    if cap <= 1 {
        return samples.first().copied().into_iter().collect();
    }
    let last = samples.len() - 1;
    (0..cap)
        .map(|index| samples[index * last / (cap - 1)])
        .collect()
}
