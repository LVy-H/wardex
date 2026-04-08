//! Event archival — moves completed events to `4_Archives/CTFs/{year}/` and
//! creates gitignore-respecting zip archives for solved challenges.

use crate::config::Config;
use crate::utils::fs::move_item;
use anyhow::Result;
use chrono::prelude::*;
use fs_err as fs;
use std::path::Path;

use super::CtfMeta;

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
