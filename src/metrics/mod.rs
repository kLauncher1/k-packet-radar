use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PlayerMetrics {
    pub player_id: u8,
    pub packet_count: u64,
    pub last_sequence: u32,
    pub sequence_gaps: u32,
    pub last_timestamp_ms: u64,
    pub rtt_ms: f64,
    pub jitter_ms: f64,
}

impl PlayerMetrics {
    pub fn new(player_id: u8) -> Self {
        Self {
            player_id,
            packet_count: 0,
            last_sequence: 0,
            sequence_gaps: 0,
            last_timestamp_ms: 0,
            rtt_ms: 0.0,
            jitter_ms: 0.0,
        }
    }
}

#[derive(Debug, Default)]
pub struct LatencyTracker {
    players: HashMap<u8, PlayerMetrics>,
}

impl LatencyTracker {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
        }
    }

    pub fn record_packet(&mut self, player_id: u8, sequence: u32, now_ms: u64) {
        let metrics = self
            .players
            .entry(player_id)
            .or_insert_with(|| PlayerMetrics::new(player_id));

        metrics.packet_count += 1;

        if metrics.last_sequence > 0 && sequence > metrics.last_sequence + 1 {
            metrics.sequence_gaps += sequence - (metrics.last_sequence + 1);
        }
        metrics.last_sequence = sequence;

        if metrics.last_timestamp_ms > 0 && now_ms > metrics.last_timestamp_ms {
            let delta = (now_ms - metrics.last_timestamp_ms) as f64;
            let diff = (delta - metrics.rtt_ms).abs();
            metrics.jitter_ms = metrics.jitter_ms * 0.8 + diff * 0.2;
            metrics.rtt_ms = metrics.rtt_ms * 0.9 + delta * 0.1;
        }

        metrics.last_timestamp_ms = now_ms;
    }

    pub fn get_metrics(&self, player_id: u8) -> Option<&PlayerMetrics> {
        self.players.get(&player_id)
    }

    pub fn all_metrics(&self) -> &HashMap<u8, PlayerMetrics> {
        &self.players
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnomalyType {
    PacketFlood {
        player_id: u8,
        rate: u32,
    },
    SequenceDiscontinuity {
        player_id: u8,
        expected: u32,
        actual: u32,
    },
    DrophackSuspected {
        player_id: u8,
        reason: String,
    },
}

#[derive(Debug)]
pub struct AnomalyDetector {
    burst_threshold_per_sec: u32,
    packet_windows: HashMap<u8, (u64, u32)>, // player -> (second_timestamp, count)
}

impl AnomalyDetector {
    pub fn new(burst_threshold_per_sec: u32) -> Self {
        Self {
            burst_threshold_per_sec,
            packet_windows: HashMap::new(),
        }
    }

    pub fn check_packet(
        &mut self,
        player_id: u8,
        sequence: u32,
        expected_seq: Option<u32>,
        now_sec: u64,
    ) -> Option<AnomalyType> {
        let window = self.packet_windows.entry(player_id).or_insert((now_sec, 0));
        if window.0 == now_sec {
            window.1 += 1;
        } else {
            window.0 = now_sec;
            window.1 = 1;
        }

        if window.1 > self.burst_threshold_per_sec {
            return Some(AnomalyType::PacketFlood {
                player_id,
                rate: window.1,
            });
        }

        if let Some(exp) = expected_seq {
            if sequence > exp + 100 {
                return Some(AnomalyType::DrophackSuspected {
                    player_id,
                    reason: format!("severe sequence jump from {exp} to {sequence}"),
                });
            } else if sequence != exp {
                return Some(AnomalyType::SequenceDiscontinuity {
                    player_id,
                    expected: exp,
                    actual: sequence,
                });
            }
        }

        None
    }
}
