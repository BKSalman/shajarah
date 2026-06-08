use dioxus::prelude::*;
#[derive(Props, Clone, PartialEq)]
pub struct FamilyManagementHeaderProps {
    pub members_count: usize,
    pub on_add_member: EventHandler<()>,
    pub on_csv_upload: EventHandler<Event<FormData>>,
}
#[component]
pub fn FamilyManagementHeader(props: FamilyManagementHeaderProps) -> Element {
    rsx! {
        div { class: "card-header mb-4",
            div { class: "flex flex-col lg:flex-row justify-between items-start lg:items-center gap-4",
                HeaderTitle { members_count: props.members_count }
                ActionButtons {
                    on_add_member: props.on_add_member,
                    on_csv_upload: props.on_csv_upload,
                }
            }
        }
    }
}
#[derive(Props, Clone, PartialEq)]
struct HeaderTitleProps {
    members_count: usize,
}
#[component]
fn HeaderTitle(props: HeaderTitleProps) -> Element {
    rsx! {
        div {
            h2 { class: "text-xl font-bold text-gray-900 flex items-center",
                svg {
                    class: "w-5 h-5 ml-2",
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
                span { class: "arabic-text", "إدارة أفراد العائلة" }
            }
            p { class: "text-sm text-gray-600 mt-1 arabic-text",
                "عرض وتحرير أعضاء شجرة العائلة - المجموع: "
                span { class: "font-medium text-forest-primary", "{props.members_count}" }
                " عضو"
            }
        }
    }
}
#[derive(Props, Clone, PartialEq)]
struct ActionButtonsProps {
    on_add_member: EventHandler<()>,
    on_csv_upload: EventHandler<Event<FormData>>,
}
#[component]
fn ActionButtons(props: ActionButtonsProps) -> Element {
    rsx! {
        div { class: "flex gap-2",
            AddMemberButton { on_click: props.on_add_member }
            ExportButton {}
            CsvUploadButton { on_upload: props.on_csv_upload }
        }
    }
}
#[derive(Props, Clone, PartialEq)]
struct AddMemberButtonProps {
    on_click: EventHandler<()>,
}
#[component]
fn AddMemberButton(props: AddMemberButtonProps) -> Element {
    rsx! {
        button {
            class: "btn btn-primary btn-sm",
            onclick: move |_| props.on_click.call(()),
            svg {
                class: "w-4 h-4",
                fill: "none",
                stroke: "currentColor",
                view_box: "0 0 24 24",
                path {
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    stroke_width: "2",
                    d: "M12 6v6m0 0v6m0-6h6m-6 0H6",
                }
            }
            "إضافة عضو جديد"
        }
    }
}
#[derive(Props, Clone, PartialEq)]
struct CsvUploadButtonProps {
    on_upload: EventHandler<Event<FormData>>,
}

#[component]
fn CsvUploadButton(props: CsvUploadButtonProps) -> Element {
    rsx! {
        label { class: "btn btn-secondary btn-sm cursor-pointer",
            svg {
                class: "w-4 h-4",
                fill: "none",
                stroke: "currentColor",
                view_box: "0 0 24 24",
                path {
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    stroke_width: "2",
                    d: "M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12",
                }
            }
            "رفع قائمة CSV"
            input {
                r#type: "file",
                accept: ".csv",
                class: "hidden",
                onchange: move |evt| props.on_upload.call(evt),
            }
        }
    }
}

#[component]
pub fn ExportButton() -> Element {
    rsx! {
        a {
            href: "/api/members/export",
            download: "exported-members.csv",
            class: "btn btn-outline btn-sm",
            svg {
                class: "w-4 h-4",
                fill: "none",
                stroke: "currentColor",
                view_box: "0 0 24 24",
                path {
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    stroke_width: "2",
                    d: "M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z",
                }
            }
            "تصدير القائمة"
        }
    }
}
