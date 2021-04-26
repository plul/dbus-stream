use std::iter::Copied;
use std::iter::Enumerate;
use std::ops::RangeFrom;
use std::slice::Iter;

use crate::type_system::unmarshal::parsers::complete::skip_null_byte;

/// A wrapper over `&[u8]` but with the ability to keep track of global alignment.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct I<'a> {
    /// The wrapped byte slice.
    pub data: &'a [u8],

    /// Alignment, in the range of 0 to 7 inclusive.
    ///
    /// This is 0 when the slice is globally aligned.
    ///
    /// Example:
    /// If the original data is [1,2,3,4,5] and this slice is [3,4], then alignment is 2.
    pub alignment: usize,
}

impl<'a> I<'a> {
    /// The data slice is assumed to be globally aligned so it starts on an 8-byte boundary.
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            alignment: 0,
            data: data.clone(),
        }
    }

    /// Skip over padding (null bytes) until at an n-byte boundary (relative to global alignment).
    pub fn advance_to_boundary(
        self,
        boundary: usize,
    ) -> Result<I<'a>, nom::Err<nom::error::Error<I<'a>>>> {
        let mut i = self;

        // Don't really expect to need this for other boundaries than 2, 4 and 8.
        debug_assert!([1, 2, 4, 8].contains(&boundary), "Sanity check");

        while i.alignment != 0 {
            i = skip_null_byte(i)?;
        }

        Ok(i)
    }
}

impl<'a> nom::Slice<RangeFrom<usize>> for I<'a> {
    fn slice(&self, range: RangeFrom<usize>) -> Self {
        let data = &self.data[range.clone()];
        let alignment: usize = (self.alignment + range.start) % 8;

        Self { alignment, data }
    }
}

impl<'a> nom::InputIter for I<'a> {
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

    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
        self.data.slice_index(count)
    }
}

impl<'a> nom::InputLength for I<'a> {
    fn input_len(&self) -> usize {
        self.data.input_len()
    }
}

impl<'a> nom::InputTake for I<'a> {
    fn take(&self, count: usize) -> Self {
        Self {
            alignment: self.alignment,
            data: self.data.take(count),
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let (suffix, prefix) = self.data.take_split(count);

        let prefix = Self {
            data: prefix,
            alignment: self.alignment,
        };

        let suffix = Self {
            data: suffix,
            alignment: (self.alignment + count) % 8,
        };

        (suffix, prefix)
    }
}

impl<'a, 'b> nom::Compare<&'b [u8]> for I<'a> {
    fn compare(&self, t: &'b [u8]) -> nom::CompareResult {
        self.data.compare(t)
    }

    fn compare_no_case(&self, t: &'b [u8]) -> nom::CompareResult {
        self.data.compare_no_case(t)
    }
}

// Replace this shit with const generics whenever the underlying nom stuff is updated with const generics.
// // The const generics version:
// impl<'a, 'b, const N: usize> nom::Compare<&'b [u8; N]> for Decoder<'a> {
//     fn compare(&self, t: &'b [u8; N]) -> nom::CompareResult {
//         self.data.compare(t)
//     }

//     fn compare_no_case(&self, t: &'b [u8; N]) -> nom::CompareResult {
//         self.data.compare_no_case(t)
//     }
// }
impl<'a, 'b> nom::Compare<&'b [u8; 1]> for I<'a> {
    fn compare(&self, t: &'b [u8; 1]) -> nom::CompareResult {
        self.data.compare(t)
    }

    fn compare_no_case(&self, t: &'b [u8; 1]) -> nom::CompareResult {
        self.data.compare_no_case(t)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use nom::*;

    #[test]
    fn tag() -> Result<(), Box<dyn std::error::Error>> {
        let i = I::new(&[0, 1, 2, 3, 4]);
        assert_eq!(i.alignment, 0);

        let t: &[u8] = &[0, 1, 2];
        let result: nom::IResult<I, I> = nom::bytes::complete::tag(t)(i);
        let (i, parsed): (I, I) = result?;

        assert_eq!(parsed.data, &[0, 1, 2]);
        assert_eq!(parsed.alignment, 0);
        assert_eq!(i.data, &[3, 4]);
        assert_eq!(i.alignment, 3);

        Ok(())
    }

    #[test]
    fn take_split_alignment() -> Result<(), Box<dyn std::error::Error>> {
        let i = I::new(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        assert_eq!(i.alignment, 0);

        let (suffix, prefix) = i.take_split(3);
        assert_eq!(prefix.data, &[0, 1, 2]);
        assert_eq!(prefix.alignment, 0);
        assert_eq!(suffix.data, &[3, 4, 5, 6, 7, 8, 9, 10]);
        assert_eq!(suffix.alignment, 3);

        Ok(())
    }
}
