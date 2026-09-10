use dioxus::{fullstack::FileStream, prelude::*};
use jiff::{ToSpan, tz::TimeZone};

use crate::{
    Route,
    config::client::Config,
    modules::{
        add_request::{server::member_requests, types::RequestStatus},
        admin::{
            components::{
                ShowModal,
                add_member_modal::AddMemberModal,
                add_requests_grid::AddRequestsGrid,
                members_grid::MembersGrid,
                spouse_fields::{SpouseSubmission, apply_spouse},
            },
            server::get_admin,
            types::MemberFormData,
        },
        invite::server::create_invite,
        member::server::{add_member, edit_member_image, members_flat},
        user::server::logout_user,
    },
    ui::modal::Modal,
};

pub mod invites;
pub mod login;
pub mod register;
pub mod settings;

enum Tab {
    Members,
    Requests,
}

#[component]
fn Sidebar(show_sidebar: Signal<bool>) -> Element {
    rsx! {
        div {
            class: if !show_sidebar() { "md:translate-x-0 translate-x-full" } else { "md:translate-x-0 translate-x-0" },
            class: "h-full w-full md:w-auto md:flex lg:flex flex-col max-sm:fixed right-0 z-10 bg-white px-10 md:px-8 py-10 md:py-5 space-y-4 shadow-lg rounded-l-lg text-nowrap transition-transform duration-150 ease-in-out",

            Link {
                to: Route::Admin {},
                class: "text-center text-4xl md:text-base block px-4 py-3 rounded-lg text-forest-dark hover:bg-forest-light transition-colors duration-200 cursor-pointer",
                "الأفراد"
            }

            Link {
                to: Route::AdminInvites {},
                class: "text-center text-4xl md:text-base block px-4 py-3 rounded-lg text-forest-dark hover:bg-forest-light transition-colors duration-200 cursor-pointer",
                "الدعوات"
            }

            Link {
                to: Route::AdminSettings {},
                class: "text-center text-4xl md:text-base block px-4 py-3 rounded-lg text-forest-dark hover:bg-forest-light transition-colors duration-200 cursor-pointer",
                "الإعدادات"
            }
        }
    }
}

