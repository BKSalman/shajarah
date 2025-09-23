use dioxus::prelude::*;

use crate::modules::{member::server::members, tree::components::node::MemberNode};
use crate::util::use_store_resource;

#[component]
pub fn Tree() -> Element {
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
                            MemberNode {
                                root,
                            }
                        }
                    }
                    None => {
                        rsx! {
                            p { "لا يوجد أعضاء في شجرة العائلة" }
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
                    p { "جاري التحميل..." }
                };
            }
        }
    }
}
