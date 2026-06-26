use crate::{
    modules::{
        admin::types::{EditMemberFormData, MemberFormData, MemberFormDataStoreExt},
        member::types::{Gender, MemberResponseFlat},
    },
    ui::{
        form::FormSection,
        key_value_pair::{KeyValueInput, KeyValuePair},
        modal::Modal,
    },
};
use chrono::{NaiveDate, NaiveTime};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct EditMemberModalProps {
    pub show: bool,
    pub member: MemberResponseFlat,
    pub on_close: EventHandler<()>,
    pub on_submit: EventHandler<EditMemberFormData>,
}

#[component]
pub fn EditMemberModal(props: EditMemberModalProps) -> Element {
    let form_data = use_store(|| MemberFormData {
        name: props.member.name.clone(),
        last_name: props.member.last_name.clone(),
        gender: Some(props.member.gender),
        birthday: props.member.birthday,
        mother_id: props.member.mother_id,
        father_id: props.member.father_id,
        personal_info: props.member.personal_info.clone(),
    });

    rsx! {
        Modal {
            show: props.show,
            title: "تحرير العضو".to_string(),
            max_width: Some("4xl".to_string()),
            on_close: move |_| props.on_close.call(()),
            form {
                id: "add-member-form",
                class: "space-y-6",
                autocomplete: "off",
                onsubmit: {
                    let member = props.member.clone();
                    move |evt| {
                        evt.prevent_default();
                        let data = form_data();

                        let submission_data = EditMemberFormData {
                            id: member.id,
                            name: (member.name != data.name).then_some(data.name),
                            last_name: (member.last_name != data.last_name).then_some(data.last_name),
                            gender: (data.gender.and_then(|gender| (gender != member.gender).then_some(gender))),
                            birthday: if data.birthday != member.birthday { data.birthday } else { None },
                            mother_id: if data.mother_id != member.mother_id { data.mother_id } else { None },
                            father_id: if data.father_id != member.father_id { data.father_id } else { None },
                            personal_info: if data.personal_info != member.personal_info { data.personal_info } else { None }
                        };

                        tracing::info!("{submission_data:?}");

                        props.on_submit.call(submission_data);
                    }
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
                                value: "{form_data().gender.map(|g| g.to_string()).unwrap_or(String::new())}",
                                onchange: move |evt| {
                                    let mut gender = form_data.gender();

                                    if evt.value() == "male" {
                                        gender.set(Some(Gender::Male));
                                    } else if evt.value() == "female" {
                                        gender.set(Some(Gender::Female));
                                    } else {
                                        gender.set(None);
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
                                value: r#"{form_data().birthday.map(|d| d.date_naive().to_string()).unwrap_or(String::new())}"#,
                                oninput: move |evt| {
                                    let date = NaiveDate::parse_from_str(&evt.value(), "%Y-%m-%d")
                                        .ok()
                                        .and_then(|d| Some(d.and_time(NaiveTime::from_hms_opt(0, 0, 0)?).and_utc()));
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
                            input {
                                name: "mother_id",
                                r#type: "number",
                                class: "input",
                                placeholder: "ادخل معرف الوالدة (رقم)",
                                value: "{form_data().mother_id.map(|id| id.to_string()).unwrap_or_default()}",
                                oninput: move |evt| {
                                    form_data.mother_id().set(evt.value().parse().ok());
                                },
                            }
                            p { class: "text-xs text-gray-500 mt-1",
                                "يمكنك البحث عن الأعضاء في القائمة أعلاه لمعرفة الأرقام"
                            }
                        }
                        div { class: "form-group",
                            label { class: "form-label", "الوالد" }
                            input {
                                name: "father_id",
                                r#type: "number",
                                class: "input",
                                placeholder: "ادخل معرف الوالد (رقم)",
                                value: "{form_data().father_id.map(|id| id.to_string()).unwrap_or_default()}",
                                oninput: move |evt| {
                                    form_data.father_id().set(evt.value().parse().ok());
                                },
                            }
                            p { class: "text-xs text-gray-500 mt-1",
                                "يمكنك البحث عن الأعضاء في القائمة أعلاه لمعرفة الأرقام"
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
                            }).unwrap_or_default(),
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
                        "تحرير العضو"
                    }
                }
            }
        }
    }
}
