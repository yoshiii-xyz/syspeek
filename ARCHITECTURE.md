# Architecture

<code>syspeek</code> uses a small pipeline with explicit boundaries:

~~~mermaid
flowchart TD
    INPUT[CLI arguments] --> CONFIG[Collection options]
    CONFIG --> COLLECTOR[Stateful collector]
    COLLECTOR --> SYSTEM[System metadata]
    COLLECTOR --> CPU[CPU and load]
    COLLECTOR --> MEMORY[RAM and swap]
    COLLECTOR --> STORAGE[Mounted volumes]
    COLLECTOR --> NETWORK[Local interfaces]
    COLLECTOR --> PROCESSES[Read-only process data]
    SYSTEM --> SNAPSHOT[Normalized Snapshot]
    CPU --> SNAPSHOT
    MEMORY --> SNAPSHOT
    STORAGE --> SNAPSHOT
    NETWORK --> SNAPSHOT
    PROCESSES --> SNAPSHOT
    SNAPSHOT --> HUMAN[Human renderer]
    SNAPSHOT --> JSON[JSON renderer]
    SNAPSHOT --> WATCH[Watch redraw loop]
~~~

The important design rule is that collectors never write terminal strings. They produce typed
values, and renderers decide how those values should look.

## Module ownership

| Module | Responsibility |
| --- | --- |
| <code>src/main.rs</code> | Process entry point, exit-code mapping, output selection |
| <code>src/cli.rs</code> | Clap parser, command hierarchy, duration and limit validation |
| <code>src/model.rs</code> | Stable normalized data model and JSON schema types |
| <code>src/collect.rs</code> | Scope-aware system collection and platform normalization |
| <code>src/render.rs</code> | Human-readable output, color policy, table layout, JSON serialization |
| <code>src/watch.rs</code> | Interactive redraw loop and terminal clear operations |
| <code>tests/</code> | CLI behavior, JSON contracts, and deterministic renderer coverage |
| <code>benches/</code> | Observational CPU and process collection benchmarks |

## Collection lifecycle

The CLI maps the command to a <code>Scope</code>. The scope determines which sysinfo refresh kinds
are enabled:

1. System metadata is available for every scope because it gives context to focused reports.
2. CPU and process scopes create one initial provider sample.
3. The first collection waits 200 milliseconds and refreshes again so usage values are based on a
   time delta rather than an uninitialized sample.
4. Memory, disks, and network interfaces are refreshed only when selected.
5. A <code>Snapshot</code> is assembled from normalized values and non-fatal warnings.
6. The snapshot is rendered as either terminal text or JSON.

The collector remains alive in watch mode. This lets later samples use the actual time between
refreshes and avoids rebuilding all provider state on every redraw.

## Normalization and availability

Operating systems expose similar concepts with different semantics. The model preserves that
boundary:

- Optional fields represent metrics that the provider could not report.
- Zero is retained when the provider reported a real zero, such as no swap configured.
- Memory uses the provider's available-memory definition. The README documents that definition as
  platform-dependent rather than relabeling it as free memory.
- CPU usage is a percentage of total system capacity. Process usage may exceed 100 percent on a
  multi-core machine because the underlying provider reports aggregate process usage.
- Disk usage is calculated as total capacity minus available capacity. The result is saturating so
  a provider with inconsistent counters cannot underflow.
- Network counters are cumulative totals from the provider, not a network scan or an interval rate.
- Paths and host-provided names are converted lossily to UTF-8 for a portable JSON boundary.

The schema has a top-level <code>schemaVersion</code>. Focused commands keep unselected top-level
sections as JSON <code>null</code> instead of silently collecting and returning data the caller did
not request.

## Error model

Collection APIs commonly return a value of zero or an empty list when a metric is unavailable.
<code>syspeek</code> does not turn every missing optional metric into a fatal error. Instead:

- A missing individual metric becomes <code>null</code> in JSON and <code>unavailable</code> in human
  output.
- An empty collection can add a warning for the relevant section.
- Provider-level unsupported targets add a system warning.
- Argument errors, incompatible output modes, and non-interactive watch mode exit with code 2.
- Output I/O failures and unexpected serialization failures exit with code 1.

This allows a report to remain useful in containers, restricted sessions, and hosts with changing
process or mount state.

## Process collection

Process collection intentionally uses a narrow refresh kind: CPU usage, resident and virtual
memory, and executable path when available. It does not request command lines, environment
variables, working directories, or process users. Tasks are disabled to avoid expanding a process
view into every thread on Linux.

Processes are copied into normalized records, sorted by the requested key with deterministic PID
tie-breaking, and truncated after collection. A process that exits during enumeration is simply
absent from that sample. No process control APIs are called.

## Terminal behavior

The renderer receives a typed snapshot and a small rendering policy. Automatic color is enabled only
when stdout is a terminal. Redirected output contains no ANSI control sequences. Watch mode is the
only path that emits terminal clear and cursor movement sequences, and it first verifies that
stdout is interactive.

The watch loop does not enable raw mode or alternate screen mode. That keeps Ctrl+C behavior
simple and avoids leaving terminal input settings behind after an interruption.

## Dependency choices

| Dependency | Reason |
| --- | --- |
| <code>sysinfo</code> | Mature native providers for Windows, Linux, and macOS system data |
| <code>clap</code> | Typed command parsing, help generation, version output, and consistent usage errors |
| <code>serde</code> and <code>serde_json</code> | Explicit, testable JSON schema without coupling it to terminal formatting |
| <code>crossterm</code> | Small cross-platform set of terminal clear and cursor operations for watch mode |

The runtime dependency set is intentionally limited. The project does not shell out to platform
commands because command names, localization, quoting, and permissions would make collection less
predictable.

## Deliberate boundaries

The first release does not include service control, sensors, GPU collection, battery data, remote
monitoring, configuration files, or plugin loading. Those features would need their own portability
and security contracts. The current model leaves room for additional optional sections without
making the first command slow or difficult to reason about.
