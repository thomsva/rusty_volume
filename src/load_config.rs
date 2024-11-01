use serde::Deserialize;
use std::error::Error;
use std::fs;
use toml::de::from_str;

// Define the Config struct
#[derive(Debug, Deserialize)]
pub struct Config {
    pub clk_pin: u8,
    pub dt_pin: u8,
    pub device: String,
    pub startup_volume: i32,
}

pub fn load_config() -> Result<Config, Box<dyn Error>> {
    // Attempt to read the config file
    let config_str = fs::read_to_string("config.toml")
        .map_err(|_| Box::<dyn Error>::from("Config file not found"))?;

    // Attempt to parse the TOML string
    let config: Config = from_str(&config_str)
        .map_err(|e| Box::<dyn Error>::from(format!("Parsing config failed: {}", e)))?;

    if config.clk_pin > 27 {
        return Err(Box::<dyn Error>::from(
            "Invalid clk_pin in config. Valid range (0-27)",
        ));
    }

    if config.dt_pin > 27 {
        return Err(Box::<dyn Error>::from(
            "Invalid dt_pin in config. Valid range (0-27)",
        ));
    }

    if config.startup_volume.is_negative() || config.startup_volume > 100 {
        return Err(Box::<dyn Error>::from(
            "Invalid max_startup_volume in config. Valid range (0-100)",
        ));
    }

    Ok(config)
}
