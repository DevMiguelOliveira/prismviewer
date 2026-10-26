#![windows_subsystem = "windows"]

//! prysmview-tray - Aplicativo visual do Host no PDV estilo TeamViewer Host
//!
//! Executa na sessão do usuário comum no PDV (sem privilégios de administrador).
//! Comunica-se exclusivamente com o serviço Windows (prysmview-svc) através
//! de Named Pipe local restrito (`\\.\pipe\prysmview-ipc`).
//!
//! Utiliza a moderna arquitetura WebView2 / Wry (HTML5, CSS3 Cyber Slate e Vanilla JS)
//! idêntica ao console administrativo PrysmAdmin, garantindo máxima fluidez,
//! estética executiva e ergonomia para operadores de caixa.

use prysmview_host::{recv_ipc_message, send_ipc_message, IpcError, IpcMessage, DEFAULT_PIPE_NAME};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
    window::{Icon, WindowBuilder},
};
use tracing::{error, info, warn};
use wry::WebViewBuilder;

// ==============================================================================
// PALETA OFICIAL CYBER PRISM (Baseada no logotipo oficial do PrysmView)
// ==============================================================================
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorRgb(pub u8, pub u8, pub u8);

impl ColorRgb {
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self(r, g, b)
    }
}

pub const COLOR_BG_OBSIDIAN: ColorRgb = ColorRgb::from_rgb(10, 13, 22); // #0a0d16 Fundo principal escuro
pub const COLOR_PANEL_NAVY: ColorRgb = ColorRgb::from_rgb(15, 20, 34); // #0f1422 Fundo de painéis e barras
pub const COLOR_CARD_BASE: ColorRgb = ColorRgb::from_rgb(20, 27, 45); // #141b2d Fundo de cards e containers
pub const COLOR_CARD_BORDER: ColorRgb = ColorRgb::from_rgb(32, 45, 75); // #202d4b Borda de vidro sutil
pub const COLOR_CYAN_NEON: ColorRgb = ColorRgb::from_rgb(0, 240, 255); // #00f0ff Destaque prismático ciano
pub const COLOR_CYAN_GLOW: ColorRgb = ColorRgb::from_rgb(0, 200, 235); // #00c8eb Ciano secundário
pub const COLOR_BLUE_ELECTRIC: ColorRgb = ColorRgb::from_rgb(0, 140, 255); // #008cff Azul elétrico vibrante
pub const COLOR_VIOLET_NEON: ColorRgb = ColorRgb::from_rgb(190, 45, 235); // #be2deb Magenta/violeta do prisma
pub const COLOR_MINT_ONLINE: ColorRgb = ColorRgb::from_rgb(0, 255, 204); // #00ffcc Glow de sinal e status online
pub const COLOR_RED_DANGER: ColorRgb = ColorRgb::from_rgb(255, 51, 85); // #ff3355 Vermelho de emergência
pub const COLOR_TEXT_WHITE: ColorRgb = ColorRgb::from_rgb(245, 248, 255); // #f5f8ff Texto principal
pub const COLOR_TEXT_MUTED: ColorRgb = ColorRgb::from_rgb(148, 163, 184); // #94a3b8 Texto secundário
pub const COLOR_TEXT_DARK: ColorRgb = ColorRgb::from_rgb(100, 116, 139); // #64748b Metadados discretos

/// Carrega o ícone oficial da janela e da barra de tarefas a partir do logo oficial.
pub fn load_brand_icon() -> Option<Icon> {
    let raw = include_bytes!("../../../assets/logo.png");
    let img = image::load_from_memory(raw).ok()?.to_rgba8();
    let (width, height) = img.dimensions();
    Icon::from_rgba(img.into_raw(), width, height).ok()
}

/// Eventos customizados enviados pela thread IPC para a thread de interface WebView.
#[derive(Debug, Clone)]
pub enum HostCustomEvent {
    StateUpdated,
}

/// Estado da solicitação de consentimento pendente.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PendingConsent {
    pub operator_name: String,
    pub reason: String,
    pub timeout_secs: u32,
    pub response: Option<bool>,
}

