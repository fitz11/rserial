use std::time::{Duration, Instant};

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Clear, HighlightSpacing, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::constants::{COMMON_BAUD_RATES, DEFAULT_THEME, MIN_COLS, MIN_ROWS};
use crate::serial::{DetectedPort, find_esp32_ports};

/// Result of the setup screen: selected port and baud rate.
pub struct SetupResult {
    pub port: String,
    pub baud: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Ports,
    BaudRates,
}

pub struct SetupScreen {
    ports: Vec<DetectedPort>,
    port_state: ListState,
    baud_state: ListState,
    focus: Focus,
    last_scan: Instant,
    /// Optional alert shown as a popup (e.g. "Connection lost").
    alert: Option<String>,
}

impl SetupScreen {
    pub fn new(alert: Option<String>) -> Self {
        let ports = find_esp32_ports();
        let mut port_state = ListState::default();
        if !ports.is_empty() {
            port_state.select(Some(0));
        }
        let mut baud_state = ListState::default();
        let default_baud_idx = COMMON_BAUD_RATES
            .iter()
            .position(|&b| b == 115200)
            .unwrap_or(0);
        baud_state.select(Some(default_baud_idx));

        Self {
            ports,
            port_state,
            baud_state,
            focus: Focus::Ports,
            last_scan: Instant::now(),
            alert,
        }
    }

    /// Run the setup screen. Returns `None` if the user quits.
    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<Option<SetupResult>> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            // Auto-scan every second
            if self.last_scan.elapsed() >= Duration::from_secs(1) {
                self.rescan();
            }

