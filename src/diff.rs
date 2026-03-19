use ratatui::text::Line;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    fn basic_diff() -> &'static str {
        concat!(
            "diff --git a/src/main.rs b/src/main.rs\n",
            "index abc1234..def5678 100644\n",
            "--- a/src/main.rs\n",
            "+++ b/src/main.rs\n",
            "@@ -1,4 +1,5 @@\n",
            "-fn old() {\n",
            "+fn new() {\n",
            " context line\n",
            "+added line\n",
            " more context\n",
        )
    }

    #[test]
    fn parse_basic_diff() {
        let files = parse_diff(basic_diff());
        assert_eq!(files.len(), 1);
        let f = &files[0];
        assert_eq!(f.filename, "src/main.rs");
        assert_eq!(f.additions, 2);
        assert_eq!(f.deletions, 1);
        assert_eq!(f.lines.len(), 6); // hunk header + 5 body lines
    }

    #[test]
    fn parse_multi_file_diff() {
        let input = concat!(
            "diff --git a/foo.rs b/foo.rs\n",
            "--- a/foo.rs\n",
            "+++ b/foo.rs\n",
            "@@ -1,1 +1,1 @@\n",
            "-old\n",
            "+new\n",
            "diff --git a/bar.rs b/bar.rs\n",
            "--- a/bar.rs\n",
            "+++ b/bar.rs\n",
            "@@ -1,1 +1,1 @@\n",
            "-alpha\n",
            "+beta\n",
        );
        let files = parse_diff(input);
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].filename, "foo.rs");
        assert_eq!(files[1].filename, "bar.rs");
    }

    #[test]
    fn parse_rename_diff() {
        let input = concat!(
            "diff --git a/old_name.rs b/new_name.rs\n",
            "similarity index 90%\n",
            "rename from old_name.rs\n",
            "rename to new_name.rs\n",
            "--- a/old_name.rs\n",
            "+++ b/new_name.rs\n",
            "@@ -1,1 +1,1 @@\n",
            "-old\n",
            "+new\n",
        );
        let files = parse_diff(input);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].filename, "new_name.rs");
    }

    #[test]
    fn parse_new_file() {
        let input = concat!(
            "diff --git a/new.rs b/new.rs\n",
            "new file mode 100644\n",
            "--- /dev/null\n",
            "+++ b/new.rs\n",
            "@@ -0,0 +1,2 @@\n",
            "+line one\n",
            "+line two\n",
        );
        let files = parse_diff(input);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].filename, "new.rs");
        assert_eq!(files[0].additions, 2);
        assert_eq!(files[0].deletions, 0);
    }

    #[test]
    fn parse_deleted_file() {
        let input = concat!(
            "diff --git a/gone.rs b/gone.rs\n",
            "deleted file mode 100644\n",
            "--- a/gone.rs\n",
            "+++ /dev/null\n",
            "@@ -1,2 +0,0 @@\n",
            "-line one\n",
            "-line two\n",
        );
        let files = parse_diff(input);
        assert_eq!(files.len(), 1);
        // deleted file: +++ /dev/null, so filename comes from old_file
        assert_eq!(files[0].filename, "gone.rs");
        assert_eq!(files[0].additions, 0);
        assert_eq!(files[0].deletions, 2);
    }

    #[test]
    fn hunk_header_line_numbers() {
        let input = concat!(
            "diff --git a/f.rs b/f.rs\n",
            "--- a/f.rs\n",
            "+++ b/f.rs\n",
            "@@ -10,3 +20,4 @@\n",
            " context\n",
            "-removed\n",
            "+added\n",
            "+also added\n",
            " context2\n",
        );
        let files = parse_diff(input);
        let lines = &files[0].lines;

        // First body line after hunk header is context at old=10, new=20
        assert_eq!(lines[1].kind, LineKind::Context);
        assert_eq!(lines[1].old_lineno, Some(10));
        assert_eq!(lines[1].new_lineno, Some(20));

        // Remove at old=11
        assert_eq!(lines[2].kind, LineKind::Remove);
        assert_eq!(lines[2].old_lineno, Some(11));
        assert_eq!(lines[2].new_lineno, None);

        // Add at new=21
        assert_eq!(lines[3].kind, LineKind::Add);
        assert_eq!(lines[3].old_lineno, None);
        assert_eq!(lines[3].new_lineno, Some(21));

        // Second add at new=22
        assert_eq!(lines[4].kind, LineKind::Add);
        assert_eq!(lines[4].new_lineno, Some(22));
    }

    #[test]
    fn line_kinds_and_content_no_prefix() {
        let files = parse_diff(basic_diff());
        let lines = &files[0].lines;

        assert_eq!(lines[0].kind, LineKind::HunkHeader);

        // Remove line: content should NOT have the leading '-'
        assert_eq!(lines[1].kind, LineKind::Remove);
        assert_eq!(lines[1].content, "fn old() {");

        // Add line: content should NOT have the leading '+'
        assert_eq!(lines[2].kind, LineKind::Add);
        assert_eq!(lines[2].content, "fn new() {");

        // Context line: content should NOT have the leading ' '
        assert_eq!(lines[3].kind, LineKind::Context);
        assert_eq!(lines[3].content, "context line");
    }

    #[test]
    fn empty_input_returns_empty_vec() {
        assert!(parse_diff("").is_empty());
        assert!(parse_diff("some random text\nno diff here\n").is_empty());
    }
}
