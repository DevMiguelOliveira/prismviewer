//! prysmview-host - Componente Host do PDV para captura e controle remoto

pub mod capture;
pub mod chat;
pub mod clipboard;
pub mod encoder;
pub mod file_transfer;
pub mod fps;
pub mod input;
pub mod ipc;
pub mod net;
pub mod sas;
pub mod service;
pub mod session_code;
pub mod signaling;
pub mod updater;

pub use capture::{CaptureWorker, CapturedFrame, DxgiCaptureError};
pub use chat::{ChatError, ChatManager, MAX_CHAT_HISTORY, MAX_CHAT_TEXT_LEN};
pub use clipboard::{ClipboardError, ClipboardManager, MAX_CLIPBOARD_BYTES};
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
pub use sas::{
    clear_reconnect_state, load_reconnect_state, release_all_keys, request_system_reboot,
    save_reconnect_state, send_sas, ReconnectState, SasError, DEFAULT_RECONNECT_STATE_PATH,
};
pub use service::{
    run_service, run_standalone, ServiceError, SERVICE_DESCRIPTION, SERVICE_DISPLAY_NAME,
    SERVICE_NAME,
};
pub use session_code::{
    SessionCodeError, SessionCodeManager, MAX_SESSION_CODE_ATTEMPTS, SESSION_CODE_TTL,
};
pub use signaling::{
    build_signed_message, to_hex, HostRegisterPayload, HostSignalingClient, PdvStatusResponse,
    SignalingError,
};
pub use updater::{
    verify_package_integrity, UpdateBackupGuard, UpdateChannel, UpdateError, UpdateManifest,
};