            if event::poll(Duration::from_millis(50))?
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                // Any keypress dismisses the alert
                if self.alert.is_some() {
                    self.alert = None;
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(None),
                    KeyCode::Tab | KeyCode::BackTab => {
                        self.focus = match self.focus {
                            Focus::Ports => Focus::BaudRates,
                            Focus::BaudRates => Focus::Ports,
                        };
                    }
                    KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
                    KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
                    KeyCode::Enter => {
                        if let Some(result) = self.try_confirm() {
                            return Ok(Some(result));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn rescan(&mut self) {
        self.last_scan = Instant::now();
        let new_ports = find_esp32_ports();

        // Preserve selection if the same port is still present
        let selected_name = self
            .port_state
            .selected()
            .and_then(|i| self.ports.get(i))
            .map(|p| p.port_name.clone());

        self.ports = new_ports;

        if self.ports.is_empty() {
            self.port_state.select(None);
        } else if let Some(name) = selected_name {
            let idx = self
                .ports
                .iter()
                .position(|p| p.port_name == name)
                .unwrap_or(0);
            self.port_state.select(Some(idx));
        } else {
            self.port_state.select(Some(0));
        }
    }

    fn move_selection(&mut self, delta: i32) {
        let (state, len) = match self.focus {
            Focus::Ports => (&mut self.port_state, self.ports.len()),
            Focus::BaudRates => (&mut self.baud_state, COMMON_BAUD_RATES.len()),
        };
        if len == 0 {
            return;
        }
        let current = state.selected().unwrap_or(0) as i32;
        let next = (current + delta).rem_euclid(len as i32) as usize;
        state.select(Some(next));
    }

    fn try_confirm(&self) -> Option<SetupResult> {
        let port_idx = self.port_state.selected()?;
        let baud_idx = self.baud_state.selected()?;
        let port = self.ports.get(port_idx)?;
        let &baud = COMMON_BAUD_RATES.get(baud_idx)?;
        Some(SetupResult {
            port: port.port_name.clone(),
            baud,
        })
    }

    fn draw(&mut self, frame: &mut Frame) {
        let full = frame.area();
        if full.width < MIN_COLS || full.height < MIN_ROWS {
            let msg = format!("Terminal too small (need {}x{})", MIN_COLS, MIN_ROWS);
            let paragraph = Paragraph::new(msg).centered();
            let centered = Layout::vertical([Constraint::Length(1)])
                .flex(Flex::Center)
                .split(full);
            frame.render_widget(paragraph, centered[0]);
            return;
        }

        frame.render_widget(Clear, full);

        let area = clamped_centered_rect(60, 22, full);

        let outer = Block::bordered()
            .title(" Serial Configuration ".bold())
            .border_style(Style::default().fg(DEFAULT_THEME.help_border));
        let inner = outer.inner(area);
        frame.render_widget(outer, area);

        let layout = Layout::vertical([
            Constraint::Length(2), // header text
            Constraint::Min(5),    // port list
            Constraint::Min(5),    // baud list
            Constraint::Length(2), // footer / keybinds
        ])
        .split(inner);

        // Header
        let header = Paragraph::new(Line::from(vec![Span::styled(
            " No ESP32 device auto-detected. Select a port and baud rate:",
            Style::default().fg(DEFAULT_THEME.help_label),
        )]));
        frame.render_widget(header, layout[0]);

        // Port list
        let port_items: Vec<ListItem> = self
            .ports
            .iter()
            .map(|p| ListItem::new(format!("  {}", p.description)))
            .collect();

        let port_block_style = if self.focus == Focus::Ports {
            Style::default().fg(DEFAULT_THEME.input_active)
        } else {
            Style::default().fg(DEFAULT_THEME.help_label)
        };

        let port_list = if port_items.is_empty() {
            List::new(vec![ListItem::new(
                "  (no ESP32 devices detected — scanning...)",
            )])
            .block(
                Block::bordered()
                    .title(" Ports ")
                    .border_style(port_block_style),
            )
        } else {
            List::new(port_items)
                .block(
                    Block::bordered()
                        .title(" Ports ")
                        .border_style(port_block_style),
                )
                .highlight_style(Style::default().fg(DEFAULT_THEME.enum_text).bold())
                .highlight_symbol("▸ ")
                .highlight_spacing(HighlightSpacing::Always)
        };
        frame.render_stateful_widget(port_list, layout[1], &mut self.port_state);

        // Baud rate list
        let baud_items: Vec<ListItem> = COMMON_BAUD_RATES
            .iter()
            .map(|b| ListItem::new(format!("  {b}")))
            .collect();

        let baud_block_style = if self.focus == Focus::BaudRates {
            Style::default().fg(DEFAULT_THEME.input_active)
        } else {
            Style::default().fg(DEFAULT_THEME.help_label)
        };

        let baud_list = List::new(baud_items)
            .block(
                Block::bordered()
                    .title(" Baud Rate ")
                    .border_style(baud_block_style),
            )
            .highlight_style(Style::default().fg(DEFAULT_THEME.enum_text).bold())
            .highlight_symbol("▸ ")
            .highlight_spacing(HighlightSpacing::Always);
        frame.render_stateful_widget(baud_list, layout[2], &mut self.baud_state);

        // Footer keybinds
        let key = Style::default().fg(DEFAULT_THEME.keybind).bold();
        let label = Style::default().fg(DEFAULT_THEME.help_label);
        let footer = Paragraph::new(Line::from(vec![
            Span::styled(" ↑↓", key),
            Span::styled(" navigate  ", label),
            Span::styled("Tab", key),
            Span::styled(" switch list  ", label),
            Span::styled("Enter", key),
            Span::styled(" connect  ", label),
            Span::styled("q", key),
            Span::styled(" quit", label),
        ]));
        frame.render_widget(footer, layout[3]);

        // Alert popup overlay
        if let Some(ref msg) = self.alert {
            self.draw_alert(frame, msg);
        }
    }

    fn draw_alert(&self, frame: &mut Frame, message: &str) {
        let alert_area = clamped_centered_rect(50, 8, frame.area());
        frame.render_widget(Clear, alert_area);

        let block = Block::bordered()
            .title(" Connection Lost ".bold())
            .border_style(Style::default().fg(DEFAULT_THEME.status_frozen));

        let text = vec![
            Line::from(""),
            Line::from(Span::styled(
                message,
                Style::default().fg(DEFAULT_THEME.status_frozen),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press any key to dismiss",
                Style::default().fg(DEFAULT_THEME.help_label),
            )),
        ];

        let paragraph = Paragraph::new(text)
            .block(block)
            .wrap(Wrap { trim: false })
            .centered();

        frame.render_widget(paragraph, alert_area);
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
