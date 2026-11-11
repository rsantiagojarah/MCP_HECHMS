use std::path::{Path, PathBuf};

use super::create_util::*;
use super::edit::ParameterChange;
use super::encoding;
use super::hms_file;
use super::stamp;
use super::ProjectStore;
use crate::error::HmsError;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Created {
    pub kind: String,
    pub name: String,
    pub file: String,
}

impl ProjectStore {
    pub fn set_in(
        &self,
        path: &Path,
        target: &str,
        component: Option<&str>,
        element: &str,
        parameter: &str,
        value: &str,
    ) -> Result<ParameterChange, HmsError> {
        let file = self.target_file(path, target, component)?;
        let text = encoding::read_text(&file)?;
        let (updated, mut change) = super::edit::replace_parameter(&text, element, parameter, value)?;
        encoding::write_text(&file, &updated)?;
        self.cache.forget(&file);
        change.element = element.to_string();
        Ok(change)
    }

    pub fn create_basin(&self, project: &Path, name: &str, units: &str) -> Result<Created, HmsError> {
        let name = clean(name)?;
        let units = if units.trim().is_empty() { "Metric" } else { units.trim() };
        let (date, time) = stamp::date_time();
        let body = format!(
            "Basin: {name}\n     Last Modified Date: {date}\n     Last Modified Time: {time}\n     Version: 4.14\n     Unit System: {units}\n     Missing Flow To Zero: No\nEnd:\n"
        );
        self.write_component(project, "Basin", name, "basin", "Filename", "", &body)
    }

    pub fn create_control(
        &self,
        project: &Path,
        name: &str,
        start_date: &str,
        start_time: &str,
        end_date: &str,
        end_time: &str,
        interval: &str,
    ) -> Result<Created, HmsError> {
        let name = clean(name)?;
        let zone = self.project_zone(project);
        let (date, time) = stamp::date_time();
        let body = format!(
            "Control: {name}\n     Last Modified Date: {date}\n     Last Modified Time: {time}\n     Version: 4.14\n     Time Zone ID: {zone}\n     Start Date: {start_date}\n     Start Time: {start_time}\n     End Date: {end_date}\n     End Time: {end_time}\n     Time Interval: {interval}\nEnd:\n"
        );
        self.write_component(project, "Control", name, "control", "FileName", "", &body)
    }

    pub fn create_met(&self, project: &Path, name: &str, basin: &str) -> Result<Created, HmsError> {
        let name = clean(name)?;
        let basin = clean(basin)?;
        let (date, time) = stamp::date_time();
        let body = format!(
            "Meteorology: {name}\n     Last Modified Date: {date}\n     Last Modified Time: {time}\n     Version: 4.14\n     Unit System: Metric\n     Precipitation Method: Specified Average\n     Evapotranspiration Method: No Evapotranspiration\n     Use Basin Model: {basin}\nEnd:\n\nPrecip Method Parameters: Specified Average\n     Allow Depth Override: No\nEnd:\n"
        );
        self.write_component(project, "Precipitation", name, "met", "Filename", "", &body)
    }

    pub fn create_element(
        &self,
        project: &Path,
        basin: Option<&str>,
        kind: &str,
        name: &str,
        downstream: &str,
        area: &str,
    ) -> Result<Created, HmsError> {
        let name = clean(name)?;
        let kind = element_kind(kind)?;
        let area = if area.trim().is_empty() { "0" } else { area.trim() };
        let file = self.basin_path(project, basin)?;
        let text = encoding::read_text(&file)?;
        if text.lines().any(|line| line.trim().eq_ignore_ascii_case(&format!("{kind}: {name}"))) {
            return Err(HmsError::message(format!("ya existe {kind} {name}")));
        }
        let (date, time) = stamp::date_time();
        let block = element_block(kind, name, downstream, area, &date, &time);
        let updated = insert_element(&text, &block);
        encoding::write_text(&file, &crlf(&updated))?;
        self.cache.forget(&file);
        Ok(Created { kind: kind.to_string(), name: name.to_string(), file: file.display().to_string() })
    }

    pub fn create_run(&self, project: &Path, name: &str, basin: &str, precip: &str, control: &str) -> Result<Created, HmsError> {
        let name = clean(name)?;
        let stem = project.file_stem().and_then(|item| item.to_str()).unwrap_or("project");
        let file = project.parent().unwrap_or(Path::new(".")).join(format!("{stem}.run"));
        let (date, time) = stamp::date_time();
        let dss = format!("{}.dss", slug(name));
        let block = format!(
            "\nRun: {name}\n     Last Modified Date: {date}\n     Last Modified Time: {time}\n     Basin: {basin}\n     Precip: {precip}\n     Control: {control}\n     DSS File: {dss}\n     Log File: {name}.log\n     Time-Series Output: Save All\nEnd:\n"
        );
        append_unique(&file, "Run", name, &block)?;
        self.cache.forget(&file);
        Ok(Created { kind: "Run".into(), name: name.to_string(), file: file.display().to_string() })
    }

