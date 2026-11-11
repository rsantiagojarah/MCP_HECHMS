use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum WorkerRequest {
    Ping,
    Open { project: String },
    Compute { run: String, kind: String },
    Close,
    #[allow(dead_code)]
    Shutdown,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct WorkerReply {
    pub ok: bool,
    pub cmd: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub elapsed_ms: u64,
    #[serde(default)]
    pub log_tail: String,
}

pub fn encode(request: &WorkerRequest) -> Result<String, crate::error::HmsError> {
    Ok(serde_json::to_string(request)?)
}

pub fn decode(line: &str) -> Result<WorkerReply, crate::error::HmsError> {
    Ok(serde_json::from_str(line.trim())?)
}
