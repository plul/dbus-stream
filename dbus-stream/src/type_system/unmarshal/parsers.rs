//! Parsers to be used with nom.

pub mod complete {
    use std::ops::RangeFrom;

    use nom::combinator::verify;
    use nom::number::complete::be_u8;

    /// Eat a single null byte.
    ///
    /// The null byte is not returned.
    pub fn skip_null_byte<I, E>(input: I) -> Result<I, nom::Err<E>>
    where
        I: nom::Slice<RangeFrom<usize>>,
        I: nom::InputIter<Item = u8>,
        I: nom::InputLength,
        I: Clone,
        E: nom::error::ParseError<I>,
    {
        let (input, _) = verify(be_u8, |&byte| byte == 0x00)(input)?;
        Ok(input)
    }
}
