mod archive;
mod challenge;
mod event;
mod import;

use anyhow::Result;
use chrono::prelude::*;
use fs_err as fs;
use serde::{Deserialize, Serialize};
use std::path::Path;

// Re-export all public items from submodules
pub use archive::archive_event;
pub use resolve::get_event_path;
pub use challenge::{add_challenge, challenge_status, generate_writeup, solve_challenge};
pub use event::{
    check_active_expiry, check_expiries, create_event, find_event_root, finish_event,
    get_active_event_root, get_context_info, list_events, schedule_event, set_active_event,
    CreateEventResult, CtfEventInfo, ListEventsResult,
};
pub use import::import_challenge;

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
