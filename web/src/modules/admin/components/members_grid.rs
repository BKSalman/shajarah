use dioxus::{fullstack::FileStream, prelude::*};

use crate::{
    components::loading::FullPageLoading,
    modules::{
        admin::components::{
            edit_member_modal::EditMemberModal, family_management_header::FamilyManagementHeader,
            member_card::MemberCard, view_member_modal::ViewMemberModal,
        },
        member::{
            server::{delete_member, edit_member, edit_member_image, upload_members_csv},
            types::{EditMember, MemberResponseFlat},
        },
    },
    ui::{modal::Modal, search_bar::SearchBar},
    util::matches_search,
};

use super::ShowModal;

#[component]
pub fn MembersGrid(
    show_modal: Signal<Option<ShowModal>>,
    members_resource: Resource<Result<Vec<MemberResponseFlat>, anyhow::Error>>,
) -> Element {
    let mut csv_upload = use_action(move |e: Event<FormData>| async move {
        if let Some(file_data) = e.files().first()
            && let Ok(data) = file_data.read_string().await
        {
            return upload_members_csv(data).await;
        }

        Err(anyhow::anyhow!(""))
    });

    use_effect(move || {
        if csv_upload.value().is_some_and(|r| r.is_ok()) {
            members_resource.restart();
        }
    });

    let members_list = (*members_resource.read())
        .as_ref()
        .and_then(|e| e.as_ref().ok())
        .cloned()
        .unwrap_or_default();

    let members_count = members_list.len();

    let mut search = use_signal(String::new);

    let visible_members = {
        let query = search();

        members_list
            .iter()
            .filter(|member| {
                matches_search(
                    &query,
                    &[
                        member.full_name.as_str(),
                        member.last_name.as_str(),
                        member.father_name.as_deref().unwrap_or_default(),
                        member.mother_name.as_deref().unwrap_or_default(),
                    ]
                    .join(" "),
                )
            })
            .cloned()
            .collect::<Vec<_>>()
    };

    let visible_count = visible_members.len();

    let mut on_member_edit = use_action(
        move |id: i64, data: EditMember, image: Option<FileStream>| async move {
            edit_member(id, data).await?;
            if let Some(image) = image {
                edit_member_image(id, image).await?;
            }

            members_resource.restart();

            show_modal.set(None);

            anyhow::Ok(())
        },
    );

    let members_grid = match &*members_resource.read() {
        Some(Ok(members)) => {
            let searching = !search().trim().is_empty();

            rsx! {
                if visible_members.is_empty() {
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
                            if searching {
                                "لا نتائج"
                            } else {
                                "لا يوجد أعضاء"
                            }
                        }
                        p { class: "text-gray-500",
                            if searching {
                                "لم يتم العثور على أعضاء مطابقين للبحث"
                            } else {
                                "لم تتم إضافة أي عضو بعد"
                            }
                        }
                    }
                } else {
                    div {
                        class: "grid grid-cols-1 md:grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-2 pb-6",
                        id: "members-grid",
                        for member in visible_members.iter() {
                            ViewMemberModal {
                                member: member.clone(),
                                on_edit: move |id| {
                                    show_modal.set(Some(ShowModal::EditMember(id)));
                                },
                                show: show_modal().is_some_and(|m| m == ShowModal::ViewMember(member.id)),
                                on_close: move |_| {
                                    show_modal.set(None);
                                },
                            }
                            EditMemberModal {
                                member: member.clone(),
                                members: members.clone(),
                                show: show_modal().is_some_and(|m| m == ShowModal::EditMember(member.id)),
                                on_close: move |_| {
                                    show_modal.set(None);
                                },
                                on_submit: move |(id, data, image): (i64, EditMember, Option<FileStream>)| {
                                    on_member_edit.call(id, data, image)
                                },
                            }
                            Modal {
                                show: show_modal().is_some_and(|m| m == ShowModal::DeleteMember(member.id)),
                                title: String::from("تأكيد حذف العضو"),
                                on_close: move |_| show_modal.set(None),
                                max_width: Some(String::from("xl")),

                                span { "هل أنت متأكد من حذف {member.name}؟" }

                                div { class: "flex justify-center gap-4",
                                    button {
                                        class: "btn btn-danger",
                                        onclick: {
                                            let member_id = member.id;
                                            move |_| async move {
                                                if delete_member(member_id).await.is_ok() {
                                                    show_modal.set(None);
                                                    members_resource.restart();
                                                }
                                            }
                                        },
                                        "حذف"
                                    }
                                    button {
                                        class: "btn btn-secondary",
                                        onclick: move |_| show_modal.set(None),
                                        "إلغاء"
                                    }
                                }
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
                                    show_modal.set(Some(ShowModal::DeleteMember(id)));
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
                FullPageLoading {
                }
            };
        }
    };

    rsx! {
        FamilyManagementHeader {
            members_count,
            on_add_member: move |_| {
                show_modal.set(Some(ShowModal::AddMember));
            },
            on_csv_upload: move |e| {
                csv_upload.call(e);
                tracing::info!("{:?}", csv_upload.value());
            },
        }
        div { class: "p-3",

            SearchBar {
                value: search(),
                placeholder: Some(
                    "ابحث عن فرد بالاسم أو اسم الأب أو الأم..."
                        .to_string(),
                ),
                result_count: Some(visible_count),
                on_input: move |value| search.set(value),
            }
        }
        {members_grid}
    }
}
