# Security Policy

## Supported Versions

Security updates are actively applied to the latest release series on the `main` branch.

| Version | Supported          |
| ------- | ------------------ |
| Latest (`0.x`) | :white_check_mark: |
| `< 0.1.0` | :x:                |

## Memory Safety & Network Hygiene Invariants

1. **Non-Invasive Architecture**: `k-packet-radar` strictly reads network packets without injecting foreign threads, patching game memory, or modifying target processes.
2. **Buffer Bounds Checking**: All packet parsing logic enforces strict slice bounds and checked slicing to eliminate buffer over-reads and panics on truncated UDP datagrams.
3. **Zero Secret Retention**: Network packet buffers do not persist sensitive user credentials or account authentication tokens; payloads are sanitized in memory.
4. **Denial-of-Service Defense**: Packet parsers reject oversized UDP datagrams (> 1500 bytes standard MTU) before allocation to prevent memory exhaustion from spoofed packet floods.

## Reporting a Vulnerability

If you discover a potential memory safety bug, denial-of-service vulnerability, or security defect:

1. **Do not open a public issue** with reproduction exploits.
2. Report the vulnerability privately via [GitHub Security Advisories](../../security/advisories/new) or contact the project maintainers.
3. Fixes are prioritized and published through coordinated security releases.
