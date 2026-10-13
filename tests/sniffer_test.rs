use k_packet_radar::{
    ActionType, CapturedPacket, PacketFilter, PacketHeader, PacketRingBuffer, PacketSniffer,
    SnifferConfig, StarCraftPacket, TurnFrame, SC_PACKET_MAGIC,
};
use std::net::UdpSocket;
use std::thread;
use std::time::Duration;

#[test]
fn test_sniffer_default_config_and_filter() {
    let config = SnifferConfig::default();
    assert_eq!(config.bind_addr, "127.0.0.1");
    assert_eq!(config.port, 6112);
    assert!(config.non_blocking);
    assert_eq!(config.ring_buffer_capacity, 1024);
    assert!(config.filter.require_sc_magic);
    assert_eq!(config.filter.min_payload_size, 12);
}

#[test]
fn test_packet_ring_buffer_operations() {
    let mut ring = PacketRingBuffer::new(3);
    assert_eq!(ring.capacity(), 3);
    assert_eq!(ring.len(), 0);
    assert!(ring.is_empty());
    assert!(!ring.is_full());

    let addr = "127.0.0.1:6112".parse().unwrap();
    let p1 = CapturedPacket {
        raw_bytes: vec![1, 2, 3],
        source_addr: addr,
        timestamp_ns: 100,
        parsed: None,
    };
    let p2 = CapturedPacket {
        raw_bytes: vec![4, 5, 6],
        source_addr: addr,
        timestamp_ns: 200,
        parsed: None,
    };
    let p3 = CapturedPacket {
        raw_bytes: vec![7, 8, 9],
        source_addr: addr,
        timestamp_ns: 300,
        parsed: None,
    };
    let p4 = CapturedPacket {
        raw_bytes: vec![10, 11, 12],
        source_addr: addr,
        timestamp_ns: 400,
        parsed: None,
    };

    assert!(ring.push(p1));
    assert!(ring.push(p2));
    assert!(ring.push(p3));
    assert!(ring.is_full());
    assert_eq!(ring.len(), 3);
    assert_eq!(ring.overflow_count(), 0);

    // 4th packet causes overflow (drops oldest p1)
    let had_cap = ring.push(p4);
    assert!(!had_cap);
    assert_eq!(ring.overflow_count(), 1);
    assert_eq!(ring.len(), 3);

    // Oldest should now be p2
    let popped = ring.pop().expect("pop item");
    assert_eq!(popped.raw_bytes, vec![4, 5, 6]);

    ring.clear();
    assert_eq!(ring.len(), 0);
    assert_eq!(ring.overflow_count(), 0);
}

#[test]
fn test_packet_filter_conditions() {
    let filter = PacketFilter {
        allowed_source_ports: vec![6112],
        min_payload_size: 12,
        max_payload_size: 1500,
        require_sc_magic: true,
    };

    let allowed_addr = "127.0.0.1:6112".parse().unwrap();
    let rejected_addr = "127.0.0.1:9999".parse().unwrap();

    let mut valid_magic_data = vec![0u8; 16];
    valid_magic_data[0..4].copy_from_slice(&SC_PACKET_MAGIC.to_be_bytes());

    let mut invalid_magic_data = vec![0u8; 16];
    invalid_magic_data[0..4].copy_from_slice(&0xDEADBEEFu32.to_be_bytes());

    // Matches valid packet from allowed port
    assert!(filter.matches(&valid_magic_data, &allowed_addr));

    // Rejects packet from non-allowed port
    assert!(!filter.matches(&valid_magic_data, &rejected_addr));

    // Rejects invalid magic
    assert!(!filter.matches(&invalid_magic_data, &allowed_addr));

    // Rejects payload too short
    assert!(!filter.matches(&valid_magic_data[..8], &allowed_addr));
}

#[test]
fn test_sniffer_unbound_error() {
    let mut sniffer = PacketSniffer::new(SnifferConfig::default());
    assert!(!sniffer.is_bound());
    assert!(sniffer.local_addr().is_err());
    assert!(sniffer.poll_captured_packet().is_err());
}

#[test]
fn test_sniffer_bind_ephemeral_and_local_addr() {
    let config = SnifferConfig::default().with_port(0); // Port 0 assigns random OS port
    let mut sniffer = PacketSniffer::new(config);

    sniffer.bind().expect("bind must succeed");
    assert!(sniffer.is_bound());

    let local = sniffer.local_addr().expect("valid local addr");
    assert_eq!(local.ip().to_string(), "127.0.0.1");
    assert!(local.port() > 0);
}

