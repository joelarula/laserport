pub const DMX_BAUD_RATE: u32 = 250_000;
pub const DMX_FRAME_SIZE: usize = 512;
pub const DMX_TEST_FRAME: [u8; DMX_FRAME_SIZE] = [0u8; DMX_FRAME_SIZE];
use serialport;
use std::time::Duration;
use std::io::Write;

/// Returns a Vec of DMX-compatible serial port names (ports that can be opened at 250_000 baud, 2 stop bits, and accept a DMX frame).
pub fn scan_dmx_ports() -> Vec<String> {
	let mut dmx_ports = Vec::new();
	match serialport::available_ports() {
		Ok(ports) => {
			for p in &ports {
				println!("Testing port: {} (type: {:?}, info: {:?})", p.port_name, p.port_type, p); // Print port info
				let result = serialport::new(&p.port_name, DMX_BAUD_RATE)
					.timeout(Duration::from_millis(10))
					.stop_bits(serialport::StopBits::Two)
					.open()
					.and_then(|mut port| {
						port.write_all(&DMX_TEST_FRAME)
							.map_err(|e| serialport::Error::new(serialport::ErrorKind::Io(std::io::ErrorKind::Other), format!("write failed: {}", e)))
					});
				if result.is_ok() {
					dmx_ports.push(p.port_name.clone());
				}
			}
		}
		Err(_) => {}
	}
	dmx_ports
}
