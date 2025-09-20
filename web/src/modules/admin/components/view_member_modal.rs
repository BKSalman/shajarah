use crate::{
    modules::{
        admin::components::ImageState,
        member::types::{Gender, MemberResponseFlat},
    },
    ui::modal::Modal,
};
use base64::prelude::*;
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ViewMemberModalProps {
    pub show: bool,
    pub member: MemberResponseFlat,
    pub on_close: EventHandler<()>,
    pub on_edit: EventHandler<i64>,
}

#[component]
pub fn ViewMemberModal(props: ViewMemberModalProps) -> Element {
    let mut image_state = use_signal(|| ImageState::Loading);
    use_effect({
        let member = props.member.clone();
        move || {
            if member.image.is_some() {
                image_state.set(ImageState::Loading);
            } else {
                image_state.set(ImageState::GeneratedAvatar);
            }
        }
    });
    if !props.show {
        return rsx! {};
    }
    rsx! {
        Modal {
            show: props.show,
            title: "تفاصيل العضو".to_string(),
            max_width: Some("2xl".to_string()),
            on_close: move |_| props.on_close.call(()),
            div { class: "space-y-6",
                MemberHeader {
                    member: props.member.clone(),
                    image_state: image_state(),
                    on_image_load: move |_| image_state.set(ImageState::UserImage),
                    on_image_error: move |_| image_state.set(ImageState::GeneratedAvatar),
                }
                div { class: "grid grid-cols-1 md:grid-cols-2 gap-6",
                    BasicInformationSection { member: props.member.clone() }
                    FamilyRelationshipsSection { member: props.member.clone() }
                }
                if let Some(personal_info) = &props.member.personal_info {
                    if !personal_info.is_empty() {
                        PersonalInformationSection { personal_info: personal_info.clone() }
                    }
                }
                QuickActionsSection {
                    member_id: props.member.id,
                    on_edit: props.on_edit,
                    on_close: props.on_close,
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct MemberHeaderProps {
    member: MemberResponseFlat,
    image_state: ImageState,
    on_image_load: EventHandler<()>,
    on_image_error: EventHandler<()>,
}

#[component]
fn MemberHeader(props: MemberHeaderProps) -> Element {
    rsx! {
        div { class: "text-center mb-6",
            div { class: "w-24 h-24 mx-auto mb-4 rounded-full overflow-hidden relative",
                match props.image_state {
                    ImageState::Loading => rsx! {
                        div { class: "w-full h-full flex items-center justify-center bg-gray-200 animate-pulse",
                            div { class: "w-6 h-6 border-2 border-primary-600 border-t-transparent rounded-full animate-spin" }
                        }
                    },
                    ImageState::UserImage => {
                        if let Some(image_url) = &props.member.image {
                            rsx! {
                                img {
                                    src: "{BASE64_STANDARD.encode(image_url)}",
                                    class: "w-full h-full object-cover transition-opacity duration-200",
                                    alt: "صورة {props.member.name} {props.member.last_name}",
                                    onload: move |_| props.on_image_load.call(()),
                                    onerror: move |_| props.on_image_error.call(()),
                                }
                            }
                        } else {
                            rsx! {}
                        }
                    }
                    ImageState::GeneratedAvatar => rsx! {
                        GeneratedAvatar { member: props.member.clone() }
                    },
                    ImageState::DefaultIcon => rsx! {
                        div { class: "w-full h-full flex items-center justify-center bg-primary-100",
                            svg {
                                class: "w-12 h-12 text-primary-600",
                                fill: "none",
                                stroke: "currentColor",
                                view_box: "0 0 24 24",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    stroke_width: "2",
                                    d: "M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z",
                                }
                            }
                        }
                    },
                    ImageState::Error => rsx! { "error loading image" },
                }
            }
            h4 { class: "text-xl font-semibold text-gray-900",
                "{props.member.name} {props.member.last_name}"
            }
            p { class: "text-gray-600", "رقم العضوية: {props.member.id}" }
            div { class: "flex justify-center mt-2",
                match props.member.gender {
                    Gender::Male => rsx! {
                        span { class: "inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-blue-100 text-blue-800",
                            "ذكر"
                        }
                    },
                    Gender::Female => rsx! {
                        span { class: "inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-pink-100 text-pink-800",
                            "أنثى"
                        }
                    },
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct GeneratedAvatarProps {
    member: MemberResponseFlat,
}

#[component]
fn GeneratedAvatar(props: GeneratedAvatarProps) -> Element {
    let initials = format!(
        "{}{}",
        props.member.name.chars().next().unwrap_or('؟'),
        props.member.last_name.chars().next().unwrap_or('؟'),
    );
    let bg_class = match props.member.gender {
        Gender::Male => "bg-blue-500 text-white",
        Gender::Female => "bg-pink-500 text-white",
    };
    rsx! {
        div { class: "w-full h-full flex items-center justify-center text-lg font-bold {bg_class}",
            "{initials}"
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct BasicInformationSectionProps {
    member: MemberResponseFlat,
}

#[component]
fn BasicInformationSection(props: BasicInformationSectionProps) -> Element {
    rsx! {
        div { class: "space-y-4",
            h5 { class: "text-lg font-semibold text-gray-900 border-b pb-2",
                "المعلومات الأساسية"
            }
            if let Some(birthday) = props.member.birthday {
                div {
                    label { class: "text-sm font-medium text-gray-500", "تاريخ الميلاد" }
                    p { class: "text-gray-900", "{birthday.date_naive()}" }
                }
                div {
                    label { class: "text-sm font-medium text-gray-500", "العمر" }
                    p { class: "text-gray-900",
                        "{chrono::Utc::now().years_since(birthday).unwrap_or(0)} سنة"
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct FamilyRelationshipsSectionProps {
    member: MemberResponseFlat,
}

#[component]
fn FamilyRelationshipsSection(props: FamilyRelationshipsSectionProps) -> Element {
    rsx! {
        div { class: "space-y-4",
            h5 { class: "text-lg font-semibold text-gray-900 border-b pb-2",
                "العلاقات العائلية"
            }
            if let Some(father_name) = &props.member.father_name {
                div {
                    label { class: "text-sm font-medium text-gray-500", "الوالد" }
                    p { class: "text-gray-900", "{father_name}" }
                }
            }
            if let Some(mother_name) = &props.member.mother_name {
                div {
                    label { class: "text-sm font-medium text-gray-500", "الوالدة" }
                    p { class: "text-gray-900", "{mother_name}" }
                }
            }
            if !props.member.children.is_empty() {
                div {
                    label { class: "text-sm font-medium text-gray-500", "الأطفال" }
                    div { class: "space-y-1",
                        for child in props.member.children {
                            p { class: "text-gray-900 text-sm", "{child.name} {child.last_name}" }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct PersonalInformationSectionProps {
    personal_info: indexmap::IndexMap<String, String>,
}

#[component]
fn PersonalInformationSection(props: PersonalInformationSectionProps) -> Element {
    rsx! {
        div { class: "mt-6",
            h5 { class: "text-lg font-semibold text-gray-900 border-b pb-2 mb-4",
                "المعلومات الشخصية"
            }
            div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                for (key , value) in props.personal_info.iter() {
                    div {
                        label { class: "text-sm font-medium text-gray-500", "{key}" }
                        p { class: "text-gray-900", "{value}" }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct QuickActionsSectionProps {
    member_id: i64,
    on_edit: EventHandler<i64>,
    on_close: EventHandler<()>,
}

#[component]
fn QuickActionsSection(props: QuickActionsSectionProps) -> Element {
    rsx! {
        div { class: "flex justify-center gap-3 mt-6 pt-6 border-t",
            button {
                class: "btn btn-secondary",
                onclick: move |_| {
                    props.on_edit.call(props.member_id);
                    props.on_close.call(());
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
                        d: "M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z",
                    }
                }
                "تحرير"
            }
            button {
                class: "btn btn-outline",
                onclick: move |_| props.on_close.call(()),
                "إغلاق"
            }
        }
    }
}
