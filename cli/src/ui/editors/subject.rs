use super::*;

use std::collections::HashMap;

use golem_certificate::schemas::subject::{Contact, Subject};

use crate::ui::{
    modal::ModalWindow,
    multiple_choice::{MultipleChoice, DONE_CANCEL},
};

const FIXED_SUBJECT_PROPERTY_NAMES: [&str; 2] = ["displayName", "contact"];
const FIXED_CONTACT_PROPERTY_NAMES: [&str; 1] = ["email"];

pub struct SubjectEditor {
    subject: Subject,
    additional_subject_properties: Vec<(String, String)>,
    additional_contact_properties: Vec<(String, String)>,
    highlight: Option<usize>,
    property_editor: Option<PropertyEditor>,
    value_editor: Option<TextInput>,
    error_message: Option<ModalMessage>,
}

enum SubjectEditorError {
    EmptyName,
    EmptyValue,
    DuplicateName,
    LockedName,
}

impl SubjectEditor {
    pub fn new(subject: Option<Subject>) -> Self {
        let subject = subject.unwrap_or(Subject {
            display_name: "Certificate Holder".into(),
            contact: Contact {
                email: "certificate.holder@example.com".into(),
                additional_properties: Default::default(),
            },
            additional_properties: Default::default(),
        });
        Self {
            additional_subject_properties: filter_string_values(&subject.additional_properties),
            additional_contact_properties: filter_string_values(
                &subject.contact.additional_properties,
            ),
            subject,
            highlight: None,
            property_editor: None,
            value_editor: None,
            error_message: None,
        }
    }

    pub fn get_subject(&self) -> Subject {
        let mut subject = self.subject.clone();
        let additional_subject_properties =
            map_string_values_to_value(&self.additional_subject_properties);
        subject
            .additional_properties
            .extend(additional_subject_properties);
        let additional_contact_properties =
            map_string_values_to_value(&self.additional_contact_properties);
        subject
            .contact
            .additional_properties
            .extend(additional_contact_properties);
        subject
    }

    fn calculate_contact_start_line(&self) -> usize {
        3 + self.additional_subject_properties.len()
    }

    fn set_error(&mut self, error_type: SubjectEditorError) {
        self.error_message = match error_type {
            SubjectEditorError::EmptyName => {
                Some(ModalMessage::new("Empty name", "Name must not be empty"))
            }
            SubjectEditorError::EmptyValue => {
                Some(ModalMessage::new("Empty value", "Value must not be empty"))
            }
            SubjectEditorError::DuplicateName => Some(ModalMessage::new(
                "Duplicate name",
                "The property already exists",
            )),
            SubjectEditorError::LockedName => Some(ModalMessage::new(
                "Locked name",
                "The property exists in the template but\ncannot be edited as it is not a string",
            )),
        }
    }

    fn insert_subject_property(&mut self, idx: usize, key: String, value: String) {
        if idx == self.additional_subject_properties.len() {
            self.additional_subject_properties
                .push((key.clone(), value.clone()));
        } else {
            self.additional_subject_properties[idx] = (key.clone(), value.clone());
        }
        self.subject
            .additional_properties
            .insert(key, serde_json::Value::String(value));
    }

    fn remove_subject_property(&mut self, idx: usize) {
        let (key, _) = self.additional_subject_properties.remove(idx);
        self.subject.additional_properties.remove(&key);
    }

    fn insert_contact_property(&mut self, idx: usize, key: String, value: String) {
        if idx == self.additional_contact_properties.len() {
            self.additional_contact_properties
                .push((key.clone(), value.clone()));
        } else {
            self.additional_contact_properties[idx] = (key.clone(), value.clone());
        }
        self.subject
            .contact
            .additional_properties
            .insert(key, serde_json::Value::String(value));
    }

    fn remove_contact_property(&mut self, idx: usize) {
        let (key, _) = self.additional_contact_properties.remove(idx);
        self.subject.contact.additional_properties.remove(&key);
    }
}

fn filter_string_values(map: &HashMap<String, serde_json::Value>) -> Vec<(String, String)> {
    map.iter()
        .filter_map(|(k, v)| {
            if v.is_string() {
                Some((k.to_owned(), v.as_str().unwrap().to_owned()))
            } else {
                None
            }
        })
        .collect()
}

fn map_string_values_to_value(map: &[(String, String)]) -> Vec<(String, serde_json::Value)> {
    map.iter()
        .map(|(k, v)| (k.to_owned(), serde_json::Value::String(v.to_owned())))
        .collect()
}

