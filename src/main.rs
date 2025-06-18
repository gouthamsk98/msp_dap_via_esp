mod serial;
mod loader;
mod protocol;
use std::thread;
use std::time::Duration;

fn main() {
    // The byte sequence to send
    let data_to_send: [u8; 8] = [0xff, 0xf9, 0x00, 0x02, 0xc1, 0x00, 0xf5, 0xe7];
    println!("Sending data to device: {:?}", data_to_send);
    // find crc of 0xff, 0xf9, 0x00, 0x02, 0xc1
    let crc = protocol::compute_crc(&data_to_send, data_to_send.len());
    // Print the calculated CRC in hex
    println!("Calculated CRC: {:02x}", crc);

    let mut new_data_to_send = data_to_send.clone();
    new_data_to_send[data_to_send.len() - 3] = crc; // Replace with calculated CRC
    // First connect to the device
    match serial::connect_to_device() {
        Ok(mut port) => {
            println!("Successfully connected to device");

            // Now send data to the connected device
            match serial::write_to_device(&mut port, &new_data_to_send) {
                Ok(_) => {
                    println!("Successfully sent data to device");

                    // Give the device a moment to process and respond
                    // thread::sleep(Duration::from_millis(100));

                    // Now try to read data from the device
                    match serial::read_from_device(&mut port, 9, 4000) {
                        Ok(received_data) => {
                            if received_data.is_empty() {
                                println!("No data received from device");
                            } else {
                                println!("Received {} bytes from device:", received_data.len());
                                print!("Data: ");
                                for byte in &received_data {
                                    print!("{:02x} ", byte);
                                }
                                println!();
                            }
                        }
                        Err(e) => eprintln!("Error reading from device: {}", e),
                    }
                }
                Err(e) => eprintln!("Error sending data: {}", e),
            }
        }
        Err(e) => eprintln!("Error connecting to device: {}", e),
    }
}
