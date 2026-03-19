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
    Left,
    Right,
}

pub struct App {
    pub files: Vec<FileDiff>,
    pub tree: SplitTree,
    pub window_start: usize,
    pub should_quit: bool,
    tty: File,
    split_queue: VecDeque<NodeId>,
    leaf_rects: Vec<(NodeId, Rect)>,
    pane_counter: usize,
    pub show_file_list: bool,
    pub file_list_cursor: usize,
}

impl App {
    pub fn new(files: Vec<FileDiff>, tty: File) -> Self {
        Self {
            files,
            tree: SplitTree::new(),
            window_start: 0,
            should_quit: false,
            tty,
            split_queue: VecDeque::from([0]),
            leaf_rects: Vec::new(),
            pane_counter: 0,
            show_file_list: false,
            file_list_cursor: 0,
        }
    }

    /// Indices into `files` Vec for non-hidden files, in order.
    fn visible_indices(&self) -> Vec<usize> {
        self.files
            .iter()
            .enumerate()
            .filter(|(_, f)| !f.hidden)
            .map(|(i, _)| i)
            .collect()
    }

    /// Resolve a slot (position in visible list) to an index in `files`.
    fn file_index_for_slot(&self, slot: usize) -> Option<usize> {
        let vis = self.visible_indices();
        let idx = self.window_start + slot;
        vis.get(idx).copied()
    }

    fn visible_count(&self) -> usize {
        self.files.iter().filter(|f| !f.hidden).count()
    }

    pub fn handle_event(&mut self) -> color_eyre::Result<()> {
        let mut buf = [0u8; 6];
        let n = self.tty.read(&mut buf)?;
        if n == 0 {
            return Ok(());
        }

        if self.show_file_list {
            return self.handle_file_list_event(buf, n);
        }

        // Escape sequences
        if buf[0] == 27 && n >= 3 && buf[1] == b'[' {
            if n >= 6 && buf[2] == b'1' && buf[3] == b';' && buf[4] == b'2' {
                match buf[5] {
                    b'A' => self.scroll_up(),
                    b'B' => self.scroll_down(),
                    b'C' => self.scroll_right(),
                    b'D' => self.scroll_left(),
                    _ => {}
                }
                return Ok(());
            }
            match buf[2] {
                b'A' => self.focus_direction(Dir::Up),
                b'B' => self.focus_direction(Dir::Down),
                b'C' => self.focus_direction(Dir::Right),
                b'D' => self.focus_direction(Dir::Left),
                _ => {}
            }
            return Ok(());
        }

        match buf[0] {
            b'q' | 27 | 3 => self.should_quit = true,
            b'a' => self.navigate_prev(),
            b'd' => self.navigate_next(),
            b's' => self.try_split(),
            b'S' => self.split_focused(None),
            b'v' => self.split_focused(Some(Direction::Horizontal)),
            b'h' => self.split_focused(Some(Direction::Vertical)),
            b'\t' => self.tree.cycle_focus(),
            b'w' => self.close_pane(),
            b' ' => self.hide_focused_file(),
            b'r' => self.reset(),
            b'f' => {
                self.show_file_list = true;
                self.file_list_cursor = 0;
            }
            b'0'..=b'9' => self.focus_by_index((buf[0] - b'0') as usize),
            _ => {}
        }

        Ok(())
    }

    fn handle_file_list_event(&mut self, buf: [u8; 6], n: usize) -> color_eyre::Result<()> {
        if buf[0] == 27 && n >= 3 && buf[1] == b'[' {
            // Shift+arrow: ESC [ 1 ; 2 A/B (reorder)
            if n >= 6 && buf[2] == b'1' && buf[3] == b';' && buf[4] == b'2' {
                match buf[5] {
                    b'A' => {
                        // Shift+Up: move file up
                        if self.file_list_cursor > 0 {
                            self.files.swap(self.file_list_cursor, self.file_list_cursor - 1);
                            self.file_list_cursor -= 1;
                        }
                    }
                    b'B' => {
                        // Shift+Down: move file down
                        if self.file_list_cursor + 1 < self.files.len() {
                            self.files.swap(self.file_list_cursor, self.file_list_cursor + 1);
                            self.file_list_cursor += 1;
                        }
                    }
                    _ => {}
                }
                return Ok(());
            }
            // Plain arrow: navigate cursor
            match buf[2] {
                b'A' => {
                    if self.file_list_cursor > 0 {
                        self.file_list_cursor -= 1;
                    }
                }
                b'B' => {
                    if self.file_list_cursor + 1 < self.files.len() {
                        self.file_list_cursor += 1;
                    }
                }
                _ => {}
            }
            return Ok(());
        }

        match buf[0] {
            b'f' | 27 => self.show_file_list = false,
            b' ' => {
                // Toggle hidden
                if self.file_list_cursor < self.files.len() {
                    let was_hidden = self.files[self.file_list_cursor].hidden;
                    self.files[self.file_list_cursor].hidden = !was_hidden;
                    self.clamp_window();
                }
            }
            b'\r' | b'\n' => {
                // Enter: close and navigate to selected file
                self.show_file_list = false;
                self.navigate_to_file(self.file_list_cursor);
            }
            b'q' | 3 => self.should_quit = true,
            _ => {}
        }

        Ok(())
    }

