use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use rdu::{App, Args, scan_dir, ui};

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::ui(f, &mut app))?;

        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press {
                // Clear status message on any key press
                app.status_message = None;
                
                match (key.code, key.modifiers) {
                    (KeyCode::Char('q'), _) | (KeyCode::Esc, _) if !app.show_help => return Ok(()),
                    (KeyCode::Esc, _) => app.show_help = false,
                    (KeyCode::Char('?'), _) => app.show_help = !app.show_help,
                    _ if app.show_help => app.show_help = false, // Any key closes help
                    // Navigation
                    (KeyCode::Down, _) | (KeyCode::Char('j'), _) => app.next(),
                    (KeyCode::Up, _) | (KeyCode::Char('k'), _) => app.previous(),
                    (KeyCode::Char('d'), KeyModifiers::CONTROL) | (KeyCode::PageDown, _) => app.page_down(),
                    (KeyCode::Char('u'), KeyModifiers::CONTROL) | (KeyCode::PageUp, _) => app.page_up(),
                    (KeyCode::Char('H'), _) | (KeyCode::Home, _) => app.go_to_first(),
                    (KeyCode::Char('G'), _) | (KeyCode::End, _) => app.go_to_last(),
                    // Actions
                    (KeyCode::Enter, _) | (KeyCode::Right, _) | (KeyCode::Char('l'), _) | (KeyCode::Char('o'), _) => app.enter_dir(),
                    (KeyCode::Backspace, _) | (KeyCode::Left, _) | (KeyCode::Char('h'), _) | (KeyCode::Char('u'), _) => app.go_up(),
                    (KeyCode::Char('r'), _) => app.refresh(),
                    // Sort options
                    (KeyCode::Char('s'), _) => app.toggle_sort_by_size(),
                    (KeyCode::Char('m'), _) => app.toggle_sort_by_mtime(),
                    (KeyCode::Char('c'), _) => app.toggle_sort_by_count(),
                    _ => {}
                }
            }
    }
}

fn setup_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Attempt to restore terminal state
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(panic_info);
    }));
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("Scanning {}... This may take a moment.", args.path.display());

    let root_node = scan_dir(&args.path, &args);

    // Setup panic hook before entering raw mode
    setup_panic_hook();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new(root_node, args);
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}