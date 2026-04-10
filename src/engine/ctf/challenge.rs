//! Challenge operations — add, solve, status, and writeup generation.
//!
//! The solve workflow: save flag → detect solution script → update notes →
//! git commit → compress → archive to `4_Archives/CTFs/{year}/{event}/{category}/{name}`.

use crate::config::Config;
use anyhow::{Context, Result};
use fs_err as fs;

use super::{add_solve_script, ChallengeMetadata, CtfMeta};

pub fn add_challenge(_config: &Config, path: &str) -> Result<std::path::PathBuf> {
    let event_root = super::get_active_event_root()?;

    let parts: Vec<&str> = path.split('/').collect();

    let (category, name) = if parts.len() == 2 {
        (parts[0].to_string(), parts[1].to_string())
    } else if parts.len() == 1 {
        // Try to infer category from CWD
        let current_dir = std::env::current_dir()?;
        // Check if current dir is a direct child of event_root
        if current_dir.parent() == Some(&event_root) {
            let cat_name = current_dir
                .file_name()
                .ok_or_else(|| {
                    anyhow::anyhow!("Cannot determine category: current directory has no name")
                })?
                .to_string_lossy();
            (cat_name.to_string(), parts[0].to_string())
        } else {
            anyhow::bail!(
                "Invalid format. Use <category>/<name> OR run inside a category folder.\n\n\
                Examples:\n  \
                wardex ctf add pwn/buffer-overflow\n  \
                wardex ctf add web/sql-injection"
            );
        }
    } else {
        anyhow::bail!(
            "Invalid format. Use <category>/<name>\n\n\
            Examples:\n  \
            wardex ctf add pwn/buffer-overflow"
        );
    };

    let category_dir = event_root.join(&category);
    if !category_dir.exists() {
        println!("Creating category: {}", category);
        fs::create_dir(&category_dir)?;
    }

    let challenge_dir = category_dir.join(&name);
    if challenge_dir.exists() {
        anyhow::bail!(
            "Challenge already exists: {:?}\n\n\
            Tip: Use a different name or remove the existing directory first.",
            challenge_dir
        );
    }

    fs::create_dir(&challenge_dir)?;
    println!("Created challenge: {}/{}", category, name);

    add_solve_script(&challenge_dir, &category)?;

    // Create .challenge.json metadata
    let meta = ChallengeMetadata::new(&name, &category);
    meta.save(&challenge_dir)?;

    Ok(challenge_dir)
}

