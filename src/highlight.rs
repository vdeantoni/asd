use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};
use similar::{ChangeTag, TextDiff};
use syntect::{
    easy::HighlightLines,
    highlighting::{FontStyle, ThemeSet},
    parsing::SyntaxSet,
};

use crate::diff::{DiffLine, FileDiff, LineKind};

const EMPHASIS_ADD_FG: Color = Color::Rgb(180, 255, 180);
const EMPHASIS_ADD_BG: Color = Color::Rgb(0, 80, 0);
const EMPHASIS_DEL_FG: Color = Color::Rgb(255, 180, 180);
const EMPHASIS_DEL_BG: Color = Color::Rgb(80, 0, 0);

pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl Highlighter {
    pub fn new() -> Self {
        Self {
            syntax_set: two_face::syntax::extra_newlines(),
            theme_set: ThemeSet::load_defaults(),
        }
    }

    pub fn highlight_file(&self, file: &mut FileDiff) {
        let syntax = self
            .syntax_set
            .find_syntax_for_file(&file.filename)
            .ok()
            .flatten()
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let theme = &self.theme_set.themes["base16-ocean.dark"];
        let mut h = HighlightLines::new(syntax, theme);

        let lines = &file.lines;
        let mut styled: Vec<Line<'static>> = Vec::with_capacity(lines.len());
        let mut i = 0;

        while i < lines.len() {
            // Detect consecutive remove→add pairs for intra-line diff
            let remove_start = i;
            while i < lines.len() && lines[i].kind == LineKind::Remove {
                i += 1;
            }
            let remove_end = i;

            let add_start = i;
            while i < lines.len() && lines[i].kind == LineKind::Add {
                i += 1;
            }
            let add_end = i;

            let n_removes = remove_end - remove_start;
            let n_adds = add_end - add_start;

            if n_removes > 0 && n_adds > 0 {
                let pairs = n_removes.min(n_adds);

                // Collect paired styled lines separately to maintain order
                let mut rem_lines: Vec<Line<'static>> = Vec::with_capacity(n_removes);
                let mut add_lines: Vec<Line<'static>> = Vec::with_capacity(n_adds);

                for j in 0..pairs {
                    let (rl, al) = self.style_pair(
                        &lines[remove_start + j],
                        &lines[add_start + j],
                        &mut h,
                    );
                    rem_lines.push(rl);
                    add_lines.push(al);
                }
                for j in pairs..n_removes {
                    rem_lines.push(self.style_line(&lines[remove_start + j], &mut h, &[]));
                }
                for j in pairs..n_adds {
                    add_lines.push(self.style_line(&lines[add_start + j], &mut h, &[]));
                }

                // Push in original order: all removes then all adds
                styled.extend(rem_lines);
                styled.extend(add_lines);
            } else {
                for j in remove_start..remove_end {
                    styled.push(self.style_line(&lines[j], &mut h, &[]));
                }
                for j in add_start..add_end {
                    styled.push(self.style_line(&lines[j], &mut h, &[]));
                }
            }

            // Non-remove, non-add line (context, hunk header)
            if i == remove_start {
                styled.push(self.style_line(&lines[i], &mut h, &[]));
                i += 1;
            }
        }

        file.styled_lines = styled;
    }

    fn style_pair(
        &self,
        rem: &DiffLine,
        add: &DiffLine,
        h: &mut HighlightLines,
    ) -> (Line<'static>, Line<'static>) {
        let diff = TextDiff::from_words(&rem.content, &add.content);
        let rem_emphasis = build_emphasis_mask(&rem.content, &diff, true);
        let add_emphasis = build_emphasis_mask(&add.content, &diff, false);

        let rem_line = self.style_line(rem, h, &rem_emphasis);
        let add_line = self.style_line(add, h, &add_emphasis);
        (rem_line, add_line)
    }

    fn style_line(
        &self,
        dl: &DiffLine,
        h: &mut HighlightLines,
        emphasis: &[bool],
    ) -> Line<'static> {
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

        let (base_fg, emph_fg, emph_bg) = match dl.kind {
            LineKind::Add => (Color::Green, EMPHASIS_ADD_FG, EMPHASIS_ADD_BG),
            LineKind::Remove => (Color::Red, EMPHASIS_DEL_FG, EMPHASIS_DEL_BG),
            _ => (Color::White, Color::White, Color::Reset),
        };

        if emphasis.is_empty() || dl.kind == LineKind::Context {
            // No intra-line diff — use syntect coloring
            let line_for_highlight = format!("{}\n", &dl.content);
            let regions = h
                .highlight_line(&line_for_highlight, &self.syntax_set)
                .unwrap_or_default();

            let is_diff_line = dl.kind == LineKind::Add || dl.kind == LineKind::Remove;

            for (style, text) in regions {
                let text = text.trim_end_matches('\n').to_string();
                if text.is_empty() {
                    continue;
                }
                let fg = if is_diff_line {
                    base_fg
                } else {
                    Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b)
                };
                let mut s = Style::default().fg(fg);
                if style.font_style.contains(FontStyle::BOLD) {
                    s = s.add_modifier(Modifier::BOLD);
                }
                if style.font_style.contains(FontStyle::ITALIC) {
                    s = s.add_modifier(Modifier::ITALIC);
                }
                spans.push(Span::styled(text, s));
            }
        } else {
            // Intra-line diff — feed syntect for state tracking, render with emphasis
            let _ = h.highlight_line(&format!("{}\n", &dl.content), &self.syntax_set);

            let chars: Vec<char> = dl.content.chars().collect();
            let mut ci = 0;

            while ci < chars.len() {
                let is_emph = emphasis.get(ci).copied().unwrap_or(false);
                let start = ci;
                while ci < chars.len()
                    && emphasis.get(ci).copied().unwrap_or(false) == is_emph
                {
                    ci += 1;
                }
                let chunk: String = chars[start..ci].iter().collect();
                let style = if is_emph {
                    Style::default()
                        .fg(emph_fg)
                        .bg(emph_bg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(base_fg)
                };
                spans.push(Span::styled(chunk, style));
            }
        }

        Line::from(spans)
    }
}

