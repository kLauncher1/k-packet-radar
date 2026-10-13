# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-10-13

### Added
- High-throughput `PacketRingBuffer` with bounded queue capacity and overflow telemetry.
- Fine-grained `PacketFilter` with port matching, payload bounds, and protocol magic verification.
- Comprehensive `SnifferStats` counters tracking received datagrams, bytes, drops, and malformed frames.
- Batch polling capability (`poll_batch`) for amortized zero-copy datagram processing.
- Dedicated integration test suite in `tests/sniffer_test.rs` covering ephemeral socket loopback, filters, and buffer overflow semantics.
- GitHub Actions CI workflow for automated linting, checking, and test verification across Linux runners.

## [0.1.0] - 2026-10-13

### Added
- Core non-invasive UDP 6112 packet sniffer and ring buffer polling loop.
- StarCraft Brood War UDP turn frame and action deserializer.
- Real-time per-player latency (RTT) and jitter tracking engine.
- Packet flood and sequence anomaly detection module for drophack defense.
- Normalized 256x256 radar grid and tactical object mapping.
- High-throughput benchmark measuring > 8.0 Mpps parsing performance.
- Universal verification suite with clippy, formatting, and unit tests.
