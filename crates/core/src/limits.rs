//! Limites estritos de tamanho de mensagem para evitar ataques de esgotamento de memória (DoS).

use crate::header::MessageType;
use thiserror::Error;

/// Limite máximo para mensagens de controle: 64 KiB
pub const MAX_CONTROL_PAYLOAD: u32 = 64 * 1024;

/// Limite máximo para um quadro de vídeo: 8 MiB
pub const MAX_VIDEO_PAYLOAD: u32 = 8 * 1024 * 1024;

/// Limite máximo para comandos de entrada (mouse/teclado): 256 bytes
pub const MAX_INPUT_PAYLOAD: u32 = 256;

/// Limite máximo para mensagens de chat / clipboard texto: 4 KiB
pub const MAX_CHAT_PAYLOAD: u32 = 4 * 1024;

/// Limite máximo para um bloco de transferência de arquivo: 1 MiB
pub const MAX_FILE_PAYLOAD: u32 = 1024 * 1024;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LimitError {
    #[error("Payload excede o limite máximo para {0:?}: recebido {1} bytes, limite é {2} bytes")]
    PayloadTooLarge(MessageType, u32, u32),
}

/// Valida o tamanho do payload antes de qualquer alocação de buffer em memória.
pub fn validate_payload_len(msg_type: MessageType, len: u32) -> Result<(), LimitError> {
    let max_len = match msg_type {
        MessageType::Control => MAX_CONTROL_PAYLOAD,
        MessageType::Video => MAX_VIDEO_PAYLOAD,
        MessageType::Input => MAX_INPUT_PAYLOAD,
        MessageType::Chat => MAX_CHAT_PAYLOAD,
        MessageType::File => MAX_FILE_PAYLOAD,
    };

    if len > max_len {
        Err(LimitError::PayloadTooLarge(msg_type, len, max_len))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_limits() {
        assert!(validate_payload_len(MessageType::Control, 64 * 1024).is_ok());
        assert!(validate_payload_len(MessageType::Video, 8 * 1024 * 1024).is_ok());
        assert!(validate_payload_len(MessageType::Input, 256).is_ok());
        assert!(validate_payload_len(MessageType::Chat, 4096).is_ok());
        assert!(validate_payload_len(MessageType::File, 1024 * 1024).is_ok());
    }

    #[test]
    fn test_exceeded_limits() {
        assert!(validate_payload_len(MessageType::Control, 64 * 1024 + 1).is_err());
        assert!(validate_payload_len(MessageType::Video, 8 * 1024 * 1024 + 1).is_err());
        assert!(validate_payload_len(MessageType::Input, 257).is_err());
        assert!(validate_payload_len(MessageType::Chat, 4097).is_err());
        assert!(validate_payload_len(MessageType::File, 1024 * 1024 + 1).is_err());
    }
}
