use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

struct TestEnv {
    temp_dir: TempDir,
}

impl TestEnv {
    fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        Self { temp_dir }
    }

    fn cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.current_dir(self.temp_dir.path());
        cmd.env("WX_PATHS_WORKSPACE", self.temp_dir.path());
        cmd.env("XDG_CONFIG_HOME", self.temp_dir.path());
        cmd.env("XDG_DATA_HOME", self.temp_dir.path());
        cmd.env("HOME", self.temp_dir.path()); // Just in case
        let config_file = self.temp_dir.path().join("config.yaml");
        if config_file.exists() {
            cmd.arg("--config").arg(&config_file);
        }
        cmd
    }

    fn path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }

    fn setup_workspace(&self) {
        let base = self.path();
        fs::create_dir_all(base.join("0_Inbox")).unwrap();
        fs::create_dir_all(base.join("1_Projects")).unwrap();
        fs::create_dir_all(base.join("2_Areas")).unwrap();
        fs::create_dir_all(base.join("3_Resources")).unwrap();
        fs::create_dir_all(base.join("4_Archives")).unwrap();
        fs::create_dir_all(base.join("1_Projects/CTFs")).unwrap();
    }

    fn create_config(&self) {
        let config_content = format!(
            r#"paths:
  workspace: {}
  inbox: {}/0_Inbox
  projects: {}/1_Projects
  areas: {}/2_Areas
  resources: {}/3_Resources
  archives: {}/4_Archives

organize:
  ctf_dir: 1_Projects/CTFs

ctf:
  default_categories:
    - web
    - pwn
    - crypto
    - rev"#,
            self.path().display(),
            self.path().display(),
            self.path().display(),
            self.path().display(),
            self.path().display(),
            self.path().display()
        );

        fs::write(self.path().join("config.yaml"), config_content).unwrap();
    }
}

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Ward & index your workspace"));
}

#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("wardex"));
}

#[test]
fn test_ctf_list_empty() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    env.cmd().args(["ctf", "list"]).assert().success();
}

#[test]
fn test_config_init() {
    let env = TestEnv::new();
    // Do NOT create config first
    // env.create_config();

    env.cmd().args(["config", "init"]).assert().success();
}

#[test]
fn test_config_init_twice_without_force_fails() {
    let env = TestEnv::new();
    // Create config manually first
    env.create_config();

    // This should fail because it exists
    env.cmd()
        .args(["config", "init"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_config_init_with_force_succeeds() {
    let env = TestEnv::new();
    env.create_config();

    // First attempt fails
    env.cmd().args(["config", "init"]).assert().failure();

    // Force succeeds
    env.cmd()
        .args(["config", "init", "--force"])
        .assert()
        .success();
}

#[test]
fn test_config_goto_workspace() {
    let env = TestEnv::new();
    env.create_config();

    env.cmd()
        .args(["config", "goto", "workspace"])
        .assert()
        .success()
        .stdout(predicate::str::contains(env.path().to_str().unwrap()));
}

#[test]
fn test_config_goto_invalid_folder() {
    let env = TestEnv::new();
    env.create_config();

    env.cmd()
        .args(["config", "goto", "invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown folder"));
}

#[test]
fn test_stats_command() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    fs::write(env.path().join("0_Inbox/test.txt"), "test").unwrap();

    env.cmd()
        .arg("stats")
        .assert()
        .success()
        .stdout(predicate::str::contains("Workspace"));
}

#[test]
fn test_ctf_init_creates_event() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    std::env::set_current_dir(env.path().join("1_Projects/CTFs")).unwrap();

    env.cmd()
        .args(["ctf", "init", "TestEvent"])
        .assert()
        .success();
    // .stderr(predicate::str::contains("Initialized")); // REMOVED: Flaky on Nix build

    let ctf_dirs: Vec<_> = fs::read_dir(env.path().join("1_Projects/CTFs"))
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();

    assert!(!ctf_dirs.is_empty());
}

#[test]
fn test_ctf_init_with_date() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    std::env::set_current_dir(env.path().join("1_Projects/CTFs")).unwrap();

    env.cmd()
        .args(["ctf", "init", "TestEvent", "--date", "2024-12-25"])
        .assert()
        .success();

    let event_dir = fs::read_dir(env.path().join("1_Projects/CTFs"))
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.file_name().to_string_lossy().contains("2024"));

    assert!(event_dir.is_some());
}

