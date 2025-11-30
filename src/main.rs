use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};

mod components;
mod scripts;
mod consts;

use crate::components::main_layout::MainLayout;
use crate::scripts::window_size_center::set_window_size_and_center;

fn main() {
    dioxus::LaunchBuilder::new()
        .with_cfg(desktop! {
            Config::new().with_window(
                WindowBuilder::new()
                    .with_always_on_top(false)
                    .with_decorations(false)
                    .with_resizable(false)
                    .with_title("ezLauncher")
            )
        })
        .launch(App);
}

#[component]
fn App() -> Element {
    set_window_size_and_center();

    rsx! {
        document::Link { rel: "icon", href: consts::FAVICON }
        document::Link { rel: "stylesheet", href: consts::MAIN_CSS }
        document::Link { rel: "stylesheet", href: consts::TAILWIND_CSS }
        MainLayout {}
    }
}