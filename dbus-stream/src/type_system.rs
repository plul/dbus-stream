use crate::type_system::types::*;

pub mod marshall;
pub mod signature;
pub mod types;
pub mod unmarshall;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Endianness {
    BigEndian,
    LittleEndian,
}

impl Endianness {
    pub fn ascii_code(&self) -> u8 {
        match self {
            Self::BigEndian => b'B',
            Self::LittleEndian => b'l',
        }
    }
}
