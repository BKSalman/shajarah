use dioxus::prelude::*;

use crate::modules::settings::{
    server::{get_settings, update_settings},
    types::{RequestRules, Settings},
};

/// A titled group of related options, drawn as its own panel so the sections
/// read apart from the rows inside them.
#[component]
fn SettingsSection(icon: String, title: String, description: String, children: Element) -> Element {
    rsx! {
        section { class: "rounded-lg border border-(--color-forest-light) bg-white shadow-sm overflow-hidden",
            div { class: "flex items-start gap-3 px-5 py-4 bg-(--color-forest-background) border-b border-(--color-forest-light)",
                span { class: "flex items-center justify-center shrink-0 w-9 h-9 rounded-lg bg-white border border-(--color-forest-light) text-(--color-forest-primary)",
                    i { class: "fa-solid {icon}" }
                }
                div { class: "min-w-0",
                    h2 { class: "text-lg font-bold text-(--color-forest-dark)", "{title}" }
                    p { class: "text-sm text-gray-600 mt-0.5", "{description}" }
                }
            }

            div { class: "divide-y divide-gray-100", {children} }
        }
    }
}

#[component]
fn SettingToggle(
    title: String,
    description: String,
    enabled: bool,
    busy: bool,
    /// State labels for the pill beside the switch, e.g. `("مطلوب", "اختياري")`.
    #[props(default = (String::from("مفعّلة"), String::from("معطّلة")))]
    state_labels: (String, String),
    on_toggle: EventHandler<bool>,
) -> Element {
    let (on_label, off_label) = state_labels;

    rsx! {
        div { class: "flex items-start justify-between gap-6 px-5 py-4 transition-colors hover:bg-gray-50",
            div { class: "min-w-0",
                h3 { class: "text-base font-semibold text-gray-900", "{title}" }
                p { class: "text-sm text-gray-500 mt-1 leading-relaxed", "{description}" }
            }

            div { class: "flex items-center gap-3 shrink-0",
                span {
                    class: "text-xs font-medium px-2 py-1 rounded-full",
                    class: if enabled { "bg-(--color-forest-light) text-(--color-forest-dark)" } else { "bg-gray-100 text-gray-500" },
                    if enabled {
                        "{on_label}"
                    } else {
                        "{off_label}"
                    }
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
        div { class: "px-5 py-4",
            h3 { class: "text-base font-semibold text-gray-900",
                "معلومات إضافية مطلوبة"
            }
            p { class: "text-sm text-gray-500 mt-1 leading-relaxed",
                "تظهر هذه الحقول في نموذج الإضافة بمفتاح ثابت، ولا يمكن إرسال الطلب دون تعبئتها."
            }

            div { class: "space-y-2 mt-3 max-w-xl",
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

                if keys.is_empty() {
                    p { class: "text-sm text-gray-400", "لا توجد حقول إضافية مطلوبة." }
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

    let required_labels = || (String::from("مطلوب"), String::from("اختياري"));

    let body = match &*settings_resource.read() {
        Some(Ok(settings)) => {
            let settings = settings.clone();
            let rules = settings.add_request_rules.clone();

            rsx! {
                SettingsSection {
                    icon: "fa-door-open",
                    title: "صفحة الإضافة العامة",
                    description: "الرابط الذي يستخدمه أفراد العائلة لإرسال طلبات إضافة.",

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

                SettingsSection {
                    icon: "fa-list-check",
                    title: "الحقول المطلوبة في طلبات الإضافة",
                    description: "الاسم الأول واسم العائلة والجنس والوالد مطلوبة دائماً.",

                    SettingToggle {
                        title: "تاريخ الولادة",
                        description: "إلزام مُقدّم الطلب بإدخال تاريخ ولادة الفرد وكل طفل.",
                        enabled: rules.require_birthday,
                        busy: save.pending(),
                        state_labels: required_labels(),
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
                        description: "إلزام مُقدّم الطلب بإرفاق صورة شخصية للفرد وكل طفل. لا ينطبق هذا على الإناث، فالصورة تبقى اختيارية لهنّ دائماً.",
                        enabled: rules.require_image,
                        busy: save.pending(),
                        state_labels: required_labels(),
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
                }

                if let Some(Err(e)) = save.value() {
                    div { class: "rounded-lg bg-red-50 text-red-700 px-4 py-3",
                        "تعذّر حفظ الإعدادات: {e}"
                    }
                }
            }
        }
        Some(Err(e)) => {
            rsx! {
                div { class: "rounded-lg bg-red-50 text-red-700 px-4 py-3",
                    "تعذّر تحميل الإعدادات: {e}"
                }
            }
        }
        None => {
            rsx! {
                div { class: "text-center text-gray-500 py-12", "جارٍ التحميل..." }
            }
        }
    };

    rsx! {
        div { dir: "rtl", class: "w-full",
            div { class: "max-w-4xl mx-auto space-y-5",
                div {
                    h1 { class: "text-2xl lg:text-3xl font-bold text-(--color-forest-dark)",
                        "الإعدادات"
                    }
                    p { class: "text-gray-600 mt-1",
                        "تحكّم في صفحة الإضافة العامة وفي الحقول التي يلزم تعبئتها في الطلبات."
                    }
                }

                {body}
            }
        }
    }
}
