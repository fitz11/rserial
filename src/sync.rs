use std::time::Instant;

#[derive(Debug, Clone)]
pub struct DeviceCommand {
    pub name: String,
    pub usage: Option<String>,
}

#[derive(Debug)]
pub enum SyncState {
    /// Waiting for initial delay before first /sync
    Idle,
    /// /sync sent, waiting for #sync-begin
    AwaitingBegin { sent_at: Instant, attempts: u32 },
    /// Accumulating command lines between #sync-begin and #sync-end
    Receiving { commands: Vec<DeviceCommand> },
    /// Sync completed successfully
    Synced,
    /// Max retries exhausted
    Failed,
}

pub fn parse_command_line(line: &str) -> DeviceCommand {
    match line.split_once(' ') {
        Some((name, usage)) => DeviceCommand {
            name: name.to_string(),
            usage: Some(usage.to_string()),
        },
        None => DeviceCommand {
            name: line.to_string(),
            usage: None,
        },
    }
}
