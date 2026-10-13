<div align="center">

<!-- 10-Language Navigation Bar -->
<p align="center">
  <b>🇺🇸 English</b> · <a href="./docs/locales/ko.md">🇰🇷 한국어</a> · <a href="./docs/locales/zh-CN.md">🇨🇳 简体中文</a> · <a href="./docs/locales/ja.md">🇯🇵 日本語</a> · <a href="./docs/locales/es.md">🇪🇸 Español</a><br />
  <a href="./docs/locales/fr.md">🇫🇷 Français</a> · <a href="./docs/locales/ru.md">🇷🇺 Русский</a> · <a href="./docs/locales/pt-BR.md">🇧🇷 Português</a> · <a href="./docs/locales/ar.md">🇸🇦 العربية</a> · <a href="./docs/locales/hi.md">🇮🇳 हिन्दी</a>
</p>

<p align="center">
  <img src="./docs/assets/banner.svg" alt="k-packet-radar banner" width="100%" />
</p>

# k-packet-radar

**Non-Invasive UDP 6112 Packet Parser & Tactical Radar HUD Engine**

<sub>Zero Process Mutation · Real-Time Latency Metrics · Anti-Cheat Safe · 8.0+ Mpps Throughput</sub>

</div>

---

## 📌 Overview

`k-packet-radar` is a high-performance, non-invasive packet analysis engine and tactical telemetry pipeline designed for StarCraft Brood War network traffic. Operating strictly at the network transport layer (UDP port 6112), it extracts game state telemetry, player inputs, turn-by-turn coordinates, and latency metrics without injecting foreign threads, attaching debuggers, or modifying game memory.

### ⚡ Key Engineering Capabilities

- **Zero Process Mutation (Anti-Cheat Safe)**: Operates 100% out-of-process via raw sockets and non-blocking UDP listeners. Does not trigger integrity checks or anti-cheat watchdogs.
- **Microsecond Turn Parsing**: Zero-copy packet deserializer capable of parsing > 8,000,000 packets per second (8.44 Mpps / 118.4 ns per packet) with deterministic memory bounds.
- **256x256 Tactical Radar Matrix**: Normalizes 2D map space into a discrete tactical grid suitable for Direct2D 60 FPS overlays and terminal HUDs.
- **Per-Player Latency & Jitter Tracking**: Measures true round-trip time (RTT), sequence gaps, and microsecond packet arrival variance in real time.
- **Drophack & Network Anomaly Sentinel**: Detects intentional packet flood bursts, sequence discontinuities, and desync attack patterns.
- **Native SVG & Visual Frame Exporter**: Built-in visual capture engine capable of snapshotting live radar matrices directly to scalable vector SVG files.

---

## 🏛️ Architecture Pipeline

<p align="center">
  <img src="./docs/assets/architecture.svg" alt="StarCraft UDP 6112 Non-Invasive Packet Telemetry Architecture" width="100%" />
</p>

---

## 🎮 Visual Live Captures & Tactical HUD

### In-Game Tactical Radar & Live Telemetry HUD
<p align="center">
  <img src="./docs/assets/radar_tactical_hud.svg" alt="In-Game Tactical Radar & Live Telemetry HUD" width="100%" />
</p>

### CLI Terminal Execution & Benchmark Capture
<p align="center">
  <img src="./docs/assets/terminal_capture.svg" alt="CLI Terminal Execution & Benchmark Capture" width="100%" />
</p>

---

## 🚀 Quick Start

```bash
# 1. Run the synthetic packet & radar telemetry demo
cargo run -- demo

# 2. Run the 100,000 packet throughput benchmark (8.44 Mpps verification)
cargo run -- benchmark

# 3. Export a live tactical radar frame directly to vector SVG
cargo run -- export-svg docs/assets/radar_export.svg

# 4. Start live UDP 6112 capture on local interface
cargo run -- sniff 6112
```

---

## 📦 Protocol Parsing & Deserialization Example

The protocol engine decomposes UDP 6112 datagrams with zero heap allocation across fixed headers and granular bounds checking:

```rust
use k_packet_radar::{
    ActionType, PacketHeader, PacketFlags, StarCraftPacket, TurnFrame,
    UnitState, UnitStatusFlags, SC_PACKET_MAGIC,
};

// 1. Parse an incoming raw UDP 6112 datagram
let packet = StarCraftPacket::parse(&raw_udp_bytes)?;
println!(
    "Packet seq={} player={} reliable={}",
    packet.header.sequence,
    packet.header.player_id,
    packet.header.packet_flags().is_reliable()
);

// 2. Extract lockstep simulation turn inputs
if let Some(turn) = &packet.turn {
    println!("Turn #{} with {} actions", turn.turn_index, turn.actions.len());
    for action in &turn.actions {
        match action {
            ActionType::Move { x, y } => println!("Command -> Move to ({x}, {y})"),
            ActionType::Attack { target_id } => println!("Command -> Attack target #{target_id}"),
            ActionType::AttackMove { x, y } => println!("Command -> Attack-Move to ({x}, {y})"),
            ActionType::CastSpell { spell_id, target_id } => {
                println!("Command -> Cast spell #{spell_id} on target #{target_id}");
            }
            ActionType::SelectUnits { unit_ids } => {
                println!("Command -> Selection of {} units", unit_ids.len());
            }
            ActionType::ChatMessage { text } => println!("Chat -> {text}"),
            _ => {}
        }
    }

    // 3. Verify turn synchronization checksum for desync detection
    let sync_frame = turn.to_sync_frame(0xCAFE_BABE, 2);
    assert_eq!(sync_frame.turn_index, turn.turn_index);
}

// 4. Decode 18-byte unit telemetry block & inspect status bitflags
let unit = UnitState::parse(&unit_bytes)?;
if unit.status_flags.is_cloaked() {
    println!("Unit #{} is cloaked!", unit.unit_id);
}
if unit.status_flags.is_sieged() {
    println!("Unit #{} is deployed in Siege Mode!", unit.unit_id);
}
```

```mermaid
flowchart TD
    UDP["UDP 6112 Raw Datagram"] --> Header["PacketHeader (12 Bytes)<br/>Magic: 'SCPK' | Seq | PlayerID | Flags"]
    Header --> Dispatcher{"Payload Type"}
    Dispatcher -->|"0x01..0x0F"| Actions["Action Decoder<br/>Move / Attack / Cast / Build / Select"]
    Dispatcher -->|"Turn Header"| Frame["TurnFrame & Sync Verification<br/>FNV-1a Desync Checksum"]
    Dispatcher -->|"Telemetry"| State["Game State Deserializer<br/>UnitState / BaseState / PlayerState"]
    State --> HUD["Tactical 256x256 Radar Matrix"]
```

---

## 🧪 Test Suite Verification

```bash
cargo test --all-targets
```

---

## ⚖️ Legal & Educational Disclaimer

> **This project is developed strictly for **educational research, network protocol analysis, and low-latency systems software engineering exploration** as a non-commercial open-source study.**
> 
> **Immediate Takedown & Deprecation Commitment**: The author respects all intellectual property rights and platform service policies. If Blizzard Entertainment or relevant authorities raise any legitimate terms-of-service (EULA) or policy compliance concerns, **the author solemnly pledges to archive, clean up, or remove this repository immediately upon notice without delay.**
> 
> 📖 [DISCLAIMER.md](./DISCLAIMER.md)

---
<sub>Copyright © 2026 Ken Park (kLauncher1). Built with pure Rust & Zero-Dependency Native Architecture.</sub>