pub fn solve_challenge(
    config: &Config,
    flag: &str,
    create: Option<String>,
    desc: Option<String>,
    no_archive: bool,
    no_commit: bool,
) -> Result<()> {
    let current_dir = if let Some(path_str) = create {
        // Mode 1: Create on the fly
        let event_root = super::get_active_event_root()?;
        let parts: Vec<&str> = path_str.split('/').collect();

        let (category, name) = if parts.len() == 2 {
            (parts[0].to_string(), parts[1].to_string())
        } else {
            // Check if we are inside a category directory
            let cwd = std::env::current_dir()?;
            if let Some(parent) = cwd.parent() {
                if parent == event_root {
                    let cat_name = cwd
                        .file_name()
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "Cannot determine category: current directory has no name"
                            )
                        })?
                        .to_string_lossy()
                        .to_string();
                    (cat_name, parts[0].to_string())
                } else {
                    anyhow::bail!(
                        "Invalid format. Use <category>/<name> or run inside a category folder."
                    );
                }
            } else {
                anyhow::bail!("Invalid format. Use <category>/<name>");
            }
        };

        let category_dir = event_root.join(&category);
        if !category_dir.exists() {
            fs::create_dir(&category_dir)?;
        }

        let challenge_dir = category_dir.join(&name);
        if !challenge_dir.exists() {
            fs::create_dir(&challenge_dir)?;
            println!("Created challenge: {}/{}", category, name);
        }

        challenge_dir
    } else {
        // Mode 2: Existing challenge (CWD)
        std::env::current_dir()?
    };

    let dir_name = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid directory name"))?;

    // 1. Save flag
    let flag_path = current_dir.join("flag.txt");
    fs::write(&flag_path, flag)?;
    println!("✓ Saved flag to {:?}", flag_path);

    // 1.5 Write Description if provided
    if let Some(description) = desc {
        let notes_path = current_dir.join("notes.md");
        let header = "\n\n## Description\n\n";

        use std::io::Write;
        let mut file = fs::File::options()
            .create(true)
            .append(true)
            .open(&notes_path)?;
        write!(file, "{}{}", header, description)?;
        println!("✓ Appended description to notes.md");
    }

    // 2. Scan for solution script (convention: solve.*)
    let mut solution_script = None;
    let candidates = [
        "solve.py",
        "solve.sh",
        "exploit.py",
        "exploit.sh",
        "solve.rb",
        "solve.js",
        "solution.txt",
    ];

    for candidate in candidates {
        if current_dir.join(candidate).exists() {
            solution_script = Some(candidate.to_string());
            break;
        }
    }

    if let Some(script_name) = &solution_script {
        println!("✓ Detected solution script: {}", script_name);
    } else {
        println!("! No standard solution script found (e.g. solve.py). Skipping writeup append.");
    }

    // 3. Update Writeup (notes.md)
    if let Some(script_name) = &solution_script {
        let script_path = current_dir.join(script_name);
        if let Ok(content) = fs::read_to_string(&script_path) {
            let notes_path = current_dir.join("notes.md");
            let notes_content = fs::read_to_string(&notes_path).unwrap_or_default();

            let ext = script_name.split('.').next_back().unwrap_or("");
            let header = format!("\n\n## Solution Code ({})\n\n```{}\n", script_name, ext);
            let footer = "\n```\n";

            // Avoid duplication if possible
            if !notes_content.contains(&header) {
                use std::io::Write;
                let mut file = fs::File::options()
                    .create(true)
                    .append(true)
                    .open(&notes_path)?;
                write!(file, "{}{}{}", header, content, footer)?;
                println!("✓ Appended {} to notes.md", script_name);
            }
        }
    }

    // 4. Git commit (Everything)
    if !no_commit {
        use std::process::Command;
        println!("\nCommitting ALL changes (history)...");

        let add_status = Command::new("git")
            .arg("add")
            .arg(".")
            .current_dir(&current_dir)
            .status();

        if let Ok(st) = add_status {
            if st.success() {
                let commit_msg = format!("Solved: {} (Flag: {})", dir_name, flag);
                let commit_status = Command::new("git")
                    .arg("commit")
                    .arg("-m")
                    .arg(&commit_msg)
                    .current_dir(&current_dir)
                    .status();

                if let Ok(c_st) = commit_status {
                    if c_st.success() {
                        println!("✓ Committed changes");
                    } else {
                        println!("! Git commit failed (nothing to commit?)");
                    }
                } else {
                    println!("! Failed to execute git commit command.");
                }
            } else {
                println!("! git add failed. Skipping commit.");
            }
        } else {
            println!("! Failed to execute git add command. Is git installed?");
        }
    }

    // 5. Compress Everything (respecting .gitignore)
    if !no_archive {
        println!("Compressing artifacts...");
        let zip_path = current_dir.join("solution.zip");
        if let Err(e) = super::archive::create_zip(&current_dir, &zip_path) {
            println!(
                "! Failed to create solution.zip (probably no git repo): {}",
                e
            );
        } else {
            println!("✓ Created solution.zip");
        }
    }

    // 6. Archive
    if !no_archive {
        if let Some(category_dir) = current_dir.parent() {
            if let Some(event_dir) = category_dir.parent() {
                let meta_path = event_dir.join(".ctf_meta.json");

                if meta_path.exists() {
                    let category_name = category_dir
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    let event_name = event_dir
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    let year = if let Some(meta) = CtfMeta::load(event_dir).ok().flatten() {
                        meta.year.to_string()
                    } else {
                        chrono::Local::now().format("%Y").to_string()
                    };

                    let target_dir = config
                        .ctf_archive_path(&year, &event_name)
                        .join(&category_name)
                        .join(dir_name);

                    if let Some(parent) = target_dir.parent() {
                        if !parent.exists() {
                            fs::create_dir_all(parent)?;
                        }
                    }

                    println!("Archiving to {:?}...", target_dir);

                    // Try rename first, fallback to copy+delete for cross-device
                    if fs::rename(&current_dir, &target_dir).is_err() {
                        let options = fs_extra::dir::CopyOptions::new();
                        fs_extra::dir::copy(
                            &current_dir,
                            target_dir
                                .parent()
                                .context("Archive target path has no parent directory")?,
                            &options,
                        )
                        .context("Failed to archive (cross-device move)")?;
                        fs::remove_dir_all(&current_dir)?;
                    }

                    println!("✓ Archived to: {}", target_dir.display());
                }
            }
        }
    }

    Ok(())
}

