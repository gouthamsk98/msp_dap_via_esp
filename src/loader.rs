use serialport::{ DataBits, FlowControl, Parity, SerialPort, StopBits };
use std::io::{ self, Write };
use std::time::Duration;
use crate::protocol::{ ProtocolHandler, SWDCommand };
use tracing::info;
pub struct SerialLoader {
    port: Box<dyn SerialPort>,
}

impl SerialLoader {
    /// Create a new ARM debug serial connection
    pub fn new(port_name: &str, baud_rate: u32) -> Result<Self, Box<dyn std::error::Error>> {
        let port = serialport
            ::new(port_name, baud_rate)
            .timeout(Duration::from_millis(1000))
            .data_bits(DataBits::Eight)
            .flow_control(FlowControl::None)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .open()?;

        Ok(SerialLoader { port })
    }
    /// Halt the Program
    pub fn halt(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let command = ProtocolHandler::new(SWDCommand::Halt);
        self.port.write_all(&command.write_frame())?;
        self.port.flush()?;

        // Wait for response
        std::thread::sleep(Duration::from_millis(10));

        // Read response to clear buffer
        let mut buffer = [0; 256];
        match self.port.read(&mut buffer) {
            Ok(_) => {}
            Err(_) => {} // Ignore timeout errors
        }

        Ok(())
    }
    /// Resume the Program
    pub fn resume(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let command = ProtocolHandler::new(SWDCommand::Resume);
        self.port.write_all(&command.write_frame())?;
        self.port.flush()?;
        // Wait for response
        std::thread::sleep(Duration::from_millis(10));
        // Read response to clear buffer
        let mut buffer = [0; 256];
        match self.port.read(&mut buffer) {
            Ok(_) => {
                todo!();
            }
            Err(_) => {} // Ignore timeout errors
        }
        Ok(())
    }

    /// Write to memory-mapped register (equivalent to OpenOCD's mww command)
    pub fn write_word(
        &mut self,
        address: u32,
        value: u32
    ) -> Result<(), Box<dyn std::error::Error>> {
        let command = ProtocolHandler::new(SWDCommand::Write {
            write_address: address,
            write_data: vec![
                ((value >> 24) & 0xff) as u8,
                ((value >> 16) & 0xff) as u8,
                ((value >> 8) & 0xff) as u8,
                (value & 0xff) as u8
            ],
        });
        // let command = format!("mww 0x{:08X} 0x{:08X}\n", address, value);
        self.port.write_all(&command.write_frame())?;
        self.port.flush()?;

        // Wait for response
        std::thread::sleep(Duration::from_millis(10));

        // Read response to clear buffer
        let mut buffer = [0; 256];
        match self.port.read(&mut buffer) {
            Ok(_) => {}
            Err(_) => {} // Ignore timeout errors
        }

        Ok(())
    }

    /// Read from memory-mapped register (equivalent to OpenOCD's mdw command)
    pub fn read_word(&mut self, address: u32) -> Result<u32, Box<dyn std::error::Error>> {
        let command = ProtocolHandler::new(SWDCommand::Read {
            start_address: address,
            length: 4, // Read 4 bytes (u32)
        });
        self.port.write_all(&command.write_frame())?;
        self.port.flush()?;

        // Wait for response
        std::thread::sleep(Duration::from_millis(50));

        // Read response
        let mut buffer = [0; 256];
        let bytes_read = self.port.read(&mut buffer)?;
        match command.read_frame(&buffer[..bytes_read]) {
            Ok(data) => {
                if data.len() < 4 {
                    return Err("Received less than 4 bytes from read_word".into());
                }
                // Convert the first 4 bytes to u32
                let value =
                    ((data[0] as u32) << 24) |
                    ((data[1] as u32) << 16) |
                    ((data[2] as u32) << 8) |
                    (data[3] as u32);
                Ok(value)
            }
            Err(e) => {
                // Log the error and return it
                info!("Error reading word from address 0x{:08X}: {}", address, e);
                Err(e.into())
            }
        }
    }

