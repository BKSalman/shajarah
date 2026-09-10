use crate::{
    modules::{
        admin::components::spouse_fields::{SpouseDraft, SpouseFields, SpouseSubmission},
        member::types::{
            EditMember, EditMemberStoreExt, Gender, MarriageStatus, MemberResponseFlat,
        },
        types::EditField,
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
pub struct EditMemberModalProps {
    pub show: bool,
    pub member: MemberResponseFlat,
    pub members: Vec<MemberResponseFlat>,
    pub on_close: EventHandler<()>,
    pub on_submit: EventHandler<(i64, EditMember, Option<FileStream>)>,
    /// Marries this member to the chosen spouse, creating them when they come
    /// from outside the family.
    pub on_marriage_apply: EventHandler<(i64, SpouseSubmission)>,
    pub on_marriage_status: EventHandler<(i64, MarriageStatus)>,
    pub on_marriage_remove: EventHandler<i64>,
}

#[component]
pub fn EditMemberModal(props: EditMemberModalProps) -> Element {
    let mut form_data = use_store(EditMember::default);
    let mut image = use_signal(|| None::<FileStream>);

    // The spouse panel acts on its own: marriages are their own rows, not
    // columns on the member, so they are saved as you go rather than on submit.
    let mut spouse_draft = use_store(SpouseDraft::default);
    let mut spouse_image = use_signal(|| None::<FileStream>);

    rsx! {
        Modal {
            show: props.show,
            title: "تحرير العضو".to_string(),
            max_width: Some("4xl".to_string()),
            on_close: move |_| {
                props.on_close.call(());
                form_data.set(EditMember::default());
            },
            form {
                id: "add-member-form",
                class: "space-y-6",
                autocomplete: "off",
                onsubmit: move |evt| {
                    evt.prevent_default();

                    tracing::info!("{:?}", form_data());
                    props.on_submit.call((props.member.id, form_data().clone(), image.take()));
                    form_data.set(EditMember::default());
                },
                FormSection {
                    title: "المعلومات الأساسية".to_string(),
                    icon_path: "M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z".to_string(),
                    icon_color: Some("primary-600".to_string()),
                    div { class: "flex flex-col",
                        div { class: "flex flex-row w-full gap-4",
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
                                    class: "input w-full",
                                    placeholder: "ادخل الاسم الأول",
                                    value: if let Some(name) = (&*form_data.name())() { name } else { props.member.name.clone() },
                                    oninput: move |evt: Event<FormData>| {
                                        if evt.value() != props.member.name {
                                            form_data.name().set(Some(evt.value()));
                                        } else {
                                            form_data.name().set(None);
                                        }
                                    },
                                }
                            }
                            div { class: "form-group",
                                label { class: "form-label", "الوالد" }
                                MemberPicker {
                                    members: props.members.clone().into_iter().map(|m| m.into()).collect(),
                                    exclude_id: Some(props.member.id),
                                    required_gender: Some(Gender::Male),
                                    initial_label: props
                                        .members
                                        .iter()
                                        .find(|m| props.member.father_id == Some(m.id))
                                        .map(|m| m.full_name.clone())
                                        .unwrap_or_default(),
                                    placeholder: "ابحث عن الوالد بالاسم...".to_string(),
                                    on_select: move |id: Option<i64>| {
                                        match id {
                                            None => form_data.father_id().set(EditField::Delete),
                                            Some(father_id) if Some(father_id) != props.member.father_id => {
                                                form_data.father_id().set(EditField::Changed(father_id));
                                            }
                                            Some(_) => form_data.father_id().set(EditField::Unchanged),
                                        }
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
                                    class: "input w-full",
                                    placeholder: "ادخل الاسم الأخير",
                                    value: if let Some(last_name) = (&*form_data.last_name())() { last_name } else { props.member.last_name.clone() },
                                    oninput: move |evt: Event<FormData>| {
                                        if evt.value() != props.member.last_name {
                                            form_data.last_name().set(Some(evt.value()));
                                        } else {
                                            form_data.last_name().set(None);
                                        }
                                    },
                                }
                            }
                        }
                        div { class: "flex flex-row w-full gap-4",
                            div { class: "form-group",
                                label { class: "form-label",
                                    "الجنس "
                                    span { class: "text-red-500", "*" }
                                }
                                select {
                                    name: "gender",
                                    required: true,
                                    class: "dropdown",
                                    value: if let Some(gender) = (&*form_data.gender())() { gender } else { props.member.gender },
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
                                    value: if let EditField::Changed(birthday) = (&*form_data.birthday())() { birthday.date().to_string() } else { props
                                        .member
                                        .birthday
                                        .clone()
                                        .map(|b| b.date().to_string())
                                        .unwrap_or(String::from("")) },
                                    oninput: move |evt: Event<FormData>| {
                                        if evt.value().is_empty() {
                                            form_data.birthday().set(EditField::Delete);
                                        } else {
                                            let date = evt
                                                .value()
                                                .parse::<jiff::civil::Date>()
                                                .ok()
                                                .and_then(|d| d.to_zoned(jiff::tz::TimeZone::UTC).ok());
                                            if props.member.birthday != date {
                                                if let Some(date) = date {
                                                    form_data.birthday().set(EditField::Changed(date));
                                                }
                                            } else {
                                                form_data.birthday().set(EditField::Unchanged);
                                            }
                                        }
                                    },
                                }
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
                                exclude_id: Some(props.member.id),
                                required_gender: Some(Gender::Female),
                                initial_label: props
                                    .members
                                    .iter()
                                    .find(|m| props.member.mother_id == Some(m.id))
                                    .map(|m| format!("{} {}", m.name, m.last_name))
                                    .unwrap_or_default(),
                                placeholder: "ابحث عن الوالدة بالاسم...".to_string(),
                                on_select: move |id: Option<i64>| {
                                    match id {
                                        None => form_data.mother_id().set(EditField::Delete),
                                        Some(mother_id) if Some(mother_id) != props.member.mother_id => {
                                            form_data.mother_id().set(EditField::Changed(mother_id));
                                        }
                                        Some(_) => form_data.mother_id().set(EditField::Unchanged),
                                    }
                                },
                            }
                        }
                    }
                }
                FormSection {
                    title: "الزيجات".to_string(),
                    icon_path: "M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z"
                        .to_string(),
                    icon_color: Some("green-600".to_string()),

                    div { class: "space-y-2",
                        if props.member.spouses.is_empty() {
                            p { class: "text-sm text-gray-500", "لا توجد زيجات مسجلة" }
                        }
                        for spouse in props.member.spouses.clone() {
                            div { key: "{spouse.marriage_id}", class: "flex items-center gap-3",
                                span { class: "flex-1 min-w-0 truncate", "{spouse.full_name}" }
                                select {
                                    class: "dropdown",
                                    onchange: move |evt| {
                                        if let Ok(status) = evt.value().parse::<MarriageStatus>() {
                                            props.on_marriage_status.call((spouse.marriage_id, status));
                                        }
                                    },
                                    option {
                                        value: "married",
                                        selected: spouse.status == MarriageStatus::Married,
                                        "متزوج"
                                    }
                                    option {
                                        value: "separated",
                                        selected: spouse.status == MarriageStatus::Separated,
                                        "منفصل"
                                    }
                                }
                                button {
                                    r#type: "button",
                                    class: "btn btn-danger btn-sm",
                                    onclick: move |_| props.on_marriage_remove.call(spouse.marriage_id),
                                    "إزالة"
                                }
                            }
                        }
                    }

                    div { class: "mt-4 pt-4 border-t border-gray-100 space-y-3",
                        SpouseFields {
                            draft: spouse_draft,
                            image: spouse_image,
                            members: props.members.clone(),
                            exclude_id: Some(props.member.id),
                            member_gender: Some(props.member.gender),
                        }

                        button {
                            r#type: "button",
                            class: "btn btn-primary btn-sm",
                            onclick: move |_| {
                                let Some(submission) = spouse_draft().submission(spouse_image.take())
                                else {
                                    return;
                                };
                                props.on_marriage_apply.call((props.member.id, submission));
                                spouse_draft.set(SpouseDraft::default());
                            },
                            "إضافة زوج/زوجة"
                        }
                    }
                }
                FormSection {
                    title: "معلومات شخصية إضافية".to_string(),
                    icon_path: "M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
                        .to_string(),
                    icon_color: Some("blue-600".to_string()),
                    KeyValueInput {
                        pairs: match &*form_data.personal_info().read() {
                            EditField::Changed(pi) => {
                                pi.iter()
                                    .map(|(key, value)| KeyValuePair {
                                        key: key.clone(),
                                        value: value.clone(),
                                    })
                                    .collect()
                            }
                            _ => {
                                if let Some(pi) = props.member.personal_info {
                                    pi.iter()
                                        .map(|(key, value)| KeyValuePair {
                                            key: key.clone(),
                                            value: value.clone(),
                                        })
                                        .collect()
                                } else {
                                    Vec::new()
                                }
                            }
                        },
                        key_placeholder: Some("المفتاح (مثل: المهنة)".to_string()),
                        value_placeholder: Some("القيمة (مثل: مهندس)".to_string()),
                        on_pairs_change: move |new_pairs: Vec<KeyValuePair>| {
                            let new_pairs = new_pairs
                                .iter()
                                .map(|pair| (pair.key.clone(), pair.value.clone()))
                                .collect();
                            form_data.personal_info().set(EditField::Changed(new_pairs))
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
                        onclick: move |_| {
                            props.on_close.call(());
                            form_data.set(EditMember::default());
                        },
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
