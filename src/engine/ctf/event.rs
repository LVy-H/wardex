use crate::config::Config;
use anyhow::{Context, Result};

use fs_err as fs;
use std::path::{Path, PathBuf};

use super::CtfMeta;

/// Result of creating a CTF event
#[derive(Debug)]
pub struct CreateEventResult {
    pub event_dir: PathBuf,
    pub categories_created: Vec<String>,
    pub already_exists: bool,
}

/// Result of listing CTF events
#[derive(Debug, Clone)]
pub struct CtfEventInfo {
    pub name: String,
    pub year: i32,
    pub date: Option<String>,
    pub challenge_count: usize,
    pub path: PathBuf,
    pub has_metadata: bool,
}

#[derive(Debug, Default)]
pub struct ListEventsResult {
    pub events: Vec<CtfEventInfo>,
    pub ctf_root_missing: bool,
}

pub fn create_event(
    config: &Config,
    name: &str,
    date: Option<String>,
    start_time: Option<i64>,
    end_time: Option<i64>,
) -> Result<CreateEventResult> {
    let ctf_root = config.ctf_root();

    if !ctf_root.exists() {
        fs::create_dir_all(&ctf_root).context("Failed to create CTF root directory")?;
    }

    let meta = CtfMeta::new(name, date.clone(), start_time, end_time);
    let folder_name = format!("{}_{}", meta.date.split('-').next().unwrap_or("0000"), name);
    let event_dir = ctf_root.join(&folder_name);

    if event_dir.exists() {
        return Ok(CreateEventResult {
            event_dir,
            categories_created: Vec::new(),
            already_exists: true,
        });
    }

    fs::create_dir(&event_dir).context("Failed to create event directory")?;

    // Create category directories
    let mut categories_created = Vec::new();
    for cat in &config.ctf.default_categories {
        fs::create_dir(event_dir.join(cat)).context("Failed to create category")?;
        categories_created.push(cat.clone());
    }

    // Create notes.md
    fs::File::create(event_dir.join("notes.md")).context("Failed to create notes.md")?;

    // Save metadata
    let mut meta = meta;
    meta.categories = categories_created.clone();
    meta.save(&event_dir)?;

    // Auto-set active event
    if let Err(e) = set_active_event(config, name) {
        println!("Warning: Failed to set active event: {}", e);
    }

    Ok(CreateEventResult {
        event_dir,
        categories_created,
        already_exists: false,
    })
}

pub fn list_events(config: &Config) -> Result<ListEventsResult> {
    let ctf_root = config.ctf_root();

    if !ctf_root.exists() {
        return Ok(ListEventsResult {
            events: Vec::new(),
            ctf_root_missing: true,
        });
    }

    let mut events = Vec::new();
    let entries = fs::read_dir(&ctf_root)?;

    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }

        let path = entry.path();
        let dir_name = entry.file_name().to_string_lossy().to_string();

        // Try to load metadata first
        if let Some(meta) = CtfMeta::load(&path) {
            let challenge_count = count_challenges(&path);
            events.push(CtfEventInfo {
                name: meta.name,
                year: meta.year,
                date: Some(meta.date),
                challenge_count,
                path,
                has_metadata: true,
            });
        } else {
            // Fallback: parse from folder name
            let year = if dir_name.len() >= 4 && dir_name[..4].chars().all(char::is_numeric) {
                dir_name[..4].parse().unwrap_or(0)
            } else {
                0
            };

            // Handle year-only directories (recurse into them)
            if dir_name.len() == 4 && dir_name.chars().all(char::is_numeric) {
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    for sub in sub_entries.flatten() {
                        if !sub.path().is_dir() {
                            continue;
                        }
                        let sub_path = sub.path();
                        let sub_name = sub.file_name().to_string_lossy().to_string();
                        let challenge_count = count_challenges(&sub_path);

                        // Check for metadata in subdirectory
                        let (name, date, has_meta) = if let Some(meta) = CtfMeta::load(&sub_path) {
                            (meta.name, Some(meta.date), true)
                        } else {
                            (sub_name, None, false)
                        };

                        events.push(CtfEventInfo {
                            name,
                            year,
                            date,
                            challenge_count,
                            path: sub_path,
                            has_metadata: has_meta,
                        });
                    }
                }
            } else {
                let challenge_count = count_challenges(&path);
                events.push(CtfEventInfo {
                    name: dir_name,
                    year,
                    date: None,
                    challenge_count,
                    path,
                    has_metadata: false,
                });
            }
        }
    }

    events.sort_by(|a, b| b.year.cmp(&a.year).then_with(|| a.name.cmp(&b.name)));

    Ok(ListEventsResult {
        events,
        ctf_root_missing: false,
    })
}

