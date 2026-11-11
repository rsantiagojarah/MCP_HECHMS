use std::io;

/// Error único del servidor. Los módulos nuevos lo reutilizan.
#[derive(Debug, thiserror::Error)]
pub enum HmsError {
    #[error("{0}")]
    Message(String),
    #[error("worker: {0}")]
    WorkerDown(String),
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
}

impl HmsError {
    pub fn message(text: impl Into<String>) -> Self {
        Self::Message(text.into())
    }

    pub fn is_worker_down(&self) -> bool {
        matches!(self, Self::WorkerDown(_))
    }
}
