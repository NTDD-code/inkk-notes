mod model;
mod storage;
mod stroke_algorithm;
mod renderer;
mod app;

use app::HandNoteApp;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("HandNote - Pure Rust Handwriting Notes")
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([850.0, 550.0]),
        ..Default::default()
    };

    eframe::run_native(
        "HandNote App",
        native_options,
        Box::new(|cc| Ok(Box::new(HandNoteApp::new(cc)))),
    )
}
