use super::blocks::Block;
use super::types::{Field, MetInfo};

pub fn from_blocks(blocks: &[Block]) -> Option<MetInfo> {
    let header = blocks.iter().find(|block| {
        matches!(block.key.as_str(), "Meteorology" | "Meteorologic" | "Met")
    })?;
    let methods = header
        .fields()
        .filter(|(name, _)| !name.starts_with("Last Modified") && *name != "Filepath Separator")
        .map(|(name, value)| Field {
            name: name.to_string(),
            value: value.to_string(),
        })
        .collect();
    Some(MetInfo {
        name: header.value.clone(),
        methods,
    })
}
