use dioxus::prelude::*;
#[derive(Props, Clone, PartialEq)]
pub struct ModalProps {
    pub show: bool,
    pub title: String,
    pub on_close: EventHandler<()>,
    pub max_width: Option<String>,
    pub children: Element,
}
#[component]
pub fn Modal(props: ModalProps) -> Element {
    let max_width_class = props.max_width.unwrap_or_else(|| "4xl".to_string());
    if !props.show {
        return rsx! {
            div {}
        };
    }
    rsx! {
        div {
            class: "fixed inset-0 flex items-center justify-center p-0 md:p-4 z-50",
            style: "background: rgba(255, 255, 255, 0.2); backdrop-filter: blur(2px);",
            onclick: move |_| props.on_close.call(()),
            div {
                class: "bg-white rounded-none md:rounded-xl w-full md:max-w-{max_width_class} h-full md:h-auto max-h-screen overflow-y-auto shadow-2xl md:border md:border-gray-200 transform transition ease-out duration-200",
                onclick: move |e| e.stop_propagation(),
                div { class: "card-header flex justify-between items-center",
                    h3 { class: "text-xl font-bold text-gray-900", "{props.title}" }
                    button {
                        class: "text-gray-400 hover:text-gray-600",
                        onclick: move |_| props.on_close.call(()),
                        svg {
                            class: "w-6 h-6",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M6 18L18 6M6 6l12 12",
                            }
                        }
                    }
                }
                div { class: "card-body", {props.children} }
            }
        }
    }
}
