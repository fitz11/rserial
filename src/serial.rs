use std::{sync::mpsc, thread, time::Duration};

use chrono::{DateTime, Local};
use serialport::{SerialPortType, available_ports};

use crate::constants::{ESP32_MANUFACTURER, SEEED_ESP32_C3, SEEED_VID};

#[derive(Debug)]
pub enum SerialCommand {
    Write(String),
    Shutdown,
}

#[derive(Debug)]
pub enum SerialEvent {
    LineReceived(SerialMessage),
    Error(String),
    /// Fatal connection loss — the serial thread has exited.
    Disconnected(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerialMessage {
    pub message: String,
    pub timestamp: DateTime<Local>,
}
impl SerialMessage {
    pub fn new(message: impl Into<String>) -> Self {
        let message = message.into();
        let timestamp = Local::now();
        SerialMessage { message, timestamp }
    }
}

#[derive(Debug)]
pub struct SerialHandle {
    cmd_tx: mpsc::Sender<SerialCommand>,
    event_rx: mpsc::Receiver<SerialEvent>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl SerialHandle {
    pub fn new(port: &str, baud: u32) -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::channel();

        let thread_handle = Some(spawn_serial_thread(port, baud, cmd_rx, event_tx));
        SerialHandle {
            cmd_tx,
            event_rx,
            thread_handle,
        }
    }

    #[allow(dead_code)]
    pub fn write(&self, data: impl Into<String>) {
        let _ = self.cmd_tx.send(SerialCommand::Write(data.into()));
    }

    pub fn writeln(&self, data: impl Into<String>) {
        let mut string = data.into();
        if !string.ends_with('\n') {
            string.push('\n');
        }
        let _ = self.cmd_tx.send(SerialCommand::Write(string));
    }

    pub fn try_recv(&self) -> Option<SerialEvent> {
        self.event_rx.try_recv().ok()
    }

    pub fn shutdown(&self) {
        let _ = self.cmd_tx.send(SerialCommand::Shutdown);
    }

    pub fn mock() -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::channel();

        let thread_handle = Some(thread::spawn(move || {
            let mut counter: u64 = 0;
            let mut last_tick = std::time::Instant::now();

            loop {
                // Handle commands (non-blocking)
                while let Ok(cmd) = cmd_rx.try_recv() {
                    match cmd {
                        SerialCommand::Write(data) => {
                            let line = data.trim_end().to_string();
                            event_tx
                                .send(SerialEvent::LineReceived(SerialMessage::new(format!(
                                    "echo: {line}"
                                ))))
                                .ok();
                        }
                        SerialCommand::Shutdown => return,
                    }
                }

                // Send periodic mock data every ~1 second
                if last_tick.elapsed() >= Duration::from_secs(1) {
                    last_tick = std::time::Instant::now();
                    counter += 1;

                    event_tx
                        .send(SerialEvent::LineReceived(SerialMessage::new(format!(
                            "[mock] tick {counter}"
                        ))))
                        .ok();

                    // Send sample graph data every few ticks
                    if counter.is_multiple_of(3) {
                        let val = (counter as f64 * 0.5).sin() * 100.0;
                        event_tx
                            .send(SerialEvent::LineReceived(SerialMessage::new(format!(
                                "#graphf {val:.2}"
                            ))))
                            .ok();
                    }
                    if counter.is_multiple_of(5) {
                        let val = (counter % 200) as i64 - 100;
                        event_tx
                            .send(SerialEvent::LineReceived(SerialMessage::new(format!(
                                "#graphi {val}"
                            ))))
                            .ok();
                    }
                }

                thread::sleep(Duration::from_millis(10));
            }
        }));

        SerialHandle {
            cmd_tx,
            event_rx,
            thread_handle,
        }
    }
}

impl Drop for SerialHandle {
    fn drop(&mut self) {
        self.shutdown();
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

pub fn find_esp32_port() -> Option<String> {
    let ports = available_ports().ok().unwrap_or_default();

    for port in ports {
        if let SerialPortType::UsbPort(info) = port.port_type
            && info.vid == SEEED_VID
            && info.pid == SEEED_ESP32_C3
        {
            return Some(port.port_name);
        }
    }
    None
}

/// A detected ESP32-compatible serial port with display info.
#[derive(Debug, Clone)]
pub struct DetectedPort {
    pub port_name: String,
    pub description: String,
}

/// Scan for all USB serial ports matching any ESP32 constant (VID, PID, or manufacturer).
pub fn find_esp32_ports() -> Vec<DetectedPort> {
    let ports = available_ports().ok().unwrap_or_default();

    ports
        .into_iter()
        .filter_map(|port| {
            if let SerialPortType::UsbPort(info) = &port.port_type {
                let vid_match = info.vid == SEEED_VID;
                let pid_match = info.pid == SEEED_ESP32_C3;
                let mfr_match = info
                    .manufacturer
                    .as_deref()
                    .is_some_and(|m| m.contains(ESP32_MANUFACTURER));

                if vid_match || pid_match || mfr_match {
                    let desc = format!(
                        "{} [VID:{:04x} PID:{:04x}{}]",
                        port.port_name,
                        info.vid,
                        info.pid,
                        info.manufacturer
                            .as_deref()
                            .map(|m| format!(" {m}"))
                            .unwrap_or_default(),
                    );
                    return Some(DetectedPort {
                        port_name: port.port_name,
                        description: desc,
                    });
                }
            }
            None
        })
        .collect()
}

fn spawn_serial_thread(
    port_name: &str,
    baud_rate: u32,
    cmd_rx: mpsc::Receiver<SerialCommand>,
    event_tx: mpsc::Sender<SerialEvent>,
) -> thread::JoinHandle<()> {
    let port_name = port_name.to_string();

    thread::spawn(move || {
        let mut port = match serialport::new(port_name, baud_rate)
            .timeout(Duration::from_millis(10))
            .open()
        {
            Ok(p) => p,
            Err(e) => {
                event_tx
                    .send(SerialEvent::Error(format!("Open error: {e}")))
                    .ok();
                return;
            }
        };

        let mut buf = [0u8; 256];
        let mut line_buffer = String::new();

        loop {
            // Handle outgoing commands (non-blocking)
            while let Ok(cmd) = cmd_rx.try_recv() {
                match cmd {
                    SerialCommand::Write(data) => {
                        if let Err(e) = port.write_all(data.as_bytes()) {
                            event_tx
                                .send(SerialEvent::Error(format!("Write error: {e}")))
                                .ok();
                        }
                    }
                    SerialCommand::Shutdown => {
                        return;
                    }
                }
            }

            // Read incoming data (blocking w/ timeout)
            match port.read(&mut buf) {
                Ok(n) if n > 0 => {
                    let chunk = String::from_utf8_lossy(&buf[..n]);
                    for c in chunk.chars() {
                        if c == '\n' {
                            if line_buffer.ends_with('\r') {
                                line_buffer.pop();
                            }
                            event_tx
                                .send(SerialEvent::LineReceived(SerialMessage::new(
                                    line_buffer.clone(),
                                )))
                                .ok();
                            line_buffer.clear();
                        } else {
                            line_buffer.push(c);
                        }
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    // expected; ignore
                }
                Err(e) => {
                    event_tx
                        .send(SerialEvent::Disconnected(format!("Read error: {e}")))
                        .ok();
                    return;
                }
                _ => {} //Ok(n==0)
            }
        }
    })
}
