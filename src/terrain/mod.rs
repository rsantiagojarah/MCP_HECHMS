use std::path::Path;

mod flow;
mod grid;
mod outline;

use grid::{is_gap, Dem};
use crate::error::HmsError;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Delineation {
    pub element: String,
    pub cells: usize,
    pub area_km2: f64,
    pub outlet_x: f64,
    pub outlet_y: f64,
    pub snapped: bool,
    pub geojson: String,
}

pub fn delineate(dem_path: &Path, outlet_x: f64, outlet_y: f64, element: &str, snap_cells: u32) -> Result<Delineation, HmsError> {
    let dem = Dem::load(dem_path)?;
    if dem.rows * dem.cols > 2_000_000 {
        return Err(HmsError::message("el modelo de elevación supera 2 millones de celdas"));
    }
    let (row, col) = dem.locate(outlet_x, outlet_y)?;
    if is_gap(dem.values[row * dem.cols + col], dem.nodata) {
        return Err(HmsError::message("el punto de salida cae en una celda sin elevación"));
    }
    let (mask, row, col, snapped) = flow::watershed(&dem, row, col, snap_cells);
    let cells = mask.iter().filter(|cell| **cell).count();
    let (x, y) = dem.center(row, col);
    let area_km2 = cells as f64 * cell_km2(&dem, y);
    let text = outline::geojson(&dem, &mask, area_km2);
    let geojson = dem_path.with_file_name(format!("{}_{}.geojson", stem(dem_path), slug(element)));
    std::fs::write(&geojson, text)?;
    Ok(Delineation {
        element: element.to_string(),
        cells,
        area_km2,
        outlet_x: x,
        outlet_y: y,
        snapped,
        geojson: geojson.display().to_string(),
    })
}

fn cell_km2(dem: &Dem, latitude: f64) -> f64 {
    let area = if dem.dx.abs() < 1.0 && dem.dy.abs() < 1.0 {
        let width = dem.dx.abs() * 111_320.0 * latitude.to_radians().cos().abs();
        let height = dem.dy.abs() * 110_540.0;
        width * height
    } else {
        dem.dx.abs() * dem.dy.abs()
    };
    area / 1_000_000.0
}

fn stem(path: &Path) -> String {
    path.file_stem().and_then(|item| item.to_str()).unwrap_or("cuenca").to_string()
}

fn slug(name: &str) -> String {
    name.split_whitespace().collect::<Vec<_>>().join("_")
}
