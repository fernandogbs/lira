use std::path::{Path, PathBuf};

use chrono::Utc;
use rusqlite::{params, Connection};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HistoryError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranscriptRow {
    pub id: i64,
    pub created_at: String,
    pub text: String,
    pub model: String,
    pub language: String,
    pub audio_ms: i64,
    pub latency_ms: i64,
}

pub struct History {
    conn: Connection,
}

impl History {
    pub fn open(path: &Path) -> Result<Self, HistoryError> {
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
            );",
        )?;
        Ok(Self { conn })
    }

    pub fn open_default() -> Result<Self, HistoryError> {
        let path = default_history_path().ok_or_else(|| {
            HistoryError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no data directory",
            ))
        })?;
        Self::open(&path)
    }

    pub fn insert(
        &self,
        text: &str,
        model: &str,
        language: &str,
        audio_ms: i64,
        latency_ms: i64,
    ) -> Result<i64, HistoryError> {
        let created_at = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO transcripts (created_at, text, model, language, audio_ms, latency_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![created_at, text, model, language, audio_ms, latency_ms],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn recent(&self, limit: usize) -> Result<Vec<TranscriptRow>, HistoryError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, created_at, text, model, language, audio_ms, latency_ms
             FROM transcripts
             ORDER BY id DESC
             LIMIT ?1",
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
}

pub fn default_history_path() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("lira").join("history.sqlite3"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn insert_and_recent() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("history.sqlite3");
        let history = History::open(&path).unwrap();
        let id = history
            .insert("hello", "large-v3-turbo-q5_0", "pt", 1200, 340)
            .unwrap();
        assert_eq!(id, 1);

        let rows = history.recent(10).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].text, "hello");
        assert_eq!(rows[0].model, "large-v3-turbo-q5_0");
        assert_eq!(rows[0].language, "pt");
        assert_eq!(rows[0].audio_ms, 1200);
        assert_eq!(rows[0].latency_ms, 340);
        assert!(!rows[0].created_at.is_empty());
    }
}
