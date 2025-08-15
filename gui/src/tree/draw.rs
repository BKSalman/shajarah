use ar_reshaper::{ArabicReshaper, ReshaperConfig, config::LigaturesFlags};
use eframe::egui;

use egui::Stroke;
use egui::epaint::PathStroke;
use egui::{
    Align, Color32, CornerRadius, FontFamily, FontId, PointerButton, Pos2, Rect, Sense, Shape,
    TextFormat, Vec2, Vec2b, Widget, epaint::CubicBezierShape, text::LayoutJob,
};
use unicode_bidi::BidiInfo;

use crate::zoom::Zoom;

#[cfg(feature = "debug-ui")]
use egui::StrokeKind;

use super::{DEFAULT_IMAGE, NODE_RADIUS, Node, SimpleNode, TreeUi, layout::LayoutTree};

const MAX_SCALE: f32 = 5.0;
const MIN_SCALE: f32 = 0.2;
const RESHAPER: ArabicReshaper = ArabicReshaper::new(ReshaperConfig::new(
    ar_reshaper::Language::Arabic,
    LigaturesFlags::default(),
));

const EXPAND_INDICATOR_SIZE: f32 = 20.;

impl TreeUi {
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        ui.style_mut().zoom(self.scale);

        let bg_resp = ui.allocate_rect(ui.max_rect(), Sense::click_and_drag());
        let viewport = bg_resp.rect;
        ui.set_clip_rect(viewport);

        if bg_resp.dragged() {
            self.pan(bg_resp.drag_delta());

            #[cfg(feature = "debug-ui")]
            log::debug!("new offset: {:?}", self.offset);
        }

        let background_clicked = bg_resp.clicked_by(PointerButton::Primary);

        if let Some(hover_pos) = ui.ctx().input(|i| i.pointer.hover_pos()) {
            if bg_resp.hovered() {
                let zoom_delta = ui.ctx().input(|i| i.zoom_delta());
                let pan_delta = ui.ctx().input(|i| i.smooth_scroll_delta);

                // there is change
                if zoom_delta != 1. {
                    let prev_scale = self.scale;
                    let new_scale = (prev_scale * zoom_delta).clamp(MIN_SCALE, MAX_SCALE);

                    self.scale(new_scale);
                    let scale_factor = self.scale / prev_scale;
                    let pos = self.offset - hover_pos.to_vec2();

                    self.offset = (pos * scale_factor) + hover_pos.to_vec2();

                    #[cfg(feature = "debug-ui")]
                    log::debug!("new offset: {:?}", self.offset);
                }

                self.pan(pan_delta);
            }
        }

        if let Some(root) = &mut self.root {
            if !self.centered {
                if let Some(layout_root) = self.layout_tree.root() {
                    let root_coords = &self.layout_tree[layout_root];
                    let center = viewport.center().to_vec2();
                    // log::debug!("{center}");

                    self.offset = Vec2::new(-root_coords.x + center.x, center.y);

                    #[cfg(feature = "debug-ui")]
                    log::debug!("offset: {:?}", self.offset);
                }

                self.centered = true;
            }

            root.draw(
                ui,
                &mut self.offset,
                self.scale,
                &mut self.layout_tree,
                vec![],
                background_clicked,
            );
        }
    }
}

