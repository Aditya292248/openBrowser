use serde_json::Value;
use std::path::Path;
use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    let runtime_dir = unique_runtime_dir();
    run_with_runtime(args, &runtime_dir)
}

fn unique_runtime_dir() -> std::path::PathBuf {
    std::env::temp_dir().join(format!("my-browser-json-test-{}", uuid::Uuid::new_v4()))
}

fn run_with_runtime(args: &[&str], runtime_dir: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_my_browser"))
        .args(args)
        .env("MY_BROWSER_RUNTIME_DIR", runtime_dir)
        .output()
        .unwrap()
}

#[test]
fn json_mode_reports_a_successful_daemon_session() {
    let runtime_dir = unique_runtime_dir();
    let started = run_with_runtime(&["--json", "daemon", "start"], &runtime_dir);
    let closed = run_with_runtime(&["--json", "close"], &runtime_dir);

    assert_eq!(started.status.code(), Some(0));
    let record = json_output(&started);
    assert_eq!(record["ok"], true);
    assert_eq!(record["command"], "daemon start");
    assert_eq!(record["result"], "browser daemon started");

    assert_eq!(closed.status.code(), Some(0));
    let record = json_output(&closed);
    assert_eq!(record["ok"], true);
    assert_eq!(record["command"], "close");
    assert_eq!(record["result"], "browser closed; daemon stopped");
}

fn json_output(output: &Output) -> Value {
    assert!(output.stderr.is_empty());
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 1);
    serde_json::from_str(stdout).unwrap()
}

#[test]
fn json_help_is_one_success_record() {
    let output = run(&["--json", "--help"]);
    assert_eq!(output.status.code(), Some(0));
    let record = json_output(&output);
    assert_eq!(record["ok"], true);
    assert_eq!(record["command"], "help");
    assert!(record["result"].as_str().unwrap().contains("--json"));
}

#[test]
fn json_parse_and_execution_errors_have_distinct_exit_codes() {
    let invalid = run(&["--json", "fill", "#name"]);
    assert_eq!(invalid.status.code(), Some(2));
    let record = json_output(&invalid);
    assert_eq!(record["ok"], false);
    assert!(record["command"].is_null());
    assert_eq!(record["error"], "`fill` requires <text>");

    let unavailable = run(&["--json", "title"]);
    assert_eq!(unavailable.status.code(), Some(1));
    let record = json_output(&unavailable);
    assert_eq!(record["ok"], false);
    assert_eq!(record["command"], "title");
    assert!(record["error"]
        .as_str()
        .unwrap()
        .contains("daemon is not running"));
}

#[test]
fn json_mode_rejects_foreground_daemon_and_text_mode_still_uses_stderr() {
    let serve = run(&["--json", "daemon", "serve"]);
    assert_eq!(serve.status.code(), Some(2));
    let record = json_output(&serve);
    assert_eq!(record["command"], "daemon serve");
    assert!(record["error"]
        .as_str()
        .unwrap()
        .contains("cannot use --json"));

    let text = run(&["title"]);
    assert_eq!(text.status.code(), Some(1));
    assert!(text.stdout.is_empty());
    assert!(std::str::from_utf8(&text.stderr)
        .unwrap()
        .starts_with("error: daemon is not running"));
}
