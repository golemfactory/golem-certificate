use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use golem_certificate::SignedCertificate;
use tui::{widgets::{ StatefulWidget }, layout::Rect};

use super::{util::{ Component, ComponentStatus, default_style, SizedComponent, Height, Width, certificate_to_string, CalculateHeight, CalculateWidth, AreaCalculators }, scrollable_text::{ ScrollableText, ScrollableTextState }};

pub struct SignedCertificateDetails {
    calculate_height: CalculateHeight,
    calculate_width: CalculateWidth,
    render_state: ScrollableTextState,
}

impl SignedCertificateDetails {
    pub fn new(
        cert: &SignedCertificate,
        indent: usize,
        detailed_signer: bool,
        (calculate_height, calculate_width): AreaCalculators,
    ) -> Self {
        let text = certificate_to_string(cert, indent, detailed_signer);
        Self { render_state: ScrollableTextState::new(text), calculate_height, calculate_width }
    }
}

impl Component for SignedCertificateDetails {
    fn render(&mut self, area: Rect, buf: &mut tui::buffer::Buffer) {
        ScrollableText::default()
            .style(default_style())
            .render(area, buf, &mut self.render_state);
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<ComponentStatus> {
        let status = match key_event.code {
            KeyCode::Esc => ComponentStatus::Escaped,
            KeyCode::Up => {
                let offset = self.render_state.offset_mut();
                *offset = offset.saturating_sub(1);
                ComponentStatus::Active
            }
            KeyCode::Down => {
                let offset = self.render_state.offset_mut();
                *offset = offset.saturating_add(1);
                ComponentStatus::Active
            }
            _ => ComponentStatus::Active
        };
        Ok(status)
    }
}

impl SizedComponent for SignedCertificateDetails {
    fn get_render_size(&self, area: Rect) -> (Height, Width) {
        ((self.calculate_height)(area.height), (self.calculate_width)(area.width))
    }
}
