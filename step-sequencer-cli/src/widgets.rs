use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

pub(crate) struct SolidBox {
    color: Color,
}

impl SolidBox {
    pub fn color(color: Color) -> Self {
        Self { color }
    }
}

impl Widget for SolidBox {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, Style::new().bg(self.color));
    }
}
