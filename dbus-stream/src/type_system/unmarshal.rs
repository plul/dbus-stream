use std::iter::Iterator;

use nom::{branch::alt, multi::many0, sequence::{pair, preceded}};
use nom::bytes::complete::tag;
use nom::bytes::complete::take;
use nom::combinator::all_consuming;
use nom::combinator::map_opt;
use nom::combinator::map_res;
use nom::combinator::value;
use nom::number::complete::be_f64;
use nom::number::complete::be_i16;
use nom::number::complete::be_i32;
use nom::number::complete::be_i64;
use nom::number::complete::be_u16;
use nom::number::complete::be_u32;
use nom::number::complete::be_u64;
use nom::number::complete::be_u8;
use nom::number::complete::le_u32;
use nom::sequence::delimited;
use nom::Finish;
use nom::IResult;

use super::signature::SingleCompleteTypeSignature;
use super::signature::HEADER_FIELD_SIGNATURE;
use super::types::*;
use super::Endianness;
use crate::message_protocol::Message;
use crate::message_protocol::MessageType;

pub mod decoder;

use self::decoder::Decoder;

trait Unmarshal: Sized {
    fn unmarshal_be<'a>(i: Decoder<'a>) -> IResult<Decoder<'a>, Self>;
}

// // The const generics version:
// impl<'a, 'b, const N: usize> nom::Compare<&'b [u8; N]> for Decoder<'a> {
//     fn compare(&self, t: &'b [u8; N]) -> nom::CompareResult {
//         self.data.compare(t)
//     }

//     fn compare_no_case(&self, t: &'b [u8; N]) -> nom::CompareResult {
//         self.data.compare_no_case(t)
//     }
// }

impl Unmarshal for DBusByte {
    fn unmarshal_be<'a>(i: Decoder<'a>) -> IResult<Decoder<'a>, Self> {
        let (i, value) = be_u8(i)?;

        let unmarshalled: Self = Self { u8: value };

        Ok((i, unmarshalled))
    }
}

impl Unmarshal for DBusBoolean {
    fn unmarshal_be<'a>(i: Decoder<'a>) -> IResult<Decoder<'a>, Self> {
        let i = i.advance_to_boundary(4)?;

        // The boolean is contained in a u32, but only 0 or 1 are valid values.
        let (i, bool): (Decoder, bool) = map_opt(be_u32, |value| match value {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        })(i)?;

        let unmarshalled: Self = Self { bool };

        Ok((i, unmarshalled))
    }
}

impl Unmarshal for DBusSignature {
    fn unmarshal_be<'a>(i: Decoder<'a>) -> IResult<Decoder<'a>, Self> {
        let parse_byte = value(SingleCompleteTypeSignature::DBusByte, tag(b"y"));
        let parse_boolean = value(SingleCompleteTypeSignature::DBusBoolean, tag(b"b"));
        let parse_int16 = value(SingleCompleteTypeSignature::DBusInt16, tag(b"n"));
        let parse_uint16 = value(SingleCompleteTypeSignature::DBusUint16, tag(b"q"));
        let parse_int32 = value(SingleCompleteTypeSignature::DBusInt32, tag(b"i"));
        let parse_uint32 = value(SingleCompleteTypeSignature::DBusUint32, tag(b"u"));
        let parse_int64 = value(SingleCompleteTypeSignature::DBusInt64, tag(b"x"));
        let parse_uint64 = value(SingleCompleteTypeSignature::DBusUint64, tag(b"t"));
        let parse_double = value(SingleCompleteTypeSignature::DBusDouble, tag(b"d"));
        let parse_string = value(SingleCompleteTypeSignature::DBusString, tag(b"s"));
        let parse_objectpath = value(SingleCompleteTypeSignature::DBusObjectPath, tag(b"o"));
        let parse_unixfiledescriptor = value(
            SingleCompleteTypeSignature::DBusUnixFileDescriptor,
            tag(b"h"),
        );
        let parse_struct = delimited(tag(b"("), DBusSignature::unmarshal_be, tag(b")"));
        let parse_variant = value(SingleCompleteTypeSignature::DBusVariant, tag(b"v"));
        let parse_array = preceded(tag(b"a"), alt((parse_single_complete_type_except_dictentry, parse_dict_entry)));
        let parse_dict_entry = delimited(tag(b"{"), pair(parse_basic_type, parse_single_complete_type_except_dictentry), tag(b"}"));

        let parse_basic_type = alt((
            parse_byte,
            parse_boolean,
            parse_int16,
            parse_uint16,
            parse_int32,
            parse_uint32,
            parse_int64,
            parse_uint64,
            parse_double,
            parse_string,
            parse_objectpath,
            parse_unixfiledescriptor,
        ));

        let parse_single_complete_type_except_dictentry = alt((parse_basic_type, parse_struct, parse_variant, parse_array));

        let (i, parsed) = many0(parse_single_complete_type_except_dictentry)(i)?;
    }
}

