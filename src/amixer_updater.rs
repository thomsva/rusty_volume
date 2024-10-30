use std::{
    error::Error,
    time::{Duration, Instant},
};
use tokio::process::Command;

const AMIXER_INTERVAL: Duration = Duration::from_millis(250);

pub struct AmixerUpdater {
    control: String,
    volume: i32,
    last_update: Instant,
}

impl AmixerUpdater {
    /// Create new updater
    pub fn new(control: Option<String>, volume: i32) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            control: control.unwrap_or("default".to_owned()),
            volume,
            last_update: Instant::now(),
        })
    }

    /// Asynchronously updates the volume using amixer command.
    pub async fn update(&mut self, volume: i32) -> Result<(), Box<dyn Error>> {
        if volume == self.volume {
            return Ok(());
        }

        if self.last_update.elapsed() < AMIXER_INTERVAL {
            return Ok(());
        }

        // Asynchronously run the `amixer` command
        Command::new("amixer")
            .args(&["set", &self.control, &format!("{}%", volume)])
            .output()
            .await?;

        // Update the internal state
        self.volume = volume;
        self.last_update = Instant::now();

        Ok(())
    }
}
