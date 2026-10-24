//! prysmview-core - Protocolo, tipos e validações sem I/O para o PrysmView.

pub const PROTOCOL_MAGIC: &[u8; 4] = b"PRSM";
pub const PROTOCOL_VERSION: u16 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    Control = 0,
    Video = 1,
    Input = 2,
    Chat = 3,
    File = 4,
}
