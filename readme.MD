# Rusty_volume

Rusty_volume is a volume control app used and tested on a Raspberry Pi 4. It uses a rotary encoder to adjust volume and a small OLED display over the I2C interface.

As of 30.10.2024 the program is still deemed to be "work in progress" as it has not yet been in real use.

## Design goal

The main goal with the program is to make the use of the rotary encoder feel as responsive and reliable as possible.

## Implementation

- Interrupts are used to wake the volume control from sleep. Wakeup is instant.
- The volume control goes back to sleep after 3 seconds of inactivity.
- When active, the encoder CLK pin is polled continuously. This uses CPU capacity but when asleep it goes back to almost zero.
- Polling of the encoder pin is done in a separate CPU thread to keep it free from interference by other tasks.
- Other tasks include updating the display and updating the volume of the audio interface. These are done asyncronously when feasible and at suitable update intevals.
- Updating of the display and audio interface stops when there is nothing to update.

## Some observations during testing

Debouncing a mechanical rotary encoder was harder than expected. At some points it felt certain that the encoder was defective. There is actual mechanical bouncing but also electrical ringing from capacitance and inductance in the circuit. This can be mitigated by adding resistors and capacitors but it can also be done by programming.

- When continously polling the CLK pin and trying to detect a 0,5ms period of continuous observations from the new state, this period has to be restarted usually 5-15 times for each state change as the pin bounces back to the previous state.
- It seems enough for very high reliability to poll and debounce only one of the pins, and then when debounced check the state of the other pin to detect the rotation direction.
- The state of the pins changes and bounces only when the controller has just been moved. Other than that it's very stable.
- Good results can be achieved by relying more on interrupts instead of continuous polling. However test showed that the controller was slighly more responsive with continuous polling.
- When polling, one CPU core goes to almost 100% at 1,8Ghz. After some seconds of inactivity it goes back to almost nothing.
- The rotary encoder can be turned very fast without missing steps but not indefinately fast.

## Further development

- Test on other hardware
- Add instructions about connecting the encoder and the OLED display.
- Make it possible for user to easily change program parameters. Currently changes to the source code is needed to change the used GPIO pins or the debouncing timeouts.
- Better error handling and other measures for maximum reliability.
- Continue cleaning up code for readibility and reliability.
- Handling the screen updates has room for improvement. It works but could be made simpler, cleaner and better.
- Add some kind of performance measuring for optimizing the debouncing and comparing different encoders.
- Add other functionality relevant to RPi based music streaming.

# Install on a Raspberry Pi

The program is easiest to install by compiling it on the Raspberry Pi itself. There is also a deploy script for someone who is familiar with Rust who wants to compile on their own PC and move the finished binary over to the RPi. But the instruction below is for installing directly to the RPi.

## Tools needed

You need to make sure you have Git and Rust installed on the Raspberry Pi before installing the program. Log in to the Raspberry Pi using SSH and then install Git.

```bash
sudo apt-get install git
```

Install Rust. Copy the installation script from [the Rust install page](https://www.rust-lang.org/tools/install). The `~/.cargo/bin`directory needs to be added to the $PATH variable. Check with `echo $PATH`. The Rust installer attempts to update the path variable but it appears to require a restart for it to show up automatically.

## Build and test

To download and build the program:

```bash
cd rusty_volume
cargo build --release
```

Downloading everything and compiling takes a couple of minutes.

After compiling a folder called target should have appeared under the project folder. When in the project folder you can now start the program with

```bash
./target/release/rusty_volume
```

## Run rusty_volume as a Systemd service

In most use cases it makes sense to have the program run automatically every time the machine is started. This can be done by defining it as a systemd service.

Create a systemd service file

```bash
sudo nano /etc/systemd/system/rusty_volume.service
```

Copy the following as the file contents

```
[Unit]
Description=Volume Control Program
After=network.target

[Service]
User=pi
WorkingDirectory=/home/pi/rusty_volume
ExecStart=/home/pi/rusty_volume/target/release/rusty_volume
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

Enable and Start the Service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable rusty_volume.service
sudo systemctl start rusty_volume.service
```

Service status, stop service and service logs

```bash
sudo systemctl status rusty_volume.service
sudo systemctl stop rusty_volume.service
journalctl -u rusty_volume.service -f
```

## Uninstall systemd service

To remove the service, first stop the service, then disable it to prevent it from starting from boot. Then remove the file and restart Systemd. Here are the needed commands.

```bash
sudo systemctl stop volume_control.service
sudo systemctl disable volume_control.service
sudo rm /etc/systemd/system/volume_control.service
sudo systemctl daemon-reload
```