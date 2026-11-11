use std::path::Path;

use super::encoding;
use super::stamp;
use super::ProjectStore;
use crate::error::HmsError;

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ScsMethod {
    pub element: String,
    pub curve_number: String,
    pub initial_abstraction: String,
    pub impervious_percent: String,
    pub lag_minutes: String,
    pub unitgraph: String,
}

pub struct ScsInput<'a> {
    pub element: &'a str,
    pub curve_number: f64,
    pub lag_minutes: f64,
    pub initial_abstraction: Option<f64>,
    pub impervious_percent: Option<f64>,
    pub unitgraph: Option<&'a str>,
}

impl ProjectStore {
    pub fn apply_scs(&self, path: &Path, basin: Option<&str>, input: ScsInput<'_>) -> Result<ScsMethod, HmsError> {
        let file = self.basin_path(path, basin)?;
        let text = encoding::read_text(&file)?;
        let updated = apply_text(&text, &input)?;
        encoding::write_text(&file, &updated.text)?;
        self.cache.forget(&file);
        Ok(updated.method)
    }
}

struct Applied {
    text: String,
    method: ScsMethod,
}

fn apply_text(text: &str, input: &ScsInput<'_>) -> Result<Applied, HmsError> {
    let curve = checked(input.curve_number, 1.0, 100.0, "el número de curva debe estar entre 1 y 100")?;
    let lag = positive(input.lag_minutes, "el tiempo de retardo debe ser mayor que cero")?;
    let impervious = checked(input.impervious_percent.unwrap_or(0.0), 0.0, 100.0, "el porcentaje impermeable debe estar entre 0 y 100")?;
    let initial = match input.initial_abstraction {
        Some(value) => non_negative(value, "la abstracción inicial no puede ser negativa")?,
        None => default_abstraction(curve, text_is_metric(text)),
    };
    let unitgraph = unitgraph(input.unitgraph.unwrap_or("standard"))?;
    let ending = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let lines = split_lines(text);
    let (start, end) = subbasin(&lines, input.element)?;
    let loss = lines_of(
        ending,
        &[
            ("LossRate", "SCS"),
            ("Percent Impervious Area", &fmt_num(impervious)),
            ("Curve Number", &fmt_num(curve)),
            ("Initial Abstraction", &fmt_num(initial)),
        ],
    );
    let transform = lines_of(ending, &[("Transform", "SCS"), ("Unitgraph Type", &unitgraph), ("Lag", &fmt_num(lag))]);
    let mut lines = rewrite(&lines, start, end, loss, transform);
    let end = block_end(&lines, start)?;
    touch(&mut lines, start, end, ending);
    Ok(Applied {
        text: lines.concat(),
        method: ScsMethod {
            element: input.element.to_string(),
            curve_number: fmt_num(curve),
            initial_abstraction: fmt_num(initial),
            impervious_percent: fmt_num(impervious),
            lag_minutes: fmt_num(lag),
            unitgraph,
        },
    })
}

fn rewrite(lines: &[String], start: usize, end: usize, loss: Vec<String>, transform: Vec<String>) -> Vec<String> {
    let mut out = lines[..=start].to_vec();
    let mut wrote_loss = false;
    let mut wrote_transform = false;
    let mut skip = false;
    for line in &lines[start + 1..end] {
        let key = field_name(line).unwrap_or("");
        if key.eq_ignore_ascii_case("LossRate") {
            out.extend(loss.iter().cloned());
            wrote_loss = true;
            skip = true;
            continue;
        }
        if key.eq_ignore_ascii_case("Transform") {
            if !wrote_loss {
                out.extend(loss.iter().cloned());
                wrote_loss = true;
            }
            out.extend(transform.iter().cloned());
            wrote_transform = true;
            skip = true;
            continue;
        }
        if key.eq_ignore_ascii_case("Baseflow") {
            if !wrote_loss {
                out.extend(loss.iter().cloned());
                wrote_loss = true;
            }
            if !wrote_transform {
                out.extend(transform.iter().cloned());
                wrote_transform = true;
            }
            skip = false;
        }
        if skip {
            continue;
        }
        out.push(line.clone());
    }
    if !wrote_loss {
        out.extend(loss);
    }
    if !wrote_transform {
        out.extend(transform);
    }
    out.extend_from_slice(&lines[end..]);
    out
}

fn lines_of(ending: &str, fields: &[(&str, &str)]) -> Vec<String> {
    fields.iter().map(|(key, value)| format!("     {key}: {value}{ending}")).collect()
}