    /// Write PC register index to DCRSR and read PC value from DCRDR
    pub fn read_pc_register(&mut self) -> Result<u32, Box<dyn std::error::Error>> {
        // Step 1: Write register index (0x0F) to DCRSR at 0xE000EDF4
        const DCRSR_ADDR: u32 = 0xe000edf4;
        const DCRDR_ADDR: u32 = 0xe000edf8;
        const PC_REG_INDEX: u32 = 0x0f;

        info!("Writing PC register index (0x{:02X}) to DCRSR (0x{:08X})", PC_REG_INDEX, DCRSR_ADDR);
        self.write_word(DCRSR_ADDR, PC_REG_INDEX)?;

        // Small delay to ensure the register transfer completes
        std::thread::sleep(Duration::from_millis(10));

        // Step 2: Read the PC value from DCRDR at 0xE000EDF8
        let pc_value = self.read_word(DCRDR_ADDR)?;
        info!("Read PC value: 0x{:08X} from DCRDR (0x{:08X})", pc_value, DCRDR_ADDR);
        Ok(pc_value)
    }

    /// Read any ARM Cortex-M register by index
    pub fn read_register(&mut self, reg_index: u32) -> Result<u32, Box<dyn std::error::Error>> {
        // Debug Registers
        const DCRSR_ADDR: u32 = 0xe000edf4;
        const DCRDR_ADDR: u32 = 0xe000edf8;

        self.write_word(DCRSR_ADDR, reg_index)?;

        std::thread::sleep(Duration::from_millis(10));
        let value = self.read_word(DCRDR_ADDR)?;
        info!("Read register index 0x{:02X} value: 0x{:08X}", reg_index, value);
        Ok(value)
    }
    pub fn set_breakpoint(&mut self, address: u32) -> Result<(), Box<dyn std::error::Error>> {
        // FPB Registers
        const FPB_CTRL: u32 = 0xe0002000; // (Control register)
        const FP_COMP0: u32 = 0xe0002008; // (Comparator 0)
        const FP_COMP1: u32 = 0xe000200c; // (Comparator 1)
        const FP_COMP2: u32 = 0xe0002014; // (Comparator 2)
        const FP_COMP3: u32 = 0xe000201c; // (Comparator 3)

        todo!("Implement set_breakpoint method");
    }
    fn software_crc(data: &[u8], length: usize) -> [u8; 4] {
        const CRC32_POLYNOMIAL: u32 = 0xedb88320; // IEEE 802.3 CRC-32 polynomial
        let mut crc = 0xffffffff_u32;

        for i in 0..length {
            let byte = data[i] as u32;
            crc = crc ^ byte;
            for _ in 0..8 {
                let mask = (crc & 1).wrapping_neg();
                crc = (crc >> 1) ^ (CRC32_POLYNOMIAL & mask);
            }
        }

        // Return as little-endian byte array
        [
            (crc & 0xff) as u8, // Least significant byte
            ((crc >> 8) & 0xff) as u8,
            ((crc >> 16) & 0xff) as u8,
            ((crc >> 24) & 0xff) as u8, // Most significant byte
        ]
    }

    pub fn check_crc(frame: &[u8]) -> Result<bool, String> {
        let data_length = ((frame[3] as u16) << 8) | (frame[2] as u16);
        let data = &frame[4..4 + (data_length as usize)];
        let crc = Self::software_crc(data, data_length as usize);

        // check if crc and last 4 bytes of frame are same
        let check_crc_value =
            ((crc[0] as u32) << 24) |
            ((crc[1] as u32) << 16) |
            ((crc[2] as u32) << 8) |
            (crc[3] as u32);

        let frame_crc_value =
            ((frame[frame.len() - 4] as u32) << 24) |
            ((frame[frame.len() - 3] as u32) << 16) |
            ((frame[frame.len() - 2] as u32) << 8) |
            (frame[frame.len() - 1] as u32);

        if check_crc_value != frame_crc_value {
            // self.debug("CRC Check Failed");
            return Err("CRC Check Failed".to_string());
        }

        Ok(true)
    }
}