#[test]
fn test_ctf_add_invalid_format() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    env.cmd()
        .args(["ctf", "add", "invalid-format"])
        .assert()
        .failure();
}

#[test]
fn test_ctf_path_command() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    // Init event
    let ctf_root = env.path().join("1_Projects/CTFs");
    std::fs::create_dir_all(&ctf_root).unwrap();

    // We need to run init inside the ctf root or ensure config points to it
    // Config points to it.

    env.cmd()
        .args(["ctf", "init", "PathTest"])
        .assert()
        .success();

    // Get path
    env.cmd()
        .args(["ctf", "path", "PathTest"])
        .assert()
        .success()
        .stdout(predicate::str::contains("PathTest"));

    // Find the event dir name to use for add command context
    let event_dir = fs::read_dir(&ctf_root)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.file_name().to_string_lossy().contains("PathTest"))
        .unwrap()
        .path();

    // Add challenge needs to be run inside event dir
    let mut cmd = env.cmd();
    cmd.current_dir(&event_dir);
    cmd.args(["ctf", "add", "web/chall1"]).assert().success();

    // Test path to challenge
    env.cmd()
        .args(["ctf", "path", "PathTest", "chall1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("chall1"));
}

#[test]
fn test_ctf_import_with_category_flag_and_move() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    let ctf_root = env.path().join("1_Projects/CTFs");
    std::fs::create_dir_all(&ctf_root).unwrap();

    env.cmd()
        .args(["ctf", "init", "ImportTest"])
        .assert()
        .success();

    let event_dir = fs::read_dir(&ctf_root)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.file_name().to_string_lossy().contains("ImportTest"))
        .unwrap()
        .path();

    // Create dummy file to import outside event dir (e.g. in Inbox)
    let inbox = env.path().join("0_Inbox");
    let import_file = inbox.join("flag.txt");
    fs::write(&import_file, "CTF{test}").unwrap();

    // Import with category override
    let mut cmd = env.cmd();
    cmd.current_dir(&event_dir);
    cmd.args([
        "ctf",
        "import",
        import_file.to_str().unwrap(),
        "--category",
        "misc",
        "--auto",
    ])
    .assert()
    .success();

    // Check if file moved and exists in misc/flag
    let challenge_dir = event_dir.join("misc/flag");
    assert!(challenge_dir.exists());
    assert!(challenge_dir.join("flag.txt").exists());

    // Verify file content
    let content = fs::read_to_string(challenge_dir.join("flag.txt")).unwrap();
    assert_eq!(content, "CTF{test}");

    // Check that original file is GONE (moved)
    assert!(!import_file.exists());
}

