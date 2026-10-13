use k_packet_radar::{
    ActionType, PacketHeader, PacketRadarError, StarCraftPacket, TurnFrame, SC_PACKET_MAGIC,
};

#[test]
fn test_packet_serialize_and_parse_roundtrip() {
    let packet = StarCraftPacket {
        header: PacketHeader {
            magic: SC_PACKET_MAGIC,
            sequence: 42,
            player_id: 3,
            flags: 0x80,
            payload_len: 11,
        },
        turn: Some(TurnFrame {
            turn_index: 999,
            latency_turns: 3,
            actions: vec![
                ActionType::Move { x: 300, y: 400 },
                ActionType::Attack { target_id: 15 },
            ],
        }),
    };

    let serialized = packet.serialize();
    let parsed = StarCraftPacket::parse(&serialized).expect("valid parse");

    assert_eq!(parsed.header.sequence, 42);
    assert_eq!(parsed.header.player_id, 3);
    assert_eq!(parsed.header.flags, 0x80);

    let turn = parsed.turn.expect("must contain turn");
    assert_eq!(turn.turn_index, 999);
    assert_eq!(turn.latency_turns, 3);
    assert_eq!(turn.actions.len(), 2);
    assert_eq!(turn.actions[0], ActionType::Move { x: 300, y: 400 });
    assert_eq!(turn.actions[1], ActionType::Attack { target_id: 15 });
}

#[test]
fn test_invalid_magic_rejected() {
    let mut raw = vec![0u8; 16];
    raw[0..4].copy_from_slice(&0x12345678u32.to_be_bytes());
    assert_eq!(
        StarCraftPacket::parse(&raw),
        Err(PacketRadarError::InvalidMagic(0x12345678))
    );
}

#[test]
fn test_buffer_too_short_rejected() {
    let raw = vec![0u8; 8];
    assert_eq!(
        StarCraftPacket::parse(&raw),
        Err(PacketRadarError::BufferTooShort {
            required: 12,
            actual: 8,
        })
    );
}

#[test]
fn test_invalid_player_id_rejected() {
    let mut raw = vec![0u8; 16];
    raw[0..4].copy_from_slice(&SC_PACKET_MAGIC.to_be_bytes());
    raw[8] = 9; // Player IDs must be 0..7
    assert_eq!(
        StarCraftPacket::parse(&raw),
        Err(PacketRadarError::InvalidPlayerId(9))
    );
}
