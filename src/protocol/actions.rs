use crate::error::{PacketRadarError, Result};

pub const MAX_SELECT_UNITS: usize = 32;
pub const MAX_CHAT_LEN: usize = 128;

/// StarCraft Brood War player command action variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionType {
    Ping,
    Move { x: u16, y: u16 },
    Attack { target_id: u16 },
    Build { unit_type: u16, x: u16, y: u16 },
    CastSpell { spell_id: u8, target_id: u16 },
    TrainUnit { unit_type: u16 },
    Stop,
    HoldPosition,
    AttackMove { x: u16, y: u16 },
    SelectUnits { unit_ids: Vec<u16> },
    RallyPoint { x: u16, y: u16 },
    LiftOff { x: u16, y: u16 },
    Land { x: u16, y: u16 },
    ResearchTech { tech_id: u16 },
    Upgrade { upgrade_id: u16 },
    ChatMessage { text: String },
}

impl ActionType {
    /// Returns the 1-byte opcode identifier for this action.
    pub fn opcode(&self) -> u8 {
        match self {
            Self::Ping => 0x00,
            Self::Move { .. } => 0x01,
            Self::Attack { .. } => 0x02,
            Self::Build { .. } => 0x03,
            Self::CastSpell { .. } => 0x04,
            Self::TrainUnit { .. } => 0x05,
            Self::Stop => 0x06,
            Self::HoldPosition => 0x07,
            Self::AttackMove { .. } => 0x08,
            Self::SelectUnits { .. } => 0x09,
            Self::RallyPoint { .. } => 0x0A,
            Self::LiftOff { .. } => 0x0B,
            Self::Land { .. } => 0x0C,
            Self::ResearchTech { .. } => 0x0D,
            Self::Upgrade { .. } => 0x0E,
            Self::ChatMessage { .. } => 0x0F,
        }
    }

