//! Suíte de Testes de Segurança e Invasão do PrysmView (Fase 12)
//!
//! Validação contra o Threat Model (docs/threat-model.md e Seção 4/9 do AGENTS.md):
//! - Rejeição prévia de estouro de buffer / DoS por payload excessivo
//! - Rejeição de magic bytes corrompidos ou desconhecidos
//! - Rejeição de versões de protocolo incompatíveis
//! - Rejeição de tipos de mensagem forjados ou desconhecidos
//! - Validação estrita de limites de mensagens antes de alocação

use prysmview_core::header::{
    Header, HeaderError, MessageType, CURRENT_PROTOCOL_VERSION, PROTOCOL_MAGIC,
};
use prysmview_core::limits::{
    validate_payload_len, LimitError, MAX_CHAT_PAYLOAD, MAX_CONTROL_PAYLOAD, MAX_FILE_PAYLOAD,
    MAX_INPUT_PAYLOAD, MAX_VIDEO_PAYLOAD,
};

#[test]
fn test_attack_dos_payload_overflow_rejected_before_allocation() {
    // 1. Ataque de overflow em mensagem de Controle (Limite: 64 KiB)
    let attack_control_len = MAX_CONTROL_PAYLOAD + 1;
    let err = validate_payload_len(MessageType::Control, attack_control_len);
    assert_eq!(
        err,
        Err(LimitError::PayloadTooLarge(
            MessageType::Control,
            attack_control_len,
            MAX_CONTROL_PAYLOAD
        ))
    );

    // 2. Ataque de overflow em frame de Vídeo (Limite: 8 MiB)
    let attack_video_len = MAX_VIDEO_PAYLOAD + 1;
    let err = validate_payload_len(MessageType::Video, attack_video_len);
    assert_eq!(
        err,
        Err(LimitError::PayloadTooLarge(
            MessageType::Video,
            attack_video_len,
            MAX_VIDEO_PAYLOAD
        ))
    );

    // 3. Ataque de overflow em comando de Entrada (Limite: 256 B)
    let attack_input_len = MAX_INPUT_PAYLOAD + 1;
    let err = validate_payload_len(MessageType::Input, attack_input_len);
    assert_eq!(
        err,
        Err(LimitError::PayloadTooLarge(
            MessageType::Input,
            attack_input_len,
            MAX_INPUT_PAYLOAD
        ))
    );

    // 4. Ataque de overflow em Chat (Limite: 4 KiB)
    let attack_chat_len = MAX_CHAT_PAYLOAD + 1;
    let err = validate_payload_len(MessageType::Chat, attack_chat_len);
    assert_eq!(
        err,
        Err(LimitError::PayloadTooLarge(
            MessageType::Chat,
            attack_chat_len,
            MAX_CHAT_PAYLOAD
        ))
    );

    // 5. Ataque de overflow em bloco de Arquivo (Limite: 1 MiB)
    let attack_file_len = MAX_FILE_PAYLOAD + 1;
    let err = validate_payload_len(MessageType::File, attack_file_len);
    assert_eq!(
        err,
        Err(LimitError::PayloadTooLarge(
            MessageType::File,
            attack_file_len,
            MAX_FILE_PAYLOAD
        ))
    );
}

#[test]
fn test_attack_header_decoding_with_forged_payload_len() {
    // Cabeçalho forjado com magic PRSM, versão v1, tipo Input (limite 256 bytes),
    // porém declarando 50 MB de payload no campo length (tentativa de exaustão de RAM).
    let mut buf = [0u8; 11];
    buf[0..4].copy_from_slice(&PROTOCOL_MAGIC);
    buf[4..6].copy_from_slice(&CURRENT_PROTOCOL_VERSION.to_be_bytes());
    buf[6] = MessageType::Input as u8;
    buf[7..11].copy_from_slice(&(50 * 1024 * 1024u32).to_be_bytes());

    let decode_result = Header::decode(&buf);

    // O decodificador DEVE rejeitar antes de qualquer tentativa de alocação de buffer
    assert!(matches!(decode_result, Err(HeaderError::LimitExceeded(_))));
}

#[test]
fn test_attack_invalid_magic_bytes_dropped_immediately() {
    let bad_magics: [[u8; 4]; 4] = [*b"HTTP", *b"GET ", *b"SSH-", [0, 0, 0, 0]];

    for magic in &bad_magics {
        let mut buf = [0u8; 11];
        buf[0..4].copy_from_slice(magic);
        buf[4..6].copy_from_slice(&CURRENT_PROTOCOL_VERSION.to_be_bytes());
        buf[6] = MessageType::Control as u8;
        buf[7..11].copy_from_slice(&100u32.to_be_bytes());

        let res = Header::decode(&buf);
        assert_eq!(res, Err(HeaderError::InvalidMagic(*magic)));
    }
}

#[test]
fn test_attack_unsupported_protocol_version() {
    let unsupported_versions = [0u16, 2u16, 100u16, 65535u16];

    for ver in unsupported_versions {
        let mut buf = [0u8; 11];
        buf[0..4].copy_from_slice(&PROTOCOL_MAGIC);
        buf[4..6].copy_from_slice(&ver.to_be_bytes());
        buf[6] = MessageType::Control as u8;
        buf[7..11].copy_from_slice(&64u32.to_be_bytes());

        let res = Header::decode(&buf);
        assert_eq!(res, Err(HeaderError::UnsupportedVersion(ver)));
    }
}

#[test]
fn test_attack_unknown_message_type_rejected() {
    let unknown_types = [5u8, 6u8, 42u8, 255u8];

    for t in unknown_types {
        let mut buf = [0u8; 11];
        buf[0..4].copy_from_slice(&PROTOCOL_MAGIC);
        buf[4..6].copy_from_slice(&CURRENT_PROTOCOL_VERSION.to_be_bytes());
        buf[6] = t;
        buf[7..11].copy_from_slice(&64u32.to_be_bytes());

        let res = Header::decode(&buf);
        assert_eq!(res, Err(HeaderError::UnknownMessageType(t)));
    }
}
