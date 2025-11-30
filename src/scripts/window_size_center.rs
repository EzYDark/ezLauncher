use dioxus::prelude::*;
use dioxus_desktop::use_window;
use dioxus_desktop::wry::dpi::{PhysicalPosition, PhysicalSize};

/// Set app window size to 60% of the screen size and center it
pub fn set_window_size_and_center() {
    let window = use_window();

    use_effect(move || {
        if let Some(monitor) = window.current_monitor() {
            let size = monitor.size();
            // Set window size to 60% of the screen size
            let width = size.width as f64 * 0.6;
            let height = size.height as f64 * 0.6;
            window.set_inner_size(PhysicalSize::new(width, height));

            // Center the window
            let x = size.width as f64 * 0.2;
            let y = size.height as f64 * 0.2;
            window.set_outer_position(PhysicalPosition::new(x, y));
        }
    });
}