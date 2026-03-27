//! Voix — Settings persistence via SQLite

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub hotkey: String,
    pub language: String,
    pub polish_level: String, // "light" | "medium" | "strong"
    pub llm_provider: String, // "ollama" | "openai"
    pub llm_model: String,
    pub whisper_model: String, // "tiny" | "base" | "small" | "medium"
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hotkey: "Option+E".to_string(),
            language: "auto".to_string(),
            polish_level: "medium".to_string(),
            llm_provider: "ollama".to_string(),
            llm_model: "qwen2.5:3b".to_string(),
            whisper_model: "small".to_string(),
        }
    }
}

pub struct SettingsStore {
    conn: Mutex<Connection>,
}

impl SettingsStore {
    pub fn new() -> Result<Self> {
        let db_path = Self::db_path()?;
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&db_path).context("Failed to open settings DB")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                original TEXT,
                polished TEXT,
                language TEXT,
                latency_ms INTEGER
            )",
            [],
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn db_path() -> Result<PathBuf> {
        let dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("voxis")
            .join("voxis.db");
        Ok(dir)
    }

    pub fn get_settings(&self) -> Result<Settings> {
        let conn = self.conn.lock().unwrap();
        let mut settings = Settings::default();

        for (key, value) in [
            ("hotkey", &mut settings.hotkey),
            ("language", &mut settings.language),
            ("polish_level", &mut settings.polish_level),
            ("llm_provider", &mut settings.llm_provider),
            ("llm_model", &mut settings.llm_model),
            ("whisper_model", &mut settings.whisper_model),
        ] {
            if let Ok(val) = conn.query_row(
                "SELECT value FROM settings WHERE key = ?",
                [key],
                |row| row.get::<_, String>(0),
            ) {
                *value = val;
            }
        }
        Ok(settings)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn add_history(&self, original: &str, polished: &str, lang: &str, latency_ms: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let timestamp = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO history (timestamp, original, polished, language, latency_ms) VALUES (?, ?, ?, ?, ?)",
            params![timestamp, original, polished, lang, latency_ms],
        )?;
        Ok(())
    }

    pub fn get_history(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, original, polished, language, latency_ms \
             FROM history ORDER BY id DESC LIMIT ?",
        )?;

        let entries = stmt
            .query_map([limit as i64], |row| {
                Ok(HistoryEntry {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    original: row.get(2)?,
                    polished: row.get(3)?,
                    language: row.get(4)?,
                    latency_ms: row.get(5)?,
                })
            })?
            .filter_map(|e| e.ok())
            .collect();

        Ok(entries)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub timestamp: String,
    pub original: String,
    pub polished: String,
    pub language: String,
    pub latency_ms: i64,
}
