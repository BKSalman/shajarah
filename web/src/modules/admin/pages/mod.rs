use super::server::get_admin;
use crate::{
    Route,
    modules::{
        add_request::{
            server::{approve_request, disapprove_request, member_requests},
            types::RequestStatus,
        },
        admin::{
            components::{
                add_member_modal::AddMemberModal, edit_member_modal::EditMemberModal,
                family_management_header::FamilyManagementHeader, member_card::MemberCard,
                request_card::RequestCard, view_member_modal::ViewMemberModal,
                view_request_modal::ViewRequestModal,
            },
            server::logout_admin,
            types::{EditMemberFormData, MemberFormData},
        },
        member::server::{
            add_member, delete_member, edit_member, members_flat, upload_members_csv,
        },
    },
};
use dioxus::prelude::*;
use uuid::Uuid;

pub mod login;
pub mod register;

enum Tab {
    Members,
    Requests,
}

#[derive(Clone, PartialEq, Debug)]
enum ShowModal {
    ViewMember(i64),
    ViewRequest(Uuid),
    AddMember,
    EditMember(i64),
}

#[component]
pub fn Admin() -> Element {
    let mut members_resource = use_resource(members_flat);
    let mut add_requests_resource = use_resource(member_requests);
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
                p { "Unauthorized." }
            };
        }
        None => {
            return rsx! {
                p { "جاري التحميل..." }
            };
        }
    };

    let mut error = use_signal(|| String::new());
    let mut tab = use_signal(|| Tab::Members);
    let mut show_modal = use_signal(|| None::<ShowModal>);

    let members_count = (&*members_resource.read())
        .as_ref()
        .and_then(|e| e.as_ref().map(|e| e.len()).ok())
        .unwrap_or(0);

    let add_requests_count = (&*add_requests_resource.read())
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

    let members_grid = match &*members_resource.read() {
        Some(Ok(members)) => {
            rsx! {
                if members.is_empty() {
                    div { class: "col-span-full text-center py-12",
                        svg {
                            class: "w-16 h-16 mx-auto text-gray-400 mb-4",
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
                        h3 { class: "text-lg font-medium text-gray-900 mb-2",
                            "لا يوجد أعضاء"
                        }
                        p { class: "text-gray-500",
                            "لم يتم العثور على أعضاء مطابقين للبحث"
                        }
                    }
                } else {
                    div {
                        class: "grid grid-cols-1 md:grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-2 pb-6",
                        id: "members-grid",
                        for member in members {
                            ViewMemberModal {
                                on_edit: move |id| {
                                    show_modal.set(Some(ShowModal::EditMember(id)));
                                },
                                show: show_modal().is_some_and(|m| m == ShowModal::ViewMember(member.id)),
                                member: member.clone(),
                                on_close: move |_| {
                                    show_modal.set(None);
                                },
                            }
                            EditMemberModal {
                                show: show_modal().is_some_and(|m| m == ShowModal::EditMember(member.id)),
                                member: member.clone(),
                                on_close: move |_| {
                                    show_modal.set(None);
                                },
                                on_submit: move |data: EditMemberFormData| async move {
                                    let _ = edit_member(
                                        data.id,
                                        data.name,
                                        data.last_name,
                                        data.father_id,
                                        data.mother_id,
                                        data.gender,
                                        data.birthday
                                    ).await;

                                    members_resource.restart();

                                    show_modal.set(None);
                                },
                            }
                            MemberCard {
                                member: member.clone(),
                                on_view: move |id| {
                                    show_modal.set(Some(ShowModal::ViewMember(id)));
                                },
                                on_edit: move |id| {
                                    show_modal.set(Some(ShowModal::EditMember(id)));
                                },
                                on_invite: move |_| {
                                    tracing::info!("on invite");
                                },
                                on_delete: move |id| async move {
                                    if delete_member(id).await.is_ok() {
                                        members_resource.restart();
                                    }
                                },
                            }
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

    let add_requests_grid = match &*add_requests_resource.read() {
        Some(Ok(requests)) => {
            rsx! {
                div { class: "card-body",
                    if requests.is_empty() {
                        div { class: "col-span-full text-center py-12",
                            svg {
                                class: "w-16 h-16 mx-auto text-gray-400 mb-4",
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
                            h3 { class: "text-lg font-medium text-gray-900 mb-2",
                                "لا يوجد أعضاء"
                            }
                            p { class: "text-gray-500",
                                "لم يتم العثور على أعضاء مطابقين للبحث"
                            }
                        }
                    } else {
                        div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 pb-6",
                            for request in requests.iter().filter(|r| matches!(r.status, RequestStatus::Pending)) {
                                ViewRequestModal {
                                    show: show_modal().is_some_and(|r| r == ShowModal::ViewRequest(request.id)),
                                    request: request.clone(),
                                    on_close: move |_| {
                                        show_modal.set(None);
                                    },
                                    on_edit: move |_| {
                                        tracing::debug!("on_edit");
                                    },
                                }

                                RequestCard {
                                    request: request.clone(),
                                    on_approve: move |id| async move {
                                        match approve_request(id).await {
                                            Ok(_) => {
                                                add_requests_resource.restart();
                                                members_resource.restart();
                                            },
                                            Err(e) => {}
                                        }
                                    },
                                    on_reject: move |id| async move {
                                        match disapprove_request(id).await {
                                            Ok(_) => {
                                                add_requests_resource.restart();
                                            },
                                            Err(e) => {}
                                        }
                                    },
                                    on_view: move |id| {
                                        show_modal.set(Some(ShowModal::ViewRequest(id)));
                                    },
                                }
                            }
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
        div { class: "w-3/4 mx-auto",
            dir: "rtl",
            div { class: "text-red-500", "{error}" }

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
                            Link {
                                class: "btn btn-outline btn-sm",
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
                            button {
                                class: "btn btn-danger btn-sm",
                                onclick: move |_| async move {
                                    let _ = logout_admin().await;
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
                on_close: move |_| {
                    show_modal.set(None);
                },
                on_submit: move |data: MemberFormData| async move {
                    let Some(gender) = data.gender else {
                        return;
                    };

                    let _ = add_member(data.name, data.last_name, data.father_id, data.mother_id, gender, data.birthday).await;

                    members_resource.restart();

                    show_modal.set(None);
                },
            }
            match &*tab.read() {
                Tab::Members => rsx! {
                    FamilyManagementHeader {
                        members_count,
                        on_add_member: move |_| {
                            show_modal.set(Some(ShowModal::AddMember));
                        },
                        on_csv_upload: move |e: Event<FormData>| async move {
                            if let Some(file_engine) = &e.files() {
                                let files = file_engine.files();
                                for file_name in files {
                                    if let Some(file) = file_engine.read_file_to_string(&file_name).await {
                                        match upload_members_csv(file).await {
                                            Ok(_) => members_resource.restart(),
                                            Err(e) => tracing::error!("{e}"),
                                        }
                                    }
                                }
                            }
                        },
                    }
                    {members_grid}
                },
                Tab::Requests => rsx! {
                    {add_requests_grid}
                },
            }
        }
    }
}
