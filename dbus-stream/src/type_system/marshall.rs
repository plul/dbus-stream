use std::convert::TryFrom;

use super::signature::SingleCompleteTypeSignature;
use crate::type_system::types::*;

impl Type {
    pub fn marshall_be(&self) -> crate::Result<Vec<u8>> {
        let vec: Vec<u8> = match self {
            Type::Basic(inner) => inner.marshall_be()?,
            Type::Container(inner) => inner.marshall_be()?,
        };

        Ok(vec)
    }
}

impl BasicType {
    pub fn marshall_be(&self) -> crate::Result<Vec<u8>> {
        let vec: Vec<u8> = match self {
            BasicType::Byte(inner) => vec![inner.marshall()],
            BasicType::Boolean(inner) => Vec::from(inner.marshall_be()),
            BasicType::Int16(inner) => Vec::from(inner.marshall_be()),
            BasicType::Uint16(inner) => Vec::from(inner.marshall_be()),
            BasicType::Int32(inner) => Vec::from(inner.marshall_be()),
            BasicType::Uint32(inner) => Vec::from(inner.marshall_be()),
            BasicType::Int64(inner) => Vec::from(inner.marshall_be()),
            BasicType::Uint64(inner) => Vec::from(inner.marshall_be()),
            BasicType::Double(inner) => Vec::from(inner.marshall_be()),
            BasicType::String(inner) => inner.marshall_be()?,
            BasicType::ObjectPath(inner) => inner.marshall_be()?,
            BasicType::Signature(inner) => inner.marshall_be()?,
            BasicType::UnixFileDescriptor(inner) => Vec::from(inner.marshall_be()),
        };

        Ok(vec)
    }
}

impl ContainerType {
    pub fn marshall_be(&self) -> crate::Result<Vec<u8>> {
        let vec: Vec<u8> = match self {
            ContainerType::Array(inner) => inner.marshall_be()?,
            ContainerType::Struct(inner) => inner.marshall_be()?,
            ContainerType::Variant(inner) => inner.marshall_be()?,
            ContainerType::Map(inner) => inner.marshall_be(),
        };

        Ok(vec)
    }
}

// pub trait Marshall {
//     fn marshall_be(&self) -> crate::Result<Vec<u8>> {
//         // Default impl, remove when all types provide an impl.
//         todo!();
//     }

//     fn marshall_le(&self) -> crate::Result<Vec<u8>> {
//         // Default impl, remove when all types provide an impl.
//         todo!();
//     }

//     fn marshall(&self, endianness: Endianness) -> crate::Result<Vec<u8>> {
//         match endianness {
//             Endianness::BigEndian => self.marshall_be(),
//             Endianness::LittleEndian => self.marshall_le(),
//         }
//     }
// }

impl DBusByte {
    pub fn marshall(&self) -> u8 {
        self.u8
    }
}

impl DBusBoolean {
    pub fn marshall_be(&self) -> [u8; 4] {
        let value: u32 = if self.bool { 1 } else { 0 };

        DBusUint32 { u32: value }.marshall_be()
    }
}

impl DBusInt16 {
    pub fn marshall_be(&self) -> [u8; 2] {
        self.i16.to_be_bytes()
    }
}

impl DBusUint16 {
    pub fn marshall_be(&self) -> [u8; 2] {
        self.u16.to_be_bytes()
    }
}

impl DBusInt32 {
    pub fn marshall_be(&self) -> [u8; 4] {
        self.i32.to_be_bytes()
    }
}

impl DBusUint32 {
    pub fn marshall_be(&self) -> [u8; 4] {
        self.u32.to_be_bytes()
    }
}

impl DBusInt64 {
    pub fn marshall_be(&self) -> [u8; 8] {
        self.i64.to_be_bytes()
    }
}

impl DBusUint64 {
    pub fn marshall_be(&self) -> [u8; 8] {
        self.u64.to_be_bytes()
    }
}

impl DBusDouble {
    pub fn marshall_be(&self) -> [u8; 8] {
        self.f64.to_be_bytes()
    }
}

impl DBusString {
    pub fn marshall_be(&self) -> crate::Result<Vec<u8>> {
        let mut v = Vec::new();

        let length = self.string.len();
        let length = u32::try_from(length)?;

        v.extend_from_slice(&length.to_be_bytes());
        v.extend(self.string.bytes());

        // Terminating null byte.
        v.push(0x00);

        Ok(v)
    }
}

