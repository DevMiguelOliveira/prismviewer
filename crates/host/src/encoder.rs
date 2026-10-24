//! prysmview-host - Pipeline de codificação de vídeo com tiles 64x64 e compressão zstd (nível 1).
//!
//! Conforme a especificação do AGENTS.md (seções 6.1 e 8):
//! - Tiles fixos de 64x64 pixels.
//! - Detecção rápida de alterações (dirty tiles) contra o buffer de referência do frame anterior.
//! - Compressão com zstd no nível 1 para priorizar latência baixa em redes modestas.
//! - Empacotamento em `VideoFrameMessage` e `TilePacket` do `prysmview-core`.

use prysmview_core::messages::{TilePacket, VideoFrameMessage};
use std::io;
use thiserror::Error;

pub const TILE_SIZE: usize = 64;
pub const ZSTD_VIDEO_LEVEL: i32 = 1;

#[derive(Debug, Error)]
pub enum EncoderError {
    #[error("Erro de compressão zstd: {0}")]
    Compression(#[from] io::Error),
    #[error("Buffer de imagem com tamanho inválido: esperado {expected}, recebido {actual}")]
    InvalidBufferSize { expected: usize, actual: usize },
}

/// Codificador de vídeo incremental por tiles 64x64.
pub struct VideoEncoder {
    width: u32,
    height: u32,
    cols: usize,
    rows: usize,
    total_tiles: usize,
    prev_tiles: Vec<Option<Vec<u8>>>,
    frame_counter: u64,
}

impl VideoEncoder {
    /// Inicializa um novo codificador para a resolução de tela informada.
    pub fn new(width: u32, height: u32) -> Self {
        let cols = (width as usize).div_ceil(TILE_SIZE);
        let rows = (height as usize).div_ceil(TILE_SIZE);
        let total_tiles = cols * rows;

        Self {
            width,
            height,
            cols,
            rows,
            total_tiles,
            prev_tiles: vec![None; total_tiles],
            frame_counter: 0,
        }
    }

    pub fn total_tiles(&self) -> usize {
        self.total_tiles
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Processa um quadro bruto BGRA (width * height * 4 bytes).
    /// Se `force_keyframe` for verdadeiro, comprime e envia todos os tiles da tela.
    pub fn encode_frame(
        &mut self,
        bgra_data: &[u8],
        force_keyframe: bool,
    ) -> Result<VideoFrameMessage, EncoderError> {
        let expected_size = (self.width * self.height * 4) as usize;
        if bgra_data.len() < expected_size {
            return Err(EncoderError::InvalidBufferSize {
                expected: expected_size,
                actual: bgra_data.len(),
            });
        }

        self.frame_counter += 1;
        let is_keyframe = force_keyframe || self.prev_tiles.iter().all(|t| t.is_none());
        let width = self.width as usize;
        let height = self.height as usize;
        let stride = width * 4;

        let mut tiles = Vec::new();

        for ty in 0..self.rows {
            for tx in 0..self.cols {
                let tile_idx = ty * self.cols + tx;
                let x0 = tx * TILE_SIZE;
                let y0 = ty * TILE_SIZE;
                let tw = TILE_SIZE.min(width.saturating_sub(x0));
                let th = TILE_SIZE.min(height.saturating_sub(y0));

                let mut tile_pixels = Vec::with_capacity(tw * th * 4);
                for y in 0..th {
                    let row_start = (y0 + y) * stride + x0 * 4;
                    let row_end = row_start + tw * 4;
                    tile_pixels.extend_from_slice(&bgra_data[row_start..row_end]);
                }

                let changed = if is_keyframe {
                    true
                } else {
                    match &self.prev_tiles[tile_idx] {
                        Some(prev) => prev != &tile_pixels,
                        None => true,
                    }
                };

                if changed {
                    let compressed = zstd::bulk::compress(&tile_pixels, ZSTD_VIDEO_LEVEL)?;
                    tiles.push(TilePacket {
                        tile_index: tile_idx as u16,
                        compressed_data: compressed,
                    });
                    self.prev_tiles[tile_idx] = Some(tile_pixels);
                }
            }
        }

        Ok(VideoFrameMessage {
            frame_id: self.frame_counter,
            is_keyframe,
            width: self.width as u16,
            height: self.height as u16,
            tiles,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_frame_is_keyframe() {
        let mut encoder = VideoEncoder::new(128, 128); // 2x2 = 4 tiles
        assert_eq!(encoder.total_tiles(), 4);

        let dummy_screen = vec![0xAA; 128 * 128 * 4];
        let msg = encoder
            .encode_frame(&dummy_screen, false)
            .expect("encode frame 1");

        assert_eq!(msg.frame_id, 1);
        assert!(msg.is_keyframe);
        assert_eq!(msg.tiles.len(), 4);
    }

    #[test]
    fn test_static_screen_zero_dirty_tiles() {
        let mut encoder = VideoEncoder::new(128, 128);
        let dummy_screen = vec![0x55; 128 * 128 * 4];

        // Frame 1: Keyframe inicial
        let _ = encoder.encode_frame(&dummy_screen, false).unwrap();

        // Frame 2: Tela sem nenhuma alteração
        let msg2 = encoder
            .encode_frame(&dummy_screen, false)
            .expect("encode frame 2");
        assert_eq!(msg2.frame_id, 2);
        assert!(!msg2.is_keyframe);
        assert_eq!(msg2.tiles.len(), 0, "tela estática não deve enviar tiles");
    }

    #[test]
    fn test_partial_dirty_tile_detection() {
        let mut encoder = VideoEncoder::new(128, 128); // 4 tiles: (0,0), (64,0), (0,64), (64,64)
        let mut screen = vec![0x00; 128 * 128 * 4];

        let _ = encoder.encode_frame(&screen, false).unwrap();

        // Altera apenas o primeiro pixel do tile 0 (topo-esquerdo)
        screen[0] = 0xFF;

        let msg = encoder
            .encode_frame(&screen, false)
            .expect("encode frame alterado");
        assert_eq!(msg.tiles.len(), 1);
        assert_eq!(msg.tiles[0].tile_index, 0);
        assert!(!msg.tiles[0].compressed_data.is_empty());
    }

    #[test]
    fn test_force_keyframe() {
        let mut encoder = VideoEncoder::new(128, 128);
        let screen = vec![0x33; 128 * 128 * 4];

        let _ = encoder.encode_frame(&screen, false).unwrap();

        // Força keyframe mesmo com tela parada
        let keyframe_msg = encoder.encode_frame(&screen, true).expect("force keyframe");
        assert!(keyframe_msg.is_keyframe);
        assert_eq!(keyframe_msg.tiles.len(), 4);
    }
}
