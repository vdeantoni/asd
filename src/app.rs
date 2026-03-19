use std::collections::VecDeque;
use std::fs::File;
use std::io::Read;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::diff::FileDiff;
use crate::layout::{NodeId, SplitNode, SplitTree};
use crate::ui;

const MIN_PANE_WIDTH: u16 = 20;
const MIN_PANE_HEIGHT: u16 = 5;
const MIN_SPLIT_DIM: u16 = 50;

enum Dir {
    Up,
    Down,
    Right,
}

pub struct App {
    pub files: Vec<FileDiff>,
    pub tree: SplitTree,
    pub window_start: usize,
    pub should_quit: bool,
    tty: File,
    /// BFS queue of leaf NodeIds to split next. Front = oldest unsplit pane.
    split_queue: VecDeque<NodeId>,
    /// Cached leaf rects from last render, keyed by NodeId.
    leaf_rects: Vec<(NodeId, Rect)>,
    /// Counter for assigning pane indices during render.
    pane_counter: usize,
}

impl App {
    pub fn new(files: Vec<FileDiff>, tty: File) -> Self {
        Self {
            files,
            tree: SplitTree::new(),
            window_start: 0,
            should_quit: false,
            tty,
            split_queue: VecDeque::from([0]), // root leaf
            leaf_rects: Vec::new(),
            pane_counter: 0,
        }
    }

    pub fn handle_event(&mut self) -> color_eyre::Result<()> {
        let mut buf = [0u8; 3];
        let n = self.tty.read(&mut buf)?;
        if n == 0 {
            return Ok(());
        }

        if buf[0] == 27 && n >= 3 && buf[1] == b'[' {
            match buf[2] {
                b'A' => self.scroll_up(),
                b'B' => self.scroll_down(),
                b'C' => self.scroll_right(),
                b'D' => self.scroll_left(),
                _ => {}
            }
            return Ok(());
        }

        match buf[0] {
            b'q' | 27 => self.should_quit = true,
            b'a' => self.navigate_prev(),
            b'd' => self.navigate_next(),
            b's' | b' ' => self.try_split(),
            b'S' => self.split_focused(None),
            b'v' => self.split_focused(Some(Direction::Horizontal)), // vertical line → left|right
            b'h' => self.split_focused(Some(Direction::Vertical)),   // horizontal line → top/bottom
            b'\t' => self.tree.cycle_focus(),
            b'w' => self.close_pane(),
            b'j' => self.focus_direction(Dir::Down),
            b'k' => self.focus_direction(Dir::Up),
            b'l' => self.focus_direction(Dir::Right),
            b'0'..=b'9' => self.focus_by_index((buf[0] - b'0') as usize),
            _ => {}
        }

        Ok(())
    }

    fn navigate_prev(&mut self) {
        if self.window_start > 0 {
            self.window_start -= 1;
            self.tree.reset_all_scroll();
        }
    }

    fn navigate_next(&mut self) {
        let max_start = self.files.len().saturating_sub(self.tree.leaf_count);
        if self.window_start < max_start {
            self.window_start += 1;
            self.tree.reset_all_scroll();
        }
    }

    fn scroll_down(&mut self) {
        let max = self.focused_file_line_count().saturating_sub(1) as u16;
        self.tree.scroll_y(1, max);
    }

    fn scroll_up(&mut self) {
        let max = self.focused_file_line_count().saturating_sub(1) as u16;
        self.tree.scroll_y(-1, max);
    }

    fn scroll_right(&mut self) {
        self.tree.scroll_x(4);
    }

    fn scroll_left(&mut self) {
        self.tree.scroll_x(-4);
    }

    fn focused_file_line_count(&self) -> usize {
        if let SplitNode::Leaf { slot, .. } = &self.tree.nodes[self.tree.focused] {
            let file_index = self.window_start + slot;
            if file_index < self.files.len() {
                return self.files[file_index].styled_lines.len();
            }
        }
        0
    }

    fn try_split(&mut self) {
        let needed = self.tree.leaf_count + 1;
        if self.window_start + needed > self.files.len() {
            return; // not enough files, noop
        }

        // BFS: try the front of the queue. If stale or too small, rotate.
        let max_tries = self.split_queue.len();
        for _ in 0..max_tries {
            let Some(&target_id) = self.split_queue.front() else {
                break;
            };

            // Skip if no longer a leaf (was already split or removed)
            if !matches!(self.tree.nodes.get(target_id), Some(SplitNode::Leaf { .. })) {
                self.split_queue.pop_front();
                continue;
            }

            // Check if this pane is big enough
            let rect = self.rect_for(target_id);
            if rect.width < MIN_SPLIT_DIM || rect.height < MIN_SPLIT_DIM {
                // Too small — rotate to back, try next
                self.split_queue.pop_front();
                self.split_queue.push_back(target_id);
                continue;
            }

            // Split it!
            self.split_queue.pop_front();

            let direction = split_direction(rect);
            let (child_a, child_b) = self.tree.split_leaf(target_id, direction);

            self.split_queue.push_back(child_a);
            self.split_queue.push_back(child_b);

            // Focus stays where it was — split is focus-agnostic.
            // But if the focused node was the one split, keep focus on child_a (same content).
            if self.tree.focused == target_id {
                self.tree.focused = child_a;
            }

            return;
        }

        // All panes too small — noop
    }

