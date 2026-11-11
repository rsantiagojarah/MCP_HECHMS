use super::blocks::Block;
use super::types::ControlSpec;

pub fn from_blocks(blocks: &[Block]) -> Option<ControlSpec> {
    let header = blocks
        .iter()
        .find(|block| block.key == "Control" || block.key == "Control Specification")?;
    Some(ControlSpec {
        name: header.value.clone(),
        start_date: value(header, "Start Date"),
        start_time: value(header, "Start Time"),
        end_date: value(header, "End Date"),
        end_time: value(header, "End Time"),
        interval: value(header, "Time Interval"),
    })
}

fn value(block: &Block, name: &str) -> String {
    block.field(name).unwrap_or("").to_string()
}
