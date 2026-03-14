use eframe::egui;

use crate::config as app_config;
use crate::models::UserConfig;

use super::app::App;

pub struct ConfigDialog {
    pub config: UserConfig,
    pub data_dir_str: String,
    pub output_folder_str: String,
}

impl ConfigDialog {
    pub fn new(current: &UserConfig) -> Self {
        Self {
            config: current.clone(),
            data_dir_str: current.data_dir.clone().unwrap_or_default(),
            output_folder_str: current.output_folder.clone().unwrap_or_default(),
        }
    }
}

pub fn show(ctx: &egui::Context, app: &mut App) {
    if app.config_dialog.is_none() {
        return;
    }

    let mut dismiss = false;
    let mut save = false;

    let dialog = app.config_dialog.as_mut().unwrap();

    egui::Window::new("Configuration")
        .collapsible(false)
        .resizable(true)
        .default_width(500.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Personal Taxpayer Data");
                ui.add_space(4.0);
                field(ui, "Personal ID (JMBG)", &mut dialog.config.personal_id);
                field(ui, "Full Name", &mut dialog.config.full_name);
                field(ui, "Address", &mut dialog.config.address);
                field(ui, "City Code", &mut dialog.config.city_code);
                field(ui, "Phone", &mut dialog.config.phone);
                field(ui, "Email", &mut dialog.config.email);

                ui.add_space(12.0);
                ui.heading("IBKR Flex Parameters");
                ui.add_space(4.0);
                field(ui, "Flex Token", &mut dialog.config.ibkr_token);
                field(ui, "Flex Query ID", &mut dialog.config.ibkr_query_id);
                ui.hyperlink_to(
                    "How to get Flex Token and Flex Query ID in IBKR",
                    "https://andgineer.github.io/ibkr-porez-rs/en/ibkr.html",
                );

                ui.add_space(12.0);
                ui.heading("App Settings");
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label("Version:");
                    ui.label(env!("CARGO_PKG_VERSION"));
                });

                dir_field(ui, "Data Directory", &mut dialog.data_dir_str);
                dir_field(ui, "Output Folder", &mut dialog.output_folder_str);

                ui.horizontal(|ui| {
                    ui.label("Config & Logs:");
                    let config_dir = app_config::config_dir();
                    ui.label(config_dir.display().to_string());
                    if ui.small_button("Open").clicked() {
                        let _ = open::that(&config_dir);
                    }
                });

                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        save = true;
                    }
                    if ui.button("Cancel").clicked() {
                        dismiss = true;
                    }
                });
            });
        });

    if save {
        let dialog = app.config_dialog.take().unwrap();
        let mut cfg = dialog.config;
        cfg.data_dir = if dialog.data_dir_str.is_empty() {
            None
        } else {
            Some(dialog.data_dir_str)
        };
        cfg.output_folder = if dialog.output_folder_str.is_empty() {
            None
        } else {
            Some(dialog.output_folder_str)
        };
        if let Err(e) = app_config::save_config(&cfg) {
            app.error_dialog = Some(e.to_string());
        } else {
            app.config = cfg;
            app.refresh_declarations();
        }
    } else if dismiss {
        app.config_dialog = None;
    }
}

fn field(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.horizontal(|ui| {
        ui.label(format!("{label}:"));
        ui.text_edit_singleline(value);
    });
}

fn dir_field(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.horizontal(|ui| {
        ui.label(format!("{label}:"));
        ui.text_edit_singleline(value);
        if ui.small_button("Browse...").clicked()
            && let Some(path) = rfd::FileDialog::new().pick_folder()
        {
            *value = path.display().to_string();
        }
        if !value.is_empty() && ui.small_button("Open").clicked() {
            let _ = open::that(value.as_str());
        }
    });
}
