//! prysmview-viewer - Gerenciador de sessão remota, telemetria HUD e captura de entrada

use eframe::egui::{self, Key, Pos2, Rect};
use prysmview_core::messages::{InputMessage, MouseButtonType};
use std::time::Instant;

/// Métricas de telemetria da sessão remota exibidas no HUD.
#[derive(Debug, Clone)]
pub struct SessionMetrics {
    pub rtt_ms: u32,
    pub current_fps: f32,
    pub frames_received: u64,
    pub tiles_rendered: u64,
    pub last_frame_time: Instant,
    pub fps_calc_time: Instant,
    pub fps_frame_count: u32,
}

impl Default for SessionMetrics {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            rtt_ms: 18,
            current_fps: 30.0,
            frames_received: 0,
            tiles_rendered: 0,
            last_frame_time: now,
            fps_calc_time: now,
            fps_frame_count: 0,
        }
    }
}

impl SessionMetrics {
    /// Registra um novo frame recebido e recalcula o FPS dinâmico.
    pub fn record_frame(&mut self, tiles_count: usize) {
        self.frames_received = self.frames_received.saturating_add(1);
        self.tiles_rendered = self.tiles_rendered.saturating_add(tiles_count as u64);
        self.fps_frame_count = self.fps_frame_count.saturating_add(1);

        let now = Instant::now();
        let elapsed = now.duration_since(self.fps_calc_time).as_secs_f32();
        if elapsed >= 1.0 {
            self.current_fps = self.fps_frame_count as f32 / elapsed;
            self.fps_frame_count = 0;
            self.fps_calc_time = now;
        }
        self.last_frame_time = now;
    }

    /// Atualiza o RTT medido (round-trip time) em milissegundos.
    pub fn update_rtt(&mut self, rtt_ms: u32) {
        self.rtt_ms = rtt_ms;
    }
}

/// Converte coordenadas de tela da interface para coordenadas normalizadas (0..65535).
pub fn normalize_coordinates(pos: Pos2, canvas_rect: Rect) -> (u16, u16) {
    if canvas_rect.width() <= 0.0 || canvas_rect.height() <= 0.0 {
        return (0, 0);
    }

    let rel_x = (pos.x - canvas_rect.min.x) / canvas_rect.width();
    let rel_y = (pos.y - canvas_rect.min.y) / canvas_rect.height();

    let norm_x = (rel_x * 65535.0).clamp(0.0, 65535.0) as u16;
    let norm_y = (rel_y * 65535.0).clamp(0.0, 65535.0) as u16;

    (norm_x, norm_y)
}

/// Mapeia teclas do egui para scan codes físicos de teclado compatíveis com o layout ABNT2 do PDV.
pub fn egui_key_to_abnt2_scancode(key: Key) -> Option<(u16, bool)> {
    // Retorna (scan_code, is_extended)
    match key {
        Key::Escape => Some((0x01, false)),
        Key::Num1 => Some((0x02, false)),
        Key::Num2 => Some((0x03, false)),
        Key::Num3 => Some((0x04, false)),
        Key::Num4 => Some((0x05, false)),
        Key::Num5 => Some((0x06, false)),
        Key::Num6 => Some((0x07, false)),
        Key::Num7 => Some((0x08, false)),
        Key::Num8 => Some((0x09, false)),
        Key::Num9 => Some((0x0A, false)),
        Key::Num0 => Some((0x0B, false)),
        Key::Backspace => Some((0x0E, false)),
        Key::Tab => Some((0x0F, false)),
        Key::Q => Some((0x10, false)),
        Key::W => Some((0x11, false)),
        Key::E => Some((0x12, false)),
        Key::R => Some((0x13, false)),
        Key::T => Some((0x14, false)),
        Key::Y => Some((0x15, false)),
        Key::U => Some((0x16, false)),
        Key::I => Some((0x17, false)),
        Key::O => Some((0x18, false)),
        Key::P => Some((0x19, false)),
        Key::Enter => Some((0x1C, false)),
        Key::A => Some((0x1E, false)),
        Key::S => Some((0x1F, false)),
        Key::D => Some((0x20, false)),
        Key::F => Some((0x21, false)),
        Key::G => Some((0x22, false)),
        Key::H => Some((0x23, false)),
        Key::J => Some((0x24, false)),
        Key::K => Some((0x25, false)),
        Key::L => Some((0x26, false)),
        Key::Z => Some((0x2C, false)),
        Key::X => Some((0x2D, false)),
        Key::C => Some((0x2E, false)),
        Key::V => Some((0x2F, false)),
        Key::B => Some((0x30, false)),
        Key::N => Some((0x31, false)),
        Key::M => Some((0x32, false)),
        Key::Space => Some((0x39, false)),
        // Teclas estendidas (Setas e Navegação)
        Key::ArrowUp => Some((0x48, true)),
        Key::ArrowDown => Some((0x50, true)),
        Key::ArrowLeft => Some((0x4B, true)),
        Key::ArrowRight => Some((0x4D, true)),
        Key::Home => Some((0x47, true)),
        Key::End => Some((0x4F, true)),
        Key::PageUp => Some((0x49, true)),
        Key::PageDown => Some((0x51, true)),
        Key::Delete => Some((0x53, true)),
        Key::Insert => Some((0x52, true)),
        _ => None,
    }
}

