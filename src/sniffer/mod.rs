use crate::error::{PacketRadarError, Result};
use crate::protocol::StarCraftPacket;
use std::net::{SocketAddr, UdpSocket};

#[derive(Debug, Clone)]
pub struct SnifferConfig {
    pub bind_addr: String,
    pub port: u16,
    pub non_blocking: bool,
}

impl Default for SnifferConfig {
    fn default() -> Self {
        Self {
            bind_addr: "127.0.0.1".to_string(),
            port: 6112,
            non_blocking: true,
        }
    }
}

pub struct PacketSniffer {
    config: SnifferConfig,
    socket: Option<UdpSocket>,
    buffer: Vec<u8>,
}

impl PacketSniffer {
    pub fn new(config: SnifferConfig) -> Self {
        Self {
            config,
            socket: None,
            buffer: vec![0u8; 2048],
        }
    }

    pub fn bind(&mut self) -> Result<()> {
        let addr = format!("{}:{}", self.config.bind_addr, self.config.port);
        let socket =
            UdpSocket::bind(&addr).map_err(|e| PacketRadarError::SocketError(e.to_string()))?;

        if self.config.non_blocking {
            socket
                .set_nonblocking(true)
                .map_err(|e| PacketRadarError::SocketError(e.to_string()))?;
        }

        self.socket = Some(socket);
        Ok(())
    }

    pub fn poll_next_packet(&mut self) -> Result<Option<(StarCraftPacket, SocketAddr)>> {
        let socket = match &self.socket {
            Some(s) => s,
            None => {
                return Err(PacketRadarError::SocketError(
                    "socket not bound; call bind() first".to_string(),
                ))
            }
        };

        match socket.recv_from(&mut self.buffer) {
            Ok((size, src)) => {
                let packet = StarCraftPacket::parse(&self.buffer[..size])?;
                Ok(Some((packet, src)))
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(PacketRadarError::SocketError(e.to_string())),
        }
    }

    pub fn parse_raw(&self, data: &[u8]) -> Result<StarCraftPacket> {
        StarCraftPacket::parse(data)
    }

    pub fn config(&self) -> &SnifferConfig {
        &self.config
    }
}
