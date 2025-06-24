use serialport::{ SerialPortType, SerialPort };
use rusb::{ Context, UsbContext };
use std::io::{ Write, Read };
use std::time::Duration;
use tracing::info;

// Define the target PID as a macro - change this to your specific device PID
macro_rules! TARGET_PID {
    () => {
        0x8055u16
    };
}

pub fn connect_to_device() -> Result<Box<dyn SerialPort>, Box<dyn std::error::Error>> {
    let target_pid = TARGET_PID!();

    // First, check if a USB device with the target PID is connected
    if !is_device_connected(target_pid)? {
        return Err(format!("Device with PID 0x{:04X} not found", target_pid).into());
    }

    info!("Device with PID 0x{:04X} found", target_pid);

    // Get available serial ports
    let ports = serialport::available_ports()?;

    if ports.is_empty() {
        return Err("No serial ports found".into());
    }

    // Try to find a serial port that might correspond to our USB device
    let mut target_port = None;

    for port in &ports {
        info!("Found port: {}", port.port_name);

        // Check if this is a USB serial port with matching PID
        if let SerialPortType::UsbPort(usb_info) = &port.port_type {
            if usb_info.pid == target_pid {
                target_port = Some(port.port_name.clone());
                info!("Found matching USB serial port: {}", port.port_name);
                break;
            }
        }
    }

    // If no matching USB port found, try the first available port
    let port_name = target_port.unwrap_or_else(|| {
        info!("No matching USB port found, using first available port: {}", ports[0].port_name);
        ports[0].port_name.clone()
    });

    // Open the serial port
    let port = serialport::new(&port_name, 115200).timeout(Duration::from_millis(1000)).open()?;

    info!("Opened serial port: {}", port_name);

    Ok(port)
}

pub fn write_to_device(
    port: &mut Box<dyn SerialPort>,
    data: &[u8]
) -> Result<(), Box<dyn std::error::Error>> {
    // Send the data
    port.write_all(data)?;
    port.flush()?;

    info!("Sent {} bytes: {:02X?}", data.len(), data);

    Ok(())
}

pub fn read_from_device(
    port: &mut Box<dyn SerialPort>,
    buffer_size: usize,
    timeout_ms: u64
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Set read timeout
    port.set_timeout(Duration::from_millis(timeout_ms))?;

    let mut buffer = vec![0u8; buffer_size];
    let mut received_data = Vec::new();

    // Try to read data
    match port.read(&mut buffer) {
        Ok(bytes_read) => {
            if bytes_read > 0 {
                received_data.extend_from_slice(&buffer[..bytes_read]);
                info!("Received {} bytes: {:02X?}", bytes_read, &buffer[..bytes_read]);
            } else {
                info!("No data received within timeout period");
            }
        }
        Err(e) => {
            // Check if it's a timeout error
            if e.kind() == std::io::ErrorKind::TimedOut {
                info!("Read timeout - no data received from ESP32");
            } else {
                return Err(e.into());
            }
        }
    }

    Ok(received_data)
}

pub fn write_and_read(
    port: &mut Box<dyn SerialPort>,
    data: &[u8],
    read_buffer_size: usize,
    read_timeout_ms: u64
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Send data first
    write_to_device(port, data)?;

    // Small delay to allow ESP32 to process and respond
    std::thread::sleep(Duration::from_millis(10));

    // Read response
    read_from_device(port, read_buffer_size, read_timeout_ms)
}

pub fn is_device_connected(target_pid: u16) -> Result<bool, Box<dyn std::error::Error>> {
    let context = Context::new()?;

    for device in context.devices()?.iter() {
        let device_desc = device.device_descriptor()?;

        if device_desc.product_id() == target_pid {
            info!(
                "Found USB device - VID: 0x{:04X}, PID: 0x{:04X}",
                device_desc.vendor_id(),
                device_desc.product_id()
            );
            return Ok(true);
        }
    }

    Ok(false)
}

// Helper function to check if device is connected using the macro
pub fn check_target_device_connected() -> Result<bool, Box<dyn std::error::Error>> {
    is_device_connected(TARGET_PID!())
}

// Convenience function to send the specific byte sequence
pub fn send_debug_sequence(
    port: &mut Box<dyn SerialPort>
) -> Result<(), Box<dyn std::error::Error>> {
    let debug_data: [u8; 8] = [0xff, 0xf9, 0x00, 0x02, 0xc2, 0x00, 0xf5, 0xe7];
    write_to_device(port, &debug_data)
}

// Convenience function to send the specific byte sequence and read response
pub fn send_debug_sequence_and_read(
    port: &mut Box<dyn SerialPort>
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let debug_data: [u8; 8] = [0xff, 0xf9, 0x00, 0x02, 0xc2, 0x00, 0xf5, 0xe7];
    write_and_read(port, &debug_data, 256, 2000) // 256 byte buffer, 2 second timeout
}
