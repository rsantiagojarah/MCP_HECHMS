use std::path::Path;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::time::timeout;

use super::classpath::{self, JavaLaunch};
use super::protocol::{WorkerReply, WorkerRequest, decode, encode};
use crate::error::HmsError;

pub struct JavaWorker {
    _child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl JavaWorker {
    pub async fn spawn(spec: &JavaLaunch) -> Result<Self, HmsError> {
        let mut child = classpath::command(spec).spawn().map_err(down)?;
        let stdin = child.stdin.take().ok_or_else(|| down("java sin stdin"))?;
        let stdout = child.stdout.take().ok_or_else(|| down("java sin stdout"))?;
        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(drain_stderr(stderr));
        }
        let mut worker = Self {
            _child: child,
            stdin,
            stdout: BufReader::new(stdout),
        };
        let reply = worker.request(&WorkerRequest::Ping, Duration::from_secs(180)).await?;
        if !reply.ok {
            return Err(HmsError::message(reply.error));
        }
        Ok(worker)
    }

    pub async fn request(&mut self, request: &WorkerRequest, limit: Duration) -> Result<WorkerReply, HmsError> {
        let line = encode(request)? + "\n";
        self.stdin.write_all(line.as_bytes()).await.map_err(down)?;
        self.stdin.flush().await.map_err(down)?;
        let reply = timeout(limit, self.read_reply())
            .await
            .map_err(|_| down("el worker Java no respondió a tiempo"))?
            .map_err(down)?;
        if !reply.ok {
            return Err(HmsError::message(if reply.error.is_empty() {
                "el worker rechazó el comando".into()
            } else {
                reply.error.clone()
            }));
        }
        Ok(reply)
    }

    async fn read_reply(&mut self) -> Result<WorkerReply, HmsError> {
        let mut line = String::new();
        loop {
            line.clear();
            let n = self.stdout.read_line(&mut line).await?;
            if n == 0 {
                return Err(HmsError::message("el worker Java cerró la salida"));
            }
            let trimmed = line.trim();
            if trimmed.starts_with('{') {
                return decode(trimmed);
            }
            tracing::debug!(target: "hms.java", "{trimmed}");
        }
    }
}

async fn drain_stderr(stderr: tokio::process::ChildStderr) {
    let mut lines = BufReader::new(stderr).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        tracing::info!(target: "hms.java", "{line}");
    }
}

pub async fn open_project(worker: &mut JavaWorker, project: &Path) -> Result<(), HmsError> {
    worker
        .request(
            &WorkerRequest::Open {
                project: project.display().to_string(),
            },
            Duration::from_secs(180),
        )
        .await?;
    Ok(())
}

fn down(err: impl ToString) -> HmsError {
    HmsError::WorkerDown(err.to_string())
}
