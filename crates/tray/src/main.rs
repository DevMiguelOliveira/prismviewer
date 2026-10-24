//! prysmview-tray - Aplicativo da bandeja do usuário no PDV
//!
//! Executa sem privilégios elevados na sessão do usuário logado.
//! Comunica-se exclusivamente com o serviço Windows (prysmview-svc) através
//! de Named Pipe local restrito (`\\.\pipe\prysmview-ipc`).
//!
//! Responsabilidades:
//! - Exibir janela de consentimento com contagem regressiva estrita de 60 segundos;
//! - Exibir código de sessão de 8 dígitos para o operador da loja;
//! - Exibir indicador visual de sessão ativa com botão de encerramento imediato (WS_EX_TOPMOST);
//! - Rejeitar automaticamente solicitações expiradas sem consentimento.

use prysmview_host::{recv_ipc_message, send_ipc_message, IpcError, IpcMessage, DEFAULT_PIPE_NAME};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

/// Gerenciador de estado reativo da aplicação de bandeja.
#[derive(Debug, Default)]
pub struct TrayState {
    pub is_session_active: bool,
    pub current_operator: Option<String>,
    pub current_code: Option<String>,
    pub session_code_expires_in: u32,
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
                    "Solicitação de consentimento recebida"
                );
                None
            }
            IpcMessage::ConsentResponse { .. } => None,
            IpcMessage::SessionCodeDisplay {
                code,
                expires_in_secs,
            } => {
                info!(code = %code, expires = %expires_in_secs, "Código de sessão recebido para exibição");
                self.current_code = Some(code.clone());
                self.session_code_expires_in = *expires_in_secs;
                None
            }
            IpcMessage::SessionActive { operator_name } => {
                info!(operator = %operator_name, "Sessão remota iniciada e ativa");
                self.is_session_active = true;
                self.current_operator = Some(operator_name.clone());
                None
            }
            IpcMessage::SessionEnded { reason } => {
                info!(reason = %reason, "Sessão remota encerrada");
                self.is_session_active = false;
                self.current_operator = None;
                self.current_code = None;
                None
            }
            IpcMessage::TerminateSession { reason } => {
                info!(reason = %reason, "Encerramento de sessão solicitado");
                self.is_session_active = false;
                None
            }
            IpcMessage::Ping(val) => Some(IpcMessage::Pong(*val)),
            IpcMessage::Pong(_) => None,
        }
    }
}

#[cfg(windows)]
mod ui {
    use std::ffi::c_void;
    use tracing::info;

    // Constantes da API Win32 User32
    const MB_YESNO: u32 = 0x00000004;
    const MB_ICONQUESTION: u32 = 0x00000020;
    const MB_ICONINFORMATION: u32 = 0x00000040;
    const MB_TOPMOST: u32 = 0x00040000;
    const MB_SETFOREGROUND: u32 = 0x00010000;
    const IDYES: i32 = 6;

    #[link(name = "user32")]
    extern "system" {
        fn MessageBoxTimeoutW(
            hwnd: *mut c_void,
            lptext: *const u16,
            lpcaption: *const u16,
            utype: u32,
            wlanguageid: u16,
            dwmilliseconds: u32,
        ) -> i32;
    }

    /// Exibe uma janela de consentimento nativa TopMost com contagem regressiva estrita.
    /// Se o usuário não responder dentro do tempo limite, a função retorna automaticamente `false`.
    pub fn prompt_consent_dialog(operator_name: &str, reason: &str, timeout_secs: u32) -> bool {
        let title_wide: Vec<u16> = "PrysmView - Solicitação de Acesso Remoto\0"
            .encode_utf16()
            .collect();

        let message = format!(
            "O operador '{}' solicita acesso remoto a este PDV.\n\n\
            Motivo informado: {}\n\n\
            Tempo restante para resposta: {} segundos.\n\n\
            Deseja PERMITIR o acesso remoto?",
            operator_name, reason, timeout_secs
        );
        let message_wide: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();

        let flags = MB_YESNO | MB_ICONQUESTION | MB_TOPMOST | MB_SETFOREGROUND;
        let timeout_ms = timeout_secs.saturating_mul(1000);

        info!(timeout_secs, "Exibindo caixa de diálogo de consentimento");

        // SAFETY: MessageBoxTimeoutW lê strings null-terminated válidas com ponteiro hwnd nulo.
        let result = unsafe {
            MessageBoxTimeoutW(
                std::ptr::null_mut(),
                message_wide.as_ptr(),
                title_wide.as_ptr(),
                flags,
                0,
                timeout_ms,
            )
        };

        result == IDYES
    }