/// Build a per-character boolean mask: true = this character was changed.
pub(crate) fn build_emphasis_mask<'a>(
    content: &'a str,
    diff: &TextDiff<'a, 'a, 'a, str>,
    is_old: bool,
) -> Vec<bool> {
    let mut mask = vec![false; content.len()];
    let mut byte_pos = 0;

    for change in diff.iter_all_changes() {
        let value = change.value();
        let dominated = match change.tag() {
            ChangeTag::Equal => {
                byte_pos += value.len();
                continue;
            }
            ChangeTag::Delete => is_old,
            ChangeTag::Insert => !is_old,
        };

        if dominated {
            let end = (byte_pos + value.len()).min(mask.len());
            for b in &mut mask[byte_pos..end] {
                *b = true;
            }
            byte_pos += value.len();
        }
    }

    // Convert byte mask to char mask
    content
        .char_indices()
        .map(|(byte_idx, _)| mask.get(byte_idx).copied().unwrap_or(false))
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn mask(old: &str, new: &str, is_old: bool) -> Vec<bool> {
        let diff = TextDiff::from_words(old, new);
        build_emphasis_mask(if is_old { old } else { new }, &diff, is_old)
    }

    #[test]
    fn equal_strings_all_false() {
        let m = mask("hello world", "hello world", true);
        assert!(m.iter().all(|&b| !b));

        let m = mask("hello world", "hello world", false);
        assert!(m.iter().all(|&b| !b));
    }

    #[test]
    fn single_word_change() {
        // "hello world" → "hello rust"
        // On the old side, "world" should be marked
        let old = "hello world";
        let new = "hello rust";

        let old_mask = mask(old, new, true);
        // "hello " = indices 0..5 should be false, "world" = 6..10 should be true
        assert!(!old_mask[0]); // 'h'
        assert!(!old_mask[5]); // ' '
        assert!(old_mask[6]); // 'w'
        assert!(old_mask[10]); // 'd'

        let new_mask = mask(old, new, false);
        assert!(!new_mask[0]); // 'h'
        assert!(!new_mask[5]); // ' '
        assert!(new_mask[6]); // 'r'
        assert!(new_mask[9]); // 't'
    }

    #[test]
    fn insertion_marks_new_side() {
        // Word-level diff: "a b" → "a inserted b"
        // "a" and "b" are equal, "inserted " is inserted
        let old = "a b";
        let new = "a inserted b";

        let old_mask = mask(old, new, true);
        // Old side: nothing was deleted, all false
        assert!(old_mask.iter().all(|&b| !b));

        let new_mask = mask(old, new, false);
        // "a " unchanged, "inserted " marked, "b" unchanged
        assert!(!new_mask[0]); // 'a'
        assert!(new_mask[2]); // 'i' in "inserted"
        assert!(new_mask[9]); // 'd' in "inserted"
        assert!(!new_mask[11]); // 'b'
    }

    #[test]
    fn deletion_marks_old_side() {
        // Word-level diff: "a deleted b" → "a b"
        let old = "a deleted b";
        let new = "a b";

        let old_mask = mask(old, new, true);
        // "a " unchanged, "deleted " marked, "b" unchanged
        assert!(!old_mask[0]); // 'a'
        assert!(old_mask[2]); // 'd' in "deleted"
        assert!(old_mask[8]); // ' ' after "deleted"
        assert!(!old_mask[10]); // 'b'

        let new_mask = mask(old, new, false);
        // New side: nothing was inserted, all false
        assert!(new_mask.iter().all(|&b| !b));
    }

    #[test]
    fn multi_word_changes() {
        let old = "the quick brown fox";
        let new = "the slow brown dog";

        let old_mask = mask(old, new, true);
        let new_mask = mask(old, new, false);

        // "the " and " brown " should not be emphasized
        assert!(!old_mask[0]); // 't'
        assert!(!old_mask[1]); // 'h'
        assert!(!old_mask[2]); // 'e'

        // "quick" (indices 4..8) should be emphasized on old side
        assert!(old_mask[4]); // 'q'

        // "fox" (indices 16..18) should be emphasized on old side
        assert!(old_mask[16]); // 'f'

        // "slow" should be emphasized on new side
        assert!(new_mask[4]); // 's'

        // "dog" should be emphasized on new side
        assert!(new_mask[16]); // 'd'
    }
}
