use std::path::Path;

use super::records::StoredSeries;
use super::session::DssPool;
use crate::error::HmsError;

impl DssPool {
    pub fn store_irregular(
        &self,
        path: &Path,
        pathname: &str,
        start_date: &str,
        times: &[i32],
        values: &[f64],
        units: &str,
        data_type: &str,
    ) -> Result<StoredSeries, HmsError> {
        let mut times = times.to_vec();
        let mut values = values.to_vec();
        self.with_writable(path, |api, file| {
            api.store_irregular(file, pathname, start_date, &mut times, &mut values, units, data_type)?;
            Ok(StoredSeries {
                pathname: pathname.to_string(),
                count: values.len(),
                units: units.to_string(),
                data_type: data_type.to_string(),
            })
        })
    }

    pub fn delete(&self, path: &Path, pathname: &str) -> Result<usize, HmsError> {
        self.with_writable(path, |api, file| {
            if api.delete_record(file, pathname).is_ok() {
                return Ok(1);
            }
            let matches = api
                .catalog(file, "/*/*/*/*/*/*/")?
                .into_iter()
                .filter(|item| same_series(item, pathname))
                .collect::<Vec<_>>();
            if matches.is_empty() {
                return Err(HmsError::message(format!("no está el registro {pathname}")));
            }
            for item in &matches {
                api.delete_record(file, item)?;
            }
            Ok(matches.len())
        })
    }
}

fn same_series(stored: &str, requested: &str) -> bool {
    let stored = super::time::pathname_parts(stored);
    let requested = super::time::pathname_parts(requested);
    let comparable = [0, 1, 2, 4, 5];
    if requested.iter().all(|part| part.is_empty()) {
        return false;
    }
    comparable.iter().all(|index| {
        requested[*index].is_empty() || stored[*index].eq_ignore_ascii_case(requested[*index])
    })
}
