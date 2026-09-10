use eframe::egui::{self, UiBuilder};
use egui::Stroke;
#[cfg(feature = "debug-ui")]
use egui::StrokeKind;
use egui::epaint::PathStroke;
use egui::{
    Color32, CornerRadius, FontFamily, FontId, PointerButton, Pos2, Rect, Response, Sense, Shape,
    TextFormat, Vec2, epaint::CubicBezierShape, text::LayoutJob,
};

use super::{
    DEFAULT_IMAGE, MarriageStatus, NODE_RADIUS, Node, Spouse, TreeUi,
    layout::{LayoutTree, SPOUSE_SLOT},
};
use crate::bidi::append_bidi;
use crate::zoom::Zoom;

const MAX_SCALE: f32 = 5.0;
const MIN_SCALE: f32 = 0.2;
const EXPAND_INDICATOR_SIZE: f32 = 20.;

impl TreeUi {
    pub fn draw(&mut self, ui: &mut egui::Ui, rect: Rect) {
        ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
            ui.style_mut().zoom(self.scale);
            let bg_resp = ui.allocate_rect(ui.max_rect(), Sense::click_and_drag());
            let viewport = bg_resp.rect;
            self.viewport = viewport;

            ui.set_clip_rect(self.viewport);

            if bg_resp.dragged() {
                self.pan(bg_resp.drag_delta());
                #[cfg(feature = "debug-ui")]
                log::debug!("new offset: {:?}", self.offset);
            }
            let background_clicked = bg_resp.clicked_by(PointerButton::Primary);

            if bg_resp.double_clicked_by(PointerButton::Primary) {
                self.request_recenter();
            }

            if let Some(hover_pos) = ui.ctx().input(|i| i.pointer.hover_pos())
                && bg_resp.hovered()
            {
                let zoom_delta = ui.ctx().input(|i| i.zoom_delta());
                let pan_delta = ui.ctx().input(|i| i.smooth_scroll_delta);
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
            if let Some(root) = &mut self.root {
                if !self.centered {
                    if let Some(layout_root) = self.layout_tree.root() {
                        let root_coords = &self.layout_tree[layout_root];
                        let center = self.viewport.center().to_vec2();
                        let final_offset = Vec2::new(
                            -root_coords.x * self.scale + center.x,
                            -root_coords.y * self.scale + center.y,
                        );
                        self.offset = final_offset;
                    }
                    self.centered = true;
                }

                let prev_id = self.selected_node.as_ref().map(|n| n.id);

                root.draw(
                    ui,
                    &mut self.offset,
                    self.scale,
                    &mut self.layout_tree,
                    &mut self.selected_node,
                    background_clicked,
                );

                if let Some(node) = &self.selected_node
                    && prev_id != Some(node.id)
                {
                    self.focus_node(node.id, ui.ctx());
                }

                if let Some(animation) = self.animation.take() {
                    ui.ctx().request_repaint();
                    self.animation = animation.animate_selection(&mut self.offset);
                }
            }
        });
    }
}

