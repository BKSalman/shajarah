use dioxus::prelude::*;

use crate::modules::settings::{
    server::{get_settings, update_settings},
    types::{RequestRules, Settings},
};

#[component]
fn SettingToggle(
    title: String,
    description: String,
    enabled: bool,
    busy: bool,
    on_toggle: EventHandler<bool>,
) -> Element {
    rsx! {
        div { class: "flex items-start justify-between gap-6 py-4",
            div {
                h3 { class: "font-medium text-forest-dark arabic-text", "{title}" }
                p { class: "text-sm text-gray-500 mt-1 arabic-text", "{description}" }
            }

            button {
                r#type: "button",
                class: "switch",
                "role": "switch",
                "aria-checked": "{enabled}",
                "aria-label": "{title}",
                disabled: busy,
                onclick: move |_| on_toggle.call(!enabled),
                div { class: "switch-knob" }
            }
        }
    }
}

#[component]
fn RequiredInfoKeys(
    settings: Settings,
    busy: bool,
    on_change: EventHandler<Vec<String>>,
) -> Element {
    let mut draft = use_signal(String::new);

    let keys = settings.add_request_rules.required_info_keys.clone();

    let mut add_key = move |keys: Vec<String>| {
        let key = draft().trim().to_string();

        if key.is_empty() || keys.contains(&key) {
            return;
        }

        let mut keys = keys;
        keys.push(key);
        draft.set(String::new());
        on_change.call(keys);
    };

    rsx! {
        div { class: "py-4",
            h3 { class: "font-medium text-forest-dark arabic-text",
                "معلومات إضافية مطلوبة"
            }
            p { class: "text-sm text-gray-500 mt-1 arabic-text",
                "تظهر هذه الحقول في نموذج الإضافة بمفتاح ثابت، ولا يمكن إرسال الطلب دون تعبئتها."
            }

            div { class: "space-y-2 mt-3",
                for key in keys.clone() {
                    div { key: "{key}", class: "flex gap-3 items-center",
                        input {
                            r#type: "text",
                            class: "input w-full flex-1 bg-gray-50 text-gray-600",
                            readonly: true,
                            value: "{key}",
                        }
                        button {
                            r#type: "button",
                            class: "btn btn-danger btn-sm",
                            disabled: busy,
                            onclick: {
                                let keys = keys.clone();
                                let key = key.clone();
                                move |_| {
                                    on_change
                                        .call(
                                            keys.iter().filter(|k| **k != key).cloned().collect(),
                                        );
                                }
                            },
                            "حذف"
                        }
                    }
                }

                div { class: "flex gap-3 items-center",
                    input {
                        r#type: "text",
                        class: "input w-full flex-1",
                        placeholder: "مثال: المهنة",
                        value: "{draft}",
                        disabled: busy,
                        oninput: move |evt| draft.set(evt.value()),
                        onkeydown: {
                            let keys = keys.clone();
                            move |evt: Event<KeyboardData>| {
                                if evt.key() == Key::Enter {
                                    evt.prevent_default();
                                    add_key(keys.clone());
                                }
                            }
                        },
                    }
                    button {
                        r#type: "button",
                        class: "btn btn-outline btn-sm",
                        disabled: busy,
                        onclick: {
                            let keys = keys.clone();
                            move |_| add_key(keys.clone())
                        },
                        "إضافة"
                    }
                }
            }
        }
    }
}

#[component]
pub fn AdminSettings() -> Element {
    let mut settings_resource = use_resource(get_settings);

    let mut save = use_action(move |settings: Settings| async move {
        update_settings(settings).await?;

        settings_resource.restart();

        anyhow::Ok(())
    });

    let body = match &*settings_resource.read() {
        Some(Ok(settings)) => {
            let settings = settings.clone();
            let rules = settings.add_request_rules.clone();
            rsx! {
                div { class: "divide-y divide-gray-100",
                    SettingToggle {
                        title: "صفحة إضافة الأفراد",
                        description: "عند التعطيل، يصبح رابط /add غير متاح لأفراد العائلة ولا يمكن إرسال طلبات إضافة جديدة.",
                        enabled: settings.add_page_enabled,
                        busy: save.pending(),
                        on_toggle: {
                            let settings = settings.clone();
                            move |enabled| {
                                save.call(Settings {
                                    add_page_enabled: enabled,
                                    ..settings.clone()
                                });
                            }
                        },
                    }
                }

                h2 { class: "text-lg font-semibold text-forest-dark arabic-heading mt-6 mb-1",
                    "الحقول المطلوبة في طلبات الإضافة"
                }
                p { class: "text-sm text-gray-500 arabic-text",
                    "الاسم الأول واسم العائلة والجنس والوالد مطلوبة دائماً."
                }

                div { class: "divide-y divide-gray-100",
                    SettingToggle {
                        title: "تاريخ الولادة",
                        description: "إلزام مُقدّم الطلب بإدخال تاريخ ولادة الفرد وكل طفل.",
                        enabled: rules.require_birthday,
                        busy: save.pending(),
                        on_toggle: {
                            let settings = settings.clone();
                            move |enabled| {
                                save.call(Settings {
                                    add_request_rules: RequestRules {
                                        require_birthday: enabled,
                                        ..settings.add_request_rules.clone()
                                    },
                                    ..settings.clone()
                                });
                            }
                        },
                    }
                    SettingToggle {
                        title: "الصورة الشخصية",
                        description: "إلزام مُقدّم الطلب بإرفاق صورة شخصية للفرد وكل طفل.",
                        enabled: rules.require_image,
                        busy: save.pending(),
                        on_toggle: {
                            let settings = settings.clone();
                            move |enabled| {
                                save.call(Settings {
                                    add_request_rules: RequestRules {
                                        require_image: enabled,
                                        ..settings.add_request_rules.clone()
                                    },
                                    ..settings.clone()
                                });
                            }
                        },
                    }
                }

                RequiredInfoKeys {
                    settings: settings.clone(),
                    busy: save.pending(),
                    on_change: move |keys: Vec<String>| {
                        save.call(Settings {
                            add_request_rules: RequestRules {
                                required_info_keys: keys,
                                ..settings.add_request_rules.clone()
                            },
                            ..settings.clone()
                        });
                    },
                }

                if let Some(Err(e)) = save.value() {
                    div { class: "rounded-lg bg-red-50 text-red-700 px-4 py-3 arabic-text mt-2",
                        "تعذّر حفظ الإعدادات: {e}"
                    }
                }
            }
        }
        Some(Err(e)) => {
            rsx! {
                div { class: "rounded-lg bg-red-50 text-red-700 px-4 py-3 arabic-text",
                    "تعذّر تحميل الإعدادات: {e}"
                }
            }
        }
        None => {
            rsx! {
                div { class: "text-center text-gray-500 py-12 arabic-text",
                    "جارٍ التحميل..."
                }
            }
        }
    };

    rsx! {
        div { dir: "rtl", class: "w-full",
            div { class: "card card-forest fade-in",
                div { class: "card-body",
                    h1 { class: "text-2xl lg:text-3xl font-bold text-forest-dark arabic-heading mb-4",
                        "الإعدادات"
                    }
                    {body}
                }
            }
        }
    }
}