/// Unmarshal a DBus message (consisting of header and body),
pub fn unmarshal_message(message: &[u8]) -> crate::Result<Message> {
    let (_i, message) = all_consuming(unmarshal_message_parse)(message)
        .finish()
        .map_err(|_err| crate::Error::ParseError)?;

    Ok(message)
}

fn unmarshal_message_parse(i: &[u8]) -> IResult<&[u8], Message> {
    // 1st byte: Endianness
    let (i, endianness) = Endianness::unmarshal(i)?;

    let parse_u32 = match endianness {
        Endianness::LittleEndian => le_u32,
        Endianness::BigEndian => be_u32,
    };

    // 2nd byte: Message type
    let (i, message_type) = MessageType::unmarshal(i)?;

    // 3rd byte: Header flags
    let (i, flag_bitfield) = be_u8(i)?;
    let flag_no_reply_expected: bool = 0x1 & flag_bitfield == 0x1;
    let flag_no_auto_start: bool = 0x2 & flag_bitfield == 0x2;
    let flag_allow_interactive_authorization: bool = 0x4 & flag_bitfield == 0x4;

    // 4th byte: Major protocol version
    let (i, major_protocol_version) = tag(&[crate::MAJOR_PROTOCOL_VERSION])(i)?;

    // 5th-8th byte: Length in bytes of the message body.
    let (i, length_in_bytes_of_message_body) = parse_u32(i)?;

    // 9th-12th byte: Serial linking message and response.
    let (i, serial) = parse_u32(i)?;

    // Unmarshal header fields
    let (i, header_field_array) = match endianness {
        Endianness::BigEndian => DBusArray::unmarshal_be(i, &HEADER_FIELD_SIGNATURE)?,
        Endianness::LittleEndian => todo!(),
    };

    todo!("Separate the header fields and package them in a nicer way?");

    todo!("Unmarshal body");

    todo!("Define return type");
}

impl DBusByte {
    fn unmarshal_be(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, value) = be_u8(i)?;

        // Create a Self type, (the unmarshalled type).
        let unmarshalled: Self = Self { u8: value };

        Ok((i, unmarshalled))
    }
}

impl DBusBoolean {
    fn unmarshal_be(i: &[u8]) -> IResult<&[u8], Self> {
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
    fn unmarshal_be(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, value) = be_i16(i)?;

        // Create a Self type, (the unmarshalled type).
        let unmarshalled: Self = Self { i16: value };

        Ok((i, unmarshalled))
    }
}

impl DBusUint16 {
    fn unmarshal_be(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, value) = be_u16(i)?;

        // Create a Self type, (the unmarshalled type).
        let unmarshalled: Self = Self { u16: value };

        Ok((i, unmarshalled))
    }
}

impl DBusInt32 {
    fn unmarshal_be(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, value) = be_i32(i)?;

        // Create a Self type, (the unmarshalled type).
        let unmarshalled: Self = Self { i32: value };

        Ok((i, unmarshalled))
    }
}

impl DBusUint32 {
    fn unmarshal_be(i: &[u8]) -> IResult<&[u8], Self> {
        // Use nom to parse a big-endian u32 from the input.
        let (i, value) = be_u32(i)?;

        // Create a Self type, (the unmarshalled type).
        let unmarshalled: Self = Self { u32: value };

        Ok((i, unmarshalled))
    }
}

impl DBusInt64 {
    fn unmarshal_be(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, value) = be_i64(i)?;

        // Create a Self type, (the unmarshalled type).
        let unmarshalled: Self = Self { i64: value };

        Ok((i, unmarshalled))
    }
}

impl DBusUint64 {
    fn unmarshal_be(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, value) = be_u64(i)?;

        // Create a Self type, (the unmarshalled type).
        let unmarshalled: Self = Self { u64: value };

        Ok((i, unmarshalled))
    }
}

impl DBusDouble {
    fn unmarshal_be(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, value) = be_f64(i)?;

        // Create a Self type, (the unmarshalled type).
        let unmarshalled: Self = Self { f64: value };

        Ok((i, unmarshalled))
    }
}

