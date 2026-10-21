mod model;
mod storage;
mod stroke_algorithm;
mod renderer;
mod theme;
mod widgets;
mod ruler;
pub mod math_engine;
pub mod math_renderer;
pub mod ui_math_studio;
mod app;

use app::InkkApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Inkk - Handwritten Notes")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([850.0, 550.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Inkk",
        native_options,
        Box::new(|cc| Ok(Box::new(InkkApp::new(cc)))),
    )
}
