use std::ffi::{CStr, CString, c_int, c_void};

use super::library::DssApi;
use crate::error::HmsError;

pub(crate) struct OpenFile(*mut c_void);

unsafe impl Send for OpenFile {}

impl OpenFile {
    pub(crate) fn as_ptr(&self) -> *mut c_void {
        self.0
    }
}

impl DssApi {
    pub(crate) fn file_version_of(&self, path: &str) -> Result<i32, HmsError> {
        let text = c_string(path)?;
        Ok(unsafe { (self.file_version)(text.as_ptr()) })
    }

    pub(crate) fn open_readonly(&self, path: &str) -> Result<OpenFile, HmsError> {
        let version = self.file_version_of(path)?;
        if version != 7 {
            return Err(HmsError::message(version_message(version)));
        }
        if self.open_ex.is_some() {
            return self.open_with(path, 1);
        }
        self.open_handle(path)
    }

    pub(crate) fn open_writable(&self, path: &str) -> Result<OpenFile, HmsError> {
        let version = self.file_version_of(path)?;
        if version == 6 {
            return Err(HmsError::message(version_message(version)));
        }
        if version != 7 && version != 0 && std::path::Path::new(path).is_file() {
            return Err(HmsError::message(version_message(version)));
        }
        self.open_with(path, 0)
    }

    fn open_handle(&self, path: &str) -> Result<OpenFile, HmsError> {
        self.open_with(path, 0)
    }

    fn open_with(&self, path: &str, readonly: c_int) -> Result<OpenFile, HmsError> {
        let text = c_string(path)?;
        let mut file = std::ptr::null_mut();
        let code = unsafe {
            if readonly == 1 {
                if let Some(open_ex) = self.open_ex {
                    open_ex(text.as_ptr(), &mut file, 1)
                } else if let Some(open_any) = self.open_any {
                    open_any(text.as_ptr(), &mut file)
                } else {
                    -1
                }
            } else if let Some(open_any) = self.open_any {
                open_any(text.as_ptr(), &mut file)
            } else if let Some(open_ex) = self.open_ex {
                open_ex(text.as_ptr(), &mut file, 0)
            } else {
                -1
            }
        };
        if code != 0 || file.is_null() {
            return Err(HmsError::message(format!("hec_dss_open devolvió {code}")));
        }
        Ok(OpenFile(file))
    }

    pub(crate) fn close_file(&self, file: OpenFile) {
        unsafe {
            (self.close)(file.as_ptr());
        }
    }
}

pub(crate) fn c_string(text: &str) -> Result<CString, HmsError> {
    CString::new(text).map_err(|_| HmsError::message("la ruta contiene un byte nulo"))
}

pub(crate) fn c_text(text: &str) -> Result<CString, HmsError> {
    c_string(text)
}

pub(crate) fn read_c_buf(buf: &[u8]) -> String {
    let end = buf.iter().position(|byte| *byte == 0).unwrap_or(buf.len());
    let bytes = &buf[..end];
    CStr::from_bytes_until_nul(buf)
        .map(|text| text.to_string_lossy().into_owned())
        .unwrap_or_else(|_| String::from_utf8_lossy(bytes).into_owned())
}

pub(crate) fn status_error(name: &str, code: c_int) -> HmsError {
    HmsError::message(format!("{name} devolvió {code}"))
}

fn version_message(version: i32) -> String {
    match version {
        6 => "el archivo es DSS 6. HEC-HMS 4.14 solo admite DSS 7: ábralo en HMS para migrarlo".into(),
        0 => "el archivo DSS no existe".into(),
        _ => format!("el archivo no es DSS 7 (código {version})"),
    }
}
