use dioxus::prelude::*;

use crate::modules::member::types::{Gender, MemberResponseFlat};

#[derive(Props, PartialEq, Clone)]
pub struct MemberCardProps {
    member: MemberResponseFlat,
    on_view: EventHandler<MemberResponseFlat>,
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
                            class: "button flex-1",
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
                            class: "button flex-1",
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
                            class: "button",
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
                            class: "button",
                            "data-style": "destructive",
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
