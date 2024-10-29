mod rotary_controller;

use embedded_graphics::{
    mono_font::{ascii::FONT_9X18, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use linux_embedded_hal::I2cdev;
use rotary_controller::RotaryController;
use rppal::gpio::Gpio;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use std::{error::Error, iter::Inspect, process::Command, time::Instant};
use std::{thread, time::Duration};

// GPIO pin constants
const CLK_PIN: u8 = 17; // GPIO pin for CLK
const DT_PIN: u8 = 27; // GPIO pin for DT

fn get_volume() -> Result<i32, Box<dyn Error>> {
    let output = Command::new("amixer").args(&["get", "PCM"]).output()?;

    // Check if the command executed successfully
    if !output.status.success() {
        return Err("Failed to execute amixer command".into());
    }

    // Convert the output to a string and find the volume level
    let stdout = String::from_utf8_lossy(&output.stdout);
    let volume_line = stdout
        .lines()
        .find(|line| line.contains("Mono:")) // Find the line with Mono volume
        .ok_or("Couldn't find Mono volume line")?;

    // Example output: "  Mono: Playback -4919 [50%] [-49.19dB] [on]"
    // Extract the percentage
    let volume_str = volume_line
        .split_whitespace()
        .nth(3) // The percentage is still the fourth item in the line
        .ok_or("Couldn't extract volume percentage")?;

    // Remove brackets and parse the percentage into an integer
    let volume_percentage = volume_str
        .trim_matches(|c| c == '[' || c == ']' || c == '%')
        .parse::<i32>()
        .map_err(|_| "Failed to parse volume percentage")?;

    Ok(volume_percentage)
}

fn set_volume(volume: i32) -> Result<(), Box<dyn Error>> {
    // Ensure the volume is within valid bounds
    if volume < 0 || volume > 100 {
        return Err("Volume must be between 0 and 100".into());
    }

    Command::new("amixer")
        .args(&["set", "PCM", &format!("{}%", volume)])
        .output()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let gpio = Gpio::new().expect("Failed to access GPIO");
    let mut volume_controller =
        RotaryController::new("Volume".to_string(), gpio, (CLK_PIN, DT_PIN), None, None);

    println!("Volume control starting...");
    let volume_interval = Duration::from_millis(20);
    let mut volume_update_time = Instant::now();
    // Thread for handling volume control logic and wake/sleep behavior
    thread::spawn(move || loop {
        if volume_controller.handle_sleep() {
            volume_controller.update_volume();
            if volume_update_time.elapsed() > volume_interval {
                let _ = set_volume(volume_controller.get_value());
                volume_update_time = Instant::now();
            }
        }
    });

    // Initialize I2C communication
    let i2c =
        I2cdev::new("/dev/i2c-1").map_err(|e| format!("Failed to initialize I2C: {:?}", e))?;
    let interface = I2CDisplayInterface::new(i2c);

    // Initialize the display in buffered graphics mode
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display
        .init()
        .map_err(|e| format!("Failed to initialize display: {:?}", e))?;
    display
        .flush()
        .map_err(|e| format!("Failed to flush display: {:?}", e))?;

    // Set up text style
    let text_style = MonoTextStyle::new(&FONT_9X18, BinaryColor::On);

    // Main thread for display updates
    loop {
        display.clear(BinaryColor::Off).unwrap();

        // Try to receive the volume update
        let volume = get_volume()?;
        let volume_text = format!("Volume: {}", volume);

        //let volume_text = format!("Volume: {}", 55);
        Text::new(&volume_text, Point::new(0, 9), text_style)
            .draw(&mut display)
            .map_err(|e| format!("Failed to draw text: {:?}", e))?;
        // Flush the display to update it with the new drawing
        display
            .flush()
            .map_err(|e| format!("Failed to flush display: {:?}", e))?;
        thread::sleep(Duration::from_millis(20));
    }
}
