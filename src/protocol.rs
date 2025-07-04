use std::result;
use tracing::info;
use crc::{ Crc, * };

pub enum SWDCommand {
    Halt,
    Resume,
    ReadBytes {
        start_address: u32,
        length: u32,
    },
    ReadWord {
        start_address: u32,
    },
    ReadWords {
        start_address: u32,
        length: u32,
    },
    Write {
        write_address: u32,
        write_data: Vec<u8>,
    },
}

pub struct ProtocolHandler {
    command: SWDCommand,
}

impl ProtocolHandler {
    pub const HEADER: [u8; 2] = [0xff, 0xf9];
    pub const FOOTER: [u8; 2] = [0xf5, 0xe7];
    pub const MAX_DATA_LENGTH: usize = 4096; // Maximum
    pub const HALT_COMMAND: u8 = 0xc1;
    pub const RESUME_COMMAND: u8 = 0xc2;
    pub const READ_WORD: u8 = 0xc3;
    pub const READ_BYTES_COMMAND: u8 = 0xc6;
    pub const READ_WORDS_COMMAND: u8 = 0xc7;
    pub const WRITE_COMMAND: u8 = 0xc4;
    pub const ACK_OFFSET: usize = 5; // Offset for ACK in the response frame
    pub const HALT_ACK: u8 = 0xd1;
    pub const HALT_ERROR: u8 = 0xe1;
    pub const RESUME_ACK: u8 = 0xd1;
    pub const RESUME_ERROR: u8 = 0xe1;
    pub const READ_ACK: u8 = 0xd2;
    pub const READ_ERROR: u8 = 0xe3;
    pub const WRITE_ACK: u8 = 0xd1;
    pub const WRITE_ERROR: u8 = 0xe1;
    pub const WORD_SIZE: usize = 4; // Size of a word in bytes

    pub fn new(command: SWDCommand) -> Self {
        ProtocolHandler { command }
    }

