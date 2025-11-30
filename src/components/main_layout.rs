use dioxus::prelude::*;

use crate::components::{resize_borders::ResizeBorders, title_bar::TitleBar};

#[component]
pub fn MainLayout() -> Element {
    rsx! {
        div { class: "relative w-screen h-screen border border-[var(--background-dark)] flex flex-col",
            ResizeBorders {}
            TitleBar {}
        }
    }
}