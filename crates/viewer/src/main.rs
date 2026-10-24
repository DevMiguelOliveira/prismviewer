//! prysmview-viewer - Ponto de entrada do aplicativo do operador

use eframe::egui::Vec2;
use prysmview_viewer::app::PrysmViewerApp;
use tracing::info;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Iniciando PrysmView Viewer para o operador...");

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size(Vec2::new(1024.0, 720.0))
            .with_min_inner_size(Vec2::new(800.0, 560.0))
            .with_title("PrysmView - Suporte Remoto PDV"),
        ..Default::default()
    };

    eframe::run_native(
        "PrysmView - Suporte Remoto PDV",
        native_options,
        Box::new(|cc| Ok(Box::new(PrysmViewerApp::new(cc)))),
    )
}
