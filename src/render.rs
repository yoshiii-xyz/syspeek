use std::{fmt::Display, fmt::Write as FmtWrite};

use crate::{
    cli::{ColorChoice, ProcessSort},
    model::{
        CpuInfo, LoadAverage, MemoryInfo, NetworkInfo, ProcessInfo, Snapshot, StorageInfo,
        SystemInfo,
    },
};

#[derive(Clone, Copy, Debug)]
pub struct HumanRenderOptions {
    pub color: ColorChoice,
    pub ascii: bool,
    pub stdout_is_terminal: bool,
    pub process_sort: ProcessSort,
}

pub fn render_json(snapshot: &Snapshot) -> serde_json::Result<String> {
    serde_json::to_string_pretty(snapshot).map(|text| format!("{text}\n"))
}

pub fn render_human(snapshot: &Snapshot, options: HumanRenderOptions) -> String {
    render_human_at_width(snapshot, options, terminal_width())
}

fn render_human_at_width(
    snapshot: &Snapshot,
    options: HumanRenderOptions,
    terminal_width: usize,
) -> String {
    let theme = Theme {
        color: match options.color {
            ColorChoice::Auto => options.stdout_is_terminal,
            ColorChoice::Always => true,
            ColorChoice::Never => false,
        },
        ascii: options.ascii,
    };
    let mut output = String::new();
    writeln!(
        output,
        "{}",
        theme.paint(
            format!("syspeek {}  scope: {}", env!("CARGO_PKG_VERSION"), snapshot.scope),
            "1;36",
        )
    )
    .expect("writing to a String cannot fail");

    render_system(&mut output, &theme, &snapshot.system);
    if let Some(cpu) = &snapshot.cpu {
        render_cpu(&mut output, &theme, cpu);
    }
    if let Some(memory) = &snapshot.memory {
        render_memory(&mut output, &theme, memory);
    }
    if let Some(storage) = &snapshot.storage {
        render_storage(&mut output, &theme, storage, terminal_width);
    }
    if let Some(network) = &snapshot.network {
        render_network(&mut output, &theme, network, terminal_width);
    }
    if let Some(processes) = &snapshot.processes {
        render_processes(&mut output, &theme, processes, options.process_sort, terminal_width);
    }
    if !snapshot.warnings.is_empty() {
        section(&mut output, &theme, "WARNINGS");
        for warning in &snapshot.warnings {
            writeln!(
                output,
                "  [{}] {}",
                theme.paint(&warning.section, "33"),
                sanitize_terminal(&warning.message)
            )
            .expect("writing to a String cannot fail");
        }
    }
    output
}

fn render_system(output: &mut String, theme: &Theme, system: &SystemInfo) {
    section(output, theme, "SYSTEM");
    pair(output, "Platform", optional(system.platform.as_ref()));
    pair(output, "Distribution", optional(system.distribution.as_ref()));
    pair(output, "OS version", optional(system.os_version.as_ref()));
    pair(output, "Kernel", optional(system.kernel_version.as_ref()));
    pair(output, "Architecture", optional(system.architecture.as_ref()));
    pair(output, "Hostname", optional(system.hostname.as_ref()));
    pair(
        output,
        "Uptime",
        system.uptime_seconds.map(format_duration).unwrap_or_else(|| "unavailable".to_string()),
    );
    pair(
        output,
        "Load average",
        system
            .load_average
            .as_ref()
            .map(format_load_average)
            .unwrap_or_else(|| "unavailable".to_string()),
    );
}

fn render_cpu(output: &mut String, theme: &Theme, cpu: &CpuInfo) {
    section(output, theme, "CPU");
    pair(output, "Model", optional(cpu.model.as_ref()));
    pair(output, "Architecture", optional(cpu.architecture.as_ref()));
    pair(output, "Topology", format_topology(cpu.physical_cores, cpu.logical_processors));
    pair(output, "Utilization", format_percent(cpu.utilization_percent));
    pair(
        output,
        "Average frequency",
        cpu.average_frequency_mhz
            .map(|frequency| format!("{frequency} MHz"))
            .unwrap_or_else(|| "unavailable".to_string()),
    );
    pair(
        output,
        "Load average",
        cpu.load_average
            .as_ref()
            .map(format_load_average)
            .unwrap_or_else(|| "unavailable".to_string()),
    );
}

fn render_memory(output: &mut String, theme: &Theme, memory: &MemoryInfo) {
    section(output, theme, "MEMORY");
    pair(
        output,
        "RAM",
        match (memory.used_bytes, memory.total_bytes) {
            (Some(used), Some(total)) => format!(
                "{} / {} ({})",
                format_bytes(used),
                format_bytes(total),
                format_percent(memory.utilization_percent)
            ),
            _ => "unavailable".to_string(),
        },
    );
    pair(
        output,
        "Available",
        memory.available_bytes.map(format_bytes).unwrap_or_else(|| "unavailable".to_string()),
    );
    if let Some(swap) = &memory.swap {
        pair(
            output,
            "Swap",
            match (swap.used_bytes, swap.total_bytes) {
                (Some(used), Some(total)) => format!(
                    "{} / {} ({})",
                    format_bytes(used),
                    format_bytes(total),
                    format_percent(swap.utilization_percent)
                ),
                _ => "unavailable".to_string(),
            },
        );
    }
}