impl DBusObjectPath {
    pub fn marshall_be(&self) -> crate::Result<Vec<u8>> {
        // Marshalls the same way as DBusString.
        self.dbus_string.marshall_be()
    }
}

impl DBusSignature {
    /// Marshall big-endian.
    ///
    /// DBusSignature marshalls the same way as DBusString, except length is on a single
    /// byte.
    pub fn marshall_be(&self) -> crate::Result<Vec<u8>> {
        let mut v = Vec::new();

        // This is a length-value encoding. The first byte is the length, on a single byte.
        // Set it to zero for now,
        v.push(0);

        // Write the marshalled single complete type signatures into the buffer.
        for single_complete_type_signature in &self.vec {
            v.extend(single_complete_type_signature.marshall());
        }

        // Now we can check what the length is, minus the byte at the front that encodes the length.
        let length = u8::try_from(v.len() - 1)?;
        // And update:
        v[0] = length;

        // Terminating null byte.
        v.push(0x00);

        // TODO: enforce max length of signature is 255 (bytes). But remember, this is only a single complete type being marshalled in this function. Somewhere else the entire signature of bodies is being composed.

        Ok(v)
    }
}

impl DBusUnixFileDescriptor {
    pub fn marshall_be(&self) -> [u8; 4] {
        todo!()
    }
}

impl DBusVariant {
    pub fn marshall_be(&self) -> crate::Result<Vec<u8>> {
        let signature = self.variant.signature().marshall();
        let value = self.variant.marshall_be()?;

        let mut v: Vec<u8> = Vec::new();
        v.extend(signature);
        v.extend(value);
        Ok(v)
    }
}

impl DBusArray {
    pub fn marshall_be(&self) -> crate::Result<Vec<u8>> {
        // Items are marshalled in sequence.
        let mut marshalled_items: Vec<u8> = Vec::new();
        for item in &self.items {
            marshalled_items.extend(item.marshall_be()?);
        }

        // First thing added to the vec is the length in bytes of the coming items.
        let mut v: Vec<u8> = Vec::new();
        let length = marshalled_items.len();
        let length = u32::try_from(length)?;
        v.extend_from_slice(&length.to_be_bytes());

        // See https://dbus.freedesktop.org/doc/api/html/structDBusHeader.html
        todo!("whoops - I suspect the alignment needs to be relative to the message as a whole");

        // Second thing in the vec is alignment padding, to align with the boundary of the items of the array.
        let boundary = self.item_type.marshalling_boundary();
        while v.len() % boundary != 0 {
            v.push(0);
        }

        // Third, we add the actual items.
        v.extend(marshalled_items);

        Ok(v)
    }
}

impl DBusStruct {
    pub fn marshall_be(&self) -> crate::Result<Vec<u8>> {
        let mut v = Vec::new();
        for field in &self.fields {
            v.extend(field.marshall_be()?);
        }
        Ok(v)
    }
}

impl DBusMap {
    pub fn marshall_be(&self) -> Vec<u8> {
        todo!()
    }
}

impl SingleCompleteTypeSignature {
    pub fn marshall(&self) -> Vec<u8> {
        let mut v: Vec<u8> = Vec::new();

        match self {
            Self::Byte => {
                v.push(b'y');
            }
            Self::Boolean => {
                v.push(b'b');
            }
            Self::Int16 => {
                v.push(b'n');
            }
            Self::Uint16 => {
                v.push(b'q');
            }
            Self::Int32 => {
                v.push(b'i');
            }
            Self::Uint32 => {
                v.push(b'u');
            }
            Self::Int64 => {
                v.push(b'x');
            }
            Self::Uint64 => {
                v.push(b't');
            }
            Self::Double => {
                v.push(b'd');
            }
            Self::String => {
                v.push(b's');
            }
            Self::ObjectPath => {
                v.push(b'o');
            }
            Self::Signature => {
                v.push(b'g');
            }
            Self::UnixFileDescriptor => {
                v.push(b'h');
            }
            Self::Array(inner) => {
                v.push(b'a');
                v.extend(inner.marshall());
            }
            Self::Struct { fields } => {
                v.push(b'(');
                for field in fields {
                    v.extend(field.marshall());
                }
                v.push(b')');
            }
            Self::Variant => {
                v.push(b'v');
            }
            Self::Map { key, value } => {
                v.push(b'a');
                v.push(b'{');
                v.extend(key.marshall());
                v.extend(value.marshall());
                v.push(b'}');
            }
        }

        // TODO: enforce max depths

        v
    }
}
