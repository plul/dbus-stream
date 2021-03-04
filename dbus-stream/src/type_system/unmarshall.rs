use std::convert::TryFrom;

use nom::bytes::complete::tag;
use nom::bytes::complete::take;
use nom::combinator::map_res;
use nom::number::complete::be_u32;
use nom::number::complete::le_u32;
use nom::IResult;

use super::types::*;

impl DBusByte {
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        todo!("Dani");
    }
}

impl DBusBoolean {
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        todo!("Dani");
    }
}

impl DBusInt16 {
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        todo!("Dani");
    }
}

impl DBusUint16 {
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        todo!("Dani");
    }
}

impl DBusInt32 {
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        todo!("Dani");
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
        todo!("Dani");
    }
}

impl DBusUint64 {
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        todo!("Dani");
    }
}

impl DBusDouble {
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        todo!("Dani");
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
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        todo!("Dani? These container types may be a little harder");
    }
}

impl DBusStruct {
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        todo!("Dani? These container types may be a little harder");
    }
}

impl DBusVariant {
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        todo!("Dani? These container types may be a little harder");
    }
}

impl DBusMap {
    fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
        todo!("Dani? These container types may be a little harder");
    }
}
