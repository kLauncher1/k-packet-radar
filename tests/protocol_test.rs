use k_packet_radar::{
    ActionType, BaseState, PacketFlags, PacketHeader, PacketRadarError, PlayerState,
    StarCraftPacket, SyncFrame, TurnFrame, UnitState, UnitStatusFlags, HEADER_SIZE,
    MAX_UDP_PAYLOAD, SC_PACKET_MAGIC,
};

#[test]
fn test_packet_header_parsing_and_validation() {
    let header = PacketHeader::new(42, 3, PacketFlags::RELIABLE | PacketFlags::HEARTBEAT, 120);
    assert!(header.validate().is_ok());

    let serialized = header.serialize();
    assert_eq!(serialized.len(), HEADER_SIZE);

    let parsed = PacketHeader::parse(&serialized).expect("valid header parse");
    assert_eq!(parsed.magic, SC_PACKET_MAGIC);
    assert_eq!(parsed.sequence, 42);
    assert_eq!(parsed.player_id, 3);
    assert_eq!(parsed.flags, 0x81);
    assert_eq!(parsed.payload_len, 120);

    let flags = parsed.packet_flags();
    assert!(flags.is_reliable());
    assert!(flags.is_heartbeat());
    assert!(!flags.is_compressed());
}

#[test]
fn test_header_error_handling() {
    // 1. Invalid Magic
    let mut raw = vec![0u8; 16];
    raw[0..4].copy_from_slice(&0xDEADBEEFu32.to_be_bytes());
    assert_eq!(
        PacketHeader::parse(&raw),
        Err(PacketRadarError::InvalidMagic(0xDEADBEEF))
    );

    // 2. Invalid Player ID
    let mut valid = PacketHeader::new(1, 0, 0, 0).serialize().to_vec();
    valid[8] = 8; // Valid player IDs are 0..=7
    assert_eq!(
        PacketHeader::parse(&valid),
        Err(PacketRadarError::InvalidPlayerId(8))
    );

    // 3. Buffer Too Short
    let short_buf = [0u8; 8];
    assert_eq!(
        PacketHeader::parse(&short_buf),
        Err(PacketRadarError::BufferTooShort {
            required: HEADER_SIZE,
            actual: 8,
        })
    );

    // 4. MTU Overflow on StarCraftPacket
    let huge_packet = vec![0u8; MAX_UDP_PAYLOAD + 1];
    assert_eq!(
        StarCraftPacket::parse(&huge_packet),
        Err(PacketRadarError::BufferOverflow(MAX_UDP_PAYLOAD + 1))
    );
}

#[test]
fn test_all_16_action_variants_roundtrip() {
    let actions = vec![
        ActionType::Ping,
        ActionType::Move { x: 1024, y: 2048 },
        ActionType::Attack { target_id: 42 },
        ActionType::Build {
            unit_type: 106,
            x: 500,
            y: 600,
        },
        ActionType::CastSpell {
            spell_id: 8,
            target_id: 99,
        },
        ActionType::TrainUnit { unit_type: 7 },
        ActionType::Stop,
        ActionType::HoldPosition,
        ActionType::AttackMove { x: 3500, y: 3200 },
        ActionType::SelectUnits {
            unit_ids: vec![101, 102, 103, 104, 105, 106],
        },
        ActionType::RallyPoint { x: 1200, y: 1400 },
        ActionType::LiftOff { x: 800, y: 900 },
        ActionType::Land { x: 850, y: 950 },
        ActionType::ResearchTech { tech_id: 15 },
        ActionType::Upgrade { upgrade_id: 3 },
        ActionType::ChatMessage {
            text: "gg gl hf!".to_string(),
        },
    ];

    assert_eq!(actions.len(), 16);

    for original in &actions {
        let serialized = original.serialize();
        assert_eq!(serialized.len(), original.serialized_size());

        let (parsed, consumed) = ActionType::parse(&serialized).expect("action parse must succeed");
        assert_eq!(consumed, serialized.len());
        assert_eq!(&parsed, original);
        assert_eq!(parsed.opcode(), original.opcode());
        assert_eq!(parsed.name(), original.name());
    }
}

#[test]
fn test_turn_frame_action_batches_and_sync() {
    let actions = vec![
        ActionType::Move { x: 100, y: 200 },
        ActionType::Attack { target_id: 55 },
        ActionType::HoldPosition,
        ActionType::ChatMessage {
            text: "scout coming".to_string(),
        },
    ];

    let frame = TurnFrame::new(1250, 2, actions.clone());
    let serialized = frame.serialize();

    let parsed_frame = TurnFrame::parse(&serialized).expect("valid turn frame parse");
    assert_eq!(parsed_frame.turn_index, 1250);
    assert_eq!(parsed_frame.latency_turns, 2);
    assert_eq!(parsed_frame.actions, actions);

    // Sync hash and SyncFrame verification
    let sync_hash = frame.compute_sync_hash();
    assert_ne!(sync_hash, 0);

    let sync_frame = frame.to_sync_frame(0x1234_5678, 2);
    assert_eq!(sync_frame.turn_index, 1250);
    assert_eq!(sync_frame.checksum, sync_hash);
    assert_eq!(sync_frame.seed, 0x1234_5678);
    assert_eq!(sync_frame.active_players, 2);

    let sync_raw = sync_frame.serialize();
    let parsed_sync = SyncFrame::parse(&sync_raw).expect("sync parse must succeed");
    assert_eq!(parsed_sync, sync_frame);
}

