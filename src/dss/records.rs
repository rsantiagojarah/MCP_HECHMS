use std::path::Path;

use super::session::DssPool;
use crate::error::HmsError;

#[derive(Debug, Clone, serde::Serialize)]
pub struct StoredSeries {
    pub pathname: String,
    pub count: usize,
    pub units: String,
    pub data_type: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PairedData {
    pub pathname: String,
    pub ordinates: Vec<f64>,
    pub values: Vec<f64>,
    pub curves: i32,
    pub units_independent: String,
    pub units_dependent: String,
    pub type_independent: String,
    pub type_dependent: String,
    pub truncated: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GridSummary {
    pub pathname: String,
    pub grid_type: i32,
    pub cells_x: i32,
    pub cells_y: i32,
    pub cell_size: f32,
    pub min: f32,
    pub max: f32,
    pub mean: f32,
    pub units: String,
    pub total_cells: i32,
    pub offset: usize,
    pub truncated: bool,
    pub sample: Vec<f32>,
}

impl DssPool {
    pub fn store_series(
        &self,
        path: &Path,
        pathname: &str,
        start_date: &str,
        start_time: &str,
        values: &[f64],
        units: &str,
        data_type: &str,
    ) -> Result<StoredSeries, HmsError> {
        let mut values = values.to_vec();
        self.with_writable(path, |api, file| {
            api.store_regular(file, pathname, start_date, start_time, &mut values, units, data_type, "")?;
            Ok(StoredSeries {
                pathname: pathname.to_string(),
                count: values.len(),
                units: units.to_string(),
                data_type: data_type.to_string(),
            })
        })
    }

    pub fn paired(&self, path: &Path, pathname: &str, limit: usize) -> Result<PairedData, HmsError> {
        self.with_file(path, |api, file| {
            let (raw, truncated) = api.read_paired(file, pathname, limit)?;
            Ok(PairedData {
                pathname: pathname.to_string(),
                ordinates: raw.ordinates,
                values: raw.values,
                curves: raw.curves,
                units_independent: raw.units_independent,
                units_dependent: raw.units_dependent,
                type_independent: raw.type_independent,
                type_dependent: raw.type_dependent,
                truncated,
            })
        })
    }

    pub fn store_paired(
        &self,
        path: &Path,
        pathname: &str,
        ordinates: &[f64],
        values: &[f64],
        curves: i32,
        units_independent: &str,
        type_independent: &str,
        units_dependent: &str,
        type_dependent: &str,
    ) -> Result<PairedData, HmsError> {
        let mut ordinates = ordinates.to_vec();
        let mut values = values.to_vec();
        self.with_writable(path, |api, file| {
            api.store_paired(
                file,
                pathname,
                &mut ordinates,
                &mut values,
                curves,
                units_independent,
                type_independent,
                units_dependent,
                type_dependent,
            )?;
            let (raw, truncated) = api.read_paired(file, pathname, 500)?;
            Ok(PairedData {
                pathname: pathname.to_string(),
                ordinates: raw.ordinates,
                values: raw.values,
                curves: raw.curves,
                units_independent: raw.units_independent,
                units_dependent: raw.units_dependent,
                type_independent: raw.type_independent,
                type_dependent: raw.type_dependent,
                truncated,
            })
        })
    }

    pub fn grid(&self, path: &Path, pathname: &str, offset: usize, limit: usize) -> Result<GridSummary, HmsError> {
        self.with_file(path, |api, file| Ok(summary(pathname, offset, api.read_grid(file, pathname, offset, limit)?)))
    }

    pub fn store_grid(
        &self,
        path: &Path,
        pathname: &str,
        cells_x: i32,
        cells_y: i32,
        cell_size: f32,
        grid_type: i32,
        srs: &str,
        units: &str,
        values: &[f32],
    ) -> Result<GridSummary, HmsError> {
        let mut values = values.to_vec();
        self.with_writable(path, |api, file| {
            api.store_grid(file, pathname, cells_x, cells_y, cell_size, grid_type, srs, units, &mut values)?;
            Ok(summary(pathname, 0, api.read_grid(file, pathname, 0, 500)?))
        })
    }
}

fn summary(pathname: &str, offset: usize, raw: super::ffi_grid::GridRaw) -> GridSummary {
    let total = raw.cells_x.saturating_mul(raw.cells_y);
    GridSummary {
        pathname: pathname.to_string(),
        grid_type: raw.grid_type,
        cells_x: raw.cells_x,
        cells_y: raw.cells_y,
        cell_size: raw.cell_size,
        min: raw.min,
        max: raw.max,
        mean: raw.mean,
        units: raw.units,
        total_cells: total,
        offset,
        truncated: offset + raw.sample.len() < total.max(0) as usize,
        sample: raw.sample,
    }
}
