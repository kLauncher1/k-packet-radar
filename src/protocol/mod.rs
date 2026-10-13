use crate::error::{PacketRadarError, Result};

pub const SC_PACKET_MAGIC: u32 = 0x5343_504B; // "SCPK"
pub const MAX_UDP_PAYLOAD: usize = 1500;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketHeader {
    pub magic: u32,
    pub sequence: u32,
    pub player_id: u8,
    pub flags: u8,
    pub payload_len: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionType {
    Move { x: u16, y: u16 },
    Attack { target_id: u16 },
    Build { unit_type: u16, x: u16, y: u16 },
    CastSpell { spell_id: u8, target_id: u16 },
    TrainUnit { unit_type: u16 },
    Ping,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnFrame {
    pub turn_index: u32,
    pub latency_turns: u8,
    pub actions: Vec<ActionType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StarCraftPacket {
    pub header: PacketHeader,
    pub turn: Option<TurnFrame>,
}

impl StarCraftPacket {
    pub fn parse(raw: &[u8]) -> Result<Self> {
        if raw.len() > MAX_UDP_PAYLOAD {
            return Err(PacketRadarError::BufferOverflow(raw.len()));
        }
        if raw.len() < 12 {
            return Err(PacketRadarError::BufferTooShort {
                required: 12,
                actual: raw.len(),
            });
        }

        let magic = u32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]);
        if magic != SC_PACKET_MAGIC {
            return Err(PacketRadarError::InvalidMagic(magic));
        }

        let sequence = u32::from_be_bytes([raw[4], raw[5], raw[6], raw[7]]);
        let player_id = raw[8];
        if player_id > 7 {
            return Err(PacketRadarError::InvalidPlayerId(player_id));
        }

        let flags = raw[9];
        let payload_len = u16::from_be_bytes([raw[10], raw[11]]) as usize;

        if raw.len() < 12 + payload_len {
            return Err(PacketRadarError::BufferTooShort {
                required: 12 + payload_len,
                actual: raw.len(),
            });
        }

        let header = PacketHeader {
            magic,
            sequence,
            player_id,
            flags,
            payload_len: payload_len as u16,
        };

        let mut turn = None;
        if payload_len >= 5 {
            let p = &raw[12..12 + payload_len];
            let turn_index = u32::from_be_bytes([p[0], p[1], p[2], p[3]]);
            let latency_turns = p[4];
            let mut actions = Vec::new();
            let mut offset = 5;

            while offset < p.len() {
                let action_code = p[offset];
                offset += 1;
                match action_code {
                    0x01 if offset + 4 <= p.len() => {
                        let x = u16::from_be_bytes([p[offset], p[offset + 1]]);
                        let y = u16::from_be_bytes([p[offset + 2], p[offset + 3]]);
                        offset += 4;
                        actions.push(ActionType::Move { x, y });
                    }
                    0x02 if offset + 2 <= p.len() => {
                        let target_id = u16::from_be_bytes([p[offset], p[offset + 1]]);
                        offset += 2;
                        actions.push(ActionType::Attack { target_id });
                    }
                    0x03 if offset + 6 <= p.len() => {
                        let unit_type = u16::from_be_bytes([p[offset], p[offset + 1]]);
                        let x = u16::from_be_bytes([p[offset + 2], p[offset + 3]]);
                        let y = u16::from_be_bytes([p[offset + 4], p[offset + 5]]);
                        offset += 6;
                        actions.push(ActionType::Build { unit_type, x, y });
                    }
                    0x04 if offset + 3 <= p.len() => {
                        let spell_id = p[offset];
                        let target_id = u16::from_be_bytes([p[offset + 1], p[offset + 2]]);
                        offset += 3;
                        actions.push(ActionType::CastSpell {
                            spell_id,
                            target_id,
                        });
                    }
                    0x05 if offset + 2 <= p.len() => {
                        let unit_type = u16::from_be_bytes([p[offset], p[offset + 1]]);
                        offset += 2;
                        actions.push(ActionType::TrainUnit { unit_type });
                    }
                    0x00 => {
                        actions.push(ActionType::Ping);
                    }
                    _ => break,
                }
            }

            turn = Some(TurnFrame {
                turn_index,
                latency_turns,
                actions,
            });
        }

        Ok(Self { header, turn })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(32);
        buf.extend_from_slice(&self.header.magic.to_be_bytes());
        buf.extend_from_slice(&self.header.sequence.to_be_bytes());
        buf.push(self.header.player_id);
        buf.push(self.header.flags);

        let mut payload = Vec::new();
        if let Some(t) = &self.turn {
            payload.extend_from_slice(&t.turn_index.to_be_bytes());
            payload.push(t.latency_turns);
            for a in &t.actions {
                match a {
                    ActionType::Move { x, y } => {
                        payload.push(0x01);
                        payload.extend_from_slice(&x.to_be_bytes());
                        payload.extend_from_slice(&y.to_be_bytes());
                    }
                    ActionType::Attack { target_id } => {
                        payload.push(0x02);
                        payload.extend_from_slice(&target_id.to_be_bytes());
                    }
                    ActionType::Build { unit_type, x, y } => {
                        payload.push(0x03);
                        payload.extend_from_slice(&unit_type.to_be_bytes());
                        payload.extend_from_slice(&x.to_be_bytes());
                        payload.extend_from_slice(&y.to_be_bytes());
                    }
                    ActionType::CastSpell {
                        spell_id,
                        target_id,
                    } => {
                        payload.push(0x04);
                        payload.push(*spell_id);
                        payload.extend_from_slice(&target_id.to_be_bytes());
                    }
                    ActionType::TrainUnit { unit_type } => {
                        payload.push(0x05);
                        payload.extend_from_slice(&unit_type.to_be_bytes());
                    }
                    ActionType::Ping => {
                        payload.push(0x00);
                    }
                }
            }
        }

        buf.extend_from_slice(&(payload.len() as u16).to_be_bytes());
        buf.extend_from_slice(&payload);
        buf
    }
}
