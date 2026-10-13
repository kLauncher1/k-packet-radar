use crate::error::{PacketRadarError, Result};
use crate::protocol::actions::ActionType;

/// Network turn frame containing scheduled inputs for a lockstep simulation cycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnFrame {
    pub turn_index: u32,
    pub latency_turns: u8,
    pub actions: Vec<ActionType>,
}

impl TurnFrame {
    pub const HEADER_LEN: usize = 5;

    pub const fn new(turn_index: u32, latency_turns: u8, actions: Vec<ActionType>) -> Self {
        Self {
            turn_index,
            latency_turns,
            actions,
        }
    }

    /// Parses a turn frame and all bundled player actions with defensive bounds checking.
    pub fn parse(raw: &[u8]) -> Result<Self> {
        if raw.len() < Self::HEADER_LEN {
            return Err(PacketRadarError::BufferTooShort {
                required: Self::HEADER_LEN,
                actual: raw.len(),
            });
        }

        let turn_index = u32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]);
        let latency_turns = raw[4];
        let mut actions = Vec::new();
        let mut offset = Self::HEADER_LEN;

        while offset < raw.len() {
            let (action, consumed) = ActionType::parse(&raw[offset..])?;
            if consumed == 0 {
                return Err(PacketRadarError::MalformedAction(
                    "zero-length action encountered during turn frame decoding".into(),
                ));
            }
            actions.push(action);
            offset += consumed;
        }

        Ok(Self {
            turn_index,
            latency_turns,
            actions,
        })
    }

    /// Serializes the turn frame into a byte vector.
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(Self::HEADER_LEN + self.actions.len() * 4);
        buf.extend_from_slice(&self.turn_index.to_be_bytes());
        buf.push(self.latency_turns);
        for action in &self.actions {
            action.serialize_into(&mut buf);
        }
        buf
    }

    /// Computes a deterministic 32-bit FNV-1a sync checksum across turn index, latency, and actions.
    pub fn compute_sync_hash(&self) -> u32 {
        let mut hash: u32 = 0x811C_9DC5;
        for byte in self.turn_index.to_be_bytes() {
            hash ^= byte as u32;
            hash = hash.wrapping_mul(0x0100_0193);
        }
        hash ^= self.latency_turns as u32;
        hash = hash.wrapping_mul(0x0100_0193);
        for action in &self.actions {
            hash ^= action.opcode() as u32;
            hash = hash.wrapping_mul(0x0100_0193);
        }
        hash
    }

    /// Generates a synchronization verification frame from this turn.
    pub fn to_sync_frame(&self, seed: u32, active_players: u8) -> SyncFrame {
        SyncFrame::new(
            self.turn_index,
            self.compute_sync_hash(),
            seed,
            active_players,
        )
    }
}

/// Periodic synchronization frame used for lockstep state verification and desync detection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncFrame {
    pub turn_index: u32,
    pub checksum: u32,
    pub seed: u32,
    pub active_players: u8,
}

impl SyncFrame {
    pub const SERIALIZED_SIZE: usize = 13;

    pub const fn new(turn_index: u32, checksum: u32, seed: u32, active_players: u8) -> Self {
        Self {
            turn_index,
            checksum,
            seed,
            active_players,
        }
    }

    /// Parses a 13-byte sync frame with bounds checks.
    pub fn parse(raw: &[u8]) -> Result<Self> {
        if raw.len() < Self::SERIALIZED_SIZE {
            return Err(PacketRadarError::BufferTooShort {
                required: Self::SERIALIZED_SIZE,
                actual: raw.len(),
            });
        }

        let turn_index = u32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]);
        let checksum = u32::from_be_bytes([raw[4], raw[5], raw[6], raw[7]]);
        let seed = u32::from_be_bytes([raw[8], raw[9], raw[10], raw[11]]);
        let active_players = raw[12];

        Ok(Self {
            turn_index,
            checksum,
            seed,
            active_players,
        })
    }

    /// Serializes the sync frame into a fixed 13-byte array without heap allocation.
    pub fn serialize(&self) -> [u8; Self::SERIALIZED_SIZE] {
        let mut buf = [0u8; Self::SERIALIZED_SIZE];
        buf[0..4].copy_from_slice(&self.turn_index.to_be_bytes());
        buf[4..8].copy_from_slice(&self.checksum.to_be_bytes());
        buf[8..12].copy_from_slice(&self.seed.to_be_bytes());
        buf[12] = self.active_players;
        buf
    }

    /// Serializes the sync frame directly into an existing byte vector.
    pub fn serialize_into(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.serialize());
    }
}
