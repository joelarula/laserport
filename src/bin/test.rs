use serialport::{SerialPort, new};
use std::time::Duration;
use std::io::Write;

/// Laser control configuration
pub struct LaserControl {
    port: Box<dyn SerialPort>,
    universe: Vec<u8>,
}

impl LaserControl {
    /// Initialize laser control with a serial port
    pub fn new(port_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let port = serialport::new(port_name, 250_000)
            .timeout(Duration::from_millis(200))
            .stop_bits(serialport::StopBits::Two)
            .open()?;
        let universe = vec![0u8; 512];
        Ok(LaserControl { port, universe })
    }

    /// Set a specific DMX channel value (1-512, 0-255)
    pub fn set_channel(&mut self, channel: u16, value: u8) -> Result<(), Box<dyn std::error::Error>> {
        if channel < 1 || channel > 512 {
            return Err("Channel must be between 1 and 512".into());
        }
        let index = (channel - 1) as usize;
        self.universe[index] = value.clamp(0, 255);
        self.port.write_all(&self.universe)?;
        Ok(())
    }

    /// Set DMX address (CH2, 1-512 mapped to 0-255)
    pub fn set_address(&mut self, address: u16) -> Result<(), Box<dyn std::error::Error>> {
        if address < 1 || address > 512 {
            return Err("Address must be between 1 and 512".into());
        }
        let dmx_value = ((address - 1) as f32 / 511.0 * 255.0) as u8;
        self.universe[1] = dmx_value;
        self.port.write_all(&self.universe)?;
        Ok(())
    }

    // Expose port for direct access (for break simulation)
    pub fn get_port(&mut self) -> &mut Box<dyn SerialPort> {
        &mut self.port
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut laser = LaserControl::new("COM4")?; // Same port as FreeStyler
    println!("Testing laser to match FreeStyler with enhanced DMX emulation...");

    // Set DMX address to 1 (match FreeStyler patch)
    laser.set_address(1)?;
    println!("Set DMX address to 1");


    // Simulate DMX break and send frames with start code (as in test2.rs)
    let mut dmx_universe = vec![0u8; 512];
    for _ in 0..10 {
        // DMX break
    laser.port.set_break()?;
    std::thread::sleep(Duration::from_micros(100));
    laser.port.clear_break()?;
        // Build DMX frame: start code (0) + universe
        let mut frame = vec![0x00];
        frame.extend_from_slice(&dmx_universe);
        laser.port.write_all(&frame)?;
        println!("Sent DMX frame: CH1 = {}, CH4 = {}", dmx_universe[0], dmx_universe[3]);
        std::thread::sleep(Duration::from_millis(20));

        // Turn laser on with pattern for next frame
        dmx_universe[0] = 255;
        dmx_universe[3] = 100;
    }


    // Extended test: continuous DMX frames with correct break and start code
    println!("Extended test with continuous frames...");
    dmx_universe[0] = 255;
    dmx_universe[3] = 100;
    for _ in 0..50 {
    laser.port.set_break()?;
    std::thread::sleep(Duration::from_micros(100));
    laser.port.clear_break()?;
        let mut frame = vec![0x00];
        frame.extend_from_slice(&dmx_universe);
        laser.port.write_all(&frame)?;
        std::thread::sleep(Duration::from_millis(20));
    }

    // Clean up
    dmx_universe[0] = 0;
    dmx_universe[3] = 0;
    laser.port.set_break()?;
    std::thread::sleep(Duration::from_micros(100));
    laser.port.clear_break()?;
    let mut frame = vec![0x00];
    frame.extend_from_slice(&dmx_universe);
    laser.port.write_all(&frame)?;
    println!("Test complete. Laser turned off.");

    Ok(())
}