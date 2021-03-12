use std::collections::HashSet;

use nom::bytes::complete::tag;
use nom::bytes::complete::take;
use nom::combinator::all_consuming;
use nom::combinator::map_opt;
use nom::combinator::map_res;
use nom::number::complete::be_f64;
use nom::number::complete::be_i16;
use nom::number::complete::be_i32;
use nom::number::complete::be_i64;
use nom::number::complete::be_u16;
use nom::number::complete::be_u32;
use nom::number::complete::be_u64;
use nom::number::complete::be_u8;
use nom::number::complete::le_u32;
use nom::Finish;
use nom::IResult;

use super::signature::HEADER_FIELD_SIGNATURE;
use super::signature::SingleCompleteTypeSignature;
use super::types::*;
use super::Endianness;
use crate::message_protocol::header::HeaderFlag;
use crate::message_protocol::MessageType;

/// Unmarshall a DBus message (consisting of header and body),
pub fn unmarshall_message(message: &[u8]) -> crate::Result<()> {
    let (i, value) = all_consuming(unmarshall_message_parse)(message)
        .finish()
        .map_err(|err| crate::Error::ParseError)?;

    Ok(value)
}

fn unmarshall_message_parse(i: &[u8]) -> IResult<&[u8], ()> {
    // 1st byte: Endianness
    let (i, endianness) = Endianness::unmarshall(i)?;

    let parse_u32 = match endianness {
        Endianness::LittleEndian => le_u32,
        Endianness::BigEndian => be_u32,
    };

    // 2nd byte: Message type
    let (i, message_type) = MessageType::unmarshall(i)?;

    // 3rd byte: Header flags
    let (i, flag_bitfield) = be_u8(i)?;
    let flags: HashSet<HeaderFlag> = {
        let mut set = HashSet::new();

        if 0x1 & flag_bitfield == 0x1 {
            set.insert(HeaderFlag::NoReplyExpected);
        }

        if 0x2 & flag_bitfield == 0x2 {
            set.insert(HeaderFlag::NoAutoStart);
        }

        if 0x4 & flag_bitfield == 0x4 {
            set.insert(HeaderFlag::AllowInteractiveAuthorization);
        }

        set
    };

    // 4th byte: Major protocol version
    let (i, major_protocol_version) = tag(&[crate::MAJOR_PROTOCOL_VERSION])(i)?;

    // 5th-8th byte: Length in bytes of the message body.
    let (i, length_in_bytes_of_message_body) = parse_u32(i)?;

    // 9th-12th byte: Serial linking message and response.
    let (i, serial) = parse_u32(i)?;

    // Unmarshall header fields
    let (i, header_field_array) = match endianness {
        Endianness::BigEndian => DBusArray::unmarshall_be(i, &HEADER_FIELD_SIGNATURE)?,
        Endianness::LittleEndian => todo!(),
    };

    todo!("Separate the header fields and package them in a nicer way?");

    todo!("Unmarshall body");

    todo!("Define return type");
}

impl DBusByte {
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, value) = be_u8(i)?;

        // Create a Self type, (the unmarshalled type).
        let unmarshalled: Self = Self { u8: value };

        Ok((i, unmarshalled))
    }
}

impl DBusBoolean {
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        // The boolean is contained in a u32, but only 0 or 1 are valid values.
        let (i, bool): (&[u8], bool) = map_opt(be_u32, |value| match value {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        })(i)?;

        let unmarshalled: Self = Self { bool };

        Ok((i, unmarshalled))
    }
}

impl DBusInt16 {
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, value) = be_i16(i)?;

        // Create a Self type, (the unmarshalled type).
        let unmarshalled: Self = Self { i16: value };

        Ok((i, unmarshalled))
    }
}

impl DBusUint16 {
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, value) = be_u16(i)?;

        // Create a Self type, (the unmarshalled type).
        let unmarshalled: Self = Self { u16: value };

        Ok((i, unmarshalled))
    }
}

impl DBusInt32 {
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, value) = be_i32(i)?;

        // Create a Self type, (the unmarshalled type).
        let unmarshalled: Self = Self { i32: value };

        Ok((i, unmarshalled))
    }
}

impl DBusUint32 {
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        // Use nom to parse a big-endian u32 from the input.
        let (i, value) = be_u32(i)?;

        // Create a Self type, (the unmarshalled type).
        let unmarshalled: Self = Self { u32: value };

        Ok((i, unmarshalled))
    }
}

impl DBusInt64 {
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, value) = be_i64(i)?;

        // Create a Self type, (the unmarshalled type).
        let unmarshalled: Self = Self { i64: value };

        Ok((i, unmarshalled))
    }
}

impl DBusUint64 {
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, value) = be_u64(i)?;

        // Create a Self type, (the unmarshalled type).
        let unmarshalled: Self = Self { u64: value };

        Ok((i, unmarshalled))
    }
}

impl DBusDouble {
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, value) = be_f64(i)?;

        // Create a Self type, (the unmarshalled type).
        let unmarshalled: Self = Self { f64: value };

        Ok((i, unmarshalled))
    }
}

impl DBusString {
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        // The first 4 bytes encode the string's length in bytes, excluding its terminating null.
        let (i, length): (&[u8], u32) = be_u32(i)?;

        // Now we know the length in bytes of the string to follow.
        // DBus strings are UTF-8 encoded, so we need to decode the bytes we take as UTF-8
        // in order to get a Rust string. Since it might not be valid UTF-8, converting the
        // bytes to a str slice returns a Result with an error type from the std lib.
        // Therefore, we map that result to nom's IResult with nom's map_res function.
        let first = take(length);
        let second = std::str::from_utf8;
        let (i, str_slice): (&[u8], &str) = map_res(first, second)(i)?;

        // The string must then be followed by a null byte:
        let (i, _) = tag(&[0])(i)?;

        // Convert the str_slice to an owned String, and crate a DBusString (Self) from it:
        let unmarshalled: Self = Self {
            string: String::from(str_slice),
        };

        Ok((i, unmarshalled))
    }
}

impl DBusObjectPath {
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        todo!("Dani");
    }
}

impl DBusSignature {
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        todo!("Dani");
    }
}

impl DBusUnixFileDescriptor {
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        todo!("Dani. But this one is low priority, could just be left as todo");
    }
}

impl DBusArray {
    fn unmarshall_be<'a>(i: &'a [u8], item_type: &SingleCompleteTypeSignature) -> IResult<&'a [u8], Self> {
        todo!("Dani? These container types may be a little harder");
    }
}

impl DBusStruct {
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        todo!("Dani? These container types may be a little harder. Consider if it needs to know the type of the fields up front?");
    }
}

impl DBusVariant {
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        todo!("Dani? These container types may be a little harder");
    }
}

impl DBusMap {
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        todo!("Dani? These container types may be a little harder. Consider if it needs to know the type of key and value up front?");
    }
}

impl Endianness {
    fn unmarshall(i: &[u8]) -> IResult<&[u8], Self> {
        map_opt(be_u8, |value| match value {
            b'B' => Some(Endianness::BigEndian),
            b'l' => Some(Endianness::LittleEndian),
            _ => None,
        })(i)
    }
}

impl MessageType {
    fn unmarshall(i: &[u8]) -> IResult<&[u8], Self> {
        map_opt(be_u8, |value| match value {
            1 => Some(MessageType::MethodCall),
            2 => Some(MessageType::MethodReturn),
            3 => Some(MessageType::Error),
            4 => Some(MessageType::Signal),
            _ => None,
        })(i)
    }
}
