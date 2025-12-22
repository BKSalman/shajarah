use dioxus::prelude::*;

use crate::{Route, config, modules::tree::components::tree::Tree};

#[component]
fn Sidebar(show_sidebar: Signal<bool>) -> Element {
    rsx! {
        div {
            class: if !show_sidebar() {
                "translate-x-full"
            } else {
                "translate-x-0"
            },
            class: "flex flex-col fixed right-0 top-16 z-10 bg-white h-full px-8 py-5 space-y-4 shadow-lg rounded-l-lg transition-transform duration-150 ease-in-out",

            Link {
                to: Route::Home {},
                class: "px-4 py-3 rounded-lg text-forest-dark hover:bg-forest-light transition-colors duration-200 cursor-pointer",
                "الصفحة الرئيسية"
            }
        }
    }
}

#[component]
pub fn Home() -> Element {
    let config = use_context::<config::client::Config>();
    let mut show_sidebar = use_signal(|| false);

    rsx! {
        div {
            class: "min-h-screen w-full bg-forest-background",
            dir: "rtl",
            div {
                button {
                    class: "btn btn-primary fixed top-5 right-5 z-20 shadow-md hover:shadow-lg transition-all duration-200",
                    onclick: move |_| {
                        show_sidebar.with_mut(|show_sidebar| {
                            *show_sidebar = !*show_sidebar;
                        });
                    },
                    if show_sidebar() {
                        "☰"
                    } else {
                        "☰"
                    }
                }
                Sidebar { show_sidebar }
            }
            div {
                class: "max-w-6xl mx-auto lg:px-4 lg:py-12 space-y-8",

                // Family name card
                div {
                    class: "card fade-in text-center py-12 px-8",
                    h1 {
                        class: "heading font-bold text-forest-dark text-4xl md:text-5xl",
                        if let Some(family_name) = config.family_name {
                            "عائلة {family_name}"
                        } else {
                            "عائلة فلان"
                        }
                    }
                }

                // Family description card
                if let Some(family_description) = config.family_description {
                    div {
                        class: "card fade-in text-center py-8 px-6",
                        h3 {
                            class: "text-forest-dark text-lg md:text-xl leading-relaxed",
                            "{family_description}"
                        }
                    }
                }

                // TODO: Events

                // TODO: Achievements
                // TODO: Remarkable family members

                // Tree section
                div {
                    class: "fade-in",
                    Tree {}
                }
            }
        }
    }
}
