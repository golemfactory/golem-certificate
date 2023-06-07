use std::fmt::Write;


use crossterm::event::{KeyEvent, KeyCode};
use golem_certificate::schemas::{certificate::key_usage::KeyUsage};
use tui::{layout::Rect, buffer::Buffer};

use super::component::*;

pub struct X {}

struct Tree {
    entries: Vec<TreeEntry>,
}

enum TreeValue {
    KeyUsage(Vec<KeyUsage>),
    Object(Tree),
    Text(String),
}

struct TreeEntry {
    name: String,
    value: TreeValue,
}

pub enum EditorEventResult {
    ExitTop,
    ExitBottom,
    KeepActive,
    Inactive,
}

pub trait EditorComponent {
    fn enter_from_below(&mut self);
    fn enter_from_top(&mut self);
    fn get_highlighted_line(&self) -> Option<usize>;
    fn handle_key_event(&mut self, key_event: KeyEvent) -> EditorEventResult;
    fn calculate_render_height(&self) -> usize ;
    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Cursor;
}

pub mod permission {
    use super::*;

    use std::collections::HashSet;

    use golem_certificate::schemas::permissions::{Permissions, PermissionDetails, OutboundPermissions};
    use tui::{widgets::{Paragraph, Widget, Clear}, style::Modifier, text::Span};
    use url::Url;

    use crate::ui::{text_input::TextInput, modal::ModalMessage, util::{default_style, highlight_style}};

    pub struct PermissionEditor {
        highlight: Option<usize>,
        pub permissions: Permissions,
        urls: Vec<Url>,
        url_editor: Option<TextInput>,
        url_error: Option<ModalMessage>,
    }

    impl PermissionEditor {
        pub fn new(permissions: Option<Permissions>) -> Self {
            let default_permissions =
                Permissions::Object(PermissionDetails {
                    outbound: Some(OutboundPermissions::Urls(HashSet::new()))
                });
            Self {
                highlight: None,
                permissions: permissions.unwrap_or(default_permissions),
                urls: vec![],
                url_editor: None,
                url_error: None,
            }
        }

        fn get_permissions_output(&self) -> String {
            let mut output = String::new();
            write!(&mut output, "Permissions").unwrap();
            match &self.permissions {
                Permissions::All => writeln!(&mut output, ": All").unwrap(),
                Permissions::Object(PermissionDetails { outbound }) => match outbound {
                    Some(outbound_details) => {
                        writeln!(&mut output, "").unwrap();
                        write!(&mut output, "  Outbound").unwrap();
                        match outbound_details {
                            OutboundPermissions::Unrestricted => {
                                writeln!(&mut output, ": Unrestricted").unwrap();
                            }
                            OutboundPermissions::Urls(_) => {
                                writeln!(&mut output, "").unwrap();
                                writeln!(&mut output, "    Urls").unwrap();
                                self.urls.iter()
                                    .for_each(|url| writeln!(&mut output, "      {}", url).unwrap());
                            },
                        }
                    }
                    None => writeln!(&mut output, "None").unwrap(),
                }
            }
            output
        }
    }

    impl EditorComponent for PermissionEditor {
        fn enter_from_below(&mut self) {
            self.highlight = Some(self.calculate_render_height() - 1);
        }

        fn enter_from_top(&mut self) {
            self.highlight = Some(0);
        }

        fn get_highlighted_line(&self) -> Option<usize> {
            self.highlight.clone()
        }

