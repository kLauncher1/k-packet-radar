use k_packet_radar::{AnomalyDetector, AnomalyType, LatencyTracker};

#[test]
fn test_latency_tracker_statistics() {
    let mut tracker = LatencyTracker::new();

    tracker.record_packet(0, 1, 1000);
    tracker.record_packet(0, 2, 1050);
    tracker.record_packet(0, 3, 1100);

    let m = tracker.get_metrics(0).expect("metrics exist");
    assert_eq!(m.packet_count, 3);
    assert_eq!(m.sequence_gaps, 0);
    assert!(m.rtt_ms > 0.0);
}

#[test]
fn test_sequence_gap_detection() {
    let mut tracker = LatencyTracker::new();

    tracker.record_packet(1, 10, 1000);
    tracker.record_packet(1, 15, 1050); // skipped 11, 12, 13, 14 (gap of 4)

    let m = tracker.get_metrics(1).expect("metrics exist");
    assert_eq!(m.sequence_gaps, 4);
}

#[test]
fn test_anomaly_detector_flood_trigger() {
    let mut detector = AnomalyDetector::new(5); // threshold 5 packets/sec

    for i in 1..=5 {
        assert_eq!(detector.check_packet(2, i, None, 100), None);
    }

    // 6th packet in same second triggers flood anomaly
    assert_eq!(
        detector.check_packet(2, 6, None, 100),
        Some(AnomalyType::PacketFlood {
            player_id: 2,
            rate: 6,
        })
    );
}