    /// Navigate so the file at `files_idx` is shown in the focused pane.
    fn navigate_to_file(&mut self, files_idx: usize) {
        if files_idx >= self.files.len() || self.files[files_idx].hidden {
            return;
        }
        let vis = self.visible_indices();
        if let Some(vis_pos) = vis.iter().position(|&i| i == files_idx) {
            // Set window_start so this file appears in slot 0
            let max_start = vis.len().saturating_sub(self.tree.leaf_count);
            self.window_start = vis_pos.min(max_start);
            self.tree.reset_all_scroll();
        }
    }

    fn hide_focused_file(&mut self) {
        if let SplitNode::Leaf { slot, .. } = &self.tree.nodes[self.tree.focused] {
            if let Some(file_index) = self.file_index_for_slot(*slot) {
                // Don't allow hiding the last visible file
                if self.visible_count() <= 1 {
                    return;
                }
                self.files[file_index].hidden = true;
                self.clamp_window();
            }
        }
    }

    /// Ensure window_start is valid after hiding files or reordering.
    fn clamp_window(&mut self) {
        let vis_count = self.visible_count();
        if vis_count == 0 {
            return;
        }
        let max_start = vis_count.saturating_sub(self.tree.leaf_count);
        if self.window_start > max_start {
            self.window_start = max_start;
        }
    }

    fn reset(&mut self) {
        for file in &mut self.files {
            file.hidden = false;
        }
        self.window_start = 0;
        self.tree = SplitTree::new();
        self.split_queue = VecDeque::from([self.tree.root]);
        self.show_file_list = false;
        self.file_list_cursor = 0;
    }

    fn navigate_prev(&mut self) {
        if self.window_start > 0 {
            self.window_start -= 1;
            self.tree.reset_all_scroll();
        }
    }

    fn navigate_next(&mut self) {
        let max_start = self.visible_count().saturating_sub(self.tree.leaf_count);
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
            if let Some(file_index) = self.file_index_for_slot(*slot) {
                return self.files[file_index].styled_lines.len();
            }
        }
        0
    }

    fn try_split(&mut self) {
        let needed = self.tree.leaf_count + 1;
        if self.window_start + needed > self.visible_count() {
            return;
        }

        let max_tries = self.split_queue.len();
        for _ in 0..max_tries {
            let Some(&target_id) = self.split_queue.front() else {
                break;
            };

            if !matches!(self.tree.nodes.get(target_id), Some(SplitNode::Leaf { .. })) {
                self.split_queue.pop_front();
                continue;
            }

            let rect = self.rect_for(target_id);
            if rect.width < MIN_SPLIT_DIM || rect.height < MIN_SPLIT_DIM {
                self.split_queue.pop_front();
                self.split_queue.push_back(target_id);
                continue;
            }

            self.split_queue.pop_front();

            let direction = split_direction(rect);
            let (child_a, child_b) = self.tree.split_leaf(target_id, direction);

            self.split_queue.push_back(child_a);
            self.split_queue.push_back(child_b);

            if self.tree.focused == target_id {
                self.tree.focused = child_a;
            }

            return;
        }
    }

    fn split_focused(&mut self, dir: Option<Direction>) {
        let needed = self.tree.leaf_count + 1;
        if self.window_start + needed > self.visible_count() {
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

            let in_dir = match dir {
                Dir::Left => dx < 0,
                Dir::Right => dx > 0,
                Dir::Up => dy < 0,
                Dir::Down => dy > 0,
            };
            if !in_dir {
                continue;
            }

            let cost = match dir {
                Dir::Left | Dir::Right => dx.abs() + dy.abs() * 3,
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

        if self.show_file_list {
            ui::render_file_list(f, size, &self.files, self.file_list_cursor);
        }

        ui::render_footer(f, chunks[1], self.show_file_list);
    }

    fn render_node(&mut self, f: &mut Frame, node_id: NodeId, area: Rect) {
        match &self.tree.nodes[node_id] {
            SplitNode::Leaf { slot, scroll_y, scroll_x } => {
                let slot = *slot;
                let scroll_y = *scroll_y;
                let scroll_x = *scroll_x;
                let pane_index = self.pane_counter;
                self.pane_counter += 1;

                self.leaf_rects.push((node_id, area));

                let vis = self.visible_indices();
                let vis_idx = self.window_start + slot;
                if let Some(&file_index) = vis.get(vis_idx) {
                    let file = &self.files[file_index];
                    let is_focused = node_id == self.tree.focused;
                    let total_visible = vis.len();
                    ui::render_pane(
                        f,
                        area,
                        file,
                        vis_idx,
                        total_visible,
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

fn split_direction(rect: Rect) -> Direction {
    let visual_height = rect.height as u32 * 2;
    let visual_width = rect.width as u32;
    if visual_height >= visual_width {
        Direction::Vertical
    } else {
        Direction::Horizontal
    }
}

fn can_split_area(area: Rect, direction: Direction) -> bool {
    match direction {
        Direction::Horizontal => area.width >= MIN_PANE_WIDTH * 2,
        Direction::Vertical => area.height >= MIN_PANE_HEIGHT * 2,
    }
}
