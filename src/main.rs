#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui::{self};
use std::default::Default;
mod app;
mod merge;

fn main() -> std::result::Result<(), eframe::Error> {
    // Log to stderr (if you run with `RUST_LOG=debug`).
    env_logger::init();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            // wide enough for the drag-drop overlay text
            .with_inner_size([540.0, 540.0])
            .with_drag_and_drop(true)
            .with_title("PDF MERGER"),
        ..Default::default()
    };
    eframe::run_native(
        "Native file dialogs and drag-and-drop files",
        options,
        Box::new(|_cc| Box::<app::pdf_merger::MyApp>::default()),
    )
}