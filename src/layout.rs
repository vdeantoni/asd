use ratatui::layout::Direction;

pub type NodeId = usize;

pub enum SplitNode {
    Leaf {
        slot: usize,
        scroll_y: u16,
        scroll_x: u16,
    },
    Split {
        direction: Direction,
        a: NodeId,
        b: NodeId,
    },
}

pub struct SplitTree {
    pub nodes: Vec<SplitNode>,
    pub root: NodeId,
    pub focused: NodeId,
    pub leaf_count: usize,
}

impl SplitTree {
    pub fn new() -> Self {
        Self {
            nodes: vec![SplitNode::Leaf { slot: 0, scroll_y: 0, scroll_x: 0 }],
            root: 0,
            focused: 0,
            leaf_count: 1,
        }
    }

    /// Split a specific leaf node. The existing content stays as child a,
    /// and a new leaf (next slot) becomes child b.
    /// Returns (child_a_id, child_b_id).
    pub fn split_leaf(&mut self, target: NodeId, direction: Direction) -> (NodeId, NodeId) {
        let current_slot = match &self.nodes[target] {
            SplitNode::Leaf { slot, .. } => *slot,
            _ => return (target, target),
        };

        let new_slot = self.leaf_count;
        let a_id = self.nodes.len();
        let b_id = a_id + 1;

        self.nodes.push(SplitNode::Leaf {
            slot: current_slot,
            scroll_y: 0,
            scroll_x: 0,
        });
        self.nodes.push(SplitNode::Leaf {
            slot: new_slot,
            scroll_y: 0,
            scroll_x: 0,
        });

        self.nodes[target] = SplitNode::Split {
            direction,
            a: a_id,
            b: b_id,
        };

        // Don't change focus here — callers decide.
        // If focused was the target, it now points at a Split node;
        // the caller must fix it.
        self.leaf_count += 1;
        self.reassign_slots();

        (a_id, b_id)
    }

    /// Close the focused pane. Replaces its parent split with the sibling.
    pub fn close_focused(&mut self) -> bool {
        if self.leaf_count <= 1 {
            return false;
        }

        let parent = match self.find_parent(self.focused) {
            Some(p) => p,
            None => return false,
        };

        let sibling_id = match &self.nodes[parent] {
            SplitNode::Split { a, b, .. } => {
                if *a == self.focused { *b } else { *a }
            }
            _ => return false,
        };

        // Replace parent with sibling content
        let sibling = std::mem::replace(
            &mut self.nodes[sibling_id],
            SplitNode::Leaf { slot: 0, scroll_y: 0, scroll_x: 0 },
        );
        self.nodes[parent] = sibling;

        self.focused = parent;
        self.leaf_count -= 1;

        // If focused landed on a Split, find a leaf within it
        if !matches!(self.nodes[self.focused], SplitNode::Leaf { .. }) {
            let leaves = self.collect_leaves(self.focused);
            if let Some(&last) = leaves.last() {
                self.focused = last;
            }
        }

        self.reassign_slots();
        true
    }

    /// Cycle focus to the next leaf pane.
    pub fn cycle_focus(&mut self) {
        let leaves = self.collect_leaves(self.root);
        if leaves.is_empty() {
            return;
        }
        let current_pos = leaves.iter().position(|&id| id == self.focused).unwrap_or(0);
        let next_pos = (current_pos + 1) % leaves.len();
        self.focused = leaves[next_pos];
    }

    /// Collect all leaf node IDs in traversal order.
    pub fn collect_leaves(&self, node_id: NodeId) -> Vec<NodeId> {
        match &self.nodes[node_id] {
            SplitNode::Leaf { .. } => vec![node_id],
            SplitNode::Split { a, b, .. } => {
                let mut leaves = self.collect_leaves(*a);
                leaves.extend(self.collect_leaves(*b));
                leaves
            }
        }
    }

    fn find_parent(&self, target: NodeId) -> Option<NodeId> {
        self.find_parent_rec(self.root, target)
    }

    fn find_parent_rec(&self, current: NodeId, target: NodeId) -> Option<NodeId> {
        match &self.nodes[current] {
            SplitNode::Leaf { .. } => None,
            SplitNode::Split { a, b, .. } => {
                if *a == target || *b == target {
                    return Some(current);
                }
                self.find_parent_rec(*a, target)
                    .or_else(|| self.find_parent_rec(*b, target))
            }
        }
    }

    fn reassign_slots(&mut self) {
        let leaves = self.collect_leaves(self.root);
        for (i, leaf_id) in leaves.iter().enumerate() {
            if let SplitNode::Leaf { slot, .. } = &mut self.nodes[*leaf_id] {
                *slot = i;
            }
        }
    }

    pub fn scroll_y(&mut self, delta: i16, max_lines: u16) {
        if let SplitNode::Leaf { scroll_y, .. } = &mut self.nodes[self.focused] {
            let new = (*scroll_y as i16 + delta).max(0) as u16;
            *scroll_y = new.min(max_lines);
        }
    }

    pub fn scroll_x(&mut self, delta: i16) {
        if let SplitNode::Leaf { scroll_x, .. } = &mut self.nodes[self.focused] {
            *scroll_x = (*scroll_x as i16 + delta).max(0) as u16;
        }
    }

