use dioxus::prelude::*;
use shared::EguiCommand;

use crate::{
    components::combobox::{Combobox, ComboboxOption},
    modules::member::search::{get_member_by_name, search_member},
};

#[component]
pub fn Tree() -> Element {
    let mut fullscreen_tree = use_signal(|| false);

    let mut member_query = use_signal(String::new);
    let mut members_search_list = use_signal(|| vec![]);
    let mut search_bar_ref = use_signal(|| None::<std::rc::Rc<MountedData>>);

    let mut ctx = use_signal(|| {
        cfg_select! {
            target_arch = "wasm32" => None::<gui::egui::Context>,
            _ => None::<()>
        }
    });
    let commands = use_signal(move || {
        cfg_select! {
            target_arch = "wasm32" => {
                let (send, recv) = futures::channel::mpsc::channel(10);
                let (ctx_send, mut ctx_recv) = futures::channel::mpsc::channel(1);

                spawn(async move {
                    gui::run_eframe(recv, ctx_send);
                });

                spawn(async move {
                    ctx.set(Some(ctx_recv.recv().await.unwrap()));
                });

                Some(send)
            }
            _ => None::<EguiCommand>
        }
    });

    let mut on_search_button = use_action(move |query: String| async move {
        if query.is_empty() {
            members_search_list.clear();
            return anyhow::Ok(());
        }

        if let Ok(Some(member)) = get_member_by_name(query.clone()).await {
            #[cfg(target_arch = "wasm32")]
            {
                use futures::SinkExt;
                if let Some(mut commands) = commands()
                    && let Some(ctx) = ctx()
                {
                    commands
                        .send(EguiCommand::HighlightMember(member.id as i32))
                        .await?;
                    ctx.request_repaint();
                }
            }
        } else {
            if let Ok(members) = search_member(query.clone()).await {
                members_search_list.set(members);
                if let Some(search_bar) = search_bar_ref() {
                    let _ = search_bar.set_focus(true).await;
                }
            }
        }

        anyhow::Ok(())
    });

    let mut search = use_action(move |query: String| async move {
        if query.is_empty() {
            members_search_list.clear();
            return anyhow::Ok(());
        }

        let members = search_member(query).await;
        if let Ok(members) = members {
            members_search_list.set(members);
        }

        anyhow::Ok(())
    });

    let mut on_select = use_action(move |id: i64| async move {
        #[cfg(target_arch = "wasm32")]
        {
            use futures::SinkExt;
            if let Some(mut commands) = commands()
                && let Some(ctx) = ctx()
            {
                commands
                    .send(EguiCommand::HighlightMember(id as i32))
                    .await?;
                ctx.request_repaint();
            }
        }

        anyhow::Ok(())
    });

    rsx! {
        div {
            class: "card flex flex-col items-center sm:border sm:border-gray-300 py-2 lg:p-8 gap-2",
            dir: "rtl",
            h3 { "شجرة العائلة" }
            div {
                class: "flex w-full gap-2",

                button {
                    class: "btn btn-primary",
                    onclick: move |_| on_search_button.call(member_query()),
                    i { class: "fa-solid fa-magnifying-glass" }
                }
                Combobox {
                    on_mount: move |element: Event<MountedData>| {
                        search_bar_ref.set(Some(element.data()));
                    },
                    on_search: move |query| {
                        member_query.set(query);
                        search.call(member_query());
                    },
                    on_select: move |option: ComboboxOption<i64>| {
                        member_query.set(option.label.clone());
                        on_select.call(option.value);
                        search.call(option.label);
                    },
                    options: members_search_list.iter().enumerate().map(|(i, member)|  {
                        ComboboxOption {
                            index: i,
                            value: member.id,
                            label: member.full_name.clone().unwrap_or_else(|| member.name.clone()),
                        }
                    }).collect(),
                }
            }
            div {
                dir: "rtl",
                class: "flex flex-col w-full",
                class: if fullscreen_tree() {
                           "h-dvh fixed top-0 left-0 w-full z-100 bg-white"
                       } else {
                           "h-150 lg:h-200"
                       },
                button {
                    class: "btn-rect btn-primary rounded-t-sm",
                    onclick: move |_| {
                        fullscreen_tree.toggle();
                    },
                    if fullscreen_tree() { "صغر الشجرة" } else { "كبر الشجرة" }
                }
                canvas {
                    id: "canvas",
                    class: "w-full h-full",
                    flex: 1,
                    touch_action: "manipulation",
                    width: "100%",
                    height: "100%",
                }
            }
        }
    }
}