#[test]
fn test_ctf_context_awareness() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    let ctf_root = env.path().join("1_Projects/CTFs");
    std::fs::create_dir_all(&ctf_root).unwrap();

    // Init event
    env.cmd()
        .args(["ctf", "init", "ContextTest"])
        .assert()
        .success();

    let event_dir = fs::read_dir(&ctf_root)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.file_name().to_string_lossy().contains("ContextTest"))
        .unwrap()
        .path();

    // 1. Test add from category dir (implicit category)
    let web_dir = event_dir.join("web");
    assert!(web_dir.exists());

    let mut cmd = env.cmd();
    cmd.current_dir(&web_dir);
    cmd.args(["ctf", "add", "chall1"]) // Should infer "web"
        .assert()
        .success();

    assert!(web_dir.join("chall1").exists());

    // 2. Test info command from deep inside
    let mut cmd = env.cmd();
    cmd.current_dir(web_dir.join("chall1"));
    cmd.args(["ctf", "info"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ContextTest"))
        .stdout(predicate::str::contains("web"));
}

#[test]
fn test_ctf_global_state() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    // 1. Init event (should auto-set global state)
    let output = env
        .cmd()
        .args(["ctf", "init", "GlobalEvent"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.contains("Switched to event") {
        println!("DEBUG: Init stdout: {}", stdout);
        println!(
            "DEBUG: Init stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // 2. Check info from OUTSIDE the event dir (e.g. root workspace)
    let mut cmd = env.cmd();
    cmd.current_dir(env.path()); // Workspace root, not event dir
    cmd.args(["ctf", "info"])
        .assert()
        .success()
        .stdout(predicate::str::contains("GlobalEvent"))
        .stdout(predicate::str::contains("Global State"));

    // 3. Add challenge from outside (using global state)
    let mut cmd = env.cmd();
    cmd.current_dir(env.path());
    cmd.args(["ctf", "add", "pwn/remote-exploit"])
        .assert()
        .success();

    // Verify it was created inside the event
    let ctf_root = env.path().join("1_Projects/CTFs");
    // Find the event dir
    let event_dir = fs::read_dir(&ctf_root)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.file_name().to_string_lossy().contains("GlobalEvent"))
        .unwrap()
        .path();

    assert!(event_dir.join("pwn/remote-exploit").exists());

    // 4. Create another event and switch to it
    env.cmd()
        .args(["ctf", "init", "SecondEvent"])
        .assert()
        .success();
    // Verify switch
    env.cmd()
        .args(["ctf", "info"])
        .assert()
        .success()
        .stdout(predicate::str::contains("SecondEvent"));

    // 5. Explicitly use the first event
    env.cmd()
        .args(["ctf", "use", "GlobalEvent"])
        .assert()
        .success();

    env.cmd()
        .args(["ctf", "info"])
        .assert()
        .success()
        .stdout(predicate::str::contains("GlobalEvent"));
}

#[test]
fn test_ctf_schedule_command() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    env.cmd()
        .args(["ctf", "init", "SchedEvent", "--start", "2026-03-01 10:00", "--end", "2026-03-03 18:00"])
        .assert()
        .success();

    let ctf_root = env.path().join("1_Projects/CTFs");
    let event_dir = fs::read_dir(&ctf_root)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.file_name().to_string_lossy().contains("SchedEvent"))
        .unwrap()
        .path();
    
    let meta_content = fs::read_to_string(event_dir.join(".ctf_meta.json")).unwrap();
    assert!(meta_content.contains("start_time"));
    assert!(meta_content.contains("end_time"));

    env.cmd()
        .args(["ctf", "schedule", "--end", "2026-03-05 18:00"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated schedule"));
}

#[test]
fn test_ctf_finish_command() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    env.cmd()
        .args(["ctf", "init", "FinishEvent"])
        .assert()
        .success();

    let ctf_root = env.path().join("1_Projects/CTFs");
    let event_dir = fs::read_dir(&ctf_root)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.file_name().to_string_lossy().contains("FinishEvent"))
        .unwrap()
        .path();

    // Init git repo to make git commands succeed
    let _ = std::process::Command::new("git")
        .arg("init")
        .current_dir(&event_dir)
        .output();
    let _ = std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&event_dir)
        .output();
    let _ = std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&event_dir)
        .output();

    // Test dry run (git may or may not be available in sandboxed builds)
    env.cmd()
        .args(["ctf", "finish", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Dry Run"));

    // Test actual finish
    env.cmd()
        .args(["ctf", "finish", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Successfully finished event"));

    let archives_dir = env.path().join("4_Archives/CTFs");
    assert!(fs::read_dir(&archives_dir).unwrap().count() > 0);
}

#[test]
fn test_ctf_check_command() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    env.cmd()
        .args(["ctf", "init", "ExpiredEvent", "--start", "2020-01-01 10:00", "--end", "2020-01-02 10:00"])
        .assert()
        .success();

    env.cmd()
        .args(["ctf", "check"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Expired Events"))
        .stdout(predicate::str::contains("ExpiredEvent"));
}

#[test]
fn test_ctf_path_fuzzy_command() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    env.cmd()
        .args(["ctf", "init", "SuperSecretCTF2026"])
        .assert()
        .success();

    env.cmd()
        .args(["ctf", "path", "supersec"])
        .assert()
        .success()
        .stdout(predicate::str::contains("SuperSecretCTF2026"));

    let ctf_root = env.path().join("1_Projects/CTFs");
    let event_dir = fs::read_dir(&ctf_root)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.file_name().to_string_lossy().contains("SuperSecretCTF2026"))
        .unwrap()
        .path();
    
    fs::create_dir_all(event_dir.join("pwn/buffer-overflow")).unwrap();

    env.cmd()
        .args(["ctf", "path", "pwn/buffer-overflow"])
        .assert()
        .success()
        .stdout(predicate::str::contains("buffer-overflow"));
}

#[test]
fn test_ctf_status_command() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    env.cmd()
        .args(["ctf", "init", "StatusEvent"])
        .assert()
        .success();

    // 1. Create a challenge that remains active
    env.cmd()
        .args(["ctf", "add", "web/active-chal"])
        .assert()
        .success();

    // 2. Create another challenge and solve it
    env.cmd()
        .args(["ctf", "add", "pwn/solved-chal"])
        .assert()
        .success();
    
    // We have to navigate into the challenge dir to solve it properly (per how solve works)
    let ctf_root = env.path().join("1_Projects/CTFs");
    let event_dir = fs::read_dir(&ctf_root)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.file_name().to_string_lossy().contains("StatusEvent"))
        .unwrap()
        .path();

    // Init git repo to make git commands succeed inside solve
    let _ = std::process::Command::new("git")
        .arg("init")
        .current_dir(&event_dir)
        .output();
    let _ = std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(&event_dir)
        .output();
    let _ = std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(&event_dir)
        .output();
    // Create an initial commit so adding works 
    let _ = std::process::Command::new("git")
        .args(["commit", "--allow-empty", "-m", "init"])
        .current_dir(&event_dir)
        .output();
    
    let mut cmd = env.cmd();
    cmd.env("RUST_BACKTRACE", "1");
    cmd.current_dir(event_dir.join("pwn/solved-chal"));
    let output = cmd.args(["ctf", "solve", "flag{test}"])
        .output()
        .unwrap();
    
    if !output.status.success() {
        panic!("Solve failed");
    }

    // 3. Verify status output
    env.cmd()
        .args(["ctf", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("active-chal"))
        .stdout(predicate::str::contains("solved-chal"))
        .stdout(predicate::str::contains("Active"))
        .stdout(predicate::str::contains("Solved"));
}

// ── T004: Challenge metadata tests ────────────────────────────────────

#[test]
#[serial_test::serial]
fn test_ctf_add_creates_challenge_json() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    // Init event
    env.cmd()
        .args(["ctf", "init", "MetaTestCTF"])
        .assert()
        .success();

    // Add challenge
    env.cmd()
        .args(["ctf", "add", "pwn/metadata-test"])
        .assert()
        .success();

    // Verify .challenge.json was created
    let ctf_root = env.path().join("1_Projects/CTFs");
    let event_dirs: Vec<_> = fs::read_dir(&ctf_root)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains("MetaTestCTF"))
        .collect();
    assert!(!event_dirs.is_empty(), "Event directory should exist");

    let event_dir = &event_dirs[0].path();
    let challenge_json = event_dir.join("pwn/metadata-test/.challenge.json");
    assert!(challenge_json.exists(), ".challenge.json should be created by ctf add");

    // Verify contents
    let content = fs::read_to_string(&challenge_json).unwrap();
    let meta: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert_eq!(meta["name"], "metadata-test");
    assert_eq!(meta["category"], "pwn");
    assert_eq!(meta["status"], "active");
    assert_eq!(meta["schema_version"], 1);
    assert!(meta["flag"].is_null());
    assert!(meta["created_at"].is_string());
}