#[test]
fn test_unit_state_parsing_and_status_flags() {
    let mut flags = UnitStatusFlags::NONE;
    assert!(!flags.is_cloaked());
    assert!(!flags.is_sieged());

    flags = UnitStatusFlags::new(
        UnitStatusFlags::CLOAKED
            | UnitStatusFlags::SIEGED
            | UnitStatusFlags::DEFENSIVE_MATRIX
            | UnitStatusFlags::STIMMED,
    );
    assert!(flags.is_cloaked());
    assert!(flags.is_sieged());
    assert!(flags.has_defensive_matrix());
    assert!(flags.is_stimmed());
    assert!(!flags.is_burrowed());
    assert!(!flags.is_hallucination());

    let unit = UnitState {
        unit_id: 1024,
        player_id: 1,
        unit_type: 30, // Terran Siege Tank
        x: 1800,
        y: 2200,
        hp: 150,
        shields: 0,
        energy: 0,
        status_flags: flags,
        target_id: 42,
    };

    let serialized = unit.serialize();
    assert_eq!(serialized.len(), UnitState::SERIALIZED_SIZE);

    let parsed = UnitState::parse(&serialized).expect("unit state parse");
    assert_eq!(parsed, unit);
    assert!(parsed.status_flags.is_sieged());

    // Defensive validation
    let mut invalid_player = serialized;
    invalid_player[2] = 8;
    assert_eq!(
        UnitState::parse(&invalid_player),
        Err(PacketRadarError::InvalidPlayerId(8))
    );

    assert_eq!(
        UnitState::parse(&serialized[..10]),
        Err(PacketRadarError::BufferTooShort {
            required: UnitState::SERIALIZED_SIZE,
            actual: 10,
        })
    );
}

#[test]
fn test_base_and_player_state_parsing() {
    // 1. BaseState
    let base = BaseState {
        base_id: 1,
        player_id: 0,
        center_x: 64,
        center_y: 128,
        minerals: 1500,
        gas: 800,
        worker_count: 18,
        is_main: true,
        is_active: true,
    };
    let base_raw = base.serialize();
    assert_eq!(base_raw.len(), BaseState::SERIALIZED_SIZE);

    let parsed_base = BaseState::parse(&base_raw).expect("base state parse");
    assert_eq!(parsed_base, base);
    assert!(parsed_base.is_main);
    assert!(parsed_base.is_active);

    // 2. PlayerState
    let player = PlayerState {
        player_id: 0,
        race: 2, // Protoss
        minerals: 450,
        gas: 200,
        supply_used: 64,
        supply_total: 100,
        apm: 245,
        units_alive: 28,
        buildings_alive: 12,
    };
    let player_raw = player.serialize();
    assert_eq!(player_raw.len(), PlayerState::SERIALIZED_SIZE);

    let parsed_player = PlayerState::parse(&player_raw).expect("player state parse");
    assert_eq!(parsed_player, player);
}

#[test]
fn test_malformed_and_truncated_payload_defenses() {
    // 1. Truncated Move action (opcode 0x01 needs 4 payload bytes, only 2 provided)
    let truncated_move = [0x01, 0x00, 0x10];
    assert!(matches!(
        ActionType::parse(&truncated_move),
        Err(PacketRadarError::BufferTooShort { .. })
    ));

    // 2. Truncated SelectUnits
    let truncated_select = [0x09, 0x04, 0x00, 0x01]; // says 4 units, but only provides 1 unit id
    assert!(matches!(
        ActionType::parse(&truncated_select),
        Err(PacketRadarError::BufferTooShort { .. })
    ));

    // 3. Excessive SelectUnits count (> 32)
    let excessive_select = [0x09, 33];
    assert!(matches!(
        ActionType::parse(&excessive_select),
        Err(PacketRadarError::MalformedAction(_))
    ));

    // 4. Excessive ChatMessage length (> 128)
    let excessive_chat = [0x0F, 129];
    assert!(matches!(
        ActionType::parse(&excessive_chat),
        Err(PacketRadarError::MalformedAction(_))
    ));

    // 5. Invalid UTF-8 in ChatMessage
    let invalid_utf8_chat = [0x0F, 0x02, 0xFF, 0xFE];
    assert!(matches!(
        ActionType::parse(&invalid_utf8_chat),
        Err(PacketRadarError::MalformedAction(_))
    ));

    // 6. Unknown opcode
    let unknown_opcode = [0xFE, 0x00, 0x01];
    assert!(matches!(
        ActionType::parse(&unknown_opcode),
        Err(PacketRadarError::MalformedAction(_))
    ));
}

#[test]
fn test_complete_starcraft_packet_roundtrip() {
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
