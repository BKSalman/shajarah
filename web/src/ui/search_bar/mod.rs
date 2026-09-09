use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct SearchBarProps {
    pub value: String,
    pub on_input: EventHandler<String>,
    #[props(default)]
    pub placeholder: Option<String>,
    /// Number of matches, shown as a hint while a query is active.
    #[props(default)]
    pub result_count: Option<usize>,
}

#[component]
pub fn SearchBar(props: SearchBarProps) -> Element {
    let placeholder = props
        .placeholder
        .clone()
        .unwrap_or_else(|| "ابحث...".to_string());

    let value = props.value.clone();
    let searching = !value.trim().is_empty();

    rsx! {
        div { class: "mb-4",
            div { class: "relative",
                span { class: "absolute inset-y-0 start-0 flex items-center ps-3 text-gray-400 pointer-events-none",
                    i { class: "fa-solid fa-magnifying-glass" }
                }
                input {
                    r#type: "text",
                    class: "input w-full ps-10 pe-10",
                    placeholder,
                    value: "{value}",
                    oninput: move |evt| props.on_input.call(evt.value()),
                }
                if searching {
                    button {
                        r#type: "button",
                        class: "absolute inset-y-0 end-0 flex items-center pe-3 text-gray-400 hover:text-gray-600",
                        title: "مسح البحث",
                        onclick: move |_| props.on_input.call(String::new()),
                        i { class: "fa-solid fa-xmark" }
                    }
                }
            }

            if searching {
                if let Some(count) = props.result_count {
                    p { class: "text-sm text-gray-600 mt-1 arabic-text", "{count} نتيجة" }
                }
            }
        }
    }
}
