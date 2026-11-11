use super::blocks::Block;
use super::types::{Component, Field, ProjectInfo};

pub fn from_blocks(blocks: &[Block]) -> ProjectInfo {
    let project = blocks.iter().find(|block| block.key == "Project");
    let mut info = ProjectInfo {
        name: project.map(|block| block.value.clone()).unwrap_or_default(),
        description: field(project, "Description"),
        version: field(project, "Version"),
        dss_file: field(project, "DSS File Name"),
        time_zone: field(project, "Time Zone ID"),
        components: Vec::new(),
        controls: Vec::new(),
        meteorology: Vec::new(),
    };
    for block in blocks.iter().filter(|block| block.key != "Project") {
        info.components.push(component(block));
    }
    info
}

fn component(block: &Block) -> Component {
    let filename = block.field("Filename").map(str::to_string);
    let fields = block
        .fields()
        .filter(|(name, _)| !name.eq_ignore_ascii_case("filename") && !name.eq_ignore_ascii_case("file name"))
        .map(|(name, value)| Field {
            name: name.to_string(),
            value: value.to_string(),
        })
        .collect();
    Component {
        kind: block.key.clone(),
        name: block.value.clone(),
        filename,
        fields,
    }
}

fn field(block: Option<&Block>, name: &str) -> String {
    block
        .and_then(|item| item.field(name))
        .unwrap_or("")
        .to_string()
}
