use crate::protocol::software_crc;
pub fn check_crc(frame: &[u8]) -> Result<bool, String> {
    let data_length = ((frame[3] as u16) << 8) | (frame[2] as u16);
    let data = &frame[4..4 + (data_length as usize)];
    let crc = software_crc(data, data_length as usize);

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
