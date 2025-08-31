use serialport::{SerialPort, new};
use laserport::dmx;
use std::time::Duration;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {


    // List all DMX-compatible ports using dmx::available_dmx_ports
    println!("Scanning available serial ports for DMX compatibility:");
    let dmx_ports = dmx::scan_dmx_ports();
    let baud_rate = 250_000; // DMX standard baud rate
    if dmx_ports.is_empty() {
        println!("No DMX-compatible ports found.");
        return Ok(());
    }


    // Select DMX port: auto if one, prompt if multiple
    let port_name = if dmx_ports.len() == 1 {
        println!("\nOnly one DMX-compatible port found: {}", dmx_ports[0]);
        &dmx_ports[0]
    } else {
        println!("\nMultiple DMX-compatible ports found:");
        for (i, p) in dmx_ports.iter().enumerate() {
            println!("  [{}] {}", i + 1, p);
        }
        use std::io::{self, Write};
        loop {
            print!("Select port by number: ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if let Ok(idx) = input.trim().parse::<usize>() {
                if idx >= 1 && idx <= dmx_ports.len() {
                    break &dmx_ports[idx - 1];
                }
            }
            println!("Invalid selection. Please enter a valid number.");
        }
    };
    println!("\nUsing DMX port: {}", port_name);
    let mut port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(100))
        .stop_bits(serialport::StopBits::Two)
        .open()?;

    println!("Connected to DMX adapter. Testing CH1 (Shutter) features...");

    // DMX universe (512 bytes, initialized to 0)
    let mut dmx_universe = vec![0u8; 512];
// Set DMX address to 1
dmx_universe[1] = 0; // CH2: Address 1 (0 maps to 1 in some implementations)
port.write_all(&dmx_universe)?;
println!("Set DMX address to 1");

// Test CH1 and CH4: Toggle shutter and activate built-in visual
for _ in 0..5 { // Repeat 5 times
    // Turn laser off (CH1 = 0)
    dmx_universe[0] = 0; // CH1 is index 0 in 1-based DMX addressing
    dmx_universe[3] = 0; // CH4: Reset pattern
    port.write_all(&dmx_universe)?;
    println!("CH1 set to 0 (Off), CH4 set to 0 (No Pattern)");
    std::thread::sleep(Duration::from_secs(2));

    // Turn laser on with built-in Christmas graphic (CH4 = 100)
    dmx_universe[0] = 255; // CH1: Shutter on
    dmx_universe[3] = 100; // CH4: Christmas graphic (example value)
    if let Err(e) = port.write_all(&dmx_universe) {
        println!("Write error: {}. Check connection or address.", e);
    } else {
        println!("CH1 set to 255 (On), CH4 set to 100 (Christmas Graphic)");
    }
    std::thread::sleep(Duration::from_secs(3)); // Increased delay for pattern display
}

    // Clean up: Turn off laser
    dmx_universe[0] = 0;
    port.write_all(&dmx_universe)?;
    println!("Test complete. Laser turned off.");

    Ok(())
}