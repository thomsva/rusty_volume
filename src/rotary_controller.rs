use rppal::gpio::{Gpio, InputPin, Trigger};
use std::time::{Duration, Instant};

const DEBOUNCE_DELAY: Duration = Duration::from_micros(500);
const RESET_DELAY: Duration = Duration::from_millis(4);
const SLEEP_DELAY: Duration = Duration::from_secs(3);

pub struct RotaryController {
    name: String,
    clk_pin: InputPin,
    dt_pin: InputPin,
    value: i32,
    min_value: i32,
    max_value: i32,
    last_activity: Instant,
    debounce_start: Instant,
    waiting: bool,
}

impl RotaryController {
    /// Creates a new RotaryController with optional min and max values.
    pub fn new(
        name: String,
        gpio: Gpio,
        gpio_pin_numbers: (u8, u8),
        min_volume: Option<i32>,
        max_volume: Option<i32>,
    ) -> Self {
        let mut clk_pin = gpio
            .get(gpio_pin_numbers.0)
            .expect("Failed to get CLK pin")
            .into_input_pullup();
        let dt_pin = gpio
            .get(gpio_pin_numbers.1)
            .expect("Failed to get DT pin")
            .into_input_pullup();
        clk_pin
            .set_interrupt(Trigger::Both, None)
            .expect("Failed to set interrupt");
        Self {
            name,
            clk_pin,
            dt_pin,
            value: 50,
            min_value: min_volume.unwrap_or(0),
            max_value: max_volume.unwrap_or(100),
            last_activity: Instant::now(),
            debounce_start: Instant::now(),
            waiting: false,
        }
    }
    /// Blocks if in sleep mode. Checks if the controller should enter sleep mode and handles waking.
    pub fn handle_sleep(&mut self) -> bool {
        if self.last_activity.elapsed() >= SLEEP_DELAY {
            println!("Volume control going to sleep");
            //Blocks until next activity on the CLK pin
            if self.clk_pin.poll_interrupt(true, None).is_ok() {
                self.last_activity = Instant::now();
                println!("Wake up volume control");
                return true;
            }
            return false;
        }
        true
    }
    /// Updates the volume if encoder activity has been detected.
    /// Returns `Some(new_value)` if the volume was updated, otherwise `None`.
    pub fn update_volume(&mut self) -> Option<i32> {
        if self.clk_detected() {
            // Activity detected. Update value and prepare to wait for next activity.
            if !self.dt_pin.is_high() {
                self.value = (self.value + 1).min(self.max_value);
            } else {
                self.value = (self.value - 1).max(self.min_value);
            }
            println!("{}: {} ", self.name, self.value);
            self.wait_for_reset();
            self.last_activity = Instant::now();

            // Return the new value if it was updated
            return Some(self.value);
        }

        // Return None if no update was made
        None
    }

    /// Detects valid encoder activity.
    fn clk_detected(&mut self) -> bool {
        if !self.clk_pin.is_high()
            && self.waiting
            && self.debounce_start.elapsed() >= DEBOUNCE_DELAY
        {
            self.waiting = false;
            return true;
        } else if !self.clk_pin.is_high() && !self.waiting {
            self.waiting = true;

            self.debounce_start = Instant::now();
        } else if self.clk_pin.is_high() {
            self.waiting = false;
        }
        false
    }

    /// Waits until the CLK pin has stabilized before next activity can be reliably registered.
    fn wait_for_reset(&mut self) {
        self.waiting = false;
        while !self.clk_pin.is_high()
            || !self.waiting
            || (self.waiting && self.debounce_start.elapsed() < RESET_DELAY)
        {
            if self.clk_pin.is_high() {
                if !self.waiting {
                    self.debounce_start = Instant::now();
                    self.waiting = true;
                }
            } else {
                self.waiting = false;
            }
        }
        self.waiting = false;
    }
}
