use crate::error::{PacketRadarError, Result};
use crate::protocol::header::MAX_PLAYERS;

/// Unit status flags in StarCraft Brood War.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UnitStatusFlags(pub u16);

impl UnitStatusFlags {
    pub const NONE: Self = Self(0x0000);
    pub const CLOAKED: u16 = 0x0001;
    pub const BURROWED: u16 = 0x0002;
    pub const IN_TRANSIT: u16 = 0x0004; // Lifted Terran building
    pub const SIEGED: u16 = 0x0008;
    pub const STIMMED: u16 = 0x0010;
    pub const STASIS: u16 = 0x0020;
    pub const INVULNERABLE: u16 = 0x0040;
    pub const HALLUCINATION: u16 = 0x0080;
    pub const UNDER_STORM: u16 = 0x0100;
    pub const DEFENSIVE_MATRIX: u16 = 0x0200;
    pub const PARASITED: u16 = 0x0400;
    pub const BLINDED: u16 = 0x0800;
    pub const PLAGUED: u16 = 0x1000;
    pub const SELECTED: u16 = 0x2000;

    #[inline]
    pub const fn new(bits: u16) -> Self {
        Self(bits)
    }

    #[inline]
    pub const fn bits(&self) -> u16 {
        self.0
    }

    #[inline]
    pub const fn contains(&self, flag: u16) -> bool {
        (self.0 & flag) == flag
    }

    #[inline]
    pub const fn is_cloaked(&self) -> bool {
        self.contains(Self::CLOAKED)
    }

    #[inline]
    pub const fn is_burrowed(&self) -> bool {
        self.contains(Self::BURROWED)
    }

    #[inline]
    pub const fn is_in_transit(&self) -> bool {
        self.contains(Self::IN_TRANSIT)
    }

    #[inline]
    pub const fn is_sieged(&self) -> bool {
        self.contains(Self::SIEGED)
    }

    #[inline]
    pub const fn is_stimmed(&self) -> bool {
        self.contains(Self::STIMMED)
    }

    #[inline]
    pub const fn is_stasis(&self) -> bool {
        self.contains(Self::STASIS)
    }

    #[inline]
    pub const fn is_invulnerable(&self) -> bool {
        self.contains(Self::INVULNERABLE)
    }

    #[inline]
    pub const fn is_hallucination(&self) -> bool {
        self.contains(Self::HALLUCINATION)
    }

    #[inline]
    pub const fn has_defensive_matrix(&self) -> bool {
        self.contains(Self::DEFENSIVE_MATRIX)
    }
}

impl From<u16> for UnitStatusFlags {
    fn from(bits: u16) -> Self {
        Self(bits)
    }
}

impl From<UnitStatusFlags> for u16 {
    fn from(flags: UnitStatusFlags) -> Self {
        flags.0
    }
}

impl std::ops::BitOr for UnitStatusFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOr<u16> for UnitStatusFlags {
    type Output = Self;
    fn bitor(self, rhs: u16) -> Self {
        Self(self.0 | rhs)
    }
}

impl std::ops::BitAnd for UnitStatusFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAnd<u16> for UnitStatusFlags {
    type Output = Self;
    fn bitand(self, rhs: u16) -> Self {
        Self(self.0 & rhs)
    }
}

/// Dynamic state for an active unit or structure instance on the map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnitState {
    pub unit_id: u16,
    pub player_id: u8,
    pub unit_type: u16,
    pub x: u16,
    pub y: u16,
    pub hp: u16,
    pub shields: u16,
    pub energy: u8,
    pub status_flags: UnitStatusFlags,
    pub target_id: u16,
}

impl UnitState {
    pub const SERIALIZED_SIZE: usize = 18;

    /// Parses a 18-byte unit telemetry block with player bounds validation.
    pub fn parse(raw: &[u8]) -> Result<Self> {
        if raw.len() < Self::SERIALIZED_SIZE {
            return Err(PacketRadarError::BufferTooShort {
                required: Self::SERIALIZED_SIZE,
                actual: raw.len(),
            });
        }

        let player_id = raw[2];
        if player_id >= MAX_PLAYERS {
            return Err(PacketRadarError::InvalidPlayerId(player_id));
        }

        let unit_id = u16::from_be_bytes([raw[0], raw[1]]);
        let unit_type = u16::from_be_bytes([raw[3], raw[4]]);
        let x = u16::from_be_bytes([raw[5], raw[6]]);
        let y = u16::from_be_bytes([raw[7], raw[8]]);
        let hp = u16::from_be_bytes([raw[9], raw[10]]);
        let shields = u16::from_be_bytes([raw[11], raw[12]]);
        let energy = raw[13];
        let status_bits = u16::from_be_bytes([raw[14], raw[15]]);
        let target_id = u16::from_be_bytes([raw[16], raw[17]]);

        Ok(Self {
            unit_id,
            player_id,
            unit_type,
            x,
            y,
            hp,
            shields,
            energy,
            status_flags: UnitStatusFlags::new(status_bits),
            target_id,
        })
    }

    /// Serializes unit telemetry into a fixed 18-byte array without allocation.
    pub fn serialize(&self) -> [u8; Self::SERIALIZED_SIZE] {
        let mut buf = [0u8; Self::SERIALIZED_SIZE];
        buf[0..2].copy_from_slice(&self.unit_id.to_be_bytes());
        buf[2] = self.player_id;
        buf[3..5].copy_from_slice(&self.unit_type.to_be_bytes());
        buf[5..7].copy_from_slice(&self.x.to_be_bytes());
        buf[7..9].copy_from_slice(&self.y.to_be_bytes());
        buf[9..11].copy_from_slice(&self.hp.to_be_bytes());
        buf[11..13].copy_from_slice(&self.shields.to_be_bytes());
        buf[13] = self.energy;
        buf[14..16].copy_from_slice(&self.status_flags.bits().to_be_bytes());
        buf[16..18].copy_from_slice(&self.target_id.to_be_bytes());
        buf
    }

