use dioxus::prelude::*;

use crate::modules::tree::types::Position;
use crate::modules::{member::server::members, tree::components::node::MemberNode};
use crate::util::use_store_resource;

#[component]
pub fn Tree(offset: ReadSignal<Position>, zoom: ReadSignal<f64>) -> Element {
    let members_resource = use_store_resource(members);

    let members = members_resource
        .transpose()
        .map(|s| s.transpose().map(|s| s.transpose()));

    rsx! {
        match members {
            Some(Ok(root)) => {
                match root {
                    Some(root) => {
                        rsx! {
                            div {
                                position: "absolute",
                                left: "{offset().x}px",
                                top: "{offset().y}px",
                                transform: "translate(-50%, -50%) scale({zoom()})",
                                display: "flex",
                                MemberNode {
                                    root,
                                }
                            }
                        }
                    }
                    None => {
                        rsx! {
                            p {
                                dir: "rtl",
                                "لا يوجد أعضاء في شجرة العائلة"
                            }
                        }
                    }
                }
            }
            Some(Err(e)) => {
                return rsx! {
                    p { "Error: {e}" }
                };
            }
            None => {
                return rsx! {
                    p {
                        dir: "rtl",
                        "جاري التحميل..."
                    }
                };
            }
        }
    }
}
