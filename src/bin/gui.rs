use eframe::egui;

fn main() -> eframe::Result {
    if std::env::var("IBKR_POREZ_DRY_RUN").is_ok() {
        eprintln!("GUI launched (dry run)");
        return Ok(());
    }

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1100.0, 700.0])
        .with_min_inner_size([800.0, 500.0])
        .with_title("ibkr-porez");

    if let Some(icon) = ibkr_porez::gui::icon::load_icon() {
        viewport = viewport.with_icon(std::sync::Arc::new(icon));
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "ibkr-porez",
        options,
        Box::new(|_cc| Ok(Box::new(ibkr_porez::gui::app::App::new()))),
    )
}
