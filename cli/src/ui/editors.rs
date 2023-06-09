use std::fmt::Write;

use crossterm::event::{KeyEvent, KeyCode};
use tui::{layout::Rect, buffer::Buffer, widgets::{Clear, Paragraph, Widget}, text::Span};

use super::{
    component::*,
    modal::ModalMessage,
    text_input::TextInput,
    util::{default_style, highlight_style},
};

pub enum EditorEventResult {
    ExitTop,
    ExitBottom,
    KeepActive,
    Escaped,
    Inactive,
}

pub enum Editor {
    NodeId,
    Permissions,
    ValidityPeriod,
}

pub use node_id::NodeIdEditor;
pub use permission::PermissionEditor;
pub use validity_period::ValidityPeriodEditor;

pub trait EditorComponent {
    fn enter_from_below(&mut self);
    fn enter_from_top(&mut self);
    fn get_highlight(&self) -> Option<usize>;
    fn handle_key_event(&mut self, key_event: KeyEvent) -> EditorEventResult;
    fn calculate_render_height(&self) -> usize;
    fn get_text_output(&self, text: &mut String);
    fn get_highlight_prefix(&self) -> Option<usize>;
    fn get_editor(&mut self) -> Option<&mut TextInput>;
    fn get_parse_error(&mut self) -> Option<&mut ModalMessage>;

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Cursor {
        let text = {
            let mut text = String::new();
            self.get_text_output(&mut text);
            text
        };
        if self.get_editor().is_some() || self.get_highlight().is_none() {
            Paragraph::new(text)
                .alignment(tui::layout::Alignment::Left)
                .style(default_style())
                .render(area, buf);

            let highlight = self.get_highlight();
            let highlight_prefix = self.get_highlight_prefix();
            if let Some(url_editor) = self.get_editor() {
                let prefix = highlight_prefix.expect("Cannot have text input active without highlight in the component") as u16;
                let editor_area = Rect {
                    x: area.x + prefix,
                    y: area.y + highlight.expect("Cannot have text input active without highlight in the component") as u16,
                    width: area.width.saturating_sub(prefix),
                    height: 1.min(area.height),
                };
                Clear.render(editor_area, buf);
                let editor_cursor = url_editor.render(editor_area, buf);
                if let Some(parse_error) = self.get_parse_error() {
                    parse_error.render(area, buf)
                } else {
                    editor_cursor
                }
            } else {
                None
            }
        } else {
            render_with_highlight(&text, self.get_highlight().unwrap(), self.get_highlight_prefix().unwrap(), area, buf);
            None
        }
    }
}

