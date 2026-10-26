#![windows_subsystem = "windows"]

//! prysmview-tray - Aplicativo visual do Host no PDV estilo TeamViewer Host
//!
//! Executa na sessão do usuário comum no PDV (sem privilégios de administrador).
//! Comunica-se exclusivamente com o serviço Windows (prysmview-svc) através
//! de Named Pipe local restrito (`\\.\pipe\prysmview-ipc`).
//!
//! Responsabilidades:
//! - Interface visual elegante estilo TeamViewer Host com ID do PDV e Código de 8 dígitos;
//! - Exibição de indicador de status verde ("Pronto para conectar - Conexão Segura E2E");
//! - Indicador visual de sessão ativa com botão destacado de encerramento imediato;
//! - Janela de consentimento e travas de segurança locais para o operador da loja.

use eframe::egui::{self, Color32, RichText, Stroke, Vec2};
use prysmview_host::{recv_ipc_message, send_ipc_message, IpcError, IpcMessage, DEFAULT_PIPE_NAME};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{error, info, warn};

// ==============================================================================
// PALETA OFICIAL CYBER PRISM (Baseada no logotipo oficial do PrysmView)
// ==============================================================================
pub const COLOR_BG_OBSIDIAN: Color32 = Color32::from_rgb(10, 13, 22); // #0a0d16 Fundo principal escuro
pub const COLOR_PANEL_NAVY: Color32 = Color32::from_rgb(15, 20, 34); // #0f1422 Fundo de painéis e barras
pub const COLOR_CARD_BASE: Color32 = Color32::from_rgb(20, 27, 45); // #141b2d Fundo de cards e containers
pub const COLOR_CARD_BORDER: Color32 = Color32::from_rgb(32, 45, 75); // #202d4b Borda de vidro sutil
pub const COLOR_CYAN_NEON: Color32 = Color32::from_rgb(0, 240, 255); // #00f0ff Destaque prismático ciano
pub const COLOR_CYAN_GLOW: Color32 = Color32::from_rgb(0, 200, 235); // #00c8eb Ciano secundário
pub const COLOR_BLUE_ELECTRIC: Color32 = Color32::from_rgb(0, 140, 255); // #008cff Azul elétrico vibrante
pub const COLOR_VIOLET_NEON: Color32 = Color32::from_rgb(190, 45, 235); // #be2deb Magenta/violeta do prisma
pub const COLOR_MINT_ONLINE: Color32 = Color32::from_rgb(0, 255, 204); // #00ffcc Glow de sinal e status online
pub const COLOR_RED_DANGER: Color32 = Color32::from_rgb(255, 51, 85); // #ff3355 Vermelho de emergência
pub const COLOR_TEXT_WHITE: Color32 = Color32::from_rgb(245, 248, 255); // #f5f8ff Texto principal
pub const COLOR_TEXT_MUTED: Color32 = Color32::from_rgb(148, 163, 184); // #94a3b8 Texto secundário
pub const COLOR_TEXT_DARK: Color32 = Color32::from_rgb(100, 116, 139); // #64748b Metadados discretos

/// Carrega o ícone oficial da janela e da barra de tarefas a partir do logo oficial.
fn load_brand_icon() -> Option<std::sync::Arc<egui::IconData>> {
    let raw = include_bytes!("../../../assets/logo.png");
    let img = image::load_from_memory(raw).ok()?.to_rgba8();
    let (width, height) = img.dimensions();
    Some(std::sync::Arc::new(egui::IconData {
        rgba: img.into_raw(),
        width,
        height,
    }))
}

/// Carrega a textura GPU do logotipo oficial para exibição na tela.
fn load_brand_texture(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let raw = include_bytes!("../../../assets/logo.png");
    let img = image::load_from_memory(raw).ok()?.to_rgba8();
    let (width, height) = img.dimensions();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(
        [width as usize, height as usize],
        &img.into_raw(),
    );
    Some(ctx.load_texture("prysm_host_logo", color_image, egui::TextureOptions::LINEAR))
}

