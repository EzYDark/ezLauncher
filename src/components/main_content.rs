use crate::components::login::modal::LoginModal;
use crate::state::AUTH;
use dioxus::prelude::*;

const STEVE_FACE: Asset = asset!("/assets/images/steve.png");

#[component]
pub fn MainContent() -> Element {
    let show_login = use_signal(|| false);

    rsx! {
        div { class: "relative flex-1 flex flex-col items-center justify-center gap-2",
            if show_login() {
                LoginModal { show_modal: show_login }
            }

            if let Some(user) = &AUTH().user {
                div { class: "w-24 h-24 relative overflow-hidden",
                    // Inner Face
                    div {
                        class: "absolute inset-0",
                        style: "background-image: url('{user.skin_url}'); background-size: 800%; background-position: 14.286% 14.286%; image-rendering: pixelated;",
                    }
                    // Outer Face
                    div {
                        class: "absolute inset-0",
                        style: "background-image: url('{user.skin_url}'); background-size: 800%; background-position: 71.429% 14.286%; image-rendering: pixelated;",
                    }
                }
                p { class: "text-xl font-bold", "{user.username}" }
                p { class: "text-sm text-gray-500", "UUID: {user.uuid}" }
                button {
                    class: "bg-[var(--background-dark)] text-[var(--text-dark)] p-2 px-4 rounded hover:bg-[var(--background-dark)] active:bg-[var(--background-light)] cursor-pointer",
                    onclick: move |_| async {},
                    "Play"
                }
            } else {
                img { class: "w-10 h-10", src: STEVE_FACE }
                p { "Not Logged In" }
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
}