mod app;
mod history;
mod qr;

use app::GallaQrApp;
use eframe::egui;

fn load_app_icon() -> Option<egui::IconData> {
    let icon_bytes = include_bytes!("../assets/gallaqr.png");
    if let Ok(img) = image::load_from_memory(icon_bytes) {
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        Some(egui::IconData {
            rgba: rgba.into_raw(),
            width,
            height,
        })
    } else {
        None
    }
}

fn main() -> eframe::Result {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("GallaQR - Generador de Códigos QR")
        .with_inner_size([1080.0, 720.0])
        .with_min_inner_size([340.0, 360.0])
        .with_decorations(true);

    if let Some(icon) = load_app_icon() {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "GallaQR",
        options,
        Box::new(|cc| {
            let mut visuals = egui::Visuals::dark();
            visuals.panel_fill = egui::Color32::from_rgb(16, 18, 24);
            visuals.window_fill = egui::Color32::from_rgb(22, 25, 33);
            cc.egui_ctx.set_visuals(visuals);

            let mut app = GallaQrApp::new(cc);

            // Pre-cargar argumentos de línea de comandos si se proporcionan
            let args: Vec<String> = std::env::args().collect();
            if args.len() >= 3 {
                app.content_input = args[1].clone();
                app.title_input = args[2].clone();
            } else if args.len() == 2 && !args[1].starts_with('-') {
                app.content_input = args[1].clone();
            }

            Ok(Box::new(app))
        }),
    )
}
