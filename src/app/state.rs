use std::collections::VecDeque;
use std::time::Instant;

use ratatui::widgets::ListState;

use crate::constants::GRAPH_BUFFER_SIZE;
use crate::serial::{SerialHandle, SerialMessage};
use crate::sync::{DeviceCommand, SyncState};

/// How the app exited.
pub enum AppExit {
    /// User chose to quit.
    Quit,
    /// Serial connection was lost.
    Disconnected(String),
    /// User manually disconnected (return to setup screen).
    ManualDisconnect,
}

/// Mode enum
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InputMode {
    #[default]
    Normal,
    Editing,
    Help,
    CommandPalette,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) enum MessageView {
    #[default]
    Received,
    Sent,
    Graphs,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Direction {
    Left,
    Right,
}

// ---------------------------------------------------------------------------
// Sub-structs
// ---------------------------------------------------------------------------

/// Text editing buffer and cursor.
#[derive(Debug)]
pub(super) struct InputBuffer {
    pub(super) text: String,
    pub(super) cursor: usize,
}

impl InputBuffer {
    fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
        }
    }

    /// Drain the buffer and return its contents, resetting the cursor.
    pub(super) fn take_input(&mut self) -> String {
        let text = std::mem::take(&mut self.text);
        self.cursor = 0;
        text
    }
}

/// Received/sent messages and graph data.
#[derive(Debug)]
pub(super) struct MessageStore {
    pub(super) received: Vec<SerialMessage>,
    pub(super) sent: Vec<SerialMessage>,
    pub(super) graph_float: VecDeque<f64>,
    pub(super) graph_int: VecDeque<i64>,
}

impl MessageStore {
    fn new() -> Self {
        Self {
            received: Vec::new(),
            sent: Vec::new(),
            graph_float: VecDeque::new(),
            graph_int: VecDeque::new(),
        }
    }

    /// Push a received message. Returns the index of the new last element.
    pub(super) fn push_received(&mut self, msg: SerialMessage) -> usize {
        self.received.push(msg);
        self.received.len().saturating_sub(1)
    }

    pub(super) fn push_sent(&mut self, msg: SerialMessage) {
        self.sent.push(msg);
    }

    pub(super) fn push_graph_float(&mut self, val: f64) {
        self.graph_float.push_back(val);
        if self.graph_float.len() > GRAPH_BUFFER_SIZE {
            self.graph_float.pop_front();
        }
    }

    pub(super) fn push_graph_int(&mut self, val: i64) {
        self.graph_int.push_back(val);
        if self.graph_int.len() > GRAPH_BUFFER_SIZE {
            self.graph_int.pop_front();
        }
    }
}

/// Display mode, toggle flags, list selection, and transient status.
#[derive(Debug)]
pub(super) struct ViewState {
    pub(super) input_mode: InputMode,
    pub(super) current_view: MessageView,
    pub(super) show_timestamps: bool,
    pub(super) receiving: bool,
    pub(super) list_state: ListState,
    pub(super) status_message: Option<(String, Instant)>,
    pub(super) graph_y_locked: Option<(f64, f64)>,
}

impl ViewState {
    fn new() -> Self {
        Self {
            input_mode: InputMode::Normal,
            current_view: MessageView::default(),
            show_timestamps: false,
            receiving: true,
            list_state: ListState::default(),
            status_message: None,
            graph_y_locked: None,
        }
    }

    pub(super) fn set_status(&mut self, msg: String) {
        self.status_message = Some((msg, Instant::now()));
    }

    /// Expire the status message if its duration has elapsed.
    pub(super) fn expire_status(&mut self, max_age: std::time::Duration) {
        if let Some((_, created_at)) = &self.status_message
            && created_at.elapsed() >= max_age
        {
            self.status_message = None;
        }
    }

    /// Auto-scroll the list to the given index.
    pub(super) fn auto_scroll(&mut self, idx: usize) {
        self.list_state.select(Some(idx));
    }
}

/// Sync protocol state and discovered device commands.
#[derive(Debug)]
pub(super) struct SyncManager {
    pub(super) device_commands: Vec<DeviceCommand>,
    pub(super) sync_state: SyncState,
    pub(super) started_at: Instant,
}

impl SyncManager {
    fn new() -> Self {
        Self {
            device_commands: Vec::new(),
            sync_state: SyncState::Idle,
            started_at: Instant::now(),
        }
    }
}

/// Command palette filter and selection state.
#[derive(Debug)]
pub(super) struct CommandPaletteState {
    pub(super) filter: String,
    pub(super) list_state: ListState,
}

impl CommandPaletteState {
    fn new() -> Self {
        Self {
            filter: String::new(),
            list_state: ListState::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

/// State struct
#[derive(Debug)]
pub struct App {
    pub(super) input: InputBuffer,
    pub(super) messages: MessageStore,
    pub(super) view: ViewState,
    pub(super) sync: SyncManager,
    pub(super) cmd_palette: CommandPaletteState,
    pub(super) serial_connection: SerialHandle,
    pub(super) port: String,
    pub(super) baud: u32,
}

impl App {
    pub fn new(port: &str, baud: u32) -> Self {
        Self {
            input: InputBuffer::new(),
            messages: MessageStore::new(),
            view: ViewState::new(),
            sync: SyncManager::new(),
            cmd_palette: CommandPaletteState::new(),
            serial_connection: SerialHandle::new(port, baud),
            port: port.to_string(),
            baud,
        }
    }

    pub fn new_mock(baud: u32) -> Self {
        Self {
            input: InputBuffer::new(),
            messages: MessageStore::new(),
            view: ViewState::new(),
            sync: SyncManager::new(),
            cmd_palette: CommandPaletteState::new(),
            serial_connection: SerialHandle::mock(),
            port: "mock".to_string(),
            baud,
        }
    }
}