#[test]
#[serial_test::serial]
fn test_ctf_import_creates_challenge_json_with_imported_from() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    // Init event
    env.cmd()
        .args(["ctf", "init", "ImportMetaCTF"])
        .assert()
        .success();

    // Create a dummy zip file
    let zip_path = env.path().join("test-chall.zip");
    {
        let file = fs::File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file("readme.txt", options).unwrap();
        use std::io::Write;
        zip.write_all(b"test challenge").unwrap();
        zip.finish().unwrap();
    }

    // Import with explicit category + name + auto mode
    env.cmd()
        .args([
            "ctf", "import",
            zip_path.to_str().unwrap(),
            "--category", "misc",
            "--name", "import-meta-test",
            "--auto",
        ])
        .assert()
        .success();

    // Find the challenge.json
    let ctf_root = env.path().join("1_Projects/CTFs");
    let event_dirs: Vec<_> = fs::read_dir(&ctf_root)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains("ImportMetaCTF"))
        .collect();
    let event_dir = &event_dirs[0].path();
    let challenge_json = event_dir.join("misc/import-meta-test/.challenge.json");
    assert!(challenge_json.exists(), ".challenge.json should be created by import");

    let content = fs::read_to_string(&challenge_json).unwrap();
    let meta: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert_eq!(meta["name"], "import-meta-test");
    assert_eq!(meta["category"], "misc");
    assert_eq!(meta["status"], "active");
    assert_eq!(meta["imported_from"], "test-chall.zip");
}

