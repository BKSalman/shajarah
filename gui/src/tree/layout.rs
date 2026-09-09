use super::{NODE_RADIUS, Node};
use crate::Gender;
use std::collections::{HashMap, HashSet};
const NODE_PADDING: f32 = NODE_RADIUS as f32 * 1.2;
/// Gap between a member's card and each spouse card drawn beside it.
pub const SPOUSE_GAP: f32 = 24.0;
/// What one spouse card adds to a node's width.
pub const SPOUSE_SLOT: f32 = NODE_RADIUS as f32 * 2. + SPOUSE_GAP;
/// this holds all the nodes, and acts as an arena allocator.
/// this is done to be able to mutate nodes while iterating them
/// in different orders
pub struct LayoutTree {
    nodes: Vec<LayoutNode>,
    /// Every id drawn as a node, so a spouse who has their own place in the
    /// tree is never drawn a second time beside their partner.
    ids: HashSet<i64>,
}
impl LayoutTree {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            ids: HashSet::new(),
        }
    }

    /// Whether this member is drawn somewhere in the tree in their own right.
    pub fn has_node(&self, id: i64) -> bool {
        self.ids.contains(&id)
    }
    pub fn update_tree(&mut self, root: Node) {
        let mut tree = Vec::new();
        let mut spouses_by_node: Vec<Vec<i64>> =
            vec![root.spouses.iter().map(|spouse| spouse.id).collect()];
        tree.push(LayoutNode {
            id: root.id,
            order: 0,
            depth: 0,
            gender: root.gender,
            father_idx: None,
            mother_idx: None,
            parent_idx: None,
            children: Vec::new(),
            x: 0.,
            y: 0.,
            mod_: 0.,
            collapsed: root.collapsed,
            spouse_extra: 0.,
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
                    parent_idx: Some(node_idx),
                    gender: child.gender,
                    order: i,
                    depth,
                    children: Vec::new(),
                    x: 0.,
                    y: 0.,
                    mod_: 0.,
                    collapsed: child.collapsed,
                    spouse_extra: 0.,
                });
                spouses_by_node.push(child.spouses.iter().map(|spouse| spouse.id).collect());
                queue.push_back((index, child));
            }
        }
        // A spouse only gets a card when they have no node of their own, so the
        // widths can only be worked out once every id is known.
        let ids: HashSet<i64> = tree.iter().map(|node| node.id).collect();
        for (node, spouses) in tree.iter_mut().zip(spouses_by_node) {
            node.spouse_extra = spouses
                .iter()
                .filter(|spouse_id| !ids.contains(spouse_id))
                .count() as f32
                * SPOUSE_SLOT;
        }

        self.nodes = tree;
        self.ids = ids;
    }
    pub fn set_root(&mut self, root: Option<Node>) {
        if let Some(root) = root {
            self.update_tree(root)
        } else {
            self.nodes = vec![];
            self.ids.clear();
        }
    }
    pub fn reset_positions(&mut self) {
        for node in &mut self.nodes {
            node.x = 0.;
            node.y = 0.;
            node.mod_ = 0.;
        }
    }
    pub fn root(&self) -> Option<usize> {
        if self.nodes.is_empty() { None } else { Some(0) }
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
        let parent = self[node]
            .parent_idx
            .expect("Nodes where `order != 0` always have parents.");
        Some(self[parent].children[order - 1])
    }
    fn left_siblings(&self, node: usize) -> Vec<usize> {
        let order = self[node].order;
        if let Some(parent) = self[node].parent_idx {
            self[parent].children[0..order].into()
        } else {
            Vec::new()
        }
    }
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
                self[node].y = if let Some(parent) = self[node].parent_idx {
                    self[parent].y + NODE_RADIUS as f32 * 2. + NODE_PADDING * 2.
                } else {
                    0.0
                };
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
            if let Some(l) = l.keys().max()
                && let Some(r) = r.keys().max()
            {
                return std::cmp::min(*l, *r);
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
    /// Where `node` sits given the sibling to its left: the two boxes touch
    /// with SIBLING_GAP between them, which for spouse-less nodes is the same
    /// 2R + 2P pitch this used to hard-code.
    fn sibling_x(&self, sibling: usize, node: usize) -> f32 {
        // half_right + half_left is 4R, so this gap keeps the spouse-less pitch
        // at the 2R + 2P (176) it has always been.
        const SIBLING_GAP: f32 = 16.0;

        self[sibling].x + self[sibling].half_right() + self[node].half_left() + SIBLING_GAP
    }

    fn initialize_x(&mut self, root: usize) {
        for node in self.post_order(root) {
            if self[node].is_leaf() {
                self[node].x = if let Some(sibling) = self.previous_sibling(node) {
                    self.sibling_x(sibling, node)
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
                    self[node].x = self.sibling_x(sibling, node);
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
                Some(if curr < acc { curr } else { acc })
            })
            .unwrap_or(0.0);
        self[root].x += shift;
        self[root].mod_ += shift;
    }
    fn finalize_x(&mut self, root: usize) {
        for node in self.breadth_first(root) {
            let shift = if let Some(parent) = self[node].parent_idx {
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
            log::debug!("layed out the tree");
        }
    }
    pub fn get(&self, id: i64) -> Option<&LayoutNode> {
        self.nodes.iter().find(|n| n.id == id)
    }
    pub fn get_mut(&mut self, id: i64) -> Option<&mut LayoutNode> {
        self.nodes.iter_mut().find(|n| n.id == id)
    }
}
impl std::ops::Index<usize> for LayoutTree {
    type Output = LayoutNode;
    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index]
    }
}
impl std::ops::IndexMut<usize> for LayoutTree {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.nodes[index]
    }
}
fn left_contour(tree: &LayoutTree, node: usize) -> HashMap<usize, f32> {
    contour(tree, node, f32::min, |n| n.x - n.half_left())
}
fn right_contour(tree: &LayoutTree, node: usize) -> HashMap<usize, f32> {
    contour(tree, node, f32::max, |n| n.x + n.half_right())
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
    id: i64,
    gender: Gender,
    pub children: Vec<usize>,
    mother_idx: Option<usize>,
    father_idx: Option<usize>,
    /// The node this one hangs off in the drawn tree, whichever parent that is.
    /// `father_idx` alone is `None` for the children of a woman, which used to
    /// strand them at the root and panic `previous_sibling`.
    parent_idx: Option<usize>,
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
    /// Width taken by the spouse cards drawn to the right of this node.
    spouse_extra: f32,
}
impl LayoutNode {
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty() || self.collapsed
    }

    /// How far this node's box reaches left of its centre. Unchanged by
    /// spouses: they are drawn on the right, where the expand indicator isn't.
    fn half_left(&self) -> f32 {
        NODE_RADIUS as f32 * 2. - NODE_PADDING
    }

    /// How far the box reaches right of centre, spouse cards included.
    fn half_right(&self) -> f32 {
        NODE_RADIUS as f32 * 2. + NODE_PADDING + self.spouse_extra
    }

    /// Number of spouse cards drawn beside this node.
    pub fn drawn_spouses(&self) -> usize {
        (self.spouse_extra / SPOUSE_SLOT).round() as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A node with `children`, deserialized the way the server sends it.
    fn node(id: i64, gender: &str, children: serde_json::Value) -> Node {
        serde_json::from_value(serde_json::json!({
            "id": id,
            "name": format!("عضو {id}"),
            "full_name": format!("عضو {id}"),
            "gender": gender,
            "birthday": null,
            "last_name": "العتيبي",
            "father_id": null,
            "mother_id": null,
            "personal_info": null,
            "children": children,
            "image": null,
            "collapsed": false,
        }))
        .expect("node json")
    }

    fn node_with_spouses(id: i64, spouses: serde_json::Value) -> Node {
        let mut value = serde_json::to_value(node(id, "male", serde_json::json!([]))).unwrap();
        value["spouses"] = spouses;
        serde_json::from_value(value).expect("node json")
    }

    fn spouse(id: i64, status: &str) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "name": format!("زوج {id}"),
            "full_name": format!("زوج {id}"),
            "status": status,
        })
    }

    #[test]
    fn siblings_keep_their_spacing_when_nobody_has_a_spouse() {
        let root = node(
            1,
            "male",
            serde_json::json!([
                node(2, "male", serde_json::json!([])),
                node(3, "male", serde_json::json!([])),
            ]),
        );

        let mut tree = LayoutTree::new();
        tree.update_tree(root);
        tree.layout();

        assert_eq!(
            tree[2].x - tree[1].x,
            NODE_RADIUS as f32 * 2. + NODE_PADDING * 2.,
            "the spouse-less pitch must not move"
        );
    }

    #[test]
    fn a_spouse_card_widens_its_node() {
        let mut root = node(
            1,
            "male",
            serde_json::json!([
                node(2, "male", serde_json::json!([])),
                node(3, "male", serde_json::json!([])),
            ]),
        );
        // the first child marries somebody from outside the tree
        root.children[0] = node_with_spouses(2, serde_json::json!([spouse(99, "married")]));

        let mut tree = LayoutTree::new();
        tree.update_tree(root);
        tree.layout();

        assert_eq!(tree[1].drawn_spouses(), 1);
        assert_eq!(
            tree[2].x - tree[1].x,
            NODE_RADIUS as f32 * 2. + NODE_PADDING * 2. + SPOUSE_SLOT,
            "a spouse card must push the next sibling over"
        );
    }

    #[test]
    fn a_spouse_already_in_the_tree_gets_no_card() {
        let mut root = node(
            1,
            "male",
            serde_json::json!([
                node(2, "male", serde_json::json!([])),
                node(3, "female", serde_json::json!([])),
            ]),
        );
        // cousins: the spouse is node 3, who is drawn in their own right
        root.children[0] = node_with_spouses(2, serde_json::json!([spouse(3, "married")]));

        let mut tree = LayoutTree::new();
        tree.update_tree(root);
        tree.layout();

        assert!(tree.has_node(3));
        assert_eq!(
            tree[1].drawn_spouses(),
            0,
            "a spouse with a node of their own must not be drawn twice"
        );
        assert_eq!(
            tree[2].x - tree[1].x,
            NODE_RADIUS as f32 * 2. + NODE_PADDING * 2.,
            "and must not take any width"
        );
    }

    #[test]
    fn a_mother_with_several_children_lays_out() {
        // Children of a woman have no `father_idx`, which used to strand them
        // at the root and panic `previous_sibling` on the second one.
        let root = node(
            1,
            "female",
            serde_json::json!([
                node(2, "male", serde_json::json!([])),
                node(3, "female", serde_json::json!([])),
                node(4, "male", serde_json::json!([])),
            ]),
        );

        let mut tree = LayoutTree::new();
        tree.update_tree(root);
        tree.layout();

        let children: Vec<f32> = (1..=3).map(|idx| tree[idx].x).collect();
        let depths: Vec<usize> = (1..=3).map(|idx| tree[idx].depth).collect();

        assert_eq!(depths, [1, 1, 1]);
        assert!(
            children[0] < children[1] && children[1] < children[2],
            "siblings should be laid out left to right, got {children:?}"
        );
        assert!(
            tree[1].y > tree[0].y,
            "children sit below their mother, got {} vs {}",
            tree[1].y,
            tree[0].y
        );
    }
}
