<p align="center">
  <img src="assets/logo.png" alt="syspeek logo" width="160" />
</p>

<h1 align="center">syspeek</h1>

<p align="center">
  Fast, readable system inspection for developer machines.
</p>

<p align="center">
  <a href="https://github.com/joshiii-xyz/syspeek/actions/workflows/ci.yml">
    <img src="https://github.com/joshiii-xyz/syspeek/actions/workflows/ci.yml/badge.svg?branch=main" alt="CI status" />
  </a>
  <a href="https://www.rust-lang.org/">
    <img src="https://img.shields.io/badge/rust-1.95%2B-orange.svg" alt="Rust 1.95 or newer" />
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT license" />
  </a>
</p>

<code>syspeek</code> gives developers a concise snapshot of the machine they are working on. It
combines operating system details, CPU and memory metrics, mounted filesystem capacity, local
network interfaces, and a read-only top-process view in one command.

It is designed for fast local diagnosis, environment discovery, and terminal workflows. It is not
an enterprise monitoring system and does not attempt to collect remote telemetry.

## Why syspeek

System information is often split across platform-specific commands, each with different output
and different failure behavior. <code>syspeek</code> provides one stable interface while preserving
the important distinction between a metric that is zero and a metric that the operating system
cannot provide.

| Capability | What it provides |
| --- | --- |
| System identity | Platform, distribution, OS version, kernel version, architecture, hostname, uptime |
| CPU | Model, physical and logical topology, sampled utilization, average frequency |
| Memory | RAM used, available, total, utilization, and swap statistics |
| Storage | Mounted volumes, filesystem, capacity, free space, removable and read-only flags |
| Network | Local interfaces, operational state, addresses, MAC address, MTU, byte counters |
| Processes | PID, name, status, CPU usage, resident memory, virtual memory, optional executable path |
| Output | Human-readable terminal output, structured JSON, and interactive watch mode |

## Install

The project is currently distributed from source. A Rust toolchain with Rust 1.95 or newer is
required.

Install a checkout as a local command:

~~~console
cargo install --path . --locked
syspeek --help
~~~

Build an optimized binary without installing it:

~~~console
cargo build --release --locked
target/release/syspeek
~~~

On Windows, the release binary is written to <code>target\release\syspeek.exe</code>.

## Quick start

Run the complete snapshot:

~~~console
syspeek
~~~

Inspect one area:

~~~console
syspeek cpu
syspeek memory
syspeek disk
syspeek network
syspeek processes --sort memory --limit 15
~~~

Use JSON in scripts:

~~~console
syspeek --json | jq '.memory, .processes.processes[0]'
~~~

The default command collects all sections. Focused commands skip unrelated collectors, which keeps
commands such as <code>syspeek cpu</code> quick and avoids enumerating processes when they are not
requested.

## Example output

The exact values depend on the host. The following abbreviated report shows the layout:

~~~text
syspeek 0.1.0  scope: all

--- SYSTEM ---
  Platform           Linux
  Distribution       Ubuntu 24.04
  OS version         Linux (Ubuntu 24.04)
  Kernel             Linux 6.8.0
  Architecture       x86_64
  Hostname           dev-host
  Uptime             2d 4h 18m 9s
  Load average       0.42, 0.38, 0.31

--- CPU ---
  Model              12th Gen Intel(R) Core(TM) i7
  Topology           8 physical / 12 logical
  Utilization        14.7%
  Average frequency  2,411 MHz

--- MEMORY ---
  RAM                9.8 GiB / 31.2 GiB (31.4%)
  Available          21.4 GiB
  Swap               0 B / 8.0 GiB (0.0%)

--- PROCESSES ---
  Showing 10 of 326 processes, sorted by CPU.
      PID      CPU     RESIDENT STATUS        NAME
     4120    12.4%    482.0 MiB Run           code
     8211     4.8%    118.3 MiB Sleep         cargo
~~~

Use <code>--ascii</code> when a terminal cannot render Unicode section separators. Color is enabled
only when stdout is a terminal by default:

~~~console
syspeek --ascii --color never
~~~

## Command reference

