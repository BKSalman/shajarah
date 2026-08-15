use dioxus::prelude::*;

use crate::modules::settings::{
    server::{get_settings, update_settings},
    types::Settings,
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
pub fn AdminSettings() -> Element {
    let mut settings_resource = use_resource(get_settings);

    let mut save = use_action(move |settings: Settings| async move {
        update_settings(settings).await?;

        settings_resource.restart();

        anyhow::Ok(())
    });

    let body = match &*settings_resource.read() {
        Some(Ok(settings)) => {
            let settings = *settings;
            rsx! {
                div { class: "divide-y divide-gray-100",
                    SettingToggle {
                        title: "صفحة إضافة الأفراد",
                        description: "عند التعطيل، يصبح رابط /add غير متاح لأفراد العائلة ولا يمكن إرسال طلبات إضافة جديدة.",
                        enabled: settings.add_page_enabled,
                        busy: save.pending(),
                        on_toggle: move |enabled| {
                            save.call(Settings {
                                add_page_enabled: enabled,
                            });
                        },
                    }
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
