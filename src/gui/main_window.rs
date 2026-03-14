use crate::models::DeclarationStatus;
use eframe::egui;
use egui_extras::{Column, TableBuilder};

use super::app::{App, BulkAction, FilterScope, SortColumn};
use super::styles;

pub fn show(ui: &mut egui::Ui, app: &mut App) {
    toolbar(ui, app);

    if let Some(ref banner) = app.warning_banner {
        ui.horizontal(|ui| {
            let rect = ui.available_rect_before_wrap();
            ui.painter().rect_filled(rect, 0.0, styles::WARNING_BG);
            ui.colored_label(styles::WARNING_TEXT, banner);
        });
        ui.add_space(2.0);
    }

    if let Some(ref text) = app.progress_text {
        let time = ui.ctx().input(|i| i.time);
        #[allow(clippy::cast_possible_truncation)]
        let progress = (0.5 + 0.5 * (time * 2.0).sin()) as f32;
        ui.add(
            egui::ProgressBar::new(progress)
                .text(text.as_str())
                .animate(true),
        );
        ui.add_space(4.0);
    } else if let Some((ref msg, color)) = app.status_message {
        ui.colored_label(color, msg);
        ui.add_space(2.0);
    }

    filter_bar(ui, app);
    ui.add_space(4.0);
    declaration_table(ui, app);
}

fn toolbar(ui: &mut egui::Ui, app: &mut App) {
    ui.horizontal(|ui| {
        let busy = app.bg_busy;

        ui.add_enabled_ui(!busy, |ui| {
            let label = if busy {
                "\u{27f3} Syncing\u{2026}"
            } else {
                "\u{27f3} Sync"
            };
            if ui
                .add(
                    egui::Button::new(egui::RichText::new(label).size(16.0))
                        .min_size(egui::vec2(100.0, 32.0)),
                )
                .clicked()
            {
                app.start_sync(false);
            }
        });

        ui.add_enabled_ui(!busy, |ui| {
            if ui
                .add(egui::Button::new("Force").min_size(egui::vec2(0.0, 32.0)))
                .on_hover_text("Force re-fetch all data from IBKR and NBS")
                .clicked()
            {
                app.confirm_force_sync = true;
            }
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Config").clicked() {
                app.config_dialog = Some(super::config_dialog::ConfigDialog::new(&app.config));
            }
            if ui.button("Import").clicked() {
                app.import_dialog = Some(super::import_dialog::ImportDialog::new());
            }
        });
    });
    ui.add_space(4.0);
}

fn filter_bar(ui: &mut egui::Ui, app: &mut App) {
    ui.horizontal(|ui| {
        ui.label("Filter:");
        for scope in FilterScope::ALL {
            if ui
                .selectable_label(app.filter == *scope, scope.label())
                .clicked()
            {
                app.set_filter(*scope);
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("Unselect all").clicked() {
                app.unselect_all();
            }
            if ui.button("Select all").clicked() {
                app.select_all();
            }

            if !app.selected.is_empty() {
                ui.separator();
                if ui.button("Apply").clicked() {
                    app.apply_bulk_action();
                }
                egui::ComboBox::from_id_salt("bulk_action")
                    .selected_text(app.bulk_action.label())
                    .show_ui(ui, |ui| {
                        for action in BulkAction::ALL {
                            ui.selectable_value(&mut app.bulk_action, *action, action.label());
                        }
                    });
                ui.label(format!("{} selected", app.selected.len()));
            }
        });
    });
}

