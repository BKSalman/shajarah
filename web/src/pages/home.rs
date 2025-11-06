use dioxus::prelude::*;

use crate::{Route, config, modules::tree::components::tree::Tree};

#[component]
fn Sidebar(show_sidebar: Signal<bool>) -> Element {
    rsx! {
        div {
            class: if !show_sidebar() {
                "hidden"
            },
            class: "flex flex-col fixed right-0 z-1 bg-(--color-forest-background) h-full px-8 py-5 space-y-10",

            Link { to: Route::Home {}, "الصفحة الرئيسية" }
        }
    }
}

#[component]
pub fn Home() -> Element {
    let config = use_context::<config::client::Config>();
    let mut show_sidebar = use_signal(|| true);

    rsx! {
        div {
            class: "h-full w-full p-3",
            dir: "rtl",
            div {
                button {
                    class: "btn p-5",
                    onclick: move |_| {
                        show_sidebar.with_mut(|show_sidebar| {
                            *show_sidebar = !*show_sidebar;
                        });
                    },
                    "<"
                }
                Sidebar { show_sidebar }
            }
            div {
                class: "p-10",
                h1 {
                    class: "heading font-bold text-forest-dark text-center",
                    if let Some(family_name) = config.family_name {
                        "عائلة {family_name}"
                    } else {
                        "عائلة فلان"
                    }
                }
            }
            if let Some(family_description) = config.family_description {
                div {
                    class: "text-center p-10",
                    h3 {
                        "{family_description}"
                    }
                }
            }
            // TODO: Achievements
            // TODO: Events
            // TODO: Remarkable family members
            Tree {}
        }
    }
}