fn contains_key(vec: &[(String, String)], key: &String) -> bool {
    vec.iter().any(|(k, _)| k == key)
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
            if let Ok(status) = error_message.handle_key_event(key_event) {
                match status {
                    ComponentStatus::Active => (),
                    _ => self.error_message = None,
                }
            }
            EditorEventResult::KeepActive
        } else if let Some(value_editor) = self.value_editor.as_mut() {
            if let Ok(status) = Component::handle_key_event(value_editor, key_event) {
                match status {
                    ComponentStatus::Active => (),
                    ComponentStatus::Closed => {
                        let text = value_editor.get_text().to_owned();
                        if text.is_empty() {
                            self.error_message =
                                Some(ModalMessage::new("Empty value", "Value must not be empty"));
                        } else {
                            let highlight = self.highlight.unwrap();
                            if highlight == 1 {
                                self.subject.display_name = text;
                            } else if highlight == self.calculate_contact_start_line() + 1 {
                                self.subject.contact.email = text;
                            }
                            self.value_editor = None;
                        }
                    }
                    ComponentStatus::Escaped => self.value_editor = None,
                }
            }
            EditorEventResult::KeepActive
        } else if let Some(property_editor) = self.property_editor.as_mut() {
            if let Ok(status) = property_editor.handle_key_event(key_event) {
                match status {
                    ComponentStatus::Active => (),
                    ComponentStatus::Closed => {
                        let (name, value) = property_editor.get_property();
                        if name.is_empty() {
                            self.set_error(SubjectEditorError::EmptyName);
                        } else if value.is_empty() {
                            self.set_error(SubjectEditorError::EmptyValue);
                        } else {
                            let highlight = self.highlight.unwrap();
                            if highlight < self.calculate_contact_start_line() {
                                if self.subject.additional_properties.contains_key(&name)
                                    && !contains_key(&self.additional_subject_properties, &name)
                                {
                                    self.set_error(SubjectEditorError::LockedName);
                                }
                                let idx = highlight - 2;
                                if FIXED_SUBJECT_PROPERTY_NAMES.contains(&name.as_str()) {
                                    self.set_error(SubjectEditorError::DuplicateName);
                                } else if idx == self.additional_subject_properties.len() {
                                    if contains_key(&self.additional_subject_properties, &name) {
                                        self.set_error(SubjectEditorError::DuplicateName);
                                    } else {
                                        self.insert_subject_property(idx, name, value);
                                        self.property_editor = None;
                                    }
                                } else {
                                    let key = &self.additional_subject_properties[idx].0;
                                    if *key != name
                                        && contains_key(&self.additional_subject_properties, &name)
                                    {
                                        self.set_error(SubjectEditorError::DuplicateName);
                                    } else {
                                        self.insert_subject_property(idx, name, value);
                                        self.property_editor = None;
                                    }
                                }
                            } else {
                                if self
                                    .subject
                                    .contact
                                    .additional_properties
                                    .contains_key(&name)
                                    && !contains_key(&self.additional_contact_properties, &name)
                                {
                                    self.set_error(SubjectEditorError::LockedName);
                                }
                                let idx = highlight - self.calculate_contact_start_line() - 2;
                                if FIXED_CONTACT_PROPERTY_NAMES.contains(&name.as_str()) {
                                    self.set_error(SubjectEditorError::DuplicateName);
                                } else if idx == self.additional_contact_properties.len() {
                                    if contains_key(&self.additional_contact_properties, &name) {
                                        self.set_error(SubjectEditorError::DuplicateName);
                                    } else {
                                        self.insert_contact_property(idx, name, value);
                                        self.property_editor = None;
                                    }
                                } else {
                                    let key = &self.additional_contact_properties[idx].0;
                                    if *key != name
                                        && contains_key(&self.additional_contact_properties, &name)
                                    {
                                        self.set_error(SubjectEditorError::DuplicateName);
                                    } else {
                                        self.insert_contact_property(idx, name, value);
                                        self.property_editor = None;
                                    }
                                }
                            }
                        }
                    }
                    ComponentStatus::Escaped => self.property_editor = None,
                }
            }
            EditorEventResult::KeepActive
        } else if let Some(highlight) = self.highlight {
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
                        self.remove_subject_property(idx);
                    } else if highlight > self.calculate_contact_start_line() + 1
                        && highlight < self.calculate_render_height() - 2
                    {
                        let idx = highlight - self.calculate_contact_start_line() - 2;
                        self.remove_contact_property(idx);
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
                        let (name, value) = &self.additional_subject_properties[idx];
                        let property_editor = PropertyEditor::new(name, value);
                        self.property_editor = Some(property_editor);
                    } else if highlight > self.calculate_contact_start_line() + 1
                        && highlight < self.calculate_render_height() - 1
                    {
                        let idx = highlight - self.calculate_contact_start_line() - 2;
                        let (name, value) = &self.additional_contact_properties[idx];
                        let property_editor = PropertyEditor::new(name, value);
                        self.property_editor = Some(property_editor);
                    } else if highlight == self.calculate_contact_start_line() - 1
                        || highlight == self.calculate_render_height() - 1
                    {
                        self.property_editor = Some(Default::default());
                    }
                    EditorEventResult::KeepActive
                }
                _ => EditorEventResult::KeepActive,
            }
        } else {
            EditorEventResult::Inactive
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
        writeln!(text).unwrap();
        writeln!(text, "  Contact").unwrap();
        writeln!(text, "    Email: {}", self.subject.contact.email).unwrap();
        for (key, value) in &self.additional_contact_properties {
            writeln!(text, "    {}: {}", key, value).unwrap();
        }
        writeln!(text).unwrap();
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

    fn render_modal(&mut self, area: Rect, buf: &mut Buffer) -> Cursor {
        let mut cursor = None;
        if let Some(property_editor) = self.property_editor.as_mut() {
            cursor = property_editor.render(area, buf);
        }
        if let Some(error_message) = self.error_message.as_mut() {
            cursor = error_message.render(area, buf);
        }
        cursor
    }
}

