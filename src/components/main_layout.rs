use dioxus::prelude::*;

use crate::components::title_bar::TitleBar;

#[component]
pub fn MainLayout() -> Element {
    rsx! {
        div { class: "w-screen h-screen border border-[var(--background-dark)] flex flex-col",
            TitleBar {}
        }
    }
}