//! Wardex CLI
//!
//! Focused on CTF management.

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use log::{error, info, warn};
use std::path::PathBuf;
use wardex::config::Config;
use wardex::core::watcher;
use wardex::engine::{auditor, cleaner, ctf, scaffold, search, stats, status, undo};
use wardex::output;
#[cfg(feature = "tui")]
use wardex::tui;

#[derive(Parser)]
#[command(name = "wardex")]
#[command(version)]
#[command(about = "Ward & index your workspace - CTF management, project organization, and more.", long_about = None)]
struct Cli {
    #[arg(long, value_name = "FILE", help = "Path to config file")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Initialize config file with defaults
    Init {
        #[arg(long, help = "Force overwrite if config exists")]
        force: bool,
    },
    /// Show current configuration
    Show,
    /// Edit config in $EDITOR
    Edit,
    /// Navigate to workspace folders (prints path for shell integration)
    Goto {
        #[arg(
            help = "Folder to navigate to: workspace|inbox|projects|areas|resources|archives|ctf"
        )]
        folder: String,
    },
}

#[derive(Subcommand)]
enum CtfCommands {
    /// Initialize a new CTF event
    Init {
        name: String,
        #[arg(long, help = "YYYY-MM-DD")]
        date: Option<String>,
        #[arg(long, help = "Start time (e.g. '2025-01-01' or '2025-01-01 10:00')")]
        start: Option<String>,
        #[arg(long, help = "End time (e.g. '2025-01-03 18:00')")]
        end: Option<String>,
    },
    /// List CTF events
    List,
    /// smart import a challenge archive
    Import {
        #[arg(help = "Path to challenge zip/tar")]
        file: PathBuf,
        #[arg(short, long, help = "Category (web, pwn, etc.)")]
        category: Option<String>,
        #[arg(short, long, help = "Challenge name (overrides filename)")]
        name: Option<String>,
        #[arg(long, help = "Skip all prompts, auto-infer everything")]
        auto: bool,
    },
    /// Solve a challenge (commit, flag, compress, archive)
    Solve {
        /// The flag value
        flag: String,
        /// Create a new challenge on the fly (format: <category>/<name>)
        #[arg(long, short = 'c')]
        create: Option<String>,
        /// Optional description/writeup to append to notes.md
        #[arg(long, short = 'd')]
        desc: Option<String>,
        /// Skip archiving the challenge after solving
        #[arg(long)]
        no_archive: bool,
        /// Skip git commit
        #[arg(long)]
        no_commit: bool,
    },
    /// Add a new challenge to current event (--cd to navigate)
    Add {
        #[arg(help = "Category/Name (e.g. pwn/stack-buffer)")]
        path: String,
        #[arg(long, help = "Output cd command for shell eval after creation")]
        cd: bool,
    },
    /// Generate writeup from notes
    Writeup,
    /// Archive an event to 4_Archives
    Archive {
        #[arg(help = "Name of the event to archive")]
        name: String,
    },
    /// Print path to CTF event or challenge (for shell integration)
    Path {
        #[arg(help = "Event name (optional, defaults to current context or latest)")]
        event: Option<String>,
        #[arg(help = "Challenge name (optional)")]
        challenge: Option<String>,
        #[arg(long, help = "Output as 'cd <path>' for eval in shell")]
        cd: bool,
    },
    /// Show current CTF context info
    Info,
    /// Set the current active event context globally
    Use {
        #[arg(help = "Event name/path to activate")]
        event: String,
    },
    /// Schedule or reschedule an event (add start/end times)
    Schedule {
        #[arg(help = "Event name (optional, uses active context otherwise)")]
        event: Option<String>,
        #[arg(long, help = "Start time (e.g. '2025-01-01 10:00')")]
        start: Option<String>,
        #[arg(long, help = "End time (e.g. '2025-01-03 18:00')")]
        end: Option<String>,
    },
    /// Clean up, commit, compress, and optionally archive an event
    Finish {
        #[arg(help = "Event name (optional, uses active context otherwise)")]
        event: Option<String>,
        #[arg(long, help = "Skip final archive step, just cleanup and commit")]
        no_archive: bool,
        #[arg(long, short, help = "Skip cleanup prompts (auto-confirm)")]
        force: bool,
        #[arg(long, help = "Show what would be cleaned without doing it")]
        dry_run: bool,
    },
    /// Check for expired or soon-to-expire events
    Check,
    /// Detailed status of challenges (Active vs Solved)
    Status,
    /// Shelve a challenge — interactive cleanup, flag, notes, and archive
    Shelve {
        /// Flag value (skips status and flag prompts if provided)
        flag: Option<String>,
        /// Add a note without prompting
        #[arg(long, short)]
        note: Option<String>,
        /// Skip file cleanup prompt
        #[arg(long)]
        no_clean: bool,
        /// Move to archives without prompting
        #[arg(long, short = 'm')]
        r#move: bool,
        /// Do not move to archives
        #[arg(long, short = 'M')]
        no_move: bool,
        /// Skip git commit
        #[arg(long)]
        no_commit: bool,
        /// Use smart defaults, skip all prompts
        #[arg(long)]
        auto: bool,
    },
    /// Alias for add --cd (deprecated, use add --cd instead)
    #[command(hide = true)]
    Work {
        #[arg(help = "Category/Name (e.g. pwn/stack-buffer)")]
        path: String,
    },
    /// Alias for shelve (deprecated, use shelve instead)
    #[command(hide = true)]
    Done {
        /// The flag value
        flag: String,
        /// Create a new challenge on the fly (format: <category>/<name>)
        #[arg(long, short = 'c')]
        create: Option<String>,
        /// Optional description/writeup
        #[arg(long, short = 'd')]
        desc: Option<String>,
        /// Skip archiving the challenge after solving
        #[arg(long)]
        no_archive: bool,
        /// Skip git commit
        #[arg(long)]
        no_commit: bool,
    },
}

