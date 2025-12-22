use dioxus::prelude::*;

#[component]
pub fn Tree() -> Element {
    let mut fullscreen_tree = use_signal(|| false);

    #[cfg(target_arch = "wasm32")]
    spawn(async move {
        gui::run_eframe();
    });

    rsx! {
        div {
            class: "flex flex-col items-center border border-gray-300 p-8",
            dir: "rtl",
            h3 { "شجرة العائلة" }
            div {
                dir: "rtl",
                class: "flex flex-col w-full",
                class: if fullscreen_tree() {
                           "h-full absolute top-0 left-0 z-100 bg-white"
                       } else {
                           "h-200 p-4"
                       },
                button {
                    class: "btn btn-primary",
                    onclick: move |e| {
                        fullscreen_tree.with_mut(|f| *f = !*f);
                    },
                    if fullscreen_tree() { "صغر الشجرة" } else { "كبر الشجرة" }
                }
                canvas {
                    id: "canvas",
                    class: "w-full h-full",
                    flex: 1,
                    touch_action: "manipulation",
                    width: "100%",
                    height: "100%",
                }
            }
        }
    }
}
