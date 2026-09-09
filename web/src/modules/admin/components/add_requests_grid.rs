use dioxus::prelude::*;

use crate::components::loading::FullPageLoading;
use crate::modules::{
    add_request::{
        server::{approve_request, disapprove_request},
        types::{RequestStatus, RequestedMember},
    },
    admin::components::{
        ShowModal, request_card::RequestCard, view_request_modal::ViewRequestModal,
    },
    member::types::MemberResponseFlat,
};

#[component]
pub fn AddRequestsGrid(
    show_modal: Signal<Option<ShowModal>>,
    members_resource: Resource<Result<Vec<MemberResponseFlat>, anyhow::Error>>,
    add_requests_resource: Resource<Result<Vec<RequestedMember>, anyhow::Error>>,
) -> Element {
    let mut on_request_approve = use_action(move |id| async move {
        approve_request(id).await?;
        add_requests_resource.restart();
        members_resource.restart();

        anyhow::Ok(())
    });

    let mut on_request_reject = use_action(move |id| async move {
        disapprove_request(id).await?;
        add_requests_resource.restart();

        anyhow::Ok(())
    });

    rsx! {
        match &*add_requests_resource.read() {
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
                                h3 { class: "text-lg font-medium text-gray-900 mb-2", "لا يوجد أعضاء" }
                                p { class: "text-gray-500",
                                    "لم يتم العثور على أعضاء مطابقين للبحث"
                                }
                            }
                        } else {
                            div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 pb-6",
                                for request in requests
                                    .iter()
                                    .filter(|r| {
                                        matches!(r.status, RequestStatus::Pending) && r.mother_request_id.is_none()
                                            && r.father_request_id.is_none()
                                    })
                                {
                                    ViewRequestModal {
                                        show: show_modal().is_some_and(|r| r == ShowModal::ViewRequest(request.id)),
                                        request: request.clone(),
                                        children_requests: requests
                                            .iter()
                                            .filter(|r| {
                                                matches!(r.status, RequestStatus::Pending)
                                                    && (r.mother_request_id == Some(request.id)
                                                        || r.father_request_id == Some(request.id))
                                            })
                                            .cloned()
                                            .collect(),
                                        on_close: move |_| {
                                            show_modal.set(None);
                                        },
                                        on_edit: move |_| {
                                            tracing::debug!("on_edit");
                                        },
                                    }

                                    RequestCard {
                                        request: request.clone(),
                                        on_approve: move |id| on_request_approve.call(id),
                                        on_reject: move |id| on_request_reject.call(id),
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
                    FullPageLoading {
                    }
                };
            }
        }
    }
}