#[allow(clippy::too_many_lines)]
fn declaration_table(ui: &mut egui::Ui, app: &mut App) {
    let row_height = 24.0;
    let available = ui.available_size();

    let table = TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .sense(egui::Sense::click())
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .min_scrolled_height(200.0)
        .max_scroll_height(available.y - 40.0)
        .column(Column::exact(30.0))
        .column(Column::initial(140.0).resizable(true))
        .column(Column::initial(80.0).resizable(true))
        .column(Column::initial(120.0).resizable(true))
        .column(Column::initial(130.0).resizable(true))
        .column(Column::initial(80.0).resizable(true))
        .column(Column::initial(100.0).resizable(true))
        .column(Column::remainder());

    let mut clicked_sort: Option<SortColumn> = None;
    let mut double_clicked_id: Option<String> = None;
    let mut action: Option<RowAction> = None;

    table
        .header(row_height, |mut header| {
            header.col(|_ui| {});
            for col in SortColumn::ALL {
                header.col(|ui| {
                    let arrow = if app.sort_column == *col {
                        if app.sort_ascending {
                            styles::SORT_ARROW_UP
                        } else {
                            styles::SORT_ARROW_DOWN
                        }
                    } else {
                        ""
                    };
                    let label = format!("{}{arrow}", col.label());
                    let resp = ui.add(
                        egui::Label::new(egui::RichText::new(label).strong())
                            .sense(egui::Sense::click()),
                    );
                    if resp.clicked() {
                        clicked_sort = Some(*col);
                    }
                    resp.on_hover_cursor(egui::CursorIcon::PointingHand);
                });
            }
            header.col(|ui| {
                ui.strong("Actions");
            });
        })
        .body(|body| {
            let decls: Vec<_> = app.declarations.clone();
            body.rows(row_height, decls.len(), |mut row| {
                let idx = row.index();
                let decl = &decls[idx];
                let id = &decl.declaration_id;

                row.col(|ui| {
                    let mut checked = app.selected.contains(id);
                    if ui.checkbox(&mut checked, "").changed() {
                        if checked {
                            app.selected.insert(id.clone());
                        } else {
                            app.selected.remove(id);
                        }
                    }
                });

                row.col(|ui| {
                    ui.label(id.as_str());
                });
                row.col(|ui| {
                    ui.label(decl.display_type());
                });
                row.col(|ui| {
                    ui.label(decl.display_period());
                });
                row.col(|ui| {
                    ui.label(decl.display_tax());
                });
                row.col(|ui| {
                    ui.label(decl.status.to_string());
                });
                row.col(|ui| {
                    ui.label(decl.created_at.format("%Y-%m-%d").to_string());
                });

                row.col(|ui| {
                    ui.horizontal(|ui| {
                        row_actions(ui, decl, &mut action, app);
                    });
                });

                if row.response().double_clicked() {
                    double_clicked_id = Some(id.clone());
                }
            });
        });

    if let Some(col) = clicked_sort {
        app.set_sort(col);
    }
    if let Some(id) = double_clicked_id {
        app.details_dialog = Some(super::details_dialog::DetailsDialog::new(&id, &app.storage));
    }
    if let Some(act) = action {
        match act {
            RowAction::Submit(id) => app.row_submit(&id),
            RowAction::Pay(id) => app.row_pay(&id),
            RowAction::Revert(id) => app.row_revert(&id),
            RowAction::SetTax(id) => {
                app.assessment_dialog = Some(super::assessment_dialog::AssessmentDialog::new(id));
            }
            RowAction::Export(id) => app.row_export(id),
        }
    }
}

enum RowAction {
    Submit(String),
    Pay(String),
    Revert(String),
    SetTax(String),
    Export(String),
}

fn row_actions(
    ui: &mut egui::Ui,
    decl: &crate::models::Declaration,
    action: &mut Option<RowAction>,
    app: &App,
) {
    let id = &decl.declaration_id;
    let status = decl.status;

    if status == DeclarationStatus::Finalized {
        if ui.small_button("Revert").clicked() {
            *action = Some(RowAction::Revert(id.clone()));
        }
    } else {
        if status == DeclarationStatus::Draft && ui.small_button("Submit").clicked() {
            *action = Some(RowAction::Submit(id.clone()));
        }

        if matches!(
            status,
            DeclarationStatus::Submitted | DeclarationStatus::Pending
        ) {
            let tax =
                crate::declaration_manager::DeclarationManager::new(&app.storage).tax_due_rsd(decl);
            if tax > rust_decimal::Decimal::ZERO && ui.small_button("Pay").clicked() {
                *action = Some(RowAction::Pay(id.clone()));
            }
        }
    }

    if status != DeclarationStatus::Draft && ui.small_button("Set Tax").clicked() {
        *action = Some(RowAction::SetTax(id.clone()));
    }

    let exporting = app.exporting_ids.contains(id);
    ui.add_enabled_ui(!exporting, |ui| {
        let label = if exporting {
            "Re-exporting..."
        } else {
            "Re-export"
        };
        if ui.small_button(label).clicked() {
            *action = Some(RowAction::Export(id.clone()));
        }
    });
}
