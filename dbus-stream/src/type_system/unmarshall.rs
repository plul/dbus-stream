use std::iter::Copied;
use std::iter::Enumerate;
use std::iter::Iterator;
use std::ops::RangeFrom;
use std::slice::Iter;

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
use nom::InputIter;
use nom::Needed;

use super::signature::SingleCompleteTypeSignature;
use super::signature::HEADER_FIELD_SIGNATURE;
use super::types::*;
use super::Endianness;
use crate::message_protocol::Message;
use crate::message_protocol::MessageType;

trait Unmarshal: Sized {
    fn unmarshall_be<'a>(i: Decoder<'a>) -> IResult<Decoder<'a>, Self>;
}

/// A wrapper over `&[u8]` but with the ability to keep track global alignment.
#[derive(Copy, Clone, Debug)]
struct Decoder<'a> {
    /// The unmodified original slice the Decoder was created with.
    /// This is assumed to begin on an 8-byte boundary.
    ///
    /// This is used for global alignment.
    original_data: &'a [u8],

    /// Slice that is chipped away at, like a normal input as it passes through nom parsers.
    data: &'a [u8],
}

impl<'a> Decoder<'a> {
    /// Create new [Decoder] from raw data.
    ///
    /// The data slice must be globally aligned so it starts on an 8-byte boundary!
    fn new(data: &'a [u8]) -> Self {
        Self {
            original_data: data,
            data: data.clone(),
        }
    }

    /// Skip over padding (null bytes) until at an n-byte boundary (relative to global alignment).
    fn advance_to_boundary(
        self,
        boundary: usize,
    ) -> Result<Decoder<'a>, nom::Err<nom::error::Error<Decoder<'a>>>> {
        let i = self;

        // Don't really expect to need this for other boundaries than 2, 4 and 8.
        debug_assert!([1, 2, 4, 8].contains(&boundary), "Sanity check");

        // Original slice is guaranteed to start on 8-byte boundary.
        let offset = while (i.original_data.len() - i.data.len()) % boundary != 0 {
            // Eat a null byte.
            let (i, _) = tag(&[0])(i)?;
        };

        Ok(i)
    }
}

impl<'a> nom::Slice<RangeFrom<usize>> for Decoder<'a> {
    fn slice(&self, range: RangeFrom<usize>) -> Self {
        Self {
            original_data: &self.original_data,
            data: &self.data[range],
        }
    }
}

impl<'a> nom::InputIter for Decoder<'a> {
    type Item = u8;
    type Iter = Enumerate<Self::IterElem>;
    type IterElem = Copied<Iter<'a, u8>>;

    fn iter_indices(&self) -> Self::Iter {
        self.data.iter_indices()
    }

    fn iter_elements(&self) -> Self::IterElem {
        self.data.iter_elements()
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.data.position(predicate)
    }

    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        self.data.slice_index(count)
    }
}

impl<'a> nom::InputLength for Decoder<'a> {
    fn input_len(&self) -> usize {
        self.data.input_len()
    }
}

impl<'a> nom::InputTake for Decoder<'a> {
    fn take(&self, count: usize) -> Self {
        Self {
            original_data: self.original_data,
            data: self.data.take(count),
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let (a, b) = self.data.take_split(count);

        let a = Self {
            original_data: self.original_data,
            data: a,
        };

        let b = Self {
            original_data: self.original_data,
            data: b,
        };

        (a, b)
    }
}

impl<'a, 'b> nom::Compare<&'b [u8]> for Decoder<'a> {
    fn compare(&self, t: &'b [u8]) -> nom::CompareResult {
        self.data.compare(t)
    }

    fn compare_no_case(&self, t: &'b [u8]) -> nom::CompareResult {
        self.data.compare_no_case(t)
    }
}

// Replace this shit with const generics whenever the underlying nom stuff is updated with const generics.
impl<'a, 'b> nom::Compare<&'b [u8; 1]> for Decoder<'a> {
    fn compare(&self, t: &'b [u8; 1]) -> nom::CompareResult {
        self.data.compare(t)
    }

    fn compare_no_case(&self, t: &'b [u8; 1]) -> nom::CompareResult {
        self.data.compare_no_case(t)
    }
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
    fn unmarshall_be<'a>(i: Decoder<'a>) -> IResult<Decoder<'a>, Self> {
        let (i, value) = be_u8(i)?;

        let unmarshalled: Self = Self { u8: value };

        Ok((i, unmarshalled))
    }
}

