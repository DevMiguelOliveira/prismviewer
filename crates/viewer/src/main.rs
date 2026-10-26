#![windows_subsystem = "windows"]

//! prysmview-viewer - Ponto de entrada do aplicativo do operador PrysmAdmin.
//! Utiliza a moderna arquitetura WebView2 / Wry (HTML5, CSS3 Cyber Slate e SVG),
//! com fallback seguro para eframe caso necessário.

use prysmview_viewer::webview_app::run_webview_app;
use tracing::{error, info};

fn main() {
    tracing_subscriber::fmt::init();
    info!("Iniciando PrysmView Admin Console (Moderno WebView2 / Tauri Core)...");

    // Tenta iniciar a interface moderna em WebView2 (Windows 10 e 11)
    if let Err(err) = run_webview_app() {
        error!("Erro ao iniciar PrysmAdmin em modo WebView2: {:?}", err);
    }
}
