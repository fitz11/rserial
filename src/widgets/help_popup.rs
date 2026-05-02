use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph, Wrap},
};

use crate::constants::DEFAULT_THEME;

pub struct HelpPopup;

impl HelpPopup {
    pub fn render_overlay(frame: &mut Frame) {
        let key = Style::default().fg(DEFAULT_THEME.keybind).bold();
        let label = Style::default().fg(DEFAULT_THEME.help_label);

        let help_text = vec![
            Line::from("Normal Mode".bold()),
            Line::from(vec![
                Span::styled("  q", key),
                Span::styled("  Quit", label),
            ]),
            Line::from(vec![
                Span::styled("  e", key),
                Span::styled("  Edit mode", label),
            ]),
            Line::from(vec![
                Span::styled("  h", key),
                Span::styled("  Toggle help", label),
            ]),
            Line::from(vec![
                Span::styled("  f", key),
                Span::styled("  Freeze/unfreeze", label),
            ]),
            Line::from(vec![
                Span::styled("  1", key),
                Span::styled("  Received messages", label),
            ]),
            Line::from(vec![
                Span::styled("  2", key),
                Span::styled("  Sent messages", label),
            ]),
            Line::from(vec![
                Span::styled("  3", key),
                Span::styled("  Graphs view", label),
            ]),
            Line::from(vec![
                Span::styled("  t", key),
                Span::styled("  Toggle timestamps", label),
            ]),
            Line::from(vec![
                Span::styled("  r", key),
                Span::styled("  Clear received messages", label),
            ]),
            Line::from(vec![
                Span::styled("  R", key),
                Span::styled("  Clear sent messages", label),
            ]),
            Line::from(vec![
                Span::styled("  Ctrl+r", key),
                Span::styled("  Clear graphs", label),
            ]),
            Line::from(vec![
                Span::styled("  c", key),
                Span::styled("  Command palette", label),
            ]),
            Line::from(vec![
                Span::styled("  s", key),
                Span::styled("  Re-sync commands", label),
            ]),
            Line::from(vec![
                Span::styled("  l", key),
                Span::styled("  Export current view", label),
            ]),
            Line::from(vec![
                Span::styled("  L", key),
                Span::styled("  Export all buffers", label),
            ]),
            Line::from(vec![
                Span::styled("  y", key),
                Span::styled("  Lock/unlock Y-axis", label),
            ]),
            Line::from(vec![
                Span::styled("  x", key),
                Span::styled("  Disconnect", label),
            ]),
            Line::from(""),
            Line::from("Editing Mode".bold()),
            Line::from(vec![
                Span::styled("  Esc", key),
                Span::styled("        Back to normal", label),
            ]),
            Line::from(vec![
                Span::styled("  Enter", key),
                Span::styled("      Send message", label),
            ]),
            Line::from(vec![
                Span::styled("  Backspace", key),
                Span::styled("  Delete character", label),
            ]),
            Line::from(vec![
                Span::styled("  ←/→", key),
                Span::styled("        Move cursor", label),
            ]),
        ];

        let popup_block = Block::bordered()
            .title("Help".bold())
            .border_style(Style::default().fg(DEFAULT_THEME.help_border));

        let popup = Paragraph::new(help_text)
            .block(popup_block)
            .wrap(Wrap { trim: false });

        let area = clamped_centered_rect(40, 25, frame.area());
        frame.render_widget(Clear, area);
        frame.render_widget(popup, area);
    }
}

fn clamped_centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let w = width.min(area.width);
    let h = height.min(area.height);
    let vertical = Layout::vertical([Constraint::Length(h)])
        .flex(Flex::Center)
        .split(area);
    Layout::horizontal([Constraint::Length(w)])
        .flex(Flex::Center)
        .split(vertical[0])[0]
}
