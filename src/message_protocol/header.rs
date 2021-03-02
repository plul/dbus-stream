use std::collections::HashSet;

use crate::type_system::*;
use crate::type_system::types::*;
use super::MessageType;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum HeaderFlag {
    NoReplyExpected,
    NoAutoStart,
    AllowInteractiveAuthorization,
}

pub struct Header {
    endianness: Endianness,
    message_type: MessageType,
    flags: HashSet<HeaderFlag>,
    major_protocol_version: u8,
    length_in_bytes_of_message_body: u32,
    serial: u32,
    header_fields: Vec<HeaderField>,
}

pub enum HeaderField {
    Path(DBusObjectPath),
    Interface(DBusString),
    Member(DBusString),
    ErrorName(DBusString),
    ReplySerial(DBusUint32),
    Destination(DBusString),
    Sender(DBusString),
    Signature(DBusSignature),
    UnixFds(DBusUint32),
}

impl HeaderField {
    pub fn decimal_code(&self) -> u8 {
        match self {
            Self::Path(_) => 1,
            Self::Interface(_) => 2,
            Self::Member(_) => 3,
            Self::ErrorName(_) => 4,
            Self::ReplySerial(_) => 5,
            Self::Destination(_) => 6,
            Self::Sender(_) => 7,
            Self::Signature(_) => 8,
            Self::UnixFds(_) => 9,
        }
    }
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