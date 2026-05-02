#![allow(
    unused,
    reason = "These constants may or may not be useful in every case."
)]

use std::time::Duration;

use ratatui::style::Color;

// Some useful constants, may be specific to my use case

pub const SEEED_VID: u16 = 0x303a; // Vendor ID (vid) for the ESP32-C3 board I am using
pub const SEEED_ESP32_C3: u16 = 0x1001; // Product ID (pid) for the ESP32-C3 board I have
pub const ESP32_MANUFACTURER: &str = "Espressif"; // Manufacturer ID for ESP32

pub const SERIAL_PORT: &str = "/dev/ttyACM1"; // The port I'm using on my desktop

pub const MIN_COLS: u16 = 80;
pub const MIN_ROWS: u16 = 24;

pub const COMMON_BAUD_RATES: &[u32] = &[9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600];

// --- Sync Protocol ---

pub const SYNC_COMMAND: &str = "/sync";
pub const SYNC_BEGIN: &str = "#sync-begin";
pub const SYNC_END: &str = "#sync-end";
pub const SYNC_ACK: &str = "#acknowledge-sync";
pub const SYNC_TIMEOUT: Duration = Duration::from_millis(2000);
pub const SYNC_MAX_RETRIES: u32 = 3;
pub const SYNC_INITIAL_DELAY: Duration = Duration::from_millis(500);
pub const SYNC_STATUS_COLOR: Color = Color::Cyan;

// --- Graph View ---

pub const GRAPH_BUFFER_SIZE: usize = 1500;

// --- Theming ---

pub struct Theme {
    pub input_active: Color,
    pub received_border: Color,
    pub sent_border: Color,
    pub status_bar_bg: Color,
    pub status_live: Color,
    pub status_frozen: Color,
    pub help_border: Color,
    pub keybind: Color,
    pub enum_text: Color,
    pub help_label: Color,
    pub graph_float_border: Color,
    pub graph_int_border: Color,
}

pub const DEFAULT_THEME: Theme = Theme {
    input_active: Color::Yellow,
    received_border: Color::Green,
    sent_border: Color::Red,
    status_bar_bg: Color::DarkGray,
    status_live: Color::Green,
    status_frozen: Color::Red,
    help_border: Color::White,
    keybind: Color::White,
    enum_text: Color::Yellow,
    help_label: Color::Gray,
    graph_float_border: Color::Cyan,
    graph_int_border: Color::Magenta,
};
