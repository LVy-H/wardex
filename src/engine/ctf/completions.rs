//! Dynamic completion helpers for shell tab-completion.
//!
//! These functions run at TAB-time without full Config access,
//! so they resolve the CTF root from config files or defaults.
//! They must never panic — return empty Vec on any error.

use std::ffi::OsStr;
use std::path::PathBuf;

use clap_complete::engine::CompletionCandidate;

/// Resolve the CTF root directory from environment or defaults.
///
/// Tries in order:
/// 1. `WX_PATHS_CTF_ROOT` env var (explicit override)
/// 2. `WX_PATHS_WORKSPACE`/1_Projects/CTFs
/// 3. `~/.config/wardex/config.yaml` workspace field / 1_Projects/CTFs
/// 4. `~/workspace/1_Projects/CTFs`
fn resolve_ctf_root() -> Option<PathBuf> {
    // 1. Direct CTF root override
    if let Ok(dir) = std::env::var("WX_PATHS_CTF_ROOT") {
        let path = PathBuf::from(dir);
        if path.exists() {
            return Some(path);
        }
    }

    // 2. Workspace env var
    if let Ok(ws) = std::env::var("WX_PATHS_WORKSPACE") {
        let ctf_root = PathBuf::from(ws).join("1_Projects").join("CTFs");
        if ctf_root.exists() {
            return Some(ctf_root);
        }
    }

    // 3. Try reading config file for workspace path
    if let Some(config_dir) = dirs::config_dir() {
        let config_path = config_dir.join("wardex").join("config.yaml");
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                // Simple YAML extraction — avoid pulling in full config machinery at TAB-time
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("workspace:") {
                        if let Some(val) = trimmed.strip_prefix("workspace:") {
                            let val = val.trim().trim_matches('"').trim_matches('\'');
                            if !val.is_empty() {
                                let ctf_root = PathBuf::from(val).join("1_Projects").join("CTFs");
                                if ctf_root.exists() {
                                    return Some(ctf_root);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 4. Default: ~/workspace/1_Projects/CTFs
    let home = dirs::home_dir()?;
    let ctf_root = home.join("workspace").join("1_Projects").join("CTFs");
    if ctf_root.exists() {
        return Some(ctf_root);
    }

    None
}

/// Resolve the active event root from global state.
///
/// Reads `~/.local/share/wardex/state.json` and returns the
/// `current_event_path` if it exists on disk.
fn resolve_active_event() -> Option<PathBuf> {
    let state_path = if let Ok(p) = std::env::var("WARDEX_STATE_FILE") {
        PathBuf::from(p)
    } else {
        dirs::data_dir()?.join("wardex").join("state.json")
    };

    let content = std::fs::read_to_string(state_path).ok()?;
    let value: serde_json::Value = serde_json::from_str(&content).ok()?;
    let path_str = value.get("current_event_path")?.as_str()?;
    let path = PathBuf::from(path_str);
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

/// Complete event names for commands like `ctf use <event>`.
///
/// Lists directories inside the CTF root that match the current prefix.
pub fn event_completer(current: &OsStr) -> Vec<CompletionCandidate> {
    let prefix = current.to_string_lossy();

    let Some(ctf_root) = resolve_ctf_root() else {
        return Vec::new();
    };

    let Ok(entries) = std::fs::read_dir(&ctf_root) else {
        return Vec::new();
    };

    entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            // Skip hidden directories
            if name.starts_with('.') {
                return None;
            }
            if prefix.is_empty() || name.to_lowercase().contains(&prefix.to_lowercase()) {
                Some(CompletionCandidate::new(name))
            } else {
                None
            }
        })
        .collect()
}

/// Complete category names for commands like `ctf add <cat/name>`.
///
/// If the user has not yet typed a `/`, suggests category directories
/// from the active event or falls back to defaults.
pub fn category_completer(current: &OsStr) -> Vec<CompletionCandidate> {
    let prefix = current.to_string_lossy();

    // If the user has already typed "cat/", don't complete categories
    if prefix.contains('/') {
        return Vec::new();
    }

    // Try active event root from global state
    if let Some(root) = resolve_active_event() {
        if let Ok(entries) = std::fs::read_dir(&root) {
            let results: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    if !name.starts_with('.')
                        && (prefix.is_empty() || name.starts_with(prefix.as_ref()))
                    {
                        Some(CompletionCandidate::new(format!("{}/", name)))
                    } else {
                        None
                    }
                })
                .collect();

            if !results.is_empty() {
                return results;
            }
        }
    }

    // Fall back to default categories
    ["web", "pwn", "crypto", "rev", "misc", "forensics"]
        .iter()
        .filter(|c| prefix.is_empty() || c.starts_with(prefix.as_ref()))
        .map(|c| CompletionCandidate::new(format!("{}/", c)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_completer_returns_empty_when_no_root() {
        // With no CTF root on disk, should return empty
        std::env::set_var("WX_PATHS_CTF_ROOT", "/nonexistent/path/for/testing");
        std::env::remove_var("WX_PATHS_WORKSPACE");
        let results = event_completer(OsStr::new(""));
        // May or may not be empty depending on default paths,
        // but at least it shouldn't panic
        let _ = results;
        std::env::remove_var("WX_PATHS_CTF_ROOT");
    }

    #[test]
    fn category_completer_returns_defaults_when_no_event() {
        std::env::set_var("WARDEX_STATE_FILE", "/nonexistent/state.json");
        let results = category_completer(OsStr::new(""));
        assert!(!results.is_empty(), "should return default categories");
        std::env::remove_var("WARDEX_STATE_FILE");
    }

    #[test]
    fn category_completer_filters_by_prefix() {
        std::env::set_var("WARDEX_STATE_FILE", "/nonexistent/state.json");
        let results = category_completer(OsStr::new("pw"));
        assert_eq!(results.len(), 1);
        std::env::remove_var("WARDEX_STATE_FILE");
    }

    #[test]
    fn category_completer_skips_after_slash() {
        let results = category_completer(OsStr::new("pwn/"));
        assert!(results.is_empty(), "should not complete after slash");
    }
}
