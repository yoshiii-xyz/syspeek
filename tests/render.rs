use syspeek::{
    cli::{ColorChoice, ProcessSort},
    model::{CpuInfo, LoadAverage, MemoryInfo, SCHEMA_VERSION, Scope, Snapshot, SystemInfo},
    render::{HumanRenderOptions, render_human, render_json},
};

fn fixture() -> Snapshot {
    Snapshot {
        schema_version: SCHEMA_VERSION,
        collected_at_unix_seconds: 1_700_000_000,
        scope: Scope::Cpu,
        system: SystemInfo {
            platform: Some("Test OS".to_string()),
            distribution: None,
            os_version: Some("1.2".to_string()),
            kernel_version: Some("kernel 1".to_string()),
            architecture: Some("x86_64".to_string()),
            hostname: Some("fixture".to_string()),
            uptime_seconds: Some(3_661),
            boot_time_unix_seconds: None,
            load_average: Some(LoadAverage {
                one_minute: 0.25,
                five_minutes: 0.50,
                fifteen_minutes: 0.75,
            }),
        },
        cpu: Some(CpuInfo {
            model: Some("Fixture CPU".to_string()),
            architecture: Some("x86_64".to_string()),
            physical_cores: Some(4),
            logical_processors: Some(8),
            utilization_percent: Some(12.5),
            average_frequency_mhz: Some(3_200),
            load_average: None,
        }),
        memory: Some(MemoryInfo {
            total_bytes: Some(8 * 1024 * 1024 * 1024),
            used_bytes: Some(2 * 1024 * 1024 * 1024),
            available_bytes: Some(6 * 1024 * 1024 * 1024),
            utilization_percent: Some(25.0),
            swap: None,
        }),
        storage: None,
        network: None,
        processes: None,
        warnings: Vec::new(),
    }
}

#[test]
fn human_output_is_readable_without_terminal_control_sequences() {
    let output = render_human(
        &fixture(),
        HumanRenderOptions {
            color: ColorChoice::Auto,
            ascii: true,
            stdout_is_terminal: false,
            process_sort: ProcessSort::Cpu,
        },
    );
    assert!(output.contains("syspeek 0.1.0  scope: cpu"));
    assert!(output.contains("SYSTEM"));
    assert!(output.contains("CPU"));
    assert!(!output.contains('\x1b'));
}

#[test]
fn json_output_preserves_null_for_uncollected_sections() {
    let value: serde_json::Value =
        serde_json::from_str(&render_json(&fixture()).expect("fixture should serialize"))
            .expect("fixture JSON should parse");
    assert_eq!(value["schemaVersion"], 1);
    assert!(value["cpu"].is_object());
    assert!(value["storage"].is_null());
    assert!(value["processes"].is_null());
}
