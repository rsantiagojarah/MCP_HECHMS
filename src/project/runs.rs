use std::path::Path;

use super::blocks::Block;
use super::cache::BlockCache;
use super::types::{Component, Field, ProjectInfo};
use crate::error::HmsError;

pub fn attach(cache: &BlockCache, project: &Path, info: &mut ProjectInfo) -> Result<(), HmsError> {
    let Some(dir) = project.parent() else {
        return Ok(());
    };
    let mut files = Vec::new();
    if let Some(stem) = project.file_stem() {
        let named = dir.join(stem).with_extension("run");
        if named.is_file() {
            files.push(named);
        }
    }
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("run") && !files.contains(&path) {
            files.push(path);
        }
    }
    for path in files {
        for block in cache.load(&path)? {
            if block.key != "Run" || info.components.iter().any(|item| item.kind == "Run" && item.name == block.value) {
                continue;
            }
            info.components.push(from_block(&block));
        }
    }
    Ok(())
}

fn from_block(block: &Block) -> Component {
    Component {
        kind: block.key.clone(),
        name: block.value.clone(),
        filename: block.field("DSS File").map(str::to_string),
        fields: block
            .fields()
            .filter(|(name, _)| !name.eq_ignore_ascii_case("dss file"))
            .map(|(name, value)| Field {
                name: name.to_string(),
                value: value.to_string(),
            })
            .collect(),
    }
}
