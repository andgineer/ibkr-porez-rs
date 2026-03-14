use eframe::egui;
use eframe::icon_data::from_png_bytes;

const ICON_PNG: &[u8] = include_bytes!("icon.png");

pub fn load_icon() -> Option<egui::IconData> {
    from_png_bytes(ICON_PNG).ok()
}
