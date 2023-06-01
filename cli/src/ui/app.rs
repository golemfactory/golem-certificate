use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    widgets::{Block, BorderType, Borders, StatefulWidget, Widget, Padding},
};

use super::util::{Component, default_style, ComponentStatus};
use super::main_menu::MainMenu;

pub struct App {
    main_menu: MainMenu,
}

impl App {
    pub fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<bool> {
        match key_event.code {
            KeyCode::Char('c') | KeyCode::Char('C') if key_event.modifiers == KeyModifiers::CONTROL => {
                Ok(true)
            }
            _ => {
                self.main_menu.handle_key_event(key_event).map(|status| status != ComponentStatus::Active)
            }
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            main_menu: MainMenu::new(),
        }
    }
}


pub struct AppScreen {}

impl StatefulWidget for AppScreen {
    type State = App;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let padding = if area.width > 30 && area.height > 10 { 1 } else { 0 };
        let main_border = Block::default()
            .title("Golem Certificate Manager")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .style(default_style())
            .padding(Padding::uniform(padding));

        let main_area = main_border.inner(area);
        main_border.render(area, buf);

        state.main_menu.render(main_area, buf);
    }
}