struct PropertyEditor {
    modal_window: ModalWindow,
    name_editor: TextInput,
    value_editor: TextInput,
    multiple_choice: MultipleChoice,
}

impl PropertyEditor {
    pub fn new(name: &str, value: &str) -> Self {
        let modal_window = ModalWindow::new("Edit Property");
        let mut name_editor = TextInput::new(255, false);
        name_editor.set_text(name.to_owned());
        let mut value_editor = TextInput::new(255, false);
        value_editor.set_text(value.to_owned());
        value_editor.active = false;
        let mut multiple_choice = MultipleChoice::new(DONE_CANCEL, 0);
        multiple_choice.active = false;
        Self {
            modal_window,
            name_editor,
            value_editor,
            multiple_choice,
        }
    }

    pub fn get_property(&self) -> (String, String) {
        (
            self.name_editor.get_text().to_owned(),
            self.value_editor.get_text().to_owned(),
        )
    }
}

impl Default for PropertyEditor {
    fn default() -> Self {
        Self::new("Name", "Value")
    }
}

impl Component for PropertyEditor {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        let status = if self.name_editor.active {
            match EditorComponent::handle_key_event(&mut self.name_editor, key_event) {
                EditorEventResult::Escaped => ComponentStatus::Escaped,
                EditorEventResult::ExitTop => {
                    self.name_editor.enter_from_top();
                    ComponentStatus::Active
                }
                EditorEventResult::Closed | EditorEventResult::ExitBottom => {
                    self.name_editor.active = false;
                    self.value_editor.enter_from_top();
                    ComponentStatus::Active
                }
                _ => ComponentStatus::Active,
            }
        } else if self.value_editor.active {
            match EditorComponent::handle_key_event(&mut self.value_editor, key_event) {
                EditorEventResult::Escaped => ComponentStatus::Escaped,
                EditorEventResult::ExitTop => {
                    self.name_editor.enter_from_below();
                    ComponentStatus::Active
                }
                EditorEventResult::Closed | EditorEventResult::ExitBottom => {
                    self.value_editor.active = false;
                    self.multiple_choice.enter_from_top();
                    ComponentStatus::Active
                }
                _ => ComponentStatus::Active,
            }
        } else {
            match EditorComponent::handle_key_event(&mut self.multiple_choice, key_event) {
                EditorEventResult::Escaped => ComponentStatus::Escaped,
                EditorEventResult::Closed => {
                    if self.multiple_choice.get_selected() == DONE_CANCEL[0] {
                        ComponentStatus::Closed
                    } else {
                        ComponentStatus::Escaped
                    }
                }
                EditorEventResult::ExitTop => {
                    self.value_editor.enter_from_below();
                    ComponentStatus::Active
                }
                EditorEventResult::ExitBottom => {
                    self.multiple_choice.active = true;
                    ComponentStatus::Active
                }
                _ => ComponentStatus::Active,
            }
        };
        Ok(status)
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Cursor {
        let height = area.height.min(6);
        let width = area.width.min(50);
        let area = self.modal_window.render(area, buf, height, width);
        let panels = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Max(3), Constraint::Max(1), Constraint::Min(0)])
            .split(area);
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Max(7), Constraint::Min(0)])
            .split(panels[0]);
        let row_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Max(1); 3]);
        let rows = row_layout.split(columns[0]);
        Paragraph::new("Name")
            .style(default_style())
            .render(rows[0], buf);
        Paragraph::new("Value")
            .style(default_style())
            .render(rows[1], buf);
        let rows = row_layout.split(columns[1]);
        let name_cursor = Component::render(&mut self.name_editor, rows[0], buf);
        let value_cursor = Component::render(&mut self.value_editor, rows[1], buf);
        let multiple_choice_cursor = Component::render(&mut self.multiple_choice, panels[1], buf);
        name_cursor.or(value_cursor).or(multiple_choice_cursor)
    }
}
