use std::iter::Copied;
use std::iter::Enumerate;
use std::ops::RangeFrom;
use std::slice::Iter;

use nom::bytes::complete::tag;

/// A wrapper over `&[u8]` but with the ability to keep track global alignment.
#[derive(Copy, Clone, Debug)]
pub struct Decoder<'a> {
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
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            original_data: data,
            data: data.clone(),
        }
    }

    /// Skip over padding (null bytes) until at an n-byte boundary (relative to global alignment).
    pub fn advance_to_boundary(
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

    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
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
