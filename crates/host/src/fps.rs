//! prysmview-host - Controle de taxa de quadros (15-30 fps), keyframe periódico e descarte de quadros intermediários.
//!
//! Conforme a especificação do AGENTS.md (seção 8 e 10):
//! - Meta de 15 a 30 fps, reduzida dinamicamente quando a rede piora (RTT alto).
//! - Keyframe completo a cada 5 segundos ou sob solicitação do viewer.
//! - No máximo um frame em trânsito por vez; se o canal estiver lento, frames intermediários
//!   são descartados (nunca acumulados).

use std::time::{Duration, Instant};

pub const MIN_FPS: u32 = 15;
pub const MAX_FPS: u32 = 30;
pub const DEFAULT_KEYFRAME_INTERVAL_SECS: u64 = 30;

/// Controlador adaptativo de taxa de vídeo e keyframes.
pub struct RateController {
    current_fps: u32,
    last_frame_sent: Option<Instant>,
    last_keyframe_sent: Option<Instant>,
    keyframe_interval: Duration,
    in_flight: bool,
    force_next_keyframe: bool,
}

impl Default for RateController {
    fn default() -> Self {
        Self::new(30, Duration::from_secs(DEFAULT_KEYFRAME_INTERVAL_SECS))
    }
}

impl RateController {
    pub fn new(initial_fps: u32, keyframe_interval: Duration) -> Self {
        let current_fps = initial_fps.clamp(MIN_FPS, MAX_FPS);
        Self {
            current_fps,
            last_frame_sent: None,
            last_keyframe_sent: None,
            keyframe_interval,
            in_flight: false,
            force_next_keyframe: true, // Primeiro frame é sempre keyframe
        }
    }

    pub fn current_fps(&self) -> u32 {
        self.current_fps
    }

    pub fn is_in_flight(&self) -> bool {
        self.in_flight
    }

    /// Solicita que o próximo frame seja um keyframe completo (ex: viewer pediu).
    pub fn request_keyframe(&mut self) {
        self.force_next_keyframe = true;
    }

    /// Verifica se é hora de enviar um frame respeitando o FPS configurado
    /// e a regra de nunca acumular se houver frame em trânsito.
    pub fn can_send_next_frame(&self) -> bool {
        // Se o frame anterior ainda está em trânsito na rede QUIC, descarta o intermediário
        if self.in_flight {
            return false;
        }

        match self.last_frame_sent {
            Some(last) => {
                let frame_interval = Duration::from_micros(1_000_000 / (self.current_fps as u64));
                last.elapsed() >= frame_interval
            }
            None => true,
        }
    }

    /// Avalia se o próximo frame deve ser codificado como keyframe (completo).
    /// Dispara se foi forçado ou se decorreu o intervalo de keyframes.
    pub fn should_encode_keyframe(&mut self) -> bool {
        self.should_encode_keyframe_adaptive(true)
    }

    /// Avaliação adaptativa de keyframe (P-03):
    /// Se a tela está sem alteração de tiles (`has_activity == false`), evita disparar
    /// keyframe redundante a menos que tenha sido solicitado explicitamente pelo viewer.
    pub fn should_encode_keyframe_adaptive(&mut self, has_activity: bool) -> bool {
        if self.force_next_keyframe {
            return true;
        }

        if !has_activity {
            return false;
        }

        match self.last_keyframe_sent {
            Some(last) => last.elapsed() >= self.keyframe_interval,
            None => true,
        }
    }

    /// Notifica que um frame foi despachado para a rede QUIC.
    pub fn on_frame_sent(&mut self, is_keyframe: bool) {
        let now = Instant::now();
        self.last_frame_sent = Some(now);
        self.in_flight = true;

        if is_keyframe {
            self.last_keyframe_sent = Some(now);
            self.force_next_keyframe = false;
        }
    }

    /// Notifica que o frame em trânsito foi confirmado pelo viewer (ACK).
    /// Adapta a meta de FPS com base no RTT medido.
    pub fn on_frame_ack(&mut self, rtt: Duration) {
        self.in_flight = false;

        // Se o RTT estiver acima de 120ms, reduz o FPS para poupar banda
        // Se o RTT estiver abaixo de 40ms, incrementa o FPS gradualmente até 30
        if rtt > Duration::from_millis(120) {
            self.current_fps = (self.current_fps.saturating_sub(2)).max(MIN_FPS);
        } else if rtt < Duration::from_millis(40) {
            self.current_fps = (self.current_fps + 1).min(MAX_FPS);
        }
    }

    /// Em caso de queda ou timeout do canal, limpa o estado de em trânsito.
    pub fn reset_in_flight(&mut self) {
        self.in_flight = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let mut controller = RateController::default();
        assert_eq!(controller.current_fps(), 30);
        assert!(!controller.is_in_flight());
        assert!(controller.can_send_next_frame());
        assert!(controller.should_encode_keyframe());
    }

    #[test]
    fn test_in_flight_drops_intermediate_frames() {
        let mut controller = RateController::default();

        // Envia primeiro frame
        assert!(controller.can_send_next_frame());
        controller.on_frame_sent(true);

        assert!(controller.is_in_flight());
        // Enquanto em voo, não pode enviar (descarta intermediários)
        assert!(!controller.can_send_next_frame());

        // Recebe ACK
        controller.on_frame_ack(Duration::from_millis(20));
        assert!(!controller.is_in_flight());
    }

    #[test]
    fn test_rtt_adaptation() {
        let mut controller = RateController::new(25, Duration::from_secs(5));

        // RTT alto: deve reduzir FPS até MIN_FPS (15)
        for _ in 0..10 {
            controller.on_frame_sent(false);
            controller.on_frame_ack(Duration::from_millis(150));
        }
        assert_eq!(controller.current_fps(), MIN_FPS);

        // RTT excelente: deve subir FPS até MAX_FPS (30)
        for _ in 0..20 {
            controller.on_frame_sent(false);
            controller.on_frame_ack(Duration::from_millis(20));
        }
        assert_eq!(controller.current_fps(), MAX_FPS);
    }

    #[test]
    fn test_request_keyframe_on_demand() {
        let mut controller = RateController::default();
        controller.on_frame_sent(true);
        controller.on_frame_ack(Duration::from_millis(10));

        assert!(!controller.should_encode_keyframe());

        // Viewer solicita keyframe
        controller.request_keyframe();
        assert!(controller.should_encode_keyframe());
    }

    #[test]
    fn test_adaptive_keyframe_suppressed_on_static_screen() {
        let mut controller = RateController::default();
        controller.on_frame_sent(true);
        controller.on_frame_ack(Duration::from_millis(10));

        // Sem atividade, não deve disparar keyframe
        assert!(!controller.should_encode_keyframe_adaptive(false));

        // Com atividade e se forçado, deve disparar
        controller.request_keyframe();
        assert!(controller.should_encode_keyframe_adaptive(false));
    }
}
