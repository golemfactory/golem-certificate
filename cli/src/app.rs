use std::io;

use anyhow::Result;
use crossterm::event::{self, Event};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use tui::backend::{Backend, CrosstermBackend};
use tui::Terminal;

use crate::ui::component::CursorPosition;

use super::ui::app::{App, AppScreen, AppStatus};

pub fn start() -> Result<()> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    terminal::enable_raw_mode()?;
    crossterm::execute!(io::stdout(), EnterAlternateScreen)?;
    terminal.clear()?;

    let mut error_message = None;
    let mut app_running = true;
    while app_running {
        match app_loop(&mut terminal, &mut app) {
            Ok(app_status) => app_running = app_status == AppStatus::Running,
            Err(err) => {
                app_running = false;
                error_message = Some(format!(
                    "Some unrecoverable error occurred: {}",
                    err.to_string()
                ));
            }
        }
    }

    terminal::disable_raw_mode()?;
    crossterm::execute!(io::stdout(), LeaveAlternateScreen)?;
    if let Some(message) = error_message {
        eprintln!("{}", message);
    }
    Ok(())
}

fn app_loop<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<AppStatus> {
    terminal.draw(|frame| {
        frame.render_stateful_widget(AppScreen {}, frame.size(), app);
        if let Some(CursorPosition { x, y}) = app.get_cursor() {
            frame.set_cursor(*x, *y);
        }
    })?;
    match event::read()? {
        Event::Key(e) => app.handle_key_event(e),
        _ => Ok(AppStatus::Running),
    }
}