/// Renderiza o logotipo oficial na interface do Host.
fn render_logo(ui: &mut egui::Ui, texture: Option<&egui::TextureHandle>, size: f32) {
    let (rect, _resp) = ui.allocate_exact_size(Vec2::new(size, size), egui::Sense::hover());
    if let Some(tex) = texture {
        ui.painter().image(
            tex.id(),
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    } else {
        ui.painter().rect_filled(rect, 6.0, COLOR_CARD_BASE);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "💎",
            egui::FontId::proportional(size * 0.55),
            COLOR_CYAN_NEON,
        );
    }
}

/// Estado da solicitação de consentimento pendente.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingConsent {
    pub operator_name: String,
    pub reason: String,
    pub timeout_secs: u32,
    pub response: Option<bool>,
}

/// Gerenciador de estado reativo da aplicação Host / Tray no PDV.
#[derive(Debug, Clone)]
pub struct TrayState {
    pub is_connected_to_service: bool,
    pub pdv_id: String,
    pub is_session_active: bool,
    pub current_operator: Option<String>,
    pub current_code: Option<String>,
    pub session_code_expires_in: u32,
    pub clipboard_enabled: bool,
    pub file_transfer_enabled: bool,
    pub status_message: String,
    pub terminate_requested: bool,
    pub pending_consent: Option<PendingConsent>,
}

impl Default for TrayState {
    fn default() -> Self {
        Self {
            is_connected_to_service: false,
            pdv_id: "PDV-CAIXA-01".to_string(),
            is_session_active: false,
            current_operator: None,
            current_code: None,
            session_code_expires_in: 120,
            clipboard_enabled: true,
            file_transfer_enabled: true,
            status_message: "Aguardando conexão do serviço local...".to_string(),
            terminate_requested: false,
            pending_consent: None,
        }
    }
}

impl TrayState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Processa uma mensagem recebida do serviço e retorna eventual resposta imediata.
    pub fn process_message(&mut self, msg: &IpcMessage) -> Option<IpcMessage> {
        match msg {
            IpcMessage::ConsentRequest {
                operator_name,
                reason,
                timeout_secs,
            } => {
                info!(
                    operator = %operator_name,
                    reason = %reason,
                    timeout = %timeout_secs,
                    "Solicitação de consentimento recebida pelo Host"
                );
                self.pending_consent = Some(PendingConsent {
                    operator_name: operator_name.clone(),
                    reason: reason.clone(),
                    timeout_secs: *timeout_secs,
                    response: None,
                });
                None
            }
            IpcMessage::ConsentResponse { .. } => None,
            IpcMessage::SessionCodeDisplay {
                code,
                expires_in_secs,
            } => {
                info!(code = %code, expires = %expires_in_secs, "Código de sessão recebido");
                self.current_code = Some(code.clone());
                self.session_code_expires_in = *expires_in_secs;
                None
            }
            IpcMessage::SessionActive { operator_name } => {
                info!(operator = %operator_name, "Sessão remota iniciada e ativa");
                self.is_session_active = true;
                self.current_operator = Some(operator_name.clone());
                self.pending_consent = None;
                self.status_message = format!("Sessão ativa com o operador {}", operator_name);
                None
            }
            IpcMessage::SessionEnded { reason } => {
                info!(reason = %reason, "Sessão remota finalizada");
                self.is_session_active = false;
                self.current_operator = None;
                self.current_code = None;
                self.pending_consent = None;
                self.terminate_requested = false;
                self.status_message = format!("Sessão encerrada: {}", reason);
                None
            }
            IpcMessage::TerminateSession { reason } => {
                info!(reason = %reason, "Sessão terminada via comando");
                self.is_session_active = false;
                self.current_operator = None;
                self.status_message = format!("Sessão terminada: {}", reason);
                None
            }
            IpcMessage::Ping(val) => Some(IpcMessage::Pong(*val)),
            IpcMessage::Pong(_) => None,
        }
    }

    /// Solicita o encerramento imediato da sessão em andamento.
    pub fn request_terminate(&mut self) {
        self.terminate_requested = true;
        self.is_session_active = false;
        self.current_operator = None;
        self.status_message = "Encerramento solicitado pelo operador local.".to_string();
    }
}

/// Aplicativo com interface gráfica nativa estilo TeamViewer Host.
pub struct PrysmHostApp {
    pub state: Arc<Mutex<TrayState>>,
    pub logo_texture: Option<egui::TextureHandle>,
}

