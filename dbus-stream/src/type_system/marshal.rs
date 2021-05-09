use std::convert::TryFrom;

use crate::type_system::signature::Signature;
use crate::type_system::types::*;

#[derive(Debug, Default)]
pub(crate) struct Encoder {
    pub buf: Vec<u8>,
}

pub(crate) trait Marshal<T> {
    fn marshal_be(&mut self, t: &T) -> crate::Result<()>;
}

impl Encoder {
    pub fn finish(self) -> Vec<u8> {
        self.buf
    }

    /// Push null bytes until aligned
    pub fn align(&mut self, alignment: usize) {
        debug_assert!(
            [1, 2, 4, 8].contains(&alignment),
            "I don't expect I'll need this method for other alignments"
        );

        while self.buf.len() % alignment != 0 {
            self.buf.push(0);
        }
    }

    pub fn extend_from_array<const N: usize>(&mut self, array: [u8; N]) {
        // Note: this way of converting the array to an iterator might be deprecated in the future, after [IntoIterator is implemented for arrays](https://github.com/rust-lang/rust/pull/65819).
        let iter = std::array::IntoIter::new(array);
        self.buf.extend(iter);
    }

    /// Reserve N bytes and return a closure that can be called to set the bytes later.
    ///
    /// The closure must be called with a mutable instance of the same [Encoder], otherwise
    /// the behaviour is undefined.
    ///
    /// This method pushes N null bytes, but returns a closure that remembers the index.
    /// The closure when called, will overwrite these same bytes.
    ///
    /// This is intended to help length-value encoding, when the length isn't known up front.
    pub fn reserve_n_bytes<'a, 'b, const N: usize>(
        &'a mut self,
    ) -> impl FnOnce(&'b mut Encoder, [u8; N]) {
        let idx = self.buf.len();

        self.buf.extend_from_slice(&[0; N]);

        let closure = move |encoder: &mut Encoder, values: [u8; N]| {
            let new_values_iter = std::array::IntoIter::new(values);
            let range = idx..idx + N;

            // Replace:
            for old_value in encoder.buf.splice(range, new_values_iter) {
                // These are the values being evicted from the vec.
                // These should be zero, that's what we set them to above.
                debug_assert_eq!(old_value, 0);
            }
        };

        closure
    }
}

impl Marshal<Type> for Encoder {
    fn marshal_be(&mut self, t: &Type) -> crate::Result<()> {
        match t {
            Type::Basic(inner) => self.marshal_be(inner),
            Type::Container(inner) => self.marshal_be(inner),
        }
    }
}

impl Marshal<BasicType> for Encoder {
    fn marshal_be(&mut self, t: &BasicType) -> crate::Result<()> {
        match t {
            BasicType::DBusByte(inner) => self.marshal_be(inner),
            BasicType::DBusBoolean(inner) => self.marshal_be(inner),
            BasicType::DBusInt16(inner) => self.marshal_be(inner),
            BasicType::DBusUint16(inner) => self.marshal_be(inner),
            BasicType::DBusInt32(inner) => self.marshal_be(inner),
            BasicType::DBusUint32(inner) => self.marshal_be(inner),
            BasicType::DBusInt64(inner) => self.marshal_be(inner),
            BasicType::DBusUint64(inner) => self.marshal_be(inner),
            BasicType::DBusDouble(inner) => self.marshal_be(inner),
            BasicType::DBusString(inner) => self.marshal_be(inner),
            BasicType::DBusObjectPath(inner) => self.marshal_be(inner),
            BasicType::DBusSignature(inner) => self.marshal_be(inner),
            BasicType::DBusUnixFileDescriptor(inner) => self.marshal_be(inner),
        }
    }
}

impl Marshal<ContainerType> for Encoder {
    fn marshal_be(&mut self, t: &ContainerType) -> crate::Result<()> {
        match t {
            ContainerType::DBusArray(inner) => self.marshal_be(inner),
            ContainerType::DBusStruct(inner) => self.marshal_be(inner),
            ContainerType::DBusVariant(inner) => self.marshal_be(inner),
            ContainerType::DBusDictEntry(inner) => self.marshal_be(inner),
        }
    }
}

impl Marshal<DBusByte> for Encoder {
    fn marshal_be(&mut self, t: &DBusByte) -> crate::Result<()> {
        self.buf.push(t.u8);
        Ok(())
    }
}

impl Marshal<DBusBoolean> for Encoder {
    fn marshal_be(&mut self, t: &DBusBoolean) -> crate::Result<()> {
        let value: u32 = if t.bool { 1 } else { 0 };
        self.marshal_be(&DBusUint32 { u32: value })
    }
}

impl Marshal<DBusInt16> for Encoder {
    fn marshal_be(&mut self, t: &DBusInt16) -> crate::Result<()> {
        self.align(2);
        self.extend_from_array(t.i16.to_be_bytes());
        Ok(())
    }
}

impl Marshal<DBusUint16> for Encoder {
    fn marshal_be(&mut self, t: &DBusUint16) -> crate::Result<()> {
        self.align(2);
        self.extend_from_array(t.u16.to_be_bytes());
        Ok(())
    }
}

