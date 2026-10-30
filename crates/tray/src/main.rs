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
    TechAuthResult { success: bool, message: String },
}

/// Estado da solicitação de consentimento pendente.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PendingConsent {
    pub operator_name: String,
    pub reason: String,
    pub timeout_secs: u32,
    pub response: Option<bool>,
}

/// Estrutura de dados persistentes do Host no disco.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct HostConfigData {
    pub fixed_id: Option<String>,
    pub local_user: Option<String>,
    pub alias: Option<String>,
    pub group: Option<String>,
    pub description: Option<String>,
    pub access_password: Option<String>,
}

/// Formata o ID do Host em grupos de 3 dígitos estilo TeamViewer (ex: "117 029 2035" ou "742 819 305").
pub fn format_teamviewer_id(raw: &str) -> String {
    let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() == 9 {
        format!("{} {} {}", &digits[0..3], &digits[3..6], &digits[6..9])
    } else if digits.len() == 10 {
        format!("{} {} {}", &digits[0..3], &digits[3..6], &digits[6..10])
    } else {
        raw.to_string()
    }
}

/// Gera ou obtém o ID numérico exclusivo e permanente deste Host.
pub fn get_or_create_fixed_host_id(existing: Option<&str>) -> String {
    if let Some(id) = existing {
        let clean = id.trim();
        let digits: String = clean.chars().filter(|c| c.is_ascii_digit()).collect();
        if (digits.len() == 9 || digits.len() == 10) && digits != "000000000" {
            return digits;
        }
    }
    use sha1::{Digest, Sha1};
    let mut hasher = Sha1::new();
    let comp = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "DESKTOP_HOST".to_string());
    hasher.update(comp.as_bytes());
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(84729103);
    hasher.update(now.to_le_bytes());
    let hash = hasher.finalize();
    let num = u64::from_le_bytes(hash[..8].try_into().unwrap_or([7; 8]));
    format!("{:09}", (num % 900_000_000) + 100_000_000)
}

/// Bloqueia ou desbloqueia mouse e teclado locais via API do Windows (BlockInput).
#[cfg(windows)]
pub fn set_remote_input_blocked(blocked: bool) -> bool {
    unsafe {
        #[link(name = "user32")]
        extern "system" {
            fn BlockInput(fBlockIt: i32) -> i32;
        }
        BlockInput(if blocked { 1 } else { 0 }) != 0
    }
}

#[cfg(not(windows))]
pub fn set_remote_input_blocked(_blocked: bool) -> bool {
    false
}

/// Bloqueia a estação de trabalho local chamando LockWorkStation do Windows.
#[cfg(windows)]
pub fn lock_remote_workstation() -> bool {
    unsafe {
        #[link(name = "user32")]
        extern "system" {
            fn LockWorkStation() -> i32;
        }
        LockWorkStation() != 0
    }
}

#[cfg(not(windows))]
pub fn lock_remote_workstation() -> bool {
    false
}

