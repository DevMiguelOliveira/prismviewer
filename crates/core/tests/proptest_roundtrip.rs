//! Testes proptest de ida e volta (roundtrip) para todas as mensagens e cabeçalhos PRSM.

use proptest::prelude::*;
use prysmview_core::header::{Header, MessageType, CURRENT_PROTOCOL_VERSION};
use prysmview_core::limits::{
    MAX_CHAT_PAYLOAD, MAX_CONTROL_PAYLOAD, MAX_FILE_PAYLOAD, MAX_INPUT_PAYLOAD,
};
use prysmview_core::messages::{
    decode_header, decode_payload, encode_frame, ChatMessage, ControlMessage, FileBlockMessage,
    InputMessage, MouseButtonType, SessionPermissions, TilePacket, VideoFrameMessage,
};

proptest! {
    /// Valida que qualquer cabeçalho válido dentro dos limites é codificado em 11 bytes e decodificado com exatidão.
    #[test]
    fn prop_header_roundtrip_control(payload_len in 0u32..=MAX_CONTROL_PAYLOAD) {
        let header = Header::new(MessageType::Control, payload_len).expect("cabeçalho válido");
        let bytes = header.encode();
        prop_assert_eq!(bytes.len(), 11);

        let decoded = Header::decode(&bytes).expect("decodificação do cabeçalho");
        prop_assert_eq!(decoded, header);
        prop_assert_eq!(decoded.magic, *b"PRSM");
        prop_assert_eq!(decoded.protocol_version, CURRENT_PROTOCOL_VERSION);
        prop_assert_eq!(decoded.msg_type, MessageType::Control);
        prop_assert_eq!(decoded.payload_len, payload_len);
    }

    #[test]
    fn prop_header_roundtrip_input(payload_len in 0u32..=MAX_INPUT_PAYLOAD) {
        let header = Header::new(MessageType::Input, payload_len).expect("cabeçalho válido");
        let bytes = header.encode();
        prop_assert_eq!(bytes.len(), 11);

        let decoded = Header::decode(&bytes).expect("decodificação do cabeçalho");
        prop_assert_eq!(decoded, header);
        prop_assert_eq!(decoded.msg_type, MessageType::Input);
        prop_assert_eq!(decoded.payload_len, payload_len);
    }

    #[test]
    fn prop_header_roundtrip_chat(payload_len in 0u32..=MAX_CHAT_PAYLOAD) {
        let header = Header::new(MessageType::Chat, payload_len).expect("cabeçalho válido");
        let bytes = header.encode();
        prop_assert_eq!(bytes.len(), 11);

        let decoded = Header::decode(&bytes).expect("decodificação do cabeçalho");
        prop_assert_eq!(decoded, header);
        prop_assert_eq!(decoded.msg_type, MessageType::Chat);
        prop_assert_eq!(decoded.payload_len, payload_len);
    }

    #[test]
    fn prop_header_roundtrip_file(payload_len in 0u32..=MAX_FILE_PAYLOAD) {
        let header = Header::new(MessageType::File, payload_len).expect("cabeçalho válido");
        let bytes = header.encode();
        prop_assert_eq!(bytes.len(), 11);

        let decoded = Header::decode(&bytes).expect("decodificação do cabeçalho");
        prop_assert_eq!(decoded, header);
        prop_assert_eq!(decoded.msg_type, MessageType::File);
        prop_assert_eq!(decoded.payload_len, payload_len);
    }

    /// Valida que qualquer payload_len acima do limite é sumariamente rejeitado.
    #[test]
    fn prop_header_rejects_overflow(
        excess in 1u32..10_000u32,
    ) {
        prop_assert!(Header::new(MessageType::Input, MAX_INPUT_PAYLOAD + excess).is_err());
        prop_assert!(Header::new(MessageType::Chat, MAX_CHAT_PAYLOAD + excess).is_err());
        prop_assert!(Header::new(MessageType::Control, MAX_CONTROL_PAYLOAD + excess).is_err());
    }

    /// Valida ida e volta de mensagens de Controle com campos dinâmicos.
    #[test]
    fn prop_control_message_roundtrip(
        version in any::<u16>(),
        client_name in "[a-zA-Z0-9_-]{1,30}",
        session_code in any::<[u8; 8]>(),
        attempts in 0u8..5u8,
        video_perm in any::<bool>(),
        input_perm in any::<bool>(),
        files_perm in any::<bool>(),
        clip_perm in any::<bool>(),
    ) {
        let msgs = vec![
            ControlMessage::Handshake {
                version,
                client_name: client_name.clone(),
            },
            ControlMessage::SessionCodeVerify { code: session_code },
            ControlMessage::SessionCodeResult {
                valid: true,
                attempts_left: attempts,
            },
            ControlMessage::Permissions(SessionPermissions {
                video: video_perm,
                input: input_perm,
                files: files_perm,
                clipboard: clip_perm,
            }),
            ControlMessage::Terminate {
                reason: "Encerramento normal".into(),
            },
            ControlMessage::Ping(12345),
            ControlMessage::Pong(12345),
        ];

        for msg in msgs {
            let frame = encode_frame(MessageType::Control, &msg).expect("encode frame");
            let header = decode_header(&frame).expect("decode header");
            prop_assert_eq!(header.msg_type, MessageType::Control);

            let decoded: ControlMessage =
                decode_payload(MessageType::Control, &frame[Header::SIZE..])
                    .expect("decode payload");
            prop_assert_eq!(decoded, msg);
        }
    }

    /// Valida ida e volta de eventos de entrada (MouseMove, MouseButton, MouseWheel, Key).
    #[test]
    fn prop_input_message_roundtrip(
        x in any::<u16>(),
        y in any::<u16>(),
        button_idx in 0u8..3u8,
        button_pressed in any::<bool>(),
        wheel_v in any::<i16>(),
        wheel_h in any::<i16>(),
        scan_code in any::<u16>(),
        extended in any::<bool>(),
        key_pressed in any::<bool>(),
    ) {
        let btn = match button_idx {
            0 => MouseButtonType::Left,
            1 => MouseButtonType::Right,
            _ => MouseButtonType::Middle,
        };

        let inputs = vec![
            InputMessage::MouseMove { x, y },
            InputMessage::MouseButton {
                button: btn,
                pressed: button_pressed,
            },
            InputMessage::MouseWheel {
                delta_v: wheel_v,
                delta_h: wheel_h,
            },
            InputMessage::Key {
                scan_code,
                extended,
                pressed: key_pressed,
            },
        ];

        for input in inputs {
            let frame = encode_frame(MessageType::Input, &input).expect("encode input frame");
            let header = decode_header(&frame).expect("decode input header");
            prop_assert_eq!(header.msg_type, MessageType::Input);

            let decoded: InputMessage =
                decode_payload(MessageType::Input, &frame[Header::SIZE..])
                    .expect("decode input payload");
            prop_assert_eq!(decoded, input);
        }
    }

    /// Valida mensagens de chat dentro do limite de 4 KiB.
    #[test]
    fn prop_chat_message_roundtrip(
        id in any::<u64>(),
        sender in "[a-zA-Z0-9 ]{1,20}",
        text in "[a-zA-Z0-9 \\.,!?-]{0,200}",
        timestamp_ms in any::<u64>(),
    ) {
        let msg = ChatMessage {
            id,
            sender,
            text,
            timestamp_ms,
        };

        let frame = encode_frame(MessageType::Chat, &msg).expect("encode chat");
        let header = decode_header(&frame).expect("decode chat header");
        prop_assert_eq!(header.msg_type, MessageType::Chat);

        let decoded: ChatMessage =
            decode_payload(MessageType::Chat, &frame[Header::SIZE..]).expect("decode chat");
        prop_assert_eq!(decoded, msg);
    }

    /// Valida blocos de arquivo com hash SHA-256 de integridade.
    #[test]
    fn prop_file_block_roundtrip(
        transfer_id in any::<u32>(),
        block_index in any::<u32>(),
        total_blocks in any::<u32>(),
        offset in any::<u64>(),
        data in prop::collection::vec(any::<u8>(), 0..2048),
        sha256 in any::<[u8; 32]>(),
    ) {
        let block = FileBlockMessage {
            transfer_id,
            block_index,
            total_blocks,
            offset,
            data,
            sha256,
        };

        let frame = encode_frame(MessageType::File, &block).expect("encode file");
        let header = decode_header(&frame).expect("decode file header");
        prop_assert_eq!(header.msg_type, MessageType::File);

        let decoded: FileBlockMessage =
            decode_payload(MessageType::File, &frame[Header::SIZE..]).expect("decode file");
        prop_assert_eq!(decoded, block);
    }

    /// Valida que quadros de vídeo com múltiplos tiles comprimidos são preservados.
    #[test]
    fn prop_video_frame_roundtrip(
        frame_id in any::<u64>(),
        is_keyframe in any::<bool>(),
        width in 800u16..3840u16,
        height in 600u16..2160u16,
        tile_index in 0u16..500u16,
        tile_bytes in prop::collection::vec(any::<u8>(), 10..200),
    ) {
        let msg = VideoFrameMessage {
            frame_id,
            is_keyframe,
            width,
            height,
            tiles: vec![TilePacket {
                tile_index,
                compressed_data: tile_bytes,
            }],
        };

        let frame = encode_frame(MessageType::Video, &msg).expect("encode video");
        let header = decode_header(&frame).expect("decode video header");
        prop_assert_eq!(header.msg_type, MessageType::Video);

        let decoded: VideoFrameMessage =
            decode_payload(MessageType::Video, &frame[Header::SIZE..]).expect("decode video");
        prop_assert_eq!(decoded, msg);
    }

    /// Fuzzing determinístico: qualquer sequência arbitrária de bytes (dados corrompidos ou maliciosos)
    /// NUNCA deve provocar pânico nos decodificadores (S-15).
    #[test]
    fn prop_fuzz_arbitrary_bytes_never_panic(
        data in prop::collection::vec(any::<u8>(), 0..4096),
    ) {
        if let Ok(header) = decode_header(&data) {
            if data.len() >= Header::SIZE {
                let payload = &data[Header::SIZE..];
                // Tenta decodificar o payload de acordo com o tipo anunciado
                let _ = decode_payload::<ControlMessage>(header.msg_type, payload);
                let _ = decode_payload::<VideoFrameMessage>(header.msg_type, payload);
                let _ = decode_payload::<InputMessage>(header.msg_type, payload);
                let _ = decode_payload::<ChatMessage>(header.msg_type, payload);
                let _ = decode_payload::<FileBlockMessage>(header.msg_type, payload);
            }
        }
    }
}

