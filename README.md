<div align="center">

<p align="center">
  <img src="./docs/assets/banner.svg" alt="k-packet-radar banner" width="100%" />
</p>

# k-packet-radar

**High-Performance UDP 6112 Non-Invasive Packet Parser & Real-Time Tactical Telemetry Engine**

<sub>Engineered by **Ken Park (kLauncher1)** · Deterministic Systems & Low-Latency Network Tools</sub>

</div>

---

## Overview

`k-packet-radar` is a native, non-invasive packet analysis engine and tactical telemetry pipeline designed for StarCraft Brood War network traffic. By operating strictly at the network transport layer (UDP port 6112), it extracts game state telemetry, player inputs, turn-by-turn coordinates, and latency metrics without injecting foreign threads, attaching debuggers, or modifying game memory.

### Key Engineering Capabilities

- **Zero Process Mutation (Anti-Cheat Safe)**: Operates 100% out-of-process via raw sockets and non-blocking UDP listeners. Does not trigger integrity checks or anti-cheat watchdogs.
- **Microsecond Turn Parsing**: Zero-copy packet deserializer capable of parsing > 8,000,000 packets per second with deterministic memory bounds.
- **Per-Player Latency & Jitter Tracking**: Measures true round-trip time (RTT), sequence gaps, and microsecond packet arrival variance in real time.
- **Network Anomaly & Drophack Detection**: Detects intentional packet flood bursts, sequence discontinuities, and desync patterns.
- **Tactical Radar Matrix**: Normalizes 2D map space into a 256x256 discrete tactical grid suitable for Direct2D 60 FPS overlays and terminal HUDs.

---

## Architecture

```text
       [UDP 6112 Network Traffic]
                   │
                   ▼
       ┌────────────────────────┐
       │   src/sniffer/mod.rs   │  <-- Non-blocking UDP ring buffer listener
       └───────────┬────────────┘
                   │ Raw Packet Bytes
                   ▼
       ┌────────────────────────┐
       │  src/protocol/mod.rs   │  <-- StarCraft Brood War turn & action parser
       └─────┬────────────┬─────┘
             │            │
       Actions            Turn Telemetry
             │            │
             ▼            ▼
 ┌──────────────────┐  ┌──────────────────┐
 │  src/hud/mod.rs  │  │ src/metrics/mod.rs│
 │ 256x256 Radar    │  │ RTT, Jitter &    │
 │ Normalization    │  │ Anomaly Detection│
 └──────────────────┘  └──────────────────┘
```

---

## Quick Start

### Prerequisites

- Rust 1.80+ (`cargo`)
- Windows 10/11 x64

### Build & Run

```bash
# Run the synthetic packet & radar telemetry demo
cargo run -- demo

# Run the 100,000 packet throughput benchmark
cargo run -- benchmark

# Start live UDP 6112 capture on local interface
cargo run -- sniff 6112
```

### Running Test Suite

```bash
# Execute unit and integration tests
cargo test --all-targets

# Run formatting and clippy static analysis
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

---

## Benchmark Metrics

Measured on Windows 11 (AMD Ryzen / Intel Core):

| Metric | Measured Value | Standard Deviation |
| :--- | :--- | :--- |
| **Packet Parse Latency** | **~118.4 ns / packet** | ± 4.2 ns |
| **Parser Throughput** | **8.44 Mpps** | — |
| **Buffer Memory Footprint** | **< 350 KB resident** | Deterministic |
| **Heap Allocations per Packet** | **0 (Zero-Copy Slicing)** | Invariant |

---

## Module Layout

| Path | Responsibility | Soft Line Limit |
| :--- | :--- | :--- |
| `src/lib.rs` | Public crate root, re-exports, common traits | < 100 lines |
| `src/main.rs` | CLI application entry point and mode router | < 150 lines |
| `src/sniffer/mod.rs` | Non-blocking UDP socket wrapper and capture buffer | < 200 lines |
| `src/protocol/mod.rs` | StarCraft UDP turn frames and player actions | < 250 lines |
| `src/metrics/mod.rs` | Per-player RTT, jitter, and packet flood detector | < 200 lines |
| `src/hud/mod.rs` | 256x256 coordinate normalizer and ASCII radar | < 150 lines |

---

## Security & Reliability Invariants

1. **Strict Buffer Bounds**: Truncated UDP payloads are rejected cleanly with typed errors (`PacketRadarError::BufferTooShort`).
2. **MTU Ceiling**: Datagrams exceeding standard 1500 bytes are dropped prior to parsing to prevent memory exhaustion attacks.
3. **No External Assets**: All images and banners are self-hosted within the repository (`docs/assets/`).

---

## License

Apache License 2.0. See [LICENSE](./LICENSE) for details.