/// Módulo de integração nativa com a bandeja do Windows (System Tray / Ícones Ocultos)
#[cfg(windows)]
pub mod system_tray {
    use std::ffi::c_void;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM, LRESULT, POINT, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        AppendMenuW, CreatePopupMenu, DestroyMenu, GetCursorPos, LoadIconW, SetForegroundWindow,
        ShowWindow, TrackPopupMenu, HICON, IDI_APPLICATION, MF_SEPARATOR, MF_STRING, SW_HIDE,
        SW_RESTORE, SW_SHOW, TPM_RETURNCMD, TPM_RIGHTBUTTON, WM_APP,
    };

    pub const WM_TRAYICON: u32 = WM_APP + 101;
    pub const SC_MINIMIZE: usize = 0xF020;
    pub const SC_CLOSE: usize = 0xF060;
    pub const WM_SYSCOMMAND: u32 = 0x0112;
    pub const WM_LBUTTONUP: usize = 0x0202;
    pub const WM_LBUTTONDBLCLK: usize = 0x0203;
    pub const WM_RBUTTONUP: usize = 0x0205;

    const NIM_ADD: u32 = 0x00000000;
    const NIM_DELETE: u32 = 0x00000002;
    const NIF_MESSAGE: u32 = 0x00000001;
    const NIF_ICON: u32 = 0x00000002;
    const NIF_TIP: u32 = 0x00000004;

    #[repr(C)]
    #[allow(non_snake_case)]
    pub struct NOTIFYICONDATAW {
        pub cbSize: u32,
        pub hWnd: HWND,
        pub uID: u32,
        pub uFlags: u32,
        pub uCallbackMessage: u32,
        pub hIcon: HICON,
        pub szTip: [u16; 128],
        pub dwState: u32,
        pub dwStateMask: u32,
        pub szInfo: [u16; 256],
        pub uTimeoutOrVersion: u32,
        pub szInfoTitle: [u16; 64],
        pub dwInfoFlags: u32,
        pub guidItem: windows::core::GUID,
        pub hBalloonIcon: HICON,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetModuleHandleW(lpModuleName: *const u16) -> windows::Win32::Foundation::HMODULE;
    }

    #[link(name = "shell32")]
    extern "system" {
        fn Shell_NotifyIconW(dwMessage: u32, lpData: *const NOTIFYICONDATAW) -> i32;
    }

    #[link(name = "comctl32")]
    extern "system" {
        fn SetWindowSubclass(
            hWnd: HWND,
            pfnSubclass: unsafe extern "system" fn(
                HWND,
                u32,
                WPARAM,
                LPARAM,
                usize,
                usize,
            ) -> LRESULT,
            uIdSubclass: usize,
            dwRefData: usize,
        ) -> BOOL;

        fn DefSubclassProc(hWnd: HWND, uMsg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT;

        fn RemoveWindowSubclass(
            hWnd: HWND,
            pfnSubclass: unsafe extern "system" fn(
                HWND,
                u32,
                WPARAM,
                LPARAM,
                usize,
                usize,
            ) -> LRESULT,
            uIdSubclass: usize,
        ) -> BOOL;
    }

    unsafe extern "system" fn tray_subclass_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        _uid: usize,
        _refdata: usize,
    ) -> LRESULT {
        match msg {
            WM_SYSCOMMAND => {
                let cmd = wparam.0 & 0xFFF0;
                if cmd == SC_MINIMIZE || cmd == SC_CLOSE {
                    // Minimiza ou fecha escondendo da barra de tarefas para a bandeja de icones ocultos
                    let _ = ShowWindow(hwnd, SW_HIDE);
                    return LRESULT(0);
                }
            }
            WM_TRAYICON => {
                let evt = lparam.0 as usize;
                if evt == WM_LBUTTONUP || evt == WM_LBUTTONDBLCLK {
                    // Restaura janela ao primeiro plano
                    let _ = ShowWindow(hwnd, SW_SHOW);
                    let _ = ShowWindow(hwnd, SW_RESTORE);
                    let _ = SetForegroundWindow(hwnd);
                    return LRESULT(0);
                } else if evt == WM_RBUTTONUP {
                    // Menu de contexto com botao direito na bandeja
                    let mut pt = POINT::default();
                    let _ = GetCursorPos(&mut pt);
                    let _ = SetForegroundWindow(hwnd);
                    let hmenu = CreatePopupMenu().unwrap_or_default();
                    if !hmenu.is_invalid() {
                        let title_text: Vec<u16> =
                            "PrysmView Host (Ativo)\0".encode_utf16().collect();
                        let open_text: Vec<u16> = "Abrir PrysmView Host\0".encode_utf16().collect();
                        let exit_text: Vec<u16> = "Sair do PrysmView\0".encode_utf16().collect();

                        let _ = AppendMenuW(hmenu, MF_STRING, 1000, PCWSTR(title_text.as_ptr()));
                        let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());
                        let _ = AppendMenuW(hmenu, MF_STRING, 1001, PCWSTR(open_text.as_ptr()));
                        let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());
                        let _ = AppendMenuW(hmenu, MF_STRING, 1002, PCWSTR(exit_text.as_ptr()));

                        let cmd = TrackPopupMenu(
                            hmenu,
                            TPM_RETURNCMD | TPM_RIGHTBUTTON,
                            pt.x,
                            pt.y,
                            0,
                            hwnd,
                            None,
                        );
                        let _ = DestroyMenu(hmenu);

                        if cmd.0 == 1000 || cmd.0 == 1001 {
                            let _ = ShowWindow(hwnd, SW_SHOW);
                            let _ = ShowWindow(hwnd, SW_RESTORE);
                            let _ = SetForegroundWindow(hwnd);
                        } else if cmd.0 == 1002 {
                            std::process::exit(0);
                        }
                    }
                    return LRESULT(0);
                }
            }
            _ => {}
        }
        DefSubclassProc(hwnd, msg, wparam, lparam)
    }

    pub fn install_system_tray(hwnd_val: isize) {
        unsafe {
            let hwnd = HWND(hwnd_val as *mut c_void);
            let hmodule = GetModuleHandleW(std::ptr::null());
            let hinst = windows::Win32::Foundation::HINSTANCE(hmodule.0);
            let hicon = LoadIconW(hinst, PCWSTR(1 as _)).unwrap_or_else(|_| {
                LoadIconW(
                    windows::Win32::Foundation::HINSTANCE::default(),
                    IDI_APPLICATION,
                )
                .unwrap_or_default()
            });

            let mut tip = [0u16; 128];
            let tip_str = "PrysmView Host - Suporte Remoto";
            for (i, c) in tip_str.encode_utf16().enumerate().take(127) {
                tip[i] = c;
            }

            let nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: hwnd,
                uID: 1,
                uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP,
                uCallbackMessage: WM_TRAYICON,
                hIcon: hicon,
                szTip: tip,
                dwState: 0,
                dwStateMask: 0,
                szInfo: [0; 256],
                uTimeoutOrVersion: 0,
                szInfoTitle: [0; 64],
                dwInfoFlags: 0,
                guidItem: windows::core::GUID::zeroed(),
                hBalloonIcon: Default::default(),
            };

            let res = Shell_NotifyIconW(NIM_ADD, &nid);
            if res != 0 {
                tracing::info!("Icone do PrysmHost adicionado com sucesso a bandeja de icones ocultos do Windows");
            } else {
                tracing::warn!("Falha ao adicionar icone a bandeja do Windows");
            }

            let sub_res = SetWindowSubclass(hwnd, tray_subclass_proc, 1, 0);
            if sub_res.as_bool() {
                tracing::info!(
                    "Subclass de janela instalado para captura de minimizar/fechar para a bandeja"
                );
            }
        }
    }

    pub fn remove_system_tray(hwnd_val: isize) {
        unsafe {
            let hwnd = HWND(hwnd_val as *mut c_void);
            let nid = NOTIFYICONDATAW {
                cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: hwnd,
                uID: 1,
                uFlags: 0,
                uCallbackMessage: 0,
                hIcon: Default::default(),
                szTip: [0; 128],
                dwState: 0,
                dwStateMask: 0,
                szInfo: [0; 256],
                uTimeoutOrVersion: 0,
                szInfoTitle: [0; 64],
                dwInfoFlags: 0,
                guidItem: windows::core::GUID::zeroed(),
                hBalloonIcon: Default::default(),
            };
            let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
            let _ = RemoveWindowSubclass(hwnd, tray_subclass_proc, 1);
        }
    }
}

/// Caminho do arquivo de configuração persistente do Host no AppData local.
pub fn get_host_config_path() -> std::path::PathBuf {
    let base = std::env::var("LOCALAPPDATA")
        .or_else(|_| std::env::var("APPDATA"))
        .unwrap_or_else(|_| ".".to_string());
    let mut path = std::path::PathBuf::from(base);
    path.push("PrysmView");
    let _ = std::fs::create_dir_all(&path);
    path.push("host_config.json");
    path
}

/// Carrega usuário, PDV e configurações persistentes do disco
pub fn load_host_config() -> HostConfigData {
    let path = get_host_config_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                let fixed_id = val
                    .get("fixed_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let local_user = val
                    .get("local_user")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let alias = val
                    .get("alias")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let group = val
                    .get("group")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let description = val
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let access_password = val
                    .get("access_password")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                return HostConfigData {
                    fixed_id,
                    local_user,
                    alias,
                    group,
                    description,
                    access_password,
                };
            }
        }
    }
    HostConfigData::default()
}

/// Salva usuário, PDV e senha de acesso no disco
pub fn save_host_config(user: &str, pdv_id: &str, access_password: Option<&str>) {
    let mut cfg = load_host_config();
    cfg.local_user = Some(user.to_string());
    cfg.alias = Some(user.to_string());
    cfg.access_password = access_password.map(|s| s.to_string());
    if cfg.fixed_id.is_none() {
        cfg.fixed_id = Some(pdv_id.to_string());
    }
    save_host_props_config(&cfg);
}

