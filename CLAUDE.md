# CLAUDE.md

## Project Overview

`asd` is a terminal-based diff viewer built in Rust. It reads unified diff from stdin and displays it in a full-screen TUI with split panes, syntax highlighting, and word-level change detection.

## Architecture

```
src/
├── main.rs        # Entry: read stdin, open /dev/tty, raw mode, event loop
├── app.rs         # App state, keybinding dispatch, split logic, file list overlay
├── diff.rs        # Unified diff parser (hand-rolled, no external dep)
├── highlight.rs   # Syntax highlighting (syntect) + word-level diff (similar crate)
├── layout.rs      # Arena-based split tree, focus management, scroll state
├── ui.rs          # Render panes, file list overlay, footer
└── demo.rs        # Built-in 10-file poem diff for demo mode
```

### Key Design Decisions

- **Stdin before TUI**: All piped input is read into a String, then `/dev/tty` is opened for keyboard input via raw `libc` calls (not crossterm's event system, which conflicts with piped stdin on macOS)
- **Arena-based split tree**: Nodes stored in `Vec<SplitNode>` with integer indices. Avoids Box-based borrow checker pain. Splitting overwrites a Leaf with a Split node in-place.
- **Visible indices abstraction**: Hidden files are tracked with a `hidden` flag on `FileDiff`. Slots index into `visible_indices()`, not the raw files Vec.
- **BFS split rotation**: `split_queue: VecDeque<NodeId>` tracks which pane to split next. Each split enqueues both children for future splits.
- **Pre-computed styling**: Syntax highlighting and word-level diff emphasis are computed once at startup and stored as `Vec<Line<'static>>`. Zero per-frame highlighting cost.
- **Terminal cell aspect ratio**: Split direction accounts for the ~2:1 height:width ratio of terminal characters (`height * 2` vs `width`).

## Building

```sh
cargo build --release
```

The release profile uses `opt-level = 3`, `lto = true`, `codegen-units = 1`, `strip = true` for maximum performance and minimum binary size.

## Testing

```sh
# Demo mode (no pipe needed)
cargo run

# With a real diff
git diff | cargo run

# Release binary
git diff | ./target/release/asd
```

## Dependencies

- `ratatui` + `crossterm` — TUI framework and terminal backend
- `syntect` — Syntax highlighting (Sublime Text engine)
- `similar` — Word-level diffing for intra-line emphasis
- `libc` — Raw terminal control (tcgetattr/tcsetattr, /dev/tty)
- `color-eyre` — Error handling

## Release

Releases are triggered by pushing a version tag:

```sh
# Bump version in Cargo.toml, commit, then:
git tag v0.X.0
git push origin main --tags
```

This triggers the GitHub Actions release workflow (cargo-dist) which builds binaries for macOS (x86/ARM) and Linux (x86/ARM), creates a GitHub Release, and publishes to the Homebrew tap.
