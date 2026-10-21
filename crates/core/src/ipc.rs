use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

use crate::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    Toggle,
    Start,
    Stop,
    Status,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub command: Command,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub ok: bool,
    pub state: AppState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl Response {
    pub fn ok(state: AppState) -> Self {
        Self {
            ok: true,
            state,
            message: None,
        }
    }

    pub fn err(state: AppState, message: impl Into<String>) -> Self {
        Self {
            ok: false,
            state,
            message: Some(message.into()),
        }
    }
}

/// Socket path: `$LIRA_SOCKET` if set (tests, a second instance), else
/// `$XDG_RUNTIME_DIR/lira.sock`, else the temp dir.
pub fn socket_path() -> PathBuf {
    if let Some(custom) = std::env::var_os("LIRA_SOCKET").filter(|v| !v.is_empty()) {
        return PathBuf::from(custom);
    }
    if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(runtime).join("lira.sock");
    }
    std::env::temp_dir().join("lira.sock")
}

#[derive(Debug, Error)]
pub enum IpcError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Client: one connection, write request line, read response line.
pub async fn client_request(path: &Path, command: Command) -> Result<Response, IpcError> {
    let mut stream = UnixStream::connect(path).await?;
    let req = Request { command };
    let line = serde_json::to_string(&req)?;
    stream.write_all(line.as_bytes()).await?;
    stream.write_all(b"\n").await?;
    stream.flush().await?;

    let mut reader = BufReader::new(stream);
    let mut response_line = String::new();
    reader.read_line(&mut response_line).await?;
    let resp: Response = serde_json::from_str(response_line.trim())?;
    Ok(resp)
}

pub type CommandHandler = std::sync::Arc<dyn Fn(Command) -> Response + Send + Sync>;

pub struct IpcServer {
    listener: UnixListener,
    path: PathBuf,
}

impl IpcServer {
    pub async fn bind(path: impl AsRef<Path>) -> Result<Self, IpcError> {
        let path = path.as_ref().to_path_buf();
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let listener = UnixListener::bind(&path)?;
        Ok(Self { listener, path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub async fn accept_one(&self, handler: &CommandHandler) -> Result<(), IpcError> {
        let (stream, _) = self.listener.accept().await?;
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        reader.read_line(&mut line).await?;
        let req: Request = serde_json::from_str(line.trim())?;
        let resp = handler(req.command);
        let out = serde_json::to_string(&resp)? + "\n";
        writer.write_all(out.as_bytes()).await?;
        writer.shutdown().await?;
        Ok(())
    }

    pub async fn run(self, handler: CommandHandler) -> Result<(), IpcError> {
        loop {
            // A bad client must not take the server down: log and keep accepting.
            if let Err(err) = self.accept_one(&handler).await {
                eprintln!("lira ipc: request failed: {err}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tempfile::tempdir;

    #[tokio::test]
    async fn round_trip_status() {
        let dir = tempdir().unwrap();
        let sock = dir.path().join("lira.sock");
        let state = Arc::new(Mutex::new(AppState::Idle));
        let handler: CommandHandler = Arc::new({
            let state = Arc::clone(&state);
            move |cmd| {
                let mut s = state.lock().unwrap();
                match cmd {
                    Command::Toggle => *s = s.toggle(),
                    Command::Start => *s = AppState::Listening,
                    Command::Stop => *s = AppState::Idle,
                    Command::Status => {}
                }
                Response::ok(*s)
            }
        });

        let server = IpcServer::bind(&sock).await.unwrap();
        let server_task = tokio::spawn({
            let handler = Arc::clone(&handler);
            async move {
                server.accept_one(&handler).await.unwrap();
            }
        });

        let resp = client_request(&sock, Command::Status).await.unwrap();
        assert!(resp.ok);
        assert_eq!(resp.state, AppState::Idle);

        server_task.await.unwrap();
    }
}
