use super::*;

use std::collections::HashSet;

use golem_certificate::schemas::certificate::key_usage::{KeyUsage, Usage};

const KEY_USAGE_ORDER: [(Usage, &str); 3] = [
    (Usage::SignCertificate, "Sign certificate"),
    (Usage::SignManifest, "Sign manifest"),
    (Usage::SignNode, "Sign node"),
];

pub struct KeyUsageEditor {
    highlight: Option<usize>,
    key_usage: KeyUsage,
    usage: HashSet<Usage>,
}

impl KeyUsageEditor {
    pub fn new(key_usage: Option<KeyUsage>) -> Self {
        let key_usage = key_usage.unwrap_or(KeyUsage::Limited([Usage::SignNode].into()));
        let usage = match &key_usage {
            KeyUsage::All => Default::default(),
            KeyUsage::Limited(usage) => usage.iter().map(|usage| usage.to_owned()).collect(),
        };
        Self {
            highlight: None,
            key_usage,
            usage,
        }
    }

    pub fn get_key_usage(&self) -> KeyUsage {
        match &self.key_usage {
            KeyUsage::All => KeyUsage::All,
            KeyUsage::Limited(_) => KeyUsage::Limited(self.usage.iter().map(|usage| usage.to_owned()).collect()),
        }
    }
}

impl Default for KeyUsageEditor {
    fn default() -> Self {
        Self::new(None)
    }
}

impl EditorComponent for KeyUsageEditor {
    fn enter_from_below(&mut self) {
        self.highlight = Some(self.calculate_render_height() - 1);
    }

    fn enter_from_top(&mut self) {
        self.highlight = Some(0);
    }

    fn get_highlight(&self) -> Option<usize> {
        self.highlight
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> EditorEventResult {
        if let Some(highlight) = self.highlight {
            match key_event.code {
                KeyCode::Esc => EditorEventResult::Escaped,
                KeyCode::Up =>
                    if highlight > 0 {
                        self.highlight = Some(highlight - 1);
                        EditorEventResult::KeepActive
                    } else {
                        self.highlight = None;
                        EditorEventResult::ExitTop
                    }
                KeyCode::Down =>
                    if highlight < self.calculate_render_height() - 1 {
                        self.highlight = Some(highlight + 1);
                        EditorEventResult::KeepActive
                    } else {
                        self.highlight = None;
                        EditorEventResult::ExitBottom
                    }
                KeyCode::Enter => {
                        if highlight == 0 {
                            self.key_usage = match &self.key_usage {
                                KeyUsage::All => KeyUsage::Limited(Default::default()),
                                KeyUsage::Limited(_) => KeyUsage::All,
                            };
                        } else if highlight > 0 && highlight < self.calculate_render_height() {
                            let idx = highlight - 1;
                            let usage = KEY_USAGE_ORDER[idx].0.clone();
                            if self.usage.contains(&usage) {
                                self.usage.remove(&usage);
                            } else {
                                self.usage.insert(usage);
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
        match &self.key_usage {
            KeyUsage::All => 1,
            KeyUsage::Limited(_) => 1 + KEY_USAGE_ORDER.len(),
        }
    }

    fn get_text_output(&self, text: &mut String) {
        write!(text, "Key usage").unwrap();
        match self.key_usage {
            KeyUsage::All => write!(text, ": All").unwrap(),
            KeyUsage::Limited(_) => {
                writeln!(text, "").unwrap();
                KEY_USAGE_ORDER.iter().for_each(|(usage, usage_str)| {
                    writeln!(text, "  [{}] {}", if self.usage.contains(usage) { "*" } else { " " }, usage_str).unwrap();
                });
            },
        }
    }

    fn get_highlight_prefix(&self) -> Option<usize> {
        self.highlight.map(|highlight| if highlight == 0 { 0 } else { 2 })
    }

    fn get_editor(&mut self) -> Option<&mut TextInput> {
        None
    }

    fn get_error_message(&mut self) -> Option<&mut ModalMessage> {
        None
    }
}
