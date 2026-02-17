use crate::app::action::{Action, OutputStream};
use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Default)]
pub struct VpmClient;

impl VpmClient {
    pub async fn run_command(
        &self,
        task_id: u64,
        _label: String,
        program: &str,
        args: Vec<String>,
        token: CancellationToken,
        action_tx: mpsc::UnboundedSender<Action>,
    ) -> Result<()> {
        let mut command = Command::new(program);
        command
            .args(args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        let mut child = command
            .spawn()
            .with_context(|| format!("failed to spawn {program}"))?;

        let mut readers: Vec<JoinHandle<()>> = Vec::new();

        if let Some(stdout) = child.stdout.take() {
            readers.push(tokio::spawn(pipe_lines(
                task_id,
                OutputStream::Stdout,
                stdout,
                action_tx.clone(),
            )));
        }

        if let Some(stderr) = child.stderr.take() {
            readers.push(tokio::spawn(pipe_lines(
                task_id,
                OutputStream::Stderr,
                stderr,
                action_tx.clone(),
            )));
        }

        let (success, cancelled, exit_code, error) = tokio::select! {
            _ = token.cancelled() => {
                let _ = child.kill().await;
                match child.wait().await {
                    Ok(status) => (false, true, status.code(), None),
                    Err(err) => (false, true, None, Some(err.to_string())),
                }
            }
            status = child.wait() => {
                match status {
                    Ok(status) => (status.success(), false, status.code(), None),
                    Err(err) => (false, false, None, Some(err.to_string())),
                }
            }
        };

        for reader in readers {
            let _ = reader.await;
        }

        let _ = action_tx.send(Action::TaskDone {
            task_id,
            success,
            cancelled,
            exit_code,
            error,
        });

        Ok(())
    }
}

async fn pipe_lines(
    task_id: u64,
    stream: OutputStream,
    reader: impl tokio::io::AsyncRead + Unpin,
    action_tx: mpsc::UnboundedSender<Action>,
) {
    let mut lines = BufReader::new(reader).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        let _ = action_tx.send(Action::TaskOutput {
            task_id,
            stream,
            line,
        });
    }
}