    /// Human-readable label for debugging and HUD event feeds.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Ping => "Ping",
            Self::Move { .. } => "Move",
            Self::Attack { .. } => "Attack",
            Self::Build { .. } => "Build",
            Self::CastSpell { .. } => "CastSpell",
            Self::TrainUnit { .. } => "TrainUnit",
            Self::Stop => "Stop",
            Self::HoldPosition => "HoldPosition",
            Self::AttackMove { .. } => "AttackMove",
            Self::SelectUnits { .. } => "SelectUnits",
            Self::RallyPoint { .. } => "RallyPoint",
            Self::LiftOff { .. } => "LiftOff",
            Self::Land { .. } => "Land",
            Self::ResearchTech { .. } => "ResearchTech",
            Self::Upgrade { .. } => "Upgrade",
            Self::ChatMessage { .. } => "ChatMessage",
        }
    }

    /// Total serialized size in bytes including opcode.
    pub fn serialized_size(&self) -> usize {
        1 + match self {
            Self::Ping | Self::Stop | Self::HoldPosition => 0,
            Self::Move { .. }
            | Self::AttackMove { .. }
            | Self::RallyPoint { .. }
            | Self::LiftOff { .. }
            | Self::Land { .. } => 4,
            Self::Attack { .. }
            | Self::TrainUnit { .. }
            | Self::ResearchTech { .. }
            | Self::Upgrade { .. } => 2,
            Self::CastSpell { .. } => 3,
            Self::Build { .. } => 6,
            Self::SelectUnits { unit_ids } => 1 + (unit_ids.len() * 2),
            Self::ChatMessage { text } => 1 + text.len(),
        }
    }

    /// Parses a single action from the slice, returning the action and the number of bytes consumed.
    pub fn parse(raw: &[u8]) -> Result<(Self, usize)> {
        if raw.is_empty() {
            return Err(PacketRadarError::BufferTooShort {
                required: 1,
                actual: 0,
            });
        }

        let opcode = raw[0];
        let p = &raw[1..];

        match opcode {
            0x00 => Ok((Self::Ping, 1)),
            0x01 => {
                if p.len() < 4 {
                    return Err(PacketRadarError::BufferTooShort {
                        required: 5,
                        actual: raw.len(),
                    });
                }
                let x = u16::from_be_bytes([p[0], p[1]]);
                let y = u16::from_be_bytes([p[2], p[3]]);
                Ok((Self::Move { x, y }, 5))
            }
            0x02 => {
                if p.len() < 2 {
                    return Err(PacketRadarError::BufferTooShort {
                        required: 3,
                        actual: raw.len(),
                    });
                }
                let target_id = u16::from_be_bytes([p[0], p[1]]);
                Ok((Self::Attack { target_id }, 3))
            }
            0x03 => {
                if p.len() < 6 {
                    return Err(PacketRadarError::BufferTooShort {
                        required: 7,
                        actual: raw.len(),
                    });
                }
                let unit_type = u16::from_be_bytes([p[0], p[1]]);
                let x = u16::from_be_bytes([p[2], p[3]]);
                let y = u16::from_be_bytes([p[4], p[5]]);
                Ok((Self::Build { unit_type, x, y }, 7))
            }
            0x04 => {
                if p.len() < 3 {
                    return Err(PacketRadarError::BufferTooShort {
                        required: 4,
                        actual: raw.len(),
                    });
                }
                let spell_id = p[0];
                let target_id = u16::from_be_bytes([p[1], p[2]]);
                Ok((
                    Self::CastSpell {
                        spell_id,
                        target_id,
                    },
                    4,
                ))
            }
            0x05 => {
                if p.len() < 2 {
                    return Err(PacketRadarError::BufferTooShort {
                        required: 3,
                        actual: raw.len(),
                    });
                }
                let unit_type = u16::from_be_bytes([p[0], p[1]]);
                Ok((Self::TrainUnit { unit_type }, 3))
            }
            0x06 => Ok((Self::Stop, 1)),
            0x07 => Ok((Self::HoldPosition, 1)),
            0x08 => {
                if p.len() < 4 {
                    return Err(PacketRadarError::BufferTooShort {
                        required: 5,
                        actual: raw.len(),
                    });
                }
                let x = u16::from_be_bytes([p[0], p[1]]);
                let y = u16::from_be_bytes([p[2], p[3]]);
                Ok((Self::AttackMove { x, y }, 5))
            }
            0x09 => {
                if p.is_empty() {
                    return Err(PacketRadarError::BufferTooShort {
                        required: 2,
                        actual: raw.len(),
                    });
                }
                let count = p[0] as usize;
                if count > MAX_SELECT_UNITS {
                    return Err(PacketRadarError::MalformedAction(format!(
                        "select units count exceeds maximum limit: {count} > {MAX_SELECT_UNITS}"
                    )));
                }
                let needed = 1 + count * 2;
                if p.len() < needed {
                    return Err(PacketRadarError::BufferTooShort {
                        required: 1 + needed,
                        actual: raw.len(),
                    });
                }
                let mut unit_ids = Vec::with_capacity(count);
                for i in 0..count {
                    let offset = 1 + i * 2;
                    let id = u16::from_be_bytes([p[offset], p[offset + 1]]);
                    unit_ids.push(id);
                }
                Ok((Self::SelectUnits { unit_ids }, 1 + needed))
            }
            0x0A => {
                if p.len() < 4 {
                    return Err(PacketRadarError::BufferTooShort {
                        required: 5,
                        actual: raw.len(),
                    });
                }
                let x = u16::from_be_bytes([p[0], p[1]]);
                let y = u16::from_be_bytes([p[2], p[3]]);
                Ok((Self::RallyPoint { x, y }, 5))
            }
            0x0B => {
                if p.len() < 4 {
                    return Err(PacketRadarError::BufferTooShort {
                        required: 5,
                        actual: raw.len(),
                    });
                }
                let x = u16::from_be_bytes([p[0], p[1]]);
                let y = u16::from_be_bytes([p[2], p[3]]);
                Ok((Self::LiftOff { x, y }, 5))
            }
            0x0C => {
                if p.len() < 4 {
                    return Err(PacketRadarError::BufferTooShort {
                        required: 5,
                        actual: raw.len(),
                    });
                }
                let x = u16::from_be_bytes([p[0], p[1]]);
                let y = u16::from_be_bytes([p[2], p[3]]);
                Ok((Self::Land { x, y }, 5))
            }
            0x0D => {
                if p.len() < 2 {
                    return Err(PacketRadarError::BufferTooShort {
                        required: 3,
                        actual: raw.len(),
                    });
                }
                let tech_id = u16::from_be_bytes([p[0], p[1]]);
                Ok((Self::ResearchTech { tech_id }, 3))
            }
            0x0E => {
                if p.len() < 2 {
                    return Err(PacketRadarError::BufferTooShort {
                        required: 3,
                        actual: raw.len(),
                    });
                }
                let upgrade_id = u16::from_be_bytes([p[0], p[1]]);
                Ok((Self::Upgrade { upgrade_id }, 3))
            }
            0x0F => {
                if p.is_empty() {
                    return Err(PacketRadarError::BufferTooShort {
                        required: 2,
                        actual: raw.len(),
                    });
                }
                let len = p[0] as usize;
                if len > MAX_CHAT_LEN {
                    return Err(PacketRadarError::MalformedAction(format!(
                        "chat message length exceeds maximum limit: {len} > {MAX_CHAT_LEN}"
                    )));
                }
                let needed = 1 + len;
                if p.len() < needed {
                    return Err(PacketRadarError::BufferTooShort {
                        required: 1 + needed,
                        actual: raw.len(),
                    });
                }
                let text_slice = &p[1..1 + len];
                let text = std::str::from_utf8(text_slice).map_err(|e| {
                    PacketRadarError::MalformedAction(format!("invalid UTF-8 in chat string: {e}"))
                })?;
                Ok((
                    Self::ChatMessage {
                        text: text.to_string(),
                    },
                    1 + needed,
                ))
            }
            unknown => Err(PacketRadarError::MalformedAction(format!(
                "unrecognized action opcode: 0x{unknown:02X}"
            ))),
        }
    }

    /// Serializes this action directly into the provided byte vector.
    pub fn serialize_into(&self, out: &mut Vec<u8>) {
        out.push(self.opcode());
        match self {
            Self::Ping | Self::Stop | Self::HoldPosition => {}
            Self::Move { x, y }
            | Self::AttackMove { x, y }
            | Self::RallyPoint { x, y }
            | Self::LiftOff { x, y }
            | Self::Land { x, y } => {
                out.extend_from_slice(&x.to_be_bytes());
                out.extend_from_slice(&y.to_be_bytes());
            }
            Self::Attack { target_id } => {
                out.extend_from_slice(&target_id.to_be_bytes());
            }
            Self::Build { unit_type, x, y } => {
                out.extend_from_slice(&unit_type.to_be_bytes());
                out.extend_from_slice(&x.to_be_bytes());
                out.extend_from_slice(&y.to_be_bytes());
            }
            Self::CastSpell {
                spell_id,
                target_id,
            } => {
                out.push(*spell_id);
                out.extend_from_slice(&target_id.to_be_bytes());
            }
            Self::TrainUnit { unit_type } => {
                out.extend_from_slice(&unit_type.to_be_bytes());
            }
            Self::SelectUnits { unit_ids } => {
                out.push(unit_ids.len() as u8);
                for id in unit_ids {
                    out.extend_from_slice(&id.to_be_bytes());
                }
            }
            Self::ResearchTech { tech_id } => {
                out.extend_from_slice(&tech_id.to_be_bytes());
            }
            Self::Upgrade { upgrade_id } => {
                out.extend_from_slice(&upgrade_id.to_be_bytes());
            }
            Self::ChatMessage { text } => {
                let bytes = text.as_bytes();
                let clamped_len = bytes.len().min(MAX_CHAT_LEN) as u8;
                out.push(clamped_len);
                out.extend_from_slice(&bytes[..clamped_len as usize]);
            }
        }
    }

    /// Serializes this action into a newly allocated byte vector.
    pub fn serialize(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.serialized_size());
        self.serialize_into(&mut buf);
        buf
    }
}