fn touch(lines: &mut [String], start: usize, end: usize, ending: &str) {
    let (date, time) = stamp::date_time();
    put_field(lines, start, end, ending, "Last Modified Date", &date);
    put_field(lines, start, end, ending, "Last Modified Time", &time);
}

fn put_field(lines: &mut [String], start: usize, end: usize, ending: &str, key: &str, value: &str) {
    if let Some(index) = field_at(lines, start, end, key) {
        let prefix = lines[index].split_once(':').map(|(left, _)| format!("{left}: ")).unwrap_or_else(|| format!("     {key}: "));
        lines[index] = format!("{prefix}{value}{ending}");
    }
}

fn subbasin(lines: &[String], element: &str) -> Result<(usize, usize), HmsError> {
    let mut found = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let Some((key, name)) = top_pair(line) else { continue };
        if key == "Subbasin" && name.eq_ignore_ascii_case(element.trim()) {
            found.push(index);
        }
    }
    let start = match found.as_slice() {
        [only] => *only,
        [] => return Err(HmsError::message(format!("no está la subcuenca {element}"))),
        _ => return Err(HmsError::message(format!("la subcuenca {element} está repetida"))),
    };
    Ok((start, block_end(lines, start)?))
}

fn block_end(lines: &[String], start: usize) -> Result<usize, HmsError> {
    lines.iter().enumerate().skip(start + 1).find_map(|(index, line)| {
        top_pair(line).filter(|(key, value)| *key == "End" && value.is_empty()).map(|_| index)
    }).ok_or_else(|| HmsError::message("la subcuenca no tiene End:"))
}

fn field_at(lines: &[String], start: usize, end: usize, key: &str) -> Option<usize> {
    (start + 1..end).find(|&index| field_name(&lines[index]).is_some_and(|name| name.eq_ignore_ascii_case(key)))
}

fn field_name(line: &str) -> Option<&str> {
    let raw = line.trim_end_matches(['\n', '\r']);
    if raw.is_empty() || !raw.starts_with([' ', '\t']) { return None; }
    let (key, _) = raw.trim_start().split_once(':')?;
    Some(key.trim())
}

fn top_pair(line: &str) -> Option<(&str, &str)> {
    let raw = line.trim_end_matches(['\n', '\r']);
    if raw.is_empty() || raw.starts_with([' ', '\t']) { return None; }
    let (key, value) = raw.split_once(':')?;
    Some((key.trim(), value.trim()))
}

fn split_lines(text: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut start = 0;
    for (index, ch) in text.char_indices() {
        if ch == '\n' {
            lines.push(text[start..=index].to_string());
            start = index + 1;
        }
    }
    if start < text.len() {
        lines.push(text[start..].to_string());
    }
    lines
}

fn text_is_metric(text: &str) -> bool {
    text.lines().find_map(|line| {
        let raw = line.trim_end_matches('\r').trim();
        let (key, value) = raw.split_once(':')?;
        key.trim().eq_ignore_ascii_case("Unit System").then(|| !value.trim().eq_ignore_ascii_case("English"))
    }).unwrap_or(true)
}

fn default_abstraction(curve: f64, metric: bool) -> f64 {
    let storage = if metric { 25400.0 / curve - 254.0 } else { 1000.0 / curve - 10.0 };
    (0.2 * storage).max(0.0)
}

fn unitgraph(value: &str) -> Result<String, HmsError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "" | "standard" | "estandar" | "estándar" => Ok("STANDARD".into()),
        "delmarva" => Ok("DELMARVA".into()),
        _ => Err(HmsError::message("el hidrograma unitario debe ser standard o delmarva")),
    }
}

fn checked(value: f64, low: f64, high: f64, message: &str) -> Result<f64, HmsError> {
    if value.is_finite() && (low..=high).contains(&value) { Ok(value) } else { Err(HmsError::message(message)) }
}

fn positive(value: f64, message: &str) -> Result<f64, HmsError> {
    if value.is_finite() && value > 0.0 { Ok(value) } else { Err(HmsError::message(message)) }
}

fn non_negative(value: f64, message: &str) -> Result<f64, HmsError> {
    if value.is_finite() && value >= 0.0 { Ok(value) } else { Err(HmsError::message(message)) }
}

fn fmt_num(value: f64) -> String {
    if (value - value.round()).abs() < 1e-9 && value.abs() < 1_000_000.0 {
        return format!("{}", value.round() as i64);
    }
    let mut text = format!("{value:.4}");
    while text.ends_with('0') { text.pop(); }
    if text.ends_with('.') { text.pop(); }
    text
}
