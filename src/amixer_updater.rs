use std::error::Error;
use tokio::process::Command;

pub struct AmixerUpdater {
    device: String,
    volume: i32,
}

impl AmixerUpdater {
    /// Create new updater
    pub fn new(control: Option<String>, volume: i32) -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            device: control.unwrap_or("default".to_owned()),
            volume,
        })
    }

    /// Updates the volume using the amixer command.
    pub async fn update(&mut self, volume: i32) -> Result<(), Box<dyn Error>> {
        if volume == self.volume {
            return Ok(());
        }
        self.volume = volume;

        Command::new("amixer")
            .args(&["set", "-M", &self.device, &format!("{}%", volume)])
            .output()
            .await?;

        Ok(())
    }
}
