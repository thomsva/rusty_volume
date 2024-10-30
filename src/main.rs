mod display_updater;
mod rotary_controller;

use display_updater::DisplayUpdater;

use rotary_controller::RotaryController;
use rppal::gpio::Gpio;

use std::error::Error;
use std::{thread, time::Duration};

// GPIO pin constants
const CLK_PIN: u8 = 17; // GPIO pin for CLK
const DT_PIN: u8 = 27; // GPIO pin for DT

const INITIAL_VOLUME: i32 = 50;

fn main() -> Result<(), Box<dyn Error>> {
    let gpio = Gpio::new().expect("Failed to access GPIO");
    let mut volume_controller =
        RotaryController::new("Volume".to_string(), gpio, (CLK_PIN, DT_PIN), None, None);

    println!("Volume control starting...");
    let mut display_updater = DisplayUpdater::new(INITIAL_VOLUME)?;

    // Thread for handling volume control logic and wake/sleep behavior
    thread::spawn(move || loop {
        if volume_controller.handle_sleep() {
            volume_controller.update_volume();
            let _ = display_updater.update(volume_controller.get_value());
        }
    });

    loop {
        thread::sleep(Duration::from_millis(20));
    }
}
