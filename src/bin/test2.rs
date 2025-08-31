use serialport::{self, DataBits, FlowControl, Parity, StopBits};
use std::io::Write;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port_name = "COM4"; // Adjust as needed
    let mut port = serialport::new(port_name, 250_000)
        .data_bits(DataBits::Eight)
        .flow_control(FlowControl::None)
        .parity(Parity::None)
        .stop_bits(StopBits::Two)
        .timeout(Duration::from_millis(10))
        .open()?;

    // Corrected: Remove argument from set_break
    port.set_break(); // Emulate break
    std::thread::sleep(Duration::from_micros(100));
    port.clear_break(); // Clear break (no arg needed)

    let mut frame = vec![0x00]; // Start code
    frame.extend_from_slice(&[255, 70, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    frame.resize(512, 0);
    port.write_all(&frame)?;

    Ok(())}