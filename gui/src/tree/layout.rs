use std::collections::HashMap;

use crate::Gender;

use super::{NODE_RADIUS, Node};

const NODE_PADDING: f32 = NODE_RADIUS as f32 * 1.2;

/// this holds all the nodes, and acts as an arena allocator.
/// this is done to be able to mutate nodes while iterating them
/// in different orders
// the layout code is based on the Reingold Tilford algorithm explained in this blog post:
// https://rachel53461.wordpress.com/2014/04/20/algorithm-for-drawing-trees/
// and shamelessly stolen from https://gitlab.com/seamsay/reingold-tilford
pub struct LayoutTree(Vec<LayoutNode>);

impl LayoutTree {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn update_tree(&mut self, root: Node) {
        let mut tree = Vec::new();
        tree.push(LayoutNode {
            id: root.id,
            order: 0,
            depth: 0,

            gender: root.gender,
            father_idx: None,
            mother_idx: None,
            children: Vec::new(),

            x: 0.,
            y: 0.,
            mod_: 0.,
            collapsed: root.collapsed,
        });

        let mut queue = std::collections::VecDeque::new();
        queue.push_back((0, root));

        while let Some((node_idx, node)) = queue.pop_front() {
            let index = tree.len();

            for (i, child) in node.children.into_iter().enumerate() {
                let index = index + i;
                let depth = tree[node_idx].depth + 1;

                tree[node_idx].children.push(index);

                let mother_idx = match node.gender {
                    Gender::Male => None,
                    Gender::Female => Some(node_idx),
                };

                let father_idx = match node.gender {
                    Gender::Male => Some(node_idx),
                    Gender::Female => None,
                };

                tree.push(LayoutNode {
                    id: child.id,
                    mother_idx,
                    father_idx,
                    gender: child.gender,

                    order: i,
                    depth,

                    children: Vec::new(),
                    x: 0.,
                    y: 0.,
                    mod_: 0.,
                    collapsed: child.collapsed,
                });

                queue.push_back((index, child));
            }
        }

        self.0 = tree;
    }

    pub fn set_root(&mut self, root: Option<Node>) {
        if let Some(root) = root {
            self.update_tree(root)
        } else {
            self.0 = vec![];
        }
    }

    pub fn reset_positions(&mut self) {
        for node in &mut self.0 {
            node.x = 0.;
            node.y = 0.;
            node.mod_ = 0.;
        }
    }

    pub fn root(&self) -> Option<usize> {
        if self.0.is_empty() { None } else { Some(0) }
    }

    fn post_order(&self, node: usize) -> Vec<usize> {
        let mut breadth_first = vec![node];
        let mut post_order = Vec::new();

        while let Some(node) = breadth_first.pop() {
            if !self[node].collapsed {
                breadth_first.extend_from_slice(&self[node].children);
            }
            post_order.push(node);
        }

        post_order.reverse();
        post_order
    }

    fn previous_sibling(&self, node: usize) -> Option<usize> {
        let order = self[node].order;
        if order == 0 {
            return None;
        }

        let father = self[node]
            .father_idx
            .expect("Nodes where `order != 0` always have parents.");

        Some(self[father].children[order - 1])
    }

    fn left_siblings(&self, node: usize) -> Vec<usize> {
        let order = self[node].order;

        if let Some(parent) = self[node].father_idx {
            self[parent].children[0..order].into()
        } else {
            Vec::new()
        }
    }

    // fn siblings_between(&self, left: usize, right: usize) -> Vec<usize> {
    //     let left_order = self[left].order;
    //     let right_order = self[right].order;

    //     if self[left].is_root() || self[right].is_root() {
    //         assert!(
    //             self[left].is_root(),
    //             "If one node is the root then both nodes must be."
    //         );
    //         assert!(
    //             self[right].is_root(),
    //             "If one node is the root then both nodes must be."
    //         );

    //         return Vec::new();
    //     }

    //     let left_parent = self[left]
    //         .father_idx
    //         .expect("`is_none` has already been checked.");

    //     let right_parent = self[right]
    //         .father_idx
    //         .expect("`is_none` has already been checked.");

    //     assert!(
    //         left_parent == right_parent,
    //         "Nodes must actually be siblings."
    //     );

    //     let parent = left_parent;

    //     self[parent].children[left_order + 1..right_order].into()
    // }

    fn breadth_first(&self, node: usize) -> Vec<usize> {
        let mut breadth_first = vec![node];
        let mut index = 0;

        while index < breadth_first.len() {
            let node = breadth_first[index];
            if !self[node].collapsed {
                breadth_first.extend_from_slice(&self[node].children);
            }
            index += 1;
        }

        breadth_first
    }

    fn initialize_y(&mut self, root: usize) {
        let mut next_row = vec![root];

        while !next_row.is_empty() {
            let row = next_row;
            next_row = Vec::new();

            let mut max = -f32::INFINITY;
            for node in &row {
                let node = *node;

                // TODO: handle mother_idx as well
                self[node].y = if let Some(parent) = self[node].father_idx {
                    self[parent].y + NODE_RADIUS as f32 * 2. + NODE_PADDING * 2.
                } else {
                    0.0
                }; //  + self[node].data.top_space();

                if self[node].y > max {
                    max = self[node].y;
                }

                if !self[node].collapsed {
                    next_row.extend_from_slice(&self[node].children);
                }
            }

            for node in &row {
                self[*node].y = max;
            }
        }
    }

