use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    rsx! {
        div {
            // dangerous_inner_html: include_str!("../../assets/dist/index.html")
        }
    }
}
