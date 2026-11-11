use std::path::{Path, PathBuf};

use super::encoding;
use crate::error::HmsError;

pub(super) fn crlf(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\n', "\r\n")
}

pub(super) fn filled(value: String, fallback: &str) -> String {
    if value.is_empty() { fallback.into() } else { value }
}

pub(super) fn require_hms(path: &Path) -> Result<(), HmsError> {
    if path.extension().and_then(|ext| ext.to_str()) == Some("hms") {
        Ok(())
    } else {
        Err(HmsError::message("hace falta la ruta del archivo .hms"))
    }
}

pub(super) fn clean(name: &str) -> Result<&str, HmsError> {
    let name = name.trim();
    if name.is_empty() || name.contains(['/', '\\', ':', '\n', '\r']) {
        Err(HmsError::message("el nombre está vacío o tiene caracteres no válidos"))
    } else {
        Ok(name)
    }
}

pub(super) fn slug(name: &str) -> String {
    name.split_whitespace().collect::<Vec<_>>().join("_")
}

pub(super) fn gage_file(project: &Path) -> Result<PathBuf, HmsError> {
    let stem = project.file_stem().and_then(|item| item.to_str()).unwrap_or("project");
    let file = project.parent().unwrap_or(Path::new(".")).join(format!("{stem}.gage"));
    if file.is_file() { Ok(file) } else { Err(HmsError::message("el proyecto no tiene archivo de pluviómetros")) }
}

pub(super) fn element_kind(kind: &str) -> Result<&'static str, HmsError> {
    match kind.trim().to_ascii_lowercase().as_str() {
        "subbasin" | "subcuenca" => Ok("Subbasin"),
        "reach" | "tramo" => Ok("Reach"),
        "junction" | "union" | "unión" => Ok("Junction"),
        "reservoir" | "embalse" => Ok("Reservoir"),
        "sink" | "sumidero" => Ok("Sink"),
        "source" | "fuente" => Ok("Source"),
        "diversion" | "derivacion" | "derivación" => Ok("Diversion"),
        _ => Err(HmsError::message("elemento desconocido. Use Subbasin, Reach, Junction, Reservoir, Sink, Source o Diversion")),
    }
}

pub(super) fn element_block(kind: &str, name: &str, downstream: &str, area: &str, date: &str, time: &str) -> String {
    let down = if downstream.trim().is_empty() { String::new() } else { format!("     Downstream: {downstream}\n") };
    let body = match kind {
        "Subbasin" => format!("     Area: {area}\n{down}     Discretization: None\n     Canopy: None\n     Surface: None\n     LossRate: None\n     Transform: None\n     Baseflow: None\n"),
        "Diversion" => format!("{down}     Diverter: Constant\n     Constant Flow: 0\n     End Diverter:\n"),
        "Reach" => format!("{down}     Route: None\n"),
        _ => down,
    };
    format!("\n{kind}: {name}\n     Last Modified Date: {date}\n     Last Modified Time: {time}\n{body}End:\n")
}

pub(super) fn insert_element(text: &str, block: &str) -> String {
    if let Some(index) = text.find("Basin Layer Properties:") {
        let mut updated = text[..index].to_string();
        if !updated.ends_with('\n') { updated.push('\n'); }
        updated.push_str(block);
        updated.push_str(&text[index..]);
        updated
    } else {
        let mut updated = text.to_string();
        if !updated.ends_with('\n') { updated.push('\n'); }
        updated.push_str(block);
        updated
    }
}

pub(super) fn append_unique(file: &Path, key: &str, name: &str, block: &str) -> Result<(), HmsError> {
    let existing = if file.is_file() { encoding::read_text(file)? } else { String::new() };
    if existing.lines().any(|line| line.trim().eq_ignore_ascii_case(&format!("{key}: {name}"))) {
        return Err(HmsError::message(format!("ya existe {key} {name}")));
    }
    let mut text = existing;
    if !text.is_empty() && !text.ends_with('\n') { text.push('\n'); }
    text.push_str(block);
    encoding::write_text(file, &crlf(&text))
}

pub(super) fn analysis_spec(kind: &str) -> Result<(&'static str, &'static str, &'static str, &'static str, &'static str), HmsError> {
    match kind.trim().to_ascii_lowercase().as_str() {
        "forecast" | "pronostico" | "pronóstico" => Ok(("Forecast Alternative", "Forecast Alternative", "forecast", "End Forecast Alternative:", "forecast")),
        "optimization" | "optimizacion" | "optimización" | "trial" => Ok(("Optimization Trial", "Optimization Analysis", "optim", "End Optimization Analysis:", "optimizer")),
        "uncertainty" | "incertidumbre" | "montecarlo" => Ok(("Monte Carlo Analysis", "Monte Carlo Analysis", "mca", "End Monte Carlo Analysis:", "montecarlo")),
        "frequency" | "frecuencia" => Ok(("Frequency Analysis", "Frequency Analysis", "fac", "End Frequency Analysis:", "frequency")),
        "ensemble" | "ensamble" => Ok(("Ensemble Analysis", "Ensemble Analysis", "ens", "End Ensemble Analysis:", "ensemble")),
        "depth-area" | "deptharea" | "profundidad-area" | "profundidad-área" => Ok(("Depth Area", "Analysis", "nals", "End:", "")),
        _ => Err(HmsError::message("análisis desconocido. Use forecast, optimization, uncertainty, frequency, ensemble o depth-area")),
    }
}

pub(super) fn analysis_text(
    extension: &str,
    header: &str,
    closer: &str,
    name: &str,
    basin: &str,
    precip: &str,
    date: &str,
    time: &str,
    window: &(String, String, String, String, String),
    zone: &str,
) -> String {
    let models = if extension == "fac" {
        String::new()
    } else {
        format!("     Basin Model Name: {basin}\n     Met Model Name: {precip}\n")
    };
    let stem = slug(name);
    let extra = match extension {
        "forecast" => format!("     Forecast Date: {}\n     Forecast Time: {}\n     DSS File: {stem}.dss\n     Log File: {stem}.log\n     Time-Series Output: Save All\n", window.2, window.3),
        "optim" | "mca" | "fac" | "ens" => format!("     DSS File: {stem}.dss\n     Log File: {stem}.log\n     Time-Series Output: Save All\n"),
        _ => String::new(),
    };
    let extra = match extension {
        "mca" => format!("{extra}     Number Of Realizations: 10\n     Seed Value: 1\n"),
        "fac" => format!("{extra}     Number of Ordinates: 5\n"),
        _ => extra,
    };
    let manager = if extension == "nals" {
        format!("Depth-Area Analysis Manager: {name}\n     Version: 4.14\nEnd:\n\n")
    } else {
        String::new()
    };
    format!(
        "{manager}{header}: {name}\n     Description: \n     Last Modified Date: {date}\n     Last Modified Time: {time}\n{models}     Start Date: {sd}\n     Start Time: {st}\n     End Date: {ed}\n     End Time: {et}\n     Time Interval: {iv}\n     Time Zone ID: {zone}\n     Version: 4.14\n{extra}{closer}\n",
        sd = window.0,
        st = window.1,
        ed = window.2,
        et = window.3,
        iv = window.4,
    )
}
