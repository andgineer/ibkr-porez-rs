use eframe::egui;
use rust_decimal::Decimal;

use crate::declaration_manager::DeclarationManager;

use super::app::App;

pub struct AssessmentDialog {
    pub declaration_id: String,
    pub tax_input: String,
    pub mark_paid: bool,
}

impl AssessmentDialog {
    pub fn new(id: String) -> Self {
        Self {
            declaration_id: id,
            tax_input: String::new(),
            mark_paid: false,
        }
    }
}

pub fn show(ctx: &egui::Context, app: &mut App) {
    if app.assessment_dialog.is_none() {
        return;
    }

    let mut dismiss = false;
    let mut apply = false;

    let dialog = app.assessment_dialog.as_mut().unwrap();

    egui::Window::new("Tax Assessment")
        .collapsible(false)
        .resizable(false)
        .default_width(350.0)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Tax due (RSD):");
                ui.text_edit_singleline(&mut dialog.tax_input);
            });
            ui.checkbox(&mut dialog.mark_paid, "Already paid");
            ui.add_space(4.0);
            ui.colored_label(
                egui::Color32::from_gray(160),
                "Enter the assessed tax amount. Check 'Already paid' if payment is done.",
            );
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("OK").clicked() {
                    apply = true;
                }
                if ui.button("Cancel").clicked() {
                    dismiss = true;
                }
            });
        });

    if apply {
        let dialog = app.assessment_dialog.take().unwrap();
        match dialog.tax_input.trim().parse::<Decimal>() {
            Ok(tax_due) => {
                let manager = DeclarationManager::new(&app.storage);
                if let Err(e) =
                    manager.set_assessed_tax(&dialog.declaration_id, tax_due, dialog.mark_paid)
                {
                    app.error_dialog = Some(e.to_string());
                }
                app.refresh_declarations();
            }
            Err(_) => {
                app.error_dialog = Some("Invalid tax amount. Enter a number.".to_string());
            }
        }
    } else if dismiss {
        app.assessment_dialog = None;
    }
}
