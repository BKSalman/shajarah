use crate::{
    modules::{
        admin::types::{MemberFormData, MemberFormDataStoreExt},
        member::types::{Gender, MemberResponseFlat},
    },
    ui::{
        form::FormSection,
        key_value_pair::{KeyValueInput, KeyValuePair},
        modal::Modal,
    },
};
use dioxus::{fullstack::FileStream, prelude::*};

use crate::modules::member::components::member_picker::MemberPicker;

#[derive(Props, Clone, PartialEq)]
pub struct AddMemberModalProps {
    pub show: bool,
    pub members: Vec<MemberResponseFlat>,
    pub on_close: EventHandler<()>,
    pub on_submit: EventHandler<(MemberFormData, Option<FileStream>)>,
}

#[component]
pub fn AddMemberModal(props: AddMemberModalProps) -> Element {
    let mut form_data = use_store(MemberFormData::default);
    let mut image = use_signal(|| None::<FileStream>);

    rsx! {
        Modal {
            show: props.show,
            title: "إضافة عضو جديد".to_string(),
            max_width: Some("4xl".to_string()),
            on_close: move |_| props.on_close.call(()),
            form {
                id: "add-member-form",
                class: "space-y-6",
                autocomplete: "off",
                onsubmit: move |evt| {
                    evt.prevent_default();
                    props.on_submit.call((form_data(), image.take()));
                    form_data.set(MemberFormData::default());
                },
                FormSection {
                    title: "المعلومات الأساسية".to_string(),
                    icon_path: "M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z".to_string(),
                    icon_color: Some("primary-600".to_string()),
                    div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                        div { class: "form-group",
                            label { class: "form-label",
                                "الاسم الأول "
                                span { class: "text-red-500", "*" }
                            }
                            input {
                                dir: "auto",
                                name: "name",
                                r#type: "text",
                                required: true,
                                class: "input",
                                placeholder: "ادخل الاسم الأول",
                                value: "{form_data().name}",
                                oninput: move |evt| {
                                    form_data.name().set(evt.value());
                                },
                            }
                        }
                        div { class: "form-group",
                            label { class: "form-label",
                                "الاسم الأخير "
                                span { class: "text-red-500", "*" }
                            }
                            input {
                                dir: "auto",
                                name: "last_name",
                                r#type: "text",
                                required: true,
                                class: "input",
                                placeholder: "ادخل الاسم الأخير",
                                value: "{form_data().last_name}",
                                oninput: move |evt| {
                                    form_data.last_name().set(evt.value());
                                },
                            }
                        }
                        div { class: "form-group",
                            label { class: "form-label",
                                "الجنس "
                                span { class: "text-red-500", "*" }
                            }
                            select {
                                name: "gender",
                                required: true,
                                class: "dropdown",
                                onchange: move |evt| {
                                    if evt.value() == "male" {
                                        form_data.gender().set(Some(Gender::Male));
                                    } else if evt.value() == "female" {
                                        form_data.gender().set(Some(Gender::Female));
                                    } else {
                                        form_data.gender().set(None);
                                    }
                                },
                                option { value: "", "اختر الجنس" }
                                option { value: "male", "ذكر" }
                                option { value: "female", "أنثى" }
                            }
                        }
                        div { class: "form-group",
                            label { class: "form-label",
                                "تاريخ الميلاد "
                                span { class: "text-red-500", "*" }
                            }
                            input {
                                dir: "auto",
                                name: "birthday",
                                r#type: "date",
                                required: true,
                                class: "input",
                                oninput: move |evt| {
                                    let date = evt
                                        .value()
                                        .parse::<jiff::civil::Date>()
                                        .ok()
                                        .and_then(|d| d.to_zoned(jiff::tz::TimeZone::UTC).ok());
                                    form_data.birthday().set(date);
                                },
                            }
                        }
                    }
                }
                FormSection {
                    title: "علاقات القرابة".to_string(),
                    icon_path: "M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z"
                        .to_string(),
                    icon_color: Some("green-600".to_string()),
                    div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                        div { class: "form-group",
                            label { class: "form-label", "الوالدة" }
                            MemberPicker {
                                members: props.members.clone().into_iter().map(|m| m.into()).collect(),
                                required_gender: Some(Gender::Female),
                                placeholder: "ابحث عن الوالدة بالاسم...".to_string(),
                                on_select: move |id| form_data.mother_id().set(id),
                            }
                        }
                        div { class: "form-group",
                            label { class: "form-label", "الوالد" }
                            MemberPicker {
                                members: props.members.clone().into_iter().map(|m| m.into()).collect(),
                                required_gender: Some(Gender::Male),
                                placeholder: "ابحث عن الوالد بالاسم...".to_string(),
                                on_select: move |id| form_data.father_id().set(id),
                            }
                        }
                    }
                }
                FormSection {
                    title: "معلومات شخصية إضافية".to_string(),
                    icon_path: "M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                        .to_string(),
                    icon_color: Some("blue-600".to_string()),
                    KeyValueInput {
                        pairs: (form_data
                                                                                                                                                                                                                                                                                                                                                                            .personal_info())()
                            .map(|pi| {
                                pi.iter()
                                    .map(|(key, value)| KeyValuePair {
                                        key: key.clone(),
                                        value: value.clone(),
                                    })
                                    .collect()
                            })
                            .unwrap_or_default(),
                        key_placeholder: Some("المفتاح (مثل: المهنة)".to_string()),
                        value_placeholder: Some("القيمة (مثل: مهندس)".to_string()),
                        on_pairs_change: move |new_pairs: Vec<KeyValuePair>| {
                            let new_pairs = new_pairs
                                .iter()
                                .map(|pair| (pair.key.clone(), pair.value.clone()))
                                .collect();
                            form_data.personal_info().set(Some(new_pairs))
                        },
                    }
                }
                FormSection {
                    title: "صورة العضو".to_string(),
                    icon_path: "M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
                        .to_string(),
                    icon_color: Some("purple-600".to_string()),
                    border_bottom: Some(false),
                    div { class: "form-group",
                        label { class: "form-label", "اختر صورة" }
                        input {
                            r#type: "file",
                            name: "image",
                            accept: "image/*",
                            class: "input",
                            oninput: move |evt| {
                                if let Some(file) = evt.files().into_iter().next() {
                                    image.set(Some(file.into()));
                                }
                            },
                        }
                        p { class: "text-xs text-gray-500 mt-1",
                            "الحد الأقصى: 25 ميجابايت"
                        }
                    }
                }
                div { class: "flex justify-end gap-3 pt-6",
                    button {
                        r#type: "button",
                        class: "btn btn-outline",
                        onclick: move |_| props.on_close.call(()),
                        "إلغاء"
                    }
                    button { r#type: "submit", class: "btn btn-primary btn-lg",
                        svg {
                            class: "w-5 h-5",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M12 6v6m0 0v6m0-6h6m-6 0H6",
                            }
                        }
                        "إضافة العضو"
                    }
                }
            }
        }
    }
}
