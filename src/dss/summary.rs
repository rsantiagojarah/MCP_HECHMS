use serde::Serialize;

use super::time::{Sample, format_offset, is_missing};

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SeriesSummary {
    pub pathname: String,
    pub units: String,
    pub data_type: String,
    pub interval_seconds: i64,
    pub count: usize,
    pub valid_count: usize,
    pub peak: Option<f64>,
    pub time_of_peak: Option<String>,
    pub min: Option<f64>,
    pub integral_value_seconds: Option<f64>,
}

pub fn summarize(
    pathname: &str,
    samples: &[Sample],
    units: &str,
    data_type: &str,
    interval_seconds: i64,
    julian_base: i32,
) -> SeriesSummary {
    let mut peak = None;
    let mut time_of_peak = None;
    let mut min = None;
    let mut valid = 0usize;
    let mut sum = 0.0;
    let mut first = None;
    let mut last = None;
    for sample in samples {
        if is_missing(sample.value) {
            continue;
        }
        valid += 1;
        sum += sample.value;
        first.get_or_insert(sample.value);
        last = Some(sample.value);
        if peak.is_none_or(|current: f64| sample.value > current) {
            peak = Some(sample.value);
            time_of_peak = Some(format_offset(julian_base, sample.offset_seconds));
        }
        if min.is_none_or(|current: f64| sample.value < current) {
            min = Some(sample.value);
        }
    }
    let integral = if valid == 0 {
        None
    } else if data_type.to_ascii_uppercase().contains("CUM") {
        Some(last.unwrap_or(0.0) - first.unwrap_or(0.0))
    } else if interval_seconds > 0 {
        Some(sum * interval_seconds as f64)
    } else {
        None
    };
    SeriesSummary {
        pathname: pathname.to_string(),
        units: units.to_string(),
        data_type: data_type.to_string(),
        interval_seconds,
        count: samples.len(),
        valid_count: valid,
        peak,
        time_of_peak,
        min,
        integral_value_seconds: integral,
    }
}
