use crate::{
    components::combobox::{Combobox, ComboboxOption},
    modules::member::types::{Gender, MemberResponseFlat},
};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct MemberPickerProps {
    pub members: Vec<MemberResponseFlat>,
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

fn member_label(member: &MemberResponseFlat) -> String {
    format!("{} {}", member.name, member.last_name)
}

#[component]
pub fn MemberPicker(props: MemberPickerProps) -> Element {
    let mut options = use_signal(Vec::<ComboboxOption<i64>>::new);

    let candidates: Vec<MemberResponseFlat> = props
        .members
        .iter()
        .filter(|m| props.exclude_id != Some(m.id))
        .filter(|m| props.required_gender.is_none_or(|g| g == m.gender))
        .cloned()
        .collect();

    rsx! {
        Combobox {
            default_value: props.initial_label.clone(),
            placeholder: props.placeholder.clone(),
            options: options(),
            on_search: move |query: String| {
                if query.trim().is_empty() {
                    options.set(Vec::new());
                    props.on_select.call(None);
                    return;
                }

                let query = query.trim().to_lowercase();
                let matches: Vec<MemberResponseFlat> = candidates
                    .iter()
                    .filter(|m| member_label(m).to_lowercase().contains(&query))
                    .cloned()
                    .collect();
                let has_duplicate_labels = matches
                    .iter()
                    .any(|m| matches.iter().filter(|o| member_label(o) == member_label(m)).count() > 1);

                options
                    .set(
                        matches
                            .iter()
                            .enumerate()
                            .map(|(index, member)| {
                                let label = if has_duplicate_labels {
                                    format!("{} (#{})", member_label(member), member.id)
                                } else {
                                    member_label(member)
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