### <code>syspeek</code>

Collects the full snapshot with system, CPU, memory, storage, network, and process sections.

### <code>syspeek system</code>

Shows host identity and operating system information. This command does not enumerate disks,
interfaces, or processes.

### <code>syspeek cpu</code>

Shows CPU model, architecture, physical cores, logical processors, sampled utilization, average
frequency, and load average where the platform reports one. CPU utilization is sampled over the
minimum interval needed by the underlying provider, currently 200 milliseconds.

### <code>syspeek memory</code>

Shows RAM total, used, available, and utilization. Swap fields are reported separately. A zero
swap total represents no configured swap when the platform exposes swap statistics.

### <code>syspeek disk</code>

Shows mounted volumes returned by the platform provider, including capacity, free space, filesystem,
removable status, and read-only status. <code>syspeek</code> does not recursively scan filesystems.

### <code>syspeek network</code>

Shows local interfaces only. It does not probe hosts, resolve services, scan ports, or make network
requests. Address and MAC visibility depends on operating-system permissions and provider support.

### <code>syspeek processes</code>

Shows a read-only process table. Use <code>--limit</code> to cap output and
<code>--sort cpu</code>, <code>--sort memory</code>, <code>--sort pid</code>, or
<code>--sort name</code> to select ordering:

~~~console
syspeek processes --sort cpu --limit 20
syspeek processes --sort memory --limit 10 --json
~~~

The collector reads process metadata, CPU usage, and memory usage. It does not read process
environment variables or command lines, and it never terminates or modifies a process.

## Options

| Option | Description |
| --- | --- |
| <code>-j</code>, <code>--json</code> | Emit the stable JSON document instead of terminal formatting |
| <code>--watch</code> | Refresh the human-readable report until interrupted |
| <code>--interval DURATION</code> | Watch delay, such as <code>500ms</code>, <code>2s</code>, or <code>1m</code>; minimum <code>200ms</code> |
| <code>--color auto\|always\|never</code> | Select ANSI color behavior |
| <code>--ascii</code> | Use ASCII separators |
| <code>-h</code>, <code>--help</code> | Show help |
| <code>-V</code>, <code>--version</code> | Show the package version |

<code>--watch</code> and <code>--json</code> are intentionally incompatible. JSON is a complete
snapshot for automation, while watch mode owns the terminal and redraws the current report.

## JSON output

JSON is a schema, not a serialization of terminal strings. Field names use camelCase and the
document includes <code>schemaVersion</code> for future compatibility:

~~~json
{
  "schemaVersion": 1,
  "collectedAtUnixSeconds": 1786732223,
  "scope": "cpu",
  "system": {
    "platform": "Linux",
    "architecture": "x86_64",
    "uptimeSeconds": 8804,
    "loadAverage": {
      "oneMinute": 0.42,
      "fiveMinutes": 0.38,
      "fifteenMinutes": 0.31
    }
  },
  "cpu": {
    "model": "12th Gen Intel(R) Core(TM) i7",
    "physicalCores": 8,
    "logicalProcessors": 12,
    "utilizationPercent": 14.7,
    "averageFrequencyMhz": 2411
  },
  "memory": null,
  "storage": null,
  "network": null,
  "processes": null,
  "warnings": []
}
~~~

The top-level scope identifies what was requested. <code>system</code> is always present because
it provides context for every report. Other top-level sections are <code>null</code> when they were
not collected. Within a collected section, an individual <code>null</code> means the provider could
not report that metric. Numeric zero remains a real zero.

Consumers should tolerate additional fields in compatible releases and use
<code>schemaVersion</code> when validating breaking changes. Warnings are non-fatal collection
notes and do not normally change the process exit status.

## Live monitoring

Watch mode reuses the collector and refreshes the selected scope at a configurable interval:

~~~console
syspeek --watch
syspeek processes --watch --interval 1s --limit 12
~~~

The renderer clears and redraws the current terminal view. It does not enable raw mode or alternate
screen mode, so Ctrl+C leaves normal terminal input settings intact. Watch mode requires an
interactive stdout terminal. Redirected and CI environments should use a normal snapshot.

