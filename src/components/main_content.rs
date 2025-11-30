use dioxus::prelude::*;

const STEVE_FACE: Asset = asset!("/assets/images/steve.png");

#[component]
pub fn MainContent() -> Element {
    rsx! {
        div { class: "flex-1 flex flex-col items-center justify-center",
            img { class: "w-10 h-10", src: STEVE_FACE }
            p { "Log In" }
        }
    }
}