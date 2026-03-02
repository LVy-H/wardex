use crate::config::Config;
use crate::utils::fs::move_item;
use anyhow::Result;
use chrono::prelude::*;
use fs_err as fs;
use std::path::{Path, PathBuf};

use super::event::list_events;
use super::CtfMeta;

pub fn get_event_path(
    config: &Config,
    event_name: Option<&str>,
    challenge_name: Option<&str>,
) -> Result<PathBuf> {
    use fuzzy_matcher::skim::SkimMatcherV2;
    use fuzzy_matcher::FuzzyMatcher;

    let events = list_events(config)?;

    if events.ctf_root_missing {
        anyhow::bail!("CTF root directory not found");
    }

    if events.events.is_empty() {
        anyhow::bail!("No CTF events found");
    }

    let matcher = SkimMatcherV2::default();

    let event_path = if let Some(name) = event_name {
        let exact = events
            .events
            .iter()
            .find(|e| e.name.to_lowercase() == name.to_lowercase());

        if let Some(e) = exact {
            e.path.clone()
        } else {
            let mut matches: Vec<_> = events
                .events
                .iter()
                .filter_map(|e| matcher.fuzzy_match(&e.name, name).map(|score| (score, e)))
                .collect();
            
            matches.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
            
            if let Some((_, best_match)) = matches.first() {
                best_match.path.clone()
            } else {
                anyhow::bail!("Event not found: {}", name);
            }
        }
    } else {
        // Prefer current event context (local or global) if available
        if let Ok(root) = super::get_active_event_root() {
            root
        } else {
            // Fallback to latest
            events
                .events
                .iter()
                .max_by_key(|e| e.year)
                .map(|e| e.path.clone())
                .ok_or_else(|| anyhow::anyhow!("No CTF events found"))?
        }
    };

    if let Some(chall) = challenge_name {
        let (cat_filter, chall_query) = if chall.contains('/') {
            let parts: Vec<&str> = chall.splitn(2, '/').collect();
            (Some(parts[0]), parts[1])
        } else {
            (None, chall)
        };

        let mut best_score = 0;
        let mut best_path = None;

        for entry in fs::read_dir(&event_path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let cat_name = entry.file_name().to_string_lossy().to_string();

                if let Some(cat) = cat_filter {
                    if cat_name != cat {
                        continue;
                    }
                }

                for chall_entry in fs::read_dir(entry.path())? {
                    let chall_entry = chall_entry?;
                    if chall_entry.file_type()?.is_dir() {
                        let cname = chall_entry.file_name().to_string_lossy().to_string();
                        
                        if cname.to_lowercase() == chall_query.to_lowercase() {
                            return Ok(chall_entry.path());
                        }

                        if let Some(score) = matcher.fuzzy_match(&cname, chall_query) {
                            if score > best_score {
                                best_score = score;
                                best_path = Some(chall_entry.path());
                            }
                        }
                    }
                }
            }
        }

        if let Some(path) = best_path {
            return Ok(path);
        }

        anyhow::bail!("Challenge not found: {}", chall);
    }

    Ok(event_path)
}

pub fn archive_event(config: &Config, name: &str) -> Result<()> {
    let ctf_root = config.ctf_root();
    // PARA Archives
    let archives_root = config.resolve_path("archives").join("CTFs");

    if !archives_root.exists() {
        fs::create_dir_all(&archives_root)?;
    }

    // Find the event folder
    let mut event_dir = ctf_root.join(name);
    // Try to find it if name is partial specific
    if !event_dir.exists() {
        // search for directory containing name
        if let Ok(entries) = fs::read_dir(&ctf_root) {
            for entry in entries.flatten() {
                let db_name = entry.file_name().to_string_lossy().to_string();
                if db_name.contains(name) {
                    event_dir = entry.path();
                    break;
                }
            }
        }
    }

    if !event_dir.exists() {
        anyhow::bail!("Event directory not found: {}", name);
    }

    // Load meta to get year
    let year = if let Some(meta) = CtfMeta::load(&event_dir) {
        meta.year.to_string()
    } else {
        Local::now().year().to_string()
    };

    let archive_year_dir = archives_root.join(&year);
    if !archive_year_dir.exists() {
        fs::create_dir_all(&archive_year_dir)?;
    }

    let target_dir = archive_year_dir.join(event_dir.file_name().unwrap());

    println!("Archiving {:?} -> {:?}", event_dir, target_dir);

    match move_item(config, &event_dir, &archive_year_dir, false) {
        Ok(_) => println!("Event archived successfully."),
        Err(e) => anyhow::bail!("Failed to archive event: {}", e),
    }

    Ok(())
}

pub(crate) fn create_zip(src_dir: &Path, dest_file: &Path) -> Result<()> {
    use ignore::WalkBuilder;
    use std::io::{Read, Write};
    use zip::write::FileOptions;

    let file = fs::File::create(dest_file)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    // Zip everything that is NOT ignored by git
    for entry in WalkBuilder::new(src_dir)
        .hidden(false)
        .git_ignore(true)
        .build()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        // Skip the zip file itself and directories
        if path.is_file() && path != dest_file {
            if let Ok(name) = path.strip_prefix(src_dir) {
                zip.start_file(name.to_string_lossy(), options)?;
                let mut f = fs::File::open(path)?;
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer)?;
                zip.write_all(&buffer)?;
            }
        }
    }
    zip.finish()?;
    Ok(())
}
