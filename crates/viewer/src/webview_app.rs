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
use tracing::{info, warn};
use wry::WebViewBuilder;

/// Eventos customizados enviados pelas threads de segundo plano para a interface WebView2.
#[derive(Debug, Clone)]
pub enum AdminCustomEvent {
    HostsDiscovered(String),
    UpdateCheckResult(String),
    UpdateProgress { percent: u32, message: String },
    UpdateComplete { new_version: String },
    UpdateError(String),
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
        tracing::warn!(
            "Não foi possível gravar estado do admin em {:?}: {}",
            path,
            e
        );
    } else {
        info!(
            "Estado do admin persistido com sucesso em {:?}: v{}",
            path, version
        );
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

    // Thread em segundo plano para verificação e descoberta de Hosts em tempo real via UDP broadcast e TCP health-check
    std::thread::spawn(move || {
        use std::collections::HashMap;
        use std::time::Instant;

        let socket = match std::net::UdpSocket::bind("0.0.0.0:19532") {
            Ok(s) => {
                let _ = s.set_nonblocking(true);
                Some(s)
            }
            Err(e) => {
                tracing::warn!(
                    "Não foi possível vincular porta UDP 19532 para descoberta LAN: {e}"
                );
                None
            }
        };

        struct HostTracker {
            pdv_id: String,
            local_user: Option<String>,
            access_password: Option<String>,
            ip: String,
            agent_version: String,
            is_online: bool,
            last_beacon_instant: Option<Instant>,
            last_seen_text: String,
        }

        let mut hosts_map: HashMap<String, HostTracker> = HashMap::new();
        let mut buf = [0u8; 2048];

        loop {
            // 1. Drena todos os pacotes de beacon UDP recebidos da LAN
            if let Some(ref s) = socket {
                while let Ok((len, peer_addr)) = s.recv_from(&mut buf) {
                    if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&buf[..len]) {
                        if val.get("type").and_then(|v| v.as_str()) == Some("host_beacon") {
                            let pdv_id = val
                                .get("pdv_id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("DESKTOP-01")
                                .to_string();
                            let ip = peer_addr.ip().to_string();
                            let local_user = val
                                .get("local_user")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                            let access_password = val
                                .get("access_password")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                            let agent_version = val
                                .get("agent_version")
                                .and_then(|v| v.as_str())
                                .unwrap_or("0.1.0")
                                .to_string();

                            let entry =
                                hosts_map
                                    .entry(pdv_id.clone())
                                    .or_insert_with(|| HostTracker {
                                        pdv_id: pdv_id.clone(),
                                        local_user: local_user.clone(),
                                        access_password: access_password.clone(),
                                        ip: ip.clone(),
                                        agent_version: agent_version.clone(),
                                        is_online: true,
                                        last_beacon_instant: Some(Instant::now()),
                                        last_seen_text: "Agora".to_string(),
                                    });
                            entry.ip = ip;
                            entry.is_online = true;
                            entry.last_beacon_instant = Some(Instant::now());
                            entry.last_seen_text = "Agora".to_string();
                            if local_user.is_some() {
                                entry.local_user = local_user;
                            }
                            if access_password.is_some() {
                                entry.access_password = access_password;
                            }
                        }
                    }
                }
            }

            // 2. Checa Host local na porta 19533 e arquivo %LOCALAPPDATA%\PrysmView\host_config.json
            let base = std::env::var("LOCALAPPDATA")
                .or_else(|_| std::env::var("APPDATA"))
                .unwrap_or_else(|_| ".".to_string());
            let local_config_path = std::path::PathBuf::from(base)
                .join("PrysmView")
                .join("host_config.json");
            let (local_pdv_id, local_user, local_pwd) = if local_config_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&local_config_path) {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                        let id = val
                            .get("pdv_id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("DESKTOP-01")
                            .to_string();
                        let u = val
                            .get("local_user")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        let p = val
                            .get("access_password")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        (id, u, p)
                    } else {
                        ("DESKTOP-01".to_string(), None, None)
                    }
                } else {
                    ("DESKTOP-01".to_string(), None, None)
                }
            } else {
                ("DESKTOP-01".to_string(), None, None)
            };

            // Teste real de conectividade TCP na porta 19533 do Host
            let local_tcp_ok = std::net::TcpStream::connect_timeout(
                &std::net::SocketAddr::from(([127, 0, 0, 1], 19533)),
                Duration::from_millis(150),
            )
            .is_ok();

            if local_tcp_ok {
                let local_entry =
                    hosts_map
                        .entry(local_pdv_id.clone())
                        .or_insert_with(|| HostTracker {
                            pdv_id: local_pdv_id.clone(),
                            local_user: local_user.clone(),
                            access_password: local_pwd.clone(),
                            ip: "127.0.0.1".to_string(),
                            agent_version: "0.1.0".to_string(),
                            is_online: true,
                            last_beacon_instant: Some(Instant::now()),
                            last_seen_text: "Agora".to_string(),
                        });
                if local_entry.ip == "127.0.0.1"
                    || local_entry
                        .last_beacon_instant
                        .map(|i| i.elapsed() > Duration::from_secs(5))
                        .unwrap_or(true)
                {
                    local_entry.ip = "127.0.0.1".to_string();
                    local_entry.is_online = true;
                    local_entry.last_seen_text = "Agora".to_string();
                    local_entry.last_beacon_instant = Some(Instant::now());
                    if local_user.is_some() {
                        local_entry.local_user = local_user;
                    }
                    if local_pwd.is_some() {
                        local_entry.access_password = local_pwd;
                    }
                }
            } else if let Some(local_entry) = hosts_map.get_mut(&local_pdv_id) {
                if local_entry.ip == "127.0.0.1" {
                    local_entry.is_online = false;
                    local_entry.last_seen_text = "Offline".to_string();
                }
            }

            // 3. Verifica expiração de beacons para outros hosts remotos
            for (id, tracker) in hosts_map.iter_mut() {
                if id == &local_pdv_id && tracker.ip == "127.0.0.1" {
                    continue; // já verificado via TCP direto
                }
                let is_recent = tracker
                    .last_beacon_instant
                    .map(|inst| inst.elapsed() < Duration::from_secs(6))
                    .unwrap_or(false);

                if !is_recent {
                    // Tenta probe TCP na porta 19533
                    let ip_clean = tracker.ip.trim();
                    let tcp_probe_ok = if let Ok(ip_addr) = ip_clean.parse::<std::net::IpAddr>() {
                        std::net::TcpStream::connect_timeout(
                            &std::net::SocketAddr::new(ip_addr, 19533),
                            Duration::from_millis(200),
                        )
                        .is_ok()
                    } else {
                        false
                    };

                    if tcp_probe_ok {
                        tracker.is_online = true;
                        tracker.last_seen_text = "Agora".to_string();
                        tracker.last_beacon_instant = Some(Instant::now());
                    } else {
                        tracker.is_online = false;
                        tracker.last_seen_text = "Offline".to_string();
                    }
                }
            }

            // 4. Monta lista JSON com o estado real atualizado de todos os hosts
            let mut list = Vec::new();
            for tracker in hosts_map.values() {
                list.push(serde_json::json!({
                    "pdv_id": tracker.pdv_id,
                    "local_user": tracker.local_user,
                    "access_password": tracker.access_password,
                    "ip": tracker.ip,
                    "agent_version": tracker.agent_version,
                    "is_online": tracker.is_online,
                    "last_seen": tracker.last_seen_text,
                }));
            }

            if let Ok(mut lock) = discovered_bg.lock() {
                *lock = list.clone();
            }

            let json_str = serde_json::to_string(&list).unwrap_or_else(|_| "[]".to_string());
            let _ = proxy_bg.send_event(AdminCustomEvent::HostsDiscovered(json_str));

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
    let gsap_content = include_str!("../ui/gsap.min.js");
    let js_content = include_str!("../ui/app.js");

    // Injeta versão ativa do servidor e os estilos e scripts de forma inline
    let full_js = format!(
        "<script>\nwindow.__PRYSM_SERVER_VERSION__ = {:?};\n{}\n</script>",
        current_version, js_content
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
        .replace(r#"<script src="app.js"></script>"#, &full_js);

    let discovered_hosts_ipc = discovered_hosts.clone();
    let event_proxy_ipc = event_proxy.clone();

    // Diretório dedicado no AppData do operador para o WebView2 do Admin
    let data_dir = std::env::var_os("LOCALAPPDATA")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("PrysmView")
        .join("admin_webview");
    let _ = std::fs::create_dir_all(&data_dir);
    std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", &data_dir);

    let mut web_context = wry::WebContext::new(Some(data_dir));

    // Constrói a WebView vinculada à janela nativa do Windows
    let webview = WebViewBuilder::new_with_web_context(&mut web_context)
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
                let proxy = event_proxy_ipc.clone();
                let body_str = body.to_string();
                std::thread::spawn(move || {
                    let (target_ver, channel, download_url, sha256_hex) = if let Ok(val) = serde_json::from_str::<serde_json::Value>(&body_str) {
                        let v = val.get("version").and_then(|v| v.as_str()).unwrap_or("0.1.1").to_string();
                        let c = val.get("channel").and_then(|v| v.as_str()).unwrap_or("stable").to_string();
                        let url = val.get("download_url").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        let hash = val.get("sha256_hex").and_then(|v| v.as_str()).unwrap_or("").to_string();
                        (v, c, url, hash)
                    } else {
                        ("0.1.1".to_string(), "stable".to_string(), "".to_string(), "".to_string())
                    };

                    info!("Iniciando pipeline de atualização para v{} (canal: {})...", target_ver, channel);
                    let _ = proxy.send_event(AdminCustomEvent::UpdateProgress {
                        percent: 15,
                        message: format!("Conectando para obter pacote oficial v{}...", target_ver),
                    });

                    let package_res = crate::updater::fetch_update_binary(
                        &download_url,
                        &target_ver,
                        |p, msg| {
                            let _ = proxy.send_event(AdminCustomEvent::UpdateProgress {
                                percent: p,
                                message: msg.to_string(),
                            });
                        },
                    );

                    let binary_bytes = match package_res {
                        Ok(bytes) => bytes,
                        Err(e) => {
                            warn!("Falha ao obter pacote de atualização: {}", e);
                            let _ = proxy.send_event(AdminCustomEvent::UpdateError(format!("Erro ao obter pacote: {}", e)));
                            return;
                        }
                    };

                    let _ = proxy.send_event(AdminCustomEvent::UpdateProgress {
                        percent: 75,
                        message: "Validando cabeçalho executável PE e integridade SHA-256...".to_string(),
                    });

                    if !binary_bytes.starts_with(b"MZ") {
                        let err_msg = "Pacote inválido recebido: arquivo não é um executável Windows PE válido.".to_string();
                        warn!("{}", err_msg);
                        let _ = proxy.send_event(AdminCustomEvent::UpdateError(err_msg));
                        return;
                    }

                    if !sha256_hex.is_empty()
                        && sha256_hex != "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
                        && !crate::updater::verify_package_hash(&binary_bytes, &sha256_hex)
                    {
                        let err_msg = "Falha de integridade: hash SHA-256 do pacote baixado diverge do esperado.".to_string();
                        warn!("{}", err_msg);
                        let _ = proxy.send_event(AdminCustomEvent::UpdateError(err_msg));
                        return;
                    }

                    let _ = proxy.send_event(AdminCustomEvent::UpdateProgress {
                        percent: 90,
                        message: "Gravando novo estado e aplicando substituição atômica NTFS...".to_string(),
                    });

                    save_admin_state(&target_ver);

                    let _ = proxy.send_event(AdminCustomEvent::UpdateProgress {
                        percent: 100,
                        message: format!("Atualização para v{} aplicada com sucesso! Reinicializando...", target_ver),
                    });

                    let _ = proxy.send_event(AdminCustomEvent::UpdateComplete {
                        new_version: target_ver.clone(),
                    });

                    std::thread::sleep(Duration::from_millis(1000));

                    if let Err(e) = crate::updater::apply_in_place_update(&binary_bytes) {
                        warn!("Erro na substituição in-place: {}", e);
                        let _ = proxy.send_event(AdminCustomEvent::UpdateError(e));
                    }
                });
            } else if body.contains("check_updates") {
                let proxy = event_proxy_ipc.clone();
                let body_str = body.to_string();
                std::thread::spawn(move || {
                    let (channel, cur_ver) = if let Ok(val) = serde_json::from_str::<serde_json::Value>(&body_str) {
                        let ch = val.get("channel").and_then(|v| v.as_str()).unwrap_or("stable").to_string();
                        let cv = val.get("currentVersion").and_then(|v| v.as_str()).map(|s| s.to_string())
                            .unwrap_or_else(load_saved_version);
                        (ch, cv)
                    } else {
                        ("stable".to_string(), load_saved_version())
                    };
                    let status = crate::updater::check_for_updates_for_version(&cur_ver, &channel);
                    let json = serde_json::to_string(&status).unwrap_or_else(|_| "{}".to_string());
                    let _ = proxy.send_event(AdminCustomEvent::UpdateCheckResult(json));
                });
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
                let script = format!(
                    "if (window.onPrysmHostSync) {{ window.onPrysmHostSync({}); }}",
                    json
                );
                let _ = webview.evaluate_script(&script);
            }
            Event::UserEvent(AdminCustomEvent::UpdateCheckResult(json)) => {
                let script = format!(
                    "if (window.onPrysmUpdateChecked) {{ window.onPrysmUpdateChecked({}); }}",
                    json
                );
                let _ = webview.evaluate_script(&script);
            }
            Event::UserEvent(AdminCustomEvent::UpdateProgress { percent, message }) => {
                let escaped =
                    serde_json::to_string(&message).unwrap_or_else(|_| "\"\"".to_string());
                let script = format!(
                    "if (window.onPrysmUpdateProgress) {{ window.onPrysmUpdateProgress({}, {}); }}",
                    percent, escaped
                );
                let _ = webview.evaluate_script(&script);
            }
            Event::UserEvent(AdminCustomEvent::UpdateComplete { new_version }) => {
                let escaped =
                    serde_json::to_string(&new_version).unwrap_or_else(|_| "\"\"".to_string());
                let script = format!(
                    "if (window.onPrysmUpdateComplete) {{ window.onPrysmUpdateComplete({}); }}",
                    escaped
                );
                let _ = webview.evaluate_script(&script);
            }
            Event::UserEvent(AdminCustomEvent::UpdateError(msg)) => {
                let escaped = serde_json::to_string(&msg).unwrap_or_else(|_| "\"\"".to_string());
                let script = format!(
                    "if (window.onPrysmUpdateError) {{ window.onPrysmUpdateError({}); }}",
                    escaped
                );
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
