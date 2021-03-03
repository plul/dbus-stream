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

impl Header {
    /// Marshall according to the endianness specified in the header
    pub fn marshall(&self) -> Vec<u8> {
        let mut v: Vec<u8> = Vec::new();

        // 1st byte: Endianness
        v.push(self.endianness.ascii_code());

        // 2nd byte: Message Type
        v.push(self.message_type.decimal_value());

        // 3rd byte: Bitwise OR flags
        let mut flags = 0;
        for flag in &self.flags {
            flags |= flag.hex_value();
        }
        v.push(flags);

        // 4th byte: Major protocol version
        v.push(crate::MAJOR_PROTOCOL_VERSION);

        // 5th to 8th byte: Length in bytes of message body
        assert_eq!(self.endianness, Endianness::BigEndian);
        v.extend_from_slice(&self.length_in_bytes_of_message_body.to_be_bytes());

        // 9th to 12th byte: Serial
        assert_eq!(self.endianness, Endianness::BigEndian);
        v.extend_from_slice(&self.serial.to_be_bytes());

        // Header fields:
        // A header field is a marshalled Array of Struct(Byte, Variant).
        for header_field in &self.header_fields {
            v.extend(header_field.marshall());
        }

        // Header must be 8-aligned with null bytes
        while v.len() % 8 > 0 {
            v.push(0);
        }

        v
    }
}
