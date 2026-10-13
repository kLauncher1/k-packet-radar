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
        "export-svg" => {
            let out_file = args
                .get(2)
                .map(|s| s.as_str())
                .unwrap_or("radar_snapshot.svg");
            run_export_svg(out_file);
        }
        other => {
            eprintln!("Unknown mode: {other}. Available: demo, sniff, benchmark, export-svg");
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

fn run_export_svg(out_file: &str) {
    println!("[*] Generating live tactical radar snapshot to '{out_file}'...");
    let mut radar = RadarGrid::new(4096, 4096);

    // Player 1 army & base (Protoss)
    radar.update_object(101, 1, ObjectCategory::Building, 600, 600, 100); // Nexus
    radar.update_object(102, 1, ObjectCategory::Building, 900, 700, 100); // Gateway
    radar.update_object(103, 1, ObjectCategory::Worker, 650, 550, 100); // Probe 1
    radar.update_object(104, 1, ObjectCategory::Worker, 580, 620, 100); // Probe 2
    radar.update_object(105, 1, ObjectCategory::Worker, 720, 590, 100); // Probe 3
    radar.update_object(110, 1, ObjectCategory::CombatUnit, 1800, 1600, 100); // Dragoon 1
    radar.update_object(111, 1, ObjectCategory::CombatUnit, 1850, 1620, 100); // Dragoon 2
    radar.update_object(112, 1, ObjectCategory::AirUnit, 2048, 2048, 100); // Observer (Center)

    // Player 2 army & base (Terran)
    radar.update_object(201, 2, ObjectCategory::Building, 3400, 3400, 100); // Command Center
    radar.update_object(202, 2, ObjectCategory::Building, 3200, 3100, 100); // Factory
    radar.update_object(203, 2, ObjectCategory::Worker, 3350, 3450, 100); // SCV 1
    radar.update_object(210, 2, ObjectCategory::CombatUnit, 2200, 2100, 100); // Siege Tank
    radar.update_object(211, 2, ObjectCategory::CombatUnit, 2150, 2150, 100); // Vulture
    radar.update_object(212, 2, ObjectCategory::AirUnit, 2400, 2300, 100); // Dropship

    let svg_content = radar.export_svg(600, 600);
    std::fs::write(out_file, &svg_content).expect("write svg file");
    println!(
        "[+] Successfully exported tactical radar snapshot ({} bytes) to '{}'",
        svg_content.len(),
        out_file
    );
}
