use crate::config::Config;
use anyhow::{Context, Result};
use fs_err as fs;


use super::{add_solve_script, CtfMeta};

pub fn add_challenge(_config: &Config, path: &str) -> Result<()> {
    let event_root = super::get_active_event_root()?;

    let parts: Vec<&str> = path.split('/').collect();

    let (category, name) = if parts.len() == 2 {
        (parts[0].to_string(), parts[1].to_string())
    } else if parts.len() == 1 {
        // Try to infer category from CWD
        let current_dir = std::env::current_dir()?;
        // Check if current dir is a direct child of event_root
        if current_dir.parent() == Some(&event_root) {
            let cat_name = current_dir.file_name().unwrap().to_string_lossy();
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

    Ok(())
}

pub fn solve_challenge(
    config: &Config,
    flag: &str,
    create: Option<String>,
    desc: Option<String>,
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
                    let cat_name = cwd.file_name().unwrap().to_string_lossy().to_string();
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

        // We MUST change directory to the challenge dir for the rest of the logic to work
        std::env::set_current_dir(&challenge_dir)?;
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
    println!("\nCommitting ALL changes (history)...");
    use std::process::Command;

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

    // 5. Compress Everything (respecting .gitignore)
    println!("Compressing artifacts...");
    let zip_path = current_dir.join("solution.zip");
    if let Err(e) = super::archive::create_zip(&current_dir, &zip_path) {
        println!("! Failed to create solution.zip (probably no git repo): {}", e);
    } else {
        println!("✓ Created solution.zip");
    }

    // 6. Archive
    println!("DEBUG: current_dir before archive logic = {:?}", current_dir);
    if let Some(category_dir) = current_dir.parent() {
        println!("DEBUG: category_dir = {:?}", category_dir);
        if let Some(event_dir) = category_dir.parent() {
            println!("DEBUG: event_dir = {:?}", event_dir);
            let meta_path = event_dir.join(".ctf_meta.json");
            println!("DEBUG: meta_path exists? {} ({:?})", meta_path.exists(), meta_path);
            
            if meta_path.exists() {
                let category_name = category_dir.file_name().unwrap().to_string_lossy();
                let event_name = event_dir.file_name().unwrap().to_string_lossy();
                let year = if let Some(meta) = CtfMeta::load(&event_dir) {
                    meta.year.to_string()
                } else {
                    chrono::Local::now().format("%Y").to_string()
                };

                let archives_root = config.resolve_path("archives").join("CTFs");
                let target_dir = archives_root
                    .join(&year)
                    .join(event_name.as_ref())
                    .join(category_name.as_ref())
                    .join(dir_name);

                if !target_dir.parent().unwrap().exists() {
                    fs::create_dir_all(target_dir.parent().unwrap())?;
                }

                println!("Archiving to {:?}...", target_dir);

                // Try rename first, fallback to copy+delete for cross-device
                if let Err(_) = fs::rename(&current_dir, &target_dir) {
                    let options = fs_extra::dir::CopyOptions::new();
                    fs_extra::dir::copy(&current_dir, target_dir.parent().unwrap(), &options)
                        .context("Failed to archive (cross-device move)")?;
                    fs::remove_dir_all(&current_dir)?;
                }
                
                println!("✓ Challenge archived. Note: Your current directory has been moved.");
            }
        }
    }

    Ok(())
}

pub fn generate_writeup(_config: &Config) -> Result<()> {
    let event_root = super::get_active_event_root()?;

    let meta =
        CtfMeta::load(&event_root).context("Failed to load CTF metadata (.ctf_meta.json)")?;
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
struct ChallengeStatus {
    #[tabled(rename = "Category")]
    category: String,
    #[tabled(rename = "Challenge")]
    challenge: String,
    #[tabled(rename = "Status")]
    status: String,
}

pub fn challenge_status(config: &Config) -> Result<()> {
    let event_root = super::get_active_event_root()?;
    let meta = CtfMeta::load(&event_root)
        .ok_or_else(|| anyhow::anyhow!("Failed to load CTF metadata (.ctf_meta.json)"))?;
    
    let event_name = event_root.file_name().unwrap().to_string_lossy();
    let archives_root = config.resolve_path("archives")
        .join("CTFs")
        .join(meta.year.to_string())
        .join(event_name.as_ref());
        
    let mut statuses = Vec::new();

    // 1. Scan Active Event Dir for Unsolved
    if let Ok(cats) = fs::read_dir(&event_root) {
        for cat in cats.flatten() {
            if cat.path().is_dir() && !cat.file_name().to_string_lossy().starts_with('.') {
                let cat_name = cat.file_name().to_string_lossy().to_string();
                if let Ok(chals) = fs::read_dir(cat.path()) {
                    for chal in chals.flatten() {
                        if chal.path().is_dir() {
                            statuses.push(ChallengeStatus {
                                category: cat_name.clone(),
                                challenge: chal.file_name().to_string_lossy().to_string(),
                                status: "⌚ Active".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    // 2. Scan Archive Dir for Solved
    println!("DEBUG: Checking archive root: {:?}", archives_root);
    if archives_root.exists() {
        if let Ok(cats) = fs::read_dir(&archives_root) {
            for cat in cats.flatten() {
                if cat.path().is_dir() {
                    let cat_name = cat.file_name().to_string_lossy().to_string();
                    if let Ok(chals) = fs::read_dir(cat.path()) {
                        for chal in chals.flatten() {
                            if chal.path().is_dir() {
                                statuses.push(ChallengeStatus {
                                    category: cat_name.clone(),
                                    challenge: chal.file_name().to_string_lossy().to_string(),
                                    status: "✓ Solved".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    } else {
        println!("DEBUG: Archive root does not exist: {:?}", archives_root);
    }

    if statuses.is_empty() {
        println!("No challenges found for event: {}", meta.name);
        return Ok(());
    }

    // Sort by Category, then Status (Active first), then Challenge Name
    statuses.sort_by(|a, b| {
        a.category.cmp(&b.category)
            .then_with(|| a.status.cmp(&b.status))
            .then_with(|| a.challenge.cmp(&b.challenge))
    });

    use tabled::{Table, settings::Style};
    let mut table = Table::new(statuses);
    table.with(Style::modern());

    println!("Status for CTF Event: {}", meta.name);
    println!("{}", table);

    Ok(())
}
