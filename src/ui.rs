use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use crate::diff::FileDiff;

pub fn render_pane(
    f: &mut Frame,
    area: Rect,
    file: &FileDiff,
    file_index: usize,
    total_files: usize,
    focused: bool,
    scroll_y: u16,
    scroll_x: u16,
    pane_index: usize,
) {
    let border_style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title_left = format!(
        " {} {{+{} -{}}} ({}/{}) ",
        file.filename,
        file.additions,
        file.deletions,
        file_index + 1,
        total_files
    );

    let title_right = format!(" [{}] ", pane_index);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(title_left)
        .title_bottom(Line::from(title_right).right_aligned());

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let paragraph = Paragraph::new(file.styled_lines.clone()).scroll((scroll_y, scroll_x));
    f.render_widget(paragraph, inner);
}

pub fn render_file_list(
    f: &mut Frame,
    area: Rect,
    files: &[FileDiff],
    cursor: usize,
) {
    let hidden_count = files.iter().filter(|f| f.hidden).count();

    // Center the overlay: 70% width, up to files.len()+4 lines tall
    let width = (area.width * 70 / 100).max(40).min(area.width.saturating_sub(4));
    let height = (files.len() as u16 + 4).min(area.height.saturating_sub(4));
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    let overlay = Rect::new(x, y, width, height);

    let title = format!(" Files ({} hidden) ", hidden_count);
    let footer_text = " x:toggle  ⇧↑↓:reorder  Enter:go  f:close ";

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .title(title)
        .title_bottom(Line::from(footer_text).centered());

    let inner = block.inner(overlay);

    // Clear the area behind the overlay
    f.render_widget(Clear, overlay);
    f.render_widget(block, overlay);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    // Build file list lines
    let inner_width = inner.width as usize;
    let lines: Vec<Line> = files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let is_cursor = i == cursor;
            let marker = if is_cursor { "▸" } else { " " };
            let num = format!("{:>2}.", i + 1);
            let stats = format!("{{+{} -{}}}", file.additions, file.deletions);
            let hidden_label = if file.hidden { " hidden" } else { "" };

            // Calculate padding
            let content_len = marker.len() + 1 + num.len() + 1 + file.filename.len() + 1 + stats.len() + hidden_label.len();
            let padding = if inner_width > content_len {
                " ".repeat(inner_width - content_len)
            } else {
                String::new()
            };

            let (fg, bg) = if is_cursor {
                (Color::Black, Color::Yellow)
            } else if file.hidden {
                (Color::DarkGray, Color::Reset)
            } else {
                (Color::White, Color::Reset)
            };

            let style = Style::default().fg(fg).bg(bg);
            let hidden_style = if is_cursor {
                style
            } else {
                Style::default().fg(Color::Red).add_modifier(Modifier::DIM)
            };

            Line::from(vec![
                Span::styled(format!("{} ", marker), style),
                Span::styled(format!("{} ", num), style),
                Span::styled(file.filename.clone(), style),
                Span::styled(format!(" {}", stats), style),
                Span::styled(padding, style),
                Span::styled(hidden_label.to_string(), hidden_style),
            ])
        })
        .collect();

    // Scroll the list if cursor is beyond visible area
    let scroll = if cursor >= inner.height as usize {
        (cursor - inner.height as usize + 1) as u16
    } else {
        0
    };

    let paragraph = Paragraph::new(lines).scroll((scroll, 0));
    f.render_widget(paragraph, inner);
}

pub fn render_footer(f: &mut Frame, area: Rect, in_file_list: bool) {
    let keys = if in_file_list {
        Line::from(vec![
            Span::styled(" ↑↓ ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
            Span::styled(" navigate ", Style::default().fg(Color::DarkGray)),
            Span::styled(" x ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
            Span::styled(" toggle hide ", Style::default().fg(Color::DarkGray)),
            Span::styled(" ⇧↑↓ ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
            Span::styled(" reorder ", Style::default().fg(Color::DarkGray)),
            Span::styled(" Enter ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
            Span::styled(" go to file ", Style::default().fg(Color::DarkGray)),
            Span::styled(" f ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
            Span::styled(" close ", Style::default().fg(Color::DarkGray)),
        ])
    } else {
        Line::from(vec![
            Span::styled(" A ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
            Span::styled(" prev ", Style::default().fg(Color::DarkGray)),
            Span::styled(" D ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
            Span::styled(" next ", Style::default().fg(Color::DarkGray)),
            Span::styled(" S ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
            Span::styled(" split ", Style::default().fg(Color::DarkGray)),
            Span::styled(" v/h ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
            Span::styled(" v/hsplit ", Style::default().fg(Color::DarkGray)),
            Span::styled(" Space ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
            Span::styled(" pgdn ", Style::default().fg(Color::DarkGray)),
            Span::styled(" x ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
            Span::styled(" hide ", Style::default().fg(Color::DarkGray)),
            Span::styled(" f ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
            Span::styled(" files ", Style::default().fg(Color::DarkGray)),
            Span::styled(" Tab/0-9 ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
            Span::styled(" focus ", Style::default().fg(Color::DarkGray)),
            Span::styled(" m/M ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
            Span::styled(" merge ", Style::default().fg(Color::DarkGray)),
            Span::styled(" -/= ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
            Span::styled(" resize ", Style::default().fg(Color::DarkGray)),
            Span::styled(" c ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
            Span::styled(" copy ", Style::default().fg(Color::DarkGray)),
            Span::styled(" o ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
            Span::styled(" editor ", Style::default().fg(Color::DarkGray)),
            Span::styled(" ←↑↓→ ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
            Span::styled(" navigate ", Style::default().fg(Color::DarkGray)),
            Span::styled(" ⇧←↑↓→ ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
            Span::styled(" scroll ", Style::default().fg(Color::DarkGray)),
            Span::styled(" r ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
            Span::styled(" reset ", Style::default().fg(Color::DarkGray)),
            Span::styled(" q ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
            Span::styled(" quit ", Style::default().fg(Color::DarkGray)),
        ])
    };

    let paragraph = Paragraph::new(keys);
    f.render_widget(paragraph, area);
}
