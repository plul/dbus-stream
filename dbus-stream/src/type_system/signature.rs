use super::types::*;
use super::BasicType;
use super::ContainerType;
use super::Type;

trait ToSignature {
    fn signature(&self) -> Signature;
}

/// Signature for a "Single Complete Type".
#[derive(Debug, Clone)]
pub enum Signature {
    Byte,
    Boolean,
    Int16,
    Uint16,
    Int32,
    Uint32,
    Int64,
    Uint64,
    Double,
    String,
    ObjectPath,
    Signature,
    UnixFileDescriptor,
    Array(Box<Signature>),
    Struct {
        fields: Vec<Signature>,
    },
    Variant(Box<Signature>),

    /// Map (Array of Dict Entries)
    Map {
        /// Key may only be a basic type, not a container type
        key: Box<Signature>,

        value: Box<Signature>,
    },
}

impl Signature {
    pub fn code(&self) -> u8 {
        match self {
            Self::Byte => b'y',
            Self::Boolean => b'b',
            Self::Int16 => b'n',
            Self::Uint16 => b'q',
            Self::Int32 => b'i',
            Self::Uint32 => b'u',
            Self::Int64 => b'x',
            Self::Uint64 => b't',
            Self::Double => b'd',
            Self::String => b's',
            Self::ObjectPath => b'o',
            Self::Signature => b'g',
            Self::UnixFileDescriptor => b'h',
            Self::Array(_) => todo!(),
            Self::Struct { fields: _ } => todo!(),
            Self::Variant(_) => b'v',
            Self::Map { key: _, value: _ } => todo!(),
        }
    }

    /// Global boundary.
    ///
    /// For example, 4 byte values are aligned to a 4-byte boundary, calculated globally.
    pub fn marshalling_boundary(&self) -> usize {
        match self {
            Self::Byte => 1,
            Self::Boolean => 4,
            Self::Int16 => 2,
            Self::Uint16 => 2,
            Self::Int32 => 4,
            Self::Uint32 => 4,
            Self::Int64 => 8,
            Self::Uint64 => 8,
            Self::Double => 8,
            Self::String => 4,
            Self::ObjectPath => 4,
            Self::Signature => 1,
            Self::UnixFileDescriptor => 4,
            Self::Array(_) => 4,
            Self::Struct { fields: _ } => 8,
            Self::Variant(_) => 1,
            Self::Map { key: _, value: _ } => 8,
        }
    }
}

impl Type {
    /// Return signature for this type.
    pub fn signature(&self) -> Signature {
        match self {
            Type::Basic(inner) => Signature::from(inner.signature()),
            Type::Container(inner) => Signature::from(inner.signature()),
        }
    }
}

impl ToSignature for BasicType {
    /// Return signature for this basic type.
    fn signature(&self) -> Signature {
        match self {
            BasicType::Byte(_) => Signature::Byte,
            BasicType::Boolean(_) => Signature::Boolean,
            BasicType::Int16(_) => Signature::Int16,
            BasicType::Uint16(_) => Signature::Uint16,
            BasicType::Int32(_) => Signature::Int32,
            BasicType::Uint32(_) => Signature::Uint32,
            BasicType::Int64(_) => Signature::Int64,
            BasicType::Uint64(_) => Signature::Uint64,
            BasicType::Double(_) => Signature::Double,
            BasicType::String(_) => Signature::String,
            BasicType::ObjectPath(_) => Signature::ObjectPath,
            BasicType::Signature(_) => Signature::Signature,
            BasicType::UnixFileDescriptor(_) => Signature::UnixFileDescriptor,
        }
    }
}

impl ToSignature for ContainerType {
    /// Return signature for this container type.
    fn signature(&self) -> Signature {
        match self {
            ContainerType::Array(dbus_array) => todo!(),
            ContainerType::Struct(dbus_struct) => todo!(),
            ContainerType::Variant(dbus_variant) => todo!(),
            ContainerType::Map(dbus_map) => todo!(),
        }
    }
}

impl ToSignature for DBusByte {
    fn signature(&self) -> Signature {
        Signature::Byte
    }
}

impl ToSignature for DBusBoolean {
    fn signature(&self) -> Signature {
        Signature::Boolean
    }
}

impl ToSignature for DBusInt16 {
    fn signature(&self) -> Signature {
        Signature::Int16
    }
}

impl ToSignature for DBusUint16 {
    fn signature(&self) -> Signature {
        Signature::Uint16
    }
}

impl ToSignature for DBusInt32 {
    fn signature(&self) -> Signature {
        Signature::Int32
    }
}

impl ToSignature for DBusUint32 {
    fn signature(&self) -> Signature {
        Signature::Uint32
    }
}

impl ToSignature for DBusInt64 {
    fn signature(&self) -> Signature {
        Signature::Int64
    }
}

impl ToSignature for DBusUint64 {
    fn signature(&self) -> Signature {
        Signature::Uint64
    }
}

impl ToSignature for DBusDouble {
    fn signature(&self) -> Signature {
        Signature::Double
    }
}

impl ToSignature for DBusString {
    fn signature(&self) -> Signature {
        Signature::String
    }
}

impl ToSignature for DBusObjectPath {
    fn signature(&self) -> Signature {
        Signature::ObjectPath
    }
}

impl ToSignature for DBusSignature {
    fn signature(&self) -> Signature {
        Signature::Signature
    }
}

impl ToSignature for DBusUnixFileDescriptor {
    fn signature(&self) -> Signature {
        Signature::UnixFileDescriptor
    }
}

impl ToSignature for DBusArray {
    fn signature(&self) -> Signature {
        Signature::Array(Box::new(self.item_type.clone()))
    }
}

impl ToSignature for DBusStruct {
    fn signature(&self) -> Signature {
        Signature::Struct {
            fields: self
                .fields
                .iter()
                .map(|(signature, _)| signature.clone())
                .collect(),
        }
    }
}

impl ToSignature for DBusVariant {
    fn signature(&self) -> Signature {
        Signature::Variant(Box::new(self.variant.signature()))
    }
}

impl ToSignature for DBusMap {
    fn signature(&self) -> Signature {
        Signature::Map {
            key: Box::new(self.key_type.clone()),
            value: Box::new(self.value_type.clone()),
        }
    }
}
