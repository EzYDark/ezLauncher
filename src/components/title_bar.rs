use dioxus::prelude::*;
use dioxus_free_icons::{Icon, icons::ld_icons::{LdMaximize2, LdMinus, LdX}};

#[component]
pub fn TitleBar() -> Element {
    let window = dioxus_desktop::use_window();

    rsx! {
        div {
            class: "w-full h-8 bg-[var(--background-dark)] border border-[var(--background-darker)] pl-2 select-none",
            onmousedown: {
                let window = window.clone();
                move |_| window.drag()
            },
            ondoubleclick: {
                let window = window.clone();
                move |_| window.set_maximized(!window.is_maximized())
            },
            div { class: "flex items-center justify-between w-full h-full",
                p { class: "text-[var(--background-lighter)] font-lilex text-sm",
                    "ezLauncher"
                }
                div { class: "flex items-center",
                    button {
                        class: "group w-8 h-8 hover:bg-[var(--grey)] active:bg-[var(--grey-dark)] flex items-center justify-center",
                        onmousedown: |e| e.stop_propagation(),
                        ondoubleclick: |e| e.stop_propagation(),
                        onclick: {
                            let window = window.clone();
                            move |_| window.set_minimized(true)
                        },
                        Icon {
                            icon: LdMinus,
                            class: "w-1/2 h-1/2 text-[var(--background-lighter)] group-hover:text-[var(--foreground)] stroke-1",
                        }
                    }
                    button {
                        class: "group w-8 h-8 hover:bg-[var(--yellow)] active:bg-[var(--yellow-dark)] flex items-center justify-center",
                        onmousedown: |e| e.stop_propagation(),
                        ondoubleclick: |e| e.stop_propagation(),
                        onclick: {
                            let window = window.clone();
                            move |_| window.set_maximized(!window.is_maximized())
                        },
                        Icon {
                            icon: LdMaximize2,
                            class: "w-2/5 h-2/5 text-[var(--background-lighter)] group-hover:text-[var(--background-darker)] stroke-1",
                        }
                    }
                    button {
                        class: "group w-8 h-8 hover:bg-[var(--red)] active:bg-[var(--red-dark)] flex items-center justify-center",
                        onmousedown: |e| e.stop_propagation(),
                        ondoubleclick: |e| e.stop_propagation(),
                        onclick: {
                            let window = window.clone();
                            move |_| window.close()
                        },
                        Icon {
                            icon: LdX,
                            class: "w-1/2 h-1/2 text-[var(--background-lighter)] group-hover:text-[var(--foreground)] stroke-1",
                        }
                    }
                }
            }
        }
    }
}