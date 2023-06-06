use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use golem_certificate::SignedNodeDescriptor;
use tui::{buffer::Buffer, layout::Rect, widgets::StatefulWidget};

use super::{
    component::*,
    display_details::node_descriptor_to_string,
    scrollable_text::{ScrollableText, ScrollableTextState},
    util::{
        default_style, AreaCalculators, CalculateHeight, CalculateWidth,
    },
};

pub struct SignedNodeDescriptorDetails {
    calculate_height: CalculateHeight,
    calculate_width: CalculateWidth,
    render_state: ScrollableTextState,
}

impl SignedNodeDescriptorDetails {
    pub fn new(
        node_descriptor: &SignedNodeDescriptor,
        indent: usize,
        detailed_signer: bool,
        (calculate_height, calculate_width): AreaCalculators,
    ) -> Self {
        let text = node_descriptor_to_string(node_descriptor, indent, detailed_signer);
        Self {
            render_state: ScrollableTextState::new(text),
            calculate_height,
            calculate_width,
        }
    }

    fn scrolling_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Up => {
                let offset = self.render_state.offset_mut();
                *offset = offset.saturating_sub(1);
            }
            KeyCode::Down => {
                let offset = self.render_state.offset_mut();
                *offset = offset.saturating_add(1);
            }
            _ => (),
        }
    }
}

impl Component for SignedNodeDescriptorDetails {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        let status = match key_event.code {
            KeyCode::Esc => ComponentStatus::Escaped,
            _ => {
                self.scrolling_key_event(key_event);
                ComponentStatus::Active
            }
        };
        Ok(status)
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) -> Cursor {
        ScrollableText::default()
            .style(default_style())
            .render(area, buf, &mut self.render_state);
        None
    }
}

impl SizedComponent for SignedNodeDescriptorDetails {
    fn get_render_size(&self, area: Rect) -> (Height, Width) {
        (
            (self.calculate_height)(area.height),
            (self.calculate_width)(area.width),
        )
    }
}
