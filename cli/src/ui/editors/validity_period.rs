use super::*;

use chrono::{DateTime, Utc, Timelike};
use golem_certificate::schemas::validity_period::ValidityPeriod;

pub struct ValidityPeriodEditor {
    not_before: String,
    not_after: String,
    highlight: Option<usize>,
    date_editor: Option<TextInput>,
    parse_error: Option<ModalMessage>,
}

impl ValidityPeriodEditor {
    pub fn new(validity_period: Option<ValidityPeriod>) -> Self {
        let now = Utc::now().with_nanosecond(0).unwrap();
        let (not_before, not_after) = match &validity_period {
            Some(ValidityPeriod { not_before, not_after }) => (not_before, not_after),
            None => {
                (&now, &now)
            }
        };
        Self {
            not_before: not_before.to_string(),
            not_after: not_after.to_string(),
            highlight: None,
            date_editor: None,
            parse_error: None,
        }
    }

    pub fn get_validity_period(&self) -> ValidityPeriod {
        ValidityPeriod {
            not_before: self.not_before.parse().unwrap(),
            not_after: self.not_after.parse().unwrap(),
        }
    }
}

impl Default for ValidityPeriodEditor {
    fn default() -> Self {
        Self::new(None)
    }
}

impl EditorComponent for ValidityPeriodEditor {
    fn enter_from_below(&mut self) {
        self.highlight = Some(2);
    }

    fn enter_from_top(&mut self) {
        self.highlight = Some(1);
    }

    fn get_highlight(&self) -> Option<usize> {
        self.highlight
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> EditorEventResult {
        if let Some(parse_error) = self.parse_error.as_mut() {
            match parse_error.handle_key_event(key_event) {
                Ok(status) => match status {
                    ComponentStatus::Active => {},
                    _ => self.parse_error = None,
                }
                Err(_) => {},
            };
            EditorEventResult::KeepActive
        } else if let Some(date_editor) = self.date_editor.as_mut() {
            match Component::handle_key_event(date_editor, key_event) {
                Ok(status) => match status {
                    ComponentStatus::Active => {},
                    ComponentStatus::Closed => {
                        match date_editor.get_text().parse::<DateTime<Utc>>() {
                            Ok(utc_time) => {
                                let datetime = date_editor.get_text().to_owned();
                                if self.highlight.unwrap() == 1 {
                                    if utc_time > self.not_after.parse::<DateTime<Utc>>().unwrap() {
                                        let error = ModalMessage::new("Datetime error", "Not Before must be before Not After");
                                        self.parse_error = Some(error);
                                    } else {
                                        self.not_before = datetime;
                                        self.date_editor = None;
                                    }
                                } else {
                                    if utc_time < self.not_before.parse::<DateTime<Utc>>().unwrap() {
                                        let error = ModalMessage::new("Datetime error", "Not After must be after Not Before");
                                        self.parse_error = Some(error);
                                    } else {
                                        self.not_after = datetime;
                                        self.date_editor = None;
                                    }
                                }
                            }
                            Err(err) => {
                                let error = ModalMessage::new("Datetime parse error", err.to_string());
                                self.parse_error = Some(error);
                            }
                        }
                    }
                    ComponentStatus::Escaped => self.date_editor = None,
                }
                Err(_) => {},
            };
            EditorEventResult::KeepActive
        } else if let Some(highlight) = self.highlight {
            match key_event.code {
                KeyCode::Esc => EditorEventResult::Escaped,
                KeyCode::Down =>
                    if highlight == 2 {
                        self.highlight = None;
                        EditorEventResult::ExitBottom
                    } else {
                        self.highlight = Some(2);
                        EditorEventResult::KeepActive
                    },
                KeyCode::Up =>
                    if highlight == 1 {
                        self.highlight = None;
                        EditorEventResult::ExitTop
                    } else {
                        self.highlight = Some(1);
                        EditorEventResult::KeepActive
                    },
                KeyCode::Enter => {
                    // 2014-11-28T21:00:09+09:00 => 25
                    let mut editor = TextInput::new(30, false);
                    if highlight == 1 {
                        editor.set_text(self.not_before.clone());
                    } else {
                        editor.set_text(self.not_after.clone());
                    }
                    self.date_editor = Some(editor);
                    EditorEventResult::KeepActive
                },
                _ => EditorEventResult::KeepActive,
            }
        } else {
            EditorEventResult::Inactive
        }
    }

    fn calculate_render_height(&self) -> usize  {
        3
    }

    fn get_text_output(&self, text: &mut String) {
        writeln!(text, "Validity Period").unwrap();
        writeln!(text, "  Not Before: {}", self.not_before).unwrap();
        writeln!(text, "  Not After:  {}", self.not_after).unwrap();
    }

    fn get_highlight_prefix(&self) -> Option<usize> {
        self.highlight.map(|_| 14)
    }

    fn get_editor(&mut self) -> Option<&mut TextInput> {
        self.date_editor.as_mut()
    }

    fn get_error_message(&mut self) -> Option<&mut ModalMessage> {
        self.parse_error.as_mut()
    }
}
