use std::ffi::c_int;

use super::ffi_open::{c_text, read_c_buf, status_error};
use super::library::DssApi;
use super::time::Sample;
use crate::error::HmsError;

pub(crate) struct Retrieved {
    pub samples: Vec<Sample>,
    pub units: String,
    pub data_type: String,
    pub julian_base: i32,
}

const MAX_VALUES: i32 = 200_000;

impl DssApi {
    pub(crate) fn retrieve(
        &self,
        file: *mut std::ffi::c_void,
        pathname: &str,
        start_date: &str,
        start_time: &str,
        end_date: &str,
        end_time: &str,
    ) -> Result<Retrieved, HmsError> {
        let path = c_text(pathname)?;
        let start_date = c_text(start_date)?;
        let start_time = c_text(start_time)?;
        let end_date = c_text(end_date)?;
        let end_time = c_text(end_time)?;
        let mut count = 0i32;
        let mut quality_width = 0i32;
        let code = unsafe {
            (self.ts_sizes)(
                file,
                path.as_ptr(),
                start_date.as_ptr(),
                start_time.as_ptr(),
                end_date.as_ptr(),
                end_time.as_ptr(),
                &mut count,
                &mut quality_width,
            )
        };
        if code != 0 {
            return Err(status_error("hec_dss_tsGetSizes", code));
        }
        if count <= 0 {
            return Err(HmsError::message("la serie no tiene valores en esa ventana"));
        }
        if count > MAX_VALUES {
            return Err(HmsError::message(format!(
                "la serie tiene {count} valores; indique una ventana más corta"
            )));
        }
        self.read_values(file, &path, &start_date, &start_time, &end_date, &end_time, count)
    }
}

impl DssApi {
    fn read_values(
        &self,
        file: *mut std::ffi::c_void,
        path: &std::ffi::CString,
        start_date: &std::ffi::CString,
        start_time: &std::ffi::CString,
        end_date: &std::ffi::CString,
        end_time: &std::ffi::CString,
        count: i32,
    ) -> Result<Retrieved, HmsError> {
        let mut times = vec![0i32; count as usize];
        let mut values = vec![0f64; count as usize];
        let mut read = 0i32;
        let mut julian = 0i32;
        let mut granularity = 60i32;
        let mut units = vec![0u8; 128];
        let mut kind = vec![0u8; 64];
        let mut zone = vec![0u8; 64];
        let code = unsafe {
            (self.ts_retrieve)(
                file,
                path.as_ptr(),
                start_date.as_ptr(),
                start_time.as_ptr(),
                end_date.as_ptr(),
                end_time.as_ptr(),
                times.as_mut_ptr(),
                values.as_mut_ptr(),
                count as c_int,
                &mut read,
                std::ptr::null_mut(),
                0,
                &mut julian,
                &mut granularity,
                units.as_mut_ptr().cast(),
                units.len() as c_int,
                kind.as_mut_ptr().cast(),
                kind.len() as c_int,
                zone.as_mut_ptr().cast(),
                zone.len() as c_int,
            )
        };
        if code != 0 {
            return Err(status_error("hec_dss_tsRetrieve", code));
        }
        let n = (read.max(0) as usize).min(count as usize);
        let step = if granularity <= 0 { 60 } else { granularity as i64 };
        let samples = (0..n)
            .map(|index| Sample {
                offset_seconds: times[index] as i64 * step,
                value: values[index],
            })
            .collect();
        Ok(Retrieved {
            samples,
            units: read_c_buf(&units),
            data_type: read_c_buf(&kind),
            julian_base: julian,
        })
    }
}