impl DBusString {
    fn unmarshal_be(i: &[u8]) -> IResult<&[u8], Self> {
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
    fn unmarshal_be(i: &[u8]) -> IResult<&[u8], Self> {
        todo!("Dani");
    }
}

impl DBusSignature {
    fn unmarshal_be(i: &[u8]) -> IResult<&[u8], Self> {
        fn unmarshal_basic_type(b: u8) -> SingleCompleteTypeSignature {
            match b {
                b'y' => SingleCompleteTypeSignature::DBusByte,
                b'b' => SingleCompleteTypeSignature::DBusBoolean,
                b'n' => SingleCompleteTypeSignature::DBusInt16,
                b'q' => SingleCompleteTypeSignature::DBusUint16,
                b'i' => SingleCompleteTypeSignature::DBusInt32,
                b'u' => SingleCompleteTypeSignature::DBusUint32,
                b'x' => SingleCompleteTypeSignature::DBusInt64,
                b't' => SingleCompleteTypeSignature::DBusUint64,
                b'd' => SingleCompleteTypeSignature::DBusDouble,
                b's' => SingleCompleteTypeSignature::DBusString,
                b'o' => SingleCompleteTypeSignature::DBusObjectPath,
                b'h' => SingleCompleteTypeSignature::DBusUnixFileDescriptor,
                _ => todo!("Dani"),
            }
        }

        let (i, length): (&[u8], u8) = be_u8(i)?;

        let mut v: Vec<SingleCompleteTypeSignature> = Vec::new();
        let mut pos: u8 = 0;
        while pos < length {
            let b: u8 = i[pos as usize];
            if b == b'a' {
                // Array
                todo!("Dani");
            } else if b == b'(' {
                // Struct
                todo!("Dani");
            } else if b == b'{' {
                // Dict
                todo!("Dani");
            } else {
                v.push(unmarshal_basic_type(b));

                pos += 1;
            }
        }

        Ok((i, Self { vec: v }))
    }
}

impl DBusUnixFileDescriptor {
    fn unmarshal_be(i: &[u8]) -> IResult<&[u8], Self> {
        todo!("Dani. But this one is low priority, could just be left as todo");
    }
}

impl DBusArray {
    fn unmarshal_be<'a>(
        i: &'a [u8],
        item_type: &SingleCompleteTypeSignature,
    ) -> IResult<&'a [u8], Self> {
        let (mut i, length): (&[u8], u32) = be_u32(i)?;

        // If the element's size is greater than the size of the length
        // field, some padding is present.
        let size: usize = item_type.marshalling_boundary();
        if size > 4 {
            i = tag(vec![0; 4 % size].as_slice())(i)?.0;
        }

        let mut dba: Self = Self::new(item_type.clone());
        match item_type {
            SingleCompleteTypeSignature::DBusByte => {
                for pos in 0..length as usize {
                    let (_, b) = DBusByte::unmarshal_be(&i[pos..])?;
                    dba.push(BasicType::from(b));
                }
            }
            SingleCompleteTypeSignature::DBusBoolean => {
                for pos in (0..length as usize).step_by(size) {
                    let (_, b) = DBusBoolean::unmarshal_be(&i[pos..])?;
                    dba.push(BasicType::from(b));
                }
            }
            SingleCompleteTypeSignature::DBusInt16 => {
                for pos in (0..length as usize).step_by(size) {
                    let (_, b) = DBusInt16::unmarshal_be(&i[pos..])?;
                    dba.push(BasicType::from(b));
                }
            }
            SingleCompleteTypeSignature::DBusUint16 => {
                for pos in (0..length as usize).step_by(size) {
                    let (_, b) = DBusUint16::unmarshal_be(&i[pos..])?;
                    dba.push(BasicType::from(b));
                }
            }
            SingleCompleteTypeSignature::DBusInt32 => {
                for pos in (0..length as usize).step_by(size) {
                    let (_, b) = DBusInt32::unmarshal_be(&i[pos..])?;
                    dba.push(BasicType::from(b));
                }
            }
            SingleCompleteTypeSignature::DBusUint32 => {
                for pos in (0..length as usize).step_by(size) {
                    let (_, b) = DBusUint32::unmarshal_be(&i[pos..])?;
                    dba.push(BasicType::from(b));
                }
            }
            SingleCompleteTypeSignature::DBusInt64 => {
                for pos in (0..length as usize).step_by(size) {
                    let (_, b) = DBusInt64::unmarshal_be(&i[pos..])?;
                    dba.push(BasicType::from(b));
                }
            }
            SingleCompleteTypeSignature::DBusUint64 => {
                for pos in (0..length as usize).step_by(size) {
                    let (_, b) = DBusUint64::unmarshal_be(&i[pos..])?;
                    dba.push(BasicType::from(b));
                }
            }
            SingleCompleteTypeSignature::DBusDouble => {
                for pos in (0..length as usize).step_by(size) {
                    let (_, b) = DBusDouble::unmarshal_be(&i[pos..])?;
                    dba.push(BasicType::from(b));
                }
            }
            SingleCompleteTypeSignature::DBusString => {
                for pos in (0..length as usize).step_by(size) {
                    let (_, b) = DBusString::unmarshal_be(&i[pos..])?;
                    dba.push(BasicType::from(b));
                }
            }
            SingleCompleteTypeSignature::DBusObjectPath => todo!("Dani"),
            SingleCompleteTypeSignature::DBusSignature => todo!("Dani"),
            SingleCompleteTypeSignature::DBusUnixFileDescriptor => todo!("Dani"),
            SingleCompleteTypeSignature::DBusArray(_) => todo!("Dani"),
            SingleCompleteTypeSignature::DBusStruct { fields: _ } => todo!("Dani"),
            SingleCompleteTypeSignature::DBusVariant => todo!("Dani"),
            SingleCompleteTypeSignature::DBusDictEntry { key: _, value: _ } => todo!("Dani"),
        };

