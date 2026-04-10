use anyhow::{Context, Result};
use config::{Config as ConfigBuilder, Environment, File, FileFormat};
use regex::Regex;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Config {
    pub paths: Paths,
    #[serde(default)]
    pub rules: Rules,
    #[serde(default)]
    pub organize: Organize,
    #[serde(default)]
    pub ctf: CtfConfig,
}

/// Explicit path configuration
#[derive(Debug, Deserialize, Clone)]
pub struct Paths {
    pub workspace: PathBuf,
    pub inbox: Option<PathBuf>,
    pub projects: Option<PathBuf>,
    pub areas: Option<PathBuf>,
    pub resources: Option<PathBuf>,
    pub archives: Option<PathBuf>,
    /// Explicit CTF root path (optional, defaults to projects/CTFs)
    pub ctf_root: Option<PathBuf>,
}

impl Default for Paths {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            workspace: home.join("workspace"),
            inbox: None,
            projects: None,
            areas: None,
            resources: None,
            archives: None,
            ctf_root: None,
        }
    }
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct Rules {
    #[serde(default)]
    pub clean: Vec<CleanRule>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CleanRule {
    pub pattern: String,
    pub target: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Organize {
    pub ctf_dir: String,
}

impl Default for Organize {
    fn default() -> Self {
        Self {
            ctf_dir: "CTFs".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct CtfConfig {
    #[serde(default)]
    pub default_categories: Vec<String>,
    pub template_file: Option<String>,
    #[serde(default = "default_grace_period_hours")]
    pub grace_period_hours: u32,
    #[serde(default)]
    pub shelve: ShelveConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ShelveConfig {
    /// File/directory patterns to pre-select for deletion during triage.
    #[serde(default = "default_blacklist")]
    pub blacklist: Vec<String>,
    /// File/directory patterns to always keep (never shown in triage).
    #[serde(default = "default_whitelist")]
    pub whitelist: Vec<String>,
}

fn default_blacklist() -> Vec<String> {
    vec![
        "node_modules".into(),
        ".venv".into(),
        "venv".into(),
        "__pycache__".into(),
        ".gdb_history".into(),
        "peda-session-".into(),
        "core".into(),
        ".cache".into(),
    ]
}

fn default_whitelist() -> Vec<String> {
    vec![
        "solve.".into(),
        "exploit.".into(),
        "solution.".into(),
        "notes.md".into(),
        "README.md".into(),
        "Dockerfile".into(),
        "docker-compose".into(),
        ".challenge.json".into(),
        ".ctf_meta.json".into(),
        "flag".into(),
    ]
}

impl Default for ShelveConfig {
    fn default() -> Self {
        Self {
            blacklist: default_blacklist(),
            whitelist: default_whitelist(),
        }
    }
}

fn default_grace_period_hours() -> u32 {
    6
}

impl Default for CtfConfig {
    fn default() -> Self {
        Self {
            default_categories: vec![
                "pwn".to_string(),
                "web".to_string(),
                "crypto".to_string(),
                "rev".to_string(),
            ],
            template_file: None,
            grace_period_hours: 6,
            shelve: ShelveConfig::default(),
        }
    }
}

impl Config {
    /// Load configuration from multiple sources (layered):
    /// 1. Default config file in current directory
    /// 2. User config in ~/.config/wardex/
    /// 3. Environment variables with WX_ prefix
    pub fn load() -> Result<Self> {
        let mut builder = ConfigBuilder::builder();

        // 1. Current directory config.yaml
        builder = builder.add_source(File::new("config", FileFormat::Yaml).required(false));

        // 2. XDG config directory
        if let Some(config_dir) = dirs::config_dir() {
            let path = config_dir.join("wardex/config.yaml");
            if path.exists() {
                builder =
                    builder.add_source(File::from(path).format(FileFormat::Yaml).required(false));
            }
        }

        // 3. Environment variables with WX_ prefix
        // e.g., WX_PATHS_WORKSPACE=/tmp/workspace
        builder = builder.add_source(
            Environment::with_prefix("WX")
                .separator("_")
                .try_parsing(true),
        );

        let config = builder.build().context("Failed to build config")?;
        config
            .try_deserialize()
            .context("Failed to deserialize config")
    }

    /// Load from a specific file path with env var overrides (WX_*).
    pub fn load_from_file(path: &std::path::Path) -> Result<Self> {
        let builder = ConfigBuilder::builder()
            .add_source(File::from(path.to_path_buf()).format(FileFormat::Yaml))
            .add_source(
                Environment::with_prefix("WX")
                    .separator("_")
                    .try_parsing(true),
            );

        let config = builder.build().context("Failed to build config")?;
        config
            .try_deserialize()
            .context("Failed to deserialize config")
    }

    /// Resolve a path key to an absolute path.
    pub fn resolve_path(&self, key: &str) -> PathBuf {
        match key {
            "workspace" => self.paths.workspace.clone(),
            "inbox" => self
                .paths
                .inbox
                .clone()
                .unwrap_or_else(|| self.paths.workspace.join("0_Inbox")),
            "projects" => self
                .paths
                .projects
                .clone()
                .unwrap_or_else(|| self.paths.workspace.join("1_Projects")),
            "areas" => self
                .paths
                .areas
                .clone()
                .unwrap_or_else(|| self.paths.workspace.join("2_Areas")),
            "resources" => self
                .paths
                .resources
                .clone()
                .unwrap_or_else(|| self.paths.workspace.join("3_Resources")),
            "archives" => self
                .paths
                .archives
                .clone()
                .unwrap_or_else(|| self.paths.workspace.join("4_Archives")),
            "ctf_root" => self.ctf_root(),
            _ => {
                // Fallback: treat as relative path from projects
                self.resolve_path("projects").join(key)
            }
        }
    }

    /// Get the archive path for a CTF event: `{archives}/CTFs/{year}/{event_name}`
    pub fn ctf_archive_path(&self, year: &str, event_name: &str) -> PathBuf {
        self.resolve_path("archives")
            .join("CTFs")
            .join(year)
            .join(event_name)
    }

    /// Get the CTF root directory
    pub fn ctf_root(&self) -> PathBuf {
        self.paths
            .ctf_root
            .clone()
            .unwrap_or_else(|| self.resolve_path("projects").join("CTFs"))
    }

    /// Validate configuration values. Returns warnings for non-fatal issues.
    pub fn validate(&self) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        if !self.paths.workspace.exists() {
            warnings.push(format!(
                "Workspace path does not exist: {:?}. It will be created on first use.",
                self.paths.workspace
            ));
        }

        for cat in &self.ctf.default_categories {
            if !cat
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
            {
                anyhow::bail!(
                    "Invalid CTF category name: '{}'. Use alphanumeric characters, hyphens, or underscores.",
                    cat
                );
            }
        }

        for rule in &self.rules.clean {
            Regex::new(&rule.pattern).with_context(|| {
                format!(
                    "Invalid regex in clean rule: '{}'. Check your config file.",
                    rule.pattern
                )
            })?;
        }

        for pat in &self.ctf.shelve.blacklist {
            if pat.trim().is_empty() {
                warnings
                    .push("Empty pattern in ctf.shelve.blacklist — will be ignored.".to_string());
            }
        }
        for pat in &self.ctf.shelve.whitelist {
            if pat.trim().is_empty() {
                warnings
                    .push("Empty pattern in ctf.shelve.whitelist — will be ignored.".to_string());
            }
        }

        Ok(warnings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_config() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"
paths:
  workspace: /home/user/workspace
"#
        )
        .unwrap();
        file
    }

    #[test]
    fn test_load_from_file() {
        let file = create_test_config();
        let config = Config::load_from_file(file.path()).unwrap();

        assert_eq!(
            config.paths.workspace,
            PathBuf::from("/home/user/workspace")
        );
    }

    #[test]
    fn test_resolve_path_direct_keys() {
        let file = create_test_config();
        let config = Config::load_from_file(file.path()).unwrap();

        assert_eq!(
            config.resolve_path("workspace"),
            PathBuf::from("/home/user/workspace")
        );
        assert_eq!(
            config.resolve_path("inbox"),
            PathBuf::from("/home/user/workspace/0_Inbox")
        );
    }

    #[test]
    fn test_resolve_path_ctf_root() {
        let file = create_test_config();
        let config = Config::load_from_file(file.path()).unwrap();

        assert_eq!(
            config.resolve_path("ctf_root"),
            PathBuf::from("/home/user/workspace/1_Projects/CTFs")
        );
    }

    #[test]
    fn test_ctf_root_helper() {
        let file = create_test_config();
        let config = Config::load_from_file(file.path()).unwrap();

        assert_eq!(
            config.ctf_root(),
            PathBuf::from("/home/user/workspace/1_Projects/CTFs")
        );
    }
}
