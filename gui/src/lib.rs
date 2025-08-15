#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod tree;
mod zoom;
use std::sync::{Arc, mpsc::Sender};

use eframe::egui;

pub use app::App;
use serde::{Deserialize, Serialize};
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
    let request = ehttp::Request::get(format!("{address}/api/members"));
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
            log::error!("failed to fetch family data: {e}");
        }
    });
}
