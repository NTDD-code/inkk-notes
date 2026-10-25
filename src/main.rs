#![windows_subsystem = "windows"]

mod app;
pub mod autosave;
pub mod cli;
pub mod flowchart_core;
pub mod flowchart_ui;
pub mod geometry_sketchpad;
pub mod library;
pub mod math_engine;
pub mod math_renderer;
mod model;
pub mod onboarding_tour;
pub mod pdf_export;
mod renderer;
mod ruler;
pub mod shortcuts;
pub mod splash_screen;
mod storage;
mod stroke_algorithm;
mod syntax_highlight;
mod theme;
pub mod ui_dialogs;
pub mod ui_math_studio;
pub mod updater;
pub mod vector_icon;
mod widgets;

use app::InkkApp;

fn load_icon() -> Option<egui::IconData> {
    let png_bytes = include_bytes!("../assets/icon.png");
    if let Ok(img) = image::load_from_memory(png_bytes) {
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

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let first = &args[1];
        if first == "--cad" || first == "cad" || first == "--sketchpad" || first == "sketchpad" {
            let mut cad_viewport = eframe::egui::ViewportBuilder::default()
                .with_title("Inkk CAD - 3D Spatial Geometry Studio")
                .with_inner_size([1200.0, 780.0])
                .with_min_inner_size([720.0, 520.0]);
            if let Some(icon) = load_icon() {
                cad_viewport = cad_viewport.with_icon(icon);
            }
            let native_options = eframe::NativeOptions {
                viewport: cad_viewport,
                ..Default::default()
            };
            return eframe::run_native(
                "Inkk CAD",
                native_options,
                Box::new(|cc| Ok(Box::new(geometry_sketchpad::CadStandaloneApp::new(cc)))),
            );
        }
        if first == "--help"
            || first == "-h"
            || first == "help"
            || first == "eval"
            || first == "run"
            || first == "matrix"
            || first == "axes"
            || first == "table"
            || first == "info"
            || first == "new"
        {
            cli::run_cli(&args);
            return Ok(());
        }
    }
    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_title("Inkk - Handwritten Notes")
        .with_inner_size([1280.0, 800.0])
        .with_min_inner_size([850.0, 550.0]);
    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(icon);
    }
    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Inkk",
        native_options,
        Box::new(|cc| Ok(Box::new(InkkApp::new(cc)))),
    )
}
