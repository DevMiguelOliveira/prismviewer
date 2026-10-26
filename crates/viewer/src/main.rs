#![windows_subsystem = "windows"]

//! prysmview-viewer - Ponto de entrada do aplicativo do operador

use eframe::egui::Vec2;
use prysmview_viewer::app::PrysmViewerApp;
use std::sync::Arc;
use tracing::info;

fn load_brand_icon() -> Option<Arc<eframe::egui::IconData>> {
    let raw = include_bytes!("../../../assets/logo.png");
    let img = image::load_from_memory(raw).ok()?.to_rgba8();
    let (width, height) = img.dimensions();
    Some(Arc::new(eframe::egui::IconData {
        rgba: img.into_raw(),
        width,
        height,
    }))
}

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Iniciando PrysmView Viewer para o operador...");

    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_inner_size(Vec2::new(1024.0, 720.0))
        .with_min_inner_size(Vec2::new(800.0, 560.0))
        .with_title("PrysmView - Suporte Remoto PDV");

    if let Some(icon) = load_brand_icon() {
        viewport = viewport.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "PrysmView - Suporte Remoto PDV",
        native_options,
        Box::new(|cc| Ok(Box::new(PrysmViewerApp::new(cc)))),
    )
}
