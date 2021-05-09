use std::ops::Deref;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take;
use nom::combinator::all_consuming;
use nom::combinator::map_opt;
use nom::combinator::map_res;
use nom::combinator::value;
use nom::combinator::map_parser;
use nom::multi::many0;
use nom::multi::many1;
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
use nom::sequence::pair;
use nom::sequence::preceded;
use nom::Finish;
use nom::IResult;

use super::signature::SingleCompleteTypeSignature;
use super::signature::HEADER_FIELD_SIGNATURE;
use super::types::*;
use super::Endianness;
use crate::message_protocol::Message;
use crate::message_protocol::MessageType;

pub mod input;
pub mod parsers;

use parsers::complete::skip_null_byte;

use self::input::I;

trait Alignment {
    fn alignment() -> usize;
}

macro_rules! impl_alignment {
    ($name:ident, $alignment:expr) => {
        impl Alignment for $name {
            fn alignment() -> usize {
                $alignment
            }
        }
    };
}

impl_alignment!(DBusByte, 1);
impl_alignment!(DBusBoolean, 4);
impl_alignment!(DBusInt16, 2);
impl_alignment!(DBusInt32, 4);
impl_alignment!(DBusInt64, 8);
impl_alignment!(DBusUint16, 2);
impl_alignment!(DBusUint32, 4);
impl_alignment!(DBusUint64, 8);
impl_alignment!(DBusDouble, 8);
impl_alignment!(DBusString, 4);
impl_alignment!(DBusObjectPath, 4);
impl_alignment!(DBusSignature, 1);
impl_alignment!(DBusUnixFileDescriptor, 4);
impl_alignment!(DBusArray, 4);
impl_alignment!(DBusStruct, 8);
impl_alignment!(DBusVariant, 1);
impl_alignment!(DBusDictEntry, 8);

impl DBusSignature {

    /// TODO - right now it doesn't look at the leading byte indicating the length. Should it?
    /// it also doesn't look for the terminating null byte, which I feel it probably should!
    fn unmarshal_be<'a>(i: I<'a>) -> IResult<I<'a>, Self> {
        fn parse_basic_type<'a>(i: I<'a>) -> IResult<I<'a>, SingleCompleteTypeSignature> {
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

            alt((
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
            ))(i)
        }

        fn parse_struct<'a>(i: I<'a>) -> IResult<I<'a>, SingleCompleteTypeSignature> {
            let tag = tag::<&[u8], I, nom::error::Error<I>>;
            let (i, fields) = delimited(
                tag(b"("),
                many1(parse_single_complete_type_except_dictentry),
                tag(b")"),
            )(i)?;

            Ok((i, SingleCompleteTypeSignature::DBusStruct { fields }))
        }

        fn parse_variant<'a>(i: I<'a>) -> IResult<I<'a>, SingleCompleteTypeSignature> {
            value(SingleCompleteTypeSignature::DBusVariant, tag(b"v"))(i)
        }

        fn parse_single_complete_type_except_dictentry<'a>(
            i: I<'a>,
        ) -> IResult<I<'a>, SingleCompleteTypeSignature> {
            alt((parse_basic_type, parse_struct, parse_variant, parse_array))(i)
        }

        fn parse_array<'a>(i: I<'a>) -> IResult<I<'a>, SingleCompleteTypeSignature> {
            preceded(
                tag(b"a"),
                alt((
                    parse_basic_type,
                    parse_struct,
                    parse_variant,
                    parse_array,
                    parse_dict_entry,
                )),
            )(i)
        }

        fn parse_dict_entry<'a>(i: I<'a>) -> IResult<I<'a>, SingleCompleteTypeSignature> {
            let tag = tag::<&[u8], I, nom::error::Error<I>>;
            let (i, (key, value)) = delimited(
                tag(b"{"),
                pair(
                    parse_basic_type,
                    parse_single_complete_type_except_dictentry,
                ),
                tag(b"}"),
            )(i)?;

            let dict_entry = SingleCompleteTypeSignature::DBusDictEntry {
                key: Box::new(key),
                value: Box::new(value),
            };

            Ok((i, dict_entry))
        }

        let (i, single_complete_type_signatures): (I, Vec<SingleCompleteTypeSignature>) =
            many0(parse_single_complete_type_except_dictentry)(i)?;

        let dbus_signature = DBusSignature {
            vec: single_complete_type_signatures,
        };

        Ok((i, dbus_signature))
    }
}

