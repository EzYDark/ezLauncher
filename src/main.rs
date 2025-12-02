// #![windows_subsystem = "windows"]

use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};

mod components;
mod consts;
mod css;
mod fonts;
mod logger;
mod scripts;
mod secrets;
mod state;

use crate::components::main_layout::MainLayout;
use crate::css::LoadCSS;
use crate::fonts::LoadFonts;
use crate::scripts::window_size_center::set_window_size_and_center;

fn main() {
    logger::init().unwrap();
    dioxus::LaunchBuilder::new()
        .with_cfg(desktop! {
            Config::new().with_window(
                WindowBuilder::new()
                    .with_always_on_top(false)
                    .with_decorations(false)
                    .with_resizable(true)
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
        LoadCSS {}
        LoadFonts {}
        MainLayout {}
    }
}
