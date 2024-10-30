use std::{
    thread,
    time::{Duration, Instant},
};

use embedded_graphics::{
    mono_font::{ascii::FONT_9X18, MonoTextStyle},
    pixelcolor::BinaryColor,
    prelude::*,
    text::Text,
};
use linux_embedded_hal::I2cdev;
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};
use std::error::Error;

const DISPLAY_INTERVAL: Duration = Duration::from_millis(20);
pub struct DisplayUpdater {
    volume: i32,
    last_update: Instant,
    display:
        Ssd1306<I2CInterface<I2cdev>, DisplaySize128x64, BufferedGraphicsMode<DisplaySize128x64>>,
}

impl DisplayUpdater {
    /// Creates a new Display_updater with an initial value.
    /// fn main() -> Result<(), Box<dyn Error>> {
    pub fn new(volume: i32) -> Result<Self, Box<dyn Error>> {
        // Initialize I2C communication
        let i2c = I2cdev::new("/dev/i2c-1")?;
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

        Ok(Self {
            display,
            volume,
            last_update: Instant::now(),
        })
    }

    pub fn update(&mut self, volume: i32) -> Result<(), Box<dyn Error>> {
        if volume == self.volume {
            return Ok(());
        }
        if self.last_update.elapsed() < DISPLAY_INTERVAL {
            self.display.clear_buffer();
            //return Ok(());
        }

        let volume_text = format!("Volume: {}", volume);

        let _ = self.write_to_display(volume_text);
        self.volume = volume;
        Ok(())
    }

    pub fn show_welcome(&mut self) -> Result<(), Box<dyn Error>> {
        let _ = self.write_to_display(String::from("Starting..."));
        thread::sleep(Duration::from_secs(2));
        let volume_text = format!("Volume: {}", self.volume);
        let _ = self.write_to_display(volume_text);
        Ok(())
    }

    fn write_to_display(&mut self, volume_text: String) -> Result<(), Box<dyn Error>> {
        self.display.clear(BinaryColor::Off).unwrap();

        let text_style = MonoTextStyle::new(&FONT_9X18, BinaryColor::On);

        Text::new(&volume_text, Point::new(0, 9), text_style)
            .draw(&mut self.display)
            .map_err(|e| format!("Failed to draw text: {:?}", e))?;
        // Flush the display to update it with the new drawing
        self.display
            .flush()
            .map_err(|e| format!("Failed to flush display: {:?}", e))?;
        self.last_update = Instant::now();
        Ok(())
    }
}
