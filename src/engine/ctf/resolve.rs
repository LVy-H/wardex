use crate::config::Config;
use anyhow::Result;
use fs_err as fs;
use std::path::PathBuf;

use super::event::list_events;

/// Resolve a CTF event (and optionally challenge) path by name, with fuzzy matching.
///
/// - If `event_name` is given, finds the event by exact or fuzzy match.
/// - If `event_name` is None, uses the active event context or latest event.
/// - If `challenge_name` is given, finds the challenge within the resolved event.
pub fn get_event_path(
    config: &Config,
    event_name: Option<&str>,
    challenge_name: Option<&str>,
) -> Result<PathBuf> {
    use fuzzy_matcher::skim::SkimMatcherV2;

    let events = list_events(config)?;

    if events.ctf_root_missing {
        anyhow::bail!("CTF root directory not found");
    }

    if events.events.is_empty() {
        anyhow::bail!("No CTF events found");
    }

    let matcher = SkimMatcherV2::default();

    let event_path = if let Some(name) = event_name {
        find_event_by_name(&events.events, name, &matcher)?
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
        return find_challenge_in_event(&event_path, chall, &matcher);
    }

    Ok(event_path)
}

/// Find an event by exact name match, then fuzzy match.
fn find_event_by_name(
    events: &[super::event::CtfEventInfo],
    name: &str,
    matcher: &fuzzy_matcher::skim::SkimMatcherV2,
) -> Result<PathBuf> {
    use fuzzy_matcher::FuzzyMatcher;

    // Exact match first
    let exact = events
        .iter()
        .find(|e| e.name.to_lowercase() == name.to_lowercase());

    if let Some(e) = exact {
        return Ok(e.path.clone());
    }

    // Fuzzy match
    let mut matches: Vec<_> = events
        .iter()
        .filter_map(|e| matcher.fuzzy_match(&e.name, name).map(|score| (score, e)))
        .collect();

    matches.sort_by_key(|(score, _)| std::cmp::Reverse(*score));

    if let Some((_, best_match)) = matches.first() {
        Ok(best_match.path.clone())
    } else {
        anyhow::bail!("Event not found: {}", name);
    }
}

/// Find a challenge within an event directory by exact or fuzzy match.
/// Supports `category/name` format to filter by category.
fn find_challenge_in_event(
    event_path: &std::path::Path,
    chall: &str,
    matcher: &fuzzy_matcher::skim::SkimMatcherV2,
) -> Result<PathBuf> {
    use fuzzy_matcher::FuzzyMatcher;

    let (cat_filter, chall_query) = if chall.contains('/') {
        let parts: Vec<&str> = chall.splitn(2, '/').collect();
        (Some(parts[0]), parts[1])
    } else {
        (None, chall)
    };

    let mut best_score = 0;
    let mut best_path = None;

    for entry in fs::read_dir(event_path)? {
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

                    // Exact match — return immediately
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
