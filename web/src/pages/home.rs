use dioxus::prelude::*;

use crate::{Route, config, modules::tree::components::tree::Tree};

#[component]
fn Sidebar(show_sidebar: Signal<bool>) -> Element {
    rsx! {
        nav {
            class: if show_sidebar() { "translate-x-0" } else { "translate-x-full" },
            class: "flex flex-col fixed right-0 z-30 bg-white h-full w-full sm:w-60 px-6 py-6 space-y-2 shadow-lg transition-transform duration-200 ease-in-out",
            dir: "rtl",

            Link {
                to: Route::Home {},
                class: "px-4 py-3 rounded-lg text-forest-dark font-medium hover:bg-forest-light transition-colors duration-200 cursor-pointer",
                onclick: move |_| show_sidebar.set(false),
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
        div { class: "min-h-screen w-full bg-forest-background", dir: "rtl",
            div { class: "sticky top-0 z-10 bg-white/90 sm:bg-white p-4",
                button {
                    class: "btn btn-primary shadow-md hover:shadow-lg transition-all duration-200",
                    aria_label: "فتح القائمة",
                    onclick: move |_| show_sidebar.toggle(),
                    i { class: "fa-solid fa-bars" }
                }
            }

            Sidebar { show_sidebar }

            main { class: "sm:max-w-6xl mx-auto sm:p-6 space-y-6 sm:space-y-8",

                section { class: "card fade-in text-center py-12 sm:py-16 px-6 sm:px-8",
                    h1 { class: "heading font-bold text-forest-dark text-3xl sm:text-4xl md:text-5xl",
                        if let Some(family_name) = config.family_name.clone() {
                            "عائلة {family_name}"
                        } else {
                            "عائلة فلان"
                        }
                    }
                    if let Some(family_description) = config.family_description {
                        p { class: "mt-4 sm:mt-6 mx-auto max-w-2xl text-forest-primary text-base sm:text-lg md:text-xl leading-relaxed",
                            "{family_description}"
                        }
                    }
                }

                // TODO: Events

                // TODO: Achievements
                // TODO: Remarkable family members
                section { class: "fade-in overflow-hidden", Tree {
                } }
            }
        }
    }
}
