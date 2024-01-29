pub mod pdf_merger {
    use eframe::egui::{self, RichText, Ui};
    use eframe::epaint::Color32;
    use log::error;
    use lopdf::{self, Document};
    use qpdf::QPdf;
    use std::default::Default;
    use std::path::PathBuf;


    /// Сontains the validity value of the PDF's files
    #[derive(PartialEq, Default)]
    enum PdfValid {
        Valid,
        Invalid,
        #[default]
        Default,
    }


    /// Contains the necessary parameters of the self application
    #[derive(Default)]
    pub struct MyApp {
        picked_path: Vec<PathBuf>,
        documents: Vec<Document>,
        pdf_valid: PdfValid,
        error: String,
    }


    /// Displays a list of open files or an error in the GUI
    fn view_file(my_app: &mut MyApp, ui: &mut Ui) {
        if my_app.pdf_valid == PdfValid::Valid {
            ui.label("Picked files:");

            for i in &my_app.picked_path {
                ui.horizontal(|ui| {
                    let path = i.display().to_string();
                    ui.monospace(path);
                });
            }
        } else if my_app.pdf_valid == PdfValid::Invalid {
            let err = format!("Error: {}", my_app.error);
            ui.label(RichText::new(err).color(Color32::RED));
        }
    }

    fn drop_file(my_app: &mut MyApp) {
        let empty_doc_vec: Vec<Document> = vec![];
        my_app.documents = empty_doc_vec;
        my_app.pdf_valid = PdfValid::Default;
    }

    fn save_file(my_app: &mut MyApp, documents: Vec<Document>) {
        if let Some(path) = rfd::FileDialog::new().save_file() {
            let merged_pdf = crate::merge::pdf::merge_and_save(documents, path);
            match merged_pdf {
                Ok(()) => println!("Save merged PDF!"),
                Err(err) => {
                    error!("{}", err);
                    my_app.error = err.to_string();
                }
            }
        }
    }

    fn open_file(my_app: &mut MyApp) {
        if let Some(path) = rfd::FileDialog::new().pick_files() {
            my_app.picked_path = path.clone();
            let mut documents: Vec<Document> = vec![];
            let mut counter: i32 = 0;
            for i in path {
                load_document(my_app, &i, &mut documents, &mut counter);
            }

            my_app.documents = documents;

            if counter == 0 {
                my_app.pdf_valid = PdfValid::Valid;
            } else if counter != 0 {
                my_app.pdf_valid = PdfValid::Invalid;
            }
        }
    }

    fn load_document(my_app: &mut MyApp, path: &PathBuf, documents: &mut Vec<Document>, counter: &mut i32) {
        let document = lopdf::Document::load(path);
        match document {
            Ok(doc) => {
                println!("PDF is OK");
                documents.push(doc);
            }
            Err(lopdf::Error::Header) => {
                error!("{}", lopdf::Error::Header);
                my_app.error = lopdf::Error::Header.to_string();
                *counter += 1;
            }
            Err(lopdf::Error::Xref(err)) => {
                error!("{}", err);
                my_app.error = err.to_string();
                xref_error(my_app, path, documents, counter);
            }
            Err(err) => {
                error!("{}", err);
                my_app.error = err.to_string();
                *counter += 1;
            }
        }
    }

    fn xref_error(my_app: &mut MyApp, path: &PathBuf, documents: &mut Vec<Document>, counter: &mut i32) {
        let recovery_pdf = qpdf::QPdf::read(path);
        match recovery_pdf {
            Ok(doc) => {
                memory(my_app, doc, documents, counter);
            }
            Err(err) => {
                error!("{}", err);
                my_app.error = err.to_string();
                *counter += 1;
            }
        }
    }

    fn memory(my_app: &mut MyApp, doc: QPdf, documents: &mut Vec<Document>, counter: &mut i32) {
        doc.enable_recovery(true);
        let mem = doc.writer().write_to_memory();
        match mem {
            Ok(mem) => {
                let count = recovery(my_app, mem, documents, counter);
                *counter += count;
            }
            Err(err) => {
                error!("{}", err);
                my_app.error = err.to_string();
                *counter += 1;
            }
        }
    }

    fn recovery(my_app: &mut MyApp, mem: Vec<u8>, documents: &mut Vec<Document>, counter: &i32) -> i32 {
        let recovery_pdf = lopdf::Document::load_mem(&mem);
        match recovery_pdf {
            Ok(recovery_pdf) => {
                documents.push(recovery_pdf);
                println!("PDF is recovery!");
                *counter
            }
            Err(err) => {
                error!("{}", err);
                my_app.error = err.to_string();
                counter + 1
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
                        save_file(self, self.documents.clone());
                    }
    
                    if ui.button("Drop files").clicked() {
                        drop_file(self);
                    }
                });
    
                view_file(self, ui);
            });
        }
    }
}