fn render_storage(output: &mut String, theme: &Theme, storage: &StorageInfo, width: usize) {
    section(output, theme, "STORAGE");
    if storage.volumes.is_empty() {
        pair(output, "Volumes", "unavailable".to_string());
        return;
    }
    for volume in &storage.volumes {
        let filesystem = volume.filesystem.as_deref().unwrap_or("unknown");
        writeln!(
            output,
            "  {}  {}",
            theme.paint(
                format!(
                    "Mount {}",
                    truncate(&volume.mount_point, width.saturating_sub(42).max(16))
                ),
                "1",
            ),
            theme.paint(format!("({})", truncate(filesystem, 10)), "2")
        )
        .expect("writing to a String cannot fail");
        let capacity = match (volume.used_bytes, volume.total_bytes, volume.available_bytes) {
            (Some(used), Some(total), Some(free)) => format!(
                "Used {} / {}  Free {}",
                format_bytes(used),
                format_bytes(total),
                format_bytes(free)
            ),
            _ => "Capacity unavailable".to_string(),
        };
        writeln!(output, "    {capacity}").expect("writing to a String cannot fail");
        let mut flags = Vec::new();
        if volume.removable == Some(true) {
            flags.push("removable");
        }
        if volume.read_only == Some(true) {
            flags.push("read-only");
        }
        if !flags.is_empty() {
            writeln!(output, "    Flags: {}", flags.join(", "))
                .expect("writing to a String cannot fail");
        }
    }
}

fn render_network(output: &mut String, theme: &Theme, network: &NetworkInfo, width: usize) {
    section(output, theme, "NETWORK");
    if network.interfaces.is_empty() {
        pair(output, "Interfaces", "unavailable".to_string());
        return;
    }
    for interface in &network.interfaces {
        let addresses = if interface.addresses.is_empty() {
            "none".to_string()
        } else {
            interface
                .addresses
                .iter()
                .map(|address| format!("{}/{}", address.address, address.prefix_length))
                .collect::<Vec<_>>()
                .join(", ")
        };
        writeln!(
            output,
            "  {}  {}  MTU {}",
            theme.paint(truncate(&interface.name, width.saturating_sub(28).max(12)), "1"),
            truncate(interface.state.as_deref().unwrap_or("unknown"), 12),
            interface.mtu.map(|mtu| mtu.to_string()).unwrap_or_else(|| "n/a".to_string())
        )
        .expect("writing to a String cannot fail");
        let counters = match (interface.total_received_bytes, interface.total_transmitted_bytes) {
            (Some(received), Some(transmitted)) => {
                format!("RX {}  TX {}", format_bytes(received), format_bytes(transmitted))
            }
            _ => "Counters unavailable".to_string(),
        };
        writeln!(output, "    {counters}").expect("writing to a String cannot fail");
        writeln!(
            output,
            "    MAC {}",
            sanitize_terminal(interface.mac_address.as_deref().unwrap_or("unavailable")),
        )
        .expect("writing to a String cannot fail");
        writeln!(
            output,
            "    Addresses: {}",
            truncate(&addresses, width.saturating_sub(15).max(16))
        )
        .expect("writing to a String cannot fail");
    }
}

fn render_processes(
    output: &mut String,
    theme: &Theme,
    processes: &ProcessInfo,
    sort: ProcessSort,
    width: usize,
) {
    section(output, theme, "PROCESSES");
    let summary = format!(
        "Showing {} of {} processes, sorted by {}.",
        processes.processes.len(),
        processes.total_count,
        sort_label(sort)
    );
    writeln!(output, "  {}", truncate(&summary, width.saturating_sub(2).max(1)))
        .expect("writing to a String cannot fail");
    if processes.processes.is_empty() {
        return;
    }
    if width < 58 {
        render_processes_compact(output, theme, processes, width);
        return;
    }
    let name_width = width.saturating_sub(46).max(3);
    writeln!(
        output,
        "  {:>7} {:>8} {:>12} {:<12}  {}",
        theme.paint(format!("{:>7}", "PID"), "2"),
        theme.paint(format!("{:>8}", "CPU"), "2"),
        theme.paint(format!("{:>12}", "RESIDENT"), "2"),
        theme.paint(format!("{:<12}", "STATUS"), "2"),
        theme.paint("NAME", "2")
    )
    .expect("writing to a String cannot fail");
    for process in &processes.processes {
        writeln!(
            output,
            "  {:>7} {:>8} {:>12} {:<12}  {}",
            process.pid,
            process
                .cpu_percent
                .map(|value| format!("{value:.1}%"))
                .unwrap_or_else(|| "n/a".to_string()),
            process.resident_memory_bytes.map(format_bytes).unwrap_or_else(|| "n/a".to_string()),
            truncate(process.status.as_deref().unwrap_or("unknown"), 12),
            truncate(&process.name, name_width),
        )
        .expect("writing to a String cannot fail");
    }
}

