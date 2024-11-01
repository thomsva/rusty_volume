mod amixer_updater;
mod display_updater;
mod load_config;
mod rotary_controller;

use amixer_updater::AmixerUpdater;
use display_updater::DisplayUpdater;

use load_config::load_config;
use rotary_controller::RotaryController;
use rppal::gpio::Gpio;

use std::error::Error;
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, Instant};

const MAIN_LOOP_INTERVAL: Duration = Duration::from_millis(50);

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    match load_config() {
        Ok(config) => {
            println!(
                "Config loaded - clk: {}, dt: {} device {} max_startup_volume {}",
                config.clk_pin, config.dt_pin, config.device, config.startup_volume
            );

            let gpio = Gpio::new().expect("Failed to access GPIO");
            let mut volume_controller = RotaryController::new(
                "Volume".to_string(),
                gpio,
                (config.clk_pin, config.dt_pin),
                None,
                None,
            );

            println!("Volume control starting...");
            let mut display_updater = DisplayUpdater::new(0)?;
            let mut amixer_updater = AmixerUpdater::new(Some(config.device), 0)?;
            let _ = display_updater.show_welcome();

            let starting_volume = amixer_updater
                .get_starting_volume(config.startup_volume)
                .await;
            volume_controller.value = starting_volume;

            let _ = display_updater.update(starting_volume);
            amixer_updater.update(starting_volume).await?;

            // Create a streaming channel to send information to main thread
            let (tx, rx) = channel();

            // Thread for handling volume control logic and wake/sleep behavior
            thread::spawn(move || loop {
                if volume_controller.handle_sleep() {
                    if let Some(new_volume) = volume_controller.update_volume() {
                        //display_updater.update(new_volume).unwrap();
                        // let _ = amixer_updater.update(new_volume);
                        tx.send(new_volume).unwrap();
                    }
                }
            });

            loop {
                // Display updating in main loop.
                // Volume information is received as mpsc messages from the other thread.

                // Attempt to read all available messages non-blocking.
                // Keep only latest message if there are many.

                let main_loop_started = Instant::now();
                let mut latest_volume = None;
                while let Ok(msg) = rx.try_recv() {
                    latest_volume = Some(msg);
                }

                // If no new messages were received, block until one arrives
                if latest_volume.is_none() {
                    latest_volume = Some(rx.recv()?);
                }

                // Update display with latest volume
                if let Some(vol) = latest_volume {
                    let _ = display_updater.update(vol);
                    let _ = amixer_updater.update(vol).await;
                }

                while main_loop_started.elapsed() < MAIN_LOOP_INTERVAL {
                    std::thread::sleep(Duration::from_millis(1));
                }
            }
        }
        Err(err_msg) => {
            eprintln!("Error reading config file: {}", err_msg);
            std::process::exit(1);
        }
    }
}
