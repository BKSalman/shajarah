use chrono::{DateTime, NaiveDateTime, Utc};
use dioxus::prelude::*;

use crate::{
    modules::member::types::Gender,
    ui::key_value_pair::{KeyValueInput, KeyValuePair},
};

pub mod server;
pub mod types;

#[component]
pub fn AddMember() -> Element {
    let mut name = use_signal(|| String::new());
    let mut last_name = use_signal(|| String::new());
    let mut gender = use_signal(|| None::<Gender>);
    let mut birthday = use_signal(|| None::<DateTime<Utc>>);
    let mut father_id = use_signal(|| None::<i64>);
    let mut extra_info_pairs = use_signal(|| Vec::<KeyValuePair>::new());
    let mut submit_message_visible = use_signal(|| false);
    let mut image_type = use_signal(|| None::<String>);
    let mut image_file = use_signal(|| None::<Vec<u8>>);

    rsx! {
        div {
            dir: "rtl",
            class: "min-h-screen flex items-center justify-center bg-gray-100 p-8",

            form {
                class: "bg-white p-8 rounded shadow-md w-full max-w-xl space-y-4",
                onsubmit: move |_| async move {
                    let Some(gender) = gender() else {
                        return;
                    };

                    let info = extra_info_pairs

                        .iter()
                        .map(|kv| (kv.key.clone(), kv.value.clone()))
                        .collect();
                    if server::add_request(
                            name(),
                            last_name(),
                            gender,
                            birthday(),
                            father_id(),
                            None,
                            info,
                            image_file(),
                            image_type(),
                        )
                        .await
                        .is_ok()
                    {
                        submit_message_visible.set(true);
                    }
                },

                h2 { class: "text-2xl font-bold text-center",
                    "تقديم معلومات فرد العائلة"
                }

                div {
                    label { class: "block mb-1", "الاسم:" }
                    input {
                        r#type: "text",
                        class: "w-full border rounded px-3 py-2",
                        required: true,
                        value: "{name}",
                        oninput: move |evt| name.set(evt.value()),
                    }
                }

                div {
                    label { class: "block mb-1", "الاسم الاخير:" }
                    input {
                        r#type: "text",
                        class: "w-full border rounded px-3 py-2",
                        required: true,
                        value: "{last_name}",
                        oninput: move |evt| last_name.set(evt.value()),
                    }
                }

                div {
                    label { class: "block mb-1", "الجنس:" }
                    select {
                        class: "w-full border rounded px-3 py-2",
                        required: true,
                        onchange: move |evt| {
                            match evt.value().as_str() {
                                "male" => {
                                    gender.set(Some(Gender::Male));
                                }
                                "female" => {
                                    gender.set(Some(Gender::Female));
                                }
                                _ => gender.set(None),
                            }
                        },

                        option { value: "", "-- اختر الجنس --" }
                        option { value: "male", "ذكر" }
                        option { value: "female", "انثى" }
                    }
                }

                div {
                    label { class: "block mb-1", "تاريخ الولادة:" }
                    input {
                        r#type: "date",
                        class: "w-full border rounded px-3 py-2",
                        required: true,
                        oninput: move |evt| {
                            birthday
                                .set(
                                    NaiveDateTime::parse_from_str(&evt.value(), "%Y-%m-%d")
                                        .map(|d| d.and_utc())
                                        .ok(),
                                )
                        },
                    }
                }

                div {
                    label { class: "block mb-1", "معرف الأب:" }
                    input {
                        r#type: "text",
                        class: "w-full border rounded px-3 py-2",
                        required: true,
                        list: "father-ids",
                        oninput: move |evt| father_id.set(evt.value().parse().ok()),
                    }
                    datalist { id: "father-ids" }
                }

                div {
                    label { class: "block mb-1", "معلومات إضافية:" }
                    KeyValueInput {
                        pairs: extra_info_pairs.read().clone(),
                        key_placeholder: "مثال: المهنة".to_string(),
                        value_placeholder: "مثال: محامي".to_string(),
                        on_pairs_change: move |new_pairs| {
                            extra_info_pairs.set(new_pairs);
                        },
                    }
                }

                div {
                    label { class: "block mb-1", "صورة (اختياري):" }
                    input {
                        r#type: "file",
                        class: "w-full",
                        accept: "image/*",
                        onchange: move |evt| async move {
                            #[cfg(feature = "web")]
                            {
                                use dioxus::web::WebFileEngineExt;

                                if let Some(file_engine) = &evt.files() {
                                    let files = file_engine.files();
                                    for file_name in files {
                                        let file = file_engine.get_web_file(&file_name).await.unwrap();
                                        let mime_type = file.type_();
                                        if let Some(file) = file_engine.read_file(&file_name).await {
                                            image_file.set(Some(file));
                                            image_type.set(Some(mime_type));
                                        }
                                    }
                                }
                            }
                        },
                    }
                }

                button {
                    r#type: "submit",
                    class: "bg-blue-600 hover:bg-blue-700 text-white px-6 py-2 rounded w-full",
                    "إرسال للمراجعة"
                }

                if *submit_message_visible.read() {
                    div { class: "text-center text-green-600 font-semibold",
                        "تم إرسال المعلومات بنجاح! شكراً لمساهمتك."
                    }
                }
            }
        }
    }
}
