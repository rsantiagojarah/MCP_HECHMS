use std::ffi::c_int;

use super::ffi_open::{c_text, read_c_buf, status_error};
use super::library::DssApi;
use crate::error::HmsError;

const PATH_WIDTH: usize = 1024;
const MAX_RECORDS: usize = 100_000;

impl DssApi {
    pub(crate) fn catalog(&self, file: *mut std::ffi::c_void, filter: &str) -> Result<Vec<String>, HmsError> {
        let count = unsafe { (self.record_count)(file) };
        if count < 0 {
            return Err(status_error("hec_dss_record_count", count));
        }
        if count == 0 {
            return Ok(Vec::new());
        }
        if count as usize > MAX_RECORDS {
            return Err(HmsError::message(
                "el catálogo supera 100000 registros; use un filtro de pathname más estrecho",
            ));
        }
        let width = PATH_WIDTH as c_int;
        let mut buffer = vec![0u8; count as usize * PATH_WIDTH];
        let mut types = vec![0i32; count as usize];
        let filter = c_text(filter)?;
        let code = unsafe {
            (self.catalog)(
                file,
                buffer.as_mut_ptr().cast(),
                types.as_mut_ptr(),
                filter.as_ptr(),
                count,
                width,
            )
        };
        if code < 0 {
            return Err(status_error("hec_dss_catalog", code));
        }
        let stored = (code as usize).min(count as usize);
        let mut paths = Vec::new();
        for index in 0..stored {
            let start = index * PATH_WIDTH;
            let text = read_c_buf(&buffer[start..start + PATH_WIDTH]);
            if !text.is_empty() {
                paths.push(text);
            }
        }
        Ok(paths)
    }
}
