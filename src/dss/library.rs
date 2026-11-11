use std::ffi::{c_char, c_int, c_void};
use std::path::Path;

use libloading::Library;

use crate::error::HmsError;

pub(crate) type OpenEx = unsafe extern "C" fn(*const c_char, *mut *mut c_void, c_int) -> c_int;
pub(crate) type OpenFn = unsafe extern "C" fn(*const c_char, *mut *mut c_void) -> c_int;
pub(crate) type SetValue = unsafe extern "C" fn(*const c_char, c_int) -> c_int;
pub(crate) type CloseFn = unsafe extern "C" fn(*mut c_void) -> c_int;
pub(crate) type FileVersion = unsafe extern "C" fn(*const c_char) -> c_int;
pub(crate) type RecordCount = unsafe extern "C" fn(*mut c_void) -> c_int;
pub(crate) type CatalogFn = unsafe extern "C" fn(
    *mut c_void,
    *mut c_char,
    *mut c_int,
    *const c_char,
    c_int,
    c_int,
) -> c_int;
pub(crate) type TsSizes = unsafe extern "C" fn(
    *mut c_void,
    *const c_char,
    *const c_char,
    *const c_char,
    *const c_char,
    *const c_char,
    *mut c_int,
    *mut c_int,
) -> c_int;
pub(crate) type TsRetrieve = unsafe extern "C" fn(
    *mut c_void,
    *const c_char,
    *const c_char,
    *const c_char,
    *const c_char,
    *const c_char,
    *mut c_int,
    *mut f64,
    c_int,
    *mut c_int,
    *mut c_int,
    c_int,
    *mut c_int,
    *mut c_int,
    *mut c_char,
    c_int,
    *mut c_char,
    c_int,
    *mut c_char,
    c_int,
) -> c_int;

pub(crate) struct DssApi {
    _lib: Library,
    pub(crate) open_ex: Option<OpenEx>,
    pub(crate) open_any: Option<OpenFn>,
    pub(crate) set_value: Option<SetValue>,
    pub(crate) close: CloseFn,
    pub(crate) file_version: FileVersion,
    pub(crate) record_count: RecordCount,
    pub(crate) catalog: CatalogFn,
    pub(crate) ts_sizes: TsSizes,
    pub(crate) ts_retrieve: TsRetrieve,
    pub(crate) ts_store: super::ffi_store::TsStoreRegular,
    pub(crate) ts_irregular: super::ffi_store::TsStoreIrregular,
    pub(crate) delete: super::ffi_store::DeleteFn,
    pub(crate) pd_info: super::ffi_paired::PdInfo,
    pub(crate) pd_retrieve: super::ffi_paired::PdRetrieve,
    pub(crate) pd_store: super::ffi_paired::PdStore,
    pub(crate) grid_retrieve: super::ffi_grid::GridRetrieve,
    pub(crate) grid_store: super::ffi_grid::GridStore,
}

impl DssApi {
    pub(crate) fn load(path: &Path) -> Result<Self, HmsError> {
        let lib = unsafe { Library::new(path) }.map_err(|err| {
            HmsError::message(format!("no se pudo cargar {}: {err}", path.display()))
        })?;
        let api = Self {
            open_ex: load_optional(&lib, b"hec_dss_open_ex"),
            open_any: load_optional(&lib, b"hec_dss_open"),
            set_value: load_optional(&lib, b"hec_dss_set_value"),
            close: load_symbol(&lib, b"hec_dss_close")?,
            file_version: load_symbol(&lib, b"hec_dss_getFileVersion")?,
            record_count: load_symbol(&lib, b"hec_dss_record_count")?,
            catalog: load_symbol(&lib, b"hec_dss_catalog")?,
            ts_sizes: load_symbol(&lib, b"hec_dss_tsGetSizes")?,
            ts_retrieve: load_symbol(&lib, b"hec_dss_tsRetrieve")?,
            ts_store: load_symbol(&lib, b"hec_dss_tsStoreRegular")?,
            ts_irregular: load_symbol(&lib, b"hec_dss_tsStoreIregular")?,
            delete: load_symbol(&lib, b"hec_dss_delete")?,
            pd_info: load_symbol(&lib, b"hec_dss_pdRetrieveInfo")?,
            pd_retrieve: load_symbol(&lib, b"hec_dss_pdRetrieve")?,
            pd_store: load_symbol(&lib, b"hec_dss_pdStore")?,
            grid_retrieve: load_symbol(&lib, b"hec_dss_gridRetrieve")?,
            grid_store: load_symbol(&lib, b"hec_dss_gridStore")?,
            _lib: lib,
        };
        if api.open_ex.is_none() && api.open_any.is_none() {
            return Err(HmsError::message("hecdss.dll no exporta hec_dss_open"));
        }
        api.silence();
        Ok(api)
    }

    fn silence(&self) {
        let Some(set_value) = self.set_value else {
            return;
        };
        let Ok(name) = std::ffi::CString::new("mlvl") else {
            return;
        };
        unsafe { set_value(name.as_ptr(), 0) };
    }
}

fn load_symbol<T: Copy>(lib: &Library, name: &[u8]) -> Result<T, HmsError> {
    load_optional(lib, name).ok_or_else(|| HmsError::message(format!("símbolo DSS ausente: {}", label(name))))
}

fn load_optional<T: Copy>(lib: &Library, name: &[u8]) -> Option<T> {
    let symbol: libloading::Symbol<T> = unsafe { lib.get(name) }.ok()?;
    Some(*symbol)
}

fn label(name: &[u8]) -> String {
    String::from_utf8_lossy(name).into_owned()
}
