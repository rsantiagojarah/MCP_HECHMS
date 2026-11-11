use std::ffi::{c_char, c_int, c_void};

use super::ffi_open::{c_text, read_c_buf, status_error};
use super::library::DssApi;
use crate::error::HmsError;

pub(crate) type PdInfo = unsafe extern "C" fn(
    *mut c_void,
    *const c_char,
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
    *mut c_int,
) -> c_int;

pub(crate) type PdRetrieve = unsafe extern "C" fn(
    *mut c_void,
    *const c_char,
    *mut f64,
    c_int,
    *mut f64,
    c_int,
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
    *mut c_char,
    c_int,
) -> c_int;

pub(crate) type PdStore = unsafe extern "C" fn(
    *mut c_void,
    *const c_char,
    *mut f64,
    c_int,
    *mut f64,
    c_int,
    c_int,
    c_int,
    *const c_char,
    *const c_char,
    *const c_char,
    *const c_char,
    *const c_char,
    c_int,
    *const c_char,
) -> c_int;

pub(crate) struct PairedRaw {
    pub ordinates: Vec<f64>,
    pub values: Vec<f64>,
    pub curves: i32,
    pub units_independent: String,
    pub units_dependent: String,
    pub type_independent: String,
    pub type_dependent: String,
}

impl DssApi {
    pub(crate) fn read_paired(&self, file: *mut c_void, pathname: &str, limit: usize) -> Result<(PairedRaw, bool), HmsError> {
        let path = c_text(pathname)?;
        let mut rows = 0i32;
        let mut curves = 0i32;
        let mut labels_len = 0i32;
        let mut unit_i = vec![0u8; 64];
        let mut unit_d = vec![0u8; 64];
        let mut type_i = vec![0u8; 64];
        let mut type_d = vec![0u8; 64];
        let info = unsafe {
            (self.pd_info)(
                file,
                path.as_ptr(),
                &mut rows,
                &mut curves,
                unit_i.as_mut_ptr().cast(),
                unit_i.len() as c_int,
                unit_d.as_mut_ptr().cast(),
                unit_d.len() as c_int,
                type_i.as_mut_ptr().cast(),
                type_i.len() as c_int,
                type_d.as_mut_ptr().cast(),
                type_d.len() as c_int,
                &mut labels_len,
            )
        };
        if info != 0 || rows <= 0 || curves <= 0 {
            return Err(status_error("hec_dss_pdRetrieveInfo", info));
        }
        let total = rows as usize * curves as usize;
        if total > 20_000 {
            return Err(HmsError::message("la curva DSS supera 20000 valores"));
        }
        let mut ordinates = vec![0.0; rows as usize];
        let mut values = vec![0.0; total];
        let mut labels = vec![0u8; (labels_len.max(1) as usize).min(8_192)];
        let mut zone = vec![0u8; 64];
        let code = unsafe {
            (self.pd_retrieve)(
                file,
                path.as_ptr(),
                ordinates.as_mut_ptr(),
                ordinates.len() as c_int,
                values.as_mut_ptr(),
                values.len() as c_int,
                &mut rows,
                &mut curves,
                unit_i.as_mut_ptr().cast(),
                unit_i.len() as c_int,
                type_i.as_mut_ptr().cast(),
                type_i.len() as c_int,
                unit_d.as_mut_ptr().cast(),
                unit_d.len() as c_int,
                type_d.as_mut_ptr().cast(),
                type_d.len() as c_int,
                labels.as_mut_ptr().cast(),
                labels.len() as c_int,
                zone.as_mut_ptr().cast(),
                zone.len() as c_int,
            )
        };
        if code != 0 {
            return Err(status_error("hec_dss_pdRetrieve", code));
        }
        let keep = limit.clamp(1, 500).min(ordinates.len());
        let truncated = keep < ordinates.len();
        ordinates.truncate(keep);
        values.truncate(keep * curves as usize);
        Ok((
            PairedRaw {
                ordinates,
                values,
                curves,
                units_independent: read_c_buf(&unit_i),
                units_dependent: read_c_buf(&unit_d),
                type_independent: read_c_buf(&type_i),
                type_dependent: read_c_buf(&type_d),
            },
            truncated,
        ))
    }

    pub(crate) fn store_paired(
        &self,
        file: *mut c_void,
        pathname: &str,
        ordinates: &mut [f64],
        values: &mut [f64],
        curves: i32,
        units_independent: &str,
        type_independent: &str,
        units_dependent: &str,
        type_dependent: &str,
    ) -> Result<(), HmsError> {
        if ordinates.is_empty() || curves <= 0 || values.len() != ordinates.len() * curves as usize {
            return Err(HmsError::message("las curvas no coinciden con las ordenadas"));
        }
        let path = c_text(pathname)?;
        let unit_i = c_text(units_independent)?;
        let type_i = c_text(type_independent)?;
        let unit_d = c_text(units_dependent)?;
        let type_d = c_text(type_dependent)?;
        let zone = c_text("")?;
        let code = unsafe {
            (self.pd_store)(
                file,
                path.as_ptr(),
                ordinates.as_mut_ptr(),
                ordinates.len() as c_int,
                values.as_mut_ptr(),
                values.len() as c_int,
                ordinates.len() as c_int,
                curves,
                unit_i.as_ptr(),
                type_i.as_ptr(),
                unit_d.as_ptr(),
                type_d.as_ptr(),
                std::ptr::null(),
                0,
                zone.as_ptr(),
            )
        };
        if code != 0 {
            return Err(status_error("hec_dss_pdStore", code));
        }
        Ok(())
    }
}
