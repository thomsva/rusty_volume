use regex::Regex;
use std::{cmp::min, error::Error};
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

    pub async fn get_starting_volume(&mut self, startup_volume: i32) -> i32 {
        // Run the amixer command to get the 'Digital' volume
        let output = Command::new("amixer")
            .args(&["get", &self.device, "-M"])
            .output()
            .await;

        match output {
            Ok(output) => {
                // Convert output to string and parse
                let output_str = String::from_utf8_lossy(&output.stdout);

                // Use regex to find the first percentage match in the output
                let re = Regex::new(r"(\d+)%").unwrap();
                if let Some(captures) = re.captures(&output_str) {
                    if let Some(volume_str) = captures.get(1) {
                        // Parse the matched volume percentage
                        if let Ok(amixer_volume) = volume_str.as_str().parse::<i32>() {
                            self.volume = min(amixer_volume, startup_volume);
                            return self.volume;
                        }
                    }
                }
                println!("Warning: Failed to parse volume, defaulting to 50%");
            }
            Err(e) => {
                println!("Error executing amixer command: {}. Defaulting to 50%", e);
            }
        }

        // Return default volume if parsing or command fails
        self.volume = startup_volume;

        startup_volume
    }
}
