use std::time::Instant;

use super::App;
use super::state::{MessageStore, SyncManager};
use crate::constants::{
    SYNC_ACK, SYNC_BEGIN, SYNC_COMMAND, SYNC_END, SYNC_INITIAL_DELAY, SYNC_MAX_RETRIES,
    SYNC_TIMEOUT,
};
use crate::serial::{SerialEvent, SerialHandle, SerialMessage};
use crate::sync::{SyncState, parse_command_line};
use crate::widgets::SyncDisplay;

/// Lines that are part of the sync protocol and should never appear in user-visible messages.
const SYNC_PROTOCOL_MARKERS: &[&str] = &[SYNC_BEGIN, SYNC_END, SYNC_ACK];

impl SyncManager {
    /// Drive the sync state machine forward. Called each loop iteration.
    pub(super) fn tick(&mut self, serial: &SerialHandle) {
        match &self.sync_state {
            SyncState::Idle => {
                if self.started_at.elapsed() >= SYNC_INITIAL_DELAY {
                    serial.writeln(SYNC_COMMAND);
                    self.sync_state = SyncState::AwaitingBegin {
                        sent_at: Instant::now(),
                        attempts: 1,
                    };
                }
            }
            SyncState::AwaitingBegin { sent_at, attempts } => {
                if sent_at.elapsed() >= SYNC_TIMEOUT {
                    if *attempts >= SYNC_MAX_RETRIES {
                        self.sync_state = SyncState::Failed;
                    } else {
                        let next_attempt = *attempts + 1;
                        serial.writeln(SYNC_COMMAND);
                        self.sync_state = SyncState::AwaitingBegin {
                            sent_at: Instant::now(),
                            attempts: next_attempt,
                        };
                    }
                }
            }
            SyncState::Receiving { .. } | SyncState::Synced | SyncState::Failed => {}
        }
    }

    /// Process a line through the sync protocol. Returns `true` if consumed.
    pub(super) fn handle_line(&mut self, line: &str, serial: &SerialHandle) -> bool {
        // Always swallow lines containing sync protocol markers (e.g. the
        // device echoing back "unknown: #acknowledge-sync") regardless of
        // current state.
        if !matches!(self.sync_state, SyncState::AwaitingBegin { .. } | SyncState::Receiving { .. })
            && SYNC_PROTOCOL_MARKERS.iter().any(|m| line.contains(m))
        {
            return true;
        }

        match &self.sync_state {
            SyncState::AwaitingBegin { .. } => {
                if line == SYNC_BEGIN {
                    self.sync_state = SyncState::Receiving {
                        commands: Vec::new(),
                    };
                    return true;
                }
            }
            SyncState::Receiving { .. } => {
                if line == SYNC_END {
                    if let SyncState::Receiving { commands } =
                        std::mem::replace(&mut self.sync_state, SyncState::Synced)
                    {
                        self.device_commands = commands;
                    }
                    serial.writeln(SYNC_ACK);
                    return true;
                }
                if let SyncState::Receiving { commands } = &mut self.sync_state {
                    commands.push(parse_command_line(line));
                }
                return true;
            }
            _ => {}
        }
        false
    }

    /// Get the current sync display state for the status bar.
    pub(super) fn display(&self) -> SyncDisplay {
        match &self.sync_state {
            SyncState::Idle | SyncState::AwaitingBegin { .. } => SyncDisplay::Pending,
            SyncState::Receiving { .. } => SyncDisplay::Receiving,
            SyncState::Synced => SyncDisplay::Synced(self.device_commands.len()),
            SyncState::Failed => SyncDisplay::Failed,
        }
    }

    /// Reset sync state for a manual re-sync.
    pub(super) fn start_resync(&mut self) {
        self.device_commands.clear();
        self.started_at = Instant::now();
        self.sync_state = SyncState::Idle;
    }
}

impl MessageStore {
    /// Try to parse a graph data line. Returns `true` if consumed.
    pub(super) fn handle_graph_line(&mut self, line: &str) -> bool {
        if let Some(val_str) = line.strip_prefix("#graphf ")
            && let Ok(val) = val_str.trim().parse::<f64>()
        {
            self.push_graph_float(val);
            return true;
        } else if let Some(val_str) = line.strip_prefix("#graphi ")
            && let Ok(val) = val_str.trim().parse::<i64>()
        {
            self.push_graph_int(val);
            return true;
        }
        false
    }
}

impl App {
    /// Poll for serial events. Returns `Some(reason)` if the connection was lost.
    pub(super) fn receive_serial(&mut self) -> Option<String> {
        if !self.view.receiving {
            return None;
        }

        while let Some(event) = self.serial_connection.try_recv() {
            match event {
                SerialEvent::LineReceived(msg) => {
                    if self.sync.handle_line(&msg.message, &self.serial_connection) {
                        // consumed by sync protocol
                    } else if self.messages.handle_graph_line(&msg.message) {
                        // consumed by graph parser
                    } else {
                        let idx = self.messages.push_received(msg);
                        self.view.auto_scroll(idx);
                    }
                }
                SerialEvent::Error(e) => {
                    let idx = self.messages.push_received(SerialMessage::new(e));
                    self.view.auto_scroll(idx);
                }
                SerialEvent::Disconnected(reason) => {
                    return Some(reason);
                }
            }
        }
        None
    }
}
