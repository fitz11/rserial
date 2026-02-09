use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Style, Stylize},
    widgets::{Block, Paragraph, Widget},
};

use crate::app::InputMode;
use crate::constants::DEFAULT_THEME;

pub struct InputField<'a> {
    input: &'a str,
    mode: InputMode,
}

impl<'a> InputField<'a> {
    pub fn new(input: &'a str, mode: InputMode) -> Self {
        Self { input, mode }
    }
}

impl Widget for InputField<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = match self.mode {
            InputMode::Editing => Block::bordered()
                .title("Input".bold())
                .border_style(Style::default().fg(DEFAULT_THEME.input_active)),
            _ => Block::bordered().title("Input".bold()),
        };

        let style = match self.mode {
            InputMode::Editing => Style::default().fg(DEFAULT_THEME.input_active),
            _ => Style::default(),
        };

        Paragraph::new(self.input)
            .style(style)
            .block(block)
            .render(area, buf);
    }
}