    pub fn compute_crc(data: &[u8], length: usize) -> u8 {
        let mut crc = 0x00u8; // Initial CRC value
        for i in 2..length - 3 {
            crc ^= data[i];
            for _ in 0..8 {
                crc = if (crc & 0x80) != 0 { (crc << 1) ^ 0x07 } else { crc << 1 }; // Polynomial 0x07
            }
        }
        crc
    }
    pub fn write_frame(&self) -> Vec<u8> {
        //match based SWDCommand
        let mut data = Vec::new();
        match &self.command {
            SWDCommand::Halt => {
                // Frame Format: ff f9 len0 len1 cmd crc f5 e7
                data.extend_from_slice(&Self::HEADER);
                data.push(0x00); // Length (high byte)
                data.push(0x00); // Length (low byte)
                data.push(Self::HALT_COMMAND); // Command
                data.push(0x00); // Placeholder for CRC (will be computed later)
                data.extend_from_slice(&Self::FOOTER);
                // calculate the length
                let data_len = data.len();
                let length = (data.len() - (&Self::HEADER.len() + &Self::FOOTER.len() + 2)) as u16; // Exclude header, footer and length bytes
                data[2] = (length >> 8) as u8;
                data[3] = (length & 0xff) as u8; //
                // Compute CRC and replace the placeholder
                let crc = Self::compute_crc(&data, data_len);
                data[data_len - 3] = crc; // Replace the placeholder with the computed CRC
            }
            SWDCommand::Resume => {
                // Frame Format: ff f9 00 02 reset crc f5 e7
                data.extend_from_slice(&Self::HEADER);
                data.push(0x00); // Length (high byte)
                data.push(0x00); // Length (low byte)
                data.push(Self::RESUME_COMMAND); // Command
                data.push(0x00); // Placeholder for CRC (will be computed later)
                data.extend_from_slice(&Self::FOOTER);
                // calculate the length
                let data_len = data.len();
                let length = (data.len() - (&Self::HEADER.len() + &Self::FOOTER.len() + 2)) as u16; // Exclude header, footer and length bytes
                data[2] = (length >> 8) as u8;
                data[3] = (length & 0xff) as u8; //
                // Compute CRC and replace the placeholder
                let crc = Self::compute_crc(&data, data_len);
                data[data_len - 3] = crc; // Replace the placeholder with the computed CRC
            }
            SWDCommand::ReadBytes { start_address, length } => {
                // Frame Format: ff f9 len0 len1 cmd addr0 addr1 addr2 addr3 crc f5 e7
                if *length > (Self::MAX_DATA_LENGTH as u32) {
                    panic!("Length exceeds maximum allowed value");
                }
                data.extend_from_slice(&Self::HEADER);
                data.push(0x00); // Length (high byte)
                data.push(0x00); // Length (low byte)
                data.push(Self::READ_BYTES_COMMAND); // Command
                data.push((start_address >> 24) as u8); // Start address (high byte)
                data.push((start_address >> 16) as u8); // Start address (mid byte)
                data.push((start_address >> 8) as u8); // Start address (low byte)
                data.push((start_address & 0xff) as u8); // Start address (low byte)
                data.push((length >> 8) as u8); // Length (high byte)
                data.push((length & 0xff) as u8); // Length (low byte)
                data.push(0x00); // Placeholder for CRC (will be computed later)
                data.extend_from_slice(&Self::FOOTER);
                // calculate the length
                let data_len = data.len();
                let length = (data.len() - (&Self::HEADER.len() + &Self::FOOTER.len() + 2)) as u16; // Exclude header, footer and length bytes
                data[2] = (length >> 8) as u8;
                data[3] = (length & 0xff) as u8; //
                // Compute CRC and replace the placeholder
                let crc = Self::compute_crc(&data, data_len);
                data[data_len - 3] = crc; // Replace the placeholder with the computed CRC
            }
            SWDCommand::ReadWord { start_address } => {
                // Frame Format: ff f9 len0 len1 cmd addr0 addr1 addr2 addr3 crc f5 e7
                data.extend_from_slice(&Self::HEADER);
                data.push(0x00); // Length (high byte)
                data.push(0x00); // Length (low byte)
                data.push(Self::READ_WORD); // Command
                data.push((start_address >> 24) as u8); // Start address (high byte)
                data.push((start_address >> 16) as u8); // Start address (mid byte)
                data.push((start_address >> 8) as u8); // Start address (low byte)
                data.push((start_address & 0xff) as u8); // Start address (low byte)
                data.push(0x00); // Placeholder for CRC (will be computed later)
                data.extend_from_slice(&Self::FOOTER);
                // calculate the length
                let data_len = data.len();
                let length = (data.len() - (&Self::HEADER.len() + &Self::FOOTER.len() + 2)) as u16; // Exclude header, footer and length bytes
                data[2] = (length >> 8) as u8;
                data[3] = (length & 0xff) as u8; //
                // Compute CRC and replace the placeholder
                let crc = Self::compute_crc(&data, data_len);
                data[data_len - 3] = crc; // Replace the placeholder with the computed CRC
            }
            SWDCommand::ReadWords { start_address, length } => {
                if *length * (Self::WORD_SIZE as u32) * u8::BITS > (Self::MAX_DATA_LENGTH as u32) {
                    panic!("Length exceeds maximum allowed value");
                }
                // Frame Format: ff f9 len0 len1 cmd addr0 addr1 addr2 addr3 crc f5 e7
                data.extend_from_slice(&Self::HEADER);
                data.push(0x00); // Length (high byte)
                data.push(0x00); // Length (low byte)
                data.push(Self::READ_WORDS_COMMAND); // Command
                data.push((start_address >> 24) as u8); // Start address (high byte)
                data.push((start_address >> 16) as u8); // Start address (mid byte)
                data.push((start_address >> 8) as u8); // Start address (low byte)
                data.push((start_address & 0xff) as u8); // Start address (low byte)
                data.push((length >> 8) as u8); // Length (high byte)
                data.push((length & 0xff) as u8); // Length (low byte)
                data.push(0x00); // Placeholder for CRC (will be computed later)
                data.extend_from_slice(&Self::FOOTER);
                // calculate the length
                let data_len = data.len();
                let length = (data.len() - (&Self::HEADER.len() + &Self::FOOTER.len() + 2)) as u16; // Exclude header, footer and length bytes
                data[2] = (length >> 8) as u8;
                data[3] = (length & 0xff) as u8; //
                // Compute CRC and replace the placeholder
                let crc = Self::compute_crc(&data, data_len);
                data[data_len - 3] = crc; // Replace the placeholder with the computed CRC
            }
            SWDCommand::Write { write_address: start_address, write_data } => {
                // check data is is not larger than MAX_DATA_LENGTH
                if write_data.len() > Self::MAX_DATA_LENGTH {
                    panic!("Data length exceeds maximum allowed value");
                }
                data.extend_from_slice(&Self::HEADER);
                data.push(0x00); // Length (high byte)
                data.push(0x00); // Length (low byte)
                data.push(Self::WRITE_COMMAND); // Command
                data.push((start_address >> 24) as u8); // Start address (high byte)
                data.push((start_address >> 16) as u8); // Start address (mid byte)
                data.push((start_address >> 8) as u8); // Start address (low byte)
                data.push((start_address & 0xff) as u8); // Start address (low byte)
                // Add the data bytes
                for byte in write_data {
                    data.push(*byte);
                }
                data.push(0x00); // Placeholder for CRC (will be computed later)
                data.extend_from_slice(&Self::FOOTER);
                // calculate the length
                let data_len = data.len();
                let length = (data.len() - (&Self::HEADER.len() + &Self::FOOTER.len() + 2)) as u16; // Exclude header, footer and length bytes
                data[2] = (length >> 8) as u8;
                data[3] = (length & 0xff) as u8;
                // Compute CRC and replace the placeholder
                let crc = Self::compute_crc(&data, data_len);
                data[data_len - 3] = crc;
            }
        }
        info!("Generated SWD frame: {:02x?}", data);
        data
    }
    pub fn read_frame(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        let crc = Self::compute_crc(data, data.len());
        info!("Computed CRC: {:#02x}", crc);
        if crc != data[data.len() - 3] {
            // todo!();
            // return Err(
            //     format!("CRC mismatch: expected {:#02x}, got {:#02x}", crc, data[data.len() - 3])
            // );
        }
        match &self.command {
            SWDCommand::Halt => {
                match data[Self::ACK_OFFSET] {
                    Self::HALT_ACK => Ok(vec![Self::HALT_ACK]),
                    Self::HALT_ERROR => Err("Halt command failed".to_string()),
                    _ => Err("Unknown response to Halt command".to_string()),
                }
            }
            SWDCommand::Resume => {
                match data[Self::ACK_OFFSET] {
                    Self::RESUME_ACK => Ok(vec![Self::RESUME_ACK]),
                    Self::RESUME_ERROR => Err("Halt command failed".to_string()),
                    _ => Err("Unknown response to Halt command".to_string()),
                }
            }
            SWDCommand::ReadBytes { start_address: _, length: data_length } => {
                match data[Self::ACK_OFFSET] {
                    Self::READ_ACK => {
                        let result =
                            data[
                                Self::ACK_OFFSET + 1..Self::ACK_OFFSET + (*data_length as usize) + 1 // Extract the data bytes
                            ].to_vec();
                        Ok(result)
                    }
                    Self::READ_ERROR => Err("Read command failed".to_string()),
                    _ => Err("Unknown response to Read command".to_string()),
                }
            }
            SWDCommand::ReadWord { start_address: _ } => {
                match data[Self::ACK_OFFSET] {
                    Self::READ_ACK => {
                        let result = data[Self::ACK_OFFSET + 1..Self::ACK_OFFSET + 5].to_vec(); // Extract the 4 bytes of the word
                        Ok(result)
                    }
                    Self::READ_ERROR => Err("Read command failed".to_string()),
                    _ => Err("Unknown response to Read command".to_string()),
                }
            }
            SWDCommand::ReadWords { start_address: _, length: data_length } => {
                match data[Self::ACK_OFFSET] {
                    Self::READ_ACK => {
                        let result =
                            data[
                                Self::ACK_OFFSET + 1..Self::ACK_OFFSET + (*data_length as usize) + 1 // Extract the data bytes
                            ].to_vec();
                        Ok(result)
                    }
                    Self::READ_ERROR => Err("Read command failed".to_string()),
                    _ => Err("Unknown response to Read command".to_string()),
                }
            }
            SWDCommand::Write { write_address: _, write_data: _ } => {
                match data[Self::ACK_OFFSET] {
                    Self::WRITE_ACK => { Ok(vec![Self::WRITE_ACK]) }
                    Self::WRITE_ERROR => Err("Write command failed".to_string()),
                    _ => Err("Unknown response to Write command".to_string()),
                }
            }
        }
    }
}
