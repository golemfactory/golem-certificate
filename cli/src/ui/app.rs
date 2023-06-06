use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    widgets::{Block, BorderType, Borders, Padding, StatefulWidget, Widget},
};

use super::{
    component::*,
    main_menu::MainMenu,
    util::default_style,
};

pub struct App {
    cursor: Cursor,
    main_menu: MainMenu,
}

#[derive(PartialEq)]
pub enum AppStatus {
    Exiting,
    Running,
}

impl App {
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<AppStatus> {
        match key_event.code {
            KeyCode::Char('c') | KeyCode::Char('C')
                if key_event.modifiers == KeyModifiers::CONTROL =>
            {
                Ok(AppStatus::Exiting)
            }
            _ => match self.main_menu.handle_key_event(key_event)? {
                ComponentStatus::Active => Ok(AppStatus::Running),
                _ => Ok(AppStatus::Exiting),
            }
        }
    }

    pub fn get_cursor(&self) -> &Cursor {
        &self.cursor
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            cursor: None,
            main_menu: MainMenu::new(),
        }
    }
}

pub struct AppScreen {}

impl StatefulWidget for AppScreen {
    type State = App;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let padding = if area.width > 30 && area.height > 10 {
            1
        } else {
            0
        };
        let main_border = Block::default()
            .title("Golem Certificate Manager")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(default_style())
            .padding(Padding::uniform(padding));

        let main_area = main_border.inner(area);
        main_border.render(area, buf);

        state.cursor = state.main_menu.render(main_area, buf);
    }
}