    /// Split the focused pane. If `dir` is None, auto-detect from aspect ratio.
    fn split_focused(&mut self, dir: Option<Direction>) {
        let needed = self.tree.leaf_count + 1;
        if self.window_start + needed > self.files.len() {
            return;
        }

        let target_id = self.tree.focused;
        if !matches!(self.tree.nodes.get(target_id), Some(SplitNode::Leaf { .. })) {
            return;
        }

        let rect = self.rect_for(target_id);
        if rect.width < MIN_SPLIT_DIM || rect.height < MIN_SPLIT_DIM {
            return;
        }

        let direction = dir.unwrap_or_else(|| split_direction(rect));
        let (child_a, child_b) = self.tree.split_leaf(target_id, direction);

        self.split_queue.retain(|&id| id != target_id);
        self.split_queue.push_back(child_a);
        self.split_queue.push_back(child_b);

        self.tree.focused = child_a;
    }

    fn close_pane(&mut self) {
        self.tree.close_focused();
        // Rebuild queue from current leaves in traversal order
        let leaves = self.tree.collect_leaves(self.tree.root);
        self.split_queue = VecDeque::from(leaves);
    }

    fn focus_by_index(&mut self, index: usize) {
        let leaves = self.tree.collect_leaves(self.tree.root);
        if let Some(&node_id) = leaves.get(index) {
            self.tree.focused = node_id;
        }
    }

    fn focus_direction(&mut self, dir: Dir) {
        let focused_rect = match self.rect_for(self.tree.focused) {
            r if r.width == 0 => return,
            r => r,
        };

        let fc_x = focused_rect.x as i32 + focused_rect.width as i32 / 2;
        let fc_y = focused_rect.y as i32 + focused_rect.height as i32 / 2;

        let mut best: Option<(NodeId, i32)> = None;

        for &(node_id, rect) in &self.leaf_rects {
            if node_id == self.tree.focused {
                continue;
            }

            let cx = rect.x as i32 + rect.width as i32 / 2;
            let cy = rect.y as i32 + rect.height as i32 / 2;
            let dx = cx - fc_x;
            let dy = cy - fc_y;

            // Check if candidate is in the correct direction
            let in_dir = match dir {
                Dir::Right => dx > 0,
                Dir::Up => dy < 0,
                Dir::Down => dy > 0,
            };
            if !in_dir {
                continue;
            }

            // Distance: weight perpendicular axis more so we prefer
            // panes that are aligned on the primary axis
            let cost = match dir {
                Dir::Right => dx.abs() + dy.abs() * 3,
                Dir::Up | Dir::Down => dy.abs() + dx.abs() * 3,
            };

            if best.is_none() || cost < best.unwrap().1 {
                best = Some((node_id, cost));
            }
        }

        if let Some((node_id, _)) = best {
            self.tree.focused = node_id;
        }
    }

    fn rect_for(&self, node_id: NodeId) -> Rect {
        self.leaf_rects
            .iter()
            .find(|(id, _)| *id == node_id)
            .map(|(_, r)| *r)
            .unwrap_or_default()
    }

    pub fn draw(&mut self, f: &mut Frame) {
        let size = f.area();
        if size.height < 2 {
            return;
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(size);

        self.leaf_rects.clear();
        self.pane_counter = 0;
        self.render_node(f, self.tree.root, chunks[0]);
        ui::render_footer(f, chunks[1]);
    }

    fn render_node(&mut self, f: &mut Frame, node_id: NodeId, area: Rect) {
        match &self.tree.nodes[node_id] {
            SplitNode::Leaf { slot, scroll_y, scroll_x } => {
                let slot = *slot;
                let scroll_y = *scroll_y;
                let scroll_x = *scroll_x;
                let file_index = self.window_start + slot;
                let pane_index = self.pane_counter;
                self.pane_counter += 1;

                self.leaf_rects.push((node_id, area));

                if file_index < self.files.len() {
                    let file = &self.files[file_index];
                    let is_focused = node_id == self.tree.focused;
                    ui::render_pane(
                        f,
                        area,
                        file,
                        file_index,
                        self.files.len(),
                        is_focused,
                        scroll_y,
                        scroll_x,
                        pane_index,
                    );
                }
            }
            SplitNode::Split { direction, a, b } => {
                let direction = *direction;
                let a = *a;
                let b = *b;

                if !can_split_area(area, direction) {
                    self.render_node(f, a, area);
                    return;
                }

                let chunks = Layout::default()
                    .direction(direction)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(area);

                self.render_node(f, a, chunks[0]);
                self.render_node(f, b, chunks[1]);
            }
        }
    }
}

/// Determine split direction based on visual aspect ratio.
/// Accounts for terminal cell aspect ratio (~2:1 height:width).
fn split_direction(rect: Rect) -> Direction {
    let visual_height = rect.height as u32 * 2;
    let visual_width = rect.width as u32;
    if visual_height >= visual_width {
        Direction::Vertical   // horizontal cut → top/bottom
    } else {
        Direction::Horizontal // vertical cut → left|right
    }
}

fn can_split_area(area: Rect, direction: Direction) -> bool {
    match direction {
        Direction::Horizontal => area.width >= MIN_PANE_WIDTH * 2,
        Direction::Vertical => area.height >= MIN_PANE_HEIGHT * 2,
    }
}