#[test]
fn test_sniffer_udp_packet_roundtrip() {
    // Bind sniffer to an ephemeral port to avoid conflict in concurrent test runs
    let config = SnifferConfig::default().with_port(0);
    let mut sniffer = PacketSniffer::new(config);
    sniffer.bind().expect("bind must succeed");

    let sniffer_addr = sniffer.local_addr().expect("local addr");

    // Construct a valid StarCraft packet
    let packet = StarCraftPacket {
        header: PacketHeader {
            magic: SC_PACKET_MAGIC,
            sequence: 101,
            player_id: 2,
            flags: 0x01,
            payload_len: 11,
        },
        turn: Some(TurnFrame {
            turn_index: 50,
            latency_turns: 2,
            actions: vec![
                ActionType::Move { x: 500, y: 600 },
                ActionType::Attack { target_id: 42 },
            ],
        }),
    };
    let serialized = packet.serialize();

    // Send the datagram via a client socket
    let sender_socket = UdpSocket::bind("127.0.0.1:0").expect("sender bind");
    sender_socket
        .send_to(&serialized, sniffer_addr)
        .expect("send packet");

    // Poll until packet is ingested (up to 500ms)
    let mut captured = None;
    for _ in 0..50 {
        if let Some(p) = sniffer.poll_captured_packet().expect("poll without error") {
            captured = Some(p);
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }

    let p = captured.expect("must have captured packet");
    assert_eq!(p.raw_bytes, serialized);
    let parsed = p.parsed.expect("parsed StarCraft packet");
    assert_eq!(parsed.header.sequence, 101);
    assert_eq!(parsed.header.player_id, 2);

    let stats = sniffer.stats();
    assert_eq!(stats.packets_received, 1);
    assert_eq!(stats.bytes_received, serialized.len() as u64);
    assert_eq!(stats.malformed_packets, 0);

    // Verify pop_packet from ring buffer
    let popped = sniffer.pop_packet().expect("popped from ring buffer");
    assert_eq!(popped.raw_bytes, serialized);
}

#[test]
fn test_sniffer_batch_polling_and_filtering() {
    // Custom filter that drops non-magic datagrams
    let filter = PacketFilter {
        allowed_source_ports: Vec::new(),
        min_payload_size: 12,
        max_payload_size: 1500,
        require_sc_magic: true,
    };
    let config = SnifferConfig::default()
        .with_port(0)
        .with_filter(filter)
        .with_ring_buffer_capacity(10);

    let mut sniffer = PacketSniffer::new(config);
    sniffer.bind().expect("bind sniffer");
    let sniffer_addr = sniffer.local_addr().expect("local addr");

    let sender_socket = UdpSocket::bind("127.0.0.1:0").expect("sender bind");

    // 1. Send valid packet
    let valid_packet = StarCraftPacket {
        header: PacketHeader {
            magic: SC_PACKET_MAGIC,
            sequence: 1,
            player_id: 0,
            flags: 0,
            payload_len: 0,
        },
        turn: None,
    };
    sender_socket
        .send_to(&valid_packet.serialize(), sniffer_addr)
        .unwrap();

    // 2. Send invalid magic packet (should be filtered)
    let garbage = vec![0xEE; 20];
    sender_socket.send_to(&garbage, sniffer_addr).unwrap();

    thread::sleep(Duration::from_millis(50));

    let count = sniffer.poll_batch(5).expect("poll batch");
    // Only 1 valid packet should be retained in the ring buffer
    assert_eq!(count, 1);
    assert_eq!(sniffer.ring_buffer().len(), 1);

    let stats = sniffer.stats();
    assert_eq!(stats.packets_received, 2);
    assert_eq!(stats.filter_matches, 1); // 1 packet filtered out
}

#[test]
fn test_sniffer_poll_next_packet_compatibility() {
    let config = SnifferConfig::default().with_port(0);
    let mut sniffer = PacketSniffer::new(config);
    sniffer.bind().expect("bind sniffer");
    let sniffer_addr = sniffer.local_addr().expect("local addr");

    let sender_socket = UdpSocket::bind("127.0.0.1:0").expect("sender bind");
    let packet = StarCraftPacket {
        header: PacketHeader {
            magic: SC_PACKET_MAGIC,
            sequence: 777,
            player_id: 1,
            flags: 0,
            payload_len: 0,
        },
        turn: None,
    };
    sender_socket
        .send_to(&packet.serialize(), sniffer_addr)
        .unwrap();

    thread::sleep(Duration::from_millis(50));

    let next = sniffer
        .poll_next_packet()
        .expect("poll next")
        .expect("has packet");
    assert_eq!(next.0.header.sequence, 777);
    assert_eq!(next.0.header.player_id, 1);
}

#[test]
fn test_sniffer_parse_raw() {
    let sniffer = PacketSniffer::new(SnifferConfig::default());
    let mut raw = vec![0u8; 12];
    raw[0..4].copy_from_slice(&SC_PACKET_MAGIC.to_be_bytes());
    raw[8] = 3;

    let parsed = sniffer.parse_raw(&raw).expect("parse valid raw");
    assert_eq!(parsed.header.player_id, 3);
}
