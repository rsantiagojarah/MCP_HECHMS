use std::ffi::{c_char, c_int, c_void};

use super::ffi_open::{c_text, status_error};
use super::library::DssApi;
use crate::error::HmsError;

pub(crate) type TsStoreRegular = unsafe extern "C" fn(
    *mut c_void,
    *const c_char,
    *const c_char,
    *const c_char,
    *mut f64,
    c_int,
    *mut c_int,
    c_int,
    c_int,
    *const c_char,
    *const c_char,
    *const c_char,
    c_int,
) -> c_int;

pub(crate) type TsStoreIrregular = unsafe extern "C" fn(
    *mut c_void,
    *const c_char,
    *const c_char,
    *mut c_int,
    c_int,
    *mut f64,
    c_int,
    *mut c_int,
    c_int,
    c_int,
    *const c_char,
    *const c_char,
    *const c_char,
    c_int,
) -> c_int;

pub(crate) type DeleteFn = unsafe extern "C" fn(*mut c_void, *const c_char) -> c_int;

impl DssApi {
    pub(crate) fn store_regular(
        &self,
        file: *mut c_void,
        pathname: &str,
        start_date: &str,
        start_time: &str,
        values: &mut [f64],
        units: &str,
        data_type: &str,
        time_zone: &str,
    ) -> Result<(), HmsError> {
        if values.is_empty() || values.len() > 20_000 {
            return Err(HmsError::message("la serie debe tener entre 1 y 20000 valores"));
        }
        let path = c_text(pathname)?;
        let start_date = c_text(start_date)?;
        let start_time = c_text(start_time)?;
        let units = c_text(units)?;
        let data_type = c_text(data_type)?;
        let time_zone = c_text(time_zone)?;
        let code = unsafe {
            (self.ts_store)(
                file,
                path.as_ptr(),
                start_date.as_ptr(),
                start_time.as_ptr(),
                values.as_mut_ptr(),
                values.len() as c_int,
                std::ptr::null_mut(),
                0,
                1,
                units.as_ptr(),
                data_type.as_ptr(),
                time_zone.as_ptr(),
                0,
            )
        };
        if code != 0 {
            return Err(status_error("hec_dss_tsStoreRegular", code));
        }
        Ok(())
    }

    pub(crate) fn store_irregular(
        &self,
        file: *mut c_void,
        pathname: &str,
        start_date: &str,
        times: &mut [i32],
        values: &mut [f64],
        units: &str,
        data_type: &str,
    ) -> Result<(), HmsError> {
        if values.is_empty() || values.len() > 20_000 || times.len() != values.len() {
            return Err(HmsError::message("la serie irregular necesita entre 1 y 20000 pares tiempo-valor"));
        }
        let path = c_text(pathname)?;
        let start_date = c_text(start_date)?;
        let units = c_text(units)?;
        let data_type = c_text(data_type)?;
        let zone = c_text("")?;
        let code = unsafe {
            (self.ts_irregular)(
                file,
                path.as_ptr(),
                start_date.as_ptr(),
                times.as_mut_ptr(),
                60,
                values.as_mut_ptr(),
                values.len() as c_int,
                std::ptr::null_mut(),
                0,
                1,
                units.as_ptr(),
                data_type.as_ptr(),
                zone.as_ptr(),
                1,
            )
        };
        if code != 0 {
            return Err(status_error("hec_dss_tsStoreIregular", code));
        }
        Ok(())
    }

    pub(crate) fn delete_record(&self, file: *mut c_void, pathname: &str) -> Result<(), HmsError> {
        let path = c_text(pathname)?;
        let code = unsafe { (self.delete)(file, path.as_ptr()) };
        if code != 0 {
            return Err(status_error("hec_dss_delete", code));
        }
        Ok(())
    }
}