        fn handle_key_event(&mut self, key_event: KeyEvent) -> EditorEventResult {
            let render_height = self.calculate_render_height();
            if let Some(error_message) = self.url_error.as_mut() {
                match error_message.handle_key_event(key_event) {
                    Ok(status) => match status {
                        ComponentStatus::Active => {},
                        _ => self.url_error = None,
                    }
                    Err(_) => {},
                }
                EditorEventResult::KeepActive
            } else if let Some(editor) = self.url_editor.as_mut() {
                match editor.handle_key_event(key_event) {
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
                                Err(err) => self.url_error = Some(ModalMessage::new("Url parse error", err.to_string())),
                            }
                        },
                        ComponentStatus::Escaped => self.url_editor = None,
                    }
                    Err(_) => {},
                }
                EditorEventResult::KeepActive
            } else if let Some(highlight) = self.highlight.as_ref() {
                let highlight = highlight.clone();
                match key_event.code {
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
                                Permissions::Object(PermissionDetails { outbound }) => match outbound.as_ref().unwrap() {
                                    OutboundPermissions::Unrestricted => {
                                        let urls = self.urls.iter().map(|url| url.to_owned()).collect::<HashSet<_>>();
                                        outbound.insert(OutboundPermissions::Urls(urls))
                                    },
                                    OutboundPermissions::Urls(_) => outbound.insert(OutboundPermissions::Unrestricted),
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

        fn render(&mut self, area: Rect, buf: &mut Buffer) -> Cursor {
            let text = self.get_permissions_output();
            if let Some(url_editor) = self.url_editor.as_mut() {
                Paragraph::new(text)
                    .alignment(tui::layout::Alignment::Left)
                    .style(default_style())
                    .render(area, buf);

                let editor_area = Rect {
                    x: area.x + 6,
                    y: area.y + self.highlight.expect("Cannot have text input active without highlight in the component") as u16,
                    width: area.width.saturating_sub(6),
                    height: 1.min(area.height),
                };
                Clear.render(editor_area, buf);
                let cursor = url_editor.render(editor_area, buf);
                if let Some(url_error) = self.url_error.as_mut() {
                    url_error.render(area, buf);
                }
                cursor
            } else {
                if let Some(highlight) = self.highlight {
                    let pre = text.lines().take(highlight).collect::<Vec<_>>().join("\n");
                    let highlighted = text.lines().skip(highlight).take(1).collect::<String>();
                    let post = text.lines().skip(highlight + 1).collect::<Vec<_>>().join("\n");
                    let highlight_area = adjust_render_area(&area, &pre);
                    Paragraph::new(pre)
                        .alignment(tui::layout::Alignment::Left)
                        .style(default_style())
                        .render(area, buf);
                    if let Some(mut area) = highlight_area {
                        let post_area = adjust_render_area(&area, &highlighted);
                        area.height = area.height.min(1);
                        let (spaces, highlighted_text) = {
                            if highlighted.is_empty() {
                                ("      ".into(), "<Add another URL>".into())
                            } else {
                                let spaces = highlighted.chars().take_while(|&c| c == ' ').collect::<String>();
                                let highlighted_text = highlighted.chars().skip(spaces.len()).collect::<String>();
                                (spaces, highlighted_text)
                            }
                        };
                        let text_area = Rect {
                            x: area.x + spaces.len() as u16,
                            y: area.y,
                            width: area.width.saturating_sub(spaces.len() as u16),
                            height: area.height,
                        };
                        Paragraph::new(spaces)
                            .alignment(tui::layout::Alignment::Left)
                            .style(default_style())
                            .render(area, buf);
                        let span =
                            Span::styled(highlighted_text, highlight_style());
                        Paragraph::new(span)
                            .alignment(tui::layout::Alignment::Left)
                            .render(text_area, buf);
                        if let Some(area) = post_area {
                            Paragraph::new(post)
                                .alignment(tui::layout::Alignment::Left)
                                .style(default_style())
                                .render(area, buf);
                        }
                    } else {
                        unreachable!("Highlighted part is not within render area");
                    }
                } else {
                    Paragraph::new(text)
                        .alignment(tui::layout::Alignment::Left)
                        .style(default_style())
                        .render(area, buf);
                }
                None
            }
        }
    }
}

fn adjust_render_area(area: &Rect, text: &String) -> Option<Rect> {
    let text_height = text.lines().count() as u16;
    let adjusted_area = Rect {
        x: area.x,
        y: area.y + text_height,
        width: area.width,
        height: area.height.saturating_sub(text_height),
    };
    if adjusted_area.height > 0 {
        Some(adjusted_area)
    } else {
        None
    }
}
