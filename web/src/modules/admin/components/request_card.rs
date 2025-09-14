use base64::prelude::*;
use dioxus::prelude::*;
use uuid::Uuid;

use crate::modules::{
    add_request::types::RequestedMember, admin::components::ImageState, member::types::Gender,
};

#[derive(Props, Clone, PartialEq)]
pub struct RequestCardProps {
    request: RequestedMember,
    on_approve: EventHandler<Uuid>,
    on_reject: EventHandler<Uuid>,
    on_view: EventHandler<Uuid>,
}

#[component]
pub fn RequestCard(props: RequestCardProps) -> Element {
    let mut image_state = use_signal(|| ImageState::Loading);
    let mut image_src = use_signal(|| String::new());

    // Initialize image loading
    use_effect(move || {
        if let Some(url) = &props.request.image {
            image_src.set(format!("{}", BASE64_STANDARD.encode(url)));
            image_state.set(ImageState::Loading);
        } else {
            image_state.set(ImageState::GeneratedAvatar);
        }
    });

    let handle_image_load = move |_| {
        image_state.set(ImageState::UserImage);
    };

    let handle_image_error = move |_| {
        image_state.set(ImageState::GeneratedAvatar);
    };

    let retry_image = move |_| {
        image_state.set(ImageState::Loading);
    };

    let get_avatar_content = || {
        let first_char = props.request.name.chars().next().unwrap_or('؟');
        first_char.to_string()
    };

    let get_gender_colors = |gender: &Gender| -> (&str, &str) {
        match gender {
            Gender::Male => (
                "bg-gradient-to-br from-blue-400 to-blue-600",
                "bg-gradient-to-br from-blue-100 to-blue-200 text-blue-700",
            ),
            Gender::Female => (
                "bg-gradient-to-br from-pink-400 to-pink-600",
                "bg-gradient-to-br from-pink-100 to-pink-200 text-pink-700",
            ),
        }
    };

    let (avatar_bg, icon_bg) = get_gender_colors(&props.request.gender);

    rsx! {
        div { class: "bg-white border border-yellow-200 rounded-lg hover:shadow-lg transition-all duration-200 overflow-hidden border-l-4 border-l-yellow-400",

            // Image Container
            div { class: "relative h-32 image-container",

                // Loading State
                if *image_state.read() == ImageState::Loading {
                    div { class: "w-full h-full flex items-center justify-center skeleton-shimmer",
                        div { class: "image-loading",
                            div { class: "spinner" }
                        }
                    }
                }

                // User Uploaded Image
                if *image_state.read() == ImageState::UserImage {
                    img {
                        src: "{image_src}",
                        alt: "{props.request.name} {props.request.last_name}",
                        class: "w-full h-full object-cover",
                        onload: handle_image_load,
                        onerror: handle_image_error,
                    }
                }

                // Generated Avatar
                if *image_state.read() == ImageState::GeneratedAvatar {
                    div { class: "w-full h-full flex items-center justify-center text-white font-bold text-xl {avatar_bg}",
                        "{get_avatar_content()}"
                    }
                }

                // Default Icon
                if *image_state.read() == ImageState::DefaultIcon {
                    div { class: "w-full h-full flex items-center justify-center {icon_bg}",
                        svg {
                            class: "w-16 h-16",
                            fill: "currentColor",
                            view_box: "0 0 24 24",
                            path { d: "M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z" }
                        }
                    }
                }

                // Error State
                if *image_state.read() == ImageState::Error {
                    div { class: "w-full h-full flex flex-col items-center justify-center bg-gray-100 text-gray-500",
                        svg {
                            class: "w-12 h-12 mb-2",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z",
                            }
                        }
                        button {
                            class: "text-xs text-blue-600 hover:text-blue-800 underline",
                            onclick: retry_image,
                            "إعادة المحاولة"
                        }
                    }
                }
            }

            // Card Content
            div { class: "p-4",

                // Name and ID
                div { class: "text-center mb-3",
                    h4 { class: "font-bold text-gray-900",
                        "{props.request.name} {props.request.last_name}"
                    }
                    p { class: "text-sm text-gray-600", "رقم المعرف: {props.request.id}" }
                }

                // Gender Badge
                div { class: "flex justify-center mb-3",
                    span { class: if props.request.gender == Gender::Male { "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800" } else { "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-pink-100 text-pink-800" },
                        if props.request.gender == Gender::Male {
                            "ذكر"
                        } else {
                            "أنثى"
                        }
                    }
                }

                // Family Relations
                if props.request.father_name.is_some() || props.request.mother_name.is_some() {
                    div { class: "text-xs text-gray-600 mb-3",
                        if let Some(father_name) = &props.request.father_name {
                            div { class: "mb-1",
                                span { class: "font-medium", "الوالد: " }
                                span { "{father_name}" }
                            }
                        }
                        if let Some(mother_name) = &props.request.mother_name {
                            div {
                                span { class: "font-medium", "الوالدة: " }
                                span { "{mother_name}" }
                            }
                        }
                    }
                }

                // Personal Info
                if let Some(personal_info) = &props.request.personal_info {
                    if !personal_info.is_empty() {
                        div { class: "text-xs text-gray-600 mb-3",
                            for (key , value) in personal_info.iter() {
                                div { key: "{key}", class: "mb-1",
                                    span { class: "font-medium", "{key}: " }
                                    span { "{value}" }
                                }
                            }
                        }
                    }
                }

                // Status Badge
                div { class: "flex justify-center mb-3",
                    span { class: "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-yellow-100 text-yellow-800",
                        "معلق"
                    }
                }

                // Action Buttons
                div { class: "flex justify-center gap-2",
                    button {
                        class: "btn btn-sm bg-green-600 hover:bg-green-700 text-white border-green-600 hover:border-green-700",
                        onclick: move |_| props.on_approve.call(props.request.id.clone()),
                        "موافقة"
                    }
                    button {
                        class: "btn btn-sm btn-outline",
                        onclick: move |_| props.on_view.call(props.request.id.clone()),
                        "عرض"
                    }
                    button {
                        class: "btn btn-sm bg-red-600 hover:bg-red-700 text-white border-red-600 hover:border-red-700",
                        onclick: move |_| props.on_reject.call(props.request.id.clone()),
                        "رفض"
                    }
                }
            }
        }
    }
}
