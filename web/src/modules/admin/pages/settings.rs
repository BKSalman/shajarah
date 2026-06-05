use dioxus::prelude::*;

use crate::modules::admin::pages::Sidebar;

#[component]
pub fn AdminSettings() -> Element {
    let mut show_sidebar = use_signal(|| false);

    rsx! {
        div {
            dir: "rtl",
            class: "sticky top-0 right-0 z-10 md:hidden! bg-white/90 p-4",
            button {
                class: "btn btn-primary shadow-md hover:shadow-lg transition-all duration-200",
                onclick: move |_| {
                    show_sidebar.toggle();
                },
                i { class: "fa-solid fa-bars" }
            }
        }
        div {
            dir: "rtl",
            class: "flex h-full w-full",

            Sidebar { show_sidebar }

            div {
                class: "w-full",
            }
        }
    }
}