pub(crate) fn count_challenges(event_dir: &Path) -> usize {
    let mut count = 0;
    if let Ok(cats) = fs::read_dir(event_dir) {
        for cat in cats.flatten() {
            if cat.path().is_dir() && cat.file_name() != ".git" {
                if let Ok(chals) = fs::read_dir(cat.path()) {
                    count += chals.flatten().filter(|c| c.path().is_dir()).count();
                }
            }
        }
    }
    count
}

/// Walk up directory tree to find CTF event root (containing .ctf_meta.json)
pub fn find_event_root() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    loop {
        if current.join(".ctf_meta.json").exists() {
            return Some(current);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

/// Get the active event root from local context or global state
pub fn get_active_event_root() -> Result<PathBuf> {
    // 1. Try local context (walking up)
    if let Some(root) = find_event_root() {
        return Ok(root);
    }
    // 2. Try global state
    let state = crate::core::state::AppState::load();
    if let Some(path) = state.get_event() {
        if path.exists() && path.join(".ctf_meta.json").exists() {
            return Ok(path);
        }
    }
    anyhow::bail!(
        "No active CTF event found.\nRun inside an event dir or use 'wardex ctf use <event>'"
    )
}

pub fn set_active_event(config: &Config, name: &str) -> Result<()> {
    use super::get_event_path;
    let path = get_event_path(config, Some(name), None)?;
    let mut state = crate::core::state::AppState::load();
    state.set_event(path.clone())?;
    println!("Switched to event: {}", name);
    println!("Context set to: {:?}", path);
    Ok(())
}

/// Get info about current CTF context
pub fn get_context_info(_config: &Config) -> Result<()> {
    let root = match get_active_event_root() {
        Ok(r) => r,
        Err(_) => {
            println!("No active CTF event context detected.");
            println!(
                "Run this command inside a CTF event directory or use 'wardex ctf use <event>'."
            );
            return Ok(());
        }
    };

    let meta = CtfMeta::load(&root).ok_or_else(|| anyhow::anyhow!("Failed to load metadata"))?;

    // Check if we are physically inside the root
    let current = std::env::current_dir()?;
    let is_local = current.starts_with(&root);

    println!("Current Event: {} ({})", meta.name, meta.year);
    println!("Root: {:?}", root);
    println!(
        "Source: {}",
        if is_local {
            "Local Directory"
        } else {
            "Global State"
        }
    );

    if is_local {
        if let Ok(rel) = current.strip_prefix(&root) {
            if rel.components().count() > 0 {
                println!("Location: ./{}", rel.display());
            } else {
                println!("Location: Event Root");
            }
        }
    }
    Ok(())
}

pub fn schedule_event(
    config: &Config,
    name: Option<&str>,
    start_time: Option<i64>,
    end_time: Option<i64>,
) -> Result<()> {
    use super::get_event_path;
    let event_path = get_event_path(config, name, None)?;
    let mut meta = CtfMeta::load(&event_path)
        .ok_or_else(|| anyhow::anyhow!("Failed to load metadata for event"))?;

    if let Some(st) = start_time {
        meta.start_time = Some(st);
    }
    if let Some(et) = end_time {
        meta.end_time = Some(et);
    }

    meta.save(&event_path)?;
    println!("✓ Updated schedule for '{}'", meta.name);
    Ok(())
}

pub fn finish_event(
    config: &Config,
    name: Option<&str>,
    no_archive: bool,
    force: bool,
    dry_run: bool,
) -> Result<()> {
    use super::get_event_path;
    use dialoguer::{theme::ColorfulTheme, MultiSelect};
    use std::process::Command;

    let event_path = get_event_path(config, name, None)?;
    let mut meta = CtfMeta::load(&event_path)
        .ok_or_else(|| anyhow::anyhow!("Failed to load metadata for event"))?;

    println!("Finishing event: {}", meta.name);

    // 1. Check if git repo exists
    let is_git = Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .current_dir(&event_path)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // 2. Find ignored files (cleanup candidates)
    if is_git {
        let mut out_str = String::new();
        match Command::new("git")
            .arg("clean")
            .arg("-dXn")
            .current_dir(&event_path)
            .output()
        {
            Ok(output) => {
                out_str = String::from_utf8_lossy(&output.stdout).to_string();
            }
            Err(e) => {
                println!("! Failed to execute git clean (missing git?): {}", e);
            }
        }
        let mut candidates = Vec::new();
        for line in out_str.lines() {
            if let Some(path) = line.strip_prefix("Would remove ") {
                candidates.push(path.to_string());
            }
        }

        if !candidates.is_empty() {
            let to_delete = if dry_run {
                println!("(Dry Run) Would remove:");
                for c in &candidates {
                    println!("  - {}", c);
                }
                Vec::new()
            } else if force {
                candidates.clone()
            } else {
                // Interactive prompt
                let defaults = vec![true; candidates.len()]; // Select all by default
                let selections = MultiSelect::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select items to delete (Space to toggle, Enter to confirm)")
                    .items(&candidates)
                    .defaults(&defaults)
                    .interact()?;

                selections.into_iter().map(|i| candidates[i].clone()).collect()
            };

            if !dry_run && !to_delete.is_empty() {
                println!("Cleaning up {} items...", to_delete.len());
                for item in to_delete {
                    let full_path = event_path.join(item);
                    if full_path.is_dir() {
                        let _ = fs::remove_dir_all(full_path);
                    } else {
                        let _ = fs::remove_file(full_path);
                    }
                }
                println!("✓ Cleanup complete.");
            }
        } else {
            println!("✓ Workspace is already clean.");
        }
    } else {
        println!("(Not a git repository, skipping cleanup based on gitignore)");
    }

    if dry_run {
        if is_git {
            println!("(Dry Run) Would commit all changes and archive event.");
        } else {
            println!("(Dry Run) Would archive event.");
        }
        return Ok(());
    }

    // 3. Commit all changes
    if is_git {
        println!("Committing final state...");
        let _ = Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(&event_path)
            .status();

        let commit_status = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(format!("Finished CTF: {}", meta.name))
            .current_dir(&event_path)
            .output();

        if let Ok(st) = commit_status {
            if st.status.success() {
                println!("✓ Changes committed.");
            } else {
                println!("- Git commit returned non-zero (nothing to commit?).");
            }
        } else {
            println!("! Failed to execute git commit command.");
        }
    }

    // 3. Mark end time if not marked
    if meta.end_time.is_none() {
        meta.end_time = Some(chrono::Local::now().timestamp());
        let _ = meta.save(&event_path);
    }

    // 4. Archive (if not no_archive)
    if !no_archive {
        super::archive_event(config, &meta.name)?;
    }

    println!("✓ Successfully finished event '{}'", meta.name);
    Ok(())
}

pub fn check_expiries(config: &Config) -> Result<()> {
    use chrono::{Local, TimeZone};
    let now = Local::now().timestamp();
    let grace_sec = (config.ctf.grace_period_hours as i64) * 3600;

    let events = list_events(config)?;
    let mut expired = Vec::new();
    let mut active = Vec::new();

    for e in events.events {
        if !e.has_metadata { continue; }
        if let Some(meta) = CtfMeta::load(&e.path) {
            if let Some(et) = meta.end_time {
                if now > et + grace_sec {
                    expired.push(meta.clone());
                } else if now >= meta.start_time.unwrap_or(0) && now <= et {
                    active.push(meta.clone());
                }
            } else if now >= meta.start_time.unwrap_or(0) {
                active.push(meta.clone());
            }
        }
    }

    if expired.is_empty() && active.is_empty() {
        println!("No active or expired events found.");
        return Ok(());
    }

    if !active.is_empty() {
        println!("=== Active Events ===");
        for m in active {
            if let Some(et) = m.end_time {
                if let chrono::LocalResult::Single(dt) = Local.timestamp_opt(et, 0) {
                    println!("- {} (Ends at {})", m.name, dt.format("%Y-%m-%d %H:%M"));
                }
            } else {
                println!("- {} (No end time set)", m.name);
            }
        }
        println!();
    }

    if !expired.is_empty() {
        println!("=== Expired Events (Past Grace Period) ===");
        for m in expired {
            println!("- {} (Consider running `wardex ctf finish {}`)", m.name, m.name);
        }
    }

    Ok(())
}

pub fn check_active_expiry(config: &Config) {
    use chrono::Local;
    if let Ok(path) = get_active_event_root() {
        if let Some(meta) = CtfMeta::load(&path) {
            if let Some(et) = meta.end_time {
                let now = Local::now().timestamp();
                let grace = (config.ctf.grace_period_hours as i64) * 3600;
                if now > et + grace {
                    log::warn!(
                        "Passive Reminder: Active event '{}' expired! Consider running `wardex ctf finish`",
                        meta.name
                    );
                } else if now > et {
                    log::info!(
                        "Passive Reminder: Active event '{}' ended. Grace period active.",
                        meta.name
                    );
                }
            }
        }
    }
}