    pub fn reset_all_scroll(&mut self) {
        for node in &mut self.nodes {
            if let SplitNode::Leaf { scroll_y, scroll_x, .. } = node {
                *scroll_y = 0;
                *scroll_x = 0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Direction;

    fn assert_slots_sequential(tree: &SplitTree) {
        for (i, &leaf_id) in tree.collect_leaves(tree.root).iter().enumerate() {
            if let SplitNode::Leaf { slot, .. } = &tree.nodes[leaf_id] {
                assert_eq!(*slot, i, "leaf {leaf_id} has wrong slot");
            }
        }
    }

    #[test]
    fn new_tree_single_leaf() {
        let tree = SplitTree::new();
        assert_eq!(tree.root, 0);
        assert_eq!(tree.focused, 0);
        assert_eq!(tree.leaf_count, 1);
        assert!(matches!(tree.nodes[0], SplitNode::Leaf { slot: 0, .. }));
    }

    #[test]
    fn split_leaf_creates_children() {
        let mut tree = SplitTree::new();
        let (a, b) = tree.split_leaf(0, Direction::Horizontal);

        assert_eq!(tree.leaf_count, 2);
        assert!(matches!(tree.nodes[0], SplitNode::Split { .. }));
        assert!(matches!(tree.nodes[a], SplitNode::Leaf { .. }));
        assert!(matches!(tree.nodes[b], SplitNode::Leaf { .. }));
        assert_ne!(a, b);
    }

    #[test]
    fn split_returns_correct_child_ids() {
        let mut tree = SplitTree::new();
        let (a, b) = tree.split_leaf(0, Direction::Vertical);

        // a should keep the original slot, b gets the new one
        if let SplitNode::Split {
            a: sa, b: sb, ..
        } = &tree.nodes[0]
        {
            assert_eq!(*sa, a);
            assert_eq!(*sb, b);
        } else {
            panic!("root should be a Split node after split_leaf");
        }
    }

    #[test]
    fn split_reassigns_slots() {
        let mut tree = SplitTree::new();
        tree.split_leaf(0, Direction::Horizontal);
        assert_slots_sequential(&tree);
    }

    #[test]
    fn close_focused_decrements_leaf_count() {
        let mut tree = SplitTree::new();
        let (_a, b) = tree.split_leaf(0, Direction::Horizontal);
        tree.focused = b;

        assert!(tree.close_focused());
        assert_eq!(tree.leaf_count, 1);
    }

    #[test]
    fn close_focused_reassigns_slots() {
        let mut tree = SplitTree::new();
        let (_a, b) = tree.split_leaf(0, Direction::Horizontal);
        tree.focused = b;
        tree.close_focused();

        let leaves = tree.collect_leaves(tree.root);
        assert_eq!(leaves.len(), 1);
        assert_slots_sequential(&tree);
    }

    #[test]
    fn close_single_pane_returns_false() {
        let mut tree = SplitTree::new();
        assert!(!tree.close_focused());
    }

    #[test]
    fn cycle_focus_wraps_around() {
        let mut tree = SplitTree::new();
        let (a, _b) = tree.split_leaf(0, Direction::Horizontal);
        tree.focused = a;

        let leaves = tree.collect_leaves(tree.root);
        assert_eq!(leaves.len(), 2);

        // Cycle through all leaves and back
        let start = tree.focused;
        let mut visited = vec![start];
        for _ in 0..leaves.len() {
            tree.cycle_focus();
            visited.push(tree.focused);
        }

        // Should visit all leaves and wrap back to start
        assert_eq!(visited.last(), Some(&start));
        // The intermediate visits should cover all leaves
        let unique: std::collections::HashSet<_> = visited.iter().collect();
        assert_eq!(unique.len(), leaves.len());
    }

    #[test]
    fn collect_leaves_correct_order() {
        let mut tree = SplitTree::new();
        // Split root into two
        let (a, _b) = tree.split_leaf(0, Direction::Horizontal);
        // Split left child into two more
        tree.split_leaf(a, Direction::Vertical);

        let leaves = tree.collect_leaves(tree.root);
        assert_eq!(leaves.len(), 3);
        assert_eq!(tree.leaf_count, 3);
        assert_slots_sequential(&tree);
    }

    #[test]
    fn scroll_y_updates_and_clamps() {
        let mut tree = SplitTree::new();

        tree.scroll_y(5, 100);
        if let SplitNode::Leaf { scroll_y, .. } = &tree.nodes[tree.focused] {
            assert_eq!(*scroll_y, 5);
        }

        // Clamp at max
        tree.scroll_y(200, 100);
        if let SplitNode::Leaf { scroll_y, .. } = &tree.nodes[tree.focused] {
            assert_eq!(*scroll_y, 100);
        }

        // Clamp at zero (negative)
        tree.scroll_y(-500, 100);
        if let SplitNode::Leaf { scroll_y, .. } = &tree.nodes[tree.focused] {
            assert_eq!(*scroll_y, 0);
        }
    }

    #[test]
    fn scroll_x_updates_and_clamps_at_zero() {
        let mut tree = SplitTree::new();

        tree.scroll_x(10);
        if let SplitNode::Leaf { scroll_x, .. } = &tree.nodes[tree.focused] {
            assert_eq!(*scroll_x, 10);
        }

        // Clamp at zero
        tree.scroll_x(-500);
        if let SplitNode::Leaf { scroll_x, .. } = &tree.nodes[tree.focused] {
            assert_eq!(*scroll_x, 0);
        }
    }

    #[test]
    fn reset_all_scroll_clears_leaves() {
        let mut tree = SplitTree::new();
        tree.split_leaf(0, Direction::Horizontal);

        // Scroll all leaves
        let leaves = tree.collect_leaves(tree.root);
        for &leaf_id in &leaves {
            tree.focused = leaf_id;
            tree.scroll_y(10, 100);
            tree.scroll_x(5);
        }

        tree.reset_all_scroll();

        for &leaf_id in &tree.collect_leaves(tree.root) {
            if let SplitNode::Leaf { scroll_y, scroll_x, .. } = &tree.nodes[leaf_id] {
                assert_eq!(*scroll_y, 0);
                assert_eq!(*scroll_x, 0);
            }
        }
    }
}
