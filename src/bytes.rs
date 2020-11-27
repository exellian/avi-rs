pub struct BigEndian;
pub struct LittleEndian;

impl LittleEndian {

    pub const fn read_i16(buf: &[u8], offset: usize) -> i16 {
        (buf[offset + 1] as i16) << 8 |
            (buf[offset + 0] as i16) << 0
    }

    pub const fn read_u16(buf: &[u8], offset: usize) -> u16 {
            (buf[offset + 1] as u16) << 8 |
            (buf[offset + 0] as u16) << 0
    }

    pub const fn read_u32(buf: &[u8], offset: usize) -> u32 {
        (buf[offset + 3] as u32) << 24 |
            (buf[offset + 2] as u32) << 16 |
            (buf[offset + 1] as u32) << 8 |
            (buf[offset + 0] as u32) << 0
    }

    pub const fn read_i32(buf: &[u8], offset: usize) -> i32 {
        (buf[offset + 3] as i32) << 24 |
            (buf[offset + 2] as i32) << 16 |
            (buf[offset + 1] as i32) << 8 |
            (buf[offset + 0] as i32) << 0
    }

    pub fn write_u16(n: u32, buf: &mut [u8], offset: usize) {
        buf[offset + 1] = (n >> 8) as u8;
        buf[offset + 0] = (n >> 0) as u8;
    }

    pub fn write_u32(n: u32, buf: &mut [u8], offset: usize) {
        buf[offset + 3] = (n >> 24) as u8;
        buf[offset + 2] = (n >> 16) as u8;
        buf[offset + 1] = (n >> 8) as u8;
        buf[offset + 0] = (n >> 0) as u8;
    }

    pub fn write_i32(n: i32, buf: &mut [u8], offset: usize) {
        buf[offset + 3] = (n >> 24) as u8;
        buf[offset + 2] = (n >> 16) as u8;
        buf[offset + 1] = (n >> 8) as u8;
        buf[offset + 0] = (n >> 0) as u8;
    }
}

impl BigEndian {

    pub const fn read_i16(buf: &[u8], offset: usize) -> i16 {
        (buf[offset + 0] as i16) << 8 |
            (buf[offset + 1] as i16) << 0
    }

    pub const fn read_u16(buf: &[u8], offset: usize) -> u16 {
        (buf[offset + 0] as u16) << 8 |
            (buf[offset + 1] as u16) << 0
    }

    pub const fn read_u32(buf: &[u8], offset: usize) -> u32 {
        (buf[offset + 0] as u32) << 24 |
            (buf[offset + 1] as u32) << 16 |
            (buf[offset + 2] as u32) << 8 |
            (buf[offset + 3] as u32) << 0
    }

    pub const fn read_i32(buf: &[u8], offset: usize) -> i32 {
        (buf[offset + 0] as i32) << 24 |
            (buf[offset + 1] as i32) << 16 |
            (buf[offset + 2] as i32) << 8 |
            (buf[offset + 3] as i32) << 0
    }

    pub fn write_u16(n: u16, buf: &mut [u8], offset: usize) {
        buf[offset + 0] = (n >> 8) as u8;
        buf[offset + 1] = (n >> 0) as u8;
    }

    pub fn write_u32(n: u32, buf: &mut [u8], offset: usize) {
        buf[offset + 0] = (n >> 24) as u8;
        buf[offset + 1] = (n >> 16) as u8;
        buf[offset + 2] = (n >> 8) as u8;
        buf[offset + 3] = (n >> 0) as u8;
    }

    pub fn write_i32(n: i32, buf: &mut [u8], offset: usize) {
        buf[offset + 0] = (n >> 24) as u8;
        buf[offset + 1] = (n >> 16) as u8;
        buf[offset + 2] = (n >> 8) as u8;
        buf[offset + 3] = (n >> 0) as u8;
    }
}