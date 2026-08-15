use dioxus::prelude::*;
use jiff::{Zoned, tz::TimeZone};
use uuid::Uuid;

use crate::modules::invite::server::{delete_invite, get_invites};

fn fmt_dt(dt: &Zoned) -> String {
    dt.strftime("%Y-%m-%d %H:%M").to_string()
}

fn fmt_opt(dt: Option<&Zoned>) -> String {
    dt.map(fmt_dt).unwrap_or_else(|| "—".to_string())
}

fn short_id(id: &Uuid) -> String {
    id.to_string().chars().take(8).collect()
}

#[component]
pub fn AdminInvites() -> Element {
    let mut invites_resource = use_resource(get_invites);

    let mut delete_action = use_action(move |id: i64| async move {
        delete_invite(id).await?;

        invites_resource.restart();

        anyhow::Ok(())
    });

    let body = match &*invites_resource.read() {
        Some(Ok(invites)) => {
            if invites.is_empty() {
                rsx! {
                    div { class: "text-center text-gray-500 py-12 arabic-text",
                        "لا توجد دعوات بعد."
                    }
                }
            } else {
                let now = jiff::Timestamp::now().to_zoned(TimeZone::UTC);
                rsx! {
                    div { class: "overflow-x-auto",
                        table { class: "w-full text-sm text-right border-collapse",
                            thead {

                                tr { class: "border-b border-gray-200 text-gray-600",
                                    th { class: "px-3 py-2 font-medium", "#" }
                                    th { class: "px-3 py-2 font-medium", "الحالة" }
                                    th { class: "px-3 py-2 font-medium", "تاريخ الإنشاء" }
                                    th { class: "px-3 py-2 font-medium", "تاريخ الانتهاء" }
                                    th { class: "px-3 py-2 font-medium",
                                        "تاريخ الاستخدام"
                                    }
                                    th { class: "px-3 py-2 font-medium", "المُستخدِم" }
                                    th { class: "px-3 py-2 font-medium", "أنشأها" }
                                    th { class: "px-3 py-2 font-medium", "الرمز" }
                                }
                            }
                            tbody {

                                for invite in invites {
                                    {
                                        let (status_label, status_class) = if invite.used_at.is_some() {
                                            ("مستخدمة", "bg-blue-100 text-blue-800")
                                        } else if invite.expires_at.as_ref().is_some_and(|e| e < &now) {
                                            ("منتهية", "bg-red-100 text-red-700")
                                        } else {
                                            ("نشطة", "bg-green-100 text-green-700")
                                        };
                                        rsx! {
                                            tr {
                                                key: "{invite.id}",
                                                class: "border-b border-gray-100 hover:bg-gray-50 transition-colors",
                                                td { class: "px-3 py-2 text-gray-500", "{invite.id}" }
                                                td { class: "px-3 py-2",
                                                    span { class: "inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium arabic-text {status_class}",
                                                        "{status_label}"
                                                    }
                                                }
                                                td { class: "px-3 py-2 whitespace-nowrap", "{fmt_dt(&invite.created_at)}" }
                                                td { class: "px-3 py-2 whitespace-nowrap",
                                                    "{invite.expires_at.as_ref().map(fmt_dt).unwrap_or_else(|| String::from(\"لا تنتهي\"))}"
                                                }
                                                td { class: "px-3 py-2 whitespace-nowrap", "{fmt_opt(invite.used_at.as_ref())}" }
                                                td {
                                                    class: "px-3 py-2 font-mono text-xs text-gray-600",
                                                    title: invite.accepted_by.map(|u| u.to_string()),
                                                    "{invite.accepted_by.as_ref().map(short_id).unwrap_or_else(|| String::from(\"—\"))}"
                                                }
                                                td {
                                                    class: "px-3 py-2 font-mono text-xs text-gray-600",
                                                    title: "{invite.created_by}",
                                                    "{short_id(&invite.created_by)}"
                                                }
                                                td {
                                                    class: "px-3 py-2 font-mono text-xs text-gray-400",
                                                    title: "{invite.token_hash}",
                                                    "{invite.token_hash.chars().take(10).collect::<String>()}…"
                                                }
                                                td { class: "px-3 py-2 whitespace-nowrap",
                                                    button {
                                                        class: "btn btn-danger",
                                                        onclick: {
                                                            let id = invite.id;
                                                            move |_| {
                                                                delete_action.call(id);
                                                            }
                                                        },
                                                        "حذف"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Some(Err(e)) => {
            rsx! {
                div { class: "rounded-lg bg-red-50 text-red-700 px-4 py-3 arabic-text",
                    "تعذّر تحميل الدعوات: {e}"
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
                        "الدعوات"
                    }
                    {body}
                }
            }
        }
    }
}
