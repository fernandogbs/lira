use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionInfo {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdapterStatus {
    pub ready: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum AdapterEvent {
    Message { session: String, body: String },
    StatusChanged(AdapterStatus),
}

pub type EventStream = Pin<Box<dyn futures_core::Stream<Item = AdapterEvent> + Send>>;

#[async_trait]
pub trait AgentAdapter: Send + Sync {
    async fn list_sessions(&self) -> Result<Vec<SessionInfo>, String>;
    async fn send(&self, session: &str, msg: &str) -> Result<(), String>;
    async fn status(&self) -> Result<AdapterStatus, String>;
    async fn summarize(&self, session: &str) -> Result<String, String>;
    fn events(&self) -> EventStream;
}

/// Minimal fake adapter for contract tests and adapter authors.
pub struct FakeAdapter {
    events_tx: mpsc::UnboundedSender<AdapterEvent>,
    events_rx: std::sync::Mutex<Option<mpsc::UnboundedReceiver<AdapterEvent>>>,
}

impl Default for FakeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeAdapter {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            events_tx: tx,
            events_rx: std::sync::Mutex::new(Some(rx)),
        }
    }

    pub fn push_event(&self, event: AdapterEvent) {
        let _ = self.events_tx.send(event);
    }
}

#[async_trait]
impl AgentAdapter for FakeAdapter {
    async fn list_sessions(&self) -> Result<Vec<SessionInfo>, String> {
        Ok(vec![SessionInfo {
            id: "fake-1".into(),
            title: "Fake session".into(),
        }])
    }

    async fn send(&self, session: &str, msg: &str) -> Result<(), String> {
        self.push_event(AdapterEvent::Message {
            session: session.to_string(),
            body: msg.to_string(),
        });
        Ok(())
    }

    async fn status(&self) -> Result<AdapterStatus, String> {
        Ok(AdapterStatus {
            ready: true,
            detail: "fake".into(),
        })
    }

    async fn summarize(&self, session: &str) -> Result<String, String> {
        Ok(format!("summary of {}", session))
    }

    fn events(&self) -> EventStream {
        let rx = self
            .events_rx
            .lock()
            .unwrap()
            .take()
            .expect("events stream already taken");
        Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx))
    }
}

pub async fn run_adapter_contract(adapter: Arc<dyn AgentAdapter>) -> Result<(), String> {
    let sessions = adapter.list_sessions().await?;
    if sessions.is_empty() {
        return Err("expected at least one session".into());
    }
    let id = &sessions[0].id;
    adapter.send(id, "hello").await?;
    let status = adapter.status().await?;
    if !status.ready {
        return Err("adapter not ready".into());
    }
    let _summary = adapter.summarize(id).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fake_adapter_contract() {
        let adapter = Arc::new(FakeAdapter::new()) as Arc<dyn AgentAdapter>;
        run_adapter_contract(adapter).await.unwrap();
    }
}
