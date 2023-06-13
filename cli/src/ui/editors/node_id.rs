use super::*;

use std::str::FromStr;

use ya_client_model::NodeId;

pub struct NodeIdEditor {
    node_id: String,
    highlight: bool,
    editor: Option<TextInput>,
    parse_error: Option<ModalMessage>,
}

impl NodeIdEditor {
    pub fn new(node_id: Option<NodeId>) -> Self {
        Self {
            node_id: node_id.map(|id| id.to_string()).unwrap_or(String::from("0x0000000000000000000000000000000000000000")),
            highlight: false,
            editor: None,
            parse_error: None,
        }
    }

    pub fn get_node_id(&self) -> NodeId {
        NodeId::from_str(&self.node_id).unwrap()
    }
}

impl Default for NodeIdEditor {
    fn default() -> Self {
        Self::new(None)
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
                    ComponentStatus::Active => (),
                    _ => self.parse_error = None,
                }
                Err(_) => (),
            }
            EditorEventResult::KeepActive
        } else if let Some(editor) = self.editor.as_mut() {
            match Component::handle_key_event(editor, key_event) {
                Ok(status) => match status {
                    ComponentStatus::Active => (),
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
                Err(_) => (),
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

    fn get_error_message(&mut self) -> Option<&mut ModalMessage> {
        self.parse_error.as_mut()
    }
}
