use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::app::InputMode;
use crate::constants::DEFAULT_THEME;

#[derive(Debug, Clone, Copy)]
pub enum SyncDisplay {
    /// Initial delay or awaiting begin
    Pending,
    /// Receiving command list
    Receiving,
    /// Sync completed with N commands
    Synced(usize),
    /// Sync failed after retries
    Failed,
}

pub struct StatusBar<'a> {
    port: &'a str,
    baud: u32,
    mode: InputMode,
    receiving: bool,
    sync: SyncDisplay,
}

impl<'a> StatusBar<'a> {
    pub fn new(
        port: &'a str,
        baud: u32,
        mode: InputMode,
        receiving: bool,
        sync: SyncDisplay,
    ) -> Self {
        Self {
            port,
            baud,
            mode,
            receiving,
            sync,
        }
    }
}

impl Widget for StatusBar<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mode_str = match self.mode {
            InputMode::Normal => "NORMAL",
            InputMode::Editing => "EDITING",
            InputMode::Help => "HELP",
            InputMode::CommandPalette => "COMMANDS",
        };
        let (status_str, status_color) = if self.receiving {
            ("LIVE", DEFAULT_THEME.status_live)
        } else {
            ("FROZEN", DEFAULT_THEME.status_frozen)
        };

        let (sync_str, sync_color) = match self.sync {
            SyncDisplay::Pending => ("SYNC...".to_string(), Color::Gray),
            SyncDisplay::Receiving => ("SYNCING".to_string(), Color::Yellow),
            SyncDisplay::Synced(n) => (format!("SYNCED({n})"), crate::constants::SYNC_STATUS_COLOR),
            SyncDisplay::Failed => ("SYNC FAILED".to_string(), Color::Red),
        };

        let left = Span::raw(format!(" {} @ {} baud", self.port, self.baud));
        let right_spans = vec![
            Span::styled(&sync_str, Style::default().fg(sync_color).bold()),
            Span::raw(" | "),
            Span::styled(mode_str, Style::default().bold()),
            Span::raw(" | "),
            Span::styled(status_str, Style::default().fg(status_color).bold()),
            Span::raw(" "),
        ];

        // Fill background
        Paragraph::new("")
            .style(Style::default().bg(DEFAULT_THEME.status_bar_bg))
            .render(area, buf);

        // Left-aligned text
        Paragraph::new(left)
            .style(Style::default().bg(DEFAULT_THEME.status_bar_bg))
            .render(area, buf);

        // Right-aligned text
        Paragraph::new(Line::from(right_spans))
            .alignment(Alignment::Right)
            .style(Style::default().bg(DEFAULT_THEME.status_bar_bg))
            .render(area, buf);
    }
}
