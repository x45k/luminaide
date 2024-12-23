use eframe::egui;
use rfd::FileDialog;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Lumina IDE",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}

#[derive(Default)]
struct MyApp {
    current_folder: Option<PathBuf>,
    folder_structure: Vec<FolderEntry>,
    file_contents: Option<String>,
    open_file_path: Option<PathBuf>,
    scroll_offset: f32,
}

#[derive(Default)]
struct FolderEntry {
    path: PathBuf,
    is_folder: bool,
    open: bool,
    children: Option<Vec<FolderEntry>>,
}

impl FolderEntry {
    fn new(path: PathBuf, is_folder: bool) -> Self {
        FolderEntry {
            path,
            is_folder,
            open: false,
            children: None,
        }
    }
}

impl MyApp {
    fn load_folder(&mut self, path: &Path) -> io::Result<()> {
        self.folder_structure = self.read_folder_structure(path)?;
        Ok(())
    }

    fn read_folder_structure(&self, path: &Path) -> io::Result<Vec<FolderEntry>> {
        let mut entries = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let children = self.read_folder_structure(&path)?;
                let mut folder_entry = FolderEntry::new(path, true);
                folder_entry.children = Some(children);
                entries.push(folder_entry);
            } else {
                entries.push(FolderEntry::new(path, false));
            }
        }
        Ok(entries)
    }

    fn open_file(&mut self, path: &Path) -> io::Result<()> {
        let mut file = fs::File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        self.file_contents = Some(contents);
        self.open_file_path = Some(path.to_path_buf());
        Ok(())
    }

    fn save_file(&self) -> io::Result<()> {
        if let Some(ref path) = self.open_file_path {
            if let Some(ref contents) = self.file_contents {
                let mut file = fs::File::create(path)?;
                file.write_all(contents.as_bytes())?;
            }
        }
        Ok(())
    }

    fn display_folder_entry(&mut self, ui: &mut egui::Ui, entry: &mut FolderEntry) {
        if entry.is_folder {
            let folder_name = entry.path.file_name().unwrap_or_default().to_string_lossy();
            let response = ui.button(folder_name.to_string());

            if response.clicked() {
                entry.open = !entry.open;
            }

            if entry.open {
                ui.indent("", |ui| {
                    if let Some(children) = &mut entry.children {
                        for child in children {
                            self.display_folder_entry(ui, child);
                        }
                    }
                });
            }
        } else {
            if ui
                .selectable_label(false, entry.path.file_name().unwrap_or_default().to_string_lossy())
                .clicked()
            {
                if let Err(err) = self.open_file(&entry.path) {
                    eprintln!("Failed to open file: {err}");
                }
            }
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("folder_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("File Explorer");

                if ui.button("Open Folder").clicked() {
                    if let Some(path) = FileDialog::new().pick_folder() {
                        if let Err(err) = self.load_folder(&path) {
                            eprintln!("Failed to load folder: {err}");
                        } else {
                            self.current_folder = Some(path);
                        }
                    }
                }

                ui.separator();

                let mut folder_structure = std::mem::take(&mut self.folder_structure);
                for entry in &mut folder_structure {
                    self.display_folder_entry(ui, entry);
                }
                self.folder_structure = folder_structure;
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                if let Some(ref mut contents) = self.file_contents {
                    egui::ScrollArea::both()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(contents)
                                    .font(egui::TextStyle::Monospace) 
                                    .code_editor()
                                    .desired_width(f32::INFINITY),
                            );
                        });
            
                    if ui.button("Save").clicked() {
                        if let Err(err) = self.save_file() {
                            eprintln!("Failed to save file: {err}");
                        }
                    }
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("No file opened.");
                    });
                }
            });            
    }
}
