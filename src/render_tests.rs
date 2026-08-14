use super::{HumanRenderOptions, render_human_at_width};
use crate::{
    cli::{ColorChoice, ProcessSort},
    model::{ProcessInfo, ProcessRecord, SCHEMA_VERSION, Scope, Snapshot, SystemInfo},
};

fn options() -> HumanRenderOptions {
    HumanRenderOptions {
        color: ColorChoice::Never,
        ascii: true,
        stdout_is_terminal: false,
        process_sort: ProcessSort::Cpu,
    }
}

#[test]
fn narrow_process_reports_stay_within_terminal_width() {
    let snapshot = Snapshot {
        schema_version: SCHEMA_VERSION,
        collected_at_unix_seconds: 1,
        scope: Scope::Processes,
        system: SystemInfo::default(),
        cpu: None,
        memory: None,
        storage: None,
        network: None,
        processes: Some(ProcessInfo {
            total_count: 1,
            processes: vec![ProcessRecord {
                pid: 42,
                name: "a-very-long-process-name".to_string(),
                executable: None,
                status: Some("Running".to_string()),
                cpu_percent: Some(12.3),
                resident_memory_bytes: Some(1024 * 1024),
                virtual_memory_bytes: Some(2 * 1024 * 1024),
            }],
        }),
        warnings: Vec::new(),
    };

    let output = render_human_at_width(&snapshot, options(), 40);

    assert!(output.lines().all(|line| line.chars().count() <= 40));
    assert!(output.contains("[Running]"));
}

#[test]
fn terminal_control_characters_are_neutralized() {
    let snapshot = Snapshot {
        schema_version: SCHEMA_VERSION,
        collected_at_unix_seconds: 1,
        scope: Scope::System,
        system: SystemInfo { hostname: Some("host\x1b[31m".to_string()), ..SystemInfo::default() },
        cpu: None,
        memory: None,
        storage: None,
        network: None,
        processes: None,
        warnings: Vec::new(),
    };

    let output = render_human_at_width(&snapshot, options(), 80);

    assert!(!output.contains('\x1b'));
    assert!(output.contains("host?[31m"));
}
