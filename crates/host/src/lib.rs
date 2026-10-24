//! prysmview-host - Componente Host do PDV para captura e controle remoto

pub mod capture;
pub mod encoder;
pub mod fps;

pub use capture::{CaptureWorker, CapturedFrame, DxgiCaptureError};
pub use encoder::{EncoderError, VideoEncoder, TILE_SIZE, ZSTD_VIDEO_LEVEL};
pub use fps::{RateController, MAX_FPS, MIN_FPS};
