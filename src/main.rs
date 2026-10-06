use std::io::{self, Stdout};
use std::time::{Duration, Instant};

use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use t2d2::app::{App, Effect};
use t2d2::persistence;
use t2d2::render;
use t2d2::ui::help;

const SAVE_DEBOUNCE: Duration = Duration::from_millis(150);
const TICK: Duration = Duration::from_millis(100);

type Tui = Terminal<CrosstermBackend<Stdout>>;

fn main() -> io::Result<()> {
    let mut terminal = setup_terminal()?;
    let store = persistence::load().unwrap_or_else(|e| {
        eprintln!("t2d2: failed to load store: {e}");
        t2d2::Store::empty()
    });
    let mut app = App::new(store);
    let result = run(&mut terminal, &mut app);
    restore_terminal(&mut terminal)?;
    // Final flush.
    let _ = persistence::save(&app.store);
    result
}

fn setup_terminal() -> io::Result<Tui> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(terminal: &mut Tui) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn run(terminal: &mut Tui, app: &mut App) -> io::Result<()> {
    let mut last_save = Instant::now();
    let mut last_tick = Instant::now();

    loop {
        // Render first so the user sees something even if events block.
        terminal.draw(|f| {
            // Help overlay goes on top of everything.
            render::render(f, &app.store, &app.mode, status_hint(app));
            if app.help_open {
                help::render(f);
            }
        })?;

        // Debounced save.
        if app.dirty && last_save.elapsed() >= SAVE_DEBOUNCE {
            let _ = persistence::save(&app.store);
            app.dirty = false;
            last_save = Instant::now();
        }

        if app.should_quit {
            return Ok(());
        }

        // Poll for events with a short timeout so we can tick.
        let timeout = TICK.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            let event = event::read()?;
            if let Event::Key(key) = event {
                let effect = app.step(key);
                match effect {
                    Effect::Save => {
                        let _ = persistence::save(&app.store);
                        app.dirty = false;
                        last_save = Instant::now();
                    }
                    Effect::Quit => {
                        let _ = persistence::save(&app.store);
                        app.dirty = false;
                        return Ok(());
                    }
                    Effect::None => {}
                }
            }
        }
        last_tick = Instant::now();
    }
}

fn status_hint(app: &App) -> &'static str {
    if app.prompt.is_some() {
        return "";
    }
    match app.mode {
        t2d2::Mode::Normal => "j/k h/l  o new  c done  d del  ? help",
        t2d2::Mode::Insert {
            field: t2d2::EditField::Title { .. },
        } => "Esc save  Tab → note",
        t2d2::Mode::Insert {
            field: t2d2::EditField::Note { .. },
        } => "Esc save  Enter newline",
    }
}