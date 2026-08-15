use dioxus::prelude::*;
use garde::Validate;
use indexmap::IndexMap;
use std::collections::HashMap;

use crate::{
    i18n::Arabic,
    modules::{
        add_request::types::{RequestData, RequestDataStoreExt},
        member::{
            components::member_picker::MemberPicker, server::members_flat_unauthorized,
            types::Gender,
        },
    },
    ui::key_value_pair::{KeyValueInput, KeyValuePair},
};
use server::add_request;

pub mod server;
pub mod types;

#[component]
pub fn AddMemberRequest() -> Element {
    let request_data = use_store(|| RequestData {
        name: None,
        last_name: None,
        gender: None,
        birthday: None,
        father_id: None,
        mother_id: None,
        info: IndexMap::new(),
        image: None,
        image_type: None,
    });

    let mut field_errors = use_signal(HashMap::<String, String>::new);
    let mut error_message = use_signal(|| Option::<String>::None);
    let mut is_submitting = use_signal(|| false);
    let mut show_success = use_signal(|| false);

    let get_field_error =
        move |field: &str| -> Option<String> { field_errors.read().get(field).cloned() };

    let has_field_error = move |field: &str| -> bool { field_errors.read().contains_key(field) };

    let mut clear_field_error = move |field: &str| {
        field_errors.with_mut(|errors| {
            errors.remove(field);
        });
    };

    let members_resource = use_resource(members_flat_unauthorized);

    let members = (*members_resource.read())
        .as_ref()
        .and_then(|e| e.as_ref().ok())
        .cloned()
        .unwrap_or_default();

    rsx! {
        div {
            class: "min-h-screen flex items-center justify-center bg-tree-texture p-6",
            dir: "rtl",

            div { class: "card card-forest w-3/4 max-w-2xl fade-in hover:shadow-forest",

                div { class: "card-body",

                    // Header
                    div { class: "text-center mb-6",
                        h1 { class: "text-3xl font-bold text-forest-dark mb-2", "شجرة" }
                        h2 { class: "text-xl font-semibold text-forest-primary",
                            "تقديم معلومات فرد العائلة"
                        }
                        p { class: "text-sm text-gray-500 mt-2",
                            "قم بإدخال بيانات الفرد ليتم مراجعتها وإضافتها للشجرة"
                        }
                    }

                    form {
                        class: "space-y-6",
                        autocomplete: "off",
                        onsubmit: move |e| {
                            e.prevent_default();
                            async move {
                                if is_submitting() {
                                    return;
                                }

                                show_success.set(false);

                                // Clear previous errors
                                error_message.set(None);
                                field_errors.set(HashMap::new());

                                is_submitting.set(true);

                                if let Err(report) = garde::with_i18n(Arabic, || request_data().validate()) {
                                    for (field_name, field_report) in report.iter() {
                                        let mut errors = field_errors.write();
                                        errors
                                            .insert(
                                                field_name.to_string(),
                                                field_report.message().to_string(),
                                            );
                                    }
                                    is_submitting.set(false);
                                    return;
                                }

                                match add_request(request_data()).await {
                                    Ok(_) => {
                                        request_data.name().set(None);
                                        request_data.last_name().set(None);
                                        request_data.gender().set(None);
                                        request_data.birthday().set(None);
                                        request_data.father_id().set(None);
                                        request_data.info().set(IndexMap::new());
                                        request_data.image().set(None);
                                        request_data.image_type().set(None);
                                        show_success.set(true);
                                    }
                                    Err(server_error) => {
                                        error_message.set(Some(server_error.to_string()));
                                    }
                                }
                                is_submitting.set(false);
                            }
                        },

                        // Name, Father, and Last Name Fields
                        div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                            div { class: "form-group min-w-0",
                                label { r#for: "name", class: "form-label",
                                    svg {
                                        class: "w-4 h-4 inline ml-2",
                                        fill: "none",
                                        stroke: "currentColor",
                                        view_box: "0 0 24 24",
                                        path {
                                            stroke_linecap: "round",
                                            stroke_linejoin: "round",
                                            stroke_width: "2",
                                            d: "M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z",
                                        }
                                    }
                                    "الاسم الأول"
                                }
                                input {
                                    id: "name",
                                    r#type: "text",
                                    required: true,
                                    disabled: is_submitting(),
                                    class: "input w-full",
                                    class: if has_field_error("name") { "border-red-300 focus:border-red-500" },
                                    class: if is_submitting() { "loading" },
                                    placeholder: "أدخل الاسم الأول",
                                    value: request_data.name().read().as_deref().unwrap_or(""),
                                    oninput: move |evt| {
                                        request_data.name().set(Some(evt.value()));
                                        clear_field_error("name");
                                    },
                                }
                                if let Some(error) = get_field_error("name") {
                                    p { class: "text-sm text-red-600 mt-1", "{error}" }
                                }
                            }

                            div { class: "form-group min-w-0",
                                label { class: "form-label", "الوالد" }
                                MemberPicker {
                                    members: members.clone(),
                                    required_gender: Some(Gender::Male),
                                    placeholder: "ابحث عن الوالد بالاسم...".to_string(),
                                    on_select: move |id| request_data.father_id().set(id),
                                }
                            }

                            div { class: "form-group min-w-0",
                                label { r#for: "last_name", class: "form-label",
                                    svg {
                                        class: "w-4 h-4 inline ml-2",
                                        fill: "none",
                                        stroke: "currentColor",
                                        view_box: "0 0 24 24",
                                        path {
                                            stroke_linecap: "round",
                                            stroke_linejoin: "round",
                                            stroke_width: "2",
                                            d: "M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z",
                                        }
                                    }
                                    "اسم العائلة"
                                }
                                input {
                                    id: "last_name",
                                    r#type: "text",
                                    required: true,
                                    disabled: is_submitting(),
                                    class: "input w-full",
                                    class: if has_field_error("last_name") { "border-red-300 focus:border-red-500" },
                                    class: if is_submitting() { "loading" },
                                    placeholder: "أدخل اسم العائلة",
                                    value: request_data.last_name().read().as_deref().unwrap_or(""),
                                    oninput: move |evt| {
                                        request_data.last_name().set(Some(evt.value()));
                                        clear_field_error("last_name");
                                    },
                                }
                                if let Some(error) = get_field_error("last_name") {
                                    p { class: "text-sm text-red-600 mt-1", "{error}" }
                                }
                            }
                        }

                        // Gender Field
                        div { class: "form-group",
                            label { r#for: "gender", class: "form-label",
                                svg {
                                    class: "w-4 h-4 inline ml-2",
                                    fill: "none",
                                    stroke: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        stroke_width: "2",
                                        d: "M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197m13.5-9a2.5 2.5 0 11-5 0 2.5 2.5 0 015 0z",
                                    }
                                }
                                "الجنس"
                            }
                            select {
                                id: "gender",
                                required: true,
                                disabled: is_submitting(),
                                class: "dropdown",
                                class: if has_field_error("gender") { "border-red-300 focus:border-red-500" },
                                class: if is_submitting() { "loading" },
                                onchange: move |evt| {
                                    match evt.value().as_str() {
                                        "male" => request_data.gender().set(Some(Gender::Male)),
                                        "female" => request_data.gender().set(Some(Gender::Female)),
                                        _ => request_data.gender().set(None),
                                    }
                                    clear_field_error("gender");
                                },

                                option { value: "", "-- اختر الجنس --" }
                                option {
                                    value: "male",
                                    selected: matches!(request_data.gender().read().as_ref(), Some(Gender::Male)),
                                    "ذكر"
                                }
                                option {
                                    value: "female",
                                    selected: matches!(request_data.gender().read().as_ref(), Some(Gender::Female)),
                                    "أنثى"
                                }
                            }
                            if let Some(error) = get_field_error("gender") {
                                p { class: "text-sm text-red-600 mt-1", "{error}" }
                            }
                        }

                        // Birthday Field
                        div { class: "form-group",
                            label { r#for: "birthday", class: "form-label",
                                svg {
                                    class: "w-4 h-4 inline ml-2",
                                    fill: "none",
                                    stroke: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        stroke_width: "2",
                                        d: "M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z",
                                    }
                                }
                                "تاريخ الولادة"
                            }
                            input {
                                id: "birthday",
                                r#type: "date",
                                required: true,
                                disabled: is_submitting(),
                                class: "input w-full",
                                class: if has_field_error("birthday") { "border-red-300 focus:border-red-500" },
                                class: if is_submitting() { "loading" },
                                value: request_data
                                    .birthday()
                                    .read()
                                    .as_ref()
                                    .map(|dt| dt.strftime("%Y-%m-%d").to_string())
                                    .unwrap_or_default(),
                                oninput: move |evt| {
                                    request_data
                                        .birthday()
                                        .set(
                                            evt
                                                .value()
                                                .parse::<jiff::civil::Date>()
                                                .ok()
                                                .and_then(|d| d.to_zoned(jiff::tz::TimeZone::UTC).ok()),
                                        );
                                    clear_field_error("birthday");
                                },
                            }
                            if let Some(error) = get_field_error("birthday") {
                                p { class: "text-sm text-red-600 mt-1", "{error}" }
                            }
                        }

                        // Extra Information Section
                        div { class: "form-group",
                            label { class: "form-label",
                                svg {
                                    class: "w-4 h-4 inline ml-2",
                                    fill: "none",
                                    stroke: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        stroke_width: "2",
                                        d: "M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z",
                                    }
                                }
                                "معلومات إضافية"
                            }
                            KeyValueInput {
                                pairs: request_data
                                    .info()
                                    .read()
                                    .iter()
                                    .map(|(key, value)| KeyValuePair {
                                        key: key.clone(),
                                        value: value.clone(),
                                    })
                                    .collect(),
                                key_placeholder: "مثال: المهنة".to_string(),
                                value_placeholder: "مثال: محامي".to_string(),
                                on_pairs_change: move |new_pairs: Vec<KeyValuePair>| {
                                    let new_info = new_pairs
                                        .iter()
                                        .map(|pair| (pair.key.clone(), pair.value.clone()))
                                        .collect();
                                    request_data.info().set(new_info);
                                },
                            }
                        }

                        // Image Upload Field
                        div { class: "form-group",
                            label { r#for: "image", class: "form-label",
                                svg {
                                    class: "w-4 h-4 inline ml-2",
                                    fill: "none",
                                    stroke: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        stroke_width: "2",
                                        d: "M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z",
                                    }
                                }
                                "صورة شخصية (اختياري)"
                            }
                            input {
                                id: "image",
                                r#type: "file",
                                accept: "image/*",
                                disabled: is_submitting(),
                                class: "input w-full",
                                class: if is_submitting() { "loading" },
                                onchange: move |evt| async move {
                                    #[cfg(feature = "web")]
                                    {
                                        if let Some(file_data) = evt.files().first() {
                                            let Some(mime_type) = file_data.content_type() else {
                                                return;
                                            };
                                            let max_size = 25 * 1024 * 1024; // 25MB
                                            let allowed_types = ["image/jpeg", "image/png", "image/gif"];

                                            if file_data.size() as usize > max_size {
                                                field_errors

                                                    .with_mut(|errors| {
                                                        errors

                                                            .insert(
                                                                "image".to_string(),
                                                                "حجم الصورة يجب أن يكون أقل من 25MB"
                                                                    .to_string(),
                                                            );
                                                    });
                                                return;
                                            }
                                            if !allowed_types.contains(&mime_type.as_str()) {
                                                field_errors
                                                    .with_mut(|errors| {
                                                        errors
                                                            .insert(
                                                                "image".to_string(),
                                                                "نوع الصورة غير مدعوم. يرجى استخدام JPG أو PNG أو GIF"
                                                                    .to_string(),
                                                            );
                                                    });
                                                return;
                                            }
                                            if let Ok(file_data) = file_data.read_bytes().await
                                            {
                                                request_data.image().set(Some(file_data.to_vec()));
                                                request_data.image_type().set(Some(mime_type));
                                                field_errors
                                                    .with_mut(|errors| {
                                                        errors.remove("image");
                                                    });
                                            }
                                        }
                                    }
                                },
                            }
                            p { class: "text-xs text-gray-500 mt-1",
                                "أحجام الصور المقبولة: JPG, PNG, GIF - الحد الأقصى: 25MB"
                            }
                            if let Some(error) = get_field_error("image") {
                                p { class: "text-sm text-red-600 mt-1", "{error}" }
                            }
                        }

                        // Submit Button
                        button {
                            r#type: "submit",
                            disabled: is_submitting(),
                            class: "btn btn-primary w-full btn-lg",
                            class: if is_submitting() { "opacity-50 cursor-not-allowed loading" },

                            if is_submitting() {
                                svg {
                                    class: "animate-spin -ml-1 mr-3 h-4 w-4 text-white",
                                    fill: "none",
                                    view_box: "0 0 24 24",
                                    circle {
                                        class: "opacity-25",
                                        cx: "12",
                                        cy: "12",
                                        r: "10",
                                        stroke: "currentColor",
                                        stroke_width: "4",
                                    }
                                    path {
                                        class: "opacity-75",
                                        fill: "currentColor",
                                        d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z",
                                    }
                                }
                                "جاري الإرسال..."
                            } else {
                                "إرسال للمراجعة"
                            }
                        }

                        if show_success() {
                            div { class: "alert alert-success",
                                div { class: "flex items-center",
                                    svg {
                                        class: "w-5 h-5 ml-2",
                                        fill: "none",
                                        stroke: "currentColor",
                                        view_box: "0 0 24 24",
                                        path {
                                            stroke_linecap: "round",
                                            stroke_linejoin: "round",
                                            stroke_width: "2",
                                            d: "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z",
                                        }
                                    }
                                    p { class: "text-green-500", "تم الإرسال بنجاح!" }
                                }
                                p {

                                    "تم إرسال معلومات العضو للمراجعة. شكراً لمساهمتك!"
                                }
                            }
                        }

                        // General Error Alert
                        if let Some(error) = error_message.read().as_ref() {
                            div { class: "alert alert-error",
                                div { class: "flex items-center",
                                    svg {
                                        class: "w-5 h-5 ml-2",
                                        fill: "none",
                                        stroke: "currentColor",
                                        view_box: "0 0 24 24",
                                        path {
                                            stroke_linecap: "round",
                                            stroke_linejoin: "round",
                                            stroke_width: "2",
                                            d: "M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z",
                                        }
                                    }
                                    p { "{error}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
