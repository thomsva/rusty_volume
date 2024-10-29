mod rotary_controller;
use rotary_controller::RotaryController;
use rppal::gpio::Gpio;
use std::{thread, time::Duration};

// GPIO pin constants
const CLK_PIN: u8 = 17; // GPIO pin for CLK
const DT_PIN: u8 = 27; // GPIO pin for DT

fn main() {
    let gpio = Gpio::new().expect("Failed to access GPIO");
    let mut volume_controller =
        RotaryController::new("Volume".to_string(), gpio, (CLK_PIN, DT_PIN), None, None);

    println!("Volume control starting...");

    // Thread for handling volume control logic and wake/sleep behavior
    thread::spawn(move || loop {
        if volume_controller.handle_sleep() {
            volume_controller.update_volume();
        }
    });

    // Main thread
    loop {
        // Update display logic here
        thread::sleep(Duration::from_secs(1));
    }
}
