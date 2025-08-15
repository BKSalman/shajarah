use chrono::{DateTime, Utc};

use eframe::egui;

use egui::{Vec2, include_image};

use indexmap::IndexMap;
use layout::LayoutTree;
use serde::{Deserialize, Serialize};

use crate::Gender;

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

    pub fn request_recenter(&mut self) {
        self.centered = false;
    }

    // fn screen_pos_to_graph(&self, pos: Pos2, viewport: Rect) -> Pos2 {
    //     (pos + self.offset - viewport.center().to_vec2()) / self.scale
    // }
}

// #[derive(Serialize, Deserialize)]
// pub struct Node {
//     id: usize,
//     name: String,
//     #[serde(skip)]
//     window_is_open: bool,

//     children: Vec<Node>,
// }

fn yes() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: i32,
    name: String,
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
    // pub fn add_child(&mut self, child: Node) {
    //     self.children.push(child);
    // }
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
