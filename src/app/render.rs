use ratatui::{
    Frame,
    layout::{Constraint, Layout, Position},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::App;
use super::state::{InputMode, MessageView};
use crate::constants::{DEFAULT_THEME, MIN_COLS, MIN_ROWS};
use crate::widgets::{
    CommandPalette, GraphView, HelpBar, HelpPopup, InputField, MessageList, StatusBar,
};

impl App {
    pub(super) fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        if area.width < MIN_COLS || area.height < MIN_ROWS {
            let msg = format!("Terminal too small (need {}x{})", MIN_COLS, MIN_ROWS);
            let paragraph = ratatui::widgets::Paragraph::new(msg).centered();
            let centered = Layout::vertical([Constraint::Length(1)])
                .flex(ratatui::layout::Flex::Center)
                .split(area);
            frame.render_widget(paragraph, centered[0]);
            return;
        }

        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ]);

        let [help_area, input_area, message_area, status_area] = vertical.areas(frame.area());

        frame.render_widget(
            HelpBar::new(self.view.input_mode, help_area.width),
            help_area,
        );
        frame.render_widget(
            InputField::new(&self.input.text, self.view.input_mode),
            input_area,
        );

        if self.view.input_mode == InputMode::Editing {
            frame.set_cursor_position(Position::new(
                input_area.x + self.input.cursor as u16 + 1,
                input_area.y + 1,
            ));
        }

        match self.view.current_view {
            MessageView::Graphs => {
                let float_data: Vec<f64> = self.messages.graph_float.iter().copied().collect();
                let int_data: Vec<i64> = self.messages.graph_int.iter().copied().collect();
                frame.render_widget(
                    GraphView::new(&float_data, &int_data, self.view.graph_y_locked),
                    message_area,
                );
            }
            _ => {
                let (messages, title, border_color) = match self.view.current_view {
                    MessageView::Received => (
                        self.messages.received.as_slice(),
                        "Received",
                        DEFAULT_THEME.received_border,
                    ),
                    MessageView::Sent => (
                        self.messages.sent.as_slice(),
                        "Sent",
                        DEFAULT_THEME.sent_border,
                    ),
                    MessageView::Graphs => unreachable!(),
                };
                frame.render_stateful_widget(
                    MessageList::new(messages, title, self.view.show_timestamps, border_color),
                    message_area,
                    &mut self.view.list_state.clone(),
                );
            }
        }

        // Status bar — show transient status message if active, else normal
        if let Some((ref msg, _)) = self.view.status_message {
            let spans = Line::from(vec![Span::styled(
                format!(" {msg}"),
                Style::default().fg(Color::Green).bold(),
            )]);
            frame.render_widget(
                Paragraph::new(spans).style(Style::default().bg(DEFAULT_THEME.status_bar_bg)),
                status_area,
            );
        } else {
            self.render_normal_status(frame, status_area);
        }

        if self.view.input_mode == InputMode::Help {
            HelpPopup::render_overlay(frame);
        }

        if self.view.input_mode == InputMode::CommandPalette {
            CommandPalette::render_overlay(
                frame,
                &self.sync.device_commands,
                &self.cmd_palette.filter,
                &mut self.cmd_palette.list_state.clone(),
            );
        }
    }

    fn render_normal_status(&self, frame: &mut Frame, status_area: ratatui::layout::Rect) {
        frame.render_widget(
            StatusBar::new(
                &self.port,
                self.baud,
                self.view.input_mode,
                self.view.receiving,
                self.sync.display(),
            ),
            status_area,
        );
    }
}
