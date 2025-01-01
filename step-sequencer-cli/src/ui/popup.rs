use ratatui::{
    layout::{Constraint, Flex, Layout},
    style::Stylize,
    widgets::{Block, Clear, Paragraph, Widget},
};

pub(crate) struct Popup {
    text: String,
}

impl Popup {
    pub fn new(text: &str) -> Self {
        Popup {
            text: text.to_string(),
        }
    }
}

impl Widget for Popup {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let vertical = Layout::vertical([Constraint::Percentage(50)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(50)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        let block = Block::bordered().title("Help").white().on_blue();
        // This is required to clear graphemes drawn by other widgets such as bordered blocks
        Clear.render(area, buf);
        Paragraph::new(self.text)
            .block(block)
            .white()
            .render(area, buf);
    }
}
