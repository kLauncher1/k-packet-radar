# Project Roadmap

This document outlines shipped milestones and planned technical capabilities for **k-packet-radar**.

---

## Shipped Capabilities

- [x] **Core Architecture & Foundation** — High-performance Rust project layout, modular domain packages, universal verification pipeline, and initial CI workflows.
- [x] **UDP 6112 Raw Packet Sniffer** — High-throughput, non-invasive packet capture pipeline operating over Windows raw sockets and loopback/LAN interfaces with zero-copy ring buffers and fine-grained packet filters.

---

## Planned Milestones

- [ ] **StarCraft Brood War Protocol Parser** — Deterministic deserialization of UDP turn frames, player actions, coordinate telemetry, and game state packets.
- [ ] **Real-Time Latency & Network Telemetry** — Per-player RTT calculation, jitter metrics, packet loss indicators, and interactive ping analysis.
- [ ] **Drophack & Network Anomaly Mitigation** — Real-time detection of malformed packet bursts, out-of-order sequence flooding, and desync indicators.
- [ ] **Tactical Radar Matrix & HUD Interop** — Mini-map coordinate normalization (256x256), radar point indexing, and low-latency visualization pipeline.
- [ ] **End-to-End Synthetic PCAP Test Suite** — Comprehensive packet replay harness for protocol verification and memory safety auditing.
- [ ] **Autonomous Release & Distribution Pipeline** — Semantic release tagging, automated binary packaging, and GitHub ecosystem integration.
- [ ] **Npcap Kernel-Mode Promiscuous Capture Interop** — Direct driver capture fallback when Windows raw socket privileges or loopback filters are restricted.
- [ ] **Embedded WebSocket & Real-Time Browser HUD Stream** — Zero-overhead binary WebSocket server broadcasting live radar matrices to external web-based streaming overlays.