pub fn generate_writeup(_config: &Config) -> Result<()> {
    let event_root = super::get_active_event_root()?;

    let meta = CtfMeta::load(&event_root)?.context("No CTF metadata found (.ctf_meta.json)")?;
    let mut writeup_content = format!("# Writeup: {}\n\nDate: {}\n\n", meta.name, meta.date);

    // Walk through categories and challenges
    if let Ok(cats) = fs::read_dir(&event_root) {
        let mut categories: Vec<_> = cats.filter_map(|e| e.ok()).collect();
        categories.sort_by_key(|e| e.file_name());

        for cat in categories {
            if cat.path().is_dir() && !cat.file_name().to_string_lossy().starts_with('.') {
                let cat_name = cat.file_name().to_string_lossy().to_string();

                if let Ok(chals) = fs::read_dir(cat.path()) {
                    let mut challenges: Vec<_> = chals.filter_map(|e| e.ok()).collect();
                    challenges.sort_by_key(|e| e.file_name());

                    for chal in challenges {
                        if chal.path().is_dir() {
                            let chal_name = chal.file_name().to_string_lossy().to_string();

                            // Check for notes
                            let notes_path = chal.path().join("notes.md");
                            let readme_path = chal.path().join("README.md");

                            let content = if notes_path.exists() {
                                fs::read_to_string(notes_path).unwrap_or_default()
                            } else if readme_path.exists() {
                                fs::read_to_string(readme_path).unwrap_or_default()
                            } else {
                                String::new()
                            };

                            if !content.trim().is_empty() {
                                writeup_content
                                    .push_str(&format!("## [{}] {}\n\n", cat_name, chal_name));
                                writeup_content.push_str(&content);
                                writeup_content.push_str("\n\n---\n\n");
                            }
                        }
                    }
                }
            }
        }
    }

    let writeup_path = event_root.join("Writeup.md");
    fs::write(&writeup_path, writeup_content)?;
    println!("Generated writeup at {:?}", writeup_path);

    Ok(())
}

#[derive(tabled::Tabled)]
struct ChallengeStatusRow {
    #[tabled(rename = "Category")]
    category: String,
    #[tabled(rename = "Challenge")]
    challenge: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Solver")]
    solved_by: String,
    #[tabled(rename = "Note")]
    note: String,
}

