#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use eframe::egui::{self, Ui};
use log::error;
use lopdf::{self, Document};
use std::default::Default;
use std::path::PathBuf;
use qpdf::QPdf;
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
    picked_path: Vec<PathBuf>,
    documents: Vec<Document>,
    pdf_valid: PdfValid,
}

fn view_file(my_app: &mut MyApp, ui: &mut Ui) {
    if my_app.pdf_valid == PdfValid::Valid {
        ui.label("Picked files:");

        for i in &my_app.picked_path {
            ui.horizontal(|ui| {
                // let error = String::from("Some error");
                // let path = i.clone().into_os_string().into_string().unwrap_or(error);
                let path = i.display().to_string();
                ui.monospace(path);
            });
           }

    } else if my_app.pdf_valid == PdfValid::Invalid {
        ui.label("Invalid file header: not a PDF!");
    }
}

fn drop_file(my_app: &mut MyApp) {
    let empty_doc_vec: Vec<Document> = vec![];
    my_app.documents = empty_doc_vec;
    my_app.pdf_valid = PdfValid::Default;
}

fn save_file(documents: Vec<Document>) {
    if let Some(path) = rfd::FileDialog::new().save_file() {
        let merged_pdf = merge_pdf::merge(documents, path);
        match merged_pdf {
            Ok(()) => println!("Save merged PDF!"),
            Err(err) => {
                error!("{}", err);
            }
        }
    }
}


fn open_file (my_app: &mut MyApp) {
    if let Some(path) = rfd::FileDialog::new().pick_files() {
        my_app.picked_path = path.clone();
        let mut documents: Vec<Document> = vec![];
        let mut counter: i32 = 0;
        for i in path {
            load_document(&i, &mut documents, &mut counter);
        }

        my_app.documents = documents;

        if counter == 0 {
            my_app.pdf_valid = PdfValid::Valid;
        } else if counter != 0 {
            my_app.pdf_valid = PdfValid::Invalid;
        }
    }
}

fn load_document(path: &PathBuf, documents: &mut Vec<Document>, counter: &mut i32) {
    let document = lopdf::Document::load(path);
    match document {
        Ok(doc) => {
            println!("PDF is OK");
            documents.push(doc);
        }
        Err(lopdf::Error::Header) => {
            error!("{}", lopdf::Error::Header);
            *counter += 1;
        }
        Err(lopdf::Error::Xref(err)) => {
            error!("{}", err);
            xref_error(path, documents, counter);
        }
        Err(err) => {
            error!("{}", err);
            *counter += 1;
        }
    }
}

fn xref_error(path: &PathBuf, documents: &mut Vec<Document>, counter: &mut i32) {
    let recovery_pdf = qpdf::QPdf::read(path);
    match recovery_pdf {
        Ok(doc) => {
            memory(doc, documents, counter);
        }
        Err(err) => {
            error!("{}", err);
            *counter += 1;
        }
    }
}

fn memory(doc: QPdf, documents: &mut Vec<Document>, counter: &mut i32) {
    doc.enable_recovery(true);
    let mem = doc.writer().write_to_memory();
    match mem {
        Ok(mem) => {
            let count = recovery(mem, documents, counter);
            *counter += count;
        }
        Err(err) => {
            error!("{}", err);
            *counter += 1;
        }
    }
}

fn recovery(mem: Vec<u8>, documents: &mut Vec<Document>, counter: &i32) -> i32 {
    let recovery_pdf = lopdf::Document::load_mem(&mem);
    match recovery_pdf {
        Ok(recovery_pdf) => {
            documents.push(recovery_pdf);
            println!("PDF is recovery!");
            return counter + 0
        }
        Err(err) => {
            error!("{}", err);
            return counter + 1
        }
    }
}


impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Click 'Open file' button for pick your PDF files");

            ui.horizontal(|ui| {
                if ui.button("Open file...").clicked() {
                    open_file(self);
                }

                if ui.button("Merge").clicked() {
                    save_file(self.documents.clone());
                }

                if ui.button("Drop files").clicked() {
                    drop_file(self);
                }
            });

            view_file(self, ui);
        });
    }
}