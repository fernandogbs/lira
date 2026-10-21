//! Claude Code adapter via headless `claude -p` / `--resume`.

use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;

use async_trait::async_trait;
use lira_adapter_api::{AdapterEvent, AdapterStatus, AgentAdapter, EventStream, SessionInfo};
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

/// Runs Claude Code in print mode. Optional `claude_bin` override for tests.
pub struct ClaudeCodeAdapter {
    claude_bin: PathBuf,
    events_tx: mpsc::UnboundedSender<AdapterEvent>,
    events_rx: std::sync::Mutex<Option<mpsc::UnboundedReceiver<AdapterEvent>>>,
    /// Extra discovery roots (tests). Default: `~/.claude/projects`.
    projects_dir: Option<PathBuf>,
}

impl ClaudeCodeAdapter {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            claude_bin: PathBuf::from("claude"),
            events_tx: tx,
            events_rx: std::sync::Mutex::new(Some(rx)),
            projects_dir: None,
        }
    }

    pub fn with_bin(bin: PathBuf) -> Self {
        let mut a = Self::new();
        a.claude_bin = bin;
        a
    }

    fn push(&self, event: AdapterEvent) {
        let _ = self.events_tx.send(event);
    }

    fn discover_sessions(&self) -> Vec<SessionInfo> {
        let root = self
            .projects_dir
            .clone()
            .or_else(|| dirs::home_dir().map(|h| h.join(".claude").join("projects")));
        let Some(root) = root else {
            return Vec::new();
        };
        if !root.is_dir() {
            return Vec::new();
        }
        let mut out = Vec::new();
        let Ok(entries) = std::fs::read_dir(&root) else {
            return out;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let title = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("session")
                .to_string();
            // Prefer newest *.jsonl session file name as id when present.
            let mut session_id = title.clone();
            if let Ok(files) = std::fs::read_dir(&path) {
                let mut newest: Option<(std::time::SystemTime, String)> = None;
                for f in files.flatten() {
                    let fp = f.path();
                    if fp.extension().and_then(|e| e.to_str()) != Some("jsonl") {
                        continue;
                    }
                    let id = fp
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string();
                    if id.is_empty() {
                        continue;
                    }
                    let modified = f
                        .metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                    if newest.as_ref().map(|(t, _)| modified > *t).unwrap_or(true) {
                        newest = Some((modified, id));
                    }
                }
                if let Some((_, id)) = newest {
                    session_id = id;
                }
            }
            out.push(SessionInfo {
                id: session_id,
                title,
            });
        }
        out.sort_by(|a, b| a.title.cmp(&b.title));
        out
    }

    async fn run_claude(&self, session: &str, msg: &str) -> Result<String, String> {
        let mut cmd = Command::new(&self.claude_bin);
        cmd.arg("-p")
            .arg(msg)
            .arg("--output-format")
            .arg("text")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Resume when the id looks like a Claude session UUID / non-placeholder.
        if session != "new" && !session.is_empty() {
            cmd.arg("--resume").arg(session);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| format!("failed to spawn claude: {e}"))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "claude exited {}: {}",
                output.status.code().unwrap_or(-1),
                err.trim()
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

impl Default for ClaudeCodeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentAdapter for ClaudeCodeAdapter {
    async fn list_sessions(&self) -> Result<Vec<SessionInfo>, String> {
        let mut sessions = self.discover_sessions();
        if sessions.is_empty() {
            sessions.push(SessionInfo {
                id: "new".into(),
                title: "New Claude session".into(),
            });
        }
        Ok(sessions)
    }

    async fn send(&self, session: &str, msg: &str) -> Result<(), String> {
        let body = self.run_claude(session, msg).await?;
        self.push(AdapterEvent::Message {
            session: session.to_string(),
            body,
        });
        Ok(())
    }

    async fn status(&self) -> Result<AdapterStatus, String> {
        let which = Command::new("sh")
            .arg("-c")
            .arg(format!("command -v {}", self.claude_bin.display()))
            .output()
            .await;
        match which {
            Ok(out) if out.status.success() => Ok(AdapterStatus {
                ready: true,
                detail: String::from_utf8_lossy(&out.stdout).trim().to_string(),
            }),
            _ => Ok(AdapterStatus {
                ready: false,
                detail: format!("{} not found on PATH", self.claude_bin.display()),
            }),
        }
    }

    async fn summarize(&self, session: &str) -> Result<String, String> {
        let prompt = "Summarize what changed in this session briefly.";
        self.run_claude(session, prompt).await
    }

    fn events(&self) -> EventStream {
        let rx = self
            .events_rx
            .lock()
            .unwrap()
            .take()
            .expect("events stream already taken");
        Box::pin(UnboundedReceiverStream::new(rx))
    }
}

/// Shared helper for wiring.
pub type SharedClaude = Arc<ClaudeCodeAdapter>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[tokio::test]
    async fn mock_bin_send_emits_event() {
        let dir = tempfile::tempdir().unwrap();
        let bin = dir.path().join("fake-claude");
        std::fs::write(&bin, "#!/bin/sh\necho \"hello from mock\"\n").unwrap();
        let mut perms = std::fs::metadata(&bin).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&bin, perms).unwrap();

        let adapter = ClaudeCodeAdapter::with_bin(bin);
        let mut stream = adapter.events();
        use futures_util::StreamExt;
        adapter.send("new", "hi").await.unwrap();
        let ev = tokio::time::timeout(std::time::Duration::from_secs(2), stream.next())
            .await
            .unwrap()
            .unwrap();
        match ev {
            AdapterEvent::Message { body, .. } => assert!(body.contains("hello from mock")),
            other => panic!("unexpected {other:?}"),
        }
    }
}