pub fn challenge_status(config: &Config, format: &str) -> Result<()> {
    let event_root = super::get_active_event_root()?;
    let meta = CtfMeta::load(&event_root)?
        .ok_or_else(|| anyhow::anyhow!("No CTF metadata found (.ctf_meta.json)"))?;

    let event_name = event_root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let archives_root = config.ctf_archive_path(&meta.year.to_string(), &event_name);

    let mut statuses = Vec::new();

    // 1. Scan Active Event Dir
    if let Ok(cats) = fs::read_dir(&event_root) {
        for cat in cats.flatten() {
            if cat.path().is_dir() && !cat.file_name().to_string_lossy().starts_with('.') {
                let cat_name = cat.file_name().to_string_lossy().to_string();
                if let Ok(chals) = fs::read_dir(cat.path()) {
                    for chal in chals.flatten() {
                        if chal.path().is_dir() {
                            let (display_status, solver, note_preview) =
                                if let Ok(Some(cmeta)) =
                                    ChallengeMetadata::load_or_migrate(&chal.path())
                                {
                                    let status_str = match cmeta.status {
                                        super::ChallengeStatus::Solved => "✓ Solved".to_string(),
                                        super::ChallengeStatus::TeamSolved => {
                                            "✓ Team".to_string()
                                        }
                                        super::ChallengeStatus::Unsolved => {
                                            "✗ Unsolved".to_string()
                                        }
                                        super::ChallengeStatus::Active => "⌚ Active".to_string(),
                                    };
                                    let solver =
                                        cmeta.solved_by.unwrap_or_else(|| "-".to_string());
                                    let note = cmeta
                                        .note
                                        .map(|n| {
                                            if n.len() > 30 {
                                                format!("{}...", &n[..27])
                                            } else {
                                                n
                                            }
                                        })
                                        .unwrap_or_else(|| "-".to_string());
                                    (status_str, solver, note)
                                } else {
                                    (
                                        "⌚ Active".to_string(),
                                        "-".to_string(),
                                        "-".to_string(),
                                    )
                                };
                            statuses.push(ChallengeStatusRow {
                                category: cat_name.clone(),
                                challenge: chal.file_name().to_string_lossy().to_string(),
                                status: display_status,
                                solved_by: solver,
                                note: note_preview,
                            });
                        }
                    }
                }
            }
        }
    }

    // 2. Scan Archive Dir for Solved
    if archives_root.exists() {
        if let Ok(cats) = fs::read_dir(&archives_root) {
            for cat in cats.flatten() {
                if cat.path().is_dir() {
                    let cat_name = cat.file_name().to_string_lossy().to_string();
                    if let Ok(chals) = fs::read_dir(cat.path()) {
                        for chal in chals.flatten() {
                            if chal.path().is_dir() {
                                statuses.push(ChallengeStatusRow {
                                    category: cat_name.clone(),
                                    challenge: chal.file_name().to_string_lossy().to_string(),
                                    status: "✓ Solved".to_string(),
                                    solved_by: "-".to_string(),
                                    note: "-".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    if statuses.is_empty() {
        println!("No challenges found for event: {}", meta.name);
        return Ok(());
    }

    // Sort by Category, then Status (Active first), then Challenge Name
    statuses.sort_by(|a, b| {
        a.category
            .cmp(&b.category)
            .then_with(|| a.status.cmp(&b.status))
            .then_with(|| a.challenge.cmp(&b.challenge))
    });

    // JSON output
    if format == "json" {
        #[derive(serde::Serialize)]
        struct JsonRow {
            category: String,
            challenge: String,
            status: String,
            solved_by: String,
            note: String,
        }
        let json_rows: Vec<JsonRow> = statuses
            .iter()
            .map(|s| JsonRow {
                category: s.category.clone(),
                challenge: s.challenge.clone(),
                status: s.status.clone(),
                solved_by: s.solved_by.clone(),
                note: s.note.clone(),
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&json_rows)?);
        return Ok(());
    }

    use tabled::{settings::Style, Table};
    let mut table = Table::new(&statuses);
    table.with(Style::modern());

    println!("Status for CTF Event: {}", meta.name);
    println!("{}", table);

    // Summary
    let total = statuses.len();
    let solved_total = statuses.iter().filter(|s| s.status.contains('✓')).count();
    let solved_me = statuses
        .iter()
        .filter(|s| s.status.contains("Solved") && s.solved_by == "me")
        .count();
    let solved_team = statuses.iter().filter(|s| s.status.contains("Team")).count();
    let unsolved = statuses
        .iter()
        .filter(|s| s.status.contains("Unsolved"))
        .count();
    let active = statuses
        .iter()
        .filter(|s| s.status.contains("Active"))
        .count();

    println!(
        "\n{} total | ✓ {} solved (me: {}, team: {}) | ✗ {} unsolved | ⌚ {} active",
        total, solved_total, solved_me, solved_team, unsolved, active
    );

    Ok(())
}
