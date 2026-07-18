use color_eyre::Result;
use ratatui::DefaultTerminal;
use crate::config::{ConfigFile, load_config};


pub enum AppViewLogin {
    Auth,
}

pub struct AppLogin {
    pub view_state: AppViewLogin,
    pub should_exit: bool,
    pub config: ConfigFile,
}

/// Init app
impl AppLogin {
    pub async fn new() -> Result<Self> {
        // init config
        let config = load_config()?;

        // init view_state
        let view_state = AppViewLogin::Auth;
        Ok(Self {
            should_exit: false,
            view_state,
            config,
        })
    }


    /// handle events
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
        }
        Ok(())
    }
}
