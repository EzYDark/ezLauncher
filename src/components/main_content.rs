use crate::components::login::modal::LoginModal;
use crate::scripts::game::VersionType;
use crate::state::AUTH;
use dioxus::prelude::*;

const STEVE_FACE: Asset = asset!("/assets/images/steve.png");

#[component]
pub fn MainContent() -> Element {
    let show_login = use_signal(|| false);
    let mut selected_version = use_signal(|| VersionType::Vanilla);

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

                // Version Selector
                select {
                    class: "bg-[var(--background-dark)] text-[var(--text-dark)] p-2 rounded mt-2 mb-2",
                    onchange: move |evt| {
                        match evt.value().as_str() {
                            "Vanilla" => selected_version.set(VersionType::Vanilla),
                            "NeoForge" => selected_version.set(VersionType::NeoForge),
                            _ => {}
                        }
                    },
                    option { value: "Vanilla", "Vanilla 1.21.1" }
                    option { value: "NeoForge", "NeoForge 21.1.65" }
                }

                button {
                    class: "bg-[var(--background-dark)] text-[var(--text-dark)] p-2 px-4 rounded hover:bg-[var(--background-dark)] active:bg-[var(--background-light)] cursor-pointer",
                    onclick: move |_| {
                        let version = selected_version();
                        spawn(async move {
                            if let Some(user) = &AUTH().user {
                                match crate::scripts::game::launch(
                                        user.username.clone(),
                                        user.uuid.clone(),
                                        user.access_token.clone(),
                                        version,
                                    )
                                    .await
                                {
                                    Ok(_) => log::info!("Game launched successfully"),
                                    Err(e) => log::error!("Game launch failed: {:?}", e),
                                }
                            }
                        });
                    },
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