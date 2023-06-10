use super::*;

use std::collections::{BTreeMap, HashMap};

use golem_certificate::schemas::subject::{Subject, Contact};

pub struct SubjectEditor {
    subject: Subject,
    additional_subject_properties: BTreeMap<String, String>,
    additional_contact_properties: BTreeMap<String, String>,
    highlight: Option<usize>,
    property_editor: Option<TextInput>,
    value_editor: Option<TextInput>,
    error_message: Option<ModalMessage>,
}

impl SubjectEditor {
    pub fn new(subject: Option<Subject>) -> Self {
        let subject = subject.unwrap_or(Subject {
            display_name: "Certificate Holder".into(),
            contact: Contact { email: "certificate.holder@example.com".into(), additional_properties: Default::default() },
            additional_properties: Default::default(),
        });
        Self {
            additional_subject_properties: filter_string_values(&subject.additional_properties),
            additional_contact_properties: filter_string_values(&subject.contact.additional_properties),
            subject: subject,
            highlight: None,
            property_editor: None,
            value_editor: None,
            error_message: None,
        }
    }

    pub fn get_subject(&self) -> Subject {
        let mut subject = self.subject.clone();
        let additional_subject_properties = map_string_values_to_value(&self.additional_subject_properties);
        subject.additional_properties.extend(additional_subject_properties);
        let additional_contact_properties = map_string_values_to_value(&self.additional_contact_properties);
        subject.contact.additional_properties.extend(additional_contact_properties);
        subject
    }

    fn calculate_contact_start_line(&self) -> usize {
        3 + self.additional_subject_properties.len()
    }
}

fn filter_string_values(map: &HashMap<String, serde_json::Value>) -> BTreeMap<String, String> {
    map.iter().filter_map(|(k, v)| {
        if v.is_string() {
            Some((k.to_owned(), v.as_str().unwrap().to_owned()))
        } else {
            None
        }
    }).collect()
}

fn map_string_values_to_value(map: &BTreeMap<String, String>) -> Vec<(String, serde_json::Value)> {
    map.iter().map(|(k, v)| (k.to_owned(), serde_json::Value::String(v.to_owned()))).collect()
}

impl Default for SubjectEditor {
    fn default() -> Self {
        Self::new(None)
    }
}

impl EditorComponent for SubjectEditor {
    fn enter_from_below(&mut self) {
        self.highlight = Some(self.calculate_render_height() - 1);
    }

    fn enter_from_top(&mut self) {
        self.highlight = Some(1);
    }

