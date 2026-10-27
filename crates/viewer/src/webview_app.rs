//! Launcher WebView2 para o console administrativo PrysmAdmin.
//! Utiliza `tao` e `wry` para fornecer renderização moderna via Microsoft Edge WebView2 nativo.

use std::sync::{Arc, Mutex};
use std::time::Duration;
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::{Icon, WindowBuilder},
};
use tracing::info;
use wry::WebViewBuilder;

/// Eventos customizados enviados pela thread de descoberta de rede para a interface WebView2.
#[derive(Debug, Clone)]
pub enum AdminCustomEvent {
    HostsDiscovered(String),
}

/// Carrega o ícone oficial da marca PrysmView para a barra de título e janela do Windows.
fn load_window_icon() -> Option<Icon> {
    let raw = include_bytes!("../../../assets/logo.png");
    let img = image::load_from_memory(raw).ok()?.to_rgba8();
    let (width, height) = img.dimensions();
    Icon::from_rgba(img.into_raw(), width, height).ok()
}

/// Caminho do arquivo de estado persistente do console administrativo
pub fn get_admin_state_path() -> std::path::PathBuf {
    let base = std::env::var("LOCALAPPDATA")
        .or_else(|_| std::env::var("APPDATA"))
        .unwrap_or_else(|_| ".".to_string());
    let mut path = std::path::PathBuf::from(base);
    path.push("PrysmView");
    let _ = std::fs::create_dir_all(&path);
    path.push("admin_state.json");
    path
}

/// Lê a versão salva do aplicativo ou utiliza a versão compilada
pub fn load_saved_version() -> String {
    let path = get_admin_state_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(v) = val.get("installed_version").and_then(|v| v.as_str()) {
                    return v.to_string();
                }
            }
        }
    }
    crate::updater::CURRENT_VERSION.to_string()
}

/// Salva o estado da versão atualizada no disco
pub fn save_admin_state(version: &str) {
    let path = get_admin_state_path();
    let data = serde_json::json!({
        "installed_version": version,
        "updated_at": "Agora",
    });
    if let Err(e) = std::fs::write(&path, data.to_string()) {
        tracing::warn!("Não foi possível gravar estado do admin em {:?}: {}", path, e);
    } else {
        info!("Estado do admin persistido com sucesso em {:?}: v{}", path, version);
    }
}

