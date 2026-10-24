//! prysmview-host - Componente Host do PDV para captura e controle remoto

pub mod capture;
pub mod encoder;
pub mod file_transfer;
pub mod fps;
pub mod input;
pub mod ipc;
pub mod net;
pub mod session_code;

pub use capture::{CaptureWorker, CapturedFrame, DxgiCaptureError};
pub use encoder::{EncoderError, VideoEncoder, TILE_SIZE, ZSTD_VIDEO_LEVEL};
pub use file_transfer::{FileTransferError, FileTransferReceiver, FileTransferSender};
pub use fps::{RateController, MAX_FPS, MIN_FPS};
pub use input::{InputInjectionError, InputInjector};
#[cfg(windows)]
pub use ipc::{connect_pipe_client, create_secure_pipe_server};
pub use ipc::{
    decode_ipc_frame, encode_ipc_frame, recv_ipc_message, send_ipc_message, IpcError, IpcMessage,
    DEFAULT_PIPE_NAME, PIPE_SECURITY_DESCRIPTOR_SDDL,
};
pub use net::{HostNetworkAdapter, NetworkError, PRYSM_ALPN};
pub use session_code::{
    SessionCodeError, SessionCodeManager, MAX_SESSION_CODE_ATTEMPTS, SESSION_CODE_TTL,
};
