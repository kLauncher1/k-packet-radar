use crate::error::{PacketRadarError, Result};
use crate::protocol::{StarCraftPacket, MAX_UDP_PAYLOAD, SC_PACKET_MAGIC};
use std::collections::VecDeque;
use std::net::{SocketAddr, UdpSocket};
use std::time::{SystemTime, UNIX_EPOCH};

/// Captured raw datagram with timestamp, source address, and optional parsed representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedPacket {
    pub raw_bytes: Vec<u8>,
    pub source_addr: SocketAddr,
    pub timestamp_ns: u64,
    pub parsed: Option<StarCraftPacket>,
}

/// Comprehensive packet capture telemetry and drop counters.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SnifferStats {
    pub packets_received: u64,
    pub bytes_received: u64,
    pub dropped_packets: u64,
    pub malformed_packets: u64,
    pub filter_matches: u64,
}

impl SnifferStats {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Bounded circular buffer for non-blocking packet ingestion without heap growth.
#[derive(Debug)]
pub struct PacketRingBuffer {
    capacity: usize,
    queue: VecDeque<CapturedPacket>,
    overflow_count: u64,
}

impl PacketRingBuffer {
    pub fn new(capacity: usize) -> Self {
        let cap = capacity.max(1);
        Self {
            capacity: cap,
            queue: VecDeque::with_capacity(cap),
            overflow_count: 0,
        }
    }

    /// Pushes a captured packet into the ring buffer.
    /// If the queue is at capacity, the oldest packet is evicted and `false` is returned.
    pub fn push(&mut self, packet: CapturedPacket) -> bool {
        let had_capacity = if self.queue.len() >= self.capacity {
            self.queue.pop_front();
            self.overflow_count += 1;
            false
        } else {
            true
        };
        self.queue.push_back(packet);
        had_capacity
    }

    pub fn pop(&mut self) -> Option<CapturedPacket> {
        self.queue.pop_front()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.queue.len() >= self.capacity
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn overflow_count(&self) -> u64 {
        self.overflow_count
    }

    pub fn clear(&mut self) {
        self.queue.clear();
        self.overflow_count = 0;
    }
}

/// Flexible packet acceptance criteria for StarCraft UDP traffic.
#[derive(Debug, Clone)]
pub struct PacketFilter {
    pub allowed_source_ports: Vec<u16>,
    pub min_payload_size: usize,
    pub max_payload_size: usize,
    pub require_sc_magic: bool,
}

impl Default for PacketFilter {
    fn default() -> Self {
        Self {
            allowed_source_ports: Vec::new(),
            min_payload_size: 12,
            max_payload_size: MAX_UDP_PAYLOAD,
            require_sc_magic: true,
        }
    }
}

impl PacketFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn matches(&self, data: &[u8], src: &SocketAddr) -> bool {
        if data.len() < self.min_payload_size || data.len() > self.max_payload_size {
            return false;
        }

        if !self.allowed_source_ports.is_empty() && !self.allowed_source_ports.contains(&src.port())
        {
            return false;
        }

        if self.require_sc_magic {
            if data.len() < 4 {
                return false;
            }
            let magic = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
            if magic != SC_PACKET_MAGIC {
                return false;
            }
        }

        true
    }
}

/// Configuration settings for socket binding and capture behavior.
#[derive(Debug, Clone)]
pub struct SnifferConfig {
    pub bind_addr: String,
    pub port: u16,
    pub non_blocking: bool,
    pub ring_buffer_capacity: usize,
    pub filter: PacketFilter,
}

impl Default for SnifferConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1".to_string(),
            port: 6112,
            non_blocking: true,
            ring_buffer_capacity: 1024,
            filter: PacketFilter::default(),
        }
    }
}

impl SnifferConfig {
    pub fn with_bind_addr(mut self, addr: impl Into<String>) -> Self {
        self.bind_addr = addr.into();
        self
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn with_non_blocking(mut self, non_blocking: bool) -> Self {
        self.non_blocking = non_blocking;
        self
    }

    pub fn with_ring_buffer_capacity(mut self, capacity: usize) -> Self {
        self.ring_buffer_capacity = capacity;
        self
    }

    pub fn with_filter(mut self, filter: PacketFilter) -> Self {
        self.filter = filter;
        self
    }
}

/// High-throughput, non-invasive UDP 6112 packet capture engine.
pub struct PacketSniffer {
    config: SnifferConfig,
    socket: Option<UdpSocket>,
    rx_buffer: Vec<u8>,
    ring_buffer: PacketRingBuffer,
    stats: SnifferStats,
}

impl PacketSniffer {
    pub fn new(config: SnifferConfig) -> Self {
        let ring_capacity = config.ring_buffer_capacity;
        Self {
            config,
            socket: None,
            rx_buffer: vec![0u8; 2048],
            ring_buffer: PacketRingBuffer::new(ring_capacity),
            stats: SnifferStats::default(),
        }
    }

