//! prysmview-core - Estruturas e codecs de mensagens do protocolo PRSM

use crate::header::{Header, HeaderError, MessageType};
use crate::limits::{self, LimitError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MessageCodecError {
    #[error("Erro no cabeçalho: {0}")]
    Header(#[from] HeaderError),
    #[error("Limite de tamanho excedido: {0}")]
    LimitExceeded(#[from] LimitError),
    #[error("Erro de serialização postcard: {0}")]
    Serialization(String),
    #[error("Buffer insuficiente: esperado pelo menos {expected} bytes, recebido {actual}")]
    BufferTooShort { expected: usize, actual: usize },
}

// -----------------------------------------------------------------------------
// 1. Controle (Stream 0)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionPermissions {
    pub video: bool,
    pub input: bool,
    pub files: bool,
    pub clipboard: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlMessage {
    Handshake {
        version: u16,
        client_name: String,
    },
    HandshakeResponse {
        accepted: bool,
        version: u16,
        message: String,
    },
    SessionCodeRequest {
        operator_name: String,
        reason: String,
    },
    SessionCodeVerify {
        code: [u8; 8],
    },
    SessionCodeResult {
        valid: bool,
        attempts_left: u8,
    },
    Permissions(SessionPermissions),
    Terminate {
        reason: String,
    },
    Ping(u64),
    Pong(u64),
}

// -----------------------------------------------------------------------------
// 2. Vídeo (Host -> Viewer)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TilePacket {
    pub tile_index: u16,
    pub compressed_data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VideoFrameMessage {
    pub frame_id: u64,
    pub is_keyframe: bool,
    pub width: u16,
    pub height: u16,
    pub tiles: Vec<TilePacket>,
}

// -----------------------------------------------------------------------------
// 3. Entrada (Viewer -> Host)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseButtonType {
    Left,
    Right,
    Middle,
    Other(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputMessage {
    MouseMove {
        x: u16,
        y: u16,
    },
    MouseButton {
        button: MouseButtonType,
        pressed: bool,
    },
    MouseWheel {
        delta_v: i16,
        delta_h: i16,
    },
    Key {
        scan_code: u16,
        extended: bool,
        pressed: bool,
    },
}

// -----------------------------------------------------------------------------
// 4. Chat e Clipboard
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: u64,
    pub sender: String,
    pub text: String,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClipboardMessage {
    pub text: String,
}

// -----------------------------------------------------------------------------
// 5. Transferência de Arquivos (Blocos de até 1 MiB com SHA-256)
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileBlockMessage {
    pub transfer_id: u32,
    pub block_index: u32,
    pub total_blocks: u32,
    pub offset: u64,
    pub data: Vec<u8>,
    pub sha256: [u8; 32],
}

// -----------------------------------------------------------------------------
// Codecs e empacotamento com validação estrita de limites
// -----------------------------------------------------------------------------

/// Serializa uma mensagem para postcard e gera o frame completo com Header de 11 bytes.
pub fn encode_frame<T: Serialize>(
    msg_type: MessageType,
    msg: &T,
) -> Result<Vec<u8>, MessageCodecError> {
    let payload =
        postcard::to_allocvec(msg).map_err(|e| MessageCodecError::Serialization(e.to_string()))?;

    let payload_len = payload.len() as u32;
    limits::validate_payload_len(msg_type, payload_len)?;

    let header = Header::new(msg_type, payload_len)?;
    let mut frame = Vec::with_capacity(Header::SIZE + payload.len());
    frame.extend_from_slice(&header.encode());
    frame.extend_from_slice(&payload);
    Ok(frame)
}

/// Decodifica o cabeçalho de um frame e valida o tamanho do payload.
pub fn decode_header(buf: &[u8]) -> Result<Header, MessageCodecError> {
    if buf.len() < Header::SIZE {
        return Err(MessageCodecError::BufferTooShort {
            expected: Header::SIZE,
            actual: buf.len(),
        });
    }

    let mut header_bytes = [0u8; Header::SIZE];
    header_bytes.copy_from_slice(&buf[0..Header::SIZE]);
    let header = Header::decode(&header_bytes)?;
    Ok(header)
}

/// Deserializa o payload a partir de bytes validados contra o limite do tipo de mensagem.
pub fn decode_payload<'a, T: Deserialize<'a>>(
    msg_type: MessageType,
    payload_bytes: &'a [u8],
) -> Result<T, MessageCodecError> {
    limits::validate_payload_len(msg_type, payload_bytes.len() as u32)?;
    postcard::from_bytes(payload_bytes).map_err(|e| MessageCodecError::Serialization(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_message_roundtrip() {
        let msg = ControlMessage::SessionCodeVerify { code: *b"12345678" };
        let frame = encode_frame(MessageType::Control, &msg).expect("encode frame");
        assert!(frame.len() >= Header::SIZE);

        let header = decode_header(&frame).expect("decode header");
        assert_eq!(header.msg_type, MessageType::Control);

        let decoded: ControlMessage =
            decode_payload(MessageType::Control, &frame[Header::SIZE..]).expect("decode payload");
        assert_eq!(decoded, msg);
    }

    #[test]
    fn test_input_key_roundtrip() {
        let msg = InputMessage::Key {
            scan_code: 0x1E, // Key 'A'
            extended: false,
            pressed: true,
        };
        let frame = encode_frame(MessageType::Input, &msg).expect("encode frame");
        let header = decode_header(&frame).expect("decode header");
        assert_eq!(header.msg_type, MessageType::Input);

        let decoded: InputMessage =
            decode_payload(MessageType::Input, &frame[Header::SIZE..]).expect("decode payload");
        assert_eq!(decoded, msg);
    }

    #[test]
    fn test_video_frame_roundtrip() {
        let msg = VideoFrameMessage {
            frame_id: 42,
            is_keyframe: true,
            width: 1920,
            height: 1080,
            tiles: vec![
                TilePacket {
                    tile_index: 0,
                    compressed_data: vec![1, 2, 3, 4],
                },
                TilePacket {
                    tile_index: 10,
                    compressed_data: vec![5, 6, 7],
                },
            ],
        };
        let frame = encode_frame(MessageType::Video, &msg).expect("encode frame");
        let header = decode_header(&frame).expect("decode header");
        assert_eq!(header.msg_type, MessageType::Video);

        let decoded: VideoFrameMessage =
            decode_payload(MessageType::Video, &frame[Header::SIZE..]).expect("decode payload");
        assert_eq!(decoded, msg);
    }

    #[test]
    fn test_chat_message_roundtrip() {
        let msg = ChatMessage {
            id: 1,
            sender: "Operador".into(),
            text: "Olá do suporte técnico".into(),
            timestamp_ms: 1700000000000,
        };
        let frame = encode_frame(MessageType::Chat, &msg).expect("encode frame");
        let decoded: ChatMessage =
            decode_payload(MessageType::Chat, &frame[Header::SIZE..]).expect("decode payload");
        assert_eq!(decoded, msg);
    }

    #[test]
    fn test_file_block_roundtrip() {
        let msg = FileBlockMessage {
            transfer_id: 100,
            block_index: 0,
            total_blocks: 1,
            offset: 0,
            data: vec![0xAA; 1024],
            sha256: [0x55; 32],
        };
        let frame = encode_frame(MessageType::File, &msg).expect("encode frame");
        let decoded: FileBlockMessage =
            decode_payload(MessageType::File, &frame[Header::SIZE..]).expect("decode payload");
        assert_eq!(decoded, msg);
    }
}
