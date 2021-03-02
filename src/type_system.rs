use crate::type_system::types::*;

pub mod marshall;
pub mod unmarshall;
pub mod marker;
pub mod types;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Endianness {
    BigEndian,
    LittleEndian,
}
