//! Display formatting for command output.
//!
//! Each `display_*` function takes a report struct from an engine module
//! and renders it to stdout/stderr via `log` macros and `println!`.

use log::{error, info, warn};

use crate::config::Config;
use crate::engine::{
    auditor::AuditReport,
    cleaner::CleanReport,
    search::{Match, SearchReport, SearchResult},
    status::StatusReport,
    undo::UndoReport,
};

pub fn display_clean_report(config: &Config, report: &CleanReport) {
    if report.inbox_not_found {
        error!("Inbox path not found: {:?}", config.resolve_path("inbox"));
        return;
    }

    if report.inbox_empty {
        warn!("Inbox is empty.");
        return;
    }

    for item in &report.moved {
        if item.dry_run {
            info!("Would move {:?} -> {:?}", item.source, item.destination);
        } else {
            info!(
                "✓ Moved {:?} -> {:?}",
                item.source.file_name().unwrap_or_default(),
                item.destination
            );
        }
    }

    for item in &report.skipped {
        log::debug!(
            "Skipped: {:?} ({})",
            item.path.file_name().unwrap_or_default(),
            item.reason
        );
    }

    for err in &report.errors {
        error!("{}", err);
    }

    info!(
        "Moved: {}, Skipped: {}, Errors: {}",
        report.moved.len(),
        report.skipped.len(),
        report.errors.len()
    );
}

pub fn display_audit_report(config: &Config, report: &AuditReport) {
    if report.workspace_not_found {
        error!(
            "Workspace not found: {:?}",
            config.resolve_path("workspace")
        );
        return;
    }

    info!("Analyzed {} items.", report.items_scanned);

    if !report.empty_folders.is_empty() {
        warn!("Empty Folders Found: {}", report.empty_folders.len());
        for p in report.empty_folders.iter().take(10) {
            println!(" - {:?}", p);
        }
        if report.empty_folders.len() > 10 {
            println!("... and {} more", report.empty_folders.len() - 10);
        }
    }

    if !report.suspicious_extensions.is_empty() {
        warn!("Suspicious Extensions (Magic Byte Mismatch):");
        for item in &report.suspicious_extensions {
            println!(
                " - {:?} (Named: .{}, Real: .{})",
                item.path, item.declared_ext, item.actual_ext
            );
        }
    }

    info!("✓ Audit Complete.");
}

pub fn display_undo_report(report: &UndoReport) {
    if report.no_log_found {
        warn!("No undo log found.");
        return;
    }

    if report.log_empty {
        warn!("Undo log is empty.");
        return;
    }

    info!("Undoing {} operations...", report.undone.len());

    for item in &report.undone {
        if item.success {
            info!(
                "✓ Reverted: {:?} -> {:?}",
                item.source.file_name().unwrap_or_default(),
                item.destination
            );
        } else {
            error!(
                "✗ Failed: {:?} ({})",
                item.source.file_name().unwrap_or_default(),
                item.error.as_deref().unwrap_or("Unknown error")
            );
        }
    }

    let success_count = report.undone.iter().filter(|i| i.success).count();
    info!(
        "Completed: {}/{} operations",
        success_count,
        report.undone.len()
    );
}

pub fn display_status_report(_config: &Config, report: &StatusReport) {
    if report.workspace_not_found {
        error!("Workspace not found.");
        return;
    }

    if report.repos.is_empty() {
        warn!("No git repositories found.");
        return;
    }

    println!("\n{:<25} {:<12} {:<15} Path", "Project", "State", "Sync");
    println!("{}", "-".repeat(80));

    for repo in &report.repos {
        let state = if repo.is_dirty {
            "⚠ Dirty"
        } else {
            "✓ Clean"
        };
        println!(
            "{:<25} {:<12} {:<15} {}",
            repo.name,
            state,
            repo.sync_status.display(),
            repo.path.display()
        );
    }

    let dirty_count = report.repos.iter().filter(|r| r.is_dirty).count();
    info!(
        "Total: {} repos ({} dirty)",
        report.repos.len(),
        dirty_count
    );
}

pub fn display_search_report(report: &SearchReport) {
    for m in &report.matches {
        let location = if let Some(ref entry) = m.archive_entry {
            format!("{} (in {})", entry, m.file_path)
        } else if let Some(line) = m.line_number {
            format!("{}:{}", m.file_path, line)
        } else {
            m.file_path.clone()
        };
        println!("✓ {}: {}", location, m.matched_text);
    }

    info!(
        "Scanned {} files, found {} matches.",
        report.files_scanned,
        report.matches.len()
    );

    if !report.errors.is_empty() {
        warn!("{} errors occurred:", report.errors.len());
        for e in report.errors.iter().take(5) {
            log::debug!("  - {}", e);
        }
    }
}

pub fn display_find_results(results: &[SearchResult], query: &str) {
    if results.is_empty() {
        warn!("No projects found matching '{}'", query);
    } else {
        println!("{:<50} {:<10}", "Project Path", "Score");
        println!("{}", "-".repeat(60));
        for res in results.iter().take(10) {
            println!("{:<50} {}", res.path.display(), res.score);
        }
    }
}

pub fn display_grep_results(matches: &[Match]) {
    for m in matches {
        println!(
            "{}:{}: {}",
            m.file_path,
            m.line_number.unwrap_or(0),
            m.matched_text
        );
    }
    info!("Found {} matches.", matches.len());
}
