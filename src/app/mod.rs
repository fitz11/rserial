mod export;
mod input;
mod render;
mod serial_handler;
mod state;

pub(crate) use state::InputMode;
pub use state::{App, AppExit};

use std::time::Duration;

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;

use crate::serial::SerialMessage;
use crate::widgets::CommandPalette;
use state::{Direction, MessageView};

const STATUS_MESSAGE_DURATION: Duration = Duration::from_secs(3);

impl App {
    pub fn run(mut self, terminal: &mut DefaultTerminal) -> Result<AppExit> {
        loop {
            self.view.expire_status(STATUS_MESSAGE_DURATION);

            terminal.draw(|frame| self.draw(frame))?;

            self.sync.tick(&self.serial_connection);

            if let Some(reason) = self.receive_serial() {
                return Ok(AppExit::Disconnected(reason));
            }

            if event::poll(Duration::from_millis(5))?
                && let Event::Key(key) = event::read()?
            {
                match self.view.input_mode {
                    InputMode::Normal => match (key.code, key.modifiers) {
                        (KeyCode::Char('e'), _) => {
                            self.view.input_mode = InputMode::Editing;
                        }
                        (KeyCode::Char('q'), _) => {
                            return Ok(AppExit::Quit);
                        }
                        (KeyCode::Char('h'), _) => {
                            self.view.input_mode = InputMode::Help;
                        }
                        (KeyCode::Char('f'), _) => {
                            self.view.receiving = !self.view.receiving;
                        }
                        (KeyCode::Char('1'), _) => {
                            self.view.current_view = MessageView::Received;
                        }
                        (KeyCode::Char('2'), _) => {
                            self.view.current_view = MessageView::Sent;
                        }
                        (KeyCode::Char('3'), _) => {
                            self.view.current_view = MessageView::Graphs;
                        }
                        (KeyCode::Char('t'), _) => {
                            self.view.show_timestamps = !self.view.show_timestamps;
                        }
                        (KeyCode::Char('r'), m) if m.contains(KeyModifiers::CONTROL) => {
                            self.messages.graph_float.clear();
                            self.messages.graph_int.clear();
                        }
                        (KeyCode::Char('r'), _) => {
                            self.messages.received.clear();
                            self.view.list_state.select(None);
                        }
                        (KeyCode::Char('R'), _) => {
                            self.messages.sent.clear();
                            self.view.list_state.select(None);
                        }
                        (KeyCode::Char('c'), _) => {
                            if !self.sync.device_commands.is_empty() {
                                self.cmd_palette.filter.clear();
                                self.cmd_palette.list_state.select(Some(0));
                                self.view.input_mode = InputMode::CommandPalette;
                            }
                        }
                        (KeyCode::Char('s'), _) => {
                            self.sync.start_resync();
                        }
                        (KeyCode::Char('l'), _) => {
                            self.export_current_view();
                        }
                        (KeyCode::Char('L'), _) => {
                            self.export_all();
                        }
                        (KeyCode::Char('y'), _) => {
                            if self.view.graph_y_locked.is_some() {
                                self.view.graph_y_locked = None;
                                self.view.set_status("Y-axis auto".into());
                            } else {
                                let data: Vec<f64> =
                                    self.messages.graph_float.iter().copied().collect();
                                if !data.is_empty() {
                                    let min =
                                        data.iter().copied().fold(f64::INFINITY, f64::min);
                                    let max =
                                        data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                                    let range = max - min;
                                    let padding = if range.abs() < f64::EPSILON {
                                        1.0
                                    } else {
                                        range * 0.1
                                    };
                                    self.view.graph_y_locked =
                                        Some((min - padding, max + padding));
                                    self.view.set_status("Y-axis locked".into());
                                }
                            }
                        }
                        (KeyCode::Char('x'), _) => {
                            return Ok(AppExit::ManualDisconnect);
                        }
                        _ => {}
                    },
                    InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                        KeyCode::Enter => {
                            let text = self.input.take_input();
                            if !text.is_empty() {
                                self.messages.push_sent(SerialMessage::new(&text));
                                self.serial_connection.writeln(&text);
                            }
                        }
                        KeyCode::Char(to_insert) => self.input.enter_char(to_insert),
                        KeyCode::Backspace => self.input.delete_char(),
                        KeyCode::Left => self.input.move_cursor(Direction::Left),
                        KeyCode::Esc => self.view.input_mode = InputMode::Normal,
                        _ => {}
                    },
                    InputMode::Editing => {}
                    InputMode::Help => match key.code {
                        KeyCode::Esc | KeyCode::Char('h') => {
                            self.view.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    },
                    InputMode::CommandPalette if key.kind == KeyEventKind::Press => {
                        match key.code {
                            KeyCode::Esc => {
                                self.view.input_mode = InputMode::Normal;
                            }
                            KeyCode::Up => {
                                let i = self
                                    .cmd_palette
                                    .list_state
                                    .selected()
                                    .unwrap_or(0)
                                    .saturating_sub(1);
                                self.cmd_palette.list_state.select(Some(i));
                            }
                            KeyCode::Down => {
                                let i = self
                                    .cmd_palette
                                    .list_state
                                    .selected()
                                    .map(|i| i + 1)
                                    .unwrap_or(0);
                                self.cmd_palette.list_state.select(Some(i));
                            }
                            KeyCode::Enter => {
                                if let Some(cmd) = CommandPalette::selected_command(
                                    &self.sync.device_commands,
                                    &self.cmd_palette.filter,
                                    &self.cmd_palette.list_state,
                                ) {
                                    self.input.text = format!("{} ", cmd.name);
                                    self.input.cursor = self.input.text.chars().count();
                                    self.view.input_mode = InputMode::Editing;
                                }
                            }
                            KeyCode::Char(c) => {
                                self.cmd_palette.filter.push(c);
                                self.cmd_palette.list_state.select(Some(0));
                            }
                            KeyCode::Backspace => {
                                self.cmd_palette.filter.pop();
                                self.cmd_palette.list_state.select(Some(0));
                            }
                            _ => {}
                        }
                    }
                    InputMode::CommandPalette => {}
                }
            }
        }
    }
}
