use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
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

pub fn render_footer(f: &mut Frame, area: Rect) {
    let keys = Line::from(vec![
        Span::styled(" A ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
        Span::styled(" prev ", Style::default().fg(Color::DarkGray)),
        Span::styled(" D ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
        Span::styled(" next ", Style::default().fg(Color::DarkGray)),
        Span::styled(" S ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
        Span::styled(" split ", Style::default().fg(Color::DarkGray)),
        Span::styled(" ⇧S ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
        Span::styled(" auto split ", Style::default().fg(Color::DarkGray)),
        Span::styled(" v ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
        Span::styled(" vsplit ", Style::default().fg(Color::DarkGray)),
        Span::styled(" h ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
        Span::styled(" hsplit ", Style::default().fg(Color::DarkGray)),
        Span::styled(" Tab/0-9 ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
        Span::styled(" focus ", Style::default().fg(Color::DarkGray)),
        Span::styled(" W ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
        Span::styled(" close ", Style::default().fg(Color::DarkGray)),
        Span::styled(" jkl ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
        Span::styled(" navigate ", Style::default().fg(Color::DarkGray)),
        Span::styled(" ←↑↓→ ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
        Span::styled(" scroll ", Style::default().fg(Color::DarkGray)),
        Span::styled(" q ", Style::default().fg(Color::Black).bg(Color::DarkGray)),
        Span::styled(" quit ", Style::default().fg(Color::DarkGray)),
    ]);

    let paragraph = Paragraph::new(keys);
    f.render_widget(paragraph, area);
}
