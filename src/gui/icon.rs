use eframe::egui;
use eframe::icon_data::from_png_bytes;

const ICON_PNG: &[u8] = include_bytes!("icon.png");

pub fn load_icon() -> Option<egui::IconData> {
    from_png_bytes(ICON_PNG).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_icon_returns_some() {
        let icon = load_icon();
        assert!(icon.is_some(), "embedded icon.png should load successfully");
    }

    #[test]
    fn load_icon_has_nonzero_dimensions() {
        let icon = load_icon().unwrap();
        assert!(icon.width > 0);
        assert!(icon.height > 0);
    }
}