fn render_with_highlight(text: &String, highlight: usize, prefix: usize, area: Rect, buf: &mut Buffer) {
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
        let (highlight_prefix, highlighted_text) = {
            if highlighted.is_empty() {
                ("      ".into(), "<Add another URL>".into())
            } else {
                let highlight_prefix = highlighted.chars().take(prefix).collect::<String>();
                let highlighted_text = highlighted.chars().skip(prefix).collect::<String>();
                (highlight_prefix, highlighted_text)
            }
        };
        let text_area = Rect {
            x: area.x + prefix as u16,
            y: area.y,
            width: area.width.saturating_sub(prefix as u16),
            height: area.height,
        };
        Paragraph::new(highlight_prefix)
            .alignment(tui::layout::Alignment::Left)
            .style(default_style())
            .render(area, buf);
        let span = Span::styled(highlighted_text, highlight_style());
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

mod node_id {
    use std::str::FromStr;

    use super::*;

    use ya_client_model::NodeId;

    pub struct NodeIdEditor {
        node_id: String,
        highlight: bool,
        editor: Option<TextInput>,
        parse_error: Option<ModalMessage>,
    }

    impl NodeIdEditor {
        pub fn new() -> Self {
            Self {
                node_id: String::from("0x0000000000000000000000000000000000000000"),
                highlight: false,
                editor: None,
                parse_error: None,
            }
        }

        pub fn get_node_id(&self) -> NodeId {
            NodeId::from_str(&self.node_id).unwrap()
        }
    }

    impl EditorComponent for NodeIdEditor {
        fn enter_from_below(&mut self) {
            self.highlight = true;
        }

        fn enter_from_top(&mut self) {
            self.highlight = true;
        }

        fn get_highlight(&self) -> Option<usize> {
            if self.highlight {
                Some(0)
            } else {
                None
            }
        }

        fn handle_key_event(&mut self, key_event: KeyEvent) -> EditorEventResult {
            if let Some(parse_error) = self.parse_error.as_mut() {
                match parse_error.handle_key_event(key_event) {
                    Ok(status) => match status {
                        ComponentStatus::Active => {},
                        _ => self.parse_error = None,
                    }
                    Err(_) => {},
                }
                EditorEventResult::KeepActive
            } else if let Some(editor) = self.editor.as_mut() {
                match editor.handle_key_event(key_event) {
                    Ok(status) => match status {
                        ComponentStatus::Active => {},
                        ComponentStatus::Closed => {
                            let node_id_string = {
                                let mut s = editor.get_text().to_owned();
                                if s.len() == 40 {
                                    s = format!("0x{}", s);
                                }
                                s
                            };
                            match NodeId::from_str(&node_id_string) {
                                Ok(_) => {
                                    self.node_id = node_id_string;
                                    self.editor = None;
                                }
                                Err(err) => {
                                    let parse_error = ModalMessage::new("NodeId parse error", err.to_string());
                                    self.parse_error = Some(parse_error);
                                },
                            }
                        },
                        ComponentStatus::Escaped => self.editor = None,
                    }
                    Err(_) => {},
                }
                EditorEventResult::KeepActive
            } else if self.highlight {
                match key_event.code {
                    KeyCode::Esc => EditorEventResult::Escaped,
                    KeyCode::Down => {
                        self.highlight = false;
                        EditorEventResult::ExitBottom
                    }
                    KeyCode::Up => {
                        self.highlight = false;
                        EditorEventResult::ExitTop
                    }
                    KeyCode::Enter => {
                        let mut editor = TextInput::new(42, false);
                        editor.set_text(self.node_id.clone());
                        self.editor = Some(editor);
                        EditorEventResult::KeepActive
                    }
                    _ => EditorEventResult::KeepActive,
                }
            } else {
                EditorEventResult::Inactive
            }
        }

        fn calculate_render_height(&self) -> usize {
            1
        }

        fn get_text_output(&self, text: &mut String) {
            writeln!(text, "NodeId: {}", self.node_id).unwrap();
        }

        fn get_highlight_prefix(&self) -> Option<usize> {
            if self.highlight {
                Some(8)
            } else {
                None
            }
        }

        fn get_editor(&mut self) -> Option<&mut TextInput> {
            self.editor.as_mut()
        }

        fn get_parse_error(&mut self) -> Option<&mut ModalMessage> {
            self.parse_error.as_mut()
        }
    }
}

pub mod permission {
    use super::*;

    use std::collections::HashSet;

    use golem_certificate::schemas::permissions::{Permissions, PermissionDetails, OutboundPermissions};
    use url::Url;

    pub struct PermissionEditor {
        highlight: Option<usize>,
        permissions: Permissions,
        urls: Vec<Url>,
        url_editor: Option<TextInput>,
        parse_error: Option<ModalMessage>,
    }

    impl PermissionEditor {
        pub fn new(permissions: Option<Permissions>) -> Self {
            let default_permissions =
                Permissions::Object(PermissionDetails {
                    outbound: Some(OutboundPermissions::Urls(HashSet::new()))
                });
            let mut urls = vec![];
            Self {
                highlight: None,
                permissions: permissions.map(|p| match &p {
                    Permissions::Object(PermissionDetails { outbound: None }) => default_permissions.clone(),
                    Permissions::Object(PermissionDetails { outbound: Some(OutboundPermissions::Urls(url_set)) }) => {
                        url_set.iter().for_each(|url| urls.push(url.to_owned()));
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

    impl EditorComponent for PermissionEditor {
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

        fn get_parse_error(&mut self) -> Option<&mut ModalMessage> {
            self.parse_error.as_mut()
        }
    }
}

pub mod validity_period {
    use crate::ui::text_input::TextInput;

    use super::*;

    use chrono::{DateTime, Utc};
    use golem_certificate::schemas::validity_period::ValidityPeriod;

    pub struct ValidityPeriodEditor {
        not_before: DateTime<Utc>,
        not_after: DateTime<Utc>,
        highlight: Option<usize>,
        date_editor: Option<TextInput>,
        parse_error: Option<ModalMessage>,
    }

    impl ValidityPeriodEditor {
        pub fn new(validity_period: Option<ValidityPeriod>) -> Self {
            let (not_before, not_after) = match validity_period {
                Some(ValidityPeriod { not_before, not_after }) => (not_before, not_after),
                None => (Utc::now(), Utc::now()),
            };
            Self {
                not_before,
                not_after,
                highlight: None,
                date_editor: None,
                parse_error: None,
            }
        }

        pub fn get_validity_period(&self) -> ValidityPeriod {
            ValidityPeriod {
                not_before: self.not_before.clone(),
                not_after: self.not_after.clone(),
            }
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
                match date_editor.handle_key_event(key_event) {
                    Ok(status) => match status {
                        ComponentStatus::Active => {},
                        ComponentStatus::Closed => {
                            match date_editor.get_text().parse::<DateTime<Utc>>() {
                                Ok(datetime) => {
                                    if self.highlight.unwrap() == 1 {
                                        self.not_before = datetime;
                                    } else {
                                        self.not_after = datetime;
                                    }
                                    self.date_editor = None;
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
                            editor.set_text(self.not_before.to_string());
                        } else {
                            editor.set_text(self.not_after.to_string());
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

        fn get_parse_error(&mut self) -> Option<&mut ModalMessage> {
            self.parse_error.as_mut()
        }
    }
}