    /// Appends the serialized unit telemetry directly into an existing byte buffer.
    pub fn serialize_into(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.serialize());
    }
}

/// Operational state for an economy base / expansion hub.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseState {
    pub base_id: u16,
    pub player_id: u8,
    pub center_x: u16,
    pub center_y: u16,
    pub minerals: u32,
    pub gas: u32,
    pub worker_count: u8,
    pub is_main: bool,
    pub is_active: bool,
}

impl BaseState {
    pub const SERIALIZED_SIZE: usize = 18;

    /// Parses an 18-byte base telemetry block with bounds checking.
    pub fn parse(raw: &[u8]) -> Result<Self> {
        if raw.len() < Self::SERIALIZED_SIZE {
            return Err(PacketRadarError::BufferTooShort {
                required: Self::SERIALIZED_SIZE,
                actual: raw.len(),
            });
        }

        let player_id = raw[2];
        if player_id >= MAX_PLAYERS {
            return Err(PacketRadarError::InvalidPlayerId(player_id));
        }

        let base_id = u16::from_be_bytes([raw[0], raw[1]]);
        let center_x = u16::from_be_bytes([raw[3], raw[4]]);
        let center_y = u16::from_be_bytes([raw[5], raw[6]]);
        let minerals = u32::from_be_bytes([raw[7], raw[8], raw[9], raw[10]]);
        let gas = u32::from_be_bytes([raw[11], raw[12], raw[13], raw[14]]);
        let worker_count = raw[15];
        let flags = raw[16];
        let is_main = (flags & 0x01) != 0;
        let is_active = (flags & 0x02) != 0;

        Ok(Self {
            base_id,
            player_id,
            center_x,
            center_y,
            minerals,
            gas,
            worker_count,
            is_main,
            is_active,
        })
    }

    /// Serializes base telemetry into a fixed 18-byte array.
    pub fn serialize(&self) -> [u8; Self::SERIALIZED_SIZE] {
        let mut buf = [0u8; Self::SERIALIZED_SIZE];
        buf[0..2].copy_from_slice(&self.base_id.to_be_bytes());
        buf[2] = self.player_id;
        buf[3..5].copy_from_slice(&self.center_x.to_be_bytes());
        buf[5..7].copy_from_slice(&self.center_y.to_be_bytes());
        buf[7..11].copy_from_slice(&self.minerals.to_be_bytes());
        buf[11..15].copy_from_slice(&self.gas.to_be_bytes());
        buf[15] = self.worker_count;
        let mut flags = 0u8;
        if self.is_main {
            flags |= 0x01;
        }
        if self.is_active {
            flags |= 0x02;
        }
        buf[16] = flags;
        buf[17] = 0; // reserved padding
        buf
    }

    /// Appends the serialized base telemetry into an existing byte buffer.
    pub fn serialize_into(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.serialize());
    }
}

/// Macro economy, supply and army state for a player.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerState {
    pub player_id: u8,
    pub race: u8,
    pub minerals: u32,
    pub gas: u32,
    pub supply_used: u16,
    pub supply_total: u16,
    pub apm: u16,
    pub units_alive: u16,
    pub buildings_alive: u16,
}

impl PlayerState {
    pub const SERIALIZED_SIZE: usize = 20;

    /// Parses a 20-byte player state block.
    pub fn parse(raw: &[u8]) -> Result<Self> {
        if raw.len() < Self::SERIALIZED_SIZE {
            return Err(PacketRadarError::BufferTooShort {
                required: Self::SERIALIZED_SIZE,
                actual: raw.len(),
            });
        }

        let player_id = raw[0];
        if player_id >= MAX_PLAYERS {
            return Err(PacketRadarError::InvalidPlayerId(player_id));
        }

        let race = raw[1];
        let minerals = u32::from_be_bytes([raw[2], raw[3], raw[4], raw[5]]);
        let gas = u32::from_be_bytes([raw[6], raw[7], raw[8], raw[9]]);
        let supply_used = u16::from_be_bytes([raw[10], raw[11]]);
        let supply_total = u16::from_be_bytes([raw[12], raw[13]]);
        let apm = u16::from_be_bytes([raw[14], raw[15]]);
        let units_alive = u16::from_be_bytes([raw[16], raw[17]]);
        let buildings_alive = u16::from_be_bytes([raw[18], raw[19]]);

        Ok(Self {
            player_id,
            race,
            minerals,
            gas,
            supply_used,
            supply_total,
            apm,
            units_alive,
            buildings_alive,
        })
    }

    /// Serializes player state into a fixed 20-byte array.
    pub fn serialize(&self) -> [u8; Self::SERIALIZED_SIZE] {
        let mut buf = [0u8; Self::SERIALIZED_SIZE];
        buf[0] = self.player_id;
        buf[1] = self.race;
        buf[2..6].copy_from_slice(&self.minerals.to_be_bytes());
        buf[6..10].copy_from_slice(&self.gas.to_be_bytes());
        buf[10..12].copy_from_slice(&self.supply_used.to_be_bytes());
        buf[12..14].copy_from_slice(&self.supply_total.to_be_bytes());
        buf[14..16].copy_from_slice(&self.apm.to_be_bytes());
        buf[16..18].copy_from_slice(&self.units_alive.to_be_bytes());
        buf[18..20].copy_from_slice(&self.buildings_alive.to_be_bytes());
        buf
    }

    /// Appends the serialized player state into an existing byte buffer.
    pub fn serialize_into(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.serialize());
    }
}
