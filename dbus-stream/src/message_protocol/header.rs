use std::collections::HashSet;

use signature::SingleCompleteTypeSignature;
use types::{BasicType, DBusSignature};
use types::ContainerType;
use types::DBusArray;
use types::DBusByte;
use types::DBusStruct;
use types::DBusVariant;
use types::Type;

use self::header_field::HeaderField;
use super::MessageType;
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
    pub fn marshall(&self) -> crate::Result<Vec<u8>> {
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

        // Header fields.
        let header_fields = {
            // Header fields is a marshalled Array of Struct(Byte, Variant).

            let header_field_array_items: Vec<Type> = self
                .header_fields
                .iter()
                .map(|header_field| {
                    Type::from(DBusStruct {
                        fields: vec![
                            Type::from(DBusByte {
                                u8: header_field.decimal_code(),
                            }),
                            Type::from(DBusVariant {
                                variant: Box::new(match header_field.clone() {
                                    HeaderField::Path(inner) => Type::from(inner),
                                    HeaderField::Interface(inner) => Type::from(inner),
                                    HeaderField::Member(inner) => Type::from(inner),
                                    HeaderField::ErrorName(inner) => Type::from(inner),
                                    HeaderField::ReplySerial(inner) => Type::from(inner),
                                    HeaderField::Destination(inner) => Type::from(inner),
                                    HeaderField::Sender(inner) => Type::from(inner),
                                    HeaderField::Signature(inner) => Type::from(inner),
                                    HeaderField::UnixFds(inner) => Type::from(inner),
                                }),
                            }),
                        ],
                    })
                })
                .collect();

            let header_fields_array = DBusArray {
                item_type: SingleCompleteTypeSignature::Struct {
                    fields: vec![
                        SingleCompleteTypeSignature::Byte,
                        SingleCompleteTypeSignature::Variant,
                    ],
                },
                items: header_field_array_items,
            };

            header_fields_array.marshall_be()?
        };
        v.extend(header_fields);

        // Header must be 8-aligned with null bytes
        while v.len() % 8 != 0 {
            v.push(0);
        }

        Ok(v)
    }
}
