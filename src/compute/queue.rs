use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;

use super::classpath;
use super::process::{JavaWorker, open_project};
use super::protocol::WorkerRequest;
use crate::config::Config;
use crate::error::HmsError;

pub struct ComputeHandle {
    config: Arc<Config>,
    slot: Mutex<Option<JavaWorker>>,
    opened: Mutex<Option<PathBuf>>,
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct ComputeOutcome {
    pub project: String,
    pub run: String,
    pub elapsed_ms: u64,
    pub log_tail: String,
}

impl ComputeHandle {
    pub fn new(config: Arc<Config>) -> Self {
        Self {
            config,
            slot: Mutex::new(None),
            opened: Mutex::new(None),
        }
    }

    pub fn warm(self: &Arc<Self>) {
        let handle = Arc::clone(self);
        tokio::spawn(async move {
            if let Err(err) = handle.ensure().await {
                tracing::warn!("worker de HEC-HMS: {err}");
            }
        });
    }

    pub async fn run(&self, project: &Path, run: &str, kind: &str, limit: Duration) -> Result<ComputeOutcome, HmsError> {
        match self.attempt(project, run, kind, limit).await {
            Err(err) if err.is_worker_down() => {
                self.reset().await;
                self.attempt(project, run, kind, limit).await
            }
            other => other,
        }
    }

    pub async fn invalidate(&self, edited: &Path) {
        let mut slot = self.slot.lock().await;
        let mut opened = self.opened.lock().await;
        let Some(current) = opened.clone() else {
            return;
        };
        let same = current.as_path() == edited || current.parent() == edited.parent();
        if !same {
            return;
        }
        if let Some(worker) = slot.as_mut() {
            let _ = worker.request(&WorkerRequest::Close, Duration::from_secs(30)).await;
        }
        *opened = None;
    }

    async fn reset(&self) {
        *self.slot.lock().await = None;
        *self.opened.lock().await = None;
    }

    async fn attempt(&self, project: &Path, run: &str, kind: &str, limit: Duration) -> Result<ComputeOutcome, HmsError> {
        self.ensure().await?;
        let mut slot = self.slot.lock().await;
        let worker = slot.as_mut().ok_or_else(|| HmsError::message("el worker no está listo"))?;
        self.ensure_open(worker, project).await?;
        let reply = worker
            .request(
                &WorkerRequest::Compute {
                    run: run.to_string(),
                    kind: kind.to_string(),
                },
                limit,
            )
            .await?;
        Ok(ComputeOutcome {
            project: project.display().to_string(),
            run: run.to_string(),
            elapsed_ms: reply.elapsed_ms,
            log_tail: reply.log_tail,
        })
    }

    async fn ensure(&self) -> Result<(), HmsError> {
        let mut slot = self.slot.lock().await;
        if slot.is_some() {
            return Ok(());
        }
        let spec = classpath::launch(&self.config)?;
        *slot = Some(JavaWorker::spawn(&spec).await?);
        Ok(())
    }

    async fn ensure_open(&self, worker: &mut JavaWorker, project: &Path) -> Result<(), HmsError> {
        let mut opened = self.opened.lock().await;
        if opened.as_deref() == Some(project) {
            return Ok(());
        }
        if opened.is_some() {
            worker.request(&WorkerRequest::Close, Duration::from_secs(60)).await?;
        }
        open_project(worker, project).await?;
        *opened = Some(project.to_path_buf());
        Ok(())
    }
}
