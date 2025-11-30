use dioxus::prelude::*;

const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

const PALETTE_CSS: Asset = asset!("/assets/css/palette.css");
const CUSTOM_SCROLLBAR_CSS: Asset = asset!("/assets/css/custom_scrollbar.css");
const FONTS_CSS: Asset = asset!("/assets/css/fonts.css");

#[component]
pub fn LoadCSS() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Link { rel: "stylesheet", href: PALETTE_CSS }
        document::Link { rel: "stylesheet", href: CUSTOM_SCROLLBAR_CSS }
        document::Link { rel: "stylesheet", href: FONTS_CSS }
    }
}