use std::path::Path;

use super::blocks::Block;
use super::cache::BlockCache;
use super::types::{Component, Field, ProjectInfo};
use crate::error::HmsError;

const EXTENSIONS: &[&str] = &["forecast", "optim", "mca", "fac", "ens", "nals"];
const FOLDERS: &[&str] = &["optimizer", "forecast", "montecarlo", "uncertainty", "frequency", "ensemble"];
const KINDS: &[&str] = &[
    "Optimization Trial",
    "Optimization Analysis",
    "Forecast Alternative",
    "Monte Carlo Analysis",
    "Frequency Analysis",
    "Ensemble Analysis",
    "Depth-Area Analysis Manager",
    "Analysis",
];

pub fn attach(cache: &BlockCache, project: &Path, info: &mut ProjectInfo) -> Result<(), HmsError> {
    let Some(dir) = project.parent() else {
        return Ok(());
    };
    let mut files = files_in(dir)?;
    for folder in FOLDERS {
        let child = dir.join(folder);
        if child.is_dir() {
            files.extend(files_in(&child)?);
        }
    }
    for path in files {
        for block in cache.load(&path)? {
            if !KINDS.iter().any(|kind| block.key.eq_ignore_ascii_case(kind)) {
                continue;
            }
            let filename = path.file_name().and_then(|name| name.to_str()).unwrap_or("");
            let listed = info.components.iter().any(|item| {
                item.filename.as_deref() == Some(filename) || (item.kind == block.key && item.name == block.value)
            });
            if listed {
                continue;
            }
            info.components.push(component(&block, &path));
        }
    }
    Ok(())
}

fn files_in(dir: &Path) -> Result<Vec<std::path::PathBuf>, HmsError> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
        if path.is_file() && EXTENSIONS.iter().any(|item| extension.eq_ignore_ascii_case(item)) {
            files.push(path);
        }
    }
    Ok(files)
}

fn component(block: &Block, path: &Path) -> Component {
    Component {
        kind: block.key.clone(),
        name: block.value.clone(),
        filename: path.file_name().map(|name| name.to_string_lossy().into_owned()),
        fields: block
            .fields()
            .map(|(name, value)| Field {
                name: name.to_string(),
                value: value.to_string(),
            })
            .collect(),
    }
}
