use crate::error::{PacketRadarError, Result};

pub const SC_PACKET_MAGIC: u32 = 0x5343_504B; // "SCPK"
pub const MAX_UDP_PAYLOAD: usize = 1500;
pub const HEADER_SIZE: usize = 12;
pub const MAX_PLAYERS: u8 = 8;

/// Packet flags indicating transmission characteristics and payload types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PacketFlags(pub u8);

impl PacketFlags {
    pub const NONE: u8 = 0x00;
    pub const RELIABLE: u8 = 0x01;
    pub const COMPRESSED: u8 = 0x02;
    pub const SYSTEM: u8 = 0x04;
    pub const MULTIPART: u8 = 0x08;
    pub const SYNC_FRAME: u8 = 0x10;
    pub const ENCRYPTED: u8 = 0x20;
    pub const REPLAY_DATA: u8 = 0x40;
    pub const HEARTBEAT: u8 = 0x80;

    #[inline]
    pub const fn new(bits: u8) -> Self {
        Self(bits)
    }

    #[inline]
    pub const fn bits(&self) -> u8 {
        self.0
    }

    #[inline]
    pub const fn contains(&self, flag: u8) -> bool {
        (self.0 & flag) == flag
    }

    #[inline]
    pub const fn is_reliable(&self) -> bool {
        self.contains(Self::RELIABLE)
    }

    #[inline]
    pub const fn is_compressed(&self) -> bool {
        self.contains(Self::COMPRESSED)
    }

    #[inline]
    pub const fn is_sync_frame(&self) -> bool {
        self.contains(Self::SYNC_FRAME)
    }

    #[inline]
    pub const fn is_heartbeat(&self) -> bool {
        self.contains(Self::HEARTBEAT)
    }
}

impl From<u8> for PacketFlags {
    fn from(bits: u8) -> Self {
        Self(bits)
    }
}

impl From<PacketFlags> for u8 {
    fn from(flags: PacketFlags) -> Self {
        flags.0
    }
}

impl std::ops::BitOr for PacketFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOr<u8> for PacketFlags {
    type Output = Self;
    fn bitor(self, rhs: u8) -> Self {
        Self(self.0 | rhs)
    }
}

impl std::ops::BitAnd for PacketFlags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }
}

impl std::ops::BitAnd<u8> for PacketFlags {
    type Output = Self;
    fn bitand(self, rhs: u8) -> Self {
        Self(self.0 & rhs)
    }
}

/// Fixed 12-byte header present on all StarCraft UDP 6112 datagrams.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PacketHeader {
    pub magic: u32,
    pub sequence: u32,
    pub player_id: u8,
    pub flags: u8,
    pub payload_len: u16,
}

impl PacketHeader {
    pub const fn new(sequence: u32, player_id: u8, flags: u8, payload_len: u16) -> Self {
        Self {
            magic: SC_PACKET_MAGIC,
            sequence,
            player_id,
            flags,
            payload_len,
        }
    }

    #[inline]
    pub fn packet_flags(&self) -> PacketFlags {
        PacketFlags(self.flags)
    }

    /// Validates the magic constant and player identifier bounds.
    pub fn validate(&self) -> Result<()> {
        if self.magic != SC_PACKET_MAGIC {
            return Err(PacketRadarError::InvalidMagic(self.magic));
        }
        if self.player_id >= MAX_PLAYERS {
            return Err(PacketRadarError::InvalidPlayerId(self.player_id));
        }
        Ok(())
    }

    /// Parses a 12-byte header from the beginning of the slice with strict bounds checks.
    pub fn parse(raw: &[u8]) -> Result<Self> {
        if raw.len() < HEADER_SIZE {
            return Err(PacketRadarError::BufferTooShort {
                required: HEADER_SIZE,
                actual: raw.len(),
            });
        }

        let magic = u32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]);
        if magic != SC_PACKET_MAGIC {
            return Err(PacketRadarError::InvalidMagic(magic));
        }

        let sequence = u32::from_be_bytes([raw[4], raw[5], raw[6], raw[7]]);
        let player_id = raw[8];
        if player_id >= MAX_PLAYERS {
            return Err(PacketRadarError::InvalidPlayerId(player_id));
        }

        let flags = raw[9];
        let payload_len = u16::from_be_bytes([raw[10], raw[11]]);

        Ok(Self {
            magic,
            sequence,
            player_id,
            flags,
            payload_len,
        })
    }

    /// Serializes the header into a fixed 12-byte buffer without heap allocation.
    pub fn serialize(&self) -> [u8; HEADER_SIZE] {
        let mut buf = [0u8; HEADER_SIZE];
        buf[0..4].copy_from_slice(&self.magic.to_be_bytes());
        buf[4..8].copy_from_slice(&self.sequence.to_be_bytes());
        buf[8] = self.player_id;
        buf[9] = self.flags;
        buf[10..12].copy_from_slice(&self.payload_len.to_be_bytes());
        buf
    }

    /// Serializes the header directly into an existing byte vector.
    pub fn serialize_into(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.serialize());
    }
}