impl Unmarshal for DBusBoolean {
    fn unmarshall_be<'a>(i: Decoder<'a>) -> IResult<Decoder<'a>, Self> {
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

/// Unmarshall a DBus message (consisting of header and body),
pub fn unmarshall_message(message: &[u8]) -> crate::Result<Message> {
    let (_i, message) = all_consuming(unmarshall_message_parse)(message)
        .finish()
        .map_err(|_err| crate::Error::ParseError)?;

    Ok(message)
}

fn unmarshall_message_parse(i: &[u8]) -> IResult<&[u8], Message> {
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
    let flag_no_reply_expected: bool = 0x1 & flag_bitfield == 0x1;
    let flag_no_auto_start: bool = 0x2 & flag_bitfield == 0x2;
    let flag_allow_interactive_authorization: bool = 0x4 & flag_bitfield == 0x4;

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
        fn unmarshall_basic_type(b: u8) -> SingleCompleteTypeSignature {
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
                v.push(unmarshall_basic_type(b));

                pos += 1;
            }
        }

        Ok((i, Self { vec: v }))
    }
}

impl DBusUnixFileDescriptor {
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        todo!("Dani. But this one is low priority, could just be left as todo");
    }
}

impl DBusArray {
    fn unmarshall_be<'a>(
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
                    let (_, b) = DBusByte::unmarshall_be(&i[pos..])?;
                    dba.push(BasicType::from(b));
                }
            }
            SingleCompleteTypeSignature::DBusBoolean => {
                for pos in (0..length as usize).step_by(size) {
                    let (_, b) = DBusBoolean::unmarshall_be(&i[pos..])?;
                    dba.push(BasicType::from(b));
                }
            }
            SingleCompleteTypeSignature::DBusInt16 => {
                for pos in (0..length as usize).step_by(size) {
                    let (_, b) = DBusInt16::unmarshall_be(&i[pos..])?;
                    dba.push(BasicType::from(b));
                }
            }
            SingleCompleteTypeSignature::DBusUint16 => {
                for pos in (0..length as usize).step_by(size) {
                    let (_, b) = DBusUint16::unmarshall_be(&i[pos..])?;
                    dba.push(BasicType::from(b));
                }
            }
            SingleCompleteTypeSignature::DBusInt32 => {
                for pos in (0..length as usize).step_by(size) {
                    let (_, b) = DBusInt32::unmarshall_be(&i[pos..])?;
                    dba.push(BasicType::from(b));
                }
            }
            SingleCompleteTypeSignature::DBusUint32 => {
                for pos in (0..length as usize).step_by(size) {
                    let (_, b) = DBusUint32::unmarshall_be(&i[pos..])?;
                    dba.push(BasicType::from(b));
                }
            }
            SingleCompleteTypeSignature::DBusInt64 => {
                for pos in (0..length as usize).step_by(size) {
                    let (_, b) = DBusInt64::unmarshall_be(&i[pos..])?;
                    dba.push(BasicType::from(b));
                }
            }
            SingleCompleteTypeSignature::DBusUint64 => {
                for pos in (0..length as usize).step_by(size) {
                    let (_, b) = DBusUint64::unmarshall_be(&i[pos..])?;
                    dba.push(BasicType::from(b));
                }
            }
            SingleCompleteTypeSignature::DBusDouble => {
                for pos in (0..length as usize).step_by(size) {
                    let (_, b) = DBusDouble::unmarshall_be(&i[pos..])?;
                    dba.push(BasicType::from(b));
                }
            }
            SingleCompleteTypeSignature::DBusString => {
                for pos in (0..length as usize).step_by(size) {
                    let (_, b) = DBusString::unmarshall_be(&i[pos..])?;
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
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        todo!("Dani? These container types may be a little harder. Consider if it needs to know the type of the fields up front?");
    }
}

impl DBusVariant {
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        todo!("Dani? These container types may be a little harder");
    }
}

impl DBusDictEntry {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unmarshall_array_of_bytes() {
        let a: [u8; 7] = [0, 0, 0, 3, 15, 16, 17];
        let b: [DBusByte; 3] = [
            DBusByte { u8: 15 },
            DBusByte { u8: 16 },
            DBusByte { u8: 17 },
        ];

        let (_, dba) = DBusArray::unmarshall_be(&a, &SingleCompleteTypeSignature::DBusByte).unwrap();

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
    fn unmarshall_basic_signature() {
        let a: [u8; 10] = [9, b'y', b'b', b'n', b'q', b'i', b'u', b'x', b't', b'd'];

        let (_, x) = DBusSignature::unmarshall_be(&a).unwrap();

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
