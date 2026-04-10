//! Shelve a challenge — Wardex's signature interactive cleanup and archival flow.
//!
//! Interactive by default: status → flag → file triage → note → archive.
//! Each step is skippable with a flag for scripting and power users.

use crate::config::Config;
use anyhow::{Context, Result};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use fs_err as fs;
use std::path::PathBuf;

use super::{ChallengeMetadata, ChallengeStatus, CtfMeta};


#[allow(clippy::too_many_arguments)]
pub fn shelve_challenge(
    config: &Config,
    flag_arg: Option<String>,
    note_arg: Option<String>,
    no_clean: bool,
    force_move: bool,
    no_move: bool,
    no_commit: bool,
    auto: bool,
) -> Result<()> {
    let current_dir = std::env::current_dir()?;

    // Load or create challenge metadata
    let mut meta = ChallengeMetadata::load_or_migrate(&current_dir)?
        .unwrap_or_else(|| {
            let name = current_dir
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            let category = current_dir
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("misc")
                .to_string();
            ChallengeMetadata::new(&name, &category)
        });

    // ── Step 1: Status ────────────────────────────────────────────────
    let (status, flag) = if let Some(f) = flag_arg {
        // Flag provided on CLI — skip status prompt
        (ChallengeStatus::Solved, Some(f))
    } else if auto {
        // Auto mode: check if flag.txt exists for compat
        let flag_path = current_dir.join("flag.txt");
        if flag_path.exists() {
            let f = fs::read_to_string(&flag_path)?.trim().to_string();
            (ChallengeStatus::Solved, Some(f))
        } else {
            (ChallengeStatus::Unsolved, None)
        }
    } else {
        prompt_status()?
    };

    meta.status = status.clone();
    meta.flag = flag;

    if status == ChallengeStatus::TeamSolved {
        if auto {
            meta.solved_by = Some("team".to_string());
        } else {
            let who: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Who solved it?")
                .default("team".to_string())
                .interact_text()?;
            meta.solved_by = Some(who);
        }
    } else if status == ChallengeStatus::Solved {
        meta.solved_by = Some("me".to_string());
    }

    // ── Step 2: File triage ───────────────────────────────────────────
    if !no_clean && !auto {
        triage_files(&current_dir, &meta, config)?;
    }

    // ── Step 3: Note ──────────────────────────────────────────────────
    if let Some(n) = note_arg {
        meta.note = Some(n);
    } else if !auto {
        let note: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Add a note? (enter to skip)")
            .allow_empty(true)
            .interact_text()?;
        if !note.is_empty() {
            meta.note = Some(note);
        }
    }

    // ── Step 4: Save metadata ─────────────────────────────────────────
    meta.shelved_at = Some(chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string());
    meta.save(&current_dir)?;

    let status_label = match &meta.status {
        ChallengeStatus::Solved => "Solved",
        ChallengeStatus::TeamSolved => "Team-Solved",
        ChallengeStatus::Unsolved => "Shelved (unsolved)",
        ChallengeStatus::Active => "Active",
    };
    println!("✓ {}: {}", status_label, meta.name);

    // ── Step 5: Git commit ────────────────────────────────────────────
    if !no_commit {
        let commit_msg = match &meta.status {
            ChallengeStatus::Solved => format!(
                "Solved: {} (Flag: {})",
                meta.name,
                meta.flag.as_deref().unwrap_or("?")
            ),
            ChallengeStatus::TeamSolved => format!("Team-Solved: {}", meta.name),
            ChallengeStatus::Unsolved => format!("Shelved: {}", meta.name),
            ChallengeStatus::Active => format!("Updated: {}", meta.name),
        };
        git_commit(&current_dir, &commit_msg);
    }

    // ── Step 6: Archive ───────────────────────────────────────────────
    let should_move = if force_move {
        true
    } else if no_move {
        false
    } else if auto {
        meta.status == ChallengeStatus::Solved || meta.status == ChallengeStatus::TeamSolved
    } else {
        Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Move to archives?")
            .default(meta.status == ChallengeStatus::Solved)
            .interact()
            .unwrap_or(false)
    };

    if should_move {
        archive_challenge(config, &current_dir, &meta)?;
    }

    Ok(())
}

