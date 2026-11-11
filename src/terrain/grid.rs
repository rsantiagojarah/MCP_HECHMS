use std::path::Path;

use tiff::decoder::{Decoder, DecodingResult};
use tiff::tags::Tag;

use crate::error::HmsError;

#[derive(Debug, Clone)]
pub struct Dem {
    pub rows: usize,
    pub cols: usize,
    pub values: Vec<f32>,
    pub nodata: f32,
    pub x_min: f64,
    pub y_max: f64,
    pub dx: f64,
    pub dy: f64,
}

impl Dem {
    pub fn load(path: &Path) -> Result<Self, HmsError> {
        let ext = path.extension().and_then(|item| item.to_str()).unwrap_or("").to_ascii_lowercase();
        if ext == "asc" || ext == "grd" {
            read_ascii(path)
        } else if ext == "tif" || ext == "tiff" {
            read_tiff(path)
        } else {
            Err(HmsError::message("el modelo de elevación debe ser .asc, .grd, .tif o .tiff"))
        }
    }

    pub fn locate(&self, x: f64, y: f64) -> Result<(usize, usize), HmsError> {
        let col = ((x - self.x_min) / self.dx).floor() as isize;
        let row = ((self.y_max - y) / self.dy).floor() as isize;
        if row < 0 || col < 0 || row as usize >= self.rows || col as usize >= self.cols {
            return Err(HmsError::message("el punto de salida queda fuera del modelo de elevación"));
        }
        Ok((row as usize, col as usize))
    }

    pub fn center(&self, row: usize, col: usize) -> (f64, f64) {
        (self.x_min + (col as f64 + 0.5) * self.dx, self.y_max - (row as f64 + 0.5) * self.dy)
    }
}

pub fn is_gap(value: f32, nodata: f32) -> bool {
    value.is_nan() || (!nodata.is_nan() && (value.to_bits() == nodata.to_bits() || (value - nodata).abs() <= 1e-4))
}

fn read_ascii(path: &Path) -> Result<Dem, HmsError> {
    let text = std::fs::read_to_string(path)?;
    let mut cols = 0usize;
    let mut rows = 0usize;
    let mut xll = 0.0;
    let mut yll = 0.0;
    let mut center = false;
    let mut dx = 0.0;
    let mut dy = 0.0;
    let mut nodata = -9999.0f32;
    let mut values = Vec::new();
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        let Some(key) = parts.next() else { continue };
        let lower = key.to_ascii_lowercase();
        if let Some(value) = header(&lower) {
            let token = parts.next().unwrap_or("");
            match value {
                "ncols" => cols = token.parse().map_err(|_| HmsError::message("ncols no es un entero"))?,
                "nrows" => rows = token.parse().map_err(|_| HmsError::message("nrows no es un entero"))?,
                "xll" => xll = number(token)?,
                "yll" => yll = number(token)?,
                "center" => center = true,
                "dx" => dx = number(token)?,
                "dy" => dy = number(token)?,
                "cell" => { dx = number(token)?; dy = dx; }
                "nodata" => nodata = number(token)? as f32,
                _ => {}
            }
            if lower.starts_with("xllcenter") || lower.starts_with("yllcenter") { center = true; }
            continue;
        }
        values.push(number(key)? as f32);
        values.extend(parts.map(|token| number(token).map(|item| item as f32)).collect::<Result<Vec<_>, _>>()?);
    }
    if cols == 0 || rows == 0 || dx == 0.0 || dy == 0.0 {
        return Err(HmsError::message("el ASCII grid no tiene tamaño o tamaño de celda"));
    }
    if values.len() != rows * cols {
        return Err(HmsError::message(format!("el grid tiene {} valores y se esperaban {}", values.len(), rows * cols)));
    }
    let (x_min, y_max) = if center { (xll - dx / 2.0, yll + (rows as f64 - 0.5) * dy) } else { (xll, yll + rows as f64 * dy) };
    Ok(Dem { rows, cols, values, nodata, x_min, y_max, dx, dy })
}

fn header(key: &str) -> Option<&'static str> {
    Some(match key {
        "ncols" => "ncols",
        "nrows" => "nrows",
        "xllcorner" | "xllcenter" => "xll",
        "yllcorner" | "yllcenter" => "yll",
        "dx" => "dx",
        "dy" => "dy",
        "cellsize" => "cell",
        "nodata_value" => "nodata",
        _ => return None,
    })
}

