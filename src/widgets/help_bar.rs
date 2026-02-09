use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Line, Span, Text},
    widgets::{Paragraph, Widget},
};

use crate::app::InputMode;
use crate::constants::DEFAULT_THEME;

pub struct HelpBar {
    mode: InputMode,
    width: u16,
}

impl HelpBar {
    pub fn new(mode: InputMode, width: u16) -> Self {
        Self { mode, width }
    }
}

impl Widget for HelpBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let key = Style::default().fg(DEFAULT_THEME.keybind).bold();
        let label = Style::default().fg(DEFAULT_THEME.help_label);

        let spans: Vec<Span> = match self.mode {
            InputMode::Normal if self.width < 80 => vec![
                Span::styled("q", key),
                Span::styled(" quit ", label),
                Span::styled("e", key),
                Span::styled(" edit ", label),
                Span::styled("h", key),
                Span::styled(" help ", label),
                Span::styled("f", key),
                Span::styled(" freeze ", label),
                Span::styled("1/2/3", key),
                Span::styled(" view ", label),
                Span::styled("t", key),
                Span::styled(" time ", label),
                Span::styled("c", key),
                Span::styled(" cmds ", label),
                Span::styled("s", key),
                Span::styled(" sync ", label),
                Span::styled("l/L", key),
                Span::styled(" export", label),
            ],
            InputMode::Normal => vec![
                Span::styled("q", key),
                Span::styled(" quit · ", label),
                Span::styled("e", key),
                Span::styled(" edit · ", label),
                Span::styled("h", key),
                Span::styled(" help · ", label),
                Span::styled("f", key),
                Span::styled(" freeze · ", label),
                Span::styled("1/2/3", key),
                Span::styled(" view · ", label),
                Span::styled("t", key),
                Span::styled(" time · ", label),
                Span::styled("r/R", key),
                Span::styled(" clear · ", label),
                Span::styled("C-r", key),
                Span::styled(" clr graphs · ", label),
                Span::styled("c", key),
                Span::styled(" cmds · ", label),
                Span::styled("s", key),
                Span::styled(" sync · ", label),
                Span::styled("l/L", key),
                Span::styled(" export", label),
            ],
            InputMode::Editing => vec![
                Span::styled("Press ", label),
                Span::styled("Esc", key),
                Span::styled(" to stop editing, ", label),
                Span::styled("Enter", key),
                Span::styled(" to send the message", label),
            ],
            InputMode::Help => vec![
                Span::styled("Press ", label),
                Span::styled("Esc", key),
                Span::styled(" to close", label),
            ],
            InputMode::CommandPalette => vec![
                Span::styled("↑↓", key),
                Span::styled(" navigate · ", label),
                Span::styled("Enter", key),
                Span::styled(" select · ", label),
                Span::styled("Esc", key),
                Span::styled(" close · ", label),
                Span::styled("type to filter", label),
            ],
        };

        let text = Text::from(Line::from(spans));
        Paragraph::new(text).render(area, buf);
    }
}
