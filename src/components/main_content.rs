use crate::components::login_modal::LoginModal;
use dioxus::prelude::*;

const STEVE_FACE: Asset = asset!("/assets/images/steve.png");

#[component]
pub fn MainContent() -> Element {
    let show_login = use_signal(|| true);

    rsx! {
        div { class: "relative flex-1 flex flex-col items-center justify-center gap-2",
            if show_login() {
                LoginModal { show_modal: show_login }
            }

            img { class: "w-10 h-10", src: STEVE_FACE }
            p { "Log In" }
            button {
                class: "bg-[var(--background-dark)] text-[var(--text-dark)] p-2 rounded hover:bg-[var(--background-dark)] active:bg-[var(--background-light)] cursor-pointer",
                onclick: move |_| {
                    let mut show_login = show_login.clone();
                    show_login.set(true);
                },
                "Log In"
            }
        }
    }
}