/// Gerenciador de estado reativo da aplicação Host no PDV.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
        let comp_name = std::env::var("COMPUTERNAME").unwrap_or_default();
        let pdv_id = if !comp_name.is_empty() {
            format!("PDV-CAIXA-01 ({comp_name})")
        } else {
            "PDV-CAIXA-01".to_string()
        };
        Self {
            is_connected_to_service: false,
            pdv_id,
            is_session_active: false,
            current_operator: None,
            current_code: None,
            session_code_expires_in: 120,
            clipboard_enabled: true,
            file_transfer_enabled: true,
            status_message: "Aguardando serviço local prysmview-svc...".to_string(),
            terminate_requested: false,
            pending_consent: None,
        }
    }
}

impl TrayState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Serializa o estado em JSON para consumo pelo frontend JavaScript no WebView2.
    pub fn to_json(&self) -> String {
        serde_json::json!({
            "isConnected": self.is_connected_to_service,
            "pdvId": self.pdv_id,
            "isSessionActive": self.is_session_active,
            "currentOperator": self.current_operator,
            "currentCode": self.current_code,
            "sessionCodeExpiresIn": self.session_code_expires_in,
            "clipboardEnabled": self.clipboard_enabled,
            "fileTransferEnabled": self.file_transfer_enabled,
            "statusMessage": self.status_message,
            "pendingConsent": self.pending_consent.as_ref().map(|c| {
                serde_json::json!({
                    "operatorName": c.operator_name,
                    "reason": c.reason,
                    "timeoutSecs": c.timeout_secs,
                    "response": c.response,
                })
            })
        })
        .to_string()
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

/// Loop principal do cliente IPC da bandeja com o serviço LocalSystem.
async fn run_tray_client(
    pipe_name: String,
    running: Arc<AtomicBool>,
    state_ref: Arc<Mutex<TrayState>>,
    event_proxy: EventLoopProxy<HostCustomEvent>,
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
                let _ = event_proxy.send_event(HostCustomEvent::StateUpdated);
                c
            }
            Err(e) => {
                if let Ok(mut st) = state_ref.lock() {
                    st.is_connected_to_service = false;
                    st.status_message = "Aguardando serviço local prysmview-svc...".to_string();
                }
                let _ = event_proxy.send_event(HostCustomEvent::StateUpdated);
                warn!("Aguardando serviço prysmview-svc iniciar ({e}). Reconectando em 2s...");
                tokio::time::sleep(Duration::from_secs(2)).await;
                continue;
            }
        };

        loop {
            // 1. Verifica se o operador local clicou no botão de encerrar sessão
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
                let _ = event_proxy.send_event(HostCustomEvent::StateUpdated);
            }

            // 2. Verifica se houve resposta a consentimento pendente
            let consent_answer = state_ref.lock().ok().and_then(|mut st| {
                if let Some(ref mut c) = st.pending_consent {
                    if let Some(accepted) = c.response.take() {
                        return Some((c.operator_name.clone(), accepted));
                    }
                }
                None
            });

            if let Some((op, accepted)) = consent_answer {
                info!(
                    operator = %op,
                    accepted = accepted,
                    "Enviando resposta de consentimento ao prysmview-svc"
                );
                let _ = send_ipc_message(
                    &mut client,
                    &IpcMessage::ConsentResponse {
                        accepted,
                    },
                )
                .await;
                if let Ok(mut st) = state_ref.lock() {
                    st.pending_consent = None;
                }
                let _ = event_proxy.send_event(HostCustomEvent::StateUpdated);
            }

            // 3. Recebe mensagens do prysmview-svc
            let msg = match recv_ipc_message(&mut client).await {
                Ok(m) => m,
                Err(IpcError::ConnectionClosed) => {
                    warn!("Serviço prysmview-svc encerrou a conexão IPC.");
                    if let Ok(mut st) = state_ref.lock() {
                        st.is_connected_to_service = false;
                    }
                    let _ = event_proxy.send_event(HostCustomEvent::StateUpdated);
                    break;
                }
                Err(e) => {
                    error!("Erro de comunicação IPC: {e}");
                    if let Ok(mut st) = state_ref.lock() {
                        st.is_connected_to_service = false;
                    }
                    let _ = event_proxy.send_event(HostCustomEvent::StateUpdated);
                    break;
                }
            };

            let maybe_reply = if let Ok(mut st) = state_ref.lock() {
                let rep = st.process_message(&msg);
                let _ = event_proxy.send_event(HostCustomEvent::StateUpdated);
                rep
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

#[cfg(windows)]
fn show_error_dialog(title: &str, message: &str) {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};

    let title_wide: Vec<u16> = OsStr::new(title)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let msg_wide: Vec<u16> = OsStr::new(message)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let _ = MessageBoxW(
            HWND::default(),
            PCWSTR(msg_wide.as_ptr()),
            PCWSTR(title_wide.as_ptr()),
            MB_OK | MB_ICONERROR,
        );
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt::try_init();
    info!("Iniciando PrysmView Host no PDV (Interface Moderna WebView2)...");

    let state = Arc::new(Mutex::new(TrayState::new()));
    let state_bg = state.clone();
    let state_render = state.clone();
    let state_ipc = state.clone();

    let running = Arc::new(AtomicBool::new(true));
    let running_bg = running.clone();

    // Event Loop do Tao com suporte a eventos de usuário (HostCustomEvent)
    let event_loop = EventLoopBuilder::<HostCustomEvent>::with_user_event().build();
    let event_proxy = event_loop.create_proxy();

    // Thread assíncrona dedicada para a comunicação Named Pipe com prysmview-svc
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");
        rt.block_on(async {
            let _ = run_tray_client(
                DEFAULT_PIPE_NAME.to_string(),
                running_bg,
                state_bg,
                event_proxy,
            )
            .await;
        });
    });

    let mut window_builder = WindowBuilder::new()
        .with_title("PrysmView Host - Atendimento ao PDV")
        .with_inner_size(LogicalSize::new(410.0, 640.0))
        .with_min_inner_size(LogicalSize::new(360.0, 520.0))
        .with_resizable(true)
        .with_visible(true);

    if let Some(icon) = load_brand_icon() {
        window_builder = window_builder.with_window_icon(Some(icon));
    }

    let window = match window_builder.build(&event_loop) {
        Ok(w) => w,
        Err(e) => {
            #[cfg(windows)]
            show_error_dialog(
                "PrysmView Host - Erro",
                &format!("Falha ao criar janela do aplicativo: {e}"),
            );
            return Err(e.into());
        }
    };
    window.set_focus();

    // Embutir HTML, CSS e JavaScript diretamente no binário para portabilidade total e segurança zero-trust
    let raw_html = include_str!("../ui/index.html");
    let css_content = include_str!("../ui/style.css");
    let js_content = include_str!("../ui/host.js");

    let initial_json = state_render.lock().unwrap().to_json();
    let state_injection = format!(
        "<script>\nwindow.__INITIAL_HOST_STATE__ = {};\n</script>",
        initial_json
    );

    let bundled_html = raw_html
        .replace(
            r#"<link rel="stylesheet" href="style.css">"#,
            &format!("<style>\n{}\n</style>", css_content),
        )
        .replace(
            r#"<script src="host.js"></script>"#,
            &format!("{}\n<script>\n{}\n</script>", state_injection, js_content),
        );

    // Diretório dedicado no AppData do usuário para o WebView2.
    // IMPRESCINDÍVEL: Quando instalado em "C:\Program Files", usuários comuns não possuem
    // permissão de gravação no diretório do programa. O WebView2 falha silenciosamente com
    // E_ACCESSDENIED (0x80070005) a menos que um data_directory gravável (%LOCALAPPDATA%) seja configurado.
    let data_dir = std::env::var_os("LOCALAPPDATA")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("PrysmView")
        .join("host_webview");
    let _ = std::fs::create_dir_all(&data_dir);
    std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", &data_dir);

    let mut web_context = wry::WebContext::new(Some(data_dir));

    // Constrói a WebView vinculada à janela nativa do Windows
    let webview = match WebViewBuilder::new_with_web_context(&mut web_context)
        .with_devtools(true)
        .with_html(&bundled_html)
        .with_ipc_handler(move |msg| {
            let body = msg.body();
            info!("Comando recebido da interface do Host: {}", body);
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(body) {
                let action = val.get("action").and_then(|v| v.as_str()).unwrap_or("");
                match action {
                    "terminate_session" => {
                        info!("Encerramento de sessão acionado pela interface do Host");
                        if let Ok(mut st) = state_ipc.lock() {
                            st.request_terminate();
                        }
                    }
                    "consent_response" => {
                        let accept = val.get("accept").and_then(|v| v.as_bool()).unwrap_or(false);
                        info!(accept = accept, "Resposta de consentimento definida pelo operador local");
                        if let Ok(mut st) = state_ipc.lock() {
                            if let Some(ref mut c) = st.pending_consent {
                                c.response = Some(accept);
                            }
                        }
                    }
                    "toggle_clipboard" => {
                        let enabled = val.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
                        if let Ok(mut st) = state_ipc.lock() {
                            st.clipboard_enabled = enabled;
                        }
                    }
                    "toggle_file_transfer" => {
                        let enabled = val.get("enabled").and_then(|v| v.as_bool()).unwrap_or(true);
                        if let Ok(mut st) = state_ipc.lock() {
                            st.file_transfer_enabled = enabled;
                        }
                    }
                    "window_minimize" => {
                        // Minimizar janela do Host
                    }
                    "window_close" => {
                        // Fechar / ocultar janela do Host
                    }
                    _ => {}
                }
            }
        })
        .build(&window)
    {
        Ok(wv) => wv,
        Err(e) => {
            error!("Falha ao instanciar WebView2: {e}");
            #[cfg(windows)]
            show_error_dialog(
                "PrysmView Host - Erro de Inicialização",
                &format!(
                    "Não foi possível abrir a janela do PrysmView Host.\n\n\
                    O componente visual Microsoft Edge WebView2 Runtime falhou ao carregar:\n{}\n\n\
                    Se este computador não possuir o WebView2 instalado, instale o 'Microsoft Edge WebView2 Runtime' para utilizar o Host.",
                    e
                ),
            );
            return Err(e.into());
        }
    };

    info!("PrysmView Host inicializado com sucesso em modo WebView2.");

    event_loop.run(move |event, _, control_flow| {
        let _ = &webview;
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(HostCustomEvent::StateUpdated) => {
                if let Ok(st) = state_render.lock() {
                    let json = st.to_json();
                    let script = format!("if (window.updateHostState) {{ window.updateHostState({}); }}", json);
                    let _ = webview.evaluate_script(&script);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                info!("Fechando janela do PrysmView Host...");
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
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
        assert_eq!(COLOR_CYAN_NEON, ColorRgb::from_rgb(0, 240, 255)); // #00f0ff
        assert_eq!(COLOR_BLUE_ELECTRIC, ColorRgb::from_rgb(0, 140, 255)); // #008cff
        assert_eq!(COLOR_VIOLET_NEON, ColorRgb::from_rgb(190, 45, 235)); // #be2deb
        assert_eq!(COLOR_MINT_ONLINE, ColorRgb::from_rgb(0, 255, 204)); // #00ffcc
        assert_eq!(COLOR_CYAN_GLOW, ColorRgb::from_rgb(0, 200, 235)); // #00c8eb
        assert_eq!(COLOR_RED_DANGER, ColorRgb::from_rgb(255, 51, 85)); // #ff3355
    }

    /// Testa que o ícone oficial é carregado com o formato esperado.
    #[test]
    fn test_tray_brand_icon_loading() {
        let icon = load_brand_icon();
        assert!(icon.is_some());
    }

    #[test]
    fn test_tray_state_to_json_serialization() {
        let mut state = TrayState::new();
        state.pdv_id = "CAIXA-09".to_string();
        state.current_code = Some("12345678".to_string());
        let json = state.to_json();
        assert!(json.contains("CAIXA-09"));
        assert!(json.contains("12345678"));
        assert!(json.contains("isConnected"));
    }
}
