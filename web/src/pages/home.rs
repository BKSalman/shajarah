use dioxus::prelude::*;

use crate::{
    Route, config,
    modules::{settings::types::Settings, tree::components::tree::Tree},
};

#[component]
fn Sidebar(show_sidebar: Signal<bool>) -> Element {
    let settings = use_context::<Settings>();

    rsx! {
        nav {
            class: if show_sidebar() { "translate-x-0" } else { "translate-x-full lg:translate-x-0" },
            class: "flex flex-col max-lg:fixed max-lg:inset-0 lg:sticky lg:top-0 lg:h-screen lg:shrink-0 right-0 z-30 bg-white w-full lg:w-auto px-4 lg:p-8 py-4 space-y-2 text-nowrap shadow-lg transition-transform duration-200 ease-in-out",
            dir: "rtl",

            button {
                class: "btn btn-primary self-start mb-2 shadow-md hover:shadow-lg transition-all duration-200 lg:hidden",
                aria_label: "إغلاق القائمة",
                onclick: move |_| show_sidebar.set(false),
                i { class: "fa-solid fa-xmark" }
            }

            div { class: "flex flex-col p-4 lg:p-0 text-center",
                Link {
                    to: Route::Home {},
                    class: "px-2 py-3 rounded-lg text-3xl lg:text-base text-forest-dark font-medium hover:bg-forest-light transition-colors duration-200 cursor-pointer",
                    onclick: move |_| show_sidebar.set(false),
                    "الصفحة الرئيسية"
                }

                if settings.add_page_enabled {
                    Link {
                        to: Route::AddMemberRequest {},
                        class: "px-2 py-3 rounded-lg text-3xl lg:text-base text-forest-dark font-medium hover:bg-forest-light transition-colors duration-200 cursor-pointer",
                        onclick: move |_| show_sidebar.set(false),
                        "طلب الإضافة"
                    }
                }
            }
        }
    }
}

#[component]
pub fn HomeLayout() -> Element {
    let mut show_sidebar = use_signal(|| false);

    rsx! {
        div {
            class: "min-h-screen w-full bg-forest-background flex",
            dir: "rtl",

            Sidebar { show_sidebar }

            div { class: "flex-1 min-w-0",
                div { class: "sticky top-0 z-10 bg-white/90 sm:bg-white p-4 lg:hidden",
                    button {
                        class: "btn btn-primary shadow-md hover:shadow-lg transition-all duration-200",
                        aria_label: "فتح القائمة",
                        onclick: move |_| show_sidebar.toggle(),
                        i { class: "fa-solid fa-bars" }
                    }
                }

                main { class: "sm:max-w-6xl mx-auto sm:p-6 space-y-6 sm:space-y-8", Outlet::<Route> {} }
            }
        }
    }
}

#[component]
pub fn Home() -> Element {
    let config = use_context::<config::client::Config>();

    rsx! {
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
