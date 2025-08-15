use dioxus::prelude::*;
use dioxus_primitives::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger,
};
use strum::IntoEnumIterator;

use crate::{
    Route,
    modules::member::{
        server::{add_member, delete_member, members_flat},
        types::{Gender, MemberResponseBrief},
    },
};

use super::server::get_admin;

pub mod login;
pub mod register;

enum Tab {
    Members,
    Requests,
}

#[component]
pub fn Admin() -> Element {
    let mut members_resource = use_resource(members_flat);
    let admin_future = use_server_future(get_admin)?;

    use_effect(move || {
        if let Some(Err(e)) = admin_future() {
            navigator().replace(Route::Home);
        }
    });

    let admin = match admin_future() {
        Some(Ok(admin)) => admin,
        Some(Err(_)) => {
            return rsx! {
                "Unauthorized."
            };
        }
        None => {
            return rsx! {
                p { "Loading..." }
            };
        }
    };

    let mut error = use_signal(|| String::new());

    let mut name = use_signal(|| String::new());
    let mut last_name = use_signal(|| String::new());
    let mut gender = use_signal(|| None::<Gender>);

    let mut tab = use_signal(|| Tab::Members);

    let genders = Gender::iter().enumerate().map(|(i, o)| {
        rsx! {
            DropdownMenuItem::<Gender> {
                class: "dropdown-menu-item",
                value: o,
                index: i,
                on_select: move |value| {
                    gender.set(Some(value));
                },
                {match o {
                    Gender::Male => {
                        "ذكر"
                    },
                    Gender::Female => {
                        "انثى"
                    }
                }}
            }
        }
    });

    let members_grid = match &*members_resource.read() {
        Some(Ok(members)) => {
            rsx! {
                if members.is_empty() {
                    div { class:"col-span-full text-center py-12",
                      svg { class:"w-16 h-16 mx-auto text-gray-400 mb-4", fill:"none", stroke:"currentColor", view_box:"0 0 24 24",
                        path { stroke_linecap:"round", stroke_linejoin:"round", stroke_width:"2", d:"M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"}
                      }
                      h3 { class:"text-lg font-medium text-gray-900 mb-2", "لا يوجد أعضاء"}
                      p { class:"text-gray-500", "لم يتم العثور على أعضاء مطابقين للبحث"}
                    }
                } else {
                    for member in members {
                            MemberCard {
                                member: member.clone(),
                                on_view: |_| {
                                    tracing::info!("on view");
                                },
                                on_edit: |_| {
                                    tracing::info!("on edit");
                                },
                                on_invite: |_| {
                                    tracing::info!("on invite");
                                },
                                on_delete: |_| {
                                    tracing::info!("on delete");
                                },
                            }

                    }
                }

            }
        }
        Some(Err(e)) => {
            return rsx! {
                p { "Error: {e}" }
            };
        }
        None => {
            return rsx! {
                p { "جاري التحميل..." }
            };
        }
    };

    rsx! {
        div {
            class: "text-red-500",
            "{error}"
        }

        div { class: "card card-forest fade-in",
            div { class: "card-body",
                div { class: "flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4",
                    div {
                        h1 { class: "text-2xl lg:text-3xl font-bold text-forest-dark arabic-heading",
                            "مرحبا، {admin.first_name}!"
                        }
                        p { class: "text-gray-600 mt-1 arabic-text",
                            "إدارة أفراد العائلة والطلبات"
                        }
                    }
                    div { class: "flex flex-col sm:flex-row gap-2",
                        Link { class: "btn btn-outline btn-sm",
                            to: Route::Home,
                            svg {
                                class: "w-4 h-4",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    d: "M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2H5a2 2 0 00-2 2v0",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                }
                            }
                            "عرض الشجرة"
                        }
                        button { class: "btn btn-danger btn-sm",
                            onclick: move |_| {
                                tracing::info!("alo");
                            },

                            svg { class: "w-4 h-4",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    d: "M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                }
                            }

                            "تسجيل الخروج"
                        }
                    }
                }
            }
        }

        div {
            input {
                class: "input",
                placeholder: "الاسم",
                oninput: move |e| name.set(e.value())
            }
            input {
                class: "input",
                placeholder: "اسم العائلة",
                oninput: move |e| last_name.set(e.value())
            }
            DropdownMenu { class: "dropdown-menu", default_open: false,
                DropdownMenuTrigger {
                    class: "dropdown-menu-trigger",
                    match gender() {
                        Some(Gender::Male) => "ذكر",
                        Some(Gender::Female) => "انثى",
                        None => "الجنس"
                    }
                }
                DropdownMenuContent { class: "dropdown-menu-content",
                    {genders}
                }
            }
        }

        button {
            class: "button",
            "data-style": "primary",

            onclick: move |_| async move {
                let Some(gender) = gender() else {
                    error.set(String::from("اختر جنس فرد العائلة"));
                    return;
                };

                let _ = add_member(name(), last_name(), gender, chrono::Utc::now()).await;
                members_resource.restart();
            },
            "إضافة العضو"
        }

        div { class: "card cursor-pointer hover:ring-2 hover:ring-forest-light transition-all duration-200 group relative flex-1",
            onclick: move |_| {
                tab.set(Tab::Members);
            },
            div { class: "card-body text-center py-2",
                div { class: "absolute top-2 left-2 opacity-0 group-hover:opacity-100 transition-opacity duration-200",
                    svg {
                        class: "w-3 h-3 text-forest-primary",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            d: "M13 7l5 5m0 0l-5 5m5-5H6",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                        }
                    }
                }
                div { class: "flex items-center justify-center w-10 h-10 bg-forest-light rounded-lg mx-auto mb-1 group-hover:bg-forest-primary transition-colors duration-200",
                    svg {
                        class: "w-5 h-5 text-forest-dark group-hover:text-white transition-colors duration-200",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            d: "M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                        }
                    }
                }
                h3 { class: "text-xl font-bold text-gray-900", "2313" }
                p { class: "text-sm text-gray-600 arabic-text", "إجمالي الأفراد" }
                div { class: "mt-1 opacity-0 group-hover:opacity-100 transition-opacity duration-200",
                    span { class: "text-xs text-forest-primary font-medium arabic-text",
                        "انقر للإدارة"
                    }
                }
            }
        }

        match &*tab.read() {
            Tab::Members => rsx! {
                div {
                    class:"grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-2 pb-6",
                    id:"members-grid",

                    {members_grid}
                }
            },
            Tab::Requests => rsx! {}
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct MemberCardProps {
    member: MemberResponseBrief,
    on_view: EventHandler<MemberResponseBrief>,
    on_edit: EventHandler<i64>,
    on_invite: EventHandler<(i64, String)>,
    on_delete: EventHandler<i64>,
}

#[component]
pub fn MemberCard(props: MemberCardProps) -> Element {
    let mut show_invite_input = use_signal(|| false);
    let mut email = use_signal(|| String::new());

    rsx! {
        div { class:"relative h-32 image-container",
            if let Some(image_data) = &props.member.image {
              img {
                class:"w-full h-full object-cover",
                src:r#"data:{props.member.image_type.clone().unwrap_or(String::from("image/jpeg"))};base64,{image_data:?}"#,
                alt:"{props.member.name} {props.member.last_name}",
                loading:"lazy",
                // onerror:"this.style.display:'none'; this.nextElementSibling.style.display:'flex';",
              }
              div { class:"w-full h-full items-center justify-center text-white font-bold text-2xl",
                   style:"display: none; background: linear-gradient(135deg, #7A9A7A, #2C4A2C);",
                "{props.member.name}"
              }
            } else {
              div { class:"w-full h-full flex items-center justify-center text-white font-bold text-2xl bg-gradient-to-br from-gray-500 to-gray-600",
                "{props.member.name} {props.member.last_name}"
              }
            }

            div { class:"absolute top-3 right-3",
                match props.member.gender {
                    Gender::Male => rsx!{
                        span { class:"inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800",
                            svg { class:"w-3 h-3 ml-1", fill:"currentColor", view_box:"0 0 20 20",
                              path { d:"M10 2L3 7v11a2 2 0 002 2h10a2 2 0 002-2V7l-7-5z" }
                            }
                            "ذكر"
                        }
                    },
                    Gender::Female => rsx! {
                          span { class:"inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-pink-100 text-pink-800",
                            svg { class:"w-3 h-3 ml-1", fill:"currentColor", view_box:"0 0 20 20",
                              path { d:"M10 2L3 7v11a2 2 0 002 2h10a2 2 0 002-2V7l-7-5z" }
                            }
                            "أنثى"
                          }
                    }
                }
            }

            div { class: "p-4",
                div { class: "mb-3",
                    h3 { class: "text-lg font-semibold text-gray-900 truncate",
                        "{props.member.name} {props.member.last_name}"
                    }
                    p { class: "text-sm text-gray-500",
                        "رقم العضوية: {props.member.id}"
                    }
                }

                // Birthday section
                if let Some(birthday) = &props.member.birthday {
                    div { class: "flex items-center text-sm text-gray-600 mb-2",
                        svg {
                            class: "w-4 h-4 ml-1",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M8 7V3a2 2 0 012-2h4a2 2 0 012 2v4m-6 0h6l1 12H7L8 7z"
                            }
                        }
                        span { "{birthday}" }
                    }
                }

                div { class: "space-y-1 mb-3",
                    // Father ID section
                    if let Some(father_id) = props.member.father_id {
                        div { class: "flex items-center text-xs text-gray-600",
                            svg {
                                class: "w-3 h-3 ml-1",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
                                }
                            }
                            span { "الوالد: ID {father_id}" }
                        }
                    }

                    // Mother ID section
                    if let Some(mother_id) = props.member.mother_id {
                        div { class: "flex items-center text-xs text-gray-600",
                            svg {
                                class: "w-3 h-3 ml-1",
                                fill: "currentColor",
                                view_box: "0 0 20 20",
                                path { d: "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" }
                            }
                            span { "الوالدة: ID {mother_id}" }
                        }
                    }
                }

                // Personal info section
                if let Some(personal_info) = &props.member.personal_info {
                    if !personal_info.is_empty() {
                        div { class: "mb-3",
                            div { class: "flex flex-wrap gap-1",
                                span { class: "inline-flex items-center px-2 py-1 rounded-full text-xs bg-gray-100 text-gray-700",
                                    "معلومات إضافية"
                                }
                            }
                        }
                    }
                }

                div { class: "pt-3 border-t border-gray-100",
                    // Invite input section (conditionally shown)
                    if show_invite_input() {
                        div { class: "mb-3",
                            div { class: "flex gap-2",
                                input {
                                    r#type: "email",
                                    class: "form-input text-sm flex-1",
                                    placeholder: "البريد الإلكتروني",
                                    value: "{email}",
                                    oninput: move |evt| email.set(evt.value()),
                                    onkeypress: move |evt| {
                                        if evt.key() == Key::Enter {
                                            props.on_invite.call((props.member.id, email()));
                                            show_invite_input.set(false);
                                            email.set(String::new());
                                        }
                                    }
                                }
                                button {
                                    class: "btn btn-success btn-sm",
                                    disabled: !email().contains('@'),
                                    onclick: move |_| {
                                        props.on_invite.call((props.member.id, email()));
                                        show_invite_input.set(false);
                                        email.set(String::new());
                                    },
                                    "إرسال"
                                }
                                button {
                                    class: "btn btn-outline btn-sm",
                                    onclick: move |_| {
                                        show_invite_input.set(false);
                                        email.set(String::new());
                                    },
                                    "إلغاء"
                                }
                            }
                        }
                    }

                    // Action buttons
                    div { class: "flex gap-2",
                        // View button
                        button {
                            class: "flex-1 btn btn-outline btn-sm",
                            title: "عرض التفاصيل",
                            onclick: {
                                let member = props.member.clone();
                                move |_| props.on_view.call(member.clone())
                            },
                            svg {
                                class: "w-4 h-4",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                                }
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
                                }
                            }
                            "عرض"
                        }

                        // Edit button
                        button {
                            class: "flex-1 btn btn-secondary btn-sm",
                            title: "تحرير",
                            onclick: {
                                let member_id = props.member.id;
                                move |_| props.on_edit.call(member_id)
                            },
                            svg {
                                class: "w-4 h-4",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
                                }
                            }
                            "تحرير"
                        }

                        // Invite button
                        button {
                            class: "btn btn-success btn-sm",
                            title: "دعوة",
                            onclick: move |_| show_invite_input.set(!show_invite_input()),
                            svg {
                                class: "w-4 h-4",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M3 8l7.89 4.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
                                }
                            }
                        }

                        // Delete button
                        button {
                            class: "btn btn-danger btn-sm",
                            title: "حذف",
                            onclick: move |_| props.on_delete.call(props.member.id),
                            svg {
                                class: "w-4 h-4",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
