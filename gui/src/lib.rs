#![warn(clippy::all, rust_2018_idioms)]
mod app;
mod bidi;
mod tree;
mod zoom;
pub use app::App;
pub use eframe::egui;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, mpsc::Sender};
use tree::Node;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Gender {
    Male,
    Female,
}

#[derive(Debug)]
enum Message {
    LoadedFamilyData(Node),
}

const FONT: &[u8] = include_bytes!("../fonts/arial.ttf");

#[cfg(target_arch = "wasm32")]
pub fn run_eframe(
    commands_rx: futures::channel::mpsc::Receiver<shared::EguiCommand>,
    mut ctx_sender: futures::channel::mpsc::Sender<egui::Context>,
) {
    use eframe::wasm_bindgen::JsCast as _;

    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async move {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("canvas")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| {
                    let ctx = cc.egui_ctx.clone();
                    futures::executor::block_on(async move {
                        use futures::SinkExt;
                        if let Err(e) = ctx_sender.send(cc.egui_ctx.clone()).await {
                            log::error!("Failed to send context: {e}");
                        }
                    });
                    Ok(Box::new(crate::App::new(cc, commands_rx)))
                }),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "arial".to_owned(),
        Arc::new(egui::FontData::from_static(FONT)),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "arial".to_owned());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .push("arial".to_owned());
    ctx.set_fonts(fonts);
}

fn load_family_data(address: &str, sender: Sender<Message>, ctx: &egui::Context) {
    let ctx = ctx.clone();
    let request = ehttp::Request::get(format!("{address}/api/v1/members/admin"));
    ehttp::fetch(request, move |res| match res {
        Ok(res) => {
            if !res.ok {
                log::error!("{res:?}");
                return;
            }
            match res.json::<Node>() {
                Ok(node) => {
                    let _ = sender.send(Message::LoadedFamilyData(node));
                    log::info!("Received family data successfully");
                    ctx.request_repaint();
                }
                Err(e) => {
                    log::error!("failed to fetch family data: {e}");
                }
            }
        }
        Err(e) => {
            // TODO: retry with the non-admin route when added
            log::error!("failed to fetch family data: {e}");
        }
    });
}
