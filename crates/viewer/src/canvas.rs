//! prysmview-viewer - Canvas de renderização incremental de vídeo por tiles 64x64

use eframe::egui::{self, Color32, ColorImage, TextureHandle, TextureOptions};
use prysmview_core::messages::VideoFrameMessage;
use std::cmp::min;
use std::io;
use thiserror::Error;

pub const TILE_SIZE: usize = 64;

#[derive(Debug, Error)]
pub enum CanvasError {
    #[error("Falha ao descompactar tile com zstd: {0}")]
    Decompression(#[from] io::Error),
    #[error("Tamanho do tile inconsistente: esperado {expected} bytes, recebido {actual}")]
    TileSizeMismatch { expected: usize, actual: usize },
    #[error("Dimensões de tela inválidas: {width}x{height}")]
    InvalidDimensions { width: u16, height: u16 },
}

/// Canvas remoto mantendo o buffer de pixels RGBA reconstruído a partir de dirty tiles.
pub struct RemoteCanvas {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
    pub texture: Option<TextureHandle>,
    pub is_dirty: bool,
}

impl Default for RemoteCanvas {
    fn default() -> Self {
        Self::new(1920, 1080)
    }
}

impl RemoteCanvas {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height * 4;
        Self {
            width,
            height,
            pixels: vec![0u8; size],
            texture: None,
            is_dirty: true,
        }
    }

    /// Redimensiona o buffer de exibição se a resolução do PDV mudou.
    pub fn ensure_dimensions(&mut self, new_width: usize, new_height: usize) {
        if self.width != new_width || self.height != new_height {
            self.width = new_width;
            self.height = new_height;
            self.pixels = vec![0u8; new_width * new_height * 4];
            self.is_dirty = true;
        }
    }

    /// Aplica um quadro de vídeo incremental (dirty tiles descomprimidos via zstd).
    pub fn apply_frame(&mut self, frame: &VideoFrameMessage) -> Result<(), CanvasError> {
        let frame_w = frame.width as usize;
        let frame_h = frame.height as usize;

        if frame_w == 0 || frame_h == 0 {
            return Err(CanvasError::InvalidDimensions {
                width: frame.width,
                height: frame.height,
            });
        }

        self.ensure_dimensions(frame_w, frame_h);

        let tiles_per_row = self.width.div_ceil(TILE_SIZE);

        for tile in &frame.tiles {
            // Descompressão rápida zstd do tile
            let decompressed = zstd::decode_all(tile.compressed_data.as_slice())?;

            let tile_idx = tile.tile_index as usize;
            let tile_col = tile_idx % tiles_per_row;
            let tile_row = tile_idx / tiles_per_row;

            let tile_x = tile_col * TILE_SIZE;
            let tile_y = tile_row * TILE_SIZE;

            let tile_w = min(TILE_SIZE, self.width.saturating_sub(tile_x));
            let tile_h = min(TILE_SIZE, self.height.saturating_sub(tile_y));

            let expected_bytes = tile_w * tile_h * 4;
            if decompressed.len() != expected_bytes {
                return Err(CanvasError::TileSizeMismatch {
                    expected: expected_bytes,
                    actual: decompressed.len(),
                });
            }

            // Blitting linha por linha do tile no buffer da tela inteira
            for row in 0..tile_h {
                let src_offset = row * tile_w * 4;
                let dst_offset = ((tile_y + row) * self.width + tile_x) * 4;

                let src_slice = &decompressed[src_offset..src_offset + tile_w * 4];
                let dst_slice = &mut self.pixels[dst_offset..dst_offset + tile_w * 4];
                dst_slice.copy_from_slice(src_slice);
            }
        }

        self.is_dirty = true;
        Ok(())
    }

    /// Atualiza ou cria a textura GPU no contexto egui se houver alterações de tiles.
    pub fn sync_texture(&mut self, ctx: &egui::Context) {
        if self.is_dirty || self.texture.is_none() {
            let color_image =
                ColorImage::from_rgba_unmultiplied([self.width, self.height], &self.pixels);

            if let Some(ref mut texture) = self.texture {
                texture.set(color_image, TextureOptions::LINEAR);
            } else {
                self.texture = Some(ctx.load_texture(
                    "prysmview_remote_screen",
                    color_image,
                    TextureOptions::LINEAR,
                ));
            }
            self.is_dirty = false;
        }
    }

    /// Obtém a cor de um pixel específico (útil para testes de integridade).
    pub fn get_pixel(&self, x: usize, y: usize) -> Option<Color32> {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) * 4;
            Some(Color32::from_rgba_premultiplied(
                self.pixels[idx],
                self.pixels[idx + 1],
                self.pixels[idx + 2],
                self.pixels[idx + 3],
            ))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prysmview_core::messages::TilePacket;

    #[test]
    fn test_canvas_initialization_and_resize() {
        let mut canvas = RemoteCanvas::new(800, 600);
        assert_eq!(canvas.pixels.len(), 800 * 600 * 4);

        canvas.ensure_dimensions(1920, 1080);
        assert_eq!(canvas.width, 1920);
        assert_eq!(canvas.height, 1080);
        assert_eq!(canvas.pixels.len(), 1920 * 1080 * 4);
    }

    #[test]
    fn test_canvas_apply_frame_with_zstd_compressed_tile() {
        let mut canvas = RemoteCanvas::new(128, 128);

        // Prepara um tile 64x64 preenchido com vermelho RGBA (255, 0, 0, 255)
        let tile_raw = [255, 0, 0, 255].repeat(64 * 64);
        let compressed = zstd::encode_all(tile_raw.as_slice(), 1).expect("compress zstd");

        let frame = VideoFrameMessage {
            frame_id: 1,
            is_keyframe: true,
            width: 128,
            height: 128,
            tiles: vec![TilePacket {
                tile_index: 0, // Tile superior esquerdo (0,0 até 63,63)
                compressed_data: compressed,
            }],
        };

        canvas.apply_frame(&frame).expect("apply frame ok");

        // Verifica que o pixel dentro do tile 0 foi atualizado para vermelho
        let pixel_in = canvas.get_pixel(32, 32).expect("pixel");
        assert_eq!(pixel_in.r(), 255);
        assert_eq!(pixel_in.g(), 0);
        assert_eq!(pixel_in.b(), 0);

        // Verifica que o pixel fora do tile (no tile 1) permanece preto (0, 0, 0, 0)
        let pixel_out = canvas.get_pixel(65, 32).expect("pixel out");
        assert_eq!(pixel_out.r(), 0);
    }

    #[test]
    fn test_canvas_rejects_invalid_dimensions() {
        let mut canvas = RemoteCanvas::new(100, 100);
        let frame = VideoFrameMessage {
            frame_id: 2,
            is_keyframe: false,
            width: 0,
            height: 100,
            tiles: vec![],
        };

        let res = canvas.apply_frame(&frame);
        assert!(res.is_err());
    }
}
