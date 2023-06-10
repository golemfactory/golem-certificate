use super::*;

use golem_certificate::schemas::permissions::{Permissions, PermissionDetails, OutboundPermissions};
use url::Url;

pub struct PermissionsEditor {
    highlight: Option<usize>,
    permissions: Permissions,
    urls: Vec<Url>,
    url_editor: Option<TextInput>,
    parse_error: Option<ModalMessage>,
}

impl PermissionsEditor {
    pub fn new(permissions: Option<Permissions>) -> Self {
        let default_url: Url = "https://golem.network".parse().unwrap();
        let default_permissions =
            Permissions::Object(PermissionDetails {
                outbound: Some(OutboundPermissions::Urls([default_url.clone()].into()))
            });
        let mut urls = vec![default_url];
        Self {
            highlight: None,
            permissions: permissions.map(|p| match &p {
                Permissions::Object(PermissionDetails { outbound: None }) => default_permissions.clone(),
                Permissions::Object(PermissionDetails { outbound: Some(OutboundPermissions::Urls(url_set)) }) => {
                    urls = url_set.iter().map(|url| url.to_owned()).collect();
                    urls.sort();
                    p
                }
                _ => p,
            }).unwrap_or(default_permissions),
            urls,
            url_editor: None,
            parse_error: None,
        }
    }

    fn get_permissions(&self) -> Permissions {
        match &self.permissions {
            Permissions::Object(PermissionDetails { outbound: Some(OutboundPermissions::Urls(_)) }) => {
                let urls = self.urls.iter().map(|url| url.to_owned()).collect();
                Permissions::Object(PermissionDetails { outbound: Some(OutboundPermissions::Urls(urls)) })
            }
            p => p.clone(),
        }
    }
}

impl Default for PermissionsEditor {
    fn default() -> Self {
        Self::new(None)
    }
}

impl EditorComponent for PermissionsEditor {
    fn enter_from_below(&mut self) {
        self.highlight = Some(self.calculate_render_height() - 1);
    }

    fn enter_from_top(&mut self) {
        self.highlight = Some(0);
    }

    fn get_highlight(&self) -> Option<usize> {
        self.highlight.clone()
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> EditorEventResult {
        let render_height = self.calculate_render_height();
        if let Some(parse_error) = self.parse_error.as_mut() {
            match parse_error.handle_key_event(key_event) {
                Ok(status) => match status {
                    ComponentStatus::Active => {},
                    _ => self.parse_error = None,
                }
                Err(_) => {},
            }
            EditorEventResult::KeepActive
        } else if let Some(editor) = self.url_editor.as_mut() {
            match Component::handle_key_event(editor, key_event) {
                Ok(status) => match status {
                    ComponentStatus::Active => {},
                    ComponentStatus::Closed => {
                        match Url::parse(editor.get_text()) {
                            Ok(url) => {
                                let idx = self.highlight.as_ref().unwrap() - 3;
                                if idx == self.urls.len() {
                                    self.urls.push(url);
                                } else {
                                    self.urls[idx] = url;
                                }
                                self.url_editor = None;
                            },
                            Err(err) => self.parse_error = Some(ModalMessage::new("Url parse error", err.to_string())),
                        }
                    },
                    ComponentStatus::Escaped => self.url_editor = None,
                }
                Err(_) => {},
            }
            EditorEventResult::KeepActive
        } else if let Some(highlight) = self.highlight {
            match key_event.code {
                KeyCode::Esc => EditorEventResult::Escaped,
                KeyCode::Down => {
                    if highlight < render_height - 1 {
                        let inc = if highlight == 1 { 2 } else { 1 };
                        self.highlight = Some(highlight + inc);
                        EditorEventResult::KeepActive
                    } else {
                        self.highlight = None;
                        EditorEventResult::ExitBottom
                    }
                }
                KeyCode::Up => {
                    if highlight > 0 {
                        let dec = if highlight == 3 { 2 } else { 1 };
                        self.highlight = Some(highlight - dec);
                        EditorEventResult::KeepActive
                    } else {
                        self.highlight = None;
                        EditorEventResult::ExitTop
                    }
                }
                KeyCode::Enter => {
                    if highlight == 0 {
                        self.permissions = match &self.permissions {
                            Permissions::All => Permissions::Object(PermissionDetails { outbound: Some(OutboundPermissions::Unrestricted) }),
                            Permissions::Object(_) => Permissions::All,
                        };
                    } else if highlight == 1 {
                        match &mut self.permissions {
                            Permissions::All => unreachable!("permission editor internal state error"),
                            Permissions::Object(PermissionDetails { outbound }) =>
                                match outbound.as_ref().unwrap() {
                                    OutboundPermissions::Unrestricted =>
                                        outbound.insert(OutboundPermissions::Urls(Default::default())),
                                    OutboundPermissions::Urls(_) =>
                                        outbound.insert(OutboundPermissions::Unrestricted),
                                },
                        };
                    } else if highlight > 2 && highlight <= render_height {
                        let idx = highlight - 3;
                        let mut editor = TextInput::new(255, false);
                        if idx < self.urls.len() {
                            editor.set_text(self.urls[idx].to_string());
                        }
                        self.url_editor = Some(editor);
                    }
                    EditorEventResult::KeepActive
                }
                KeyCode::Delete | KeyCode::Backspace => {
                    if highlight > 2 && highlight < render_height {
                        let idx = highlight - 3;
                        self.urls.remove(idx);
                        if highlight > 3 {
                            self.highlight = Some(highlight - 1);
                        }
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
        match &self.permissions {
            Permissions::All => 1,
            Permissions::Object(PermissionDetails { outbound }) =>
                match outbound.as_ref() {
                    Some(outbound_details) => match outbound_details {
                        OutboundPermissions::Unrestricted => 2,
                        OutboundPermissions::Urls(_) => 4 + self.urls.len(),
                    }
                    None => 1,
                }
        }
    }

    fn get_text_output(&self, text: &mut String) {
        write!(text, "Permissions").unwrap();
        match &self.permissions {
            Permissions::All => writeln!(text, ": All").unwrap(),
            Permissions::Object(PermissionDetails { outbound: None }) => writeln!(text, "None").unwrap(),
            Permissions::Object(PermissionDetails { outbound: Some(outbound_details) }) => {
                writeln!(text, "").unwrap();
                write!(text, "  Outbound").unwrap();
                match outbound_details {
                    OutboundPermissions::Unrestricted => {
                        writeln!(text, ": Unrestricted").unwrap();
                    }
                    OutboundPermissions::Urls(_) => {
                        writeln!(text, "").unwrap();
                        writeln!(text, "    Urls").unwrap();
                        self.urls.iter()
                            .for_each(|url| writeln!(text, "      {}", url).unwrap());
                    }
                }
            }
        }
    }

    fn get_highlight_prefix(&self) -> Option<usize> {
        self.highlight.map(|highlight| {
            match highlight {
                0 => 0,
                1 => 2,
                _ => 6,
            }
        })
    }

    fn get_editor(&mut self) -> Option<&mut TextInput> {
        self.url_editor.as_mut()
    }

    fn get_error_message(&mut self) -> Option<&mut ModalMessage> {
        self.parse_error.as_mut()
    }

    fn get_empty_highlight_filler(&self) -> (String, String) {
        ("      ".into(), "<Add another URL>".into())
    }
}