fn read_tiff(path: &Path) -> Result<Dem, HmsError> {
    let file = std::fs::File::open(path)?;
    let mut decoder = Decoder::new(file).map_err(|err| HmsError::message(format!("GeoTIFF: {err}")))?;
    let (cols, rows) = decoder.dimensions().map_err(|err| HmsError::message(format!("GeoTIFF: {err}")))?;
    let nodata = gdal_nodata(&mut decoder).unwrap_or(f32::NAN);
    let (x_min, y_max, dx, dy) = geo_frame(&mut decoder, rows);
    let image = decoder.read_image().map_err(|err| HmsError::message(format!("GeoTIFF: {err}")))?;
    let values = samples(image, (rows as usize) * (cols as usize))?;
    Ok(Dem { rows: rows as usize, cols: cols as usize, values, nodata, x_min, y_max, dx, dy })
}

fn geo_frame(decoder: &mut Decoder<std::fs::File>, rows: u32) -> (f64, f64, f64, f64) {
    let scale = tag_f64(decoder, 33550);
    let tie = tag_f64(decoder, 33922);
    let dx = scale.first().copied().filter(|value| *value != 0.0).unwrap_or(1.0);
    let dy = scale.get(1).copied().filter(|value| *value != 0.0).unwrap_or(dx);
    let (col, row, x, y) = if tie.len() >= 6 { (tie[0], tie[1], tie[3], tie[4]) } else { (0.0, 0.0, 0.0, rows as f64) };
    (x - col * dx, y + row * dy, dx.abs(), dy.abs())
}

fn tag_f64(decoder: &mut Decoder<std::fs::File>, tag: u16) -> Vec<f64> {
    decoder.find_tag(Tag::Unknown(tag)).ok().flatten().map(|value| doubles(&value)).unwrap_or_default()
}

fn gdal_nodata(decoder: &mut Decoder<std::fs::File>) -> Option<f32> {
    let value = decoder.find_tag(Tag::Unknown(42113)).ok().flatten()?;
    match value {
        tiff::decoder::ifd::Value::Ascii(text) => text.trim().parse().ok(),
        other => doubles(&other).first().map(|item| *item as f32),
    }
}

fn doubles(value: &tiff::decoder::ifd::Value) -> Vec<f64> {
    match value {
        tiff::decoder::ifd::Value::List(items) => items.iter().filter_map(one_f64).collect(),
        other => one_f64(other).into_iter().collect(),
    }
}

fn one_f64(value: &tiff::decoder::ifd::Value) -> Option<f64> {
    use tiff::decoder::ifd::Value::*;
    match *value {
        Double(item) => Some(item),
        Float(item) => Some(item as f64),
        Unsigned(item) => Some(item as f64),
        Signed(item) => Some(item as f64),
        Short(item) => Some(item as f64),
        SignedShort(item) => Some(item as f64),
        Byte(item) => Some(item as f64),
        UnsignedBig(item) => Some(item as f64),
        SignedBig(item) => Some(item as f64),
        Rational(num, den) if den != 0 => Some(num as f64 / den as f64),
        SRational(num, den) if den != 0 => Some(num as f64 / den as f64),
        _ => None,
    }
}

fn samples(image: DecodingResult, count: usize) -> Result<Vec<f32>, HmsError> {
    let mut values = match image {
        DecodingResult::F32(items) => items,
        DecodingResult::F64(items) => items.into_iter().map(|item| item as f32).collect(),
        DecodingResult::U8(items) => items.into_iter().map(|item| item as f32).collect(),
        DecodingResult::U16(items) => items.into_iter().map(|item| item as f32).collect(),
        DecodingResult::U32(items) => items.into_iter().map(|item| item as f32).collect(),
        DecodingResult::I16(items) => items.into_iter().map(|item| item as f32).collect(),
        DecodingResult::I32(items) => items.into_iter().map(|item| item as f32).collect(),
        DecodingResult::I8(items) => items.into_iter().map(|item| item as f32).collect(),
        _ => return Err(HmsError::message("el GeoTIFF no es una banda numérica")),
    };
    if values.len() > count && values.len() % count == 0 {
        let bands = values.len() / count;
        values = values.into_iter().step_by(bands).collect();
    }
    if values.len() != count {
        return Err(HmsError::message("el GeoTIFF no tiene una banda de elevación completa"));
    }
    Ok(values)
}

fn number(text: &str) -> Result<f64, HmsError> {
    text.parse().map_err(|_| HmsError::message(format!("número no válido: {text}")))
}
