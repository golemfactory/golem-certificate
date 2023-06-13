use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Paragraph, Widget},
};

use super::{
    component::*,
    keypair::CreateKeyPairDialog,
    util::{default_style, get_middle_rectangle, highlight_style},
    verify_document::VerifyDocument, certificate::CertificateDocumentEditor, node_descriptor::NodeDescriptorEditor, document_editor::SignedDocumentEditor,
};

const MENU_ITEMS: [&str; 8] = [
    "Verify document",
    "",
    "Create certificate",
    "Create node descriptor",
    "",
    "Create key pair",
    "",
    "Exit",
];

pub struct MainMenu {
    child: Option<Box<dyn Component>>,
    items: Vec<&'static str>,
    selected_item: usize,
}

impl MainMenu {
    pub fn new() -> Self {
        Self {
            child: None,
            items: MENU_ITEMS.into(),
            selected_item: 0,
        }
    }

    fn navigation_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        match key_event.code {
            KeyCode::Up => {
                if self.selected_item > 0 {
                    self.selected_item -= 1;
                }
                while self.selected_item > 0 && self.items[self.selected_item].is_empty() {
                    self.selected_item -= 1;
                }
            }
            KeyCode::Down => {
                let max = self.items.len() - 1;
                if self.selected_item < max {
                    self.selected_item += 1;
                }
                while self.selected_item < max && self.items[self.selected_item].is_empty() {
                    self.selected_item += 1;
                }
            }
            KeyCode::Enter => match self.selected_item {
                0 => self.child = Some(Box::new(VerifyDocument::new()?)),
                2 => self.child = Some(Box::new(
                    SignedDocumentEditor::new(Box::new(CertificateDocumentEditor::default()))
                )),
                3 => self.child = Some(Box::new(NodeDescriptorEditor::new())),
                5 => self.child = Some(Box::new(CreateKeyPairDialog::new()?)),
                7 => return Ok(ComponentStatus::Closed),
                _ => {}
            },
            KeyCode::Esc => return Ok(ComponentStatus::Closed),
            _ => {}
        }
        Ok(ComponentStatus::Active)
    }

    fn render_self(&self, area: Rect, buf: &mut Buffer) {
        let longest_item = self.items.iter().map(|i| i.len() as u16).max().unwrap();

        let menu_area = get_middle_rectangle(area, self.items.len() as u16, longest_item);

        let row_constraints = {
            let mut constraints = vec![Constraint::Max(1); self.items.len()];
            constraints.push(Constraint::Min(0));
            constraints
        };

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraints)
            .split(menu_area);

        self.items.iter().enumerate().for_each(|(idx, &item)| {
            if item.len() > 0 {
                let style = if self.selected_item == idx {
                    highlight_style()
                } else {
                    default_style()
                };
                Paragraph::new(item)
                    .style(style)
                    .alignment(Alignment::Center)
                    .render(rows[idx], buf);
            }
        });
    }
}

impl Default for MainMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for MainMenu {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        if let Some(component) = &mut self.child {
            match component.handle_key_event(key_event)? {
                ComponentStatus::Active => {},
                _ => self.child = None,
            };
            Ok(ComponentStatus::Active)
        } else {
            self.navigation_key_event(key_event)
        }
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Cursor {
        if let Some(component) = &mut self.child {
            component.render(area, buf)
        } else {
            self.render_self(area, buf);
            None
        }
    }
}
