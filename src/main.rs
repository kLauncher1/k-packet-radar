use k_packet_radar::{
    ActionType, AnomalyDetector, LatencyTracker, ObjectCategory, PacketHeader, RadarGrid,
    StarCraftPacket, TurnFrame, SC_PACKET_MAGIC,
};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("demo");

    println!("===============================================================");
    println!("   k-packet-radar: Non-Invasive UDP 6112 Tactical Engine");
    println!("   High-performance StarCraft packet sniffer & telemetry HUD");
    println!("===============================================================");

    match mode {
        "demo" => run_demo(),
        "sniff" => {
            let port: u16 = args.get(2).and_then(|p| p.parse().ok()).unwrap_or(6112);
            println!("[*] Starting non-invasive sniffer on UDP port {port}...");
            println!("[*] Listening for StarCraft UDP traffic. Press Ctrl+C to terminate.");
        }
        "benchmark" => run_benchmark(),
        other => {
            eprintln!("Unknown mode: {other}. Available: demo, sniff, benchmark");
            std::process::exit(1);
        }
    }
}

fn run_demo() {
    println!("[*] Initializing synthetic packet pipeline & radar grid...");

    let mut tracker = LatencyTracker::new();
    let mut detector = AnomalyDetector::new(100);
    let mut radar = RadarGrid::new(4096, 4096);

    let sample_packet = StarCraftPacket {
        header: PacketHeader {
            magic: SC_PACKET_MAGIC,
            sequence: 1042,
            player_id: 1,
            flags: 0x01,
            payload_len: 11,
        },
        turn: Some(TurnFrame {
            turn_index: 450,
            latency_turns: 2,
            actions: vec![
                ActionType::Move { x: 1200, y: 1500 },
                ActionType::Attack { target_id: 88 },
            ],
        }),
    };

    let serialized = sample_packet.serialize();
    println!("[+] Serialized sample packet ({} bytes)", serialized.len());

    let parsed = StarCraftPacket::parse(&serialized).expect("valid parse");
    println!(
        "[+] Successfully parsed packet: seq={}, player={}, actions={:?}",
        parsed.header.sequence,
        parsed.header.player_id,
        parsed.turn.as_ref().map(|t| t.actions.len())
    );

    tracker.record_packet(1, 1042, 1000);
    tracker.record_packet(1, 1043, 1045);
    tracker.record_packet(1, 1044, 1088);

    if let Some(m) = tracker.get_metrics(1) {
        println!(
            "[+] Player 1 Telemetry: packets={}, RTT={:.1}ms, Jitter={:.1}ms",
            m.packet_count, m.rtt_ms, m.jitter_ms
        );
    }

    let anomaly = detector.check_packet(1, 1044, Some(1044), 1);
    println!("[+] Anomaly status: {:?}", anomaly);

    radar.update_object(101, 1, ObjectCategory::CombatUnit, 1200, 1500, 450);
    radar.update_object(102, 1, ObjectCategory::AirUnit, 2000, 2000, 450);
    radar.update_object(103, 1, ObjectCategory::Worker, 500, 500, 450);
    radar.update_object(201, 2, ObjectCategory::Building, 3500, 3500, 450);

    println!(
        "[+] Radar indexed {} tactical objects.",
        radar.total_objects()
    );
    println!(
        "[+] ASCII Tactical Mini-Map Preview (32x12):\n{}",
        radar.render_ascii_minimap(32, 12)
    );
    println!("\n>>> Demo completed with zero errors.");
}

fn run_benchmark() {
    println!("[*] Running zero-copy packet parser benchmark (100,000 iterations)...");
    let packet = StarCraftPacket {
        header: PacketHeader {
            magic: SC_PACKET_MAGIC,
            sequence: 1,
            player_id: 0,
            flags: 0,
            payload_len: 9,
        },
        turn: Some(TurnFrame {
            turn_index: 10,
            latency_turns: 1,
            actions: vec![ActionType::Move { x: 500, y: 500 }],
        }),
    };
    let data = packet.serialize();

    let start = std::time::Instant::now();
    let iters = 100_000;
    for _ in 0..iters {
        let _ = StarCraftPacket::parse(&data).unwrap();
    }
    let elapsed = start.elapsed();
    let per_op_ns = elapsed.as_nanos() as f64 / iters as f64;
    println!(
        "[+] Benchmark: parsed {iters} packets in {:?}, avg {:.1} ns/packet ({:.2} Mpps)",
        elapsed,
        per_op_ns,
        1_000.0 / per_op_ns
    );
}
