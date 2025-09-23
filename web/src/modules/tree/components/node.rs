use dioxus::prelude::*;

use crate::modules::member::types::{MemberResponse, MemberResponseStoreExt};

#[component]
pub fn MemberNode(root: Store<MemberResponse>) -> Element {
    let mut is_collapsed = use_signal(|| true);

    rsx! {
        div {
            class: "flex flex-col justify-start items-center",
            div {
                class: "flex flex-col justify-center items-center",
                button {
                    ondoubleclick: move |_| {
                        let mut is_collapsed = is_collapsed.write();
                        *is_collapsed = !*is_collapsed;
                    },
                    onclick: move |_| {
                    },
                    img {
                        width: "30px",
                        height: "30px",
                        border_radius: "50%",
                        src: "https://placehold.co/300x300",
                    },
                }
                p { "{root.name()}" }
                if !root.children().is_empty() {
                    if is_collapsed() {
                        p {
                            position: "relative",
                            bottom: "30px",
                            right: "30px",
                            ">"
                        }
                    } else {
                        p {
                            position: "relative",
                            bottom: "30px",
                            right: "30px",
                            "V"
                        }
                    }
                }
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