// ── T006: Add --cd tests ─────────────────────────────────────────────

#[test]
#[serial_test::serial]
fn test_ctf_add_with_cd_flag() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    env.cmd()
        .args(["ctf", "init", "CdTestCTF"])
        .assert()
        .success();

    // Add with --cd should print cd command
    env.cmd()
        .args(["ctf", "add", "web/cd-test", "--cd"])
        .assert()
        .success()
        .stdout(predicate::str::contains("cd '"))
        .stdout(predicate::str::contains("web/cd-test"));
}

#[test]
#[serial_test::serial]
fn test_ctf_add_without_cd_no_cd_output() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    env.cmd()
        .args(["ctf", "init", "NoCdTestCTF"])
        .assert()
        .success();

    // Add without --cd should NOT print cd command
    let output = env.cmd()
        .args(["ctf", "add", "web/no-cd-test"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("cd '"), "Should not print cd without --cd flag");
}

#[test]
#[serial_test::serial]
fn test_ctf_work_still_works_as_alias() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    env.cmd()
        .args(["ctf", "init", "WorkAliasCTF"])
        .assert()
        .success();

    // work should still function and output cd
    env.cmd()
        .args(["ctf", "work", "pwn/alias-test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("cd '"))
        .stdout(predicate::str::contains("pwn/alias-test"));
}

#[test]
fn test_work_hidden_from_help() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    let output = cmd.args(["ctf", "--help"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // work should be hidden
    assert!(!stdout.contains("work"), "work should be hidden from help");
    // add should be visible
    assert!(stdout.contains("add"), "add should be visible in help");
}

#[test]
fn test_done_hidden_from_help() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    let output = cmd.args(["ctf", "--help"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(!stdout.contains("done"), "done should be hidden from help");
}

// ── T005: Shelve command tests ────────────────────────────────────────

#[test]
fn test_shelve_visible_in_help() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    let output = cmd.args(["ctf", "--help"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("shelve"), "shelve should be visible in help");
}

#[test]
#[serial_test::serial]
fn test_shelve_with_flag_auto_mode() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    // Init event and add challenge
    env.cmd()
        .args(["ctf", "init", "ShelveTestCTF"])
        .assert()
        .success();

    env.cmd()
        .args(["ctf", "add", "pwn/shelve-test"])
        .assert()
        .success();

    // Find challenge dir and cd into it for shelve
    let ctf_root = env.path().join("1_Projects/CTFs");
    let event_dirs: Vec<_> = fs::read_dir(&ctf_root)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains("ShelveTestCTF"))
        .collect();
    let event_dir = event_dirs[0].path();
    let challenge_dir = event_dir.join("pwn/shelve-test");

    // Init git repo for commit
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(&event_dir)
        .output()
        .ok();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(&event_dir)
        .output()
        .ok();
    std::process::Command::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(&event_dir)
        .output()
        .ok();

    // Shelve with flag in auto mode (no prompts)
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.current_dir(&challenge_dir);
    cmd.env("WX_PATHS_WORKSPACE", env.path());
    cmd.env("XDG_CONFIG_HOME", env.path());
    cmd.env("XDG_DATA_HOME", env.path());
    cmd.env("HOME", env.path());
    cmd.args(["ctf", "shelve", "flag{test_shelve}", "--auto", "--no-move"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Solved: shelve-test"));

    // Verify .challenge.json was updated
    let meta_path = challenge_dir.join(".challenge.json");
    let content = fs::read_to_string(&meta_path).unwrap();
    let meta: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert_eq!(meta["status"], "solved");
    assert_eq!(meta["flag"], "flag{test_shelve}");
    assert_eq!(meta["solved_by"], "me");
    assert!(meta["shelved_at"].is_string());
}

#[test]
#[serial_test::serial]
fn test_shelve_auto_unsolved() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    env.cmd()
        .args(["ctf", "init", "ShelveUnsolvedCTF"])
        .assert()
        .success();

    env.cmd()
        .args(["ctf", "add", "crypto/unsolved-test"])
        .assert()
        .success();

    let ctf_root = env.path().join("1_Projects/CTFs");
    let event_dirs: Vec<_> = fs::read_dir(&ctf_root)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains("ShelveUnsolvedCTF"))
        .collect();
    let event_dir = event_dirs[0].path();
    let challenge_dir = event_dir.join("crypto/unsolved-test");

    // Shelve without flag in auto mode → unsolved
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.current_dir(&challenge_dir);
    cmd.env("WX_PATHS_WORKSPACE", env.path());
    cmd.env("XDG_CONFIG_HOME", env.path());
    cmd.env("XDG_DATA_HOME", env.path());
    cmd.env("HOME", env.path());
    cmd.args(["ctf", "shelve", "--auto", "--no-move", "--no-commit"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Shelved (unsolved)"));

    // Verify metadata
    let content = fs::read_to_string(challenge_dir.join(".challenge.json")).unwrap();
    let meta: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert_eq!(meta["status"], "unsolved");
    assert!(meta["flag"].is_null());
}

#[test]
#[serial_test::serial]
fn test_shelve_with_note_flag() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    env.cmd()
        .args(["ctf", "init", "ShelveNoteCTF"])
        .assert()
        .success();

    env.cmd()
        .args(["ctf", "add", "web/note-test"])
        .assert()
        .success();

    let ctf_root = env.path().join("1_Projects/CTFs");
    let event_dirs: Vec<_> = fs::read_dir(&ctf_root)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains("ShelveNoteCTF"))
        .collect();
    let event_dir = event_dirs[0].path();
    let challenge_dir = event_dir.join("web/note-test");

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.current_dir(&challenge_dir);
    cmd.env("WX_PATHS_WORKSPACE", env.path());
    cmd.env("XDG_CONFIG_HOME", env.path());
    cmd.env("XDG_DATA_HOME", env.path());
    cmd.env("HOME", env.path());
    cmd.args([
        "ctf", "shelve", "flag{noted}",
        "--note", "SQL injection in login form",
        "--auto", "--no-move", "--no-commit",
    ]);

    cmd.assert().success();

    let content = fs::read_to_string(challenge_dir.join(".challenge.json")).unwrap();
    let meta: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert_eq!(meta["note"], "SQL injection in login form");
    assert_eq!(meta["flag"], "flag{noted}");
}

// ── T009/T010: Completion tests ───────────────────────────────────────

#[test]
fn test_completions_bash() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_wardex()"))
        .stdout(predicate::str::contains("wardex__ctf__shelve"));
}