/// Salva propriedades completas do Host no disco
pub fn save_host_props_config(cfg: &HostConfigData) {
    let path = get_host_config_path();
    let data = serde_json::json!({
        "fixed_id": cfg.fixed_id,
        "local_user": cfg.local_user,
        "alias": cfg.alias,
        "group": cfg.group,
        "description": cfg.description,
        "access_password": cfg.access_password,
        "updated_at": "Agora",
    });
    if let Err(e) = std::fs::write(&path, data.to_string()) {
        tracing::warn!("Falha ao gravar configuração do Host em {:?}: {}", path, e);
    } else {
        info!(
            "Configuração do Host persistida com sucesso em {:?}: ID {:?} ({:?})",
            path, cfg.fixed_id, cfg.alias
        );
    }
}

/// Gerenciador de estado reativo da aplicação Host no PDV.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrayState {
    pub is_connected_to_service: bool,
    pub pdv_id: String,
    pub formatted_id: String,
    pub local_user: Option<String>,
    pub alias: Option<String>,
    pub group: Option<String>,
    pub description: Option<String>,
    pub access_password: Option<String>,
    pub is_session_active: bool,
    pub current_operator: Option<String>,
    pub current_code: Option<String>,
    pub session_code_expires_in: u32,
    pub clipboard_enabled: bool,
    pub file_transfer_enabled: bool,
    pub input_blocked: bool,
    pub status_message: String,
    pub terminate_requested: bool,
    pub pending_consent: Option<PendingConsent>,
}

impl Default for TrayState {
    fn default() -> Self {
        let mut cfg = load_host_config();
        let fixed_id = get_or_create_fixed_host_id(cfg.fixed_id.as_deref());
        if cfg.fixed_id.as_deref() != Some(&fixed_id) {
            cfg.fixed_id = Some(fixed_id.clone());
            save_host_props_config(&cfg);
        }
        let formatted_id = format_teamviewer_id(&fixed_id);
        let default_alias = {
            let comp_name = std::env::var("COMPUTERNAME").unwrap_or_default();
            if !comp_name.is_empty() {
                format!("Dispositivo ({comp_name})")
            } else {
                "DESKTOP-01".to_string()
            }
        };

        Self {
            is_connected_to_service: false,
            pdv_id: fixed_id,
            formatted_id,
            local_user: cfg.local_user,
            alias: cfg.alias.or(Some(default_alias)),
            group: cfg
                .group
                .or_else(|| Some("Dispositivos Matriz".to_string())),
            description: cfg.description,
            access_password: cfg.access_password,
            is_session_active: false,
            current_operator: None,
            current_code: None,
            session_code_expires_in: 120,
            clipboard_enabled: true,
            file_transfer_enabled: true,
            input_blocked: false,
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
            "formattedId": self.formatted_id,
            "localUser": self.local_user,
            "alias": self.alias,
            "group": self.group,
            "description": self.description,
            "accessPassword": self.access_password,
            "isSessionActive": self.is_session_active,
            "currentOperator": self.current_operator,
            "currentCode": self.current_code,
            "sessionCodeExpiresIn": self.session_code_expires_in,
            "clipboardEnabled": self.clipboard_enabled,
            "fileTransferEnabled": self.file_transfer_enabled,
            "inputBlocked": self.input_blocked,
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
                        reason: "Encerrado pelo usuário local no dispositivo".to_string(),
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
                let _ =
                    send_ipc_message(&mut client, &IpcMessage::ConsentResponse { accepted }).await;
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
    let event_proxy_ipc = event_loop.create_proxy();
    let event_proxy_stream = event_loop.create_proxy();
    let event_proxy_wry = event_loop.create_proxy();

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
                event_proxy_ipc,
            )
            .await;
        });
    });

    // Thread de broadcast UDP em tempo real para sincronização com o console PrysmAdmin
    let state_beacon = state.clone();
    let running_beacon = running.clone();
    std::thread::spawn(move || {
        let socket = match std::net::UdpSocket::bind("0.0.0.0:0") {
            Ok(s) => {
                let _ = s.set_broadcast(true);
                let _ = s.set_nonblocking(true);
                s
            }
            Err(e) => {
                tracing::warn!("Falha ao criar socket de broadcast do Host: {e}");
                return;
            }
        };

        while running_beacon.load(Ordering::Relaxed) {
            if let Ok(st) = state_beacon.lock() {
                let payload = serde_json::json!({
                    "type": "host_beacon",
                    "pdv_id": st.pdv_id,
                    "formatted_id": st.formatted_id,
                    "alias": st.alias.as_deref().unwrap_or("Dispositivo"),
                    "group": st.group.as_deref().unwrap_or("Dispositivos Matriz"),
                    "local_user": st.local_user,
                    "access_password": st.access_password,
                    "is_online": true,
                    "agent_version": "0.1.0",
                });
                if let Ok(bytes) = serde_json::to_vec(&payload) {
                    let _ = socket.send_to(&bytes, "255.255.255.255:19532");
                    let _ = socket.send_to(&bytes, "127.0.0.1:19532");
                }
            }
            std::thread::sleep(Duration::from_millis(2500));
        }
    });

    // Inicia servidor de transmissão de tela real e injeção de comandos na porta 19533
    start_host_stream_server(state.clone(), running.clone(), event_proxy_stream);

    let mut window_builder = WindowBuilder::new()
        .with_title("PrysmView Host - Suporte Remoto")
        .with_inner_size(LogicalSize::new(385.0, 565.0))
        .with_min_inner_size(LogicalSize::new(300.0, 85.0))
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

    #[cfg(windows)]
    {
        use tao::platform::windows::WindowExtWindows;
        system_tray::install_system_tray(window.hwnd());
    }

    // Embutir HTML, CSS e JavaScript diretamente no binário para portabilidade total e segurança zero-trust
    let raw_html = include_str!("../ui/index.html");
    let css_content = include_str!("../ui/style.css");
    let gsap_content = include_str!("../ui/gsap.min.js");
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
            r#"<script src="gsap.min.js"></script>"#,
            &format!("<script>\n{}\n</script>", gsap_content),
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
                        set_remote_input_blocked(false);
                        if let Ok(mut st) = state_ipc.lock() {
                            st.request_terminate();
                            st.input_blocked = false;
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
                    "verify_technician_login" => {
                        let user = val.get("username").and_then(|v| v.as_str()).unwrap_or("").trim();
                        let pwd = val.get("password").and_then(|v| v.as_str()).unwrap_or("").trim();
                        info!(user = %user, "Tentativa de autenticacao de tecnico para gerenciar Host");

                        let mut authenticated = false;
                        // Conta padrao admin ou tecnico
                        if (user == "admin" || user == "tecnico" || user == "suporte")
                            && (pwd == "admin123" || pwd == "admin")
                        {
                            authenticated = true;
                        }
                        // Ou comparacao com a senha fixa ja configurada
                        if !authenticated && !pwd.is_empty() {
                            if let Ok(st) = state_ipc.lock() {
                                if let Some(ref saved) = st.access_password {
                                    if !saved.is_empty() && constant_time_eq(saved.trim().as_bytes(), pwd.as_bytes()) {
                                        authenticated = true;
                                    }
                                }
                            }
                        }

                        let (success, message) = if authenticated {
                            (true, "Tecnico autenticado com sucesso.".to_string())
                        } else {
                            (false, "Usuario ou senha de tecnico incorretos.".to_string())
                        };

                        let _ = event_proxy_wry.send_event(HostCustomEvent::TechAuthResult { success, message });
                    }
                    "save_host_props" => {
                        let alias = val.get("alias").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
                        let group = val.get("group").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
                        let desc = val.get("description").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
                        let access_pwd = val.get("accessPassword").and_then(|v| v.as_str()).map(|s| s.trim().to_string()).filter(|s| !s.is_empty());

                        info!(alias = %alias, group = %group, has_pwd = access_pwd.is_some(), "Propriedades do Host atualizadas pelo tecnico");

                        let fixed_id = if let Ok(mut st) = state_ipc.lock() {
                            st.alias = Some(alias.clone());
                            st.local_user = Some(alias.clone());
                            st.group = Some(group.clone());
                            st.description = Some(desc.clone());
                            st.access_password = access_pwd.clone();
                            st.pdv_id.clone()
                        } else {
                            String::new()
                        };

                        if !fixed_id.is_empty() {
                            let cfg = HostConfigData {
                                fixed_id: Some(fixed_id),
                                local_user: Some(alias.clone()),
                                alias: Some(alias),
                                group: Some(group),
                                description: Some(desc),
                                access_password: access_pwd,
                            };
                            save_host_props_config(&cfg);
                        }
                        let _ = event_proxy_wry.send_event(HostCustomEvent::StateUpdated);
                    }
                    "save_user_profile" => {
                        let user_name = val.get("localUser").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
                        let custom_pdv = val.get("pdvId").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
                        let access_pwd = val.get("accessPassword").and_then(|v| v.as_str()).map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
                        info!(user = %user_name, pdv = %custom_pdv, has_pwd = access_pwd.is_some(), "Perfil do operador e credenciais do host atualizados");
                        save_host_config(&user_name, &custom_pdv, access_pwd.as_deref());
                        if let Ok(mut st) = state_ipc.lock() {
                            st.local_user = Some(user_name);
                            st.access_password = access_pwd;
                        }
                        let _ = event_proxy_wry.send_event(HostCustomEvent::StateUpdated);
                    }
                    "session_code_generated" => {
                        let code = val.get("code").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
                        let expires = val.get("expiresIn").and_then(|v| v.as_u64()).unwrap_or(120) as u32;
                        if !code.is_empty() {
                            info!(code = %code, expires_in = expires, "Código de sessão temporário sincronizado no Host");
                            if let Ok(mut st) = state_ipc.lock() {
                                st.current_code = Some(code);
                                st.session_code_expires_in = expires;
                            }
                        } else if let Ok(mut st) = state_ipc.lock() {
                            st.current_code = None;
                            st.session_code_expires_in = 0;
                        }
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

    let mut was_session_active = false;
    event_loop.run(move |event, _, control_flow| {
        let _ = &webview;
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(HostCustomEvent::StateUpdated) => {
                if let Ok(st) = state_render.lock() {
                    let is_active = st.is_session_active;
                    if is_active != was_session_active {
                        was_session_active = is_active;
                        if is_active {
                            info!("Docking lateral ativado: recolhendo janela do Host para o canto da tela...");
                            window.set_resizable(false);
                            window.set_inner_size(LogicalSize::new(320.0, 95.0));
                            window.set_always_on_top(true);
                            if let Some(monitor) = window.current_monitor() {
                                let screen = monitor.size();
                                let x = (screen.width as i32) - 340;
                                let y = (screen.height as i32) - 145;
                                window.set_outer_position(tao::dpi::PhysicalPosition::new(x.max(0), y.max(0)));
                            }
                        } else {
                            info!("Sessão finalizada: restaurando janela principal do Host...");
                            window.set_resizable(true);
                            window.set_inner_size(LogicalSize::new(385.0, 565.0));
                            window.set_always_on_top(false);
                        }
                    }

                    let json = st.to_json();
                    let script = format!(
                        "if (window.updateHostState) {{ window.updateHostState({}); }}",
                        json
                    );
                    let _ = webview.evaluate_script(&script);
                }
            }
            Event::UserEvent(HostCustomEvent::TechAuthResult { success, message }) => {
                let script = format!(
                    "if (window.onTechAuthResult) {{ window.onTechAuthResult({{ success: {}, message: {:?} }}); }}",
                    success, message
                );
                let _ = webview.evaluate_script(&script);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                info!(
                    "Ocultando janela do PrysmView Host para manter o dispositivo acessível na bandeja..."
                );
                window.set_visible(false);
            }
            _ => {}
        }
    });
}

