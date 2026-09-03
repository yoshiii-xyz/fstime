use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_fstime"))
}

fn temp_root(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("fstime-cli-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn version_and_help_are_available() {
    let version = Command::new(binary()).arg("--version").output().unwrap();
    assert!(version.status.success());
    assert!(String::from_utf8_lossy(&version.stdout).contains("fstime"));
    let help = Command::new(binary()).arg("--help").output().unwrap();
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("profile"));
}

#[test]
fn profile_compare_and_report_work() {
    let root = temp_root("flow");
    fs::write(root.join("data"), b"data").unwrap();
    let first = Command::new(binary())
        .args(["profile", root.to_str().unwrap(), "--cache-label", "cold"])
        .output()
        .unwrap();
    assert!(first.status.success());
    let first_json: Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(first_json["cache_label"], "cold");
    let first_path = root.join("cold.json");
    fs::write(&first_path, &first.stdout).unwrap();
    let second = Command::new(binary())
        .args(["profile", root.to_str().unwrap(), "--cache-label", "warm"])
        .output()
        .unwrap();
    assert!(second.status.success());
    let second_path = root.join("warm.json");
    fs::write(&second_path, &second.stdout).unwrap();
    let comparison = Command::new(binary())
        .args([
            "compare",
            first_path.to_str().unwrap(),
            second_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(comparison.status.success());
    let comparison_json: Value = serde_json::from_slice(&comparison.stdout).unwrap();
    assert_eq!(comparison_json["left_cache_label"], "cold");
    let explained = Command::new(binary())
        .args(["report", first_path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(explained.status.success());
    assert!(String::from_utf8_lossy(&explained.stdout).contains("stat calls:"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn missing_root_reports_one() {
    let root = temp_root("missing");
    let output = Command::new(binary())
        .args(["profile", root.join("missing").to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["complete"], false);
    fs::remove_dir_all(root).unwrap();
}
