use crate::bytes::BigEndian;
use std::fmt::{Debug, Formatter};
use std::fmt;

#[derive(Eq, Copy, Clone)]
pub struct FourCC(u32);

impl FourCC {
    pub(crate) const fn from(value: u32) -> Self {
        FourCC {
            0: value
        }
    }

    pub(crate) const fn from_bytes(bytes: &[u8;4]) -> FourCC {
        FourCC {
            0: BigEndian::read_u32(bytes, 0)
        }
    }
}

impl Into<[u8;4]> for &FourCC {

    fn into(self) -> [u8; 4] {
        let mut buf = [0u8;4];
        BigEndian::write_u32(self.0, &mut buf, 0);
        buf
    }
}

impl Debug for FourCC {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut buf = [0u8;4];
        BigEndian::write_u32(self.0, &mut buf, 0);
        write!(f, "{}", std::str::from_utf8(&buf).unwrap())
    }
}

impl PartialEq for FourCC {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}