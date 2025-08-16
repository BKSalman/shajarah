use dioxus::prelude::*;
use dioxus_primitives::dropdown_menu::DropdownMenuItem;
use strum::IntoEnumIterator;

use crate::{
    Route,
    modules::{
        admin::components::{
            add_member_modal::AddMemberModal, family_management_header::FamilyManagementHeader,
            member_card::MemberCard, view_member_modal::ViewMemberModal,
        },
        member::{server::members_flat, types::Gender},
    },
};

use super::server::get_admin;

pub mod login;
pub mod register;

enum Tab {
    Members,
    Requests,
}

#[derive(Clone, Debug)]
enum ShowModal {
    ViewMember,
    AddMember,
    EditMember,
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
                p { "جاري التحميل..." }
            };
        }
    };

    let mut error = use_signal(|| String::new());

    let mut name = use_signal(|| String::new());
    let mut last_name = use_signal(|| String::new());
    let mut gender = use_signal(|| None::<Gender>);

    let mut tab = use_signal(|| Tab::Members);
    let mut show_modal = use_signal(|| None::<ShowModal>);

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

    let members_count = (&*members_resource.read())
        .as_ref()
        .and_then(|e| e.as_ref().map(|e| e.len()).ok())
        .unwrap_or(0);

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
                            ViewMemberModal {
                                show: show_modal().is_some_and(|m| matches!(m, ShowModal::ViewMember)),
                                member: member.clone(),
                                on_close: move |_| {
                                    show_modal.set(None);
                                },
                                on_edit: move |_| {
                                    show_modal.set(Some(ShowModal::EditMember));
                                }
                            }

                            MemberCard {
                                member: member.clone(),
                                on_view: move |_| {
                                    tracing::info!("on view");
                                    show_modal.set(Some(ShowModal::ViewMember));
                                    tracing::info!("{show_modal:?}");
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
                h3 { class: "text-xl font-bold text-gray-900", "{members_count}" }
                p { class: "text-sm text-gray-600 arabic-text", "إجمالي الأفراد" }
                div { class: "mt-1 opacity-0 group-hover:opacity-100 transition-opacity duration-200",
                    span { class: "text-xs text-forest-primary font-medium arabic-text",
                        "انقر للإدارة"
                    }
                }
            }
        }

        AddMemberModal {
            show: show_modal().is_some_and(|m| matches!(m, ShowModal::AddMember)),
            on_close: move |_| {
                show_modal.set(None);
            },
            on_submit: |_| {}
        }

        match &*tab.read() {
            Tab::Members => rsx! {
                FamilyManagementHeader {
                    members_count: members_count,
                    on_add_member: move |_| {
                        show_modal.set(Some(ShowModal::AddMember));
                    },
                    on_csv_upload: move |e| {},
                }

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