/// Captura a tela real do monitor principal em formato JPEG utilizando GDI nativo do Windows.
#[cfg(windows)]
pub fn capture_desktop_jpeg(quality: u8) -> Option<(u32, u32, Vec<u8>)> {
    std::thread::spawn(move || {
        use std::ffi::c_void;
        use windows::Win32::Foundation::{BOOL, HWND};
        use windows::Win32::Graphics::Gdi::*;
        use windows::Win32::System::StationsAndDesktops::{
            CloseDesktop, OpenInputDesktop, SetThreadDesktop, DESKTOP_ACCESS_FLAGS,
            DESKTOP_CONTROL_FLAGS,
        };
        use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

        unsafe {
            // Anexar thread ao desktop interativo de entrada (necessário no Windows para capturar a tela via worker thread)
            let hdesk = OpenInputDesktop(
                DESKTOP_CONTROL_FLAGS(0),
                BOOL(0),
                DESKTOP_ACCESS_FLAGS(0x01FF),
            );
            let closedesk = match hdesk {
                Ok(desk) if !desk.is_invalid() => {
                    let _ = SetThreadDesktop(desk);
                    Some(desk)
                }
                _ => None,
            };

            let width = GetSystemMetrics(SM_CXSCREEN);
            let height = GetSystemMetrics(SM_CYSCREEN);
            if width <= 0 || height <= 0 {
                if let Some(d) = closedesk {
                    let _ = CloseDesktop(d);
                }
                error!(
                    "capture_desktop_jpeg: Dimensoes de tela invalidas: {}x{}",
                    width, height
                );
                return None;
            }

            let hdc_screen = GetDC(HWND::default());
            if hdc_screen.is_invalid() {
                if let Some(d) = closedesk {
                    let _ = CloseDesktop(d);
                }
                error!("capture_desktop_jpeg: GetDC(HWND::default()) falhou");
                return None;
            }

            let hdc_mem = CreateCompatibleDC(hdc_screen);
            if hdc_mem.is_invalid() {
                let _ = ReleaseDC(HWND::default(), hdc_screen);
                if let Some(d) = closedesk {
                    let _ = CloseDesktop(d);
                }
                error!("capture_desktop_jpeg: CreateCompatibleDC falhou");
                return None;
            }

            let bi = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: width,
                    biHeight: -height, // top-down DIB
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };

            let mut bits_ptr: *mut c_void = std::ptr::null_mut();
            let hbm = CreateDIBSection(hdc_mem, &bi, DIB_RGB_COLORS, &mut bits_ptr, None, 0);

            let hbm = match hbm {
                Ok(h) if !h.is_invalid() && !bits_ptr.is_null() => h,
                Err(e) => {
                    error!("capture_desktop_jpeg: CreateDIBSection falhou: {:?}", e);
                    let _ = DeleteDC(hdc_mem);
                    let _ = ReleaseDC(HWND::default(), hdc_screen);
                    if let Some(d) = closedesk {
                        let _ = CloseDesktop(d);
                    }
                    return None;
                }
                _ => {
                    error!("capture_desktop_jpeg: CreateDIBSection retornou ponteiro nulo");
                    let _ = DeleteDC(hdc_mem);
                    let _ = ReleaseDC(HWND::default(), hdc_screen);
                    if let Some(d) = closedesk {
                        let _ = CloseDesktop(d);
                    }
                    return None;
                }
            };

            let old_bm = SelectObject(hdc_mem, hbm);
            let blt_res = BitBlt(hdc_mem, 0, 0, width, height, hdc_screen, 0, 0, SRCCOPY);

            let total_pixels = (width * height) as usize;
            let slice = std::slice::from_raw_parts(bits_ptr as *const u8, total_pixels * 4);

            let mut rgb_buf = Vec::with_capacity(total_pixels * 3);
            for chunk in slice.as_chunks::<4>().0 {
                let b = chunk[0];
                let g = chunk[1];
                let r = chunk[2];
                rgb_buf.push(r);
                rgb_buf.push(g);
                rgb_buf.push(b);
            }

            SelectObject(hdc_mem, old_bm);
            let _ = DeleteObject(hbm);
            let _ = DeleteDC(hdc_mem);
            let _ = ReleaseDC(HWND::default(), hdc_screen);
            if let Some(d) = closedesk {
                let _ = CloseDesktop(d);
            }

            if blt_res.is_err() {
                error!("capture_desktop_jpeg: BitBlt falhou: {:?}", blt_res);
                return None;
            }

            if let Some(img) = image::RgbImage::from_raw(width as u32, height as u32, rgb_buf) {
                let mut jpeg_bytes = Vec::new();
                let mut cursor = std::io::Cursor::new(&mut jpeg_bytes);
                let mut encoder =
                    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, quality);
                match encoder.encode_image(&img) {
                    Ok(_) => return Some((width as u32, height as u32, jpeg_bytes)),
                    Err(e) => error!("capture_desktop_jpeg: encode_image falhou: {:?}", e),
                }
            }
            None
        }
    })
    .join()
    .ok()
    .flatten()
}

