//! prysmview-host - Componente Host do PDV para captura e controle remoto

pub mod capture;
pub mod encoder;
pub mod fps;
pub mod input;
pub mod net;

pub use capture::{CaptureWorker, CapturedFrame, DxgiCaptureError};
pub use encoder::{EncoderError, VideoEncoder, TILE_SIZE, ZSTD_VIDEO_LEVEL};
pub use fps::{RateController, MAX_FPS, MIN_FPS};
pub use input::{InputInjectionError, InputInjector};
pub use net::{HostNetworkAdapter, NetworkError, PRYSM_ALPN};
