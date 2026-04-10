use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecentEvent {
    pub path: PathBuf,
    pub name: String,
    pub accessed_at: String,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct AppState {
    pub current_event_path: Option<PathBuf>,
    pub current_challenge_path: Option<PathBuf>,
    #[serde(default)]
    pub previous_event_path: Option<PathBuf>,
    #[serde(default)]
    pub recent_events: Vec<RecentEvent>,
}

impl AppState {
    pub fn load() -> Self {
        if let Some(path) = Self::get_state_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(state) = serde_json::from_str(&content) {
                        return state;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(path) = Self::get_state_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let content = serde_json::to_string_pretty(self)?;
            fs::write(path, content)?;
        }
        Ok(())
    }

    pub fn set_event(&mut self, path: PathBuf) -> anyhow::Result<()> {
        if !path.exists() {
            anyhow::bail!("Event path does not exist: {:?}", path);
        }
        let canonical = fs::canonicalize(&path)?;

        // Save current as previous (for `ctf use -`)
        if let Some(current) = &self.current_event_path {
            if *current != canonical {
                self.previous_event_path = Some(current.clone());
            }
        }

        // Add to recent events (dedup, cap at 5)
        let name = canonical
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let now = chrono::Local::now()
            .format("%Y-%m-%dT%H:%M:%S")
            .to_string();

        self.recent_events.retain(|r| r.path != canonical);
        self.recent_events.insert(
            0,
            RecentEvent {
                path: canonical.clone(),
                name,
                accessed_at: now,
            },
        );
        self.recent_events.truncate(5);

        self.current_event_path = Some(canonical);
        self.save()
    }

    pub fn get_event(&self) -> Option<PathBuf> {
        self.current_event_path.clone().filter(|p| p.exists())
    }

    pub fn get_previous_event(&self) -> Option<PathBuf> {
        self.previous_event_path.clone().filter(|p| p.exists())
    }

    pub fn clear(&mut self) -> anyhow::Result<()> {
        self.current_event_path = None;
        self.current_challenge_path = None;
        self.save()
    }

    fn get_state_path() -> Option<PathBuf> {
        if let Ok(p) = std::env::var("WARDEX_STATE_FILE") {
            return Some(PathBuf::from(p));
        }
        dirs::data_dir().map(|d| d.join("wardex").join("state.json"))
    }
}
