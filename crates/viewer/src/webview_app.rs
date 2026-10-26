//! Launcher WebView2 para o console administrativo PrysmAdmin.
//! Utiliza `tao` e `wry` para fornecer renderização moderna via Microsoft Edge WebView2 nativo.

use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Icon, WindowBuilder},
};
use tracing::info;
use wry::WebViewBuilder;

/// Carrega o ícone oficial da marca PrysmView para a barra de título e janela do Windows.
fn load_window_icon() -> Option<Icon> {
    let raw = include_bytes!("../../../assets/logo.png");
    let img = image::load_from_memory(raw).ok()?.to_rgba8();
    let (width, height) = img.dimensions();
    Icon::from_rgba(img.into_raw(), width, height).ok()
}

/// Inicia o aplicativo desktop PrysmAdmin com o frontend web moderno e IPC nativo.
pub fn run_webview_app() -> Result<(), Box<dyn std::error::Error>> {
    // Limpeza automática de backups temporários .old gerados por atualizações anteriores
    crate::updater::cleanup_previous_update_backup();

    info!("Inicializando EventLoop do PrysmAdmin (WebView2 / Wry)...");
    let event_loop = EventLoop::new();

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

    // Injeta os estilos e scripts de forma inline
    let bundled_html = raw_html
        .replace(
            r#"<link rel="stylesheet" href="style.css">"#,
            &format!("<style>\n{}\n</style>", css_content),
        )
        .replace(
            r#"<script src="app.js"></script>"#,
            &format!("<script>\n{}\n</script>", js_content),
        );

    // Constrói a WebView vinculada à janela nativa do Windows
    let webview = WebViewBuilder::new()
        .with_devtools(true)
        .with_html(&bundled_html)
        .with_ipc_handler(|msg| {
            let body = msg.body();
            info!("Comando recebido do console de suporte: {}", body);
            if body.contains("check_updates") {
                let status = crate::updater::check_for_updates("stable");
                info!("Checagem de atualização executada: {:?}", status);
            } else if body.contains("apply_update") {
                info!("Comando de instalação in-place de atualização recebido com sucesso.");
            } else if body.contains("export_audit_csv") {
                info!("Comando de exportação de registros de auditoria CSV recebido.");
            }
        })
        .build(&window)?;

    info!("Janela PrysmAdmin inicializada com sucesso em modo WebView2.");

    event_loop.run(move |event, _, control_flow| {
        // Garante que a WebView permaneça viva durante toda a execução da interface
        let _ = &webview;
        *control_flow = ControlFlow::Wait;

        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            info!("Encerrando console administrativo PrysmAdmin...");
            *control_flow = ControlFlow::Exit;
        }
    });
}
