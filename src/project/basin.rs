use super::blocks::Block;
use super::types::{BasinInfo, Element, Field};

const SKIP: &[&str] = &[
    "Canvas X",
    "Canvas Y",
    "Label X",
    "Label Y",
    "Last Modified Date",
    "Last Modified Time",
    "Filepath Separator",
];

/// Tipos conocidos. Un elemento nuevo entra en esta lista, o se reconoce si tiene Downstream.
const ELEMENT_KEYS: &[&str] = &[
    "Subbasin",
    "Reach",
    "Junction",
    "Reservoir",
    "Sink",
    "Source",
    "Diversion",
];

pub fn from_blocks(blocks: &[Block]) -> BasinInfo {
    let header = blocks.iter().find(|block| block.key == "Basin");
    BasinInfo {
        name: header.map(|block| block.value.clone()).unwrap_or_default(),
        unit_system: header
            .and_then(|block| block.field("Unit System"))
            .unwrap_or("")
            .to_string(),
        version: header
            .and_then(|block| block.field("Version"))
            .unwrap_or("")
            .to_string(),
        elements: blocks.iter().filter(|block| is_element(block)).map(element).collect(),
    }
}

fn is_element(block: &Block) -> bool {
    ELEMENT_KEYS.contains(&block.key.as_str()) || block.field("Downstream").is_some()
}

fn element(block: &Block) -> Element {
    let downstream = block.field("Downstream").map(str::to_string);
    let area = block
        .field("Area")
        .or_else(|| block.field("Area (KM2)"))
        .map(str::to_string);
    let parameters = block
        .fields()
        .filter(|(name, _)| !skip(name) && *name != "Downstream" && *name != "Area")
        .map(|(name, value)| Field {
            name: name.to_string(),
            value: value.to_string(),
        })
        .collect();
    Element {
        kind: block.key.clone(),
        name: block.value.clone(),
        downstream,
        area,
        parameters,
    }
}

fn skip(name: &str) -> bool {
    SKIP.contains(&name) || name.starts_with("Canvas ")
}
