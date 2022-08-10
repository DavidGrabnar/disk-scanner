use crate::app::utils::Bytes;
use eframe::egui::{CollapsingHeader, Direction, Layout, Ui};
use eframe::{egui, Storage};
use egui_extras::{Size, TableBody, TableBuilder};
use std::borrow::Borrow;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use sysinfo::{DiskExt, System, SystemExt};

mod tree;
mod utils;

#[derive(PartialEq)]
pub enum SidePanelView {
    None,
    Sources,
    Profile,
    Settings,
}

#[derive(PartialEq)]
pub enum MainPanelView {
    Blank,
    Explorer,
    Scanner,
}

#[derive(Default)]
pub struct Common {
    name: String,
}

#[derive(Default)]
pub struct Directory {
    common: Common,
    children: Vec<Element>,
}

pub struct File {
    common: Common,
}

pub enum Element {
    Directory(Directory),
    File(File),
}

#[derive(Default)]
pub struct Scan {
    directory: Directory,
    started_at: Option<SystemTime>,
    finished_at: Option<SystemTime>,
}

#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct DiskScanner {
    #[cfg_attr(feature = "persistence", serde(skip))]
    side_panel_view: SidePanelView,

    #[cfg_attr(feature = "persistence", serde(skip))]
    main_panel_view: MainPanelView,

    #[cfg_attr(feature = "persistence", serde(skip))]
    system: System,

    scan: Arc<Mutex<Scan>>,
    // background: (Sender<Scan>, Receiver<Scan>),
}

impl Default for DiskScanner {
    fn default() -> Self {
        Self {
            side_panel_view: SidePanelView::None,
            main_panel_view: MainPanelView::Blank,
            system: System::new_all(), // TODO only disk and disk list ?,
            scan: Arc::new(Mutex::new(Scan::default())),
        }
    }
}

impl DiskScanner {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customized the look at feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        #[cfg(feature = "persistence")]
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        //         self.system.refresh_all(); TODO is this needed ?

        Default::default()
    }

    fn menu_toggle_button(&mut self, ui: &mut Ui, view: SidePanelView, text: &str) {
        if ui
            .selectable_label(self.side_panel_view == view, text)
            .clicked()
        {
            if self.side_panel_view != view {
                self.side_panel_view = view;
            } else {
                self.side_panel_view = SidePanelView::None;
            }
        }
    }
}

