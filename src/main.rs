use argh::FromArgs;

use crate::app::{App, AppExit};
use crate::setup::SetupScreen;

use color_eyre::Result;

mod app;
mod constants;
mod serial;
mod setup;
mod sync;
mod widgets;

/// RSerial CLI flags
#[derive(Debug, FromArgs)]
struct Cli {
    /// baud rate of the serial connection. Defaults to 115200
    #[argh(option, default = "115200")]
    baud_rate: u32,

    /// use a mock serial connection (no physical device needed)
    #[argh(switch, short = 'm')]
    mock: bool,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut cli: Cli = argh::from_env();

    let mut terminal = ratatui::init();
    let mut alert: Option<String> = None;

    loop {
        let app = if cli.mock {
            cli.mock = false; // only auto-enter mock on first iteration
            App::new_mock(cli.baud_rate)
        } else {
            let (port, baud) = match (&alert, serial::find_esp32_port()) {
                // Auto-connect only when there's no alert (i.e. not returning from a disconnect)
                (None, Some(port)) => (port, cli.baud_rate),
                _ => {
                    let setup = SetupScreen::new(alert.take());
                    match setup.run(&mut terminal)? {
                        Some(result) => (result.port, result.baud),
                        None => break,
                    }
                }
            };
            App::new(port.as_str(), baud)
        };

        match app.run(&mut terminal)? {
            AppExit::Quit => break,
            AppExit::Disconnected(reason) => {
                alert = Some(reason);
            }
            AppExit::ManualDisconnect => {}
        }
    }

    ratatui::restore();
    Ok(())
}
