//! CTF event management engine.
//!
//! Manages the full lifecycle of CTF competitions:
//! - **Event lifecycle**: `init` → `schedule` → `use` → `finish` → `archive`
//! - **Challenge workflow**: `add`/`import` → `solve` (flag, commit, compress, archive)
//! - **Context tracking**: Detects event root from CWD or global state
//!
//! ## Submodules
//! - [`event`] — Event creation, listing, scheduling, finishing
//! - [`challenge`] — Challenge add/solve/status/writeup
//! - [`import`] — Smart archive import with category detection
//! - [`archive`] — Event archival and zip creation
//! - [`resolve`] — Fuzzy path resolution for events and challenges

mod archive;
mod challenge;
pub mod completions;
mod event;
mod import;
mod resolve;
mod shelve;

use anyhow::Result;
use chrono::prelude::*;
use fs_err as fs;
use serde::{Deserialize, Serialize};
use std::path::Path;

// Re-export all public items from submodules
pub use archive::archive_event;
pub use challenge::{add_challenge, challenge_status, generate_writeup, solve_challenge};
pub use event::{
    check_active_expiry, check_expiries, create_event, find_event_root, finish_event,
    get_active_event_root, get_context_info, list_events, schedule_event, set_active_event,
    CreateEventResult, CtfEventInfo, ListEventsResult,
};
pub use import::import_challenge;
pub use resolve::get_event_path;
pub use shelve::shelve_challenge;

/// CTF event metadata stored in .ctf_meta.json
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CtfMeta {
    pub name: String,
    pub date: String,
    pub year: i32,
    pub created_at: i64,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_time: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_time: Option<i64>,
}

impl CtfMeta {
    pub fn new(
        name: &str,
        date: Option<String>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Self {
        let now = Local::now();
        let (year, date_str) = if let Some(d) = date {
            let y = d
                .split('-')
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(now.year());
            (y, d)
        } else {
            (now.year(), now.format("%Y-%m-%d").to_string())
        };

        Self {
            name: name.to_string(),
            date: date_str,
            year,
            created_at: now.timestamp(),
            categories: Vec::new(),
            start_time,
            end_time,
        }
    }

    /// Load metadata from a CTF event directory.
    /// Returns `Ok(None)` if the metadata file doesn't exist,
    /// `Err` if the file exists but can't be read or parsed.
    pub fn load(event_dir: &Path) -> Result<Option<Self>> {
        let meta_path = event_dir.join(".ctf_meta.json");
        if meta_path.exists() {
            let content = fs::read_to_string(&meta_path)
                .map_err(|e| anyhow::anyhow!("Failed to read {:?}: {}", meta_path, e))?;
            let meta = serde_json::from_str(&content)
                .map_err(|e| anyhow::anyhow!("Failed to parse {:?}: {}", meta_path, e))?;
            Ok(Some(meta))
        } else {
            Ok(None)
        }
    }

    /// Save metadata to a CTF event directory
    pub fn save(&self, event_dir: &Path) -> Result<()> {
        let meta_path = event_dir.join(".ctf_meta.json");
        let content = serde_json::to_string_pretty(self)?;
        fs::write(meta_path, content)?;
        Ok(())
    }
}

/// Schema version for .challenge.json — increment when fields change.
pub const CHALLENGE_SCHEMA_VERSION: u32 = 1;

/// Challenge completion status.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ChallengeStatus {
    Active,
    Solved,
    TeamSolved,
    Unsolved,
}

impl std::fmt::Display for ChallengeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Solved => write!(f, "solved"),
            Self::TeamSolved => write!(f, "team-solved"),
            Self::Unsolved => write!(f, "unsolved"),
        }
    }
}

/// Per-challenge metadata stored in .challenge.json
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChallengeMetadata {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub name: String,
    pub category: String,
    pub status: ChallengeStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub solved_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub imported_from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shelved_at: Option<String>,
    pub created_at: String,
}

fn default_schema_version() -> u32 {
    CHALLENGE_SCHEMA_VERSION
}

impl ChallengeMetadata {
    /// Create new metadata for a freshly created challenge.
    pub fn new(name: &str, category: &str) -> Self {
        Self {
            schema_version: CHALLENGE_SCHEMA_VERSION,
            name: name.to_string(),
            category: category.to_string(),
            status: ChallengeStatus::Active,
            flag: None,
            solved_by: None,
            note: None,
            imported_from: None,
            shelved_at: None,
            created_at: Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
        }
    }

    /// Load metadata from a challenge directory.
    /// Returns `Ok(None)` if no metadata file exists.
    pub fn load(challenge_dir: &Path) -> Result<Option<Self>> {
        let meta_path = challenge_dir.join(".challenge.json");
        if meta_path.exists() {
            let content = fs::read_to_string(&meta_path)?;
            let meta: Self = serde_json::from_str(&content)?;
            if meta.schema_version > CHALLENGE_SCHEMA_VERSION {
                log::warn!(
                    "Challenge metadata has schema version {} (this binary understands up to {}). Some fields may be ignored.",
                    meta.schema_version, CHALLENGE_SCHEMA_VERSION
                );
            }
            Ok(Some(meta))
        } else {
            Ok(None)
        }
    }

    /// Try loading .challenge.json; if absent but flag.txt exists, migrate.
    pub fn load_or_migrate(challenge_dir: &Path) -> Result<Option<Self>> {
        if let Some(meta) = Self::load(challenge_dir)? {
            return Ok(Some(meta));
        }

        // Migration: flag.txt → .challenge.json
        let flag_path = challenge_dir.join("flag.txt");
        if flag_path.exists() {
            let flag = fs::read_to_string(&flag_path)?.trim().to_string();
            let name = challenge_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            let category = challenge_dir
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("misc")
                .to_string();

            let mut meta = Self::new(&name, &category);
            meta.status = ChallengeStatus::Solved;
            meta.flag = Some(flag);
            meta.save(challenge_dir)?;
            log::info!("Migrated flag.txt → .challenge.json for {}", name);
            return Ok(Some(meta));
        }

        Ok(None)
    }

    /// Save metadata to the challenge directory.
    pub fn save(&self, challenge_dir: &Path) -> Result<()> {
        let meta_path = challenge_dir.join(".challenge.json");
        let content = serde_json::to_string_pretty(self)?;
        fs::write(meta_path, content)?;
        Ok(())
    }
}

fn add_solve_script(challenge_dir: &Path, category: &str) -> Result<()> {
    let template = match category {
        "pwn" => crate::core::templates::SOLVE_PY_PWN,
        "web" => crate::core::templates::SOLVE_PY_WEB,
        _ => crate::core::templates::SOLVE_PY_GENERIC,
    };

    fs::write(challenge_dir.join("solve.py"), template)?;
    println!("Created solve.py template");
    Ok(())
}
