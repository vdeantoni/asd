use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use syntect::{
    easy::HighlightLines,
    highlighting::{FontStyle, ThemeSet},
    parsing::SyntaxSet,
};

use crate::diff::{DiffLine, FileDiff, LineKind};

pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl Highlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: SyntaxSet::load_defaults_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    /// Pre-compute styled lines for a file diff using syntax highlighting.
    pub fn highlight_file(&self, file: &mut FileDiff) {
        let syntax = self
            .syntax_set
            .find_syntax_for_file(&file.filename)
            .ok()
            .flatten()
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let theme = &self.theme_set.themes["base16-ocean.dark"];
        let mut h = HighlightLines::new(syntax, theme);

        file.styled_lines = file
            .lines
            .iter()
            .map(|dl| self.style_line(dl, &mut h))
            .collect();
    }

    fn style_line(&self, dl: &DiffLine, h: &mut HighlightLines) -> Line<'static> {
        if dl.kind == LineKind::HunkHeader {
            return Line::from(Span::styled(
                dl.content.clone(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        let prefix = build_prefix(dl);
        let prefix_style = match dl.kind {
            LineKind::Add => Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            LineKind::Remove => Style::default()
                .fg(Color::Red)
                .add_modifier(Modifier::BOLD),
            _ => Style::default().fg(Color::DarkGray),
        };

        let mut spans = vec![Span::styled(prefix, prefix_style)];

        // Feed content to syntect for highlighting
        let line_for_highlight = format!("{}\n", &dl.content);
        let regions = h
            .highlight_line(&line_for_highlight, &self.syntax_set)
            .unwrap_or_default();

        let diff_fg = match dl.kind {
            LineKind::Add => Some(Color::Green),
            LineKind::Remove => Some(Color::Red),
            _ => None,
        };

        for (style, text) in regions {
            let text = text.trim_end_matches('\n').to_string();
            if text.is_empty() {
                continue;
            }

            let fg = if diff_fg.is_some() {
                // For add/remove lines, blend: use syntect color but dim it,
                // then overlay diff color for the main tone
                diff_fg.unwrap()
            } else {
                Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b)
            };

            let mut ratatui_style = Style::default().fg(fg);
            if style.font_style.contains(FontStyle::BOLD) {
                ratatui_style = ratatui_style.add_modifier(Modifier::BOLD);
            }
            if style.font_style.contains(FontStyle::ITALIC) {
                ratatui_style = ratatui_style.add_modifier(Modifier::ITALIC);
            }

            spans.push(Span::styled(text, ratatui_style));
        }

        Line::from(spans)
    }
}

fn build_prefix(dl: &DiffLine) -> String {
    let old = match dl.old_lineno {
        Some(n) => format!("{:>4}", n),
        None => "    ".to_string(),
    };
    let new = match dl.new_lineno {
        Some(n) => format!("{:>4}", n),
        None => "    ".to_string(),
    };
    let prefix_char = match dl.kind {
        LineKind::Add => "+",
        LineKind::Remove => "-",
        LineKind::Context => " ",
        LineKind::HunkHeader => "@",
    };
    format!("{} {} {} ", old, new, prefix_char)
}
