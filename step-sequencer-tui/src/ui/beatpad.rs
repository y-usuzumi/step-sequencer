use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::Widget,
};

pub(crate) struct BeatPad {
    color: Color,
    is_active: bool,
}

impl BeatPad {
    pub fn color(color: Color) -> Self {
        Self {
            color,
            is_active: false,
        }
    }

    pub fn set_active(mut self, is_active: bool) -> Self {
        self.is_active = is_active;
        self
    }
}

impl Widget for BeatPad {
    fn render(self, mut area: Rect, buf: &mut Buffer) {
        if !self.is_active {
            area = Rect {
                x: area.x + 1,
                y: area.y + 1,
                width: if area.width > 2 { area.width - 2 } else { 0 },
                height: if area.height > 2 { area.height - 2 } else { 0 },
            };
        }
        buf.set_style(area, Style::new().bg(self.color));
    }
}