/// Unmarshal a DBus message (consisting of header and body),
pub fn unmarshal_message(message: &[u8]) -> crate::Result<Message> {
    let (_i, message) = all_consuming(unmarshal_message_inner)(I::new(message))
        .finish()
        .map_err(|_err| crate::Error::ParseError)?;

    Ok(message)
}

fn unmarshal_message_inner<'i>(i: I<'i>) -> IResult<I<'i>, Message> {
    // 1st byte: Endianness
    let (i, endianness) = map_opt(be_u8, |value| match value {
        b'B' => Some(Endianness::BigEndian),
        b'l' => Some(Endianness::LittleEndian),
        _ => None,
    })(i)?;

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
        Endianness::BigEndian => {
            SingleCompleteTypeSignature::DBusArray(Box::new(HEADER_FIELD_SIGNATURE.clone()))
                .unmarshal_inner(i, endianness)?
        }
        Endianness::LittleEndian => todo!(),
    };
    // Unpack unmarshalled type.
    let header_field_array: DBusArray = match header_field_array {
        Type::Container(ContainerType::DBusArray(dbus_array)) => dbus_array,
        _ => unreachable!(),
    };

    todo!("Separate the header fields and package them in a nicer way?");

    todo!("Unmarshal body");

    todo!("Define return type");
}

macro_rules! impl_unmarshal_be {
    ($name:ident, $parser:ident) => {
        impl $name {
            fn unmarshal_be<'a>(i: I<'a>) -> IResult<I<'a>, Self> {
                let i = i.advance_to_boundary(Self::alignment())?;
                let (i, value) = $parser(i)?;
                let unmarshalled: Self = Self::from(value);
                Ok((i, unmarshalled))
            }
        }
    };
}

impl_unmarshal_be!(DBusByte, be_u8);
impl_unmarshal_be!(DBusInt16, be_i16);
impl_unmarshal_be!(DBusInt32, be_i32);
impl_unmarshal_be!(DBusInt64, be_i64);
impl_unmarshal_be!(DBusUint16, be_u16);
impl_unmarshal_be!(DBusUint32, be_u32);
impl_unmarshal_be!(DBusUint64, be_u64);
impl_unmarshal_be!(DBusDouble, be_f64);

impl DBusBoolean {
    fn unmarshal_be<'a>(i: I<'a>) -> IResult<I<'a>, Self> {
        let i = i.advance_to_boundary(Self::alignment())?;

        // The boolean is contained in a u32, but only 0 or 1 are valid values.
        let (i, boolean): (I, bool) = map_opt(be_u32, |value| match value {
            0 => Some(false),
            1 => Some(true),
            _ => None,
        })(i)?;

        let unmarshalled: Self = Self { bool: boolean };
        Ok((i, unmarshalled))
    }
}

impl MessageType {
    fn unmarshal<'a>(i: I<'a>) -> IResult<I<'a>, Self> {
        todo!()
        //         map_opt(be_u8, |value| match value {
        //             1 => Some(MessageType::MethodCall),
        //             2 => Some(MessageType::MethodReturn),
        //             3 => Some(MessageType::Error),
        //             4 => Some(MessageType::Signal),
        //             _ => None,
        //         })(i)
    }
}

