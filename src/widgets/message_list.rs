use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, List, ListItem, ListState, StatefulWidget},
};

use crate::constants::DEFAULT_THEME;
use crate::serial::SerialMessage;

pub struct MessageList<'a> {
    messages: &'a [SerialMessage],
    title: &'a str,
    show_timestamps: bool,
    border_color: Color,
}

impl<'a> MessageList<'a> {
    pub fn new(
        messages: &'a [SerialMessage],
        title: &'a str,
        show_timestamps: bool,
        border_color: Color,
    ) -> Self {
        Self {
            messages,
            title,
            show_timestamps,
            border_color,
        }
    }
}

impl StatefulWidget for MessageList<'_> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let enum_style = Style::default().fg(DEFAULT_THEME.enum_text);

        let items: Vec<ListItem> = self
            .messages
            .iter()
            .enumerate()
            .map(|(i, m)| {
                if self.show_timestamps {
                    let ts = m.timestamp.format("%H:%M:%S");
                    ListItem::new(Line::from(vec![
                        Span::styled(format!("{i}: "), enum_style),
                        Span::raw(format!("[{ts}] {}", m.message)),
                    ]))
                } else {
                    ListItem::new(Line::from(vec![
                        Span::styled(format!("{i}: "), enum_style),
                        Span::raw(&m.message),
                    ]))
                }
            })
            .collect();

        let list = List::new(items).block(
            Block::bordered()
                .title(self.title.bold())
                .border_style(Style::default().fg(self.border_color)),
        );

        StatefulWidget::render(list, area, buf, state);
    }
}
