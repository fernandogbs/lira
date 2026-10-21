//! SQLite store: transcripts + agent session registry.

use std::path::{Path, PathBuf};

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranscriptRow {
    pub id: i64,
    pub created_at: String,
    pub text: String,
    pub model: String,
    pub language: String,
    pub audio_ms: i64,
    pub latency_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionRow {
    pub id: String,
    pub title: String,
    pub adapter: String,
    pub cwd: Option<String>,
    pub last_used_at: String,
}

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS transcripts (
                id INTEGER PRIMARY KEY,
                created_at TEXT NOT NULL,
                text TEXT NOT NULL,
                model TEXT NOT NULL,
                language TEXT NOT NULL,
                audio_ms INTEGER NOT NULL,
                latency_ms INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                adapter TEXT NOT NULL,
                cwd TEXT,
                last_used_at TEXT NOT NULL
            );",
        )?;
        Ok(Self { conn })
    }

    pub fn open_default() -> Result<Self, StoreError> {
        let path = default_store_path().ok_or_else(|| {
            StoreError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no data directory",
            ))
        })?;
        Self::open(&path)
    }

    pub fn insert_transcript(
        &self,
        text: &str,
        model: &str,
        language: &str,
        audio_ms: i64,
        latency_ms: i64,
    ) -> Result<i64, StoreError> {
        let created_at = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO transcripts (created_at, text, model, language, audio_ms, latency_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![created_at, text, model, language, audio_ms, latency_ms],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn recent_transcripts(&self, limit: usize) -> Result<Vec<TranscriptRow>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, created_at, text, model, language, audio_ms, latency_ms
             FROM transcripts ORDER BY id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(TranscriptRow {
                id: row.get(0)?,
                created_at: row.get(1)?,
                text: row.get(2)?,
                model: row.get(3)?,
                language: row.get(4)?,
                audio_ms: row.get(5)?,
                latency_ms: row.get(6)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn upsert_session(&self, session: &SessionRow) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO sessions (id, title, adapter, cwd, last_used_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
               title=excluded.title,
               adapter=excluded.adapter,
               cwd=excluded.cwd,
               last_used_at=excluded.last_used_at",
            params![
                session.id,
                session.title,
                session.adapter,
                session.cwd,
                session.last_used_at
            ],
        )?;
        Ok(())
    }

    pub fn list_sessions(&self) -> Result<Vec<SessionRow>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, adapter, cwd, last_used_at
             FROM sessions ORDER BY last_used_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(SessionRow {
                id: row.get(0)?,
                title: row.get(1)?,
                adapter: row.get(2)?,
                cwd: row.get(3)?,
                last_used_at: row.get(4)?,
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn touch_session(&self, id: &str) -> Result<(), StoreError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE sessions SET last_used_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        Ok(())
    }
}

pub fn default_store_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("lira").join("lira.sqlite3"))
}

/// Legacy history path used by M1 (`history.sqlite3`).
pub fn legacy_history_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("lira").join("history.sqlite3"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn session_round_trip() {
        let dir = tempdir().unwrap();
        let store = Store::open(&dir.path().join("t.sqlite3")).unwrap();
        store
            .upsert_session(&SessionRow {
                id: "s1".into(),
                title: "Backend".into(),
                adapter: "claude-code".into(),
                cwd: Some("/tmp/proj".into()),
                last_used_at: "2026-01-01T00:00:00Z".into(),
            })
            .unwrap();
        let list = store.list_sessions().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].title, "Backend");
    }
}
