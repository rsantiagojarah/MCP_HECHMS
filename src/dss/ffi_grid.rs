use std::ffi::{c_char, c_int, c_void};

use super::ffi_open::{c_text, read_c_buf, status_error};
use super::library::DssApi;
use crate::error::HmsError;

pub(crate) type GridRetrieve = unsafe extern "C" fn(
    *mut c_void,
    *const c_char,
    c_int,
    *mut c_int,
    *mut c_int,
    *mut c_int,
    *mut c_int,
    *mut c_int,
    *mut c_int,
    *mut c_int,
    *mut c_int,
    *mut c_int,
    *mut c_int,
    *mut c_int,
    *mut c_char,
    c_int,
    *mut c_char,
    c_int,
    *mut c_char,
    c_int,
    *mut c_char,
    c_int,
    *mut c_char,
    c_int,
    *mut f32,
    *mut f32,
    *mut f32,
    *mut f32,
    *mut f32,
    *mut f32,
    *mut f32,
    *mut f32,
    c_int,
    *mut c_int,
    *mut f32,
    c_int,
) -> c_int;

pub(crate) type GridStore = unsafe extern "C" fn(
    *mut c_void,
    *const c_char,
    c_int,
    c_int,
    c_int,
    c_int,
    c_int,
    c_int,
    c_int,
    c_int,
    c_int,
    c_int,
    c_int,
    c_int,
    *const c_char,
    *const c_char,
    *const c_char,
    *const c_char,
    *const c_char,
    f32,
    f32,
    f32,
    f32,
    f32,
    f32,
    f32,
    *mut f32,
    *mut c_int,
    *mut f32,
) -> c_int;

pub(crate) struct GridRaw {
    pub grid_type: i32,
    pub cells_x: i32,
    pub cells_y: i32,
    pub cell_size: f32,
    pub min: f32,
    pub max: f32,
    pub mean: f32,
    pub units: String,
    pub sample: Vec<f32>,
}

impl DssApi {
    pub(crate) fn read_grid(&self, file: *mut c_void, pathname: &str, offset: usize, limit: usize) -> Result<GridRaw, HmsError> {
        let mut raw = self.grid_call(file, pathname, false, &mut [])?;
        let cells = raw.cells_x as usize * raw.cells_y as usize;
        if (1..=262_144).contains(&cells) {
            let mut data = vec![0.0f32; cells];
            raw = self.grid_call(file, pathname, true, &mut data)?;
            let start = offset.min(data.len());
            let end = (start + limit.clamp(1, 500)).min(data.len());
            raw.sample = data[start..end].to_vec();
        }
        Ok(raw)
    }

    pub(crate) fn store_grid(
        &self,
        file: *mut c_void,
        pathname: &str,
        cells_x: i32,
        cells_y: i32,
        cell_size: f32,
        grid_type: i32,
        srs_name: &str,
        units_text: &str,
        values: &mut [f32],
    ) -> Result<(), HmsError> {
        let cells = cells_x as usize * cells_y as usize;
        if cells == 0 || cells > 20_000 || values.len() != cells {
            return Err(HmsError::message("la grilla debe tener entre 1 y 20000 celdas"));
        }
        let finite: Vec<f32> = values.iter().copied().filter(|value| value.is_finite()).collect();
        let min = finite.iter().copied().fold(f32::MAX, f32::min);
        let max = finite.iter().copied().fold(f32::MIN, f32::max);
        let mean = if finite.is_empty() { 0.0 } else { finite.iter().sum::<f32>() / finite.len() as f32 };
        let path = c_text(pathname)?;
        let units = c_text(units_text)?;
        let source = c_text("MCP")?;
        let srs = c_text(srs_name)?;
        let definition = c_text("")?;
        let zone = c_text("UTC")?;
        let code = unsafe {
            (self.grid_store)(
                file,
                path.as_ptr(),
                grid_type,
                0,
                0,
                0,
                cells_x,
                cells_y,
                0,
                0,
                0,
                0,
                1,
                0,
                units.as_ptr(),
                source.as_ptr(),
                srs.as_ptr(),
                definition.as_ptr(),
                zone.as_ptr(),
                cell_size,
                0.0,
                0.0,
                -3.4028234e38,
                max,
                min,
                mean,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                values.as_mut_ptr(),
            )
        };
        if code != 0 {
            return Err(status_error("hec_dss_gridStore", code));
        }
        Ok(())
    }

    fn grid_call(&self, file: *mut c_void, pathname: &str, with_data: bool, data: &mut [f32]) -> Result<GridRaw, HmsError> {
        let path = c_text(pathname)?;
        let mut kind = 0i32;
        let mut data_type = 0i32;
        let mut left_x = 0i32;
        let mut left_y = 0i32;
        let mut cells_x = 0i32;
        let mut cells_y = 0i32;
        let mut ranges = 0i32;
        let mut srs_type = 0i32;
        let mut offset = 0i32;
        let mut interval = 0i32;
        let mut stamped = 0i32;
        let mut cell_size = 0f32;
        let mut origin_x = 0f32;
        let mut origin_y = 0f32;
        let mut null_value = 0f32;
        let mut max = 0f32;
        let mut min = 0f32;
        let mut mean = 0f32;
        let mut exceeding = 0i32;
        let mut units = vec![0u8; 64];
        let mut source = vec![0u8; 128];
        let mut srs_name = vec![0u8; 64];
        let mut srs = vec![0u8; 256];
        let mut zone = vec![0u8; 64];
        let code = unsafe {
            (self.grid_retrieve)(
                file,
                path.as_ptr(),
                i32::from(with_data),
                &mut kind,
                &mut data_type,
                &mut left_x,
                &mut left_y,
                &mut cells_x,
                &mut cells_y,
                &mut ranges,
                &mut srs_type,
                &mut offset,
                &mut interval,
                &mut stamped,
                units.as_mut_ptr().cast(),
                units.len() as c_int,
                source.as_mut_ptr().cast(),
                source.len() as c_int,
                srs_name.as_mut_ptr().cast(),
                srs_name.len() as c_int,
                srs.as_mut_ptr().cast(),
                srs.len() as c_int,
                zone.as_mut_ptr().cast(),
                zone.len() as c_int,
                &mut cell_size,
                &mut origin_x,
                &mut origin_y,
                &mut null_value,
                &mut max,
                &mut min,
                &mut mean,
                std::ptr::null_mut(),
                0,
                &mut exceeding,
                if data.is_empty() { std::ptr::null_mut() } else { data.as_mut_ptr() },
                data.len() as c_int,
            )
        };
        if code != 0 {
            return Err(status_error("hec_dss_gridRetrieve", code));
        }
        Ok(GridRaw {
            grid_type: kind,
            cells_x,
            cells_y,
            cell_size,
            min,
            max,
            mean,
            units: read_c_buf(&units),
            sample: Vec::new(),
        })
    }
}