#[component]
pub fn AdminLayout() -> Element {
    let mut show_sidebar = use_signal(|| false);
    let show_modal = use_context_provider(|| Signal::new(None::<ShowModal>));
    let admin_future = use_server_future(get_admin)?;

    let admin = admin_future.unwrap()?;

    rsx! {
        div {
            dir: "rtl",
            class: "sticky top-0 right-0 z-10 md:hidden! bg-white/90 p-4",
            button {
                class: "btn btn-primary shadow-md hover:shadow-lg transition-all duration-200",
                onclick: move |_| {
                    show_sidebar.toggle();
                },
                i { class: "fa-solid fa-bars" }
            }
        }
        div { dir: "rtl", class: "flex gap-4 h-full",

            Sidebar { show_sidebar }
            div { class: "w-full",

                div { class: "card card-forest fade-in mb-3",
                    div { class: "card-body",
                        div { class: "flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4",
                            div {

                                h1 { class: "text-center md:text-start text-2xl lg:text-3xl font-bold text-forest-dark arabic-heading",
                                    "مرحبا، {admin.first_name}!"
                                }
                                p { class: "text-center md:text-start text-gray-600 mt-1 arabic-text",
                                    "إدارة أفراد العائلة والطلبات"
                                }
                            }
                            div { class: "flex sm:flex-row gap-2",
                                Invite { show_modal }
                                Link {
                                    class: "btn btn-outline btn-sm",
                                    to: Route::Home,
                                    i { class: "fa-brands fa-linktree" }
                                    "عرض الشجرة"
                                }
                                button {
                                    class: "btn btn-danger btn-sm",
                                    onclick: move |_| async move {
                                        let _ = logout_user().await;
                                        let _ = navigator().replace(Route::Home);
                                    },
                                    svg {
                                        class: "w-4 h-4",
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

                Outlet::<Route> {
                }
            }
        }
    }
}

#[component]
fn Invite(show_modal: Signal<Option<ShowModal>>) -> Element {
    let mut date = use_signal(|| None::<jiff::civil::Date>);

    let mut create =
        use_action(move |date: Option<jiff::Zoned>| async move { create_invite(date).await });

    let config = use_context::<Config>();

    rsx! {
        button {
            class: "btn btn-primary",
            onclick: move |_| {
                show_modal.set(Some(ShowModal::UserInvite));
            },
            "دعوة للشجرة"
        }

        Modal {
            show: matches!(&*show_modal.read(), Some(ShowModal::UserInvite)),
            title: "دعوة".to_string(),
            max_width: Some("2xl".to_string()),
            on_close: move |_| show_modal.set(None),
            div { class: "space-y-6",
                div {

                    div { class: "flex flex-col mb-3",
                        span { "تنتهي بعد:" }
                        input {
                            class: "input",
                            r#type: "date",
                            value: date().map(|d| d.to_string()).unwrap_or_else(|| String::from("")),
                            onchange: move |evt| {
                                if let Ok(data) = evt.parsed::<jiff::civil::Date>() {
                                    date.set(Some(data));
                                }
                            },
                        }
                    }
                    div { class: "flex gap-2",
                        button {
                            class: "btn btn-sm btn-primary",
                            onclick: move |_| {
                                date.set(None);
                            },
                            "أبداً"
                        }
                        button {
                            class: "btn btn-sm btn-primary",
                            onclick: move |_| {
                                date.set(Some(jiff::Timestamp::now().to_zoned(TimeZone::UTC).date() + 1.day()));
                            },
                            "بعد يوم"
                        }
                        button {
                            class: "btn btn-sm btn-primary",
                            onclick: move |_| {
                                date.set(Some(jiff::Timestamp::now().to_zoned(TimeZone::UTC).date() + 1.week()));
                            },
                            "بعد أسبوع"
                        }
                        button {
                            class: "btn btn-sm btn-primary",
                            onclick: move |_| {
                                date.set(
                                    Some(jiff::Timestamp::now().to_zoned(TimeZone::UTC).date() + 1.month()),
                                );
                            },
                            "بعد شهر"
                        }
                    }
                }

                match create.value() {
                    Some(Ok(token)) => rsx! {
                        div { class: "rounded-lg border border-green-200 bg-green-50 p-4 space-y-3",
                            div { class: "flex items-center gap-2 text-green-700 font-medium arabic-text",
                                i { class: "fa-solid fa-circle-check" }
                                span { "تم إنشاء الدعوة بنجاح" }
                            }

                            div { class: "flex items-stretch gap-2",
                                input {
                                    readonly: true,
                                    dir: "ltr",
                                    class: "input flex-1 min-w-0 bg-white font-mono text-sm text-gray-700 truncate",
                                    value: "{config.base_url}register?invite_token={token}",
                                }
                                button {
                                    class: "btn btn-primary shrink-0",
                                    title: "نسخ الرابط",
                                    "onclick": "navigator.clipboard.writeText(\"{config.base_url}register?invite_token={token}\");",
                                    i { class: "fa-solid fa-copy" }
                                }
                            }

                            p { class: "text-xs text-green-600 arabic-text",
                                "هذا الرابط صالح لمدة يوم واحد."
                            }
                        }
                    },
                    Some(Err(e)) => rsx! {
                        div { class: "rounded-lg bg-red-50 text-red-700 px-4 py-3 arabic-text",
                            "تعذّر إنشاء الدعوة: {e}"
                        }
                    },
                    None => rsx! {},
                }

                div { class: "flex justify-end",
                    button {
                        class: "btn btn-primary",
                        disabled: create.pending(),
                        onclick: move |_| {
                            let date = date()
                                .and_then(|d| d.to_zoned(TimeZone::UTC).ok());

                            create.call(date);
                        },
                        if create.pending() {
                            "جارٍ الإنشاء..."
                        } else {
                            "إنشاء دعوة"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn Admin() -> Element {
    let mut members_resource = use_resource(members_flat);

    let add_requests_resource = use_resource(member_requests);

    let mut error = use_signal(String::new);
    let mut tab = use_signal(|| Tab::Members);
    let mut show_modal = use_context::<Signal<Option<ShowModal>>>();

    let members_list = (*members_resource.read())
        .as_ref()
        .and_then(|e| e.as_ref().ok())
        .cloned()
        .unwrap_or_default();

    let members_count = members_list.len();

    let add_requests_count = (*add_requests_resource.read())
        .as_ref()
        .and_then(|e| {
            e.as_ref()
                .map(|r| {
                    r.iter()
                        .filter(|r| matches!(r.status, RequestStatus::Pending))
                        .count()
                })
                .ok()
        })
        .unwrap_or(0);

    let mut on_member_add = use_action(
        move |(data, image, spouse): (
            MemberFormData,
            Option<FileStream>,
            Option<SpouseSubmission>,
        )| async move {
            let Some(gender) = data.gender else {
                anyhow::bail!("Gender must be set");
            };

            let id = add_member(
                data.name,
                data.last_name,
                data.father_id,
                data.mother_id,
                gender,
                data.birthday,
            )
            .await?;
            if let Some(image) = image {
                edit_member_image(id, image).await?;
            }
            if let Some(spouse) = spouse {
                apply_spouse(id, spouse).await?;
            }
            members_resource.restart();
            show_modal.set(None);

            Ok(())
        },
    );

    rsx! {
        div { class: "w-full", dir: "rtl",
            div { class: "text-red-500", "{error}" }

            div { class: "flex flex-col md:flex-row gap-4 slide-in",
                div {
                    class: "card cursor-pointer hover:ring-2 hover:ring-forest-light transition-all duration-200 group relative flex-1",
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

                // Pending Requests Card - Enhanced with Urgency -->
                div {
                    class: "card cursor-pointer transition-all duration-200 group relative overflow-hidden flex-1",
                    class: if add_requests_count > 0 { "hover:ring-2 hover:ring-amber-400 border-r-4 border-r-amber-400 bg-gradient-to-r from-amber-50 to-white" },
                    class: if add_requests_count == 0 { "hover:ring-2 hover:ring-gray-300" },
                    onclick: move |_| {
                        tab.set(Tab::Requests);
                    },

                    if add_requests_count > 0 {
                        // Urgency indicator for pending requests -->
                        div { class: "absolute top-0 right-0 w-0 h-0 border-l-[25px] border-l-transparent border-t-[25px] border-t-amber-400",
                            span { class: "absolute -top-5 right-1 text-white text-xs font-bold rotate-[-45deg]",
                                "!"
                            }
                        }
                    }

                    div { class: "card-body text-center py-2",
                        // Hover indicator -->
                        div { class: "absolute top-2 left-2 opacity-0 group-hover:opacity-100 transition-opacity duration-200",
                            svg {
                                class: "w-3 h-3",
                                class: if add_requests_count > 0 { "text-amber-600" } else { "text-gray-400" },
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M13 7l5 5m0 0l-5 5m5-5H6",
                                }
                            }
                        }

                        div {
                            class: "flex items-center justify-center w-10 h-10 rounded-lg mx-auto mb-1 transition-colors duration-200",
                            class: if add_requests_count > 0 { "bg-amber-100 group-hover:bg-amber-200" } else { "bg-gray-100 group-hover:bg-gray-200" },
                            svg {
                                class: "w-5 h-5",
                                class: if add_requests_count > 0 { "text-amber-600" } else { "text-gray-400" },
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.746 0 3.332.477 4.5 1.253v13C19.832 18.477 18.246 18 16.5 18c-1.746 0-3.332.477-4.5 1.253",
                                }
                            }
                        }
                        h3 {
                            class: "text-xl font-bold",
                            class: if add_requests_count > 0 { "text-amber-600" } else { "text-gray-900" },
                            "{add_requests_count}"
                        }
                        p { class: "text-sm text-gray-600 arabic-text", "طلبات الإضافة" }

                        // Status indicators
                        if add_requests_count > 0 {
                            div { class: "mt-1",
                                span { class: "inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-amber-100 text-amber-800 arabic-text",
                                    "يحتاج مراجعة فورية"
                                }
                            }
                        } else {
                            div { class: "mt-1",
                                span { class: "text-xs text-gray-500 opacity-0 group-hover:opacity-100 transition-opacity duration-200 arabic-text",
                                    "انقر للمراجعة"
                                }
                            }
                        }
                    }
                }
            }

            AddMemberModal {
                show: show_modal().is_some_and(|m| matches!(m, ShowModal::AddMember)),
                members: members_list.clone(),
                on_close: move |_| {
                    show_modal.set(None);
                },
                on_submit: move |args: (
                    MemberFormData,
                    Option<FileStream>,
                    Option<SpouseSubmission>,
                )| on_member_add.call(args),
            }

            match &*tab.read() {
                Tab::Members => rsx! {
                    MembersGrid { show_modal, members_resource }
                },
                Tab::Requests => rsx! {
                    AddRequestsGrid { show_modal, members_resource, add_requests_resource }
                },
            }
        }
    }
}
