use std::fmt::Display;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Paragraph, Widget},
};

use super::{
    component::*,
    util::{default_style, highlight_style}, editors::{EditorComponent, EditorEventResult},
};

pub const DONE_CANCEL: [&str; 2] = ["Done", "Cancel"];

pub struct MultipleChoice {
    pub active: bool,
    choices: Vec<String>,
    selected: usize,
}

impl MultipleChoice {
    pub fn new<S, I>(choices: I, selected: usize) -> Self
    where
        S: Display,
        I: IntoIterator<Item = S>,
    {
        let choices: Vec<String> = choices.into_iter().map(|c| format!(" {} ", c)).collect();
        Self {
            active: true,
            choices,
            selected,
        }
    }

    pub fn get_render_width(&self) -> u16 {
        self.choices.iter().map(|c| c.len() + 2).sum::<usize>() as u16
    }

    pub fn get_selected(&self) -> String {
        self.choices[self.selected].trim().into()
    }

    fn selection_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Left => self.selected = self.selected.saturating_sub(1),
            KeyCode::Right => {
                if self.selected < self.choices.len() - 1 {
                    self.selected += 1;
                }
            }
            _ => {},
        }
    }
}

impl Component for MultipleChoice {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        let res = match key_event.code {
            KeyCode::Esc => ComponentStatus::Escaped,
            KeyCode::Enter => ComponentStatus::Closed,
            _ => {
                self.selection_key_event(key_event);
                ComponentStatus::Active
            }
        };
        Ok(res)
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Cursor {
        let choice_constraints =
            vec![Constraint::Ratio(1, self.choices.len() as u32); self.choices.len()];

        let choice_areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(choice_constraints)
            .split(area);
        let mut styles = vec![default_style(); self.choices.len()];
        if self.active {
            styles[self.selected] = highlight_style();
        }

        self.choices
            .iter()
            .zip(choice_areas.into_iter())
            .zip(styles.into_iter())
            .for_each(|((choice, &area), style)| {
                Paragraph::new(choice.clone())
                    .alignment(Alignment::Center)
                    .style(style)
                    .render(area, buf);
            });
        None
    }
}

impl EditorComponent for MultipleChoice {
    fn enter_from_below(&mut self) {
        self.active = true;
        self.selected = self.choices.len() - 1;
    }

    fn enter_from_top(&mut self) {
        self.active = true;
        self.selected = 0;
    }

    fn get_highlight(&self) -> Option<usize> {
            None
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> EditorEventResult {
        match key_event.code {
            KeyCode::Esc => EditorEventResult::Escaped,
            KeyCode::Enter => EditorEventResult::Closed,
            KeyCode::Down => {
                self.active = false;
                EditorEventResult::ExitBottom
            }
            KeyCode::Up => {
                self.active = false;
                EditorEventResult::ExitTop
            }
            _ => {
                self.selection_key_event(key_event);
                EditorEventResult::KeepActive
            }
        }
    }

    fn calculate_render_height(&self) -> usize {
        1
    }

    fn get_text_output(&self, _text: &mut String) {
        unimplemented!("MultipleChoice does not support rendering into text")
    }

    fn get_highlight_prefix(&self) -> Option<usize> {
        None
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Cursor {
        Component::render(self, area, buf)
    }
}
