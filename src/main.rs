mod amixer_updater;
mod display_updater;
mod rotary_controller;

use amixer_updater::AmixerUpdater;
use display_updater::DisplayUpdater;

use rotary_controller::RotaryController;
use rppal::gpio::Gpio;

use std::error::Error;
use std::sync::mpsc::channel;
use std::thread;

// GPIO pin constants
const CLK_PIN: u8 = 17; // GPIO pin for CLK
const DT_PIN: u8 = 27; // GPIO pin for DT

const INITIAL_VOLUME: i32 = 50;
const SOUND_CONTROL: &str = "PCM";

fn main() -> Result<(), Box<dyn Error>> {
    let gpio = Gpio::new().expect("Failed to access GPIO");
    let mut volume_controller =
        RotaryController::new("Volume".to_string(), gpio, (CLK_PIN, DT_PIN), None, None);

    println!("Volume control starting...");
    let mut display_updater = DisplayUpdater::new(INITIAL_VOLUME)?;
    let mut amixer_updater = AmixerUpdater::new(Some(SOUND_CONTROL.to_string()), INITIAL_VOLUME)?;
    let _ = display_updater.show_welcome();

    // Create a streaming channel to send information to main thread
    let (tx, rx) = channel();

    // Thread for handling volume control logic and wake/sleep behavior
    thread::spawn(move || loop {
        if volume_controller.handle_sleep() {
            if let Some(new_volume) = volume_controller.update_volume() {
                //display_updater.update(new_volume).unwrap();
                let _ = amixer_updater.update(new_volume);
                tx.send(new_volume).unwrap();
            }
        }
    });

    loop {
        let mut latest_volume = None;
        while let Ok(msg) = rx.try_recv() {
            latest_volume = Some(msg);
        }

        // Process only the latest received volume if there is one
        if let Some(vol) = latest_volume {
            //println!("rx: {}", vol);
            let _ = display_updater.update(vol);
        }
    }
}
