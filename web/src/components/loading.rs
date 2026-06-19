use dioxus::prelude::*;

/// A centered, full-page loading spinner used while a page's data is being
/// fetched (e.g. the private-tree auth guard).
#[component]
pub fn FullPageLoading() -> Element {
    rsx! {
        div { class: "loading-overlay",
            div { class: "spinner" }
        }
    }
}
