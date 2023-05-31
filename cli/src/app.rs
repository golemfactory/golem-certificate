use std::io;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use tui::backend::{Backend, CrosstermBackend};
use tui::Terminal;

use super::ui::app::{App, AppScreen};

pub fn start() -> Result<()> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    terminal::enable_raw_mode()?;
    crossterm::execute!(io::stdout(), EnterAlternateScreen)?;
    terminal.clear()?;

    let mut error_message = None;
    while app.running {
        app_loop(&mut terminal, &mut app).map_err(|err| {
            app.running = false;
            error_message = Some(format!("Some unrecoverable error occurred: {}", err.to_string()));
        });
    }


    terminal::disable_raw_mode()?;
    crossterm::execute!(io::stdout(), LeaveAlternateScreen)?;
    if let Some(message) = error_message {
        eprintln!("{}", message);
    }
    Ok(())
}

fn app_loop<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    terminal.draw(|frame| frame.render_stateful_widget(AppScreen {}, frame.size(), app))?;
    match event::read()? {
        Event::Key(e) => {
            if (e.code == KeyCode::Char('c') || e.code == KeyCode::Char('C')) && e.modifiers == KeyModifiers::CONTROL {
                app.running = false;
            } else {
                app.handle_key_event(e)?;
            }
        },
        Event::Resize(_w, _h) => {},
        _ => unimplemented!()
    }
    Ok(())
}