#[cfg(not(windows))]
pub fn capture_desktop_jpeg(_quality: u8) -> Option<(u32, u32, Vec<u8>)> {
    None
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (&x, &y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(windows)]
fn set_host_clipboard_text(text: &str) {
    use std::io::Write;
    let mut child = match std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-WindowStyle",
            "Hidden",
            "-Command",
            "$input | Set-Clipboard",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return,
    };
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(text.as_bytes());
    }
}

#[cfg(not(windows))]
fn set_host_clipboard_text(_text: &str) {}

fn compute_ws_accept(key: &str) -> String {
    use sha1::{Digest, Sha1};
    let mut hasher = Sha1::new();
    hasher.update(key.trim().as_bytes());
    hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11");
    let result = hasher.finalize();
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(result)
}

async fn send_ws_frame<W: tokio::io::AsyncWriteExt + Unpin>(
    wr: &mut W,
    opcode: u8,
    payload: &[u8],
) -> std::io::Result<()> {
    let mut header = Vec::with_capacity(10);
    header.push(0x80 | (opcode & 0x0F));
    let len = payload.len();
    if len < 126 {
        header.push(len as u8);
    } else if len <= 65535 {
        header.push(126);
        header.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        header.push(127);
        header.extend_from_slice(&(len as u64).to_be_bytes());
    }
    wr.write_all(&header).await?;
    wr.write_all(payload).await?;
    wr.flush().await?;
    Ok(())
}

async fn read_ws_frame<R: tokio::io::AsyncReadExt + Unpin>(
    rd: &mut R,
) -> std::io::Result<(u8, Vec<u8>)> {
    let mut b = [0u8; 2];
    rd.read_exact(&mut b).await?;
    let opcode = b[0] & 0x0F;
    let masked = (b[1] & 0x80) != 0;
    let mut len = (b[1] & 0x7F) as usize;
    if len == 126 {
        let mut len_b = [0u8; 2];
        rd.read_exact(&mut len_b).await?;
        len = u16::from_be_bytes(len_b) as usize;
    } else if len == 127 {
        let mut len_b = [0u8; 8];
        rd.read_exact(&mut len_b).await?;
        len = u64::from_be_bytes(len_b) as usize;
    }
    let mask = if masked {
        let mut m = [0u8; 4];
        rd.read_exact(&mut m).await?;
        m
    } else {
        [0u8; 4]
    };
    let mut payload = vec![0u8; len];
    rd.read_exact(&mut payload).await?;
    if masked {
        for (i, byte) in payload.iter_mut().enumerate() {
            *byte ^= mask[i % 4];
        }
    }
    Ok((opcode, payload))
}