impl SingleCompleteTypeSignature {
    pub fn unmarshal<'i>(&self, i: &'i [u8], endianness: Endianness) -> crate::Result<Type> {
        let (_i, type_) = all_consuming(|i| self.unmarshal_inner(i, endianness))(I::new(i))
            .finish()
            .map_err(|_err| crate::Error::ParseError)?;
        Ok(type_)
    }

    fn unmarshal_inner<'i>(&self, i: I<'i>, endianness: Endianness) -> IResult<I<'i>, Type> {
        if endianness == Endianness::LittleEndian {
            todo!();
        }

        let (i, type_): (I<'i>, Type) = match self {
            Self::DBusByte => {
                let (i, inner) = DBusByte::unmarshal_be(i)?;
                (i, Type::from(inner))
            }
            Self::DBusBoolean => {
                let i = i.advance_to_boundary(DBusBoolean::alignment())?;
                // The boolean is contained in a u32, but only 0 or 1 are valid values.
                let (i, boolean): (I, bool) = map_opt(be_u32, |value| match value {
                    0 => Some(false),
                    1 => Some(true),
                    _ => None,
                })(i)?;
                (i, Type::from(DBusBoolean { bool: boolean }))
            }
            Self::DBusInt16 => {
                let (i, inner) = DBusInt16::unmarshal_be(i)?;
                (i, Type::from(inner))
            }
            Self::DBusUint16 => {
                let (i, inner) = DBusUint16::unmarshal_be(i)?;
                (i, Type::from(inner))
            }
            Self::DBusInt32 => {
                let (i, inner) = DBusInt32::unmarshal_be(i)?;
                (i, Type::from(inner))
            }
            Self::DBusUint32 => {
                let (i, inner) = DBusUint32::unmarshal_be(i)?;
                (i, Type::from(inner))
            }
            Self::DBusInt64 => {
                let (i, inner) = DBusInt64::unmarshal_be(i)?;
                (i, Type::from(inner))
            }
            Self::DBusUint64 => {
                let (i, inner) = DBusUint64::unmarshal_be(i)?;
                (i, Type::from(inner))
            }
            Self::DBusDouble => {
                let (i, inner) = DBusDouble::unmarshal_be(i)?;
                (i, Type::from(inner))
            }
            Self::DBusString => {
                let (i, dbus_string) = DBusString::unmarshal(i, endianness)?;
                (i, Type::from(dbus_string))
            }
            Self::DBusObjectPath => {
                let (i, dbus_string) = DBusString::unmarshal(i, endianness)?;
                let dbus_object_path = DBusObjectPath::from(dbus_string);
                (i, Type::from(dbus_object_path))
            }
            Self::DBusSignature => match endianness {
                Endianness::BigEndian => {
                    let (i, sig) = DBusSignature::unmarshal_be(i)?;
                    (i, Type::from(sig))
                }
                Endianness::LittleEndian => {
                    todo!()
                }
            },
            Self::DBusArray(item_type) => {
                let i = i.advance_to_boundary(DBusArray::alignment())?;
                let (i, length_of_array_data_in_bytes): (I, u32) = match endianness {
                    Endianness::BigEndian => be_u32,
                    Endianness::LittleEndian => le_u32,
                }(i)?;
                let i = i.advance_to_boundary(item_type.marshalling_boundary())?;
                let (i, items): (I, Vec<Type>) = map_parser(take(length_of_array_data_in_bytes), all_consuming(many0(|i| item_type.unmarshal_inner(i, endianness))))(i)?;

                let dbus_array = DBusArray { item_type: item_type.deref().clone(), items };

                (i, Type::from(dbus_array))
            }
            Self::DBusStruct { fields } => {
                todo!();
            }
            Self::DBusVariant => {
                todo!();
            }
            Self::DBusUnixFileDescriptor => {
                todo!();
            }
            Self::DBusDictEntry { key, value } => {
                todo!();
            }
        };

        Ok((i, type_))
    }
}

impl DBusString {
    fn unmarshal<'i>(i: I<'i>, endianness: Endianness) -> IResult<I<'i>, Self> {
        if endianness == Endianness::LittleEndian {
            todo!();
        }

        let i = i.advance_to_boundary(Self::alignment())?;

        // The first 4 bytes encode the string's length in bytes, excluding its terminating null.
        let (i, length): (I, u32) = be_u32(i)?;

        // Now we know the length in bytes of the string that follows.
        // DBus strings are UTF-8 encoded, so we need to decode the bytes we take as UTF-8
        // in order to get a Rust string. Since it might not be valid UTF-8, converting the
        // bytes to a str slice returns a Result with an error type from the std lib.
        // Therefore, we map that result to nom's IResult with nom's map_res function.
        let first = take(length);
        let second = |i: I<'i>| std::str::from_utf8(i.data);
        let (i, str_slice): (I, &str) = map_res(first, second)(i)?;

        // The string must then be followed by a null byte:
        let i = skip_null_byte(i)?;

        Ok((i, DBusString::from(str_slice)))
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

        let dba: DBusArray = match SingleCompleteTypeSignature::DBusArray(Box::new(SingleCompleteTypeSignature::DBusByte)).unmarshal(&a, Endianness::BigEndian).unwrap() {
            Type::Container(ContainerType::DBusArray(dbus_array)) => dbus_array,
            _ => panic!(),
        };

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
        let a: [u8; 9] = [b'y', b'b', b'n', b'q', b'i', b'u', b'x', b't', b'd'];

        let (i, x) = DBusSignature::unmarshal_be(I::new(&a)).unwrap();

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
