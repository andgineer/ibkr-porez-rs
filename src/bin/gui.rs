#![cfg_attr(windows, windows_subsystem = "windows")]

use eframe::egui;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn window_title() -> String {
    format!("IBKR Porez v{VERSION}")
}

fn log_file_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("ibkr-porez")
        .join("gui.log")
}

fn setup_panic_hook() {
    let log_path = log_file_path();
    std::panic::set_hook(Box::new(move |info| {
        let msg = format!(
            "{}\n{info}\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        );
        let _ = std::fs::create_dir_all(log_path.parent().unwrap());
        let _ = std::fs::write(&log_path, &msg);
        eprintln!("{msg}");
    }));
}

fn log_error(msg: &str) {
    eprintln!("{msg}");
    let log_path = log_file_path();
    let _ = std::fs::create_dir_all(log_path.parent().unwrap());
    let timestamped = format!(
        "{}\n{msg}\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    );
    let _ = std::fs::write(&log_path, &timestamped);
}

fn make_viewport() -> egui::ViewportBuilder {
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1100.0, 700.0])
        .with_min_inner_size([800.0, 500.0]);

    if let Some(icon) = ibkr_porez::gui::icon::load_icon() {
        viewport = viewport.with_icon(std::sync::Arc::new(icon));
    }
    viewport
}

fn wgpu_adapter_selector() -> eframe::egui_wgpu::NativeAdapterSelectorMethod {
    use eframe::wgpu;
    std::sync::Arc::new(
        |adapters: &[wgpu::Adapter], surface: Option<&wgpu::Surface<'_>>| {
            let compatible: Vec<&wgpu::Adapter> = if let Some(surface) = surface {
                adapters
                    .iter()
                    .filter(|a| a.is_surface_supported(surface))
                    .collect()
            } else {
                adapters.iter().collect()
            };

            if compatible.is_empty() {
                return Err("no compatible adapters found".into());
            }

            // Prefer hardware, accept software (WARP) as fallback
            let pick = compatible
                .iter()
                .find(|a| !matches!(a.get_info().device_type, wgpu::DeviceType::Cpu))
                .or(compatible.first())
                .unwrap();

            Ok((*pick).clone())
        },
    )
}

fn make_wgpu_options() -> eframe::egui_wgpu::WgpuConfiguration {
    let wgpu_options = eframe::egui_wgpu::WgpuConfiguration {
        wgpu_setup: eframe::egui_wgpu::WgpuSetup::CreateNew(
            eframe::egui_wgpu::WgpuSetupCreateNew {
                native_adapter_selector: Some(wgpu_adapter_selector()),
                ..Default::default()
            },
        ),
        ..Default::default()
    };
    wgpu_options
}

fn make_native_options(title: &str) -> eframe::NativeOptions {
    let options = eframe::NativeOptions {
        viewport: make_viewport().with_title(title),
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: make_wgpu_options(),
        ..Default::default()
    };
    options
}

fn main() {
    setup_panic_hook();

    let title = window_title();
    let options = make_native_options(&title);

    if let Err(e) = eframe::run_native(
        &title,
        options,
        Box::new(|_cc| Ok(Box::new(ibkr_porez::gui::app::App::new()))),
    ) {
        log_error(&format!("GUI failed to start: {e}"));
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_title_contains_version() {
        assert_eq!(window_title(), format!("IBKR Porez v{VERSION}"));
    }

    #[test]
    fn native_options_use_wgpu_renderer() {
        let options = make_native_options("test");
        assert!(matches!(options.renderer, eframe::Renderer::Wgpu));
    }

    #[test]
    fn native_options_propagate_title() {
        let options = make_native_options("test title");
        assert_eq!(options.viewport.title, Some("test title".into()));
    }

    #[test]
    fn native_options_set_expected_sizes() {
        let options = make_native_options("test");
        assert_eq!(options.viewport.inner_size.unwrap(), [1100.0, 700.0].into());
        assert_eq!(
            options.viewport.min_inner_size.unwrap(),
            [800.0, 500.0].into()
        );
    }
}
