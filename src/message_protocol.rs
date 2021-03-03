pub mod body;
pub mod header;

use self::body::Body;
use self::header::header_field;
use self::header::Header;
use crate::type_system::types::*;

pub struct Message {
    pub header: Header,
    pub body: Body,
}

pub enum MessageType {
    MethodCall,
    MethodReturn,
    Error,
    Signal,
}

pub struct MethodCall {
    pub destination: header_field::Destination,
    pub path: header_field::Path,
    pub interface: Option<header_field::Interface>,
    pub member: header_field::Member,
    pub body: Body,
}

impl MessageType {
    pub fn decimal_value(&self) -> u8 {
        match self {
            Self::MethodCall => 1,
            Self::MethodReturn => 2,
            Self::Error => 3,
            Self::Signal => 4,
        }
    }
}

// impl Message {
//     pub fn unmarshall(data: &[u8]) -> crate::Result<Self> {
//         let endianness = match data[0] {
//             b'l' => Endianness::LittleEndian,
//             b'B' => Endianness::BigEndian,
//             _ => todo!(),
//         };

//         let message_type = match data[1] {
//             1 => MessageType::MethodCall,
//             2 => MessageType::MethodReturn,
//             3 => MessageType::Error,
//             4 => MessageType::Signal,
//             _ => todo!(),
//         };

//         let flags: HashSet<HeaderFlag> = {
//             let mut set = HashSet::new();

//             if 0x1 & data[2] == 0x1 {
//                 set.insert(HeaderFlag::NoReplyExpected);
//             }

//             if 0x2 & data[2] == 0x2 {
//                 set.insert(HeaderFlag::NoAutoStart);
//             }

//             if 0x4 & data[2] == 0x4 {
//                 set.insert(HeaderFlag::AllowInteractiveAuthorization);
//             }

//             set
//         };

//         let major_protocol_version = data[3];
//         if major_protocol_version != crate::MAJOR_PROTOCOL_VERSION {
//             todo!();
//         }

//         let length_in_bytes_of_message_body: u32 = match endianness {
//             Endianness::LittleEndian => u32::from_le_bytes(data[4..8].try_into()?),
//             Endianness::BigEndian => u32::from_be_bytes(data[4..8].try_into()?),
//         };

//         let serial: u32 = match endianness {
//             Endianness::LittleEndian => u32::from_le_bytes(data[8..12].try_into()?),
//             Endianness::BigEndian => u32::from_be_bytes(data[8..12].try_into()?),
//         };

//         let header_fields: Vec<HeaderField> = {
//             // The number of bytes of data in the array, after the padding:
//             let array_length: u32 = match endianness {
//                 Endianness::LittleEndian => u32::from_le_bytes(data[12..16].try_into()?),
//                 Endianness::BigEndian => u32::from_be_bytes(data[12..16].try_into()?),
//             };

//             // The array contains structs, and structs are always 8-byte aligned.
//             // So we should expect 4 bytes of padding:
//             if !((data[16] == 0) && (data[17] == 0) && (data[18] == 0) && (data[19] == 0)) {
//                 todo!();
//             }

//             let mut vec = Vec::new();
//             let mut bytes_consumed = 0;
//             let mut offset = 20;
//             while bytes_consumed < array_length {
//                 let field_code = data[offset];
//                 offset += 1;

//                 let variant_signature = todo!();

//                 // Pseudo:
//                 let variant = variant_signature.get_type().unmarshal(&data[offset..]);

//                 offset += todo!();

//                 vec.push(variant);
//             }

//             vec
//         };

//         todo!();
//     }
// }
