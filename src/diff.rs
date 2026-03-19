use ratatui::text::Line;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    Add,
    Remove,
    Context,
    HunkHeader,
}

pub struct DiffLine {
    pub kind: LineKind,
    pub content: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
}

pub struct FileDiff {
    pub filename: String,
    pub additions: usize,
    pub deletions: usize,
    pub lines: Vec<DiffLine>,
    pub styled_lines: Vec<Line<'static>>,
    pub hidden: bool,
}

pub fn parse_diff(input: &str) -> Vec<FileDiff> {
    let mut files: Vec<FileDiff> = Vec::new();
    let lines: Vec<&str> = input.lines().collect();
    let len = lines.len();
    let mut i = 0;

    while i < len {
        // Look for "diff --git a/... b/..."
        if !lines[i].starts_with("diff --git ") {
            i += 1;
            continue;
        }

        let (old_name, new_name) = parse_diff_header(lines[i]);
        i += 1;

        // Skip optional extended headers (old mode, new mode, index, similarity, etc.)
        while i < len
            && !lines[i].starts_with("diff --git ")
            && !lines[i].starts_with("--- ")
            && !lines[i].starts_with("@@ ")
        {
            i += 1;
        }

        // Parse --- and +++ if present
        let mut old_file = old_name.clone();
        let mut new_file = new_name.clone();

        if i < len && lines[i].starts_with("--- ") {
            old_file = lines[i][4..].trim_start_matches("a/").to_string();
            i += 1;
        }
        if i < len && lines[i].starts_with("+++ ") {
            new_file = lines[i][4..].trim_start_matches("b/").to_string();
            i += 1;
        }

        let mut diff_lines: Vec<DiffLine> = Vec::new();
        let mut additions: usize = 0;
        let mut deletions: usize = 0;
        let mut old_lineno: u32 = 0;
        let mut new_lineno: u32 = 0;

        // Parse hunks
        while i < len && !lines[i].starts_with("diff --git ") {
            let line = lines[i];

            if line.starts_with("@@ ") {
                // Parse hunk header: @@ -old_start,old_count +new_start,new_count @@
                if let Some((os, ns)) = parse_hunk_header(line) {
                    old_lineno = os;
                    new_lineno = ns;
                }
                diff_lines.push(DiffLine {
                    kind: LineKind::HunkHeader,
                    content: line.to_string(),
                    old_lineno: None,
                    new_lineno: None,
                });
            } else if let Some(rest) = line.strip_prefix('+') {
                additions += 1;
                diff_lines.push(DiffLine {
                    kind: LineKind::Add,
                    content: rest.to_string(),
                    old_lineno: None,
                    new_lineno: Some(new_lineno),
                });
                new_lineno += 1;
            } else if let Some(rest) = line.strip_prefix('-') {
                deletions += 1;
                diff_lines.push(DiffLine {
                    kind: LineKind::Remove,
                    content: rest.to_string(),
                    old_lineno: Some(old_lineno),
                    new_lineno: None,
                });
                old_lineno += 1;
            } else if line.starts_with(' ') || line.is_empty() {
                let content = if line.is_empty() {
                    String::new()
                } else {
                    line[1..].to_string()
                };
                diff_lines.push(DiffLine {
                    kind: LineKind::Context,
                    content,
                    old_lineno: Some(old_lineno),
                    new_lineno: Some(new_lineno),
                });
                old_lineno += 1;
                new_lineno += 1;
            } else if line.starts_with('\\') {
                // "\ No newline at end of file" — skip
            }

            i += 1;
        }

        let filename = if new_file == "/dev/null" {
            old_file.clone()
        } else {
            new_file.clone()
        };

        files.push(FileDiff {
            filename,
            additions,
            deletions,
            lines: diff_lines,
            styled_lines: Vec::new(),
            hidden: false,
        });
    }

    files
}

fn parse_diff_header(line: &str) -> (String, String) {
    // "diff --git a/foo/bar.rs b/foo/bar.rs"
    let rest = &line["diff --git ".len()..];
    // Find the split point: "a/... b/..."
    // The tricky part is filenames can contain spaces. We find " b/" as separator.
    if let Some(pos) = rest.find(" b/") {
        let old = rest[2..pos].to_string(); // skip "a/"
        let new = rest[pos + 3..].to_string(); // skip " b/"
        (old, new)
    } else {
        // Fallback
        let parts: Vec<&str> = rest.splitn(2, ' ').collect();
        if parts.len() == 2 {
            (
                parts[0].trim_start_matches("a/").to_string(),
                parts[1].trim_start_matches("b/").to_string(),
            )
        } else {
            (rest.to_string(), rest.to_string())
        }
    }
}

fn parse_hunk_header(line: &str) -> Option<(u32, u32)> {
    // @@ -old_start[,old_count] +new_start[,new_count] @@
    let trimmed = line.trim_start_matches("@@ ");
    let end = trimmed.find(" @@")?;
    let range_part = &trimmed[..end];

    let mut old_start = 1u32;
    let mut new_start = 1u32;

    for part in range_part.split(' ') {
        if let Some(s) = part.strip_prefix('-') {
            old_start = s.split(',').next()?.parse().ok()?;
        } else if let Some(s) = part.strip_prefix('+') {
            new_start = s.split(',').next()?.parse().ok()?;
        }
    }

    Some((old_start, new_start))
}