fn parse_fuzzy_time(time_str: &str) -> Option<i64> {
    use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};

    // Try YYYY-MM-DD HH:MM
    if let Ok(dt) = NaiveDateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M") {
        return Local.from_local_datetime(&dt).single().map(|d| d.timestamp());
    }
    // Try YYYY-MM-DD
    if let Ok(d) = NaiveDate::parse_from_str(time_str, "%Y-%m-%d") {
        let dt = d.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        return Local.from_local_datetime(&dt).single().map(|d| d.timestamp());
    }

    None
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new project
    Init {
        #[arg(short, long, help = "Project type (rust, python, node)")]
        type_: String,
        #[arg(short, long, help = "Project name")]
        name: String,
    },
    /// Sort items from Inbox into Projects/Resources
    Clean {
        #[arg(long, help = "Simulate moves without executing")]
        dry_run: bool,
    },
    /// Manage CTF events
    Ctf {
        #[command(subcommand)]
        command: CtfCommands,
    },
    /// Audit workspace health (files, empty folders)
    Audit,
    /// Undo last movement operation
    Undo {
        #[arg(short, long, default_value_t = 1)]
        count: usize,
    },
    /// Watch Inbox and auto-sort
    Watch,
    /// Show git status dashboard
    Status,
    /// Search for flags recursively
    Search {
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(short, long)]
        pattern: Option<String>,
    },
    /// Fuzzy find projects
    Find { name: String },
    /// Grep Content in Projects/Resources
    Grep { pattern: String },
    /// Show workspace analytics
    Stats,
    /// Launch interactive TUI dashboard
    Dashboard,
    /// Quick file/project info
    Info { path: Option<PathBuf> },
    /// Manage configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Generate shell completion scripts
    Completions {
        #[arg(help = "Shell to generate completions for (bash, zsh)")]
        shell: Shell,
    },
}