    fn get_highlight(&self) -> Option<usize> {
        self.highlight
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> EditorEventResult {
        if let Some(error_message) = self.error_message.as_mut() {
            match error_message.handle_key_event(key_event) {
                Ok(status) => match status {
                    ComponentStatus::Active => {},
                    _ => self.error_message = None,
                }
                Err(_) => {},
            }
            EditorEventResult::KeepActive
        } else if let Some(value_editor) = self.value_editor.as_mut() {
            match value_editor.handle_key_event(key_event) {
                Ok(status) => match status {
                    ComponentStatus::Active => {},
                    ComponentStatus::Closed => {
                        let text = value_editor.get_text().to_owned();
                        if text.is_empty() {
                            self.error_message = Some(ModalMessage::new("Empty value", "Value must not be empty"));
                        } else {
                            let highlight = self.highlight.unwrap();
                            if highlight == 1 {
                                self.subject.display_name = text;
                            } else if highlight == self.calculate_contact_start_line() + 1 {
                                self.subject.contact.email = text;
                            }
                            self.value_editor = None;
                        }
                    },
                    ComponentStatus::Escaped => self.value_editor = None,
                }
                Err(_) => {},
            }
            EditorEventResult::KeepActive
        } else if let Some(property_editor) = self.property_editor.as_mut() {
            match property_editor.handle_key_event(key_event) {
                Ok(status) => match status {
                    ComponentStatus::Active => {},
                    ComponentStatus::Closed => {},
                    ComponentStatus::Escaped => self.property_editor = None,
                },
                Err(_) => {},
            }
            EditorEventResult::KeepActive
        } else {
            if let Some(highlight) = self.highlight {
                match key_event.code {
                    KeyCode::Esc => EditorEventResult::Escaped,
                    KeyCode::Down => {
                        if highlight < self.calculate_render_height() - 1 {
                            let mut new_highlight = highlight + 1;
                            if new_highlight == self.calculate_contact_start_line() {
                                new_highlight += 1;
                            };
                            self.highlight = Some(new_highlight);
                            EditorEventResult::KeepActive
                        } else {
                            self.highlight = None;
                            EditorEventResult::ExitBottom
                        }
                    }
                    KeyCode::Up => {
                        if highlight > 1 {
                            let mut new_highlight = highlight - 1;
                            if new_highlight == self.calculate_contact_start_line() {
                                new_highlight -= 1;
                            };
                            self.highlight = Some(new_highlight);
                            EditorEventResult::KeepActive
                        } else {
                            self.highlight = None;
                            EditorEventResult::ExitTop
                        }
                    }
                    KeyCode::Delete | KeyCode::Backspace => {
                        if highlight > 1 && highlight < self.calculate_contact_start_line() - 1 {
                            let idx = highlight - 2;
                            let key = self.additional_subject_properties.keys().nth(idx).unwrap().to_owned();
                            if self.subject.additional_properties.contains_key(&key) {
                                self.subject.additional_properties.remove(&key);
                            }
                            self.additional_subject_properties.remove(&key);
                        } else if highlight > self.calculate_contact_start_line() && highlight < self.calculate_render_height() - 2 {
                            let idx = highlight - self.calculate_contact_start_line() - 2;
                            let key = self.additional_contact_properties.keys().nth(idx).unwrap().to_owned();
                            if self.subject.contact.additional_properties.contains_key(&key) {
                                self.subject.contact.additional_properties.remove(&key);
                            }
                            self.additional_contact_properties.remove(&key);
                        }
                        EditorEventResult::KeepActive
                    }
                    KeyCode::Enter => {
                        if highlight == 1 {
                            let mut editor = TextInput::new(255, false);
                            editor.set_text(self.subject.display_name.clone());
                            self.value_editor = Some(editor);
                        } else if highlight == self.calculate_contact_start_line() + 1 {
                            let mut editor = TextInput::new(255, false);
                            editor.set_text(self.subject.contact.email.clone());
                            self.value_editor = Some(editor);
                        } else if highlight > 1 && highlight < self.calculate_contact_start_line() - 1 {
                            let idx = highlight - 2;
                            let key = self.additional_subject_properties.keys().nth(idx).unwrap().to_owned();
                            let mut editor = TextInput::new(42, false);
                            editor.set_text(self.additional_subject_properties.get(&key).unwrap().to_owned());
                            self.value_editor = Some(editor);
                        } else if highlight > self.calculate_contact_start_line() && highlight < self.calculate_render_height() - 2 {
                            let idx = highlight - self.calculate_contact_start_line() - 2;
                            let key = self.additional_contact_properties.keys().nth(idx).unwrap().to_owned();
                            let mut editor = TextInput::new(42, false);
                            editor.set_text(self.additional_contact_properties.get(&key).unwrap().to_owned());
                            self.value_editor = Some(editor);
                        } else if highlight == self.calculate_render_height() - 1 {
                            let mut editor = TextInput::new(42, false);
                            editor.set_text("".to_owned());
                            self.property_editor = Some(editor);
                        } else if highlight == self.calculate_contact_start_line() - 1 {
                            let mut editor = TextInput::new(42, false);
                            editor.set_text("".to_owned());
                            self.property_editor = Some(editor);
                        }
                        EditorEventResult::KeepActive
                    }
                    _ => EditorEventResult::KeepActive,
                }
            } else {
                EditorEventResult::Inactive
            }
        }
    }

    fn calculate_render_height(&self) -> usize {
        3 + self.additional_subject_properties.len() + 3 + self.additional_contact_properties.len()
    }

    fn get_text_output(&self, text: &mut String) {
        writeln!(text, "Subject").unwrap();
        writeln!(text, "  Display Name: {}", self.subject.display_name).unwrap();
        for (key, value) in &self.additional_subject_properties {
            writeln!(text, "  {}: {}", key, value).unwrap();
        }
        writeln!(text, "").unwrap();
        writeln!(text, "  Contact").unwrap();
        writeln!(text, "    Email: {}", self.subject.contact.email).unwrap();
        for (key, value) in &self.additional_contact_properties {
            writeln!(text, "    {}: {}", key, value).unwrap();
        }
        writeln!(text, "").unwrap();
    }

    fn get_highlight_prefix(&self) -> Option<usize> {
        self.highlight.map(|highlight| {
            let contact_start_line = self.calculate_contact_start_line();
            if highlight == 1 {
                16
            } else if highlight < contact_start_line {
                2
            } else if highlight == contact_start_line + 1 {
                11
            } else {
                4
            }
        })
    }

    fn get_editor(&mut self) -> Option<&mut TextInput> {
        self.value_editor.as_mut()
    }

    fn get_error_message(&mut self) -> Option<&mut ModalMessage> {
        self.error_message.as_mut()
    }

    fn get_empty_highlight_filler(&self) -> (String, String) {
        if self.highlight.unwrap_or(0) < self.calculate_contact_start_line() {
            ("  ".into(), "<Add more details>".into())
        } else {
            ("    ".into(), "<Add more contact details>".into())
        }
    }
}
