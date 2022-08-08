use crate::app::utils::Bytes;
use eframe::egui::{CollapsingHeader, Ui};
use eframe::{egui, epi};
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

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct DiskScanner {
    #[cfg_attr(feature = "persistence", serde(skip))]
    side_panel_view: SidePanelView,

    #[cfg_attr(feature = "persistence", serde(skip))]
    system: System,
}

impl Default for DiskScanner {
    fn default() -> Self {
        Self {
            side_panel_view: SidePanelView::None,
            system: System::new_all(), // TODO only disk and disk list ?
        }
    }
}

impl DiskScanner {
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

impl epi::App for DiskScanner {
    fn name(&self) -> &str {
        "Disk scanner"
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::Context,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }

        self.system.refresh_all();

        println!("=> disks:");
        for disk in self.system.disks() {
            println!("{:?}", disk);
        }
    }

    /// Called by the frame work to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        // Examples of how to create different panels and windows.
        // Pick whichever suits you.
        // Tip: a good default choice is to just keep the `CentralPanel`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

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

                                        ui.label(format!(
                                            "{} ({})",
                                            disk.name().to_str().unwrap(),
                                            disk.mount_point().to_str().unwrap()
                                        ));
                                        // TODO full width 2nd column & color background as percentage of used space
                                        ui.label(format!(
                                            "{}/{} ({:.1}%)",
                                            Bytes::new(disk.available_space()),
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

        egui::CentralPanel::default().show(ctx, |ui| {});

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