/// Search for config file in priority order
fn find_config(cli_path: &Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = cli_path {
        if path.exists() {
            return Ok(path.clone());
        } else {
            anyhow::bail!("Config file not found: {:?}", path);
        }
    }

    let mut candidates = Vec::new();

    if let Some(config_dir) = dirs::config_dir() {
        candidates.push(config_dir.join("wardex/config.yaml"));
    }
    candidates.push(PathBuf::from("config.yaml"));

    for path in &candidates {
        if path.exists() {
            return Ok(path.clone());
        }
    }

    let searched: Vec<String> = candidates.iter().map(|p| format!("  - {:?}", p)).collect();
    anyhow::bail!(
        "Config file not found. Searched locations:\n{}\n\nUse --config <path> to specify a config file.",
        searched.join("\n")
    );
}

fn main() -> Result<()> {
    // Initialize logger with colored output
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp(None)
        .init();

    let cli = Cli::parse();

    // Completions command — no config needed
    if let Commands::Completions { shell } = &cli.command {
        let mut cmd = Cli::command();
        clap_complete::generate(*shell, &mut cmd, "wardex", &mut std::io::stdout());
        return Ok(());
    }

    // Check if we are initializing config (don't load it if so)
    if let Commands::Config {
        command: ConfigCommands::Init { .. },
    } = &cli.command
    {
        let config = Config::default(); // Dummy config
                                        // We still need to pass config to handle_config_command, but handle_config_command doesn't use it for Init
        if let Commands::Config { command } = &cli.command {
            handle_config_command(&config, command, cli.config.as_ref())?;
        }
        return Ok(());
    }

    let config = match find_config(&cli.config) {
        Ok(config_path) => Config::load_from_file(&config_path)?,
        Err(_) if matches!(&cli.command, Commands::Ctf { .. }) => {
            // CTF commands work without config — use defaults
            Config::default()
        }
        Err(e) => return Err(e),
    };

    match &cli.command {
        Commands::Init { type_, name } => {
            scaffold::init_project(&config, name, type_)?;
        }
        Commands::Clean { dry_run } => {
            let report = cleaner::clean_inbox(&config, *dry_run)?;
            output::display_clean_report(&config, &report);
        }
        Commands::Ctf { command } => {
            if !matches!(command, CtfCommands::Check | CtfCommands::List) {
                ctf::check_active_expiry(&config);
            }
            match command {
            CtfCommands::Init { name, date, start, end } => {
                let start_ts = start.as_deref().and_then(parse_fuzzy_time);
                let end_ts = end.as_deref().and_then(parse_fuzzy_time);

                if let Some(s) = start {
                    if start_ts.is_none() { log::warn!("Failed to parse start time '{}'", s); }
                }
                if let Some(e) = end {
                    if end_ts.is_none() { log::warn!("Failed to parse end time '{}'", e); }
                }

                let result = ctf::create_event(&config, name, date.clone(), start_ts, end_ts)?;

                if result.already_exists {
                    error!("Event directory already exists: {:?}", result.event_dir);
                } else {
                    info!("✓ Initialized: {:?}", result.event_dir);
                    info!("  + Categories: {}", result.categories_created.join(", "));
                    info!("  + File: notes.md");
                    info!("  + Metadata: .ctf_meta.json");
                }
            }
            CtfCommands::List => {
                let result = ctf::list_events(&config)?;

                if result.ctf_root_missing {
                    warn!("No CTF directory found.");
                    return Ok(());
                }

                if result.events.is_empty() {
                    warn!("No CTF events found.");
                    return Ok(());
                }

                println!(
                    "{:<30} {:<6} {:<12} {:<10}",
                    "Event", "Year", "Date", "Challenges"
                );
                println!("{}", "-".repeat(60));

                for event in &result.events {
                    let date_str = event.date.as_deref().unwrap_or("-");
                    let meta_indicator = if event.has_metadata { "" } else { "*" };
                    println!(
                        "{:<30} {:<6} {:<12} {:<10}{}",
                        event.name, event.year, date_str, event.challenge_count, meta_indicator
                    );
                }

                if result.events.iter().any(|e| !e.has_metadata) {
                    log::debug!("* Events without metadata file");
                }
            }
            CtfCommands::Import {
                file,
                category,
                name,
                auto,
            } => {
                ctf::import_challenge(&config, file, category.clone(), name.clone(), *auto)?;
            }
            CtfCommands::Solve { flag, create, desc, no_archive, no_commit } => {
                ctf::solve_challenge(&config, flag, create.clone(), desc.clone(), *no_archive, *no_commit)?;
            }
            CtfCommands::Add { path, cd } => {
                let challenge_dir = ctf::add_challenge(&config, path)?;
                if *cd {
                    println!("cd '{}'", challenge_dir.display());
                }
            }
            CtfCommands::Writeup => {
                ctf::generate_writeup(&config)?;
            }
            CtfCommands::Archive { name } => {
                ctf::archive_event(&config, name)?;
            }
            CtfCommands::Path { event, challenge, cd } => {
                let mut event = event.clone();
                let mut challenge = challenge.clone();

                if challenge.is_none() && event.as_ref().is_some_and(|e| e.contains('/')) {
                    challenge = event.clone();
                    event = None;
                }

                let path = ctf::get_event_path(&config, event.as_deref(), challenge.as_deref())?;
                if *cd {
                    println!("cd '{}'", path.display());
                } else {
                    println!("{}", path.display());
                }
            }
            CtfCommands::Info => {
                ctf::get_context_info(&config)?;
            }
            CtfCommands::Use { event } => {
                ctf::set_active_event(&config, event)?;
            }
            CtfCommands::Schedule { event, start, end } => {
                let start_ts = start.as_deref().and_then(parse_fuzzy_time);
                let end_ts = end.as_deref().and_then(parse_fuzzy_time);
                ctf::schedule_event(&config, event.as_deref(), start_ts, end_ts)?;
            }
            CtfCommands::Finish { event, no_archive, force, dry_run } => {
                ctf::finish_event(&config, event.as_deref(), *no_archive, *force, *dry_run)?;
            }
            CtfCommands::Check => {
                ctf::check_expiries(&config)?;
            }
            CtfCommands::Status => {
                ctf::challenge_status(&config)?;
            }
            CtfCommands::Shelve { flag, note, no_clean, r#move, no_move, no_commit, auto } => {
                ctf::shelve_challenge(
                    &config,
                    flag.clone(),
                    note.clone(),
                    *no_clean,
                    *r#move,
                    *no_move,
                    *no_commit,
                    *auto,
                )?;
            }
            CtfCommands::Work { path } => {
                let challenge_dir = ctf::add_challenge(&config, path)?;
                println!("cd '{}'", challenge_dir.display());
            }
            CtfCommands::Done { flag, create, desc, no_archive, no_commit } => {
                ctf::solve_challenge(&config, flag, create.clone(), desc.clone(), *no_archive, *no_commit)?;
            }
        }
        },
        Commands::Audit => {
            info!("Auditing workspace...");
            let report = auditor::audit_workspace(&config)?;
            output::display_audit_report(&config, &report);
        }
        Commands::Undo { count } => {
            let report = undo::undo_last(&config, *count)?;
            output::display_undo_report(&report);
        }
        Commands::Watch => {
            watcher::watch_inbox(&config)?;
        }
        Commands::Status => {
            info!("Scanning workspace: {:?}", config.resolve_path("workspace"));
            let report = status::show_status(&config)?;
            output::display_status_report(&config, &report);
        }
        Commands::Search { path, pattern } => {
            info!("Searching for flags in {:?}...", path);
            let report = search::find_flags(path, pattern.clone())?;
            output::display_search_report(&report);
        }
        Commands::Find { name } => {
            let results = search::find_project(&config, name)?;
            output::display_find_results(&results, name);
        }
        Commands::Grep { pattern } => {
            info!("Grepping in Projects & Resources...");
            let matches = search::content_search(&config, pattern)?;
            output::display_grep_results(&matches);
        }
        Commands::Stats => {
            let stats = stats::get_stats(&config)?;
            stats::print_stats(&stats);
        }
        Commands::Dashboard => {
            #[cfg(feature = "tui")]
            {
                tui::run(&config)?;
            }
            #[cfg(not(feature = "tui"))]
            {
                eprintln!("TUI dashboard is not enabled. Rebuild with: cargo install --features tui");
            }
        }
        Commands::Info { path } => {
            let target = path.clone().unwrap_or_else(|| PathBuf::from("."));
            info!("Info for: {:?}", target);
            if target.exists() {
                let meta = fs_err::metadata(&target)?;
                println!(
                    "Type: {:?}",
                    if target.is_dir() { "Directory" } else { "File" }
                );
                println!("Size: {} bytes", meta.len());
                println!("Modified: {:?}", meta.modified()?);
            } else {
                error!("Path not found");
            }
        }
        Commands::Config { command } => {
            handle_config_command(&config, command, cli.config.as_ref())?;
        }
        Commands::Completions { .. } => unreachable!("handled before config loading"),
    }

    Ok(())
}

