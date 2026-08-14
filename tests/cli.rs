use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;

fn syspeek() -> Command {
    Command::cargo_bin("syspeek").expect("binary should be built by Cargo")
}

#[test]
fn help_describes_the_main_workflows() {
    syspeek()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Inspect the current machine"))
        .stdout(predicate::str::contains("processes"));
}

#[test]
fn version_matches_package_metadata() {
    syspeek().arg("--version").assert().success().stdout(predicate::str::contains("syspeek 0.1.0"));
}

#[test]
fn default_json_is_valid_and_contains_the_full_snapshot() {
    let output = syspeek().arg("--json").output().expect("command should run");
    assert!(output.status.success());
    let document: Value = serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    assert_eq!(document["schemaVersion"], 1);
    assert_eq!(document["scope"], "all");
    assert!(document["system"].is_object());
    assert!(document["cpu"].is_object());
    assert!(document["memory"].is_object());
    assert!(document["storage"].is_object());
    assert!(document["network"].is_object());
    assert!(document["processes"].is_object());
}

#[test]
fn focused_commands_leave_unrequested_sections_null() {
    let output = syspeek().args(["cpu", "--json"]).output().expect("command should run");
    assert!(output.status.success());
    let document: Value = serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    assert_eq!(document["scope"], "cpu");
    assert!(document["cpu"].is_object());
    assert!(document["memory"].is_null());
    assert!(document["storage"].is_null());
    assert!(document["network"].is_null());
    assert!(document["processes"].is_null());
}

#[test]
fn process_limit_is_respected() {
    let output = syspeek()
        .args(["processes", "--limit", "2", "--sort", "memory", "--json"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let document: Value = serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    let processes =
        document["processes"]["processes"].as_array().expect("process list should be an array");
    assert!(processes.len() <= 2);
}

#[test]
fn process_sorting_is_descending_for_memory() {
    let output = syspeek()
        .args(["processes", "--limit", "10", "--sort", "memory", "--json"])
        .output()
        .expect("command should run");
    assert!(output.status.success());
    let document: Value = serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
    let processes =
        document["processes"]["processes"].as_array().expect("process list should be an array");
    for pair in processes.windows(2) {
        let current =
            pair[0]["residentMemoryBytes"].as_u64().expect("resident memory should be numeric");
        let next =
            pair[1]["residentMemoryBytes"].as_u64().expect("resident memory should be numeric");
        assert!(current >= next);
    }
}

#[test]
fn invalid_arguments_use_usage_exit_code() {
    syspeek()
        .args(["--interval", "10ms"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("watch interval"));
}

#[test]
fn watch_json_combination_is_rejected_before_terminal_checks() {
    syspeek()
        .args(["--watch", "--json"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("cannot be combined"));
}

#[test]
fn zero_process_limit_is_rejected() {
    syspeek()
        .args(["processes", "--limit", "0"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("greater than zero"));
}

#[test]
fn watch_mode_requires_interactive_stdout() {
    syspeek()
        .arg("--watch")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("interactive terminal"));
}

#[test]
fn focused_commands_select_only_the_requested_sections() {
    let cases = [
        (&["system", "--json"][..], "system", "system"),
        (&["memory", "--json"][..], "memory", "memory"),
        (&["disk", "--json"][..], "disk", "storage"),
        (&["network", "--json"][..], "network", "network"),
        (&["processes", "--limit", "1", "--json"][..], "processes", "processes"),
    ];
    let sections = ["cpu", "memory", "storage", "network", "processes"];

    for (args, expected_scope, selected_section) in cases {
        let output = syspeek().args(args).output().expect("command should run");
        assert!(output.status.success(), "command failed for {args:?}");
        let document: Value =
            serde_json::from_slice(&output.stdout).expect("stdout should be JSON");
        assert_eq!(document["scope"], expected_scope);
        assert!(document[selected_section].is_object());
        for section in sections {
            if section != selected_section {
                assert!(document[section].is_null(), "section {section} was collected");
            }
        }
    }
}