impl Node {
    pub fn draw(
        &mut self,
        ui: &mut egui::Ui,
        offset: &mut Vec2,
        scale: f32,
        layout_tree: &mut LayoutTree,
        mut lineage: Vec<SimpleNode>,
        background_clicked: bool,
    ) {
        let stroke = ui.visuals().widgets.noninteractive.fg_stroke;
        let coords = {
            let layout_node = layout_tree
                .get(self.id)
                .expect("probably didn't update the layout tree");

            Pos2::new(
                offset.x + layout_node.x * scale,
                offset.y + layout_node.y * scale,
            )
        };

        let image_rect = Rect::from_center_size(
            coords,
            (Vec2::splat(NODE_RADIUS as f32 * 2.) * scale) + Vec2::splat(1.0), // add one pixel to cover the whole background circle
        );

        let response = ui.allocate_rect(image_rect, Sense::click());

        if response.clicked() {
            self.collapsed = !self.collapsed;
            layout_tree
                .get_mut(self.id)
                .expect("node should exist in layout tree")
                .collapsed = self.collapsed;

            let prev_x = layout_tree
                .get(self.id)
                .expect("node should exist in layout tree")
                .x;

            layout_tree.layout();

            let new_node = layout_tree
                .get(self.id)
                .expect("node should exist in layout tree");

            offset.x -= (new_node.x - prev_x) * scale;
        }

        if response.secondary_clicked() {
            self.window_is_open = !self.window_is_open;
        }

        let text_style = FontId::new(24.0 * scale, FontFamily::Monospace);

        let painter = ui.painter();

        let mut job = LayoutJob::default();
        job.append(
            &shape_text(&self.name),
            0.0,
            TextFormat {
                font_id: text_style.clone(),
                color: ui.visuals().text_color(),
                ..Default::default()
            },
        );
        let galley = painter.layout_job(job);

        #[cfg(feature = "debug-ui")]
        let galley_c = galley.clone();

        let text_coords = Pos2::new(
            coords.x - galley.size().x / 2.,
            coords.y + NODE_RADIUS as f32 * scale,
        );
        let text_size = galley.size();

        painter.galley(text_coords, galley, Color32::WHITE);

        if !self.collapsed {
            for child in self.children.iter() {
                let child_coords = layout_tree
                    .get(child.id)
                    .expect("probably didn't update the layout tree");
                let child_coords = Pos2::new(
                    offset.x + child_coords.x * scale,
                    offset.y + child_coords.y * scale,
                );

                if child_coords.x == coords.x {
                    painter.line_segment(
                        [
                            child_coords,
                            text_coords + Vec2::new(text_size.x / 2., text_size.y),
                        ],
                        Stroke::new(stroke.width * 2., stroke.color),
                    );
                } else {
                    let control_point1 =
                        Pos2::new(text_coords.x + text_size.x / 2., child_coords.y);

                    #[cfg(feature = "debug-ui")]
                    painter.circle_filled(control_point1, 10., Color32::WHITE);

                    let control_point2 = Pos2::new(child_coords.x, text_coords.y + text_size.y);

                    #[cfg(feature = "debug-ui")]
                    painter.circle_filled(control_point2, 10., Color32::YELLOW);

                    painter.add(Shape::CubicBezier(CubicBezierShape::from_points_stroke(
                        [
                            Pos2::new(
                                text_coords.x + text_size.x / 2.,
                                text_coords.y + text_size.y,
                            ),
                            control_point1,
                            control_point2,
                            Pos2::new(child_coords.x, child_coords.y),
                        ],
                        false,
                        Color32::TRANSPARENT,
                        Stroke::new(stroke.width * 2., stroke.color),
                    )));
                }
            }
        }

        let window_pos =
            coords + Vec2::new(NODE_RADIUS as f32 * 1.2, -(NODE_RADIUS as f32 / 2.)) * scale;

        if background_clicked && self.window_is_open {
            self.window_is_open = false;
        }

        egui::Window::new(self.id.to_string())
            .id(egui::Id::new(self.id))
            .max_width(180.)
            .auto_sized()
            .resizable(false)
            .constrain(false)
            .default_pos(window_pos)
            .collapsible(false)
            .title_bar(false)
            .scroll(Vec2b::TRUE)
            .enabled(true)
            .open(&mut self.window_is_open)
            .current_pos(window_pos)
            .show(ui.ctx(), |ui| {
                ui.with_layout(egui::Layout::top_down(Align::RIGHT), |ui| {
                    let image = self
                        .image
                        .as_ref()
                        .map(|i| egui::ImageSource::Bytes {
                            uri: format!("{}-{}", self.id, self.name).into(),
                            bytes: egui::load::Bytes::from(i.clone()),
                        })
                        .unwrap_or(DEFAULT_IMAGE);

                    egui::Image::new(image)
                        .maintain_aspect_ratio(true)
                        .show_loading_spinner(true)
                        .ui(ui);

                    #[cfg(feature = "debug-ui")]
                    {
                        let layout_node = layout_tree
                            .get(self.id)
                            .expect("probably didn't update the layout tree");

                        ui.label(format!("{{ x: {}, y: {} }}", layout_node.x, layout_node.y));

                        ui.label(layout_node.depth.to_string());
                    }

                    ui.label(self.id.to_string());

                    let lineage = lineage
                        .iter()
                        .rev()
                        .map(|l| format!("{} ", l.name.clone()))
                        .take(2)
                        .collect::<String>();
                    ui.heading(shape_text(&format!(
                        "{} {}{}",
                        self.name, lineage, self.last_name
                    )));

                    if let Some(personal_info) = self.personal_info.as_ref() {
                        if !personal_info.is_empty() {
                            ui.add_space(10.);
                            ui.label(shape_text("المعلومات الشخصية:"));

                            for (key, value) in personal_info {
                                let key = shape_text(&format!("{key}: "));
                                let value = shape_text(value);
                                ui.horizontal(|ui| {
                                    ui.label(key);
                                    ui.label(value);
                                });
                            }
                        }
                    }
                });
            });

        lineage.push(self.clone().into());

        if !self.collapsed {
            for child in self.children.iter_mut() {
                child.draw(
                    ui,
                    offset,
                    scale,
                    layout_tree,
                    lineage.clone(),
                    background_clicked,
                );
            }
        }

        let painter = ui.painter();
        painter.circle_filled(coords, NODE_RADIUS as f32 * scale, Color32::LIGHT_BLUE);

        if !self.children.is_empty() {
            if self.collapsed {
                let mut shape = Shape::convex_polygon(
                    vec![
                        coords + Vec2::new(-(NODE_RADIUS as f32), NODE_RADIUS as f32) * scale,
                        coords
                            + Vec2::new(-(NODE_RADIUS as f32), NODE_RADIUS as f32) * scale
                            + Vec2::new(-EXPAND_INDICATOR_SIZE / 2., -EXPAND_INDICATOR_SIZE / 2.)
                                * scale,
                        coords
                            + Vec2::new(-(NODE_RADIUS as f32), NODE_RADIUS as f32) * scale
                            + Vec2::new(EXPAND_INDICATOR_SIZE / 2., -EXPAND_INDICATOR_SIZE / 2.)
                                * scale,
                    ],
                    stroke.color,
                    PathStroke::NONE,
                );

                shape.translate(Vec2::new(-1., 5.));

                painter.add(shape);
            } else {
                let mut shape = Shape::convex_polygon(
                    vec![
                        coords
                            + Vec2::new(-(NODE_RADIUS as f32), NODE_RADIUS as f32) * scale
                            + Vec2::new(0., -EXPAND_INDICATOR_SIZE / 1.8) * scale,
                        coords
                            + Vec2::new(-(NODE_RADIUS as f32), NODE_RADIUS as f32) * scale
                            + Vec2::new(-EXPAND_INDICATOR_SIZE / 1.8, 0.) * scale,
                        coords
                            + Vec2::new(-(NODE_RADIUS as f32), NODE_RADIUS as f32) * scale
                            + Vec2::new(EXPAND_INDICATOR_SIZE / 1.8, 0.) * scale,
                    ],
                    stroke.color,
                    PathStroke::NONE,
                );

                shape.translate(Vec2::new(-1., 5.));

                painter.add(shape);
            }
        }

        #[cfg(feature = "debug-ui")]
        painter.rect_stroke(
            image_rect,
            CornerRadius::ZERO,
            Stroke::new(2.0, Color32::GREEN),
            StrokeKind::Middle,
        );

        let image = self
            .image
            .as_ref()
            .map(|i| egui::ImageSource::Bytes {
                uri: format!("{}-{}", self.id, self.name).into(),
                bytes: egui::load::Bytes::from(i.clone()),
            })
            .unwrap_or(DEFAULT_IMAGE);

        egui::Image::new(image)
            .corner_radius(CornerRadius::same(NODE_RADIUS) * scale)
            .maintain_aspect_ratio(true)
            .show_loading_spinner(true)
            .paint_at(ui, image_rect);

        if response.hovered() {
            let painter = ui.painter();
            painter.circle_stroke(coords, NODE_RADIUS as f32 * scale, stroke);
        }

        #[cfg(feature = "debug-ui")]
        painter.rect_stroke(
            Rect {
                min: Pos2::new(text_coords.x, text_coords.y),
                max: Pos2::new(
                    text_coords.x + galley_c.size().x,
                    text_coords.y + galley_c.size().y,
                ),
            },
            CornerRadius::ZERO,
            Stroke::new(1., Color32::GREEN),
            StrokeKind::Middle,
        );
    }
}

fn shape_text(input: &str) -> String {
    let mut output = String::new();
    if input.is_empty() {
        return output;
    }

    let bidi_info = BidiInfo::new(input, None);
    for paragraph in bidi_info.paragraphs.iter() {
        let (levels, runs) = bidi_info.visual_runs(paragraph, paragraph.range.clone());

        for run in runs {
            let run_level = levels[run.start];
            let text = &input[paragraph.range.clone()][run];

            if run_level.is_rtl() {
                output.push_str(&RESHAPER.reshape(text).chars().rev().collect::<String>());
            } else {
                output.push_str(text);
            }
        }
    }

    output
}
