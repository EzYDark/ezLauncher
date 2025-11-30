use dioxus::prelude::*;

use crate::components::title_bar::TitleBar;

#[component]
pub fn MainLayout() -> Element {
    rsx! {
        TitleBar {}
    }
}