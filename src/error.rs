use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PacketRadarError {
    BufferTooShort { required: usize, actual: usize },
    InvalidMagic(u32),
    InvalidPlayerId(u8),
    MalformedTurnHeader,
    SocketError(String),
    BufferOverflow(usize),
    MalformedAction(String),
    MalformedState(String),
}

impl fmt::Display for PacketRadarError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BufferTooShort { required, actual } => {
                write!(
                    f,
                    "buffer too short: required {required} bytes, got {actual}"
                )
            }
            Self::InvalidMagic(m) => write!(f, "invalid packet magic: 0x{m:08X}"),
            Self::InvalidPlayerId(id) => write!(f, "invalid player identifier: {id}"),
            Self::MalformedTurnHeader => write!(f, "malformed StarCraft turn header"),
            Self::SocketError(msg) => write!(f, "socket I/O error: {msg}"),
            Self::BufferOverflow(len) => write!(f, "packet exceeds maximum allowed MTU: {len}"),
            Self::MalformedAction(msg) => write!(f, "malformed player action: {msg}"),
            Self::MalformedState(msg) => write!(f, "malformed game state payload: {msg}"),
        }
    }
}

impl std::error::Error for PacketRadarError {}

pub type Result<T> = std::result::Result<T, PacketRadarError>;