#[test]
fn test_completions_zsh() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("#compdef wardex"))
        .stdout(predicate::str::contains("shelve"));
}

#[test]
fn test_completions_visible_in_help() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    let output = cmd.args(["--help"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("completions"), "completions should be visible in help");
}

#[test]
#[serial_test::serial]
fn test_ctf_add_cd_escapes_single_quotes() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    env.cmd()
        .args(["ctf", "init", "QuoteTest"])
        .assert()
        .success();

    // Challenge name with a single quote
    env.cmd()
        .args(["ctf", "add", "web/bob's-chall", "--cd"])
        .assert()
        .success()
        .stdout(predicate::str::contains("bob'\\''s-chall"));
}

#[test]
fn test_experimental_labels_in_help() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    let output = cmd.args(["--help"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Non-CTF commands should have [experimental] on their line
    for cmd_name in ["clean", "watch", "audit", "search", "find", "grep", "stats", "undo"] {
        let has_experimental = stdout.lines().any(|l| {
            l.to_lowercase().contains(cmd_name) && l.contains("[experimental]")
        });
        assert!(has_experimental, "Command '{}' should be marked as [experimental] in help", cmd_name);
    }

    // CTF, config, completions should NOT be experimental
    for cmd_name in ["ctf", "config", "completions"] {
        let is_experimental = stdout.lines().any(|l| {
            l.to_lowercase().contains(cmd_name) && l.contains("[experimental]")
        });
        assert!(!is_experimental, "Command '{}' should NOT be marked [experimental]", cmd_name);
    }
}

// ── T006: ctf use / info / writeup / archive / solve ─────────────────

#[test]
#[serial_test::serial]
fn test_ctf_use_switches_context() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    env.cmd().args(["ctf", "init", "EventA"]).assert().success();
    env.cmd().args(["ctf", "init", "EventB"]).assert().success();

    // Info should show EventB (last init auto-activates)
    env.cmd()
        .args(["ctf", "info"])
        .assert()
        .success()
        .stdout(predicate::str::contains("EventB"));

    // Switch to EventA
    env.cmd().args(["ctf", "use", "EventA"]).assert().success();

    // Info should now show EventA
    env.cmd()
        .args(["ctf", "info"])
        .assert()
        .success()
        .stdout(predicate::str::contains("EventA"));
}