    /// Binds the sniffer socket to the configured address and port.
    pub fn bind(&mut self) -> Result<()> {
        let addr = format!("{}:{}", self.config.bind_addr, self.config.port);
        let socket =
            UdpSocket::bind(&addr).map_err(|e| PacketRadarError::SocketError(e.to_string()))?;

        if self.config.non_blocking {
            socket
                .set_nonblocking(true)
                .map_err(|e| PacketRadarError::SocketError(e.to_string()))?;
        }

        self.socket = Some(socket);
        Ok(())
    }

    pub fn is_bound(&self) -> bool {
        self.socket.is_some()
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        let socket = self.socket.as_ref().ok_or_else(|| {
            PacketRadarError::SocketError("socket not bound; call bind() first".to_string())
        })?;
        socket
            .local_addr()
            .map_err(|e| PacketRadarError::SocketError(e.to_string()))
    }

    /// Receives one datagram from the socket, evaluates filters, updates stats,
    /// and enqueues into the ring buffer.
    pub fn poll_captured_packet(&mut self) -> Result<Option<CapturedPacket>> {
        let socket = match &self.socket {
            Some(s) => s,
            None => {
                return Err(PacketRadarError::SocketError(
                    "socket not bound; call bind() first".to_string(),
                ))
            }
        };

        match socket.recv_from(&mut self.rx_buffer) {
            Ok((size, src)) => {
                self.stats.packets_received += 1;
                self.stats.bytes_received += size as u64;

                let raw_slice = &self.rx_buffer[..size];
                if !self.config.filter.matches(raw_slice, &src) {
                    self.stats.filter_matches += 1;
                    return Ok(None);
                }

                let timestamp_ns = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_nanos() as u64)
                    .unwrap_or(0);

                let parsed = match StarCraftPacket::parse(raw_slice) {
                    Ok(p) => Some(p),
                    Err(_) => {
                        self.stats.malformed_packets += 1;
                        None
                    }
                };

                let packet = CapturedPacket {
                    raw_bytes: raw_slice.to_vec(),
                    source_addr: src,
                    timestamp_ns,
                    parsed,
                };

                if !self.ring_buffer.push(packet.clone()) {
                    self.stats.dropped_packets += 1;
                }

                Ok(Some(packet))
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(PacketRadarError::SocketError(e.to_string())),
        }
    }

    /// Backward-compatible single-packet polling helper.
    pub fn poll_next_packet(&mut self) -> Result<Option<(StarCraftPacket, SocketAddr)>> {
        while let Some(captured) = self.poll_captured_packet()? {
            if let Some(parsed) = captured.parsed {
                return Ok(Some((parsed, captured.source_addr)));
            }
        }
        Ok(None)
    }

    /// Batches non-blocking datagram reads into the internal ring buffer up to `max_batch`.
    pub fn poll_batch(&mut self, max_batch: usize) -> Result<usize> {
        let mut count = 0;
        for _ in 0..max_batch {
            match self.poll_captured_packet()? {
                Some(_) => count += 1,
                None => break,
            }
        }
        Ok(count)
    }

    /// Pops the next received packet from the internal ring buffer queue.
    pub fn pop_packet(&mut self) -> Option<CapturedPacket> {
        self.ring_buffer.pop()
    }

    pub fn parse_raw(&self, data: &[u8]) -> Result<StarCraftPacket> {
        StarCraftPacket::parse(data)
    }

    pub fn config(&self) -> &SnifferConfig {
        &self.config
    }

    pub fn stats(&self) -> &SnifferStats {
        &self.stats
    }

    pub fn ring_buffer(&self) -> &PacketRingBuffer {
        &self.ring_buffer
    }

    pub fn reset_stats(&mut self) {
        self.stats.reset();
    }
}
