//! prysmview-host - Componente Host do PDV para captura e controle remoto

pub mod capture;
pub mod encoder;

pub use capture::{CaptureWorker, CapturedFrame, DxgiCaptureError};
pub use encoder::{EncoderError, VideoEncoder, TILE_SIZE, ZSTD_VIDEO_LEVEL};
