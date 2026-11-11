use serde::Serialize;

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    pub offset_seconds: i64,
    pub value: f64,
}

pub fn is_missing(value: f64) -> bool {
    !value.is_finite() || value < -1.0e30
}

/// Día juliano de HEC: el 1 es 01Jan1900.
pub fn format_offset(julian_base: i32, offset_seconds: i64) -> String {
    let days = offset_seconds.div_euclid(86_400);
    let seconds = offset_seconds.rem_euclid(86_400) as i32;
    format_julian(julian_base.saturating_add(days as i32), seconds)
}

pub fn format_julian(day: i32, seconds: i32) -> String {
    let unix_days = day as i64 - 25_568;
    let (year, month, day_of_month) = civil_from_unix(unix_days);
    let seconds = seconds.rem_euclid(86_400);
    let hour = seconds / 3600;
    let minute = (seconds % 3600) / 60;
    format!(
        "{day_of_month:02}{}{year} {hour:02}:{minute:02}",
        MONTHS[(month as usize).saturating_sub(1).min(11)]
    )
}

pub fn interval_seconds(e_part: &str) -> Option<i64> {
    let text = e_part.trim().to_ascii_uppercase();
    let split = text.find(|ch: char| ch.is_ascii_alphabetic())?;
    let count: i64 = text[..split].parse().ok()?;
    let unit = &text[split..];
    let factor = if unit.starts_with("MIN") {
        60
    } else if unit.starts_with("HOUR") || unit == "HR" {
        3_600
    } else if unit.starts_with("DAY") {
        86_400
    } else if unit.starts_with("SEC") {
        1
    } else {
        return None;
    };
    Some(count * factor)
}

pub fn pathname_parts(pathname: &str) -> Vec<&str> {
    let text = pathname.trim();
    let text = text.strip_prefix('/').unwrap_or(text);
    let text = text.strip_suffix('/').unwrap_or(text);
    let mut parts: Vec<&str> = text.split('/').collect();
    while parts.len() < 6 {
        parts.push("");
    }
    parts.truncate(6);
    parts
}

pub fn d_part_window(pathname: &str) -> (String, String, String, String) {
    let parts = pathname_parts(pathname);
    let d_part = parts.get(3).copied().unwrap_or("");
    if let Some((start, end)) = d_part.split_once(" - ") {
        return (start.trim().into(), "0000".into(), end.trim().into(), "2400".into());
    }
    if d_part.len() >= 9 && !d_part.contains('*') {
        return (d_part.into(), "0000".into(), d_part.into(), "2400".into());
    }
    (String::new(), String::new(), String::new(), String::new())
}

fn civil_from_unix(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Point {
    pub time: String,
    pub value: f64,
}