fn render_processes_compact(
    output: &mut String,
    theme: &Theme,
    processes: &ProcessInfo,
    width: usize,
) {
    writeln!(
        output,
        "  {:>7} {:>8} {:>12}",
        theme.paint(format!("{:>7}", "PID"), "2"),
        theme.paint(format!("{:>8}", "CPU"), "2"),
        theme.paint(format!("{:>12}", "RESIDENT"), "2")
    )
    .expect("writing to a String cannot fail");
    for process in &processes.processes {
        writeln!(
            output,
            "  {:>7} {:>8} {:>12}",
            process.pid,
            process
                .cpu_percent
                .map(|value| format!("{value:.1}%"))
                .unwrap_or_else(|| "n/a".to_string()),
            process.resident_memory_bytes.map(format_bytes).unwrap_or_else(|| "n/a".to_string()),
        )
        .expect("writing to a String cannot fail");
        let details =
            format!("{} [{}]", process.name, process.status.as_deref().unwrap_or("unknown"));
        writeln!(output, "    {}", truncate(&details, width.saturating_sub(4).max(8)))
            .expect("writing to a String cannot fail");
    }
}

fn section(output: &mut String, theme: &Theme, title: &str) {
    let separator = if theme.ascii { "---" } else { "────" };
    writeln!(
        output,
        "\n{} {} {}",
        theme.paint(separator, "34"),
        theme.paint(title, "1;34"),
        theme.paint(separator, "34")
    )
    .expect("writing to a String cannot fail");
}

fn pair(output: &mut String, label: &str, value: String) {
    writeln!(output, "  {label:<18} {}", sanitize_terminal(&value))
        .expect("writing to a String cannot fail");
}

fn optional<T: Display>(value: Option<&T>) -> String {
    value.map(ToString::to_string).unwrap_or_else(|| "unavailable".to_string())
}

fn format_topology(physical: Option<usize>, logical: Option<usize>) -> String {
    match (physical, logical) {
        (Some(physical), Some(logical)) => format!("{physical} physical / {logical} logical"),
        (None, Some(logical)) => format!("{logical} logical, physical unavailable"),
        (Some(physical), None) => format!("{physical} physical, logical unavailable"),
        (None, None) => "unavailable".to_string(),
    }
}

fn format_load_average(load: &LoadAverage) -> String {
    format!("{:.2}, {:.2}, {:.2}", load.one_minute, load.five_minutes, load.fifteen_minutes)
}

fn format_percent(value: Option<f32>) -> String {
    value.map(|value| format!("{value:.1}%")).unwrap_or_else(|| "unavailable".to_string())
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 { format!("{bytes} B") } else { format!("{value:.1} {}", UNITS[unit]) }
}

fn format_duration(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    let remaining_seconds = seconds % 60;
    if days > 0 {
        format!("{days}d {hours}h {minutes}m {remaining_seconds}s")
    } else if hours > 0 {
        format!("{hours}h {minutes}m {remaining_seconds}s")
    } else if minutes > 0 {
        format!("{minutes}m {remaining_seconds}s")
    } else {
        format!("{remaining_seconds}s")
    }
}

fn sort_label(sort: ProcessSort) -> &'static str {
    match sort {
        ProcessSort::Cpu => "CPU",
        ProcessSort::Memory => "memory",
        ProcessSort::Pid => "PID",
        ProcessSort::Name => "name",
    }
}

fn truncate(value: &str, max_chars: usize) -> String {
    let value = sanitize_terminal(value);
    if value.chars().count() <= max_chars {
        return value;
    }
    let shortened: String = value.chars().take(max_chars.saturating_sub(3)).collect();
    format!("{shortened}...")
}

fn sanitize_terminal(value: &str) -> String {
    value.chars().map(|character| if character.is_control() { '?' } else { character }).collect()
}

fn terminal_width() -> usize {
    crossterm::terminal::size().map(|(width, _)| width as usize).unwrap_or(80).max(40)
}

struct Theme {
    color: bool,
    ascii: bool,
}

impl Theme {
    fn paint(&self, value: impl AsRef<str>, code: &str) -> String {
        let value = sanitize_terminal(value.as_ref());
        if self.color { format!("\x1b[{code}m{value}\x1b[0m") } else { value }
    }
}

#[cfg(test)]
#[path = "render_tests.rs"]
mod tests;
