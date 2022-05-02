pub fn u24_from_le_bytes(bytes: [u8; 3]) -> u32 {
    ((bytes[2] as u32) << 16) | ((bytes[1] as u32) << 8) | (bytes[0] as u32)
}

pub fn u24_to_le_bytes(v: u32) -> [u8; 3] {
    assert!((v & 0xFF_00_00_00) == 0);

    [
        (v & 0xff) as u8,
        ((v & 0xff_00) >> 8) as u8,
        ((v & 0xff_00_00) >> 16) as u8,
    ]
}
