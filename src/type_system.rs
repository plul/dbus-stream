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
