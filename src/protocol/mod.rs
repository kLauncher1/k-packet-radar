pub mod actions;
pub mod frame;
pub mod header;
pub mod state;

use crate::error::{PacketRadarError, Result};

pub use actions::{ActionType, MAX_CHAT_LEN, MAX_SELECT_UNITS};
pub use frame::{SyncFrame, TurnFrame};
pub use header::{
    PacketFlags, PacketHeader, HEADER_SIZE, MAX_PLAYERS, MAX_UDP_PAYLOAD, SC_PACKET_MAGIC,
};
pub use state::{BaseState, PlayerState, UnitState, UnitStatusFlags};

/// Top-level decoded StarCraft Brood War UDP 6112 datagram.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StarCraftPacket {
    pub header: PacketHeader,
    pub turn: Option<TurnFrame>,
}

impl StarCraftPacket {
    pub const fn new(header: PacketHeader, turn: Option<TurnFrame>) -> Self {
        Self { header, turn }
    }

    /// Parses a complete StarCraft UDP packet from the raw datagram bytes with strict validation.
    pub fn parse(raw: &[u8]) -> Result<Self> {
        if raw.len() > MAX_UDP_PAYLOAD {
            return Err(PacketRadarError::BufferOverflow(raw.len()));
        }
        if raw.len() < HEADER_SIZE {
            return Err(PacketRadarError::BufferTooShort {
                required: HEADER_SIZE,
                actual: raw.len(),
            });
        }

        let header = PacketHeader::parse(raw)?;
        let payload_len = header.payload_len as usize;

        if raw.len() < HEADER_SIZE + payload_len {
            return Err(PacketRadarError::BufferTooShort {
                required: HEADER_SIZE + payload_len,
                actual: raw.len(),
            });
        }

        let mut turn = None;
        if payload_len >= TurnFrame::HEADER_LEN {
            let payload = &raw[HEADER_SIZE..HEADER_SIZE + payload_len];
            let turn_frame = TurnFrame::parse(payload)?;
            turn = Some(turn_frame);
        }

        Ok(Self { header, turn })
    }

    /// Serializes the packet into a byte vector, updating the payload length header field.
    pub fn serialize(&self) -> Vec<u8> {
        let mut payload = Vec::new();
        if let Some(t) = &self.turn {
            payload.extend_from_slice(&t.serialize());
        }

        let mut header = self.header.clone();
        header.payload_len = payload.len() as u16;

        let mut buf = Vec::with_capacity(HEADER_SIZE + payload.len());
        header.serialize_into(&mut buf);
        buf.extend_from_slice(&payload);
        buf
    }
}