    fn fix_overlaps(&mut self, right: usize) {
        fn max_depth(l: &HashMap<usize, f32>, r: &HashMap<usize, f32>) -> usize {
            if let Some(l) = l.keys().max() {
                if let Some(r) = r.keys().max() {
                    return std::cmp::min(*l, *r);
                }
            }

            0
        }

        let right_node_contour = left_contour(self, right);

        for left in self.left_siblings(right) {
            let left_node_contour = right_contour(self, left);
            let mut shift = 0.0;

            log::debug!(
                "{right}::{}:: left contour: {right_node_contour:#?}, right contour: {left_node_contour:#?}",
                self[right].depth
            );

            for depth in self[right].depth..=max_depth(&right_node_contour, &left_node_contour) {
                let gap = right_node_contour[&depth] - left_node_contour[&depth];
                if gap + shift < 0.0 {
                    shift = -gap;
                }
            }

            log::debug!("left: {left}, right: {right}, shift: {shift}");

            self[right].x += shift;
            self[right].mod_ += shift;
        }
    }

    fn initialize_x(&mut self, root: usize) {
        for node in self.post_order(root) {
            if self[node].is_leaf() {
                self[node].x = if let Some(sibling) = self.previous_sibling(node) {
                    self[sibling].x + NODE_RADIUS as f32 * 2. + NODE_PADDING * 2.
                } else {
                    0.0
                };
            } else {
                let mid = {
                    let first = self[*self[node]
                        .children
                        .first()
                        .expect("Only leaf nodes have no children.")]
                    .x;
                    let last = self[*self[node]
                        .children
                        .last()
                        .expect("Only leaf nodes have no children.")]
                    .x;

                    (first + last) / 2.0
                };

                if let Some(sibling) = self.previous_sibling(node) {
                    self[node].x = self[sibling].x + NODE_RADIUS as f32 * 2. + NODE_PADDING * 2.;
                    self[node].mod_ = self[node].x - mid;
                } else {
                    self[node].x = mid;
                }

                self.fix_overlaps(node);
            }
        }
    }

    fn ensure_positive_x(&mut self, root: usize) {
        let contour = left_contour(self, root);
        let shift = -contour
            .values()
            .fold(None, |acc, curr| {
                let acc = acc.unwrap_or(f32::INFINITY);
                let curr = *curr;
                // log::debug!("curr: {curr}, acc: {acc}");
                Some(if curr < acc { curr } else { acc })
            })
            .unwrap_or(0.0);

        // log::debug!("shift: {shift}");

        self[root].x += shift;
        self[root].mod_ += shift;
    }

    fn finalize_x(&mut self, root: usize) {
        for node in self.breadth_first(root) {
            let shift = if let Some(parent) = self[node].father_idx {
                self[parent].mod_
            } else {
                0.0
            };

            self[node].x += shift;
            self[node].mod_ += shift;
        }
    }

    pub fn layout(&mut self) {
        log::debug!("laying out the tree");

        if let Some(root) = self.root() {
            self.reset_positions();

            self.initialize_y(root);
            self.initialize_x(root);

            self.ensure_positive_x(root);
            self.finalize_x(root);
        }
    }

    pub fn get(&self, id: i32) -> Option<&LayoutNode> {
        self.0.iter().find(|n| n.id == id)
    }

    pub fn get_mut(&mut self, id: i32) -> Option<&mut LayoutNode> {
        self.0.iter_mut().find(|n| n.id == id)
    }
}

impl std::ops::Index<usize> for LayoutTree {
    type Output = LayoutNode;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl std::ops::IndexMut<usize> for LayoutTree {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

fn left_contour(tree: &LayoutTree, node: usize) -> HashMap<usize, f32> {
    contour(tree, node, f32::min, |n| {
        n.x - NODE_RADIUS as f32 * 2. + NODE_PADDING
    })
}

fn right_contour(tree: &LayoutTree, node: usize) -> HashMap<usize, f32> {
    contour(tree, node, f32::max, |n| {
        n.x + NODE_RADIUS as f32 * 2. + NODE_PADDING
    })
}

fn contour<C, E>(tree: &LayoutTree, node: usize, cmp: C, edge: E) -> HashMap<usize, f32>
where
    C: Fn(f32, f32) -> f32,
    E: Fn(&LayoutNode) -> f32,
{
    let mut stack = vec![(0.0, node)];
    let mut contour = HashMap::new();

    while let Some((mod_, node)) = stack.pop() {
        let depth = tree[node].depth;
        let shifted = edge(&tree[node]) + mod_;
        let new = if let Some(current) = contour.get(&depth) {
            cmp(*current, shifted)
        } else {
            shifted
        };
        let mod_ = mod_ + tree[node].mod_;

        contour.insert(depth, new);

        if !tree[node].collapsed {
            stack.extend(tree[node].children.iter().map(|c| (mod_, *c)));
        }
    }

    contour
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LayoutNode {
    id: i32,
    gender: Gender,

    pub children: Vec<usize>,

    mother_idx: Option<usize>,
    father_idx: Option<usize>,

    /// The position of this node among it's siblings.
    ///
    /// Can also be thought of as the number of left-siblings this node has.
    pub order: usize,
    /// The depth of this node.
    ///
    /// Can be thought of as the number of edges between this node and the root node.
    pub depth: usize,

    pub x: f32,
    pub y: f32,

    /// How much to shift this node's children by
    mod_: f32,

    pub collapsed: bool,
}

impl LayoutNode {
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty() || self.collapsed
    }

    // pub fn is_root(&self) -> bool {
    //     self.mother_idx.is_none() && self.father_idx.is_none()
    // }
}
