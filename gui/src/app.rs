use crate::{
    Message,
    bidi::bidi_job,
    load_family_data, setup_fonts,
    tree::{MarriageStatus, TreeUi},
};
use eframe::egui::{self, Align, TextFormat, Widget as _};
use shared::EguiCommand;
use std::sync::mpsc::{self, Receiver, Sender};

pub struct App {
    tree: TreeUi,
    message_receiver: Receiver<Message>,
    message_sender: Sender<Message>,
    backend_address: String,
    commands_rx: futures::channel::mpsc::Receiver<EguiCommand>,
}

impl App {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        commands_rx: futures::channel::mpsc::Receiver<EguiCommand>,
    ) -> Self {
        setup_fonts(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let (sender, receiver) = mpsc::channel();
        #[cfg(not(target_arch = "wasm32"))]
        let address = "http://localhost:3001";
        #[cfg(target_arch = "wasm32")]
        let address = "";
        load_family_data(address, sender.clone(), &cc.egui_ctx);
        Self {
            tree: TreeUi::new(None, &cc.egui_ctx),
            message_sender: sender.clone(),
            message_receiver: receiver,
            backend_address: address.to_string(),
            commands_rx,
        }
    }
}

impl eframe::App for App {
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {}

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        while let Ok(message) = self.message_receiver.try_recv() {
            log::debug!("got {message:?}");
            match message {
                Message::LoadedFamilyData(root_node) => {
                    self.tree.set_root(Some(root_node));
                    log::debug!("set the root");
                    self.tree.layout();
                    log::debug!("laid out the tree");
                }
            }
        }

        while let Ok(command) = self.commands_rx.try_recv() {
            match command {
                EguiCommand::Ping => {
                    log::info!("ping");
                }
                EguiCommand::HighlightMember(member_id) => {
                    log::info!("{member_id}");
                    self.tree.reset_node_selection();
                    self.tree.focus_node(member_id, ui);
                }
            }
        }

        egui::Panel::top("top_panel").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ui.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                let is_debug = cfg!(debug_assertions);
                if is_debug {
                    egui::widgets::global_theme_preference_buttons(ui);

                    let reload = ui.button("⟳").on_hover_text("Refresh tree");
                    if reload.clicked() {
                        load_family_data(&self.backend_address, self.message_sender.clone(), ui);
                        self.tree.request_recenter();
                    }

                    let label = ui.label("backend address:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.backend_address)
                            .hint_text("http://localhost:3001"),
                    )
                    .labelled_by(label.id);
                }
            });
        });

        if let Some(node) = self.tree.selected_node() {
            egui::Panel::bottom("member").min_size(200.).show(ui, |ui| {
                ui.with_layout(egui::Layout::right_to_left(Align::TOP), |ui| {
                    let image = node.image();
                    egui::Image::new(image)
                        .maintain_aspect_ratio(true)
                        .show_loading_spinner(true)
                        .ui(ui);
                    ui.with_layout(egui::Layout::top_down(Align::RIGHT), |ui| {
                        let heading = TextFormat {
                            font_id: egui::TextStyle::Heading.resolve(ui.style()),
                            color: ui.visuals().text_color(),
                            ..Default::default()
                        };
                        ui.label(bidi_job(node.full_name(), heading.clone()));
                        // Spouses drawn in their own right get no card beside
                        // this node, so this is where they show up.
                        if !node.spouses().is_empty() {
                            ui.add_space(10.);
                            ui.label(bidi_job("الزواج:", heading.clone()));
                            for spouse in node.spouses() {
                                let status = match spouse.status {
                                    MarriageStatus::Married => "متزوج",
                                    MarriageStatus::Separated => "منفصل",
                                };
                                ui.label(bidi_job(
                                    &format!("{}: {status}", spouse.full_name),
                                    heading.clone(),
                                ));
                            }
                        }
                        if let Some(personal_info) = node.personal_info()
                            && !personal_info.is_empty()
                        {
                            ui.add_space(10.);
                            ui.label(bidi_job("المعلومات الشخصية:", heading.clone()));
                            for (key, value) in personal_info {
                                ui.label(bidi_job(&format!("{key}: {value}"), heading.clone()));
                            }
                        }
                    })
                });
            });
        };

        let tree_rect = ui.available_rect_before_wrap();

        egui::CentralPanel::default().show(ui, |ui| {
            if ui.input(|i| i.key_pressed(egui::Key::F5)) {
                load_family_data(&self.backend_address, self.message_sender.clone(), ui);
            }
            self.tree.draw(ui, tree_rect);
        });
    }
}
