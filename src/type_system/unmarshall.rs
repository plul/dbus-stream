use nom::bytes::complete::take;
use nom::number::complete::be_u32;
use nom::number::complete::le_u32;
use nom::IResult;

use super::*;

impl DBusString {
    // fn unmarshall_be(i: &[u8]) -> IResult<&[u8], Self> {
    //     let (i, length) = be_u32(i)?;
    //     let (i, string_bytes) = take(length)(i)?;
    //     let string = String::from_utf8(string_bytes)?;
    //     let null = tag(null);
    // }
}
