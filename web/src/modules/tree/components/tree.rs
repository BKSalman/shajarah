use dioxus::prelude::*;
use shared::EguiCommand;

use crate::{
    components::combobox::{Combobox, ComboboxOption},
    modules::member::search::search_member,
};

#[component]
pub fn Tree() -> Element {
    let mut fullscreen_tree = use_signal(|| false);

    let mut members_search_list = use_signal(|| vec![]);

    let mut ctx = use_signal(|| {
        cfg_select! {
            target_arch = "wasm32" => None::<gui::egui::Context>,
            _ => None::<()>
        }
    });
    let commands = use_hook(move || {
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
            _ => None::<()>
        }
    });

    let mut on_input = use_action(move |query: String| async move {
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

    let mut on_select = use_action(move |id: i64| {
        let commands = commands.clone();
        async move {
            #[cfg(target_arch = "wasm32")]
            {
                use futures::SinkExt;
                if let Some(mut commands) = commands
                    && let Some(ctx) = ctx()
                {
                    commands
                        .send(EguiCommand::HighlightMember(id as i32))
                        .await?;
                    ctx.request_repaint();
                }
            }

            anyhow::Ok(())
        }
    });

    // use futures::SinkExt;
    // commands.send(EguiCommand::HighlightMember(query)).await?;
    // ctx.request_repaint();

    rsx! {
        div {
            class: "card flex flex-col items-center sm:border sm:border-gray-300 py-2 lg:p-8",
            dir: "rtl",
            h3 { "شجرة العائلة" }
            Combobox {
                on_search: move |query| {
                    tracing::info!("query: {query}");
                    on_input.call(query);
                },
                on_select: move |option: ComboboxOption<i64>| {
                    tracing::info!("select: {option:?}");
                    on_select.call(option.value);
                },
                options: members_search_list.iter().enumerate().map(|(i, member)|  {
                    ComboboxOption {
                        index: i,
                        value: member.id,
                        label: member.full_name.clone().unwrap_or_else(|| member.name.clone()),
                    }
                }).collect(),
            }
            div {
                dir: "rtl",
                class: "flex flex-col w-full",
                class: if fullscreen_tree() {
                           "h-full absolute top-0 left-0 z-100 bg-white"
                       } else {
                           "h-150 lg:h-200"
                       },
                button {
                    class: "btn-rect btn-primary",
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
