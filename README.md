# asd

A terminal-based side-by-side diff viewer with syntax highlighting, split panes, and word-level change detection.

Pipe any unified diff into `asd` and get a rich, navigable TUI for reviewing changes.

## Installation

### Cargo (from source)

```sh
cargo install asd
```

### Homebrew

```sh
brew install vdeantoni/tap/asd
```

### Prebuilt binaries

Download a binary from the [latest GitHub release](https://github.com/vdeantoni/asd/releases/latest).

## Usage

```sh
# Review git changes
git diff | asd

# Compare two files
diff -u old.txt new.txt | asd

# Run without args for a built-in demo
asd
```

## Keybindings

### Navigation

| Key | Action |
| --- | --- |
| `a` | Previous file |
| `d` | Next file |
| `Arrow Up` | Focus pane above |
| `Arrow Down` | Focus pane below |
| `Arrow Left` | Focus pane to the left |
| `Arrow Right` | Focus pane to the right |
| `Tab` | Cycle focus through panes |
| `0`–`9` | Focus pane by index |

### Scrolling

| Key | Action |
| --- | --- |
| `Shift+Arrow Up` | Scroll up |
| `Shift+Arrow Down` | Scroll down |
| `Shift+Arrow Left` | Scroll left |
| `Shift+Arrow Right` | Scroll right |

### Panes

| Key | Action |
| --- | --- |
| `s` / `Space` | Auto-split pane |
| `S` | Split pane (auto-detect direction) |
| `v` | Split vertically (left/right) |
| `h` | Split horizontally (top/bottom) |
| `w` | Close focused pane |

### General

| Key | Action |
| --- | --- |
| `q` / `Esc` | Quit |

## License

[MIT](LICENSE)