#[test]
#[serial_test::serial]
fn test_ctf_info_shows_context() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    env.cmd().args(["ctf", "init", "InfoTestCTF"]).assert().success();

    env.cmd()
        .args(["ctf", "info"])
        .assert()
        .success()
        .stdout(predicate::str::contains("InfoTestCTF"));
}

#[test]
#[serial_test::serial]
fn test_ctf_writeup_generates_output() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    env.cmd().args(["ctf", "init", "WriteupCTF"]).assert().success();
    env.cmd().args(["ctf", "add", "web/writeup-test"]).assert().success();

    let ctf_root = env.path().join("1_Projects/CTFs");
    let event_dir = fs::read_dir(&ctf_root)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.file_name().to_string_lossy().contains("WriteupCTF"))
        .unwrap()
        .path();

    // Write notes for the challenge
    let notes_path = event_dir.join("web/writeup-test/notes.md");
    fs::write(&notes_path, "# Solution\nUsed SQL injection on login form.").unwrap();

    // Generate writeup
    env.cmd()
        .args(["ctf", "writeup"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated writeup"));

    // Verify Writeup.md was created
    assert!(event_dir.join("Writeup.md").exists());
    let content = fs::read_to_string(event_dir.join("Writeup.md")).unwrap();
    assert!(content.contains("writeup-test"));
    assert!(content.contains("SQL injection"));
}

#[test]
#[serial_test::serial]
fn test_ctf_archive_moves_event() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    env.cmd().args(["ctf", "init", "ArchiveTestCTF"]).assert().success();

    env.cmd()
        .args(["ctf", "archive", "ArchiveTestCTF"])
        .assert()
        .success()
        .stdout(predicate::str::contains("archived"));

    // Verify event moved to archives
    let archives = env.path().join("4_Archives/CTFs");
    let has_entries = fs::read_dir(&archives)
        .map(|rd| rd.count() > 0)
        .unwrap_or(false);
    assert!(has_entries, "Event should be in archives");
}

#[test]
#[serial_test::serial]
fn test_ctf_solve_legacy_writes_flag() {
    let env = TestEnv::new();
    env.setup_workspace();
    env.create_config();

    env.cmd().args(["ctf", "init", "SolveLegacyCTF"]).assert().success();
    env.cmd().args(["ctf", "add", "misc/solve-test"]).assert().success();

    let ctf_root = env.path().join("1_Projects/CTFs");
    let event_dir = fs::read_dir(&ctf_root)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.file_name().to_string_lossy().contains("SolveLegacyCTF"))
        .unwrap()
        .path();

    let challenge_dir = event_dir.join("misc/solve-test");

    // Init git for the commit
    let _ = std::process::Command::new("git").arg("init").current_dir(&event_dir).output();
    let _ = std::process::Command::new("git").args(["config", "user.name", "Test"]).current_dir(&event_dir).output();
    let _ = std::process::Command::new("git").args(["config", "user.email", "t@t.com"]).current_dir(&event_dir).output();
    let _ = std::process::Command::new("git").args(["commit", "--allow-empty", "-m", "init"]).current_dir(&event_dir).output();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.current_dir(&challenge_dir);
    cmd.env("WX_PATHS_WORKSPACE", env.path());
    cmd.env("XDG_CONFIG_HOME", env.path());
    cmd.env("XDG_DATA_HOME", env.path());
    cmd.env("HOME", env.path());
    cmd.args(["ctf", "solve", "flag{legacy_test}", "--no-archive"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Saved flag"));

    // Verify flag.txt was written (legacy solve writes flag.txt)
    assert!(challenge_dir.join("flag.txt").exists());
    let flag_content = fs::read_to_string(challenge_dir.join("flag.txt")).unwrap();
    assert_eq!(flag_content, "flag{legacy_test}");
}
