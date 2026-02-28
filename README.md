# rserial

A terminal-based serial port communication tool built for ESP32 development. Provides an interactive TUI for real-time serial communication, data visualization, and device command discovery.

![Rust](https://img.shields.io/badge/Rust-2024_edition-orange)

## Features

- **Interactive TUI** — Real-time message viewer for sent and received serial data
- **Graph visualization** — Sparkline graphs for numerical data streamed from devices
- **Device command discovery** — Automatic sync protocol to discover and browse device commands
- **Command palette** — Searchable list of device commands with descriptions
- **Log export** — Export messages to timestamped log files
- **Multiple baud rates** — 9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600
- **Mock mode** — Test the interface without hardware connected
- **Freeze/unfreeze** — Pause incoming data for inspection

## Installation

### From source

```bash
git clone <repo-url>
cd rserial
cargo build --release
```

The binary will be at `./target/release/rserial`.

## Usage

```bash
rserial                        # Launch with device selection screen
rserial --mock                 # Launch in mock mode (no hardware needed)
rserial --baud-rate 9600       # Set a specific baud rate (default: 115200)
```

On launch, rserial presents a setup screen where you can select a connected serial port and baud rate. ESP32 devices (Seeed ESP32-C3) are auto-detected.

## Keybindings

### Normal Mode

| Key       | Action                            |
|-----------|-----------------------------------|
| `q`       | Quit                              |
| `e`       | Enter edit mode (type a message)  |
| `h`       | Toggle help overlay               |
| `f`       | Freeze/unfreeze receiving         |
| `1`       | View received messages            |
| `2`       | View sent messages                |
| `3`       | View graphs                       |
| `t`       | Toggle timestamps                 |
| `r` / `R` | Clear received / sent messages    |
| `Ctrl+r`  | Clear graph data                  |
| `c`       | Open command palette              |
| `s`       | Re-sync with device               |
| `l` / `L` | Export current view / export all  |
| `x`       | Disconnect and return to setup    |

### Edit Mode

| Key         | Action           |
|-------------|------------------|
| `Enter`     | Send message     |
| `Esc`       | Cancel           |
| `Backspace` | Delete character |
| `Left/Right`| Move cursor      |

## Device Protocols

### Sync Protocol

rserial can automatically discover commands supported by a connected device. The protocol works as follows:

1. rserial sends `/sync` to the device
2. Device responds with `#sync-begin`
3. Device sends commands, one per line: `<command> [description]`
4. Device sends `#sync-end`
5. rserial acknowledges with `#acknowledge-sync`

Discovered commands are browsable via the command palette (`c`).

### Graph Protocol

Devices can stream numerical data for live graph display by prefixing lines with:

- `#graphf <float>` — Float values (e.g., `#graphf 3.14`)
- `#graphi <int>` — Integer values (e.g., `#graphi 42`)

Data is displayed as sparkline graphs with min/max/last statistics.

## Project Structure

```
src/
├── main.rs              # Entry point, terminal setup, CLI args
├── constants.rs         # Colors, sync markers, baud rates
├── serial.rs            # Serial port I/O (threaded, channel-based)
├── setup.rs             # Device/baud rate selection screen
├── sync.rs              # Sync protocol state machine
├── app/
│   ├── mod.rs           # Event loop and input dispatch
│   ├── state.rs         # Core application state
│   ├── render.rs        # TUI layout and rendering
│   ├── serial_handler.rs# Serial message processing
│   ├── input.rs         # Text input buffer
│   └── export.rs        # Log file export
└── widgets/
    ├── message_list.rs  # Message display widget
    ├── status_bar.rs    # Status line widget
    ├── input_field.rs   # Input field widget
    ├── help_bar.rs      # Context-sensitive help bar
    ├── help_popup.rs    # Full help overlay
    ├── command_palette.rs # Command search overlay
    └── graph_view.rs    # Sparkline graph widget
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| [ratatui](https://crates.io/crates/ratatui) | TUI framework |
| [crossterm](https://crates.io/crates/crossterm) | Terminal events and rendering backend |
| [serialport](https://crates.io/crates/serialport) | Serial port communication |
| [argh](https://crates.io/crates/argh) | CLI argument parsing |
| [chrono](https://crates.io/crates/chrono) | Timestamps and date formatting |
| [color-eyre](https://crates.io/crates/color-eyre) | Error handling |

## Requirements

- Rust (2024 edition)
- A connected serial device, or use `--mock` for testing
