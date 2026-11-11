use std::path::Path;

mod analyses;
mod basin;
mod create;
mod create_util;
mod blocks;
mod cache;
mod control;
mod discover;
mod edit;
mod encoding;
mod hms_file;
mod met;
mod runs;
mod scs;
mod stamp;
mod types;

use blocks::Block;
use cache::BlockCache;
use types::{BasinInfo, MetInfo, ProjectInfo};
use crate::error::HmsError;

pub use basin::from_blocks as basin_from;
pub use scs::{ScsInput, ScsMethod};
pub use blocks::parse;

#[derive(Default)]
pub struct ProjectStore {
    cache: BlockCache,
}

impl ProjectStore {
    pub fn info(&self, path: &Path) -> Result<ProjectInfo, HmsError> {
        let blocks = self.cache.load(path)?;
        let mut info = hms_file::from_blocks(&blocks);
        info.controls = self.attached_controls(path, &blocks)?;
        info.meteorology = self.attached_met(path, &blocks)?;
        if is_hms(path) {
            runs::attach(&self.cache, path, &mut info)?;
            analyses::attach(&self.cache, path, &mut info)?;
        }
        Ok(info)
    }

    pub fn set_parameter(
        &self,
        path: &Path,
        basin: Option<&str>,
        element: &str,
        parameter: &str,
        value: &str,
    ) -> Result<edit::ParameterChange, HmsError> {
        let file = self.basin_path(path, basin)?;
        let text = encoding::read_text(&file)?;
        let (updated, change) = edit::replace_parameter(&text, element, parameter, value)?;
        encoding::write_text(&file, &updated)?;
        self.cache.forget(&file);
        Ok(change)
    }

    pub fn basin(&self, path: &Path, name: Option<&str>) -> Result<BasinInfo, HmsError> {
        let basin_path = self.basin_path(path, name)?;
        let blocks = self.cache.load(&basin_path)?;
        Ok(basin::from_blocks(&blocks))
    }

    fn attached_met(&self, path: &Path, blocks: &[Block]) -> Result<Vec<MetInfo>, HmsError> {
        let mut found = Vec::new();
        for component in hms_file::from_blocks(blocks).components {
            if !is_met_kind(&component.kind) {
                continue;
            }
            let Some(filename) = component.filename else {
                continue;
            };
            let file = discover::sibling(path, &filename);
            if file.is_file() {
                if let Some(info) = met::from_blocks(&self.cache.load(&file)?) {
                    found.push(info);
                }
            }
        }
        Ok(found)
    }

    pub fn meteorology(&self, path: &Path) -> Result<Option<MetInfo>, HmsError> {
        if !is_hms(path) {
            return Ok(met::from_blocks(&self.cache.load(path)?));
        }
        let blocks = self.cache.load(path)?;
        for component in hms_file::from_blocks(&blocks).components {
            if !is_met_kind(&component.kind) {
                continue;
            }
            let Some(filename) = component.filename else {
                continue;
            };
            let file = discover::sibling(path, &filename);
            if file.is_file() {
                return Ok(met::from_blocks(&self.cache.load(&file)?));
            }
        }
        Ok(None)
    }

    fn basin_path(&self, path: &Path, name: Option<&str>) -> Result<std::path::PathBuf, HmsError> {
        if !is_hms(path) {
            return Ok(path.to_path_buf());
        }
        let blocks = self.cache.load(path)?;
        let component = hms_file::from_blocks(&blocks)
            .components
            .into_iter()
            .find(|item| item.kind == "Basin" && name.is_none_or(|wanted| item.name == wanted))
            .ok_or_else(|| HmsError::message("la cuenca no está en el proyecto"))?;
        let filename = component
            .filename
            .ok_or_else(|| HmsError::message("la cuenca no tiene archivo"))?;
        Ok(discover::sibling(path, &filename))
    }

    fn attached_controls(&self, path: &Path, blocks: &[Block]) -> Result<Vec<types::ControlSpec>, HmsError> {
        let mut specs = Vec::new();
        for component in hms_file::from_blocks(blocks).components {
            if component.kind != "Control" && component.kind != "Control Specification" {
                continue;
            }
            let Some(filename) = component.filename else {
                continue;
            };
            let file = discover::sibling(path, &filename);
            if file.is_file() {
                if let Some(spec) = control::from_blocks(&self.cache.load(&file)?) {
                    specs.push(spec);
                }
            }
        }
        Ok(specs)
    }
}

fn is_hms(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("hms")
}

fn is_met_kind(kind: &str) -> bool {
    matches!(kind, "Meteorology" | "Meteorologic" | "Met" | "Precipitation")
}
