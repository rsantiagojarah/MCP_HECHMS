use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::ffi_open::OpenFile;
use super::library::DssApi;
use super::summary::{SeriesSummary, summarize};
use super::time::{d_part_window, interval_seconds};
use super::window::SeriesWindow;
use crate::config::Config;
use crate::error::HmsError;

const MISSING_DLL: &str = "hecdss.dll no está en HEC-HMS 4.14. Defina HECDSS_DLL con la DLL de HEC-DSS 7 (7-JA o posterior) o cópiela en la carpeta bin de HMS";

struct Pool {
    api: DssApi,
    files: HashMap<PathBuf, OpenFile>,
}

pub struct DssPool {
    dll: Option<PathBuf>,
    inner: Mutex<Option<Pool>>,
}

impl DssPool {
    pub fn new(config: &Config) -> Self {
        Self {
            dll: config.hecdss_dll.clone(),
            inner: Mutex::new(None),
        }
    }

    pub fn catalog(&self, path: &Path, filter: &str, limit: usize) -> Result<CatalogPage, HmsError> {
        self.with_file(path, |api, file| {
            let paths = api.catalog(file, filter)?;
            let total = paths.len();
            let pathnames = paths.into_iter().take(limit).collect::<Vec<_>>();
            Ok(CatalogPage {
                total,
                truncated: total > pathnames.len(),
                pathnames,
            })
        })
    }

    pub fn summary(&self, path: &Path, pathname: &str, window: &TimeWindow) -> Result<SeriesSummary, HmsError> {
        self.with_file(path, |api, file| {
            let got = api.retrieve(file, pathname, &window.start_date, &window.start_time, &window.end_date, &window.end_time)?;
            Ok(summarize(
                pathname,
                &got.samples,
                &got.units,
                &got.data_type,
                series_interval(pathname, &got.samples),
                got.julian_base,
            ))
        })
    }

    pub fn window(&self, path: &Path, pathname: &str, window: &TimeWindow, max_points: usize) -> Result<SeriesWindow, HmsError> {
        self.with_file(path, |api, file| {
            let got = api.retrieve(file, pathname, &window.start_date, &window.start_time, &window.end_date, &window.end_time)?;
            Ok(super::window::build(pathname, &got, got.julian_base, max_points))
        })
    }

    pub fn release_all(&self) {
        let mut guard = self.inner.lock().unwrap_or_else(|err| err.into_inner());
        if let Some(pool) = guard.as_mut() {
            let files = std::mem::take(&mut pool.files);
            for (_, file) in files {
                pool.api.close_file(file);
            }
        }
    }

    pub(crate) fn with_file<T>(
        &self,
        path: &Path,
        body: impl FnOnce(&DssApi, *mut std::ffi::c_void) -> Result<T, HmsError>,
    ) -> Result<T, HmsError> {
        self.access(path, false, body)
    }

    pub(crate) fn with_writable<T>(
        &self,
        path: &Path,
        body: impl FnOnce(&DssApi, *mut std::ffi::c_void) -> Result<T, HmsError>,
    ) -> Result<T, HmsError> {
        self.access(path, true, body)
    }

    fn access<T>(
        &self,
        path: &Path,
        create: bool,
        body: impl FnOnce(&DssApi, *mut std::ffi::c_void) -> Result<T, HmsError>,
    ) -> Result<T, HmsError> {
        let mut guard = self.inner.lock().unwrap_or_else(|err| err.into_inner());
        if guard.is_none() {
            let dll = self.dll.clone().ok_or_else(|| HmsError::message(MISSING_DLL))?;
            *guard = Some(Pool {
                api: DssApi::load(&dll)?,
                files: HashMap::new(),
            });
        }
        let pool = guard.as_mut().expect("pool");
        let key = path.to_path_buf();
        if !pool.files.contains_key(&key) {
            let opened = if create {
                pool.api.open_writable(&path.display().to_string())?
            } else {
                pool.api.open_readonly(&path.display().to_string())?
            };
            pool.files.insert(key.clone(), opened);
        }
        let ptr = pool.files.get(&key).expect("abierto").as_ptr();
        body(&pool.api, ptr)
    }
}

#[derive(Debug, Clone)]
pub struct TimeWindow {
    pub start_date: String,
    pub start_time: String,
    pub end_date: String,
    pub end_time: String,
}

impl TimeWindow {
    pub fn resolve(pathname: &str, start_date: &str, start_time: &str, end_date: &str, end_time: &str) -> Self {
        if start_date.is_empty() && end_date.is_empty() {
            let (start_date, start_time, end_date, end_time) = d_part_window(pathname);
            return Self { start_date, start_time, end_date, end_time };
        }
        Self {
            start_date: start_date.to_string(),
            start_time: if start_time.is_empty() { "0000".into() } else { start_time.into() },
            end_date: end_date.to_string(),
            end_time: if end_time.is_empty() { "2400".into() } else { end_time.into() },
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CatalogPage {
    pub total: usize,
    pub truncated: bool,
    pub pathnames: Vec<String>,
}

fn series_interval(pathname: &str, samples: &[super::time::Sample]) -> i64 {
    let parts = super::time::pathname_parts(pathname);
    if let Some(seconds) = parts.get(4).and_then(|part| interval_seconds(part)) {
        return seconds;
    }
    if samples.len() >= 2 {
        return samples[1].offset_seconds - samples[0].offset_seconds;
    }
    0
}

pub type SharedDss = Arc<DssPool>;
