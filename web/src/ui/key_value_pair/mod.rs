use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
}

#[derive(Props, Clone, PartialEq)]
pub struct KeyValueInputProps {
    pub pairs: Vec<KeyValuePair>,
    pub on_pairs_change: EventHandler<Vec<KeyValuePair>>,
    pub key_placeholder: Option<String>,
    pub value_placeholder: Option<String>,
}

#[component]
pub fn KeyValueInput(props: KeyValueInputProps) -> Element {
    let key_placeholder = props
        .key_placeholder
        .unwrap_or_else(|| "المفتاح".to_string());
    let value_placeholder = props
        .value_placeholder
        .unwrap_or_else(|| "القيمة".to_string());

    let pairs = props.pairs.clone();
    let on_pairs_change = props.on_pairs_change;

    rsx! {
        div { class: "space-y-3",
            for (index, pair) in pairs.iter().enumerate() {
                div { class: "flex gap-3 items-center",
                    div { class: "flex-1",
                        input {
                            r#type: "text",
                            value: "{pair.key}",
                            placeholder: "{key_placeholder}",
                            class: "form-input w-full",
                            oninput: {
                                let pairs = pairs.clone();
                                let on_pairs_change = on_pairs_change;
                                move |evt| {
                                    let mut new_pairs = pairs.clone();
                                    new_pairs[index].key = evt.value();
                                    on_pairs_change.call(new_pairs);
                                }
                            }
                        }
                    }
                    div { class: "flex-1",
                        input {
                            r#type: "text",
                            value: "{pair.value}",
                            placeholder: "{value_placeholder}",
                            class: "form-input w-full",
                            oninput: {
                                let pairs = pairs.clone();
                                let on_pairs_change = on_pairs_change;
                                move |evt| {
                                    let mut new_pairs = pairs.clone();
                                    new_pairs[index].value = evt.value();
                                    on_pairs_change.call(new_pairs);
                                }
                            }
                        }
                    }
                    button {
                        r#type: "button",
                        class: "btn btn-danger btn-sm",
                        onclick: {
                            let pairs = pairs.clone();
                            let on_pairs_change = on_pairs_change;
                            move |_| {
                                let mut new_pairs = pairs.clone();
                                new_pairs.remove(index);
                                on_pairs_change.call(new_pairs);
                            }
                        },
                        "حذف"
                    }
                }
            }
            button {
                r#type: "button",
                class: "btn btn-outline btn-sm",
                onclick: {
                    let pairs = pairs.clone();
                    let on_pairs_change = on_pairs_change;
                    move |_| {
                        let mut new_pairs = pairs.clone();
                        new_pairs.push(KeyValuePair {
                            key: String::new(),
                            value: String::new(),
                        });
                        on_pairs_change.call(new_pairs);
                    }
                },
                svg {
                    class: "w-4 h-4",
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M12 6v6m0 0v6m0-6h6m-6 0H6"
                    }
                }
                "إضافة معلومة"
            }
        }
    }
}
