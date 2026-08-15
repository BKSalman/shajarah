use crate::{
    components::combobox::{Combobox, ComboboxOption},
    modules::member::types::{Gender, MemberUnauthorizedResponseFlat},
};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct MemberPickerProps {
    pub members: Vec<MemberUnauthorizedResponseFlat>,
    #[props(default = None)]
    pub exclude_id: Option<i64>,
    #[props(default = None)]
    pub required_gender: Option<Gender>,
    #[props(default = String::new())]
    pub initial_label: String,
    #[props(default = "ابحث بالاسم...".to_string())]
    pub placeholder: String,
    pub on_select: EventHandler<Option<i64>>,
}

#[component]
pub fn MemberPicker(props: MemberPickerProps) -> Element {
    let candidates: Vec<MemberUnauthorizedResponseFlat> = props
        .members
        .iter()
        .filter(|m| props.exclude_id != Some(m.id))
        .filter(|m| props.required_gender.is_none_or(|g| g == m.gender))
        .cloned()
        .collect();

    let mut options = use_signal(|| {
        let has_duplicate_labels = candidates.iter().any(|m| {
            candidates
                .iter()
                .filter(|o| o.full_name == m.full_name)
                .count()
                > 1
        });
        candidates
            .iter()
            .enumerate()
            .map(|(index, member)| {
                let label = if has_duplicate_labels {
                    format!("{} (#{})", member.full_name, member.id)
                } else {
                    member.full_name.clone()
                };
                ComboboxOption {
                    index,
                    value: member.id,
                    label,
                }
            })
            .collect()
    });

    rsx! {
        Combobox {
            default_value: props.initial_label.clone(),
            placeholder: props.placeholder.clone(),
            options: options(),
            on_search: move |query: String| {
                if query.trim().is_empty() {
                    let has_duplicate_labels = candidates
                        .iter()
                        .any(|m| {
                            candidates.iter().filter(|o| o.full_name == m.full_name).count() > 1
                        });
                    options
                        .set(
                            candidates
                                .iter()
                                .enumerate()
                                .map(|(index, member)| {
                                    let label = if has_duplicate_labels {
                                        format!("{} (#{})", member.full_name, member.id)
                                    } else {
                                        member.full_name.clone()
                                    };
                                    ComboboxOption {
                                        index,
                                        value: member.id,
                                        label,
                                    }
                                })
                                .collect(),
                        );
                    props.on_select.call(None);
                    return;
                }

                let query = query.trim().to_lowercase();
                let matches: Vec<MemberUnauthorizedResponseFlat> = candidates
                    .iter()
                    .filter(|m| m.full_name.to_lowercase().contains(&query))
                    .cloned()
                    .collect();
                let has_duplicate_labels = matches
                    .iter()
                    .any(|m| {
                        matches.iter().filter(|o| o.full_name == m.full_name).count() > 1
                    });
                options
                    .set(
                        matches
                            .iter()
                            .enumerate()
                            .map(|(index, member)| {
                                let label = if has_duplicate_labels {
                                    format!("{} (#{})", member.full_name, member.id)
                                } else {
                                    member.full_name.clone()
                                };
                                ComboboxOption {
                                    index,
                                    value: member.id,
                                    label,
                                }
                            })
                            .collect(),
                    );
            },
            on_select: move |option: ComboboxOption<i64>| {
                props.on_select.call(Some(option.value));
            },
        }
    }
}
