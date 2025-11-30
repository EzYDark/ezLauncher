use dioxus::prelude::*;
use dioxus::desktop::tao::window::ResizeDirection;

#[component]
pub fn ResizeBorders() -> Element {
    let window = dioxus_desktop::use_window();

    rsx! {
        // Top
        div {
            class: "absolute -top-[2px] -left-[2px] w-[calc(100%+4px)] h-2 z-50 cursor-n-resize",
            onmousedown: {
                let window = window.clone();
                move |_| {
                    let _ = window.drag_resize_window(ResizeDirection::North);
                }
            },
        }
        // Bottom
        div {
            class: "absolute -bottom-[2px] -left-[2px] w-[calc(100%+4px)] h-2 z-50 cursor-s-resize",
            onmousedown: {
                let window = window.clone();
                move |_| {
                    let _ = window.drag_resize_window(ResizeDirection::South);
                }
            },
        }
        // Left
        div {
            class: "absolute -top-[2px] -left-[2px] h-[calc(100%+4px)] w-2 z-50 cursor-w-resize",
            onmousedown: {
                let window = window.clone();
                move |_| {
                    let _ = window.drag_resize_window(ResizeDirection::West);
                }
            },
        }
        // Right
        div {
            class: "absolute -top-[2px] -right-[2px] h-[calc(100%+4px)] w-2 z-50 cursor-e-resize",
            onmousedown: {
                let window = window.clone();
                move |_| {
                    let _ = window.drag_resize_window(ResizeDirection::East);
                }
            },
        }
        // Corners
        // Top Left
        div {
            class: "absolute -top-[2px] -left-[2px] w-4 h-4 z-51 cursor-nw-resize",
            onmousedown: {
                let window = window.clone();
                move |_| {
                    let _ = window.drag_resize_window(ResizeDirection::NorthWest);
                }
            },
        }
        // Top Right
        div {
            class: "absolute -top-[2px] -right-[2px] w-4 h-4 z-51 cursor-ne-resize",
            onmousedown: {
                let window = window.clone();
                move |_| {
                    let _ = window.drag_resize_window(ResizeDirection::NorthEast);
                }
            },
        }
        // Bottom Left
        div {
            class: "absolute -bottom-[2px] -left-[2px] w-4 h-4 z-51 cursor-sw-resize",
            onmousedown: {
                let window = window.clone();
                move |_| {
                    let _ = window.drag_resize_window(ResizeDirection::SouthWest);
                }
            },
        }
        // Bottom Right
        div {
            class: "absolute -bottom-[2px] -right-[2px] w-4 h-4 z-51 cursor-se-resize",
            onmousedown: {
                let window = window.clone();
                move |_| {
                    let _ = window.drag_resize_window(ResizeDirection::SouthEast);
                }
            },
        }
    }
}
