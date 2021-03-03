use std::collections::HashSet;

use self::header_field::HeaderField;
use super::MessageType;
use crate::type_system::types::*;
use crate::type_system::*;

pub mod header_field;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum HeaderFlag {
    NoReplyExpected,
    NoAutoStart,
    AllowInteractiveAuthorization,
}

pub struct Header {
    pub endianness: Endianness,
    pub message_type: MessageType,
    pub flags: HashSet<HeaderFlag>,
    pub major_protocol_version: u8,
    pub length_in_bytes_of_message_body: u32,
    pub serial: u32,
    pub header_fields: Vec<HeaderField>,
}

impl HeaderFlag {
    pub fn hex_value(&self) -> u8 {
        match self {
            Self::NoReplyExpected => 0x1,
            Self::NoAutoStart => 0x2,
            Self::AllowInteractiveAuthorization => 0x4,
        }
    }
}