## Platform support

Windows, Linux, and macOS are first-class targets. The collection provider reports only data that
the operating system and the current permissions expose.

| Area | Windows | Linux | macOS |
| --- | --- | --- | --- |
| System identity | Native provider | Native provider, including distribution when available | Native provider |
| Load average | Not reported by the provider | Reported where supported | Reported where supported |
| Memory | Windows available-memory semantics | Available-memory semantics from the provider | Native memory semantics |
| Storage | Mounted volumes | Useful mounts, with provider defaults for virtual and network filesystems | Mounted volumes |
| Network | Local adapter state and counters | Local interface state and counters | Local interface state and counters |
| Processes | Permission-sensitive metadata | Permission-sensitive process data | Privacy-sensitive process data |

Containers, WSL, virtual machines, removable drives, disconnected interfaces, and rapidly changing
process lists can expose platform-specific behavior. Missing data is represented as
<code>null</code> in JSON or <code>unavailable</code> in terminal output. <code>syspeek</code> does
not invent values to make the platforms look identical.

Temperature sensors, GPUs, services, battery data, and container detection are intentionally
outside the first release. They can be added as focused collectors when reliable cross-platform
semantics are available.

## Architecture

The implementation separates collection, normalized data, and presentation:

~~~mermaid
flowchart LR
    CLI[CLI input] --> PLAN[Scope and refresh plan]
    PLAN --> COLLECT[sysinfo collectors]
    COLLECT --> MODEL[Normalized Snapshot]
    MODEL --> HUMAN[Terminal renderer]
    MODEL --> JSON[JSON renderer]
    MODEL --> WATCH[Watch loop]
    COLLECT --> WARN[Non-fatal warnings]
    WARN --> MODEL
~~~

See [ARCHITECTURE.md](ARCHITECTURE.md) for module ownership, refresh behavior, error semantics,
and engineering tradeoffs.

## Performance

Focused commands avoid work outside their scope. The default process view enumerates once and
sorts in memory, then limits the displayed records. CPU and process utilization use the provider's
two-sample model, with a 200 millisecond initial sample interval. Watch mode reuses its collector
between refreshes, so subsequent utilization values use the elapsed watch interval.

The benchmark suite is intentionally observational. It does not fail on absolute timing:

~~~console
cargo bench --bench snapshot --locked
~~~

Run it on representative hosts if a change affects process enumeration, storage discovery, or
rendering overhead.

## Security and privacy

<code>syspeek</code> is a local, read-only diagnostic utility:

- It does not write files, change permissions, alter network configuration, or modify processes.
- It does not terminate, inject into, or signal processes.
- It does not execute shell commands or invoke external programs to gather metrics.
- It does not read process environments or command lines.
- It does not collect credentials, tokens, private keys, or arbitrary environment variables.
- It reports local paths, hostnames, interface addresses, and MAC addresses only when the selected
  provider exposes them through the requested system section.

Treat terminal output and JSON as local machine information. Process paths and network addresses
can be sensitive in shared logs. See [SECURITY.md](SECURITY.md) for reporting guidance.

## Development

The project needs Rust 1.95 or newer. From a checkout:

~~~console
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --locked
cargo build --release --locked
~~~

Use <code>cargo run -- --help</code> to exercise the CLI from a checkout. See
[CONTRIBUTING.md](CONTRIBUTING.md) for the expected workflow and platform testing notes.

## Exit codes

| Code | Meaning |
| ---: | --- |
| <code>0</code> | Snapshot or watch session completed successfully |
| <code>1</code> | Runtime output failure or unexpected fatal error |
| <code>2</code> | Invalid arguments, incompatible options, or an unavailable interactive watch terminal |

Unavailable individual metrics are non-fatal. The command returns success when it can produce a
valid snapshot with partial data.

## Project status

<code>syspeek</code> is in initial public development. The compatibility boundary for automation is
the versioned JSON schema and documented CLI behavior. See [CHANGELOG.md](CHANGELOG.md) for the
release-notes strategy.

## License

Distributed under the [MIT License](LICENSE).
