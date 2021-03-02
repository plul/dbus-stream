use nom::IResult;
use nom::{
    bytes::complete::take,
    number::complete::{be_u32, le_u32},
};

use super::*;

impl DBusString {
    // fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
    //     let (i, length) = be_u32(i)?;
    //     let (i, string_bytes) = take(length)(i)?;
    //     let string = String::from_utf8(string_bytes)?;
    //     let null = tag(null);
    // }
}