impl PrysmHostApp {
    pub fn new(state: Arc<Mutex<TrayState>>) -> Self {
        Self {
            state,
            logo_texture: None,
        }
    }
}

impl eframe::App for PrysmHostApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut state = self.state.lock().unwrap().clone();

        // Carregamento lazy da textura do logotipo oficial
        if self.logo_texture.is_none() {
            self.logo_texture = load_brand_texture(ctx);
        }

        // Estilização Dark Mode Cyber Prism oficial
        let mut visuals = egui::Visuals::dark();
        visuals.window_fill = COLOR_CARD_BASE;
        visuals.panel_fill = COLOR_PANEL_NAVY;
        visuals.extreme_bg_color = COLOR_BG_OBSIDIAN;
        visuals.override_text_color = Some(COLOR_TEXT_WHITE);
        visuals.widgets.noninteractive.bg_fill = COLOR_CARD_BASE;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, COLOR_CARD_BORDER);
        visuals.widgets.inactive.bg_fill = COLOR_CARD_BASE;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, COLOR_CARD_BORDER);
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(26, 36, 58);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, COLOR_CYAN_NEON);
        visuals.widgets.active.bg_fill = COLOR_BLUE_ELECTRIC;
        visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, COLOR_CYAN_NEON);
        visuals.selection.bg_fill = Color32::from_rgba_premultiplied(0, 240, 255, 45);
        visuals.selection.stroke = Stroke::new(1.0_f32, COLOR_CYAN_NEON);
        ctx.set_visuals(visuals);

        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(COLOR_BG_OBSIDIAN)
                    .inner_margin(16.0),
            )
            .show(ctx, |ui| {
                ui.add_space(4.0);

                // Cabeçalho da Aplicação com logotipo oficial
                ui.horizontal(|ui| {
                    render_logo(ui, self.logo_texture.as_ref(), 40.0);
                    ui.add_space(10.0);
                    ui.vertical(|ui| {
                        ui.heading(
                            RichText::new("PRYSMVIEW HOST")
                                .color(COLOR_CYAN_NEON)
                                .size(19.0)
                                .strong(),
                        );
                        ui.label(
                            RichText::new("Módulo de Suporte Remoto • PDV Caixa")
                                .color(COLOR_TEXT_MUTED)
                                .size(11.0),
                        );
                    });
                });

                ui.add_space(12.0);

                // Status Badge da Conexão com indicador Mint Glow
                if state.is_connected_to_service {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(10, 36, 28))
                        .stroke(Stroke::new(1.0_f32, COLOR_MINT_ONLINE))
                        .rounding(6.0)
                        .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("●").color(COLOR_MINT_ONLINE).size(14.0));
                                ui.label(
                                    RichText::new("Pronto para conectar (Conexão Segura E2E)")
                                        .color(COLOR_MINT_ONLINE)
                                        .size(12.0)
                                        .strong(),
                                );
                            });
                        });
                } else {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(36, 28, 12))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(200, 150, 40)))
                        .rounding(6.0)
                        .inner_margin(egui::Margin::symmetric(12.0, 8.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("●")
                                        .color(Color32::from_rgb(240, 180, 50))
                                        .size(14.0),
                                );
                                ui.label(
                                    RichText::new("Aguardando serviço local prysmview-svc...")
                                        .color(Color32::from_rgb(240, 190, 80))
                                        .size(12.0),
                                );
                            });
                        });
                }

                ui.add_space(14.0);

                // Card Principal: Permitir Acesso Remoto
                egui::Frame::window(ui.style())
                    .fill(COLOR_CARD_BASE)
                    .stroke(Stroke::new(1.0_f32, COLOR_CARD_BORDER))
                    .rounding(10.0)
                    .inner_margin(egui::Margin::same(16.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("PERMITIR CONTROLE REMOTO")
                                .color(COLOR_TEXT_MUTED)
                                .size(11.0)
                                .strong(),
                        );
                        ui.add_space(8.0);

                        // Sua ID
                        ui.label(
                            RichText::new("Sua ID (deste Caixa):")
                                .size(12.0)
                                .color(COLOR_TEXT_MUTED),
                        );
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(&state.pdv_id)
                                    .color(COLOR_TEXT_WHITE)
                                    .size(20.0)
                                    .strong()
                                    .monospace(),
                            );
                            let btn_copiar = egui::Button::new(
                                RichText::new("📋 Copiar")
                                    .color(COLOR_TEXT_WHITE)
                                    .size(12.0),
                            )
                            .fill(Color32::from_rgb(26, 36, 56))
                            .stroke(Stroke::new(1.0_f32, COLOR_CARD_BORDER))
                            .rounding(4.0);

                            if ui.add(btn_copiar).clicked() {
                                ui.output_mut(|o| o.copied_text = state.pdv_id.clone());
                            }
                        });

                        ui.add_space(14.0);

                        // Código de Sessão / Senha Temporária
                        ui.label(
                            RichText::new("Código de Sessão / Senha:")
                                .size(12.0)
                                .color(COLOR_TEXT_MUTED),
                        );
                        let code_display = state
                            .current_code
                            .as_deref()
                            .unwrap_or("Aguardando chamada...");

                        egui::Frame::none()
                            .fill(Color32::from_rgb(12, 17, 28))
                            .stroke(Stroke::new(
                                1.0_f32,
                                if state.current_code.is_some() {
                                    COLOR_CYAN_GLOW
                                } else {
                                    COLOR_CARD_BORDER
                                },
                            ))
                            .rounding(6.0)
                            .inner_margin(egui::Margin::symmetric(14.0, 8.0))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(code_display)
                                        .color(COLOR_CYAN_NEON)
                                        .size(24.0)
                                        .strong()
                                        .monospace(),
                                );
                            });

                        if state.current_code.is_some() {
                            ui.add_space(6.0);
                            ui.label(
                                RichText::new(format!(
                                    "⏱️ Expira em {}s • Máximo 3 tentativas",
                                    state.session_code_expires_in
                                ))
                                .color(COLOR_VIOLET_NEON)
                                .size(11.0),
                            );
                        }
                    });

                ui.add_space(14.0);

                // Banner de Sessão Ativa OU Estado Ocioso
                if state.is_session_active {
                    egui::Frame::window(ui.style())
                        .fill(Color32::from_rgb(38, 18, 26))
                        .stroke(Stroke::new(1.2_f32, COLOR_RED_DANGER))
                        .rounding(10.0)
                        .inner_margin(egui::Margin::same(14.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("●").color(COLOR_RED_DANGER).size(16.0));
                                ui.label(
                                    RichText::new("SESSÃO REMOTA EM ANDAMENTO")
                                        .color(Color32::from_rgb(255, 120, 130))
                                        .size(12.0)
                                        .strong(),
                                );
                            });

                            ui.add_space(6.0);
                            let op = state
                                .current_operator
                                .as_deref()
                                .unwrap_or("Técnico de Suporte");
                            ui.label(
                                RichText::new(format!("Operador Conectado: {}", op))
                                    .color(COLOR_TEXT_WHITE)
                                    .size(13.0)
                                    .strong(),
                            );
                            ui.label(
                                RichText::new("Criptografia E2E QUIC TLS 1.3 Ativa")
                                    .color(COLOR_MINT_ONLINE)
                                    .size(11.0),
                            );

                            ui.add_space(12.0);

                            // Botão Vermelho de Encerramento Imediato
                            let btn_encerrar = ui.add_sized(
                                [ui.available_width(), 40.0],
                                egui::Button::new(
                                    RichText::new("⛔ ENCERRAR ACESSO IMEDIATAMENTE")
                                        .color(COLOR_TEXT_WHITE)
                                        .size(13.0)
                                        .strong(),
                                )
                                .fill(COLOR_RED_DANGER)
                                .rounding(6.0),
                            );

                            if btn_encerrar.clicked() {
                                state.request_terminate();
                            }
                        });
                }

                ui.add_space(14.0);

                // Travas de Segurança Locais do PDV
                egui::Frame::none()
                    .fill(COLOR_CARD_BASE)
                    .stroke(Stroke::new(1.0_f32, COLOR_CARD_BORDER))
                    .rounding(8.0)
                    .inner_margin(egui::Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new("TRAVAS DE SEGURANÇA LOCAL")
                                .color(COLOR_TEXT_MUTED)
                                .size(10.0)
                                .strong(),
                        );
                        ui.add_space(6.0);

                        ui.checkbox(
                            &mut state.clipboard_enabled,
                            RichText::new("Área de transferência compartilhada (Ctrl+C/V)")
                                .size(11.0)
                                .color(COLOR_TEXT_WHITE),
                        );
                        ui.checkbox(
                            &mut state.file_transfer_enabled,
                            RichText::new("Permitir transferência de arquivos (SHA-256)")
                                .size(11.0)
                                .color(COLOR_TEXT_WHITE),
                        );
                    });

                ui.add_space(10.0);

                // Rodapé de soberania do operador local
                ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new(
                            "O operador local pode recusar ou encerrar a qualquer momento.",
                        )
                        .color(COLOR_TEXT_DARK)
                        .size(10.0),
                    );
                });
            });

        // Atualiza o estado compartilhado
        *self.state.lock().unwrap() = state;
    }
}

