use std::time::Duration;

use crate::Gender;
use eframe::egui::{self, Rect};
use egui::{Vec2, include_image};
use indexmap::IndexMap;
use jiff::Zoned;
use layout::LayoutTree;
use serde::{Deserialize, Serialize};
pub mod draw;
pub mod layout;

const DEFAULT_IMAGE: egui::ImageSource<'static> = include_image!("../../assets/avatar.png");
const NODE_RADIUS: u8 = 40;

const TREE_MEMORY_ID: &'static str = "treeui";

pub struct TreeUi {
    pub offset: Vec2,
    centered: bool,
    scale: f32,
    pub root: Option<Node>,
    pub layout_tree: LayoutTree,
    pub viewport: Rect,
    selected_node: Option<Node>,
    animation: Option<SelectionAnimation>,
}

impl TreeUi {
    pub fn new(root: Option<Node>, ctx: &egui::Context) -> Self {
        let mut tree = LayoutTree::new();
        tree.set_root(root.clone());

        let animation =
            ctx.memory_mut(|m| m.data.get_temp::<SelectionAnimation>(TREE_MEMORY_ID.into()));

        Self {
            offset: Vec2::ZERO,
            centered: false,
            scale: 1.,
            layout_tree: tree,
            root,
            viewport: Rect::ZERO,
            selected_node: None,
            animation,
        }
    }

    pub fn set_root(&mut self, root: Option<Node>) {
        self.root = root;
        self.layout_tree.set_root(self.root.clone());
    }

    pub fn layout(&mut self) {
        self.layout_tree.layout();
    }

    fn scale(&mut self, new_scale: f32) {
        self.scale = new_scale;
    }

    fn pan(&mut self, delta: Vec2) {
        self.offset += delta;
    }

    pub fn focus_node(&mut self, id: i64, ctx: &egui::Context) {
        fn uncollapse_ancestors(node: &mut Node, id: i64) -> bool {
            if node.id == id {
                return true; // found it, start uncollapsing on the way up
            }
            for child in node.children.iter_mut() {
                if uncollapse_ancestors(child, id) {
                    node.collapsed = false; // uncollapse this ancestor
                    return true;
                }
            }
            false
        }

        if let Some(root) = self.root.as_mut() {
            uncollapse_ancestors(root, id);
            self.root = Some(root.clone());
            self.layout_tree.set_root(self.root.clone());
            self.layout();

            if let Some(node) = self.layout_tree.get(id) {
                let center = self.viewport.center().to_vec2();
                let final_offset = Vec2::new(
                    -node.x * self.scale + center.x,
                    -node.y * self.scale + center.y,
                );

                let animation = SelectionAnimation::new(
                    self.offset.clone(),
                    Duration::from_millis(75),
                    final_offset,
                );

                ctx.memory_mut(|m| m.data.insert_temp(TREE_MEMORY_ID.into(), animation.clone()));

                self.animation = Some(animation);
            }
        }
    }

    pub fn request_recenter(&mut self) {
        self.centered = false;
    }

    pub fn selected_node(&self) -> Option<&Node> {
        self.selected_node.as_ref()
    }

    pub fn reset_node_selection(&mut self) {
        self.selected_node.take();
    }
}

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: i64,
    name: String,
    full_name: String,
    gender: Gender,
    birthday: Option<Zoned>,
    last_name: String,
    father_id: Option<i32>,
    mother_id: Option<i32>,
    pub personal_info: Option<IndexMap<String, String>>,
    pub children: Vec<Node>,
    image: Option<Vec<u8>>,
    #[serde(default = "yes")]
    collapsed: bool,
}

impl Node {
    pub fn image(&self) -> egui::ImageSource<'_> {
        self.image
            .as_ref()
            .map(|i| egui::ImageSource::Bytes {
                uri: format!("{}-{}", self.id, self.name).into(),
                bytes: egui::load::Bytes::from(i.clone()),
            })
            .unwrap_or(DEFAULT_IMAGE)
    }

    pub fn full_name(&self) -> &str {
        &self.full_name
    }

    pub fn personal_info(&self) -> Option<&IndexMap<String, String>> {
        self.personal_info.as_ref()
    }
}

#[derive(Clone)]
struct SelectionAnimation {
    start: jiff::Timestamp,
    duration: Duration,
    starting_offset: Vec2,
    final_offset: Vec2,
}

impl SelectionAnimation {
    fn new(starting_offset: Vec2, duration: Duration, final_offset: Vec2) -> Self {
        Self {
            start: jiff::Timestamp::now(),
            duration,
            starting_offset,
            final_offset,
        }
    }

    /// Returns `None` once finished
    fn animate_selection(self, current_offset: &mut Vec2) -> Option<Self> {
        let elapsed = jiff::Timestamp::now()
            .duration_since(self.start)
            .unsigned_abs();
        if elapsed >= self.duration {
            *current_offset = self.final_offset;
            return None;
        }
        let t = elapsed.as_secs_f32() / self.duration.as_secs_f32();
        *current_offset = self.starting_offset + (self.final_offset - self.starting_offset) * t;
        Some(self)
    }
}
