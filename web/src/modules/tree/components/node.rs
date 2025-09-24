use dioxus::prelude::*;

use crate::modules::member::types::{MemberResponse, MemberResponseStoreExt};

#[component]
pub fn MemberNode(root: Store<MemberResponse>) -> Element {
    let mut is_collapsed = use_signal(|| true);

    rsx! {
        div {
            flex_shrink: "0",
            class: "flex flex-col justify-start items-center",
            div {
                class: "flex flex-col justify-center items-center",
                img {
                    width: "30px",
                    height: "30px",
                    border_radius: "50%",
                    src: "https://placehold.co/300x300",
                },
                if !root.children().is_empty() {
                    button {
                        onclick: move |_| {
                            let mut is_collapsed = is_collapsed.write();
                            *is_collapsed = !*is_collapsed;
                        },
                        position: "relative",
                        bottom: "30px",
                        right: "30px",
                        if is_collapsed() { "▶" } else { "▼" }
                    }
                }
                p { "{root.name()}" }
            }
            if !is_collapsed() {
                div {
                    class: "flex gap-6",
                    for node in root.children().iter() {
                        MemberNode {
                            root: node,
                        }
                    }
                }
            }
        }
    }
}
