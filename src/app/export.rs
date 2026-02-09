use std::fmt::Write as FmtWrite;
use std::io;

use chrono::Local;

use super::App;
use super::state::MessageView;
use crate::serial::SerialMessage;

impl App {
    /// Export the currently viewed buffer to a log file.
    pub(super) fn export_current_view(&mut self) {
        match self.write_export(false) {
            Ok(filename) => {
                self.view.set_status(format!("Exported to {filename}"));
            }
            Err(e) => {
                self.view.set_status(format!("Export failed: {e}"));
            }
        }
    }

    /// Export all buffers to a single log file.
    pub(super) fn export_all(&mut self) {
        match self.write_export(true) {
            Ok(filename) => {
                self.view.set_status(format!("Exported to {filename}"));
            }
            Err(e) => {
                self.view.set_status(format!("Export failed: {e}"));
            }
        }
    }

    fn write_export(&self, all: bool) -> io::Result<String> {
        let mut output = String::new();
        self.format_header(&mut output);

        if all {
            Self::format_messages(&mut output, "Received Messages", &self.messages.received);
            Self::format_messages(&mut output, "Sent Messages", &self.messages.sent);
            self.format_graphs(&mut output);
        } else {
            match self.view.current_view {
                MessageView::Received => {
                    Self::format_messages(
                        &mut output,
                        "Received Messages",
                        &self.messages.received,
                    );
                }
                MessageView::Sent => {
                    Self::format_messages(&mut output, "Sent Messages", &self.messages.sent);
                }
                MessageView::Graphs => {
                    self.format_graphs(&mut output);
                }
            }
        }

        let filename = Local::now()
            .format("rserial_log_%Y%m%d_%H%M%S.log")
            .to_string();
        std::fs::write(&filename, &output)?;
        Ok(filename)
    }

    fn format_header(&self, out: &mut String) {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S");
        let _ = writeln!(out, "=== rserial log export ===");
        let _ = writeln!(out, "Port:        {}", self.port);
        let _ = writeln!(out, "Baud rate:   {}", self.baud);
        let _ = writeln!(out, "Export time:  {now}");
        let _ = writeln!(out, "=====================================");
        let _ = writeln!(out);
    }

    fn format_messages(out: &mut String, title: &str, messages: &[SerialMessage]) {
        let _ = writeln!(out, "--- {title} ({} entries) ---", messages.len());
        for msg in messages {
            let ts = msg.timestamp.format("%H:%M:%S");
            let _ = writeln!(out, "[{ts}] {}", msg.message);
        }
        let _ = writeln!(out);
    }

    fn format_graphs(&self, out: &mut String) {
        let _ = writeln!(out, "--- Graph Data ---");
        let _ = writeln!(
            out,
            "Float values ({} entries):",
            self.messages.graph_float.len()
        );
        let floats: Vec<String> = self
            .messages
            .graph_float
            .iter()
            .map(|v| v.to_string())
            .collect();
        let _ = writeln!(out, "{}", floats.join(", "));
        let _ = writeln!(
            out,
            "Integer values ({} entries):",
            self.messages.graph_int.len()
        );
        let ints: Vec<String> = self
            .messages
            .graph_int
            .iter()
            .map(|v| v.to_string())
            .collect();
        let _ = writeln!(out, "{}", ints.join(", "));
        let _ = writeln!(out);
    }
}