impl Node {
    pub fn draw(
        &mut self,
        ui: &mut egui::Ui,
        offset: &mut Vec2,
        scale: f32,
        layout_tree: &mut LayoutTree,
        selected_node: &mut Option<Node>,
        background_clicked: bool,
    ) {
        let stroke = ui.visuals().widgets.noninteractive.fg_stroke;
        let node_coords = {
            let layout_node = layout_tree
                .get(self.id)
                .expect("probably didn't update the layout tree");
            Pos2::new(
                offset.x + layout_node.x * scale,
                offset.y + layout_node.y * scale,
            )
        };

        let image_rect = Rect::from_center_size(
            node_coords,
            (Vec2::splat(NODE_RADIUS as f32 * 2.) * scale) + Vec2::splat(1.0),
        );
        let image_res = ui.allocate_rect(image_rect, Sense::click());

        // A spouse who has a node of their own is already drawn there; only the
        // ones from outside the tree get a card beside this node. Cards go to
        // the right, the side the expand indicator leaves free.
        let spouses: Vec<Spouse> = self
            .spouses
            .iter()
            .filter(|spouse| !layout_tree.has_node(spouse.id))
            .cloned()
            .collect();

        let spouse_cards: Vec<(Spouse, Pos2, Rect)> = spouses
            .into_iter()
            .enumerate()
            .map(|(i, spouse)| {
                let center = node_coords + Vec2::new(SPOUSE_SLOT * (i + 1) as f32, 0.) * scale;
                let rect = Rect::from_center_size(
                    center,
                    (Vec2::splat(NODE_RADIUS as f32 * 2.) * scale) + Vec2::splat(1.0),
                );
                (spouse, center, rect)
            })
            .collect();

        // Hover only: the bottom panel shows a real node, and a spouse card
        // isn't one.
        let spouse_responses: Vec<Response> = spouse_cards
            .iter()
            .map(|(spouse, _, rect)| {
                ui.allocate_rect(*rect, Sense::hover())
                    .on_hover_text(&spouse.full_name)
            })
            .collect();

        let indicator_rect = Rect::from_min_size(
            image_rect.min - Vec2::new(EXPAND_INDICATOR_SIZE * 2., 0.),
            Vec2::new(EXPAND_INDICATOR_SIZE * 2., image_rect.height()),
        );
        let indicator_res = ui.allocate_rect(indicator_rect, Sense::click());
        if !self.children.is_empty() && indicator_res.clicked() {
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
        } else if image_res.clicked() {
            if selected_node.as_ref().is_some_and(|sn| sn.id == self.id) {
                selected_node.take();
            } else {
                *selected_node = Some(self.clone());
            }
        }

        let text_style = FontId::new(24.0 * scale, FontFamily::Proportional);
        let painter = ui.painter();
        let mut job = LayoutJob::default();
        append_bidi(
            &mut job,
            &self.name,
            &TextFormat {
                font_id: text_style.clone(),
                color: ui.visuals().text_color(),
                ..Default::default()
            },
        );
        let galley = painter.layout_job(job);
        #[cfg(feature = "debug-ui")]
        let galley_c = galley.clone();
        let text_coords = Pos2::new(
            node_coords.x - galley.size().x / 2.,
            node_coords.y + NODE_RADIUS as f32 * scale,
        );
        let text_size = galley.size();
        painter.galley(text_coords, galley, Color32::WHITE);

        // Each segment joins a card to the one on its left, and carries the
        // state of the marriage it stands for: solid married, dashed separated.
        let mut link_from = node_coords;
        for (spouse, center, _) in &spouse_cards {
            let a = Pos2::new(link_from.x + NODE_RADIUS as f32 * scale, link_from.y);
            let b = Pos2::new(center.x - NODE_RADIUS as f32 * scale, center.y);
            let link_stroke = Stroke::new(stroke.width * 2., stroke.color);

            match spouse.status {
                MarriageStatus::Married => {
                    painter.line_segment([a, b], link_stroke);
                }
                MarriageStatus::Separated => painter.extend(Shape::dashed_line(
                    &[a, b],
                    Stroke::new(link_stroke.width, link_stroke.color.gamma_multiply(0.6)),
                    5.0 * scale,
                    4.0 * scale,
                )),
            }

            link_from = *center;
        }

        if !self.collapsed {
            for child in self.children.iter() {
                let child_coords = layout_tree
                    .get(child.id)
                    .expect("probably didn't update the layout tree");
                let child_coords = Pos2::new(
                    offset.x + child_coords.x * scale,
                    offset.y + child_coords.y * scale,
                );
                if child_coords.x == node_coords.x {
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

        if background_clicked {
            selected_node.take();
        }
        if !self.collapsed {
            for child in self.children.iter_mut() {
                child.draw(
                    ui,
                    offset,
                    scale,
                    layout_tree,
                    selected_node,
                    background_clicked,
                );
            }
        }
        let painter = ui.painter();
        painter.circle_filled(node_coords, NODE_RADIUS as f32 * scale, Color32::LIGHT_BLUE);

        if !self.children.is_empty() {
            let coords = Pos2::new(image_rect.min.x, image_rect.max.y)
                - Vec2::new(EXPAND_INDICATOR_SIZE, EXPAND_INDICATOR_SIZE) * scale;
            if self.collapsed {
                let shape = Shape::convex_polygon(
                    vec![
                        coords,
                        coords
                            + Vec2::new(-EXPAND_INDICATOR_SIZE / 2., -EXPAND_INDICATOR_SIZE / 2.)
                                * scale,
                        coords
                            + Vec2::new(EXPAND_INDICATOR_SIZE / 2., -EXPAND_INDICATOR_SIZE / 2.)
                                * scale,
                    ],
                    stroke.color,
                    PathStroke::NONE,
                );
                painter.add(shape);
            } else {
                let shape = Shape::convex_polygon(
                    vec![
                        coords + Vec2::new(0., -EXPAND_INDICATOR_SIZE / 1.8) * scale,
                        coords + Vec2::new(-EXPAND_INDICATOR_SIZE / 1.8, 0.) * scale,
                        coords + Vec2::new(EXPAND_INDICATOR_SIZE / 1.8, 0.) * scale,
                    ],
                    stroke.color,
                    PathStroke::NONE,
                );
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
        if image_res.hovered() {
            let painter = ui.painter();
            painter.circle_stroke(node_coords, NODE_RADIUS as f32 * scale, stroke);
        }

        for ((spouse, center, rect), response) in spouse_cards.iter().zip(&spouse_responses) {
            let painter = ui.painter();
            painter.circle_filled(*center, NODE_RADIUS as f32 * scale, Color32::LIGHT_GRAY);

            let mut job = LayoutJob::default();
            append_bidi(
                &mut job,
                &spouse.name,
                &TextFormat {
                    font_id: FontId::new(20.0 * scale, FontFamily::Proportional),
                    color: ui.visuals().text_color(),
                    ..Default::default()
                },
            );
            let galley = painter.layout_job(job);
            painter.galley(
                Pos2::new(
                    center.x - galley.size().x / 2.,
                    center.y + NODE_RADIUS as f32 * scale,
                ),
                galley,
                Color32::WHITE,
            );

            egui::Image::new(DEFAULT_IMAGE)
                .corner_radius(CornerRadius::same(NODE_RADIUS) * scale)
                .maintain_aspect_ratio(true)
                .paint_at(ui, *rect);

            if response.hovered() {
                ui.painter()
                    .circle_stroke(*center, NODE_RADIUS as f32 * scale, stroke);
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::layout::LayoutTree;

    fn tree_with_spouse(status: &str, spouse_id: i64) -> (Node, LayoutTree) {
        let root: Node = serde_json::from_value(serde_json::json!({
            "id": 1,
            "name": "فهد",
            "full_name": "فهد العتيبي",
            "gender": "male",
            "birthday": null,
            "last_name": "العتيبي",
            "father_id": null,
            "mother_id": null,
            "personal_info": null,
            "children": [],
            "spouses": [{
                "id": spouse_id,
                "name": "نورة",
                "full_name": "نورة القحطاني",
                "status": status,
            }],
            "image": null,
            "collapsed": false,
        }))
        .expect("node json");

        let mut layout_tree = LayoutTree::new();
        layout_tree.update_tree(root.clone());
        layout_tree.layout();

        (root, layout_tree)
    }

    /// Runs one headless frame and hands back what was painted.
    fn painted(node: &mut Node, layout_tree: &mut LayoutTree) -> Vec<Shape> {
        let ctx = egui::Context::default();
        ctx.set_fonts(egui::FontDefinitions::empty());

        let mut offset = Vec2::ZERO;
        let mut selected = None;

        let mut output = ctx.run_ui(Default::default(), |ui| {
            node.draw(ui, &mut offset, 1.0, layout_tree, &mut selected, false);
        });

        let shapes = output
            .shapes
            .drain(..)
            .map(|clipped| clipped.shape)
            .collect();

        // the frame is never painted, so its texture deltas go unapplied
        output.textures_delta.clear();

        shapes
    }

    fn circles(shapes: &[Shape]) -> Vec<Pos2> {
        shapes
            .iter()
            .filter_map(|shape| match shape {
                Shape::Circle(circle) => Some(circle.center),
                _ => None,
            })
            .collect()
    }

    fn line_segments(shapes: &[Shape]) -> usize {
        shapes
            .iter()
            .filter(|shape| matches!(shape, Shape::LineSegment { .. }))
            .count()
    }

    #[test]
    fn an_outside_spouse_is_drawn_beside_the_member() {
        let (mut root, mut layout_tree) = tree_with_spouse("married", 99);
        let shapes = painted(&mut root, &mut layout_tree);

        let centers = circles(&shapes);
        assert_eq!(
            centers.len(),
            2,
            "the member and the spouse each get a card"
        );
        assert_eq!(
            centers[1].x - centers[0].x,
            crate::tree::layout::SPOUSE_SLOT,
            "the spouse sits one slot to the right"
        );
        assert_eq!(centers[1].y, centers[0].y, "and on the same row");
    }

    #[test]
    fn a_married_link_is_solid_and_a_separated_one_is_dashed() {
        let (mut root, mut layout_tree) = tree_with_spouse("married", 99);
        let married = line_segments(&painted(&mut root, &mut layout_tree));

        let (mut root, mut layout_tree) = tree_with_spouse("separated", 99);
        let separated = line_segments(&painted(&mut root, &mut layout_tree));

        assert_eq!(married, 1, "a marriage is one unbroken line");
        assert!(
            separated > married,
            "a separated marriage is dashed, got {separated} segments"
        );
    }

    #[test]
    fn a_spouse_with_their_own_node_is_not_drawn_twice() {
        // The spouse's id is the member's own node id here, standing in for the
        // cousin case where the spouse is already somewhere in the tree.
        let (mut root, mut layout_tree) = tree_with_spouse("married", 1);
        let shapes = painted(&mut root, &mut layout_tree);

        assert_eq!(
            circles(&shapes).len(),
            1,
            "only the member's own card should be painted"
        );
        assert_eq!(line_segments(&shapes), 0, "and no marriage link");
    }
}