/// Extrai e converte eventos de entrada do egui para mensagens InputMessage do protocolo PRSM.
pub fn collect_input_events(
    input_state: &egui::InputState,
    canvas_rect: Rect,
) -> Vec<InputMessage> {
    let mut messages = Vec::new();

    // 1. Movimentação do ponteiro do mouse
    if let Some(pos) = input_state.pointer.latest_pos() {
        if canvas_rect.contains(pos) {
            let (x, y) = normalize_coordinates(pos, canvas_rect);
            messages.push(InputMessage::MouseMove { x, y });
        }
    }

    // 2. Cliques de botões do mouse
    if input_state
        .pointer
        .button_pressed(egui::PointerButton::Primary)
    {
        messages.push(InputMessage::MouseButton {
            button: MouseButtonType::Left,
            pressed: true,
        });
    }
    if input_state
        .pointer
        .button_released(egui::PointerButton::Primary)
    {
        messages.push(InputMessage::MouseButton {
            button: MouseButtonType::Left,
            pressed: false,
        });
    }

    if input_state
        .pointer
        .button_pressed(egui::PointerButton::Secondary)
    {
        messages.push(InputMessage::MouseButton {
            button: MouseButtonType::Right,
            pressed: true,
        });
    }
    if input_state
        .pointer
        .button_released(egui::PointerButton::Secondary)
    {
        messages.push(InputMessage::MouseButton {
            button: MouseButtonType::Right,
            pressed: false,
        });
    }

    if input_state
        .pointer
        .button_pressed(egui::PointerButton::Middle)
    {
        messages.push(InputMessage::MouseButton {
            button: MouseButtonType::Middle,
            pressed: true,
        });
    }
    if input_state
        .pointer
        .button_released(egui::PointerButton::Middle)
    {
        messages.push(InputMessage::MouseButton {
            button: MouseButtonType::Middle,
            pressed: false,
        });
    }

    // 3. Roda do mouse (Scroll)
    if input_state.raw_scroll_delta.y != 0.0 || input_state.raw_scroll_delta.x != 0.0 {
        messages.push(InputMessage::MouseWheel {
            delta_v: input_state.raw_scroll_delta.y as i16,
            delta_h: input_state.raw_scroll_delta.x as i16,
        });
    }

    // 4. Teclas de teclado
    for event in &input_state.events {
        if let egui::Event::Key {
            key,
            pressed,
            repeat: false,
            ..
        } = event
        {
            if let Some((scan_code, extended)) = egui_key_to_abnt2_scancode(*key) {
                messages.push(InputMessage::Key {
                    scan_code,
                    extended,
                    pressed: *pressed,
                });
            }
        }
    }

    messages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_coordinates() {
        let rect = Rect::from_min_max(Pos2::new(100.0, 100.0), Pos2::new(500.0, 300.0));

        // Canto superior esquerdo
        let (x, y) = normalize_coordinates(Pos2::new(100.0, 100.0), rect);
        assert_eq!((x, y), (0, 0));

        // Centro da tela
        let (x, y) = normalize_coordinates(Pos2::new(300.0, 200.0), rect);
        assert_eq!((x, y), (32767, 32767));

        // Canto inferior direito
        let (x, y) = normalize_coordinates(Pos2::new(500.0, 300.0), rect);
        assert_eq!((x, y), (65535, 65535));

        // Fora dos limites (deve sofrer clamp)
        let (x, y) = normalize_coordinates(Pos2::new(50.0, 400.0), rect);
        assert_eq!((x, y), (0, 65535));
    }

    #[test]
    fn test_abnt2_scancode_mapping() {
        assert_eq!(egui_key_to_abnt2_scancode(Key::A), Some((0x1E, false)));
        assert_eq!(egui_key_to_abnt2_scancode(Key::Enter), Some((0x1C, false)));
        assert_eq!(egui_key_to_abnt2_scancode(Key::ArrowUp), Some((0x48, true)));
        assert_eq!(egui_key_to_abnt2_scancode(Key::Delete), Some((0x53, true)));
    }

    #[test]
    fn test_session_metrics() {
        let mut metrics = SessionMetrics::default();
        assert_eq!(metrics.rtt_ms, 18);

        metrics.update_rtt(25);
        assert_eq!(metrics.rtt_ms, 25);

        metrics.record_frame(10);
        assert_eq!(metrics.frames_received, 1);
        assert_eq!(metrics.tiles_rendered, 10);
    }
}
