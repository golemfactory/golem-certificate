use tui::{widgets::{Paragraph, StatefulWidget, Widget}, layout::{Rect, Alignment}, buffer::Buffer, style::Style};

pub struct ScrollableTextState {
    lines: Vec<String>,
    offset: usize,
}

#[allow(dead_code)]
impl ScrollableTextState {
    pub fn new<S: Into<String>>(text: S) -> Self {
        let lines = text.into().lines().map(|l| l.to_owned()).collect();
        let offset = 0;
        Self { lines, offset }
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn offset_mut(&mut self) -> &mut usize {
        &mut self.offset
    }

    fn render_range(&self, height: usize) -> (usize, usize) {
        let last = self.lines.len() - 1;
        let mut start = self.offset.min(last);
        let mut end = start + 1;
        let mut current_height = 1;
        while current_height < height && end < self.lines.len() {
            end += 1;
            current_height += 1;
        }
        while current_height < height && start > 0 {
            start -= 1;
            current_height += 1;
        }
        (start, end)
    }
}

pub struct ScrollableText {
    alignment: Alignment,
    style: Style,
}

#[allow(dead_code)]
impl ScrollableText {
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Default for ScrollableText {
    fn default() -> Self {
        Self {
            alignment: Alignment::Left,
            style: Default::default(),
        }
    }
}

impl StatefulWidget for ScrollableText {
    type State = ScrollableTextState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let (start, end) = state.render_range(area.height as usize);
        *state.offset_mut() = start;
        let rendered_text = state.lines.iter().skip(start).take(end - start).map(|l| l.to_owned()).collect::<Vec<_>>();
        Paragraph::new(rendered_text.join("\n"))
            .alignment(self.alignment)
            .style(self.style)
            .render(area, buf);
    }
}
