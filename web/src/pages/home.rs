use dioxus::prelude::*;

use crate::{config, modules::tree::components::tree::Tree};

#[component]
pub fn Home() -> Element {
    let config = use_context::<config::client::Config>();

    rsx! {
        div {
            class: "h-full w-full",
            div {
                class: "h-80",
                h1 {
                    class: "heading font-bold text-forest-dark text-center",
                    if let Some(family_name) = config.family_name {
                        "عائلة {family_name}"
                    } else {
                        "العائلة فلان"
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