fn handle_config_command(
    config: &Config,
    command: &ConfigCommands,
    explicit_config_path: Option<&PathBuf>,
) -> Result<()> {
    match command {
        ConfigCommands::Init { force } => {
            let config_file = if let Some(path) = explicit_config_path {
                path.clone()
            } else {
                let config_path = dirs::config_dir()
                    .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
                    .join("wardex");
                fs_err::create_dir_all(&config_path)?;
                config_path.join("config.yaml")
            };

            if config_file.exists() && !force {
                anyhow::bail!(
                    "Config file already exists at {:?}\n\n\
                    Use --force to overwrite",
                    config_file
                );
            }

            let default_config = wardex::core::templates::DEFAULT_CONFIG;

            fs_err::write(&config_file, default_config)?;
            println!("✓ Config initialized at: {:?}", config_file);
            println!("\nEdit with: wardex config edit");
        }

        ConfigCommands::Show => {
            println!("📋 Current Configuration\n");
            println!("Paths:");
            println!("  workspace:  {:?}", config.resolve_path("workspace"));
            println!("  inbox:      {:?}", config.resolve_path("inbox"));
            println!("  projects:   {:?}", config.resolve_path("projects"));
            println!("  areas:      {:?}", config.resolve_path("areas"));
            println!("  resources:  {:?}", config.resolve_path("resources"));
            println!("  archives:   {:?}", config.resolve_path("archives"));
            println!("  ctf_root:   {:?}", config.ctf_root());
        }

        ConfigCommands::Edit => {
            let config_file = dirs::config_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
                .join("wardex/config.yaml");

            if !config_file.exists() {
                anyhow::bail!(
                    "Config file not found at {:?}\n\n\
                    Initialize with: wardex config init",
                    config_file
                );
            }

            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());

            std::process::Command::new(&editor)
                .arg(&config_file)
                .status()?;

            println!("✓ Config edited. Changes will apply on next wardex command.");
        }

        ConfigCommands::Goto { folder } => {
            let path = match folder.as_str() {
                "workspace" => config.resolve_path("workspace"),
                "inbox" => config.resolve_path("inbox"),
                "projects" => config.resolve_path("projects"),
                "areas" => config.resolve_path("areas"),
                "resources" => config.resolve_path("resources"),
                "archives" => config.resolve_path("archives"),
                "ctf" => config.ctf_root(),
                _ => anyhow::bail!(
                    "Unknown folder: {}\n\n\
                    Available: workspace, inbox, projects, areas, resources, archives, ctf",
                    folder
                ),
            };

            if !path.exists() {
                eprintln!("Warning: Path does not exist: {:?}", path);
            }

            println!("{}", path.display());
        }
    }

    Ok(())
}
