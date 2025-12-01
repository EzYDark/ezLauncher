use dioxus::prelude::*;

#[component]
pub fn LoginModal(show_modal: Signal<bool>) -> Element {
    rsx! {
        // Background
        div {
            class: "absolute w-full h-full bg-[var(--background-darker)]/50 flex items-center justify-center",
            onclick: {
                let mut show_modal = show_modal.clone();
                move |_| show_modal.set(false)
            },
            // Modal
            div {
                class: "bg-[var(--background)] p-5 py-20 rounded-lg w-1/3 h-1/2 flex flex-col items-center justify-between",
                onclick: move |e| e.stop_propagation(),
                div { class: "flex flex-col gap-2 items-center",
                    h2 { "Ely.by Log In" }
                    p { class: "text-center",
                        "You will be redirected to Ely.by to authorize this application and log in to your account."
                    }
                }
                button { class: "border border-[var(--background-darker)] bg-[var(--background-dark)] text-[var(--text-dark)] p-2 px-4 rounded hover:bg-[var(--background-dark)] active:bg-[var(--background-light)] cursor-pointer",
                    "Log In"
                }
            }
        }
    }
}