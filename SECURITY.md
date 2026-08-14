# Security policy

## Scope

<code>syspeek</code> is intended to be a local, read-only diagnostics utility. It should not write
files, change permissions, alter network configuration, execute arbitrary commands, or control
processes.

The output can still contain sensitive local information, including hostnames, filesystem paths,
interface addresses, and MAC addresses. Review output before sharing it in tickets, logs, or public
issues.

## Reporting a vulnerability

Please do not publish an unpatched security issue with working exploit details. Use the repository
host's private vulnerability reporting mechanism when it is enabled, or contact the project
maintainer privately with:

- A clear description of the issue and affected version.
- Reproduction steps or a minimal proof of concept.
- The operating system, architecture, and Rust version used.
- The impact and any suggested mitigation.

Security reports are prioritized over feature requests. Do not include credentials, private keys,
access tokens, or unrelated personal data in a report.
