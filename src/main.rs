mod app;
mod demo;
mod diff;
mod highlight;
mod layout;
mod ui;

use std::env;
use std::fs::OpenOptions;
use std::io::{self, IsTerminal, Read};
use std::os::unix::io::AsRawFd;

use app::App;
use diff::parse_diff;
use highlight::Highlighter;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> color_eyre::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    if let Some(arg) = args.first() {
        match arg.as_str() {
            "--version" | "-V" => {
                println!("asd {}", VERSION);
                return Ok(());
            }
            "--help" | "-h" | "help" => {
                print_help();
                return Ok(());
            }
            other => {
                eprintln!("Unknown option: {}", other);
                eprintln!("Run 'asd --help' for usage.");
                std::process::exit(1);
            }
        }
    }

    color_eyre::install()?;

    let mut files = if io::stdin().is_terminal() {
        // No pipe — run demo mode with built-in poem diffs
        demo::demo_files()
    } else {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        let f = parse_diff(&input);
        if f.is_empty() {
            eprintln!("No diff content found.");
            std::process::exit(1);
        }
        f
    };

    // Pre-compute syntax highlighting for all files
    let highlighter = Highlighter::new();
    for file in &mut files {
        highlighter.highlight_file(file);
    }

    // Open /dev/tty for keyboard input (stdin may be a consumed pipe)
    let tty = OpenOptions::new().read(true).write(true).open("/dev/tty")?;

    // Enable raw mode on the tty so we get individual keypresses
    let tty_fd = tty.as_raw_fd();
    let mut original_termios: libc::termios = unsafe { std::mem::zeroed() };
    if unsafe { libc::tcgetattr(tty_fd, &mut original_termios) } != 0 {
        color_eyre::eyre::bail!("Failed to get terminal attributes");
    }
    let mut raw = original_termios;
    unsafe { libc::cfmakeraw(&mut raw) };
    if unsafe { libc::tcsetattr(tty_fd, libc::TCSANOW, &raw) } != 0 {
        color_eyre::eyre::bail!("Failed to set raw mode");
    }

    // Set up ratatui with stdout (which is still the terminal)
    let backend = ratatui::backend::CrosstermBackend::new(io::stdout());
    let mut terminal = ratatui::Terminal::new(backend)?;

    // Enter alternate screen
    crossterm::execute!(io::stdout(), crossterm::terminal::EnterAlternateScreen)?;

    let mut app = App::new(files, tty);
    let result = run_loop(&mut terminal, &mut app);

    // Restore terminal
    crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;
    unsafe {
        libc::tcsetattr(tty_fd, libc::TCSANOW, &original_termios);
    }

    result
}

fn run_loop(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> color_eyre::Result<()> {
    loop {
        terminal.draw(|f| app.draw(f))?;

        app.handle_event()?;

        if app.should_quit {
            return Ok(());
        }
    }
}

fn print_help() {
    println!("asd {} — terminal diff viewer", VERSION);
    println!();
    println!("USAGE:");
    println!("    git diff | asd");
    println!("    diff -u old.txt new.txt | asd");
    println!("    asd                          (demo mode)");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help       Print this help message");
    println!("    -V, --version    Print version");
    println!();
    println!("KEYBINDINGS:");
    println!("    a / d            Previous / next file");
    println!("    s                Auto-split (BFS rotation)");
    println!("    S                Split focused pane (auto direction)");
    println!("    v / h            Vertical / horizontal split");
    println!("    m                Undo last split (merge)");
    println!("    M                Merge focused pane with sibling");
    println!("    Space            Page down");
    println!("    x                Hide focused file");
    println!("    f                File list overlay");
    println!("    r                Reset to initial state");
    println!("    Tab / 0-9        Cycle / jump to pane");
    println!("    Arrows           Focus pane in direction");
    println!("    Shift+Arrows     Scroll");
    println!("    q / Esc / Ctrl+C Quit");
}
