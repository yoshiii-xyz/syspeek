# Contributing

Thanks for helping improve <code>syspeek</code>. Keep changes focused, portable, and honest about
what each operating system can report.

## Development setup

Install Rust 1.95 or newer, clone the repository, and run the checks below from the repository
root:

~~~console
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
cargo build --release
~~~

Exercise the binary directly while developing:

~~~console
cargo run -- --help
cargo run -- --json
cargo run -- processes --sort memory --limit 10
~~~

The project uses no runtime configuration files and does not need a privileged account for normal
development.

## Code guidelines

- Keep collection code independent from terminal layout.
- Prefer typed optional fields over sentinel strings or fabricated values.
- Do not read credentials, environment variables, process command lines, or unrelated files.
- Avoid shelling out when a mature native API is available.
- Preserve deterministic ordering for lists and process tie-breaks.
- Document platform-specific behavior in the README or architecture document.
- Keep user-facing text professional and free of decorative symbols.

## Tests

CLI tests should validate behavior without depending on exact host metrics. Renderer tests should
use deterministic fixtures. Collector behavior can be exercised through integration tests when the
assertion is resilient to changing process lists, mounts, interfaces, and permissions.

When changing the JSON model, update the tests that cover schema version, focused scope, and null
availability semantics. When changing terminal layout, update deterministic renderer coverage and
review the output manually.

## Platform testing

The CI matrix covers Windows, Linux, and macOS. When a change touches a provider-specific behavior,
test on the affected operating system if possible. Pay particular attention to:

- Permission-limited process data.
- Containers and WSL.
- Removable or read-only volumes.
- Interfaces without addresses.
- Hosts without swap or load-average support.
- Processes that exit while a snapshot is being collected.

Do not make CI depend on the exact number of processes, mount points, interface names, or machine
model on the runner.

## Pull requests

Keep pull requests small enough to review. Include:

1. A concise description of the behavior change.
2. The platforms exercised locally.
3. Tests or deterministic fixtures for the changed behavior.
4. Documentation updates for user-visible flags, output, or platform limitations.
5. Any follow-up work that is intentionally outside the change.

Do not claim benchmarks, releases, or platform behavior that was not actually validated.
