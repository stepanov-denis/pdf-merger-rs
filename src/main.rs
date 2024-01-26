#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui::{self, ScrollArea};
use eframe::glow::MAX_VERTEX_SHADER_STORAGE_BLOCKS;
use std::fmt::write;
use std::path::PathBuf;
use log::{debug, error, log_enabled, info, Level};
use std::default::Default;
use std::fmt::Write as _;
use std::fs::File;
use std::io::{Write, BufReader, BufRead, Error};
use qpdf::*;
use qpdf::QPdf;
use lopdf::{self, Document};
mod merge_pdf;


fn main() -> std::result::Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([540.0, 540.0]) // wide enough for the drag-drop overlay text
            .with_drag_and_drop(true)
            .with_title("PDF MERGER"),
        ..Default::default()
    };
    eframe::run_native(
        "Native file dialogs and drag-and-drop files",
        options,
        Box::new(|_cc| Box::<MyApp>::default()),
    )
}


#[derive(PartialEq)]
enum PdfValid {
    Valid,
    Invalid,
    Default
}


impl Default for PdfValid {
    fn default() -> Self {
        // Возвращаемое значение при использовании PdfValid::default()
        PdfValid::Default
    }
}


#[derive(Default)]
struct MyApp {
    dropped_files: Vec<egui::DroppedFile>,
    picked_path: Vec<PathBuf>,
    documents: Vec<Document>,
    pdf_valid: PdfValid,
}

fn recovery_pdf(mem: Vec<u8>, mut documents: Vec<Document>, mut count: i32) {
    let recovery_pdf = lopdf::Document::load_mem(&mem);
    match recovery_pdf {
        Ok(recovery_pdf) => {
            println!("PDF is recovery!");
            documents.push(recovery_pdf);
        }
        Err(err) => {
            println!("{}", err);
            error!("{}", err);
            count += 1;
        }
    }
}


impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Click 'Open file' button for pick your PDF files");

            ui.horizontal(|ui| {
                if ui.button("Open file...").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_files() {
                        self.picked_path = path.clone();
                        let mut documents: Vec<Document> = vec![];
                        let mut count: i32 = 0;
                        for i in path {
                            let document = lopdf::Document::load(i.clone());
                            match document {
                                Ok(doc) => {
                                    println!("PDF is OK");
                                    documents.push(doc);
                                }
                                Err(lopdf::Error::Header) => {
                                    println!("{}", lopdf::Error::Header);
                                    error!("{}", lopdf::Error::Header);
                                    count += 1;
                                }
                                Err(lopdf::Error::Xref(err)) => {
                                    println!("{}", err);
                                    error!("{}", err);
                                    let recovery_pdf = qpdf::QPdf::read(i);
                                    match recovery_pdf {
                                        Ok(doc) => {
                                            doc.enable_recovery(true);
                                            let mem = doc.writer().write_to_memory();
                                            match mem {
                                                Ok(mem) => {
                                                    let recovery_pdf = lopdf::Document::load_mem(&mem);
                                                    match recovery_pdf {
                                                        Ok(recovery_pdf) => {
                                                            println!("PDF is recovery!");
                                                            documents.push(recovery_pdf);
                                                        }
                                                        Err(err) => {
                                                            println!("{}", err);
                                                            error!("{}", err);
                                                            count += 1;
                                                        }
                                                    }
                                                }
                                                Err(err) => {
                                                    println!("{}", err);
                                                    error!("{}", err);
                                                    count += 1;
                                                }
                                            }
                                        }
                                        Err(err) => {
                                            println!("{}", err);
                                            error!("{}", err);
                                            count += 1;
                                        }
                                    }
                                }
                                Err(err) => {
                                    println!("{}", err);
                                    error!("{}", err);
                                    count +1;
                                }
                            }
                             
                        }

                        self.documents = documents;

                        if count == 0 {
                            self.pdf_valid = PdfValid::Valid;
                        } else {
                            self.pdf_valid = PdfValid::Invalid;
                        }
                    }
                }

                if ui.button("Merge").clicked() {
                    if let Some(path) = rfd::FileDialog::new().save_file() {
                        for i in &self.picked_path {
                        let new_document = merge_pdf::merge(self.documents.clone(), path.clone());
                        match new_document {
                            Ok(()) => info!("Save merged PDF!"),
                            Err(err) => {
                                println!("{}", err);
                                error!("{}", err);
                            }
                        }
                    }
                    }
                }

                if ui.button("Drop files").clicked() {
                    let empty_doc_vec: Vec<Document> = vec![];
                    self.documents = empty_doc_vec;
                    self.pdf_valid = PdfValid::Default;
                }
            });

        //    ui.horizontal(|ui|{
            if self.pdf_valid == PdfValid::Valid {
                ui.label("Picked files:");

                for i in &self.picked_path {
                    ui.horizontal(|ui| {
                        let alert_no_file = String::from("No file!");
                        let path = i.clone().into_os_string().into_string().unwrap_or(alert_no_file);
                        ui.monospace(path);
        
                    });
                   }

            } else if self.pdf_valid == PdfValid::Invalid {
                ui.label("Invalid file header: not a PDF!");
            }
        //    });

            // Show dropped files (if any):
            if !self.dropped_files.is_empty() {
                ui.group(|ui| {
                    ui.label("Dropped files:");

                    for file in &self.dropped_files {
                        let mut info = if let Some(path) = &file.path {
                            path.display().to_string()
                        } else if !file.name.is_empty() {
                            file.name.clone()
                        } else {
                            "???".to_owned()
                        };

                        let mut additional_info = vec![];
                        if !file.mime.is_empty() {
                            additional_info.push(format!("type: {}", file.mime));
                        }
                        if let Some(bytes) = &file.bytes {
                            additional_info.push(format!("{} bytes", bytes.len()));
                        }
                        if !additional_info.is_empty() {
                            info += &format!(" ({})", additional_info.join(", "));
                        }

                        ui.label(info);
                    }
                });
            }
        });

        preview_files_being_dropped(ctx);

        // Collect dropped files:
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                self.dropped_files = i.raw.dropped_files.clone();
            }
        });
    }
}


/// Preview hovering files:
fn preview_files_being_dropped(ctx: &egui::Context) {
    use egui::*;
    use std::fmt::Write as _;

    if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
        let text = ctx.input(|i| {
            let mut text = "Dropping files:\n".to_owned();
            for file in &i.raw.hovered_files {
                if let Some(path) = &file.path {
                    write!(text, "\n{}", path.display()).ok();
                } else if !file.mime.is_empty() {
                    write!(text, "\n{}", file.mime).ok();
                } else {
                    text += "\n???";
                }
            }
            text
        });

        let painter =
            ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

        let screen_rect = ctx.screen_rect();
        painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(192));
        painter.text(
            screen_rect.center(),
            Align2::CENTER_CENTER,
            text,
            TextStyle::Heading.resolve(&ctx.style()),
            Color32::WHITE,
        );
    }
}