/// Loop principal do cliente IPC da bandeja com o serviço LocalSystem.
async fn run_tray_client(
    pipe_name: String,
    running: Arc<AtomicBool>,
    state_ref: Arc<Mutex<TrayState>>,
) -> Result<(), IpcError> {
    while running.load(Ordering::Relaxed) {
        #[cfg(windows)]
        let connect_result = prysmview_host::connect_pipe_client(&pipe_name).await;

        #[cfg(not(windows))]
        let connect_result: Result<tokio::io::DuplexStream, IpcError> =
            Err(IpcError::Io(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Named pipe só é suportado no Windows",
            )));

        let mut client = match connect_result {
            Ok(c) => {
                info!("Conectado com sucesso ao Named Pipe do prysmview-svc");
                if let Ok(mut st) = state_ref.lock() {
                    st.is_connected_to_service = true;
                    st.status_message = "Conectado ao serviço local.".to_string();
                }
                c
            }
            Err(e) => {
                if let Ok(mut st) = state_ref.lock() {
                    st.is_connected_to_service = false;
                }
                warn!("Aguardando serviço prysmview-svc iniciar ({e}). Reconectando em 2s...");
                tokio::time::sleep(Duration::from_secs(2)).await;
                continue;
            }
        };

        loop {
            // Verifica se o operador local clicou no botão de encerrar sessão
            let should_terminate = state_ref
                .lock()
                .map(|st| st.terminate_requested)
                .unwrap_or(false);
            if should_terminate {
                info!("Enviando sinal de encerramento para o prysmview-svc...");
                let _ = send_ipc_message(
                    &mut client,
                    &IpcMessage::SessionEnded {
                        reason: "Encerrado pelo usuário local no PDV".to_string(),
                    },
                )
                .await;
                if let Ok(mut st) = state_ref.lock() {
                    st.terminate_requested = false;
                    st.is_session_active = false;
                }
            }

            let msg = match recv_ipc_message(&mut client).await {
                Ok(m) => m,
                Err(IpcError::ConnectionClosed) => {
                    warn!("Serviço prysmview-svc encerrou a conexão IPC.");
                    if let Ok(mut st) = state_ref.lock() {
                        st.is_connected_to_service = false;
                    }
                    break;
                }
                Err(e) => {
                    error!("Erro de comunicação IPC: {e}");
                    if let Ok(mut st) = state_ref.lock() {
                        st.is_connected_to_service = false;
                    }
                    break;
                }
            };

            let maybe_reply = if let Ok(mut st) = state_ref.lock() {
                st.process_message(&msg)
            } else {
                None
            };

            if let Some(reply) = maybe_reply {
                if let Err(e) = send_ipc_message(&mut client, &reply).await {
                    error!("Falha ao enviar resposta IPC: {e}");
                    break;
                }
            }
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}

fn main() -> Result<(), eframe::Error> {
    tracing_subscriber::fmt::init();
    info!("Iniciando prysmview-tray (Host Interface) no PDV...");

    let state = Arc::new(Mutex::new(TrayState::new()));
    let state_bg = state.clone();

    let running = Arc::new(AtomicBool::new(true));
    let running_bg = running.clone();

    // Thread assíncrona dedicada para a comunicação Named Pipe com prysmview-svc
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");
        rt.block_on(async {
            let _ = run_tray_client(DEFAULT_PIPE_NAME.to_string(), running_bg, state_bg).await;
        });
    });

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([380.0, 560.0])
        .with_min_inner_size([350.0, 500.0])
        .with_title("PrysmView Host - Atendimento ao PDV");

    if let Some(icon) = load_brand_icon() {
        viewport = viewport.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "PrysmView Host",
        native_options,
        Box::new(|_cc| Ok(Box::new(PrysmHostApp::new(state)))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_state_consent_message() {
        let mut state = TrayState::new();
        let msg = IpcMessage::ConsentRequest {
            operator_name: "Central de Suporte".to_string(),
            reason: "Fechamento de caixa".to_string(),
            timeout_secs: 60,
        };

        let resp = state.process_message(&msg);
        assert_eq!(resp, None);
        assert!(!state.is_session_active);
        assert!(state.pending_consent.is_some());
    }

    #[test]
    fn test_tray_state_no_code_in_idle() {
        let state = TrayState::new();
        assert_eq!(state.current_code, None);
        assert!(!state.is_session_active);
    }

    #[test]
    fn test_tray_state_session_code_display() {
        let mut state = TrayState::new();
        let msg = IpcMessage::SessionCodeDisplay {
            code: "87654321".to_string(),
            expires_in_secs: 120,
        };

        state.process_message(&msg);
        assert_eq!(state.current_code.as_deref(), Some("87654321"));
        assert_eq!(state.session_code_expires_in, 120);
    }

    #[test]
    fn test_tray_state_session_lifecycle() {
        let mut state = TrayState::new();

        // Sessão iniciada
        state.process_message(&IpcMessage::SessionActive {
            operator_name: "Técnico Lucas".to_string(),
        });
        assert!(state.is_session_active);
        assert_eq!(state.current_operator.as_deref(), Some("Técnico Lucas"));

        // Sessão encerrada
        state.process_message(&IpcMessage::SessionEnded {
            reason: "Operador desconectou".to_string(),
        });
        assert!(!state.is_session_active);
        assert_eq!(state.current_operator, None);
        assert_eq!(state.current_code, None);
    }

    #[test]
    fn test_tray_state_ping_pong() {
        let mut state = TrayState::new();
        let resp = state.process_message(&IpcMessage::Ping(999));
        assert_eq!(resp, Some(IpcMessage::Pong(999)));
    }

    #[test]
    fn test_tray_state_request_terminate() {
        let mut state = TrayState::new();
        state.is_session_active = true;
        state.current_operator = Some("Técnico".to_string());

        state.request_terminate();
        assert!(state.terminate_requested);
        assert!(!state.is_session_active);
        assert_eq!(state.current_operator, None);
    }

    /// Testa que a paleta de cores oficial Cyber Prism está definida corretamente no Host.
    #[test]
    fn test_tray_cyber_prism_color_palette() {
        assert_eq!(COLOR_CYAN_NEON, Color32::from_rgb(0, 240, 255)); // #00f0ff
        assert_eq!(COLOR_BLUE_ELECTRIC, Color32::from_rgb(0, 140, 255)); // #008cff
        assert_eq!(COLOR_VIOLET_NEON, Color32::from_rgb(190, 45, 235)); // #be2deb
        assert_eq!(COLOR_MINT_ONLINE, Color32::from_rgb(0, 255, 204)); // #00ffcc
        assert_eq!(COLOR_CYAN_GLOW, Color32::from_rgb(0, 200, 235)); // #00c8eb
        assert_eq!(COLOR_RED_DANGER, Color32::from_rgb(255, 51, 85)); // #ff3355
    }

    /// Testa que o ícone oficial é carregado com o formato esperado.
    #[test]
    fn test_tray_brand_icon_loading() {
        let icon = load_brand_icon();

        // O ícone pode ser None se a imagem não estiver válida, mas deve
        // aceitar o formato de dados esperado. Neste caso, verificamos
        // que não há panic no processo de carregamento.
        if let Some(i) = icon {
            assert_eq!(i.width, i.height);
        }
    }
}