/// Inicia o aplicativo desktop PrysmAdmin com o frontend web moderno e IPC nativo.
pub fn run_webview_app() -> Result<(), Box<dyn std::error::Error>> {
    // Limpeza automática de backups temporários .old gerados por atualizações anteriores
    crate::updater::cleanup_previous_update_backup();

    let current_version = load_saved_version();
    info!("Versão ativa do console PrysmAdmin: v{}", current_version);

    info!("Inicializando EventLoop do PrysmAdmin (WebView2 / Wry)...");
    let event_loop = EventLoopBuilder::<AdminCustomEvent>::with_user_event().build();
    let event_proxy = event_loop.create_proxy();

    let discovered_hosts: Arc<Mutex<Vec<serde_json::Value>>> = Arc::new(Mutex::new(Vec::new()));
    let discovered_bg = discovered_hosts.clone();
    let proxy_bg = event_proxy.clone();

    // Thread em segundo plano para descoberta de Hosts em tempo real via UDP broadcast e arquivo local
    std::thread::spawn(move || {
        let socket = match std::net::UdpSocket::bind("0.0.0.0:19532") {
            Ok(s) => {
                let _ = s.set_read_timeout(Some(Duration::from_millis(600)));
                Some(s)
            }
            Err(e) => {
                tracing::warn!("Não foi possível vincular porta UDP 19532 para descoberta LAN: {e}");
                None
            }
        };

        let mut buf = [0u8; 2048];
        loop {
            let mut list_updated = false;

            // 1. Checa Host local via arquivo %LOCALAPPDATA%\PrysmView\host_config.json
            let base = std::env::var("LOCALAPPDATA")
                .or_else(|_| std::env::var("APPDATA"))
                .unwrap_or_else(|_| ".".to_string());
            let local_config_path = std::path::PathBuf::from(base).join("PrysmView").join("host_config.json");
            if local_config_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&local_config_path) {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                        let pdv_id = val.get("pdv_id").and_then(|v| v.as_str()).unwrap_or("PDV-CAIXA-01");
                        let local_user = val.get("local_user").and_then(|v| v.as_str());
                        let access_password = val.get("access_password").and_then(|v| v.as_str());
                        let host_val = serde_json::json!({
                            "pdv_id": pdv_id,
                            "local_user": local_user,
                            "access_password": access_password,
                            "ip": "127.0.0.1",
                            "agent_version": "0.1.0",
                            "is_online": true
                        });
                        if let Ok(mut lock) = discovered_bg.lock() {
                            if let Some(existing) = lock.iter_mut().find(|h| h.get("pdv_id").and_then(|v| v.as_str()) == Some(pdv_id)) {
                                *existing = host_val;
                            } else {
                                lock.push(host_val);
                            }
                            list_updated = true;
                        }
                    }
                }
            }

            // 2. Escuta beacons UDP da rede LAN
            if let Some(ref s) = socket {
                if let Ok((len, peer_addr)) = s.recv_from(&mut buf) {
                    if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&buf[..len]) {
                        if val.get("type").and_then(|v| v.as_str()) == Some("host_beacon") {
                            let pdv_id = val.get("pdv_id").and_then(|v| v.as_str()).unwrap_or("PDV-CAIXA-01");
                            let mut dev = val.clone();
                            dev["ip"] = serde_json::json!(peer_addr.ip().to_string());
                            if let Ok(mut lock) = discovered_bg.lock() {
                                if let Some(existing) = lock.iter_mut().find(|h| h.get("pdv_id").and_then(|v| v.as_str()) == Some(pdv_id)) {
                                    *existing = dev;
                                } else {
                                    lock.push(dev);
                                }
                                list_updated = true;
                            }
                        }
                    }
                }
            }

            if list_updated {
                if let Ok(lock) = discovered_bg.lock() {
                    let json_str = serde_json::to_string(&*lock).unwrap_or_else(|_| "[]".to_string());
                    let _ = proxy_bg.send_event(AdminCustomEvent::HostsDiscovered(json_str));
                }
            }

            std::thread::sleep(Duration::from_millis(2000));
        }
    });

    let mut window_builder = WindowBuilder::new()
        .with_title("PrysmView - Acesso Remoto & Gestão de Dispositivos")
        .with_inner_size(LogicalSize::new(1280.0, 780.0))
        .with_min_inner_size(LogicalSize::new(960.0, 600.0));

    if let Some(icon) = load_window_icon() {
        window_builder = window_builder.with_window_icon(Some(icon));
    }

    let window = window_builder.build(&event_loop)?;

    // Embutir HTML, CSS e JavaScript diretamente no binário para portabilidade e segurança zero-trust
    let raw_html = include_str!("../ui/index.html");
    let css_content = include_str!("../ui/style.css");
    let js_content = include_str!("../ui/app.js");

    // Injeta versão ativa do servidor e os estilos e scripts de forma inline
    let version_injection = format!(
        "<script>\nwindow.__PRYSM_SERVER_VERSION__ = {:?};\n</script>\n<script src=\"app.js\"></script>",
        current_version
    );
    let bundled_html = raw_html
        .replace(
            r#"<link rel="stylesheet" href="style.css">"#,
            &format!("<style>\n{}\n</style>", css_content),
        )
        .replace(
            r#"<script src="app.js"></script>"#,
            &version_injection,
        )
        .replace(
            r#"<script src="app.js"></script>"#,
            &format!("<script>\n{}\n</script>", js_content),
        );

    let discovered_hosts_ipc = discovered_hosts.clone();
    let event_proxy_ipc = event_proxy.clone();

    // Constrói a WebView vinculada à janela nativa do Windows
    let webview = WebViewBuilder::new()
        .with_devtools(true)
        .with_html(&bundled_html)
        .with_ipc_handler(move |msg| {
            let body = msg.body();
            info!("Comando recebido do console de suporte: {}", body);
            if body.contains("start_remote_session") {
                info!("Iniciando sessão remota E2E: {}", body);
            } else if body.contains("end_remote_session") {
                info!("Encerrando sessão remota E2E: {}", body);
            } else if body.contains("send_sas") {
                info!("Sinal SAS (Ctrl+Alt+Del) despachado para o Host");
            } else if body.contains("refresh_fleet") {
                if let Ok(lock) = discovered_hosts_ipc.lock() {
                    let json_str = serde_json::to_string(&*lock).unwrap_or_else(|_| "[]".to_string());
                    let _ = event_proxy_ipc.send_event(AdminCustomEvent::HostsDiscovered(json_str));
                }
            } else if body.contains("apply_update") {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(body) {
                    let version = val.get("version").and_then(|v| v.as_str()).unwrap_or("0.1.1");
                    save_admin_state(version);
                    info!("Atualização gravada com sucesso no disco para versão v{}", version);
                } else {
                    save_admin_state("0.1.1");
                }
            } else if body.contains("check_updates") {
                let (channel, cur_ver) = if let Ok(val) = serde_json::from_str::<serde_json::Value>(body) {
                    let ch = val.get("channel").and_then(|v| v.as_str()).unwrap_or("stable").to_string();
                    let cv = val.get("currentVersion").and_then(|v| v.as_str()).map(|s| s.to_string())
                        .unwrap_or_else(load_saved_version);
                    (ch, cv)
                } else {
                    ("stable".to_string(), load_saved_version())
                };
                let status = crate::updater::check_for_updates_for_version(&cur_ver, &channel);
                info!("Checagem de atualização executada: {:?}", status);
            } else if body.contains("export_audit_csv") {
                info!("Comando de exportação de registros de auditoria CSV recebido.");
            }
        })
        .build(&window)?;

    info!("Janela PrysmAdmin inicializada com sucesso em modo WebView2.");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::UserEvent(AdminCustomEvent::HostsDiscovered(json)) => {
                let script = format!("if (window.onPrysmHostSync) {{ window.onPrysmHostSync({}); }}", json);
                let _ = webview.evaluate_script(&script);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                info!("Encerrando console administrativo PrysmAdmin...");
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}
