use dioxus::prelude::*;

const EGUI: &str = include_str!("../../../../dist/index.html");

#[component]
pub fn Tree() -> Element {
    let mut fullscreen_tree = use_signal(|| false);

    rsx! {
        div {
            dir: "rtl",
            class: "flex flex-col w-full",
            class: if fullscreen_tree() {
                       "h-full absolute top-0 left-0"
                   } else {
                       "h-80 p-4 border border-gray-300"
                   },
            h3 { "شجرة العائلة" }
            button {
                class: "btn btn-primary",
                onclick: move |_| {
                    fullscreen_tree.with_mut(|f| *f = !*f);
                },
                if fullscreen_tree() { "صغر الشجرة" } else { "كبر الشجرة" }
            }
            div {
                flex: 1,
                height: "100%",
                dangerous_inner_html: EGUI,
            }
        }
    }
}
