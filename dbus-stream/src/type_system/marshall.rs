use std::convert::TryFrom;

use crate::type_system::types::*;
use crate::type_system::Endianness;

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

//         while v.len() % 8 > 0 {
//             v.push(0x00);
//         }

//         Ok(v)
//     }
// }

// impl DBusByte {
//     fn marshall_be(&self) -> crate::Result<Vec<u8>> {
//         Ok(vec![self.u8])
//     }
// }

impl DBusBoolean {}

impl DBusInt16 {}

impl DBusUint16 {}

impl DBusInt32 {}

impl DBusUint32 {}

impl DBusInt64 {}

impl DBusUint64 {}

impl DBusDouble {}

impl DBusString {
    fn marshall_be(&self) -> crate::Result<Vec<u8>> {
        let mut v = Vec::new();

        let length = self.string.len();
        let length = u32::try_from(length)?;

        v.extend_from_slice(&length.to_be_bytes());
        v.extend(self.string.bytes());
        v.push(0x00);
        while v.len() % 4 > 0 {
            v.push(0x00);
        }

        Ok(v)
    }
}

impl DBusObjectPath {
    fn marshall_be(&self) -> crate::Result<Vec<u8>> {
        // Encodes the same way as DBusString
        self.dbus_string.marshall_be()
    }
}

impl DBusSignature {
    fn marshall_be(&self) -> crate::Result<Vec<u8>> {
        // Encodes the same way as DBusString
        self.dbus_string.marshall_be()
    }
}

impl DBusUnixFileDescriptor {}

impl DBusVariant {
    fn marshall_be(&self) -> crate::Result<Vec<u8>> {
        todo!("signature, and then inner value");
    }
}

impl DBusArray {
    fn marshall_be(&self) -> crate::Result<Vec<u8>> {
        // A UINT32 giving the length of the array data in bytes, followed by alignment
        // padding to the alignment boundary of the array element type, followed by each
        // array element.
        todo!()
    }
}

impl DBusMap {}
