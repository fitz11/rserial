use ratatui::{
    Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Clear, List, ListItem, ListState, Paragraph},
};

use crate::constants::DEFAULT_THEME;
use crate::sync::DeviceCommand;

pub struct CommandPalette;

impl CommandPalette {
    pub fn render_overlay(
        frame: &mut Frame,
        commands: &[DeviceCommand],
        filter: &str,
        list_state: &mut ListState,
    ) {
        let filtered: Vec<&DeviceCommand> = commands
            .iter()
            .filter(|cmd| {
                if filter.is_empty() {
                    true
                } else {
                    cmd.name.to_lowercase().contains(&filter.to_lowercase())
                }
            })
            .collect();

        let popup_block = Block::bordered()
            .title("Commands".bold())
            .border_style(Style::default().fg(DEFAULT_THEME.help_border));

        // Filter input line
        let filter_line = if filter.is_empty() {
            Line::from(Span::styled(
                "  type to filter...",
                Style::default().fg(DEFAULT_THEME.help_label).italic(),
            ))
        } else {
            Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(filter, Style::default().fg(DEFAULT_THEME.keybind)),
            ])
        };

        // Build list items
        let items: Vec<ListItem> = filtered
            .iter()
            .map(|cmd| {
                let mut spans = vec![Span::styled(
                    &cmd.name,
                    Style::default().fg(DEFAULT_THEME.enum_text).bold(),
                )];
                if let Some(usage) = &cmd.usage {
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled(
                        usage,
                        Style::default().fg(DEFAULT_THEME.help_label),
                    ));
                }
                ListItem::new(Line::from(spans))
            })
            .collect();

        let item_count = items.len();
        let list_height = (item_count as u16).max(1) + 4; // +2 border, +1 filter, +1 separator
        let popup_height = list_height.clamp(6, 20);

        let area = clamped_centered_rect(50, popup_height, frame.area());
        frame.render_widget(Clear, area);

        // Split popup area: border top (1) + filter (1) + separator (1) + list + border bottom (1)
        let inner = popup_block.inner(area);
        frame.render_widget(popup_block, area);

        if inner.height < 2 {
            return;
        }

        let inner_layout = Layout::vertical([
            Constraint::Length(1), // filter
            Constraint::Length(1), // separator
            Constraint::Min(1),    // list
        ])
        .split(inner);

        frame.render_widget(Paragraph::new(filter_line), inner_layout[0]);
        frame.render_widget(
            Paragraph::new("  ─────────────────────────────────────────────")
                .style(Style::default().fg(DEFAULT_THEME.help_label)),
            inner_layout[1],
        );

        // Clamp selection to filtered range
        if item_count == 0 {
            list_state.select(None);
        } else if let Some(sel) = list_state.selected() {
            if sel >= item_count {
                list_state.select(Some(item_count - 1));
            }
        } else {
            list_state.select(Some(0));
        }

        let list = List::new(items)
            .highlight_symbol("▸ ")
            .highlight_style(Style::default().bold().reversed());

        frame.render_stateful_widget(list, inner_layout[2], list_state);
    }

    /// Returns the name of the currently selected command (after filtering).
    pub fn selected_command<'a>(
        commands: &'a [DeviceCommand],
        filter: &str,
        list_state: &ListState,
    ) -> Option<&'a DeviceCommand> {
        let filtered: Vec<&DeviceCommand> = commands
            .iter()
            .filter(|cmd| {
                if filter.is_empty() {
                    true
                } else {
                    cmd.name.to_lowercase().contains(&filter.to_lowercase())
                }
            })
            .collect();

        list_state.selected().and_then(|i| filtered.get(i).copied())
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