    /// Exibe o código de sessão de 8 dígitos para o operador da loja.
    pub fn display_session_code(code: &str, expires_in_secs: u32) {
        let title_wide: Vec<u16> = "PrysmView - Código de Sessão\0".encode_utf16().collect();

        let message = format!(
            "Código de Acesso do PDV: {}\n\n\
            Informe este código de 8 dígitos para o operador de suporte.\n\
            O código expira em {} segundos (máximo 3 tentativas).",
            code, expires_in_secs
        );
        let message_wide: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();

        let flags = MB_ICONINFORMATION | MB_TOPMOST | MB_SETFOREGROUND;
        let timeout_ms = expires_in_secs.saturating_mul(1000);

        // SAFETY: MessageBoxTimeoutW lê strings null-terminated válidas.
        unsafe {
            let _ = MessageBoxTimeoutW(
                std::ptr::null_mut(),
                message_wide.as_ptr(),
                title_wide.as_ptr(),
                flags,
                0,
                timeout_ms,
            );
        }
    }
}

#[cfg(not(windows))]
mod ui {
    pub fn prompt_consent_dialog(_operator_name: &str, _reason: &str, _timeout_secs: u32) -> bool {
        false
    }

    pub fn display_session_code(_code: &str, _expires_in_secs: u32) {}
}

/// Loop principal do cliente IPC da bandeja.
async fn run_tray_client(pipe_name: String, running: Arc<AtomicBool>) -> Result<(), IpcError> {
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
                c
            }
            Err(e) => {
                warn!("Aguardando serviço prysmview-svc iniciar ({e}). Reconectando em 2s...");
                tokio::time::sleep(Duration::from_secs(2)).await;
                continue;
            }
        };

        let mut state = TrayState::new();

        loop {
            let msg = match recv_ipc_message(&mut client).await {
                Ok(m) => m,
                Err(IpcError::ConnectionClosed) => {
                    warn!("Serviço prysmview-svc encerrou a conexão IPC.");
                    break;
                }
                Err(e) => {
                    error!("Erro de comunicação IPC: {e}");
                    break;
                }
            };

            if let Some(reply) = state.process_message(&msg) {
                if let Err(e) = send_ipc_message(&mut client, &reply).await {
                    error!("Falha ao enviar resposta IPC: {e}");
                    break;
                }
            }

            match msg {
                IpcMessage::ConsentRequest {
                    operator_name,
                    reason,
                    timeout_secs,
                } => {
                    let op_name = operator_name.clone();
                    let reas = reason.clone();
                    let accepted = tokio::task::spawn_blocking(move || {
                        ui::prompt_consent_dialog(&op_name, &reas, timeout_secs)
                    })
                    .await
                    .unwrap_or(false);

                    info!(
                        accepted,
                        "Resposta de consentimento definida pelo operador local"
                    );
                    let resp = IpcMessage::ConsentResponse { accepted };
                    if let Err(e) = send_ipc_message(&mut client, &resp).await {
                        error!("Falha ao enviar resposta de consentimento: {e}");
                        break;
                    }
                }
                IpcMessage::SessionCodeDisplay {
                    code,
                    expires_in_secs,
                } => {
                    tokio::task::spawn_blocking(move || {
                        ui::display_session_code(&code, expires_in_secs);
                    });
                }
                _ => {}
            }
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Iniciando prysmview-tray na sessão do usuário...");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    tokio::spawn(async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            warn!("Erro ao registrar handler de Ctrl+C: {e}");
        }
        info!("Sinal de término recebido. Encerrando prysmview-tray...");
        r.store(false, Ordering::Relaxed);
    });

    if let Err(e) = run_tray_client(DEFAULT_PIPE_NAME.to_string(), running).await {
        error!("Erro fatal no prysmview-tray: {e}");
    }
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
}