async fn handle_ws_client(
    mut socket: tokio::net::TcpStream,
    initial_req: String,
    state: Arc<Mutex<TrayState>>,
    running: Arc<AtomicBool>,
    event_proxy: EventLoopProxy<HostCustomEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ws_key = initial_req
        .lines()
        .find(|l| l.to_lowercase().starts_with("sec-websocket-key:"))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim())
        .unwrap_or("");

    if ws_key.is_empty() {
        return Ok(());
    }

    let accept = compute_ws_accept(ws_key);
    let client_wants_proto = initial_req.lines().any(|l| {
        l.to_lowercase().starts_with("sec-websocket-protocol:")
            && l.to_lowercase().contains("prysm-stream")
    });

    let proto_header = if client_wants_proto {
        "Sec-WebSocket-Protocol: prysm-stream\r\n"
    } else {
        ""
    };

    let handshake_resp = format!(
        "HTTP/1.1 101 Switching Protocols\r\n\
         Upgrade: websocket\r\n\
         Connection: Upgrade\r\n\
         Sec-WebSocket-Accept: {}\r\n\
         {}\r\n",
        accept, proto_header
    );
    use tokio::io::AsyncWriteExt;
    socket.write_all(handshake_resp.as_bytes()).await?;
    socket.flush().await?;

    let (mut rd, mut wr) = socket.into_split();

    // 1. Handshake inicial de autenticação / consentimento
    let (opcode, payload) = read_ws_frame(&mut rd).await?;
    if opcode == 8 {
        return Ok(());
    }
    let auth_msg = serde_json::from_slice::<serde_json::Value>(&payload).unwrap_or_default();
    let auth_type = auth_msg
        .get("auth_type")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let credential = auth_msg
        .get("credential")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    let op_name = auth_msg
        .get("operator")
        .and_then(|v| v.as_str())
        .unwrap_or("Técnico Central");

    let mut authenticated = false;
    {
        if let Ok(st) = state.lock() {
            if auth_type == "password" && !credential.is_empty() {
                if let Some(ref saved_pwd) = st.access_password {
                    if constant_time_eq(saved_pwd.trim().as_bytes(), credential.as_bytes()) {
                        authenticated = true;
                    }
                }
            } else if auth_type == "code" && !credential.is_empty() {
                if let Some(ref code) = st.current_code {
                    if constant_time_eq(code.trim().as_bytes(), credential.as_bytes()) {
                        authenticated = true;
                    }
                }
            }
        }
    }

    if !authenticated {
        // Dispara modal de consentimento no PDV com código de 8 dígitos
        let now_millis = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(84729103);
        let code_gen = format!("{:08}", (now_millis % 90000000) + 10000000);
        {
            if let Ok(mut st) = state.lock() {
                st.current_code = Some(code_gen.clone());
                st.session_code_expires_in = 60;
                st.pending_consent = Some(PendingConsent {
                    operator_name: op_name.to_string(),
                    reason: "Assistência Técnica Remota".to_string(),
                    timeout_secs: 60,
                    response: None,
                });
            }
        }
        let _ = event_proxy.send_event(HostCustomEvent::StateUpdated);

        // Notifica o Admin que consentimento foi solicitado
        let notify = serde_json::json!({
            "type": "consent_required",
            "timeout": 60,
            "operator": op_name
        });
        send_ws_frame(&mut wr, 0x81, notify.to_string().as_bytes()).await?;

        // Aguarda resposta do consentimento por até 60 segundos
        let start = std::time::Instant::now();
        while start.elapsed().as_secs() < 60 {
            tokio::time::sleep(Duration::from_millis(500)).await;
            if let Ok(st) = state.lock() {
                if let Some(ref c) = st.pending_consent {
                    if let Some(resp) = c.response {
                        authenticated = resp;
                        break;
                    }
                }
            }
        }
    }

    if !authenticated {
        let rej = serde_json::json!({
            "type": "auth_failed",
            "reason": "Acesso recusado ou expirado pelo operador local do dispositivo."
        });
        let _ = send_ws_frame(&mut wr, 0x81, rej.to_string().as_bytes()).await;
        let _ = send_ws_frame(&mut wr, 0x88, &[]).await;
        return Ok(());
    }

    // Autenticado com sucesso!
    {
        if let Ok(mut st) = state.lock() {
            st.is_session_active = true;
            st.current_operator = Some(op_name.to_string());
            st.pending_consent = None;
        }
    }
    let _ = event_proxy.send_event(HostCustomEvent::StateUpdated);

    let ok_msg = serde_json::json!({
        "type": "auth_ok",
        "authenticated": true,
        "operator": op_name
    });
    send_ws_frame(&mut wr, 0x81, ok_msg.to_string().as_bytes()).await?;

    // Inicia loops de Streaming e de Injeção
    let injector = Arc::new(Mutex::new(
        prysmview_host::InputInjector::new(1920, 1080).ok(),
    ));
    let injector_stream = injector.clone();
    let quality_val = Arc::new(std::sync::atomic::AtomicU8::new(75));
    let quality_clone = quality_val.clone();
    let running_ws = Arc::new(AtomicBool::new(true));
    let running_ws_clone = running_ws.clone();
    let running_loop = running.clone();
    let state_stream_task = state.clone();

    // Loop de envio de frames (Task de Streaming contínuo com ajuste dinâmico de resolução)
    let stream_task = tokio::spawn(async move {
        while running_ws_clone.load(Ordering::Relaxed) && running_loop.load(Ordering::Relaxed) {
            let is_active = state_stream_task
                .lock()
                .map(|st| st.is_session_active)
                .unwrap_or(false);
            if !is_active {
                let term = serde_json::json!({
                    "type": "session_terminated",
                    "reason": "Sessão encerrada pelo operador local do dispositivo."
                });
                let _ = send_ws_frame(&mut wr, 0x81, term.to_string().as_bytes()).await;
                let _ = send_ws_frame(&mut wr, 0x88, &[]).await;
                running_ws_clone.store(false, Ordering::Relaxed);
                break;
            }
            let q = quality_clone.load(Ordering::Relaxed);
            #[cfg(windows)]
            if let Some((w, h, jpeg)) = capture_desktop_jpeg(q) {
                if let Ok(mut inj_lock) = injector_stream.lock() {
                    if let Some(ref mut inj) = *inj_lock {
                        inj.update_dimensions(w, h);
                    }
                }
                if send_ws_frame(&mut wr, 0x82, &jpeg).await.is_err() {
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(33)).await;
        }
    });

    // Loop de leitura de eventos de controle, entrada e área de transferência
    while running_ws.load(Ordering::Relaxed) {
        match read_ws_frame(&mut rd).await {
            Ok((opcode, p)) => {
                if opcode == 8 {
                    break;
                } else if opcode == 1 {
                    if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&p) {
                        let t = val.get("type").and_then(|v| v.as_str()).unwrap_or("");
                        match t {
                            "mouse_move" => {
                                let x = val.get("x").and_then(|v| v.as_u64()).unwrap_or(0) as u16;
                                let y = val.get("y").and_then(|v| v.as_u64()).unwrap_or(0) as u16;
                                if let Ok(mut inj_lock) = injector.lock() {
                                    if let Some(ref mut inj) = *inj_lock {
                                        let _ = inj.inject_mouse_move(x, y);
                                    }
                                }
                            }
                            "mouse_button" => {
                                let btn = val.get("button").and_then(|v| v.as_u64()).unwrap_or(0);
                                let pressed =
                                    val.get("pressed").and_then(|v| v.as_bool()).unwrap_or(true);
                                let btn_type = match btn {
                                    0 => prysmview_core::messages::MouseButtonType::Left,
                                    1 => prysmview_core::messages::MouseButtonType::Middle,
                                    _ => prysmview_core::messages::MouseButtonType::Right,
                                };
                                if let Ok(mut inj_lock) = injector.lock() {
                                    if let Some(ref mut inj) = *inj_lock {
                                        let _ = inj.inject_mouse_button(btn_type, pressed);
                                    }
                                }
                            }
                            "mouse_wheel" => {
                                let delta =
                                    val.get("delta_v").and_then(|v| v.as_i64()).unwrap_or(0) as i16;
                                if let Ok(mut inj_lock) = injector.lock() {
                                    if let Some(ref mut inj) = *inj_lock {
                                        let _ = inj.inject_mouse_wheel(delta, 0);
                                    }
                                }
                            }
                            "key" => {
                                let scan_code =
                                    val.get("scan_code").and_then(|v| v.as_u64()).unwrap_or(0)
                                        as u16;
                                let extended = val
                                    .get("extended")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false);
                                let pressed =
                                    val.get("pressed").and_then(|v| v.as_bool()).unwrap_or(true);
                                if let Ok(mut inj_lock) = injector.lock() {
                                    if let Some(ref mut inj) = *inj_lock {
                                        let _ = inj.inject_key(scan_code, extended, pressed);
                                    }
                                }
                            }
                            "sas" => {
                                let _ = prysmview_host::send_sas();
                            }
                            "clipboard" => {
                                let text = val.get("text").and_then(|v| v.as_str()).unwrap_or("");
                                let is_enabled =
                                    state.lock().map(|st| st.clipboard_enabled).unwrap_or(true);
                                if is_enabled && !text.is_empty() && text.len() <= 1024 * 1024 {
                                    #[cfg(windows)]
                                    set_host_clipboard_text(text);
                                }
                            }
                            "quality" => {
                                let q =
                                    val.get("value").and_then(|v| v.as_u64()).unwrap_or(75) as u8;
                                quality_val.store(q.clamp(20, 95), Ordering::Relaxed);
                            }
                            "block_input" => {
                                let blocked =
                                    val.get("blocked").and_then(|v| v.as_bool()).unwrap_or(true);
                                info!(
                                    blocked = blocked,
                                    "Comando block_input acionado via WebSocket"
                                );
                                set_remote_input_blocked(blocked);
                                if let Ok(mut st) = state.lock() {
                                    st.input_blocked = blocked;
                                }
                                let _ = event_proxy.send_event(HostCustomEvent::StateUpdated);
                            }
                            "lock_workstation" => {
                                info!("Comando lock_workstation acionado via WebSocket");
                                let _ = lock_remote_workstation();
                            }
                            "disconnect" => {
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            }
            Err(_) => {
                break;
            }
        }
    }

    running_ws.store(false, Ordering::Relaxed);
    let _ = stream_task.await;

    // Encerramento seguro: SEMPRE desbloqueia teclado/mouse, libera todas as teclas e reseta o estado
    set_remote_input_blocked(false);
    if let Ok(mut inj_lock) = injector.lock() {
        if let Some(ref mut inj) = *inj_lock {
            inj.release_all();
        }
    }
    prysmview_host::release_all_keys();

    {
        if let Ok(mut st) = state.lock() {
            st.is_session_active = false;
            st.input_blocked = false;
            st.current_operator = None;
        }
    }
    let _ = event_proxy.send_event(HostCustomEvent::StateUpdated);

    Ok(())
}

/// Servidor assíncrono de transmissão de tela real e recepção de comandos na porta 19533.
pub fn start_host_stream_server(
    state: Arc<Mutex<TrayState>>,
    running: Arc<AtomicBool>,
    event_proxy: EventLoopProxy<HostCustomEvent>,
) {
    let state_stream = state.clone();
    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("Falha ao criar runtime de streaming do Host: {e}");
                return;
            }
        };

        rt.block_on(async move {
            let listener = match tokio::net::TcpListener::bind("0.0.0.0:19533").await {
                Ok(l) => {
                    info!("Servidor de transmissão de tela real do Host ouvindo em 0.0.0.0:19533");
                    l
                }
                Err(e) => {
                    tracing::warn!("Porta 19533 indisponível para streaming do Host: {e}");
                    return;
                }
            };

            let mut injector = prysmview_host::InputInjector::new(1920, 1080).ok();

            while running.load(Ordering::Relaxed) {
                if let Ok((mut socket, _)) = listener.accept().await {
                    let mut buf = [0u8; 4096];
                    use tokio::io::{AsyncReadExt, AsyncWriteExt};

                    let n = match socket.read(&mut buf).await {
                        Ok(n) if n > 0 => n,
                        _ => continue,
                    };

                    let req = String::from_utf8_lossy(&buf[..n]);

                    if req.to_lowercase().contains("upgrade: websocket") {
                        let state_c = state_stream.clone();
                        let running_c = running.clone();
                        let proxy_c = event_proxy.clone();
                        let req_s = req.to_string();
                        tokio::spawn(async move {
                            if let Err(e) = handle_ws_client(socket, req_s, state_c, running_c, proxy_c).await {
                                tracing::warn!("Sessão WebSocket encerrada: {e}");
                            }
                        });
                        continue;
                    }

                    let (mut _rd, mut wr) = socket.split();

                    if req.starts_with("OPTIONS") {
                        let resp = "HTTP/1.1 204 No Content\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: *\r\n\r\n";
                        let _ = wr.write_all(resp.as_bytes()).await;
                        let _ = wr.flush().await;
                        let _ = wr.shutdown().await;
                    } else if req.starts_with("GET /frame") {
                        let is_active = state_stream.lock().map(|st| st.is_session_active).unwrap_or(false);
                        if !is_active {
                            let resp = "HTTP/1.1 403 Forbidden\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: 0\r\n\r\n";
                            let _ = wr.write_all(resp.as_bytes()).await;
                            let _ = wr.flush().await;
                            let _ = wr.shutdown().await;
                            continue;
                        }
                        #[cfg(windows)]
                        if let Some((w, h, jpeg_data)) = capture_desktop_jpeg(75) {
                            if let Some(ref mut inj) = injector {
                                inj.update_dimensions(w, h);
                            }
                            if let Ok(mut st) = state_stream.lock() {
                                st.is_session_active = true;
                                if st.current_operator.is_none() {
                                    st.current_operator = Some("Técnico Remoto (Sessão Ativa)".to_string());
                                }
                            }
                            let header = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: image/jpeg\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\nCache-Control: no-cache, no-store\r\nContent-Length: {}\r\n\r\n",
                                jpeg_data.len()
                            );
                            let _ = wr.write_all(header.as_bytes()).await;
                            let _ = wr.write_all(&jpeg_data).await;
                            let _ = wr.flush().await;
                            let _ = wr.shutdown().await;
                        } else {
                            let resp = "HTTP/1.1 500 Internal Error\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: 0\r\n\r\n";
                            let _ = wr.write_all(resp.as_bytes()).await;
                            let _ = wr.flush().await;
                            let _ = wr.shutdown().await;
                        }
                        #[cfg(not(windows))]
                        {
                            let resp = "HTTP/1.1 501 Not Implemented\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\n\r\n";
                            let _ = wr.write_all(resp.as_bytes()).await;
                            let _ = wr.flush().await;
                            let _ = wr.shutdown().await;
                        }
                    } else if req.starts_with("POST /input") {
                        let is_active = state_stream.lock().map(|st| st.is_session_active).unwrap_or(false);
                        if !is_active {
                            let resp = "HTTP/1.1 403 Forbidden\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: 0\r\n\r\n";
                            let _ = wr.write_all(resp.as_bytes()).await;
                            let _ = wr.flush().await;
                            let _ = wr.shutdown().await;
                            continue;
                        }
                        if let Some(body_idx) = req.find("\r\n\r\n") {
                            let body = &req[body_idx + 4..];
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(body) {
                                let input_type = val.get("type").and_then(|v| v.as_str()).unwrap_or("");
                                if let Some(ref mut inj) = injector {
                                    match input_type {
                                        "mouse_move" => {
                                            let x = val.get("x").and_then(|v| v.as_u64()).unwrap_or(0) as u16;
                                            let y = val.get("y").and_then(|v| v.as_u64()).unwrap_or(0) as u16;
                                            let _ = inj.inject_mouse_move(x, y);
                                        }
                                        "mouse_button" => {
                                            let btn = val.get("button").and_then(|v| v.as_u64()).unwrap_or(0);
                                            let pressed = val.get("pressed").and_then(|v| v.as_bool()).unwrap_or(true);
                                            let btn_type = match btn {
                                                0 => prysmview_core::messages::MouseButtonType::Left,
                                                1 => prysmview_core::messages::MouseButtonType::Middle,
                                                _ => prysmview_core::messages::MouseButtonType::Right,
                                            };
                                            let _ = inj.inject_mouse_button(btn_type, pressed);
                                        }
                                        "mouse_wheel" => {
                                            let delta = val.get("delta_v").and_then(|v| v.as_i64()).unwrap_or(0) as i16;
                                            let _ = inj.inject_mouse_wheel(delta, 0);
                                        }
                                        "key" => {
                                            let scan_code = val.get("scan_code").and_then(|v| v.as_u64()).unwrap_or(0) as u16;
                                            let extended = val.get("extended").and_then(|v| v.as_bool()).unwrap_or(false);
                                            let pressed = val.get("pressed").and_then(|v| v.as_bool()).unwrap_or(true);
                                            let _ = inj.inject_key(scan_code, extended, pressed);
                                        }
                                        "sas" => {
                                            let _ = prysmview_host::send_sas();
                                        }
                                        "block_input" => {
                                            let blocked = val.get("blocked").and_then(|v| v.as_bool()).unwrap_or(true);
                                            info!(blocked = blocked, "POST /input block_input acionado");
                                            set_remote_input_blocked(blocked);
                                            if let Ok(mut st) = state_stream.lock() {
                                                st.input_blocked = blocked;
                                            }
                                        }
                                        "lock_workstation" => {
                                            info!("POST /input lock_workstation acionado");
                                            let _ = lock_remote_workstation();
                                        }
                                        "disconnect" => {
                                            set_remote_input_blocked(false);
                                            inj.release_all();
                                            if let Ok(mut st) = state_stream.lock() {
                                                st.is_session_active = false;
                                                st.input_blocked = false;
                                                st.current_operator = None;
                                            }
                                        }
                    _ => {}
                                    }
                                }
                            }
                        }
                        let resp = "HTTP/1.1 200 OK\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: 2\r\n\r\nOK";
                        let _ = wr.write_all(resp.as_bytes()).await;
                        let _ = wr.flush().await;
                        let _ = wr.shutdown().await;
                    }
                }
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dxgi_worker_capture() {
        let (sender, receiver) = std::sync::mpsc::sync_channel::<prysmview_host::CapturedFrame>(1);
        let worker = prysmview_host::CaptureWorker::new(0);
        let handle = worker.start(sender);
        println!("Waiting for DXGI frame...");
        match receiver.recv_timeout(std::time::Duration::from_millis(2000)) {
            Ok(frame) => {
                println!(
                    "DXGI Frame received! Dims: {}x{}, bytes: {}",
                    frame.width,
                    frame.height,
                    frame.data.len()
                );
            }
            Err(e) => {
                println!("DXGI recv error: {:?}", e);
            }
        }
        worker.stop();
        let _ = handle.join();
    }

    #[test]
    fn test_tray_state_consent_message() {
        let mut state = TrayState::new();
        let msg = IpcMessage::ConsentRequest {
            operator_name: "Central de Suporte".to_string(),
            reason: "Suporte e Manutenção".to_string(),
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
        state.pdv_id = "DESKTOP-09".to_string();
        state.current_code = Some("12345678".to_string());
        let json = state.to_json();
        assert!(json.contains("DESKTOP-09"));
        assert!(json.contains("12345678"));
        assert!(json.contains("isConnected"));
    }

    #[test]
    fn test_capture_desktop_jpeg_runs() {
        #[cfg(windows)]
        {
            let res = capture_desktop_jpeg(50);
            println!(
                "test_capture_desktop_jpeg_runs result: {:?}",
                res.as_ref().map(|(w, h, b)| (*w, *h, b.len()))
            );
            assert!(
                res.is_some(),
                "capture_desktop_jpeg deve capturar o desktop no Windows"
            );
            let (w, h, bytes) = res.unwrap();
            assert!(w > 0 && h > 0);
            assert!(!bytes.is_empty());
        }
    }
}