impl Marshal<DBusInt32> for Encoder {
    fn marshal_be(&mut self, t: &DBusInt32) -> crate::Result<()> {
        self.align(4);
        self.extend_from_array(t.i32.to_be_bytes());
        Ok(())
    }
}

impl Marshal<DBusUint32> for Encoder {
    fn marshal_be(&mut self, t: &DBusUint32) -> crate::Result<()> {
        self.align(4);
        self.extend_from_array(t.u32.to_be_bytes());
        Ok(())
    }
}

impl Marshal<DBusInt64> for Encoder {
    fn marshal_be(&mut self, t: &DBusInt64) -> crate::Result<()> {
        self.align(8);
        self.extend_from_array(t.i64.to_be_bytes());
        Ok(())
    }
}

impl Marshal<DBusUint64> for Encoder {
    fn marshal_be(&mut self, t: &DBusUint64) -> crate::Result<()> {
        self.align(8);
        self.extend_from_array(t.u64.to_be_bytes());
        Ok(())
    }
}

impl Marshal<DBusDouble> for Encoder {
    fn marshal_be(&mut self, t: &DBusDouble) -> crate::Result<()> {
        self.align(8);
        self.extend_from_array(t.f64.to_be_bytes());
        Ok(())
    }
}

impl Marshal<DBusString> for Encoder {
    fn marshal_be(&mut self, t: &DBusString) -> crate::Result<()> {
        self.align(4);

        // Length of string (in bytes):
        let length: usize = t.string.len();
        let length: u32 = u32::try_from(length)?;
        let length: [u8; 4] = length.to_be_bytes();
        self.extend_from_array(length);

        // The Rust string is UTF-8, and DBus uses UTF-8 for its strings too.
        self.buf.extend(t.string.bytes());

        // Terminating null byte.
        self.buf.push(0x00);

        Ok(())
    }
}

impl Marshal<DBusObjectPath> for Encoder {
    fn marshal_be(&mut self, t: &DBusObjectPath) -> crate::Result<()> {
        // Marshalls the same way as DBusString.
        self.marshal_be(&t.dbus_string)
    }
}

impl Marshal<DBusSignature> for Encoder {
    fn marshal_be(&mut self, t: &DBusSignature) -> crate::Result<()> {
        // Reserve 1 byte for the length. We don't know the exact length yet.
        let specify_length = self.reserve_n_bytes::<1>();

        // Mark the offset, so we know where the items start.
        let offset_first_item = self.buf.len();

        // Write the marshalled single complete type signatures into the buffer.
        for single_complete_type_signature in &t.vec {
            self.buf.extend(single_complete_type_signature.serialize());
        }

        // Check what the length is
        let length = u8::try_from(self.buf.len() - offset_first_item)?;
        specify_length(self, length.to_be_bytes());

        // Terminating null byte.
        self.buf.push(0x00);

        Ok(())
    }
}

impl Marshal<DBusUnixFileDescriptor> for Encoder {
    fn marshal_be(&mut self, t: &DBusUnixFileDescriptor) -> crate::Result<()> {
        todo!()
    }
}

impl Marshal<DBusVariant> for Encoder {
    fn marshal_be(&mut self, t: &DBusVariant) -> crate::Result<()> {
        // Single Complete Type signature of variant value
        let sig = t.variant.signature();
        let dbus_signature = DBusSignature { vec: vec![sig] };

        // Variant signature
        self.marshal_be(&dbus_signature)?;
        // Variant inner type
        self.marshal_be(&*t.variant)?;

        Ok(())
    }
}

impl Marshal<DBusArray> for Encoder {
    fn marshal_be(&mut self, t: &DBusArray) -> crate::Result<()> {
        // The DBus array is length-value encoded, and the length is 4 aligned:
        self.align(4);

        // Reserve 4 bytes for the length. We don't know the exact length yet.
        let specify_length = self.reserve_n_bytes::<4>();

        // Add the padding that comes after the length, and before the first item.
        self.align(t.item_type.marshalling_boundary());

        // Mark the offset, so we know where the items start.
        let offset_first_item = self.buf.len();

        // Marshal the items.
        for item in &t.items {
            debug_assert_eq!(item.signature(), t.item_type, "Sanity check");
            self.marshal_be(item)?;
        }

        let array_data_length = u32::try_from(self.buf.len() - offset_first_item)?;
        specify_length(self, array_data_length.to_be_bytes());

        Ok(())
    }
}

impl Marshal<DBusStruct> for Encoder {
    fn marshal_be(&mut self, t: &DBusStruct) -> crate::Result<()> {
        // Struct starts on 8-byte boundary regardless of the type of its fields.
        self.align(8);

        for field in &t.fields {
            self.marshal_be(field)?;
        }

        Ok(())
    }
}

impl Marshal<DBusDictEntry> for Encoder {
    fn marshal_be(&mut self, t: &DBusDictEntry) -> crate::Result<()> {
        todo!()
    }
}
