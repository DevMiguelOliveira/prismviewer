//! prysmview-host - Componente Host do PDV para captura e controle remoto

pub mod capture;

pub use capture::{CaptureWorker, CapturedFrame, DxgiCaptureError};
