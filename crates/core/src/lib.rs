//! prysmview-core - Protocolo, tipos, codecs e validações sem I/O para o PrysmView.

pub mod header;
pub mod limits;
pub mod messages;

pub use header::{
    Header, HeaderError, MessageType, CURRENT_PROTOCOL_VERSION, HEADER_SIZE, PROTOCOL_MAGIC,
};
pub use limits::{
    validate_payload_len, LimitError, MAX_CHAT_PAYLOAD, MAX_CONTROL_PAYLOAD, MAX_FILE_PAYLOAD,
    MAX_INPUT_PAYLOAD, MAX_VIDEO_PAYLOAD,
};
pub use messages::{
    decode_header, decode_payload, encode_frame, ChatMessage, ClipboardMessage, ControlMessage,
    FileBlockMessage, InputMessage, MessageCodecError, MouseButtonType, SessionPermissions,
    TilePacket, VideoFrameMessage,
};
