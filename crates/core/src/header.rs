//! prysmview-core - Cabeçalho do protocolo PRSM
//!
//! Todo quadro transmitido na rede inicia com um cabeçalho fixo de 11 bytes:
//! - magic: 4 bytes ASCII "PRSM"
//! - version: u16 big-endian
//! - msg_type: u8 (0=Control, 1=Video, 2=Input, 3=Chat, 4=File)
//! - payload_len: u32 big-endian

use crate::limits::{self, LimitError};
use thiserror::Error;

pub const PROTOCOL_MAGIC: [u8; 4] = *b"PRSM";
pub const CURRENT_PROTOCOL_VERSION: u16 = 1;
pub const HEADER_SIZE: usize = 11;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MessageType {
    Control = 0,
    Video = 1,
    Input = 2,
    Chat = 3,
    File = 4,
}

impl TryFrom<u8> for MessageType {
    type Error = HeaderError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(Self::Control),
            1 => Ok(Self::Video),
            2 => Ok(Self::Input),
            3 => Ok(Self::Chat),
            4 => Ok(Self::File),
            other => Err(HeaderError::UnknownMessageType(other)),
        }
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum HeaderError {
    #[error("Magic inválido: esperado b\"PRSM\", recebido {0:?}")]
    InvalidMagic([u8; 4]),
    #[error("Versão de protocolo incompatível: {0}")]
    UnsupportedVersion(u16),
    #[error("Tipo de mensagem desconhecido: {0}")]
    UnknownMessageType(u8),
    #[error("Limite de payload excedido: {0}")]
    LimitExceeded(#[from] LimitError),
}

/// Cabeçalho fixo de 11 bytes de todo pacote PRSM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Header {
    pub magic: [u8; 4],
    pub protocol_version: u16,
    pub msg_type: MessageType,
    pub payload_len: u32,
}

impl Header {
    pub const SIZE: usize = HEADER_SIZE;

    /// Cria um novo cabeçalho válido com a versão atual do protocolo.
    pub fn new(msg_type: MessageType, payload_len: u32) -> Result<Self, HeaderError> {
        limits::validate_payload_len(msg_type, payload_len)?;
        Ok(Self {
            magic: PROTOCOL_MAGIC,
            protocol_version: CURRENT_PROTOCOL_VERSION,
            msg_type,
            payload_len,
        })
    }

    /// Codifica o cabeçalho em 11 bytes (big-endian).
    pub fn encode(&self) -> [u8; 11] {
        let mut buf = [0u8; 11];
        buf[0..4].copy_from_slice(&self.magic);
        buf[4..6].copy_from_slice(&self.protocol_version.to_be_bytes());
        buf[6] = self.msg_type as u8;
        buf[7..11].copy_from_slice(&self.payload_len.to_be_bytes());
        buf
    }

    /// Decodifica e valida um cabeçalho de 11 bytes.
    /// Rejeita magics inválidos, versões incompatíveis e tamanhos acima do limite.
    pub fn decode(buf: &[u8; 11]) -> Result<Self, HeaderError> {
        let magic = [buf[0], buf[1], buf[2], buf[3]];
        if magic != PROTOCOL_MAGIC {
            return Err(HeaderError::InvalidMagic(magic));
        }

        let protocol_version = u16::from_be_bytes([buf[4], buf[5]]);
        if protocol_version != CURRENT_PROTOCOL_VERSION {
            return Err(HeaderError::UnsupportedVersion(protocol_version));
        }

        let msg_type = MessageType::try_from(buf[6])?;
        let payload_len = u32::from_be_bytes([buf[7], buf[8], buf[9], buf[10]]);

        limits::validate_payload_len(msg_type, payload_len)?;

        Ok(Self {
            magic,
            protocol_version,
            msg_type,
            payload_len,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_valid() {
        let h = Header::new(MessageType::Control, 1024).expect("header válido");
        let bytes = h.encode();
        assert_eq!(bytes.len(), 11);
        assert_eq!(&bytes[0..4], b"PRSM");

        let decoded = Header::decode(&bytes).expect("decode com sucesso");
        assert_eq!(decoded, h);
    }

    #[test]
    fn test_invalid_magic() {
        let mut bytes = [0u8; 11];
        bytes[0..4].copy_from_slice(b"NOPE");
        bytes[4..6].copy_from_slice(&1u16.to_be_bytes());
        bytes[6] = 0;
        bytes[7..11].copy_from_slice(&100u32.to_be_bytes());

        let res = Header::decode(&bytes);
        assert!(matches!(res, Err(HeaderError::InvalidMagic(_))));
    }

    #[test]
    fn test_unsupported_version() {
        let mut bytes = [0u8; 11];
        bytes[0..4].copy_from_slice(b"PRSM");
        bytes[4..6].copy_from_slice(&999u16.to_be_bytes());
        bytes[6] = 0;
        bytes[7..11].copy_from_slice(&100u32.to_be_bytes());

        let res = Header::decode(&bytes);
        assert!(matches!(res, Err(HeaderError::UnsupportedVersion(999))));
    }

    #[test]
    fn test_unknown_msg_type() {
        let mut bytes = [0u8; 11];
        bytes[0..4].copy_from_slice(b"PRSM");
        bytes[4..6].copy_from_slice(&1u16.to_be_bytes());
        bytes[6] = 99;
        bytes[7..11].copy_from_slice(&100u32.to_be_bytes());

        let res = Header::decode(&bytes);
        assert!(matches!(res, Err(HeaderError::UnknownMessageType(99))));
    }

    #[test]
    fn test_exceeding_payload_rejected_in_header() {
        // Controle não pode ter mais de 64 KiB
        let mut bytes = [0u8; 11];
        bytes[0..4].copy_from_slice(b"PRSM");
        bytes[4..6].copy_from_slice(&1u16.to_be_bytes());
        bytes[6] = MessageType::Control as u8;
        bytes[7..11].copy_from_slice(&(65536u32 + 1).to_be_bytes());

        let res = Header::decode(&bytes);
        assert!(matches!(res, Err(HeaderError::LimitExceeded(_))));
    }
}
