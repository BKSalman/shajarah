use crate::Gender;
use chrono::{DateTime, Utc};
use eframe::egui::{self, Rect};
use egui::{Vec2, include_image};
use indexmap::IndexMap;
use layout::LayoutTree;
use serde::{Deserialize, Serialize};
pub mod draw;
pub mod layout;

const DEFAULT_IMAGE: egui::ImageSource<'static> = include_image!("../../assets/avatar.png");
const NODE_RADIUS: u8 = 40;

pub struct TreeUi {
    pub offset: Vec2,
    centered: bool,
    scale: f32,
    pub root: Option<Node>,
    pub layout_tree: LayoutTree,
    pub viewport: Rect,
    selected_node: Option<Node>,
}

impl TreeUi {
    pub fn new(root: Option<Node>) -> Self {
        let mut tree = LayoutTree::new();
        tree.set_root(root.clone());
        Self {
            offset: Vec2::ZERO,
            centered: false,
            scale: 1.,
            layout_tree: tree,
            root,
            viewport: Rect::ZERO,
            selected_node: None,
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

    pub fn focus_node(&mut self, id: i32) {
        fn uncollapse_ancestors(node: &mut Node, id: i32) -> bool {
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
                self.offset = Vec2::new(-node.x + center.x, -node.y + center.y);
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
    pub id: i32,
    name: String,
    full_name: String,
    gender: Gender,
    birthday: Option<DateTime<Utc>>,
    last_name: String,
    father_id: Option<i32>,
    mother_id: Option<i32>,
    pub personal_info: Option<IndexMap<String, String>>,
    pub children: Vec<Node>,
    image: Option<Vec<u8>>,
    /// used for displaying or hiding the member info window
    #[serde(skip)]
    window_is_open: bool,
    #[serde(default = "yes")]
    collapsed: bool,
}

impl Node {
    pub fn image<'a>(&'a self) -> egui::ImageSource<'a> {
        self.image
            .as_ref()
            .map(|i| egui::ImageSource::Bytes {
                uri: format!("{}-{}", self.id, self.name).into(),
                bytes: egui::load::Bytes::from(i.clone()),
            })
            .unwrap_or(DEFAULT_IMAGE)
    }

    pub fn name<'a>(&'a self) -> &'a str {
        &self.name
    }

    pub fn full_name<'a>(&'a self) -> &'a str {
        &self.full_name
    }

    pub fn personal_info<'a>(&'a self) -> Option<&'a IndexMap<String, String>> {
        self.personal_info.as_ref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleNode {
    pub id: i32,
    name: String,
    gender: Gender,
    birthday: Option<DateTime<Utc>>,
    last_name: String,
}

impl From<Node> for SimpleNode {
    fn from(value: Node) -> Self {
        Self {
            id: value.id,
            name: value.name,
            gender: value.gender,
            birthday: value.birthday,
            last_name: value.last_name,
        }
    }
}

impl From<&Node> for SimpleNode {
    fn from(value: &Node) -> Self {
        Self {
            id: value.id,
            name: value.name.clone(),
            gender: value.gender,
            birthday: value.birthday,
            last_name: value.last_name.clone(),
        }
    }
}
