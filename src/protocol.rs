use crc::{ Crc, * };
pub fn software_crc(data: &[u8], length: usize) -> [u8; 4] {
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