/// Interactive status selection.
fn prompt_status() -> Result<(ChallengeStatus, Option<String>)> {
    let choices = &["I solved it", "Team solved it", "Unsolved — shelve for later"];

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("What happened with this challenge?")
        .items(choices)
        .default(0)
        .interact()?;

    match selection {
        0 => {
            let flag: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter the flag")
                .interact_text()?;
            Ok((ChallengeStatus::Solved, Some(flag)))
        }
        1 => {
            let flag: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter the flag (if known, or enter to skip)")
                .allow_empty(true)
                .interact_text()?;
            let flag = if flag.is_empty() { None } else { Some(flag) };
            Ok((ChallengeStatus::TeamSolved, flag))
        }
        _ => Ok((ChallengeStatus::Unsolved, None)),
    }
}

/// File triage with blacklist/whitelist from config.
fn triage_files(challenge_dir: &PathBuf, meta: &ChallengeMetadata, config: &Config) -> Result<()> {
    let blacklist = &config.ctf.shelve.blacklist;
    let whitelist = &config.ctf.shelve.whitelist;

    let mut entries: Vec<(String, u64, bool)> = Vec::new(); // (name, size, default_delete)

    for entry in std::fs::read_dir(challenge_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        let size = dir_size(&entry.path());
        let is_blacklisted = blacklist.iter().any(|pat| name.starts_with(pat.as_str()));
        let is_whitelisted = whitelist.iter().any(|pat| name.starts_with(pat.as_str()));

        // Also whitelist the imported original
        let is_imported = meta
            .imported_from
            .as_ref()
            .is_some_and(|imp| name == *imp);

        if is_whitelisted || is_imported {
            continue; // Always keep, don't show in triage
        }

        entries.push((name, size, is_blacklisted));
    }

    if entries.is_empty() {
        return Ok(());
    }

    // Sort: blacklisted first, then by size descending
    entries.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| b.1.cmp(&a.1)));

    let labels: Vec<String> = entries
        .iter()
        .map(|(name, size, blacklisted)| {
            let size_str = format_size(*size);
            let marker = if *blacklisted { " [trash]" } else { "" };
            format!("{:<30} {:>10}{}", name, size_str, marker)
        })
        .collect();

    // Pre-select blacklisted items for deletion
    let defaults: Vec<bool> = entries.iter().map(|(_, _, bl)| *bl).collect();

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Select files to DELETE (space to toggle, enter to confirm)")
        .items(&labels)
        .defaults(&defaults)
        .interact()?;

    let mut deleted_size: u64 = 0;
    for idx in selections {
        let (name, size, _) = &entries[idx];
        let path = challenge_dir.join(name);
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            fs::remove_file(&path)?;
        }
        deleted_size += size;
    }

    if deleted_size > 0 {
        println!("✓ Cleaned up {}", format_size(deleted_size));
    }

    Ok(())
}

/// Move challenge to PARA archives.
fn archive_challenge(
    config: &Config,
    challenge_dir: &PathBuf,
    meta: &ChallengeMetadata,
) -> Result<()> {
    if let Some(category_dir) = challenge_dir.parent() {
        if let Some(event_dir) = category_dir.parent() {
            let event_dir_name = event_dir
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let event_meta = CtfMeta::load(event_dir)?.unwrap_or_else(|| {
                CtfMeta::new(&event_dir_name, None, None, None)
            });

            let event_name = &event_dir_name;
            let year = event_meta.year.to_string();

            let target_dir = config
                .ctf_archive_path(&year, event_name)
                .join(&meta.category)
                .join(&meta.name);

            if let Some(parent) = target_dir.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }

            // Try rename first, fallback to copy+delete for cross-device
            if fs::rename(challenge_dir, &target_dir).is_err() {
                let options = fs_extra::dir::CopyOptions::new();
                fs_extra::dir::copy(
                    challenge_dir,
                    target_dir.parent().context("Archive target path has no parent directory")?,
                    &options,
                )
                .context("Failed to archive (cross-device move)")?;
                fs::remove_dir_all(challenge_dir)?;
            }

            println!("✓ Archived to: {}", target_dir.display());
        }
    }
    Ok(())
}

fn git_commit(dir: &std::path::Path, message: &str) {
    use std::process::Command;

    let add_status = Command::new("git")
        .arg("add")
        .arg(".")
        .current_dir(dir)
        .status();

    if let Ok(st) = add_status {
        if st.success() {
            let commit_status = Command::new("git")
                .arg("commit")
                .arg("-m")
                .arg(message)
                .current_dir(dir)
                .status();

            match commit_status {
                Ok(st) if st.success() => println!("✓ Committed: {}", message),
                _ => println!("! Git commit skipped (nothing to commit?)"),
            }
        }
    }
}

fn dir_size(path: &std::path::Path) -> u64 {
    if path.is_file() {
        std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
    } else {
        let mut total = 0u64;
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                total += dir_size(&entry.path());
            }
        }
        total
    }
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