        Ok((i, dba))
    }
}

impl DBusStruct {
    fn unmarshal_be(i: &[u8]) -> IResult<&[u8], Self> {
        todo!("Dani? These container types may be a little harder. Consider if it needs to know the type of the fields up front?");
    }
}

impl DBusVariant {
    fn unmarshal_be(i: &[u8]) -> IResult<&[u8], Self> {
        todo!("Dani? These container types may be a little harder");
    }
}

impl DBusDictEntry {
    fn unmarshal_be(i: &[u8]) -> IResult<&[u8], Self> {
        todo!("Dani? These container types may be a little harder. Consider if it needs to know the type of key and value up front?");
    }
}

impl Endianness {
    fn unmarshal(i: &[u8]) -> IResult<&[u8], Self> {
        map_opt(be_u8, |value| match value {
            b'B' => Some(Endianness::BigEndian),
            b'l' => Some(Endianness::LittleEndian),
            _ => None,
        })(i)
    }
}

impl MessageType {
    fn unmarshal(i: &[u8]) -> IResult<&[u8], Self> {
        map_opt(be_u8, |value| match value {
            1 => Some(MessageType::MethodCall),
            2 => Some(MessageType::MethodReturn),
            3 => Some(MessageType::Error),
            4 => Some(MessageType::Signal),
            _ => None,
        })(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unmarshal_array_of_bytes() {
        let a: [u8; 7] = [0, 0, 0, 3, 15, 16, 17];
        let b: [DBusByte; 3] = [
            DBusByte { u8: 15 },
            DBusByte { u8: 16 },
            DBusByte { u8: 17 },
        ];

        let (_, dba) = DBusArray::unmarshal_be(&a, &SingleCompleteTypeSignature::DBusByte).unwrap();

        assert_eq!(b.len(), dba.items.len());
        for (idx, e) in dba.items.iter().enumerate() {
            if let Type::Basic(BasicType::DBusByte(dbb)) = e {
                assert_eq!(dbb.u8, b[idx].u8);
            } else {
                panic!();
            }
        }
    }

    #[test]
    fn unmarshal_basic_signature() {
        let a: [u8; 10] = [9, b'y', b'b', b'n', b'q', b'i', b'u', b'x', b't', b'd'];

        let (_, x) = DBusSignature::unmarshal_be(&a).unwrap();

        assert_eq!(x.vec.len(), 9);
        assert_eq!(x.vec[0], SingleCompleteTypeSignature::DBusByte);
        assert_eq!(x.vec[1], SingleCompleteTypeSignature::DBusBoolean);
        assert_eq!(x.vec[2], SingleCompleteTypeSignature::DBusInt16);
        assert_eq!(x.vec[3], SingleCompleteTypeSignature::DBusUint16);
        assert_eq!(x.vec[4], SingleCompleteTypeSignature::DBusInt32);
        assert_eq!(x.vec[5], SingleCompleteTypeSignature::DBusUint32);
        assert_eq!(x.vec[6], SingleCompleteTypeSignature::DBusInt64);
        assert_eq!(x.vec[7], SingleCompleteTypeSignature::DBusUint64);
        assert_eq!(x.vec[8], SingleCompleteTypeSignature::DBusDouble);
    }
}
