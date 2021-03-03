use std::convert::TryFrom;

use crate::type_system::types::*;

impl Type {
    pub fn marshall_be(&self) -> crate::Result<Vec<u8>> {
        let vec: Vec<u8> = match self {
            Type::Basic(inner) => inner.marshall_be()?,
            Type::Container(inner) => inner.marshall_be(),
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
    pub fn marshall_be(&self) -> Vec<u8> {
        match self {
            ContainerType::Array(inner) => inner.marshall_be(),
            ContainerType::Struct(inner) => inner.marshall_be(),
            ContainerType::Variant(inner) => inner.marshall_be(),
            ContainerType::Map(inner) => inner.marshall_be(),
        }
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

// // DBus Struct with two fields
// impl<T, U> (T, U)
// where
//     T: DBusType,
//     U: DBusType,
// {
//     fn marshall_be(&self) -> crate::Result<Vec<u8>> {
//         let mut v = Vec::new();

//         v.extend_from_slice(&self.0.marshall_be()?);
//         v.extend_from_slice(&self.1.marshall_be()?);

//         while v.len() % 8 != 0 {
//             v.push(0x00);
//         }

//         Ok(v)
//     }
// }

impl DBusByte {
    fn marshall(&self) -> u8 {
        self.u8
    }
}

impl DBusBoolean {
    fn marshall_be(&self) -> [u8; 4] {
        let value: u32 = if self.bool { 1 } else { 0 };

        DBusUint32 { u32: value }.marshall_be()
    }
}

impl DBusInt16 {
    fn marshall_be(&self) -> [u8; 2] {
        self.i16.to_be_bytes()
    }
}

impl DBusUint16 {
    fn marshall_be(&self) -> [u8; 2] {
        self.u16.to_be_bytes()
    }
}

impl DBusInt32 {
    fn marshall_be(&self) -> [u8; 4] {
        self.i32.to_be_bytes()
    }
}

impl DBusUint32 {
    fn marshall_be(&self) -> [u8; 4] {
        self.u32.to_be_bytes()
    }
}

impl DBusInt64 {
    fn marshall_be(&self) -> [u8; 8] {
        self.i64.to_be_bytes()
    }
}

impl DBusUint64 {
    fn marshall_be(&self) -> [u8; 8] {
        self.u64.to_be_bytes()
    }
}

impl DBusDouble {
    fn marshall_be(&self) -> [u8; 8] {
        self.f64.to_be_bytes()
    }
}

impl DBusString {
    fn marshall_be(&self) -> crate::Result<Vec<u8>> {
        let mut v = Vec::new();

        let length = self.string.len();
        let length = u32::try_from(length)?;

        v.extend_from_slice(&length.to_be_bytes());
        v.extend(self.string.bytes());
        v.push(0x00);
        while v.len() % 4 != 0 {
            v.push(0x00);
        }

        Ok(v)
    }
}

impl DBusObjectPath {
    fn marshall_be(&self) -> crate::Result<Vec<u8>> {
        // Marshalls the same way as DBusString.
        self.dbus_string.marshall_be()
    }
}

impl DBusSignature {
    fn marshall_be(&self) -> crate::Result<Vec<u8>> {
        // Marshalls the same way as DBusString.
        self.dbus_string.marshall_be()
    }
}

impl DBusUnixFileDescriptor {
    fn marshall_be(&self) -> [u8; 4] {
        todo!()
    }
}

impl DBusVariant {
    fn marshall_be(&self) -> Vec<u8> {
        todo!("signature, and then inner value");
    }
}

impl DBusArray {
    fn marshall_be(&self) -> Vec<u8> {
        // A UINT32 giving the length of the array data in bytes, followed by alignment
        // padding to the alignment boundary of the array element type, followed by each
        // array element.
        todo!()
    }
}

impl DBusStruct {
    fn marshall_be(&self) -> Vec<u8> {
        todo!()
    }
}

impl DBusMap {
    fn marshall_be(&self) -> Vec<u8> {
        todo!()
    }
}