impl eframe::App for DiskScanner {
    /// Called by the frame work to save state before shutdown.
    #[cfg(feature = "persistence")]
    fn save(&mut self, _storage: &mut dyn Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });

                self.menu_toggle_button(ui, SidePanelView::Sources, "Sources");
                self.menu_toggle_button(ui, SidePanelView::Profile, "Profile");
                self.menu_toggle_button(ui, SidePanelView::Settings, "Settings");

                egui::widgets::global_dark_light_mode_switch(ui);
            });
        });

        if self.side_panel_view != SidePanelView::None {
            egui::SidePanel::left("side_panel").show(ctx, |ui| {
                let title = match self.side_panel_view {
                    SidePanelView::None => "",
                    SidePanelView::Sources => "Sources",
                    SidePanelView::Profile => "Profile",
                    SidePanelView::Settings => "Settings",
                };

                ui.heading(title);
                ui.separator();

                match self.side_panel_view {
                    SidePanelView::None => {}
                    SidePanelView::Sources => {
                        CollapsingHeader::new("Local disks").show(ui, |ui| {
                            egui::Grid::new("grid_content")
                                .striped(true)
                                .show(ui, |ui| {
                                    for disk in self.system.disks() {
                                        let used_space_ratio = 1.0
                                            - disk.available_space() as f32
                                                / disk.total_space() as f32;

                                        ui.menu_button(
                                            format!(
                                                "{} ({})",
                                                disk.name().to_str().unwrap(),
                                                disk.mount_point().to_str().unwrap()
                                            ),
                                            |ui| {
                                                // TODO context_menu rather than click event ?
                                                if ui.button("Scan").clicked() {
                                                    ui.close_menu();
                                                    self.main_panel_view = MainPanelView::Scanner;

                                                    // reset
                                                    let counter = Arc::clone(&self.scan);
                                                    let mut scan = counter.lock().unwrap();
                                                    scan.directory = Directory {
                                                        common: Common {
                                                            name: String::from(
                                                                disk.name().to_str().unwrap(),
                                                            )
                                                        },
                                                        children: vec![],
                                                    };
                                                    scan.started_at = None;
                                                    scan.finished_at = None;

                                                    // execute
                                                    scan.started_at =
                                                        Option::from(SystemTime::now());

                                                    for entry in
                                                        fs::read_dir(disk.mount_point()).unwrap()
                                                    {
                                                        let entry = entry.unwrap();
                                                        let name = String::from(
                                                            entry
                                                                .file_name()
                                                                .to_str()
                                                                .unwrap(),
                                                        );

                                                        let result = fs::metadata(&entry.path());
                                                        if let Err(error) = result {
                                                            println!("Get metadata error on '{}' ({})", name, error);
                                                            continue;
                                                        }
                                                        let metadata = result.unwrap();

                                                        let element: Element = if metadata.is_dir() {
                                                            Element::Directory(Directory { common: Common { name }, children: vec![] })
                                                        } else if metadata.is_file() {
                                                            Element::File(File { common: Common { name } })
                                                        } else {
                                                            panic!("Dir entry is netiher directory, neither file");
                                                        };

                                                        scan.directory.children.push(element);
                                                    }

                                                    scan.finished_at =
                                                        Option::from(SystemTime::now());
                                                }
                                            },
                                        );
                                        // TODO full width 2nd column & color background as percentage of used space
                                        ui.label(format!(
                                            "{}/{} ({:.1}%)",
                                            Bytes::new(disk.total_space() - disk.available_space()),
                                            Bytes::new(disk.total_space()),
                                            used_space_ratio * 100.0
                                        ));
                                        ui.end_row();
                                    }
                                });
                        });
                    }
                    SidePanelView::Profile => {}
                    SidePanelView::Settings => {}
                }

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label("powered by ");
                        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                        ui.label(" and ");
                        ui.hyperlink_to(
                            "eframe",
                            "https://github.com/emilk/egui/tree/master/eframe",
                        );
                    });
                });
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| match self.main_panel_view {
            MainPanelView::Blank => {}
            MainPanelView::Explorer => {}
            MainPanelView::Scanner => {
                let counter = Arc::clone(&self.scan);
                let scan = counter.lock().unwrap();

                if scan.started_at.is_none() {
                    ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
                        ui.label("Select a source for scanning");
                    });
                } else {
                    TableBuilder::new(ui)
                        .striped(true)
                        .cell_layout(egui::Layout::left_to_right())
                        .column(Size::initial(300.0).at_least(200.0))
                        .column(Size::initial(60.0).at_least(40.0))
                        .column(Size::initial(60.0).at_least(40.0))
                        .column(Size::initial(60.0).at_least(40.0))
                        .column(Size::initial(60.0).at_least(40.0))
                        .column(Size::initial(60.0).at_least(40.0))
                        .column(Size::initial(60.0).at_least(40.0))
                        .column(Size::remainder().at_least(40.0))
                        .resizable(true)
                        .header(20.0, |mut header| {
                            header.col(|ui| {
                                ui.heading("Name");
                            });
                            // TODO add visual percentage as background of 'Percentage' column
                            header.col(|ui| {
                                ui.heading("Percent");
                            });
                            header.col(|ui| {
                                ui.heading("Size");
                            });
                            header.col(|ui| {
                                ui.heading("Items");
                            });
                            header.col(|ui| {
                                ui.heading("Files");
                            });
                            header.col(|ui| {
                                ui.heading("Subdirs");
                            });
                            header.col(|ui| {
                                ui.heading("Last change");
                            });
                            header.col(|ui| {
                                ui.heading("Attributes");
                            });
                        })
                        .body(|mut body| {
                            scan_table_directory(&mut body, &scan.directory);
                        });
                }
            }
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            egui::warn_if_debug_build(ui);
        });

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally chose either panels OR windows.");
            });
        }
    }
}

fn scan_table_directory(body: &mut TableBody, directory: &Directory) {
    scan_table_row(body, &directory.common);
    for element in directory.children.as_slice() {
        match element {
            Element::Directory(directory) => scan_table_directory(body, directory),
            Element::File(file) => scan_table_row(body, &file.common),
        }
    }
}

fn scan_table_row(body: &mut TableBody, element: &Common) {
    body.row(18.0, |mut row| {
        row.col(|ui| {
            ui.label(element.name.as_str());
        });
        row.col(|ui| {
            ui.label(char::from_u32(0x1f550).unwrap().to_string());
        });
        row.col(|ui| {
            ui.label("Normal row");
        });
        row.col(|ui| {
            ui.label("Normal row");
        });
        row.col(|ui| {
            ui.label("Normal row");
        });
        row.col(|ui| {
            ui.label("Normal row");
        });
        row.col(|ui| {
            ui.label("Normal row");
        });
        row.col(|ui| {
            ui.style_mut().wrap = Option::from(false);
            ui.label("Normal row");
        });
    });
}