    pub fn create_gage(&self, project: &Path, name: &str) -> Result<Created, HmsError> {
        let name = clean(name)?;
        let stem = project.file_stem().and_then(|item| item.to_str()).unwrap_or("project");
        let file = project.parent().unwrap_or(Path::new(".")).join(format!("{stem}.gage"));
        if !file.is_file() {
            encoding::write_text(&file, &crlf(&format!("Gage Manager: {stem}\n     Version: 4.14\nEnd:\n")))?;
        }
        let (date, time) = stamp::date_time();
        let block = format!(
            "\nGage: {name}\n     Last Modified Date: {date}\n     Last Modified Time: {time}\n     Gage Type: Precipitation\n     Precipitation Type: Incremental\n     Units: MM\n     Data Type: PER-CUM\n     Data Source Type: Manual Entry\nEnd:\n"
        );
        append_unique(&file, "Gage", name, &block)?;
        self.cache.forget(&file);
        Ok(Created { kind: "Gage".into(), name: name.to_string(), file: file.display().to_string() })
    }

    pub fn create_analysis(
        &self,
        project: &Path,
        kind: &str,
        name: &str,
        basin: &str,
        precip: &str,
        control: &str,
    ) -> Result<Created, HmsError> {
        let name = clean(name)?;
        let spec = analysis_spec(kind)?;
        let (date, time) = stamp::date_time();
        let window = self.control_window(project, control);
        let body = analysis_text(spec.2, spec.1, spec.3, name, basin, precip, &date, &time, &window, &self.project_zone(project));
        self.write_component(project, spec.0, name, spec.2, "Filename", spec.4, &body)
    }
}

impl ProjectStore {
    fn write_component(&self, project: &Path, key: &str, name: &str, extension: &str, file_key: &str, folder: &str, body: &str) -> Result<Created, HmsError> {
        require_hms(project)?;
        let filename = format!("{}.{}", slug(name), extension);
        let parent = project.parent().unwrap_or(Path::new("."));
        let dir = if folder.is_empty() { parent.to_path_buf() } else { parent.join(folder) };
        std::fs::create_dir_all(&dir)?;
        let file = dir.join(&filename);
        if file.exists() {
            return Err(HmsError::message(format!("ya existe {}", file.display())));
        }
        let hms = encoding::read_text(project)?;
        if hms_file::from_blocks(&super::parse(&hms)).components.iter().any(|item| item.kind == key && item.name == name) {
            return Err(HmsError::message(format!("ya existe {key} {name}")));
        }
        encoding::write_text(&file, &crlf(body))?;
        let mut updated = hms;
        if !updated.ends_with('\n') {
            updated.push('\n');
        }
        updated.push_str(&crlf(&format!("\n{key}: {name}\n     {file_key}: {filename}\nEnd:\n")));
        encoding::write_text(project, &updated)?;
        self.cache.forget(project);
        self.cache.forget(&file);
        Ok(Created { kind: key.into(), name: name.to_string(), file: file.display().to_string() })
    }

    fn target_file(&self, path: &Path, target: &str, component: Option<&str>) -> Result<PathBuf, HmsError> {
        match target.trim().to_ascii_lowercase().as_str() {
            "" | "basin" | "cuenca" => self.basin_path(path, component),
            "control" => self.component_file(path, &["Control", "Control Specification"], component, "el control"),
            "met" | "meteorologia" | "meteorología" => self.component_file(path, &["Precipitation", "Meteorology", "Meteorologic", "Met"], component, "la meteorología"),
            "gage" | "pluviometro" | "pluviómetro" => gage_file(path),
            _ => Err(HmsError::message("destino desconocido. Use basin, control, met o gage")),
        }
    }

    fn component_file(&self, path: &Path, kinds: &[&str], name: Option<&str>, label: &str) -> Result<PathBuf, HmsError> {
        let blocks = self.cache.load(path)?;
        let component = hms_file::from_blocks(&blocks)
            .components
            .into_iter()
            .find(|item| kinds.contains(&item.kind.as_str()) && name.is_none_or(|wanted| item.name == wanted))
            .ok_or_else(|| HmsError::message(format!("{label} no está en el proyecto")))?;
        let filename = component.filename.ok_or_else(|| HmsError::message(format!("{label} no tiene archivo")))?;
        Ok(super::discover::sibling(path, &filename))
    }

    fn project_zone(&self, project: &Path) -> String {
        self.cache.load(project).ok().and_then(|blocks| {
            blocks.iter().find(|block| block.key == "Project").and_then(|block| block.field("Time Zone ID")).map(str::to_string)
        }).unwrap_or_else(|| "America/Lima".into())
    }

    fn control_window(&self, project: &Path, name: &str) -> (String, String, String, String, String) {
        let fallback = ("1 January 2000".into(), "00:00".into(), "2 January 2000".into(), "24:00".into(), "15".into());
        let Ok(file) = self.component_file(project, &["Control", "Control Specification"], Some(name).filter(|item| !item.is_empty()), "el control") else {
            return fallback;
        };
        let Ok(blocks) = self.cache.load(&file) else { return fallback };
        let Some(spec) = super::control::from_blocks(&blocks) else { return fallback };
        (filled(spec.start_date, &fallback.0), filled(spec.start_time, &fallback.1), filled(spec.end_date, &fallback.2), filled(spec.end_time, &fallback.3), filled(spec.interval, &fallback.4))
    }
}
