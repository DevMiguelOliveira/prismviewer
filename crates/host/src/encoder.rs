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

/// Região retangular na tela para otimização com dirty rects DXGI (P-02).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub left: u32,
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
}

impl Rect {
    pub fn new(left: u32, top: u32, right: u32, bottom: u32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    /// Avalia se dois retângulos se intersectam.
    pub fn intersects(&self, other: &Rect) -> bool {
        self.left < other.right
            && self.right > other.left
            && self.top < other.bottom
            && self.bottom > other.top
    }
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
        self.encode_frame_dirty(bgra_data, None, force_keyframe)
    }

    /// Processa um quadro com otimização antecipada por dirty rects (P-02).
    /// Tiles fora dos retângulos alterados são ignorados sem alocação ou leitura de memória.
    pub fn encode_frame_dirty(
        &mut self,
        bgra_data: &[u8],
        dirty_rects: Option<&[Rect]>,
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

        // Se a tela não é keyframe e dirty_rects foi fornecido vazio, nenhum tile mudou
        if !is_keyframe {
            if let Some(rects) = dirty_rects {
                if rects.is_empty() {
                    return Ok(VideoFrameMessage {
                        frame_id: self.frame_counter,
                        is_keyframe: false,
                        width: self.width as u16,
                        height: self.height as u16,
                        tiles: Vec::new(),
                    });
                }
            }
        }

        let mut tiles = Vec::new();

        for ty in 0..self.rows {
            for tx in 0..self.cols {
                let tile_idx = ty * self.cols + tx;
                let x0 = tx * TILE_SIZE;
                let y0 = ty * TILE_SIZE;
                let tw = TILE_SIZE.min(width.saturating_sub(x0));
                let th = TILE_SIZE.min(height.saturating_sub(y0));

                let tile_rect = Rect::new(x0 as u32, y0 as u32, (x0 + tw) as u32, (y0 + th) as u32);

                // Otimização P-02: Se não for keyframe e dirty_rects foi passado, pula tiles não afetados
                if !is_keyframe {
                    if let Some(rects) = dirty_rects {
                        if !rects.iter().any(|r| r.intersects(&tile_rect)) {
                            continue;
                        }
                    }
                }

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

    #[test]
    fn test_rect_intersects_logic() {
        let r1 = Rect::new(0, 0, 64, 64);
        let r2 = Rect::new(32, 32, 96, 96);
        let r3 = Rect::new(64, 64, 128, 128); // adjacente, não sobreposto
        let r4 = Rect::new(100, 100, 200, 200);

        assert!(r1.intersects(&r2));
        assert!(r2.intersects(&r1));
        assert!(!r1.intersects(&r3));
        assert!(!r1.intersects(&r4));
    }

    #[test]
    fn test_dirty_rects_filter_skips_unaffected_tiles() {
        let mut encoder = VideoEncoder::new(128, 128); // 4 tiles: (0,0), (64,0), (0,64), (64,64)
        let mut screen = vec![0x11; 128 * 128 * 4];

        // Frame inicial (keyframe)
        let _ = encoder.encode_frame(&screen, false).unwrap();

        // Altera apenas o tile 3 (inferior direito: x=64..128, y=64..128)
        let tile3_start = (64 * 128 + 64) * 4;
        screen[tile3_start] = 0x99;

        // Passa dirty rect cobrindo apenas a área do tile 3
        let dirty = [Rect::new(64, 64, 128, 128)];
        let msg = encoder
            .encode_frame_dirty(&screen, Some(&dirty), false)
            .expect("encode dirty");

        assert_eq!(
            msg.tiles.len(),
            1,
            "apenas o tile no dirty rect deve ser codificado"
        );
        assert_eq!(msg.tiles[0].tile_index, 3);
    }

    #[test]
    fn test_empty_dirty_rects_returns_zero_tiles() {
        let mut encoder = VideoEncoder::new(128, 128);
        let screen = vec![0x22; 128 * 128 * 4];

        let _ = encoder.encode_frame(&screen, false).unwrap();

        // Informa lista de dirty rects vazia (DXGI detectou zero mudanças)
        let empty_dirty: [Rect; 0] = [];
        let msg = encoder
            .encode_frame_dirty(&screen, Some(&empty_dirty), false)
            .expect("encode empty dirty");

        assert!(!msg.is_keyframe);
        assert_eq!(msg.tiles.len(), 0);
    }
}
