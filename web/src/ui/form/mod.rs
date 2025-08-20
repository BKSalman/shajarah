use dioxus::prelude::*;
#[derive(Props, Clone, PartialEq)]
pub struct FormSectionProps {
    pub title: String,
    pub icon_path: String,
    pub icon_color: Option<String>,
    pub children: Element,
    pub border_bottom: Option<bool>,
}
#[component]
pub fn FormSection(props: FormSectionProps) -> Element {
    let icon_color = props
        .icon_color
        .unwrap_or_else(|| "primary-600".to_string());
    let border_class = if props.border_bottom.unwrap_or(true) {
        "border-b pb-6"
    } else {
        ""
    };
    rsx! {
        div { class: "{border_class}",
            h4 { class: "text-lg font-semibold text-gray-900 mb-4 flex items-center",
                svg {
                    class: "w-5 h-5 ml-2 text-{icon_color}",
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "{props.icon_path}",
                    }
                }
                "{props.title}"
            }
            {props.children}
        }
    }
}
