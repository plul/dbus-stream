use super::types::*;
use super::BasicType;
use super::ContainerType;
use super::Type;

lazy_static::lazy_static! {
    /// Signature of a header field, which is always STRUCT of (BYTE,VARIANT).
    pub static ref HEADER_FIELD_SIGNATURE: SingleCompleteTypeSignature =
        SingleCompleteTypeSignature::Struct {
            fields: vec![
                SingleCompleteTypeSignature::Byte,
                SingleCompleteTypeSignature::Variant,
            ],
        };
}

trait ToSignature {
    fn signature(&self) -> SingleCompleteTypeSignature;
}

/// Signature for a "Single Complete Type".
#[derive(Debug, Clone)]
pub enum SingleCompleteTypeSignature {
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
    Array(Box<SingleCompleteTypeSignature>),
    Struct {
        fields: Vec<SingleCompleteTypeSignature>,
    },
    Variant,

    /// Map (Array of Dict Entries)
    Map {
        /// Key may only be a basic type, not a container type
        key: Box<SingleCompleteTypeSignature>,

        value: Box<SingleCompleteTypeSignature>,
    },
}

impl SingleCompleteTypeSignature {
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
            Self::Variant => 1,
            Self::Map { key: _, value: _ } => 8,
        }
    }
}

impl Type {
    /// Return signature for this type.
    pub fn signature(&self) -> SingleCompleteTypeSignature {
        match self {
            Type::Basic(inner) => SingleCompleteTypeSignature::from(inner.signature()),
            Type::Container(inner) => SingleCompleteTypeSignature::from(inner.signature()),
        }
    }
}

impl ToSignature for BasicType {
    /// Return signature for this basic type.
    fn signature(&self) -> SingleCompleteTypeSignature {
        match self {
            BasicType::Byte(_) => SingleCompleteTypeSignature::Byte,
            BasicType::Boolean(_) => SingleCompleteTypeSignature::Boolean,
            BasicType::Int16(_) => SingleCompleteTypeSignature::Int16,
            BasicType::Uint16(_) => SingleCompleteTypeSignature::Uint16,
            BasicType::Int32(_) => SingleCompleteTypeSignature::Int32,
            BasicType::Uint32(_) => SingleCompleteTypeSignature::Uint32,
            BasicType::Int64(_) => SingleCompleteTypeSignature::Int64,
            BasicType::Uint64(_) => SingleCompleteTypeSignature::Uint64,
            BasicType::Double(_) => SingleCompleteTypeSignature::Double,
            BasicType::String(_) => SingleCompleteTypeSignature::String,
            BasicType::ObjectPath(_) => SingleCompleteTypeSignature::ObjectPath,
            BasicType::Signature(_) => SingleCompleteTypeSignature::Signature,
            BasicType::UnixFileDescriptor(_) => SingleCompleteTypeSignature::UnixFileDescriptor,
        }
    }
}

impl ToSignature for ContainerType {
    /// Return signature for this container type.
    fn signature(&self) -> SingleCompleteTypeSignature {
        match self {
            ContainerType::Array(dbus_array) => todo!(),
            ContainerType::Struct(dbus_struct) => todo!(),
            ContainerType::Variant(dbus_variant) => todo!(),
            ContainerType::Map(dbus_map) => todo!(),
        }
    }
}

impl ToSignature for DBusByte {
    fn signature(&self) -> SingleCompleteTypeSignature {
        SingleCompleteTypeSignature::Byte
    }
}

impl ToSignature for DBusBoolean {
    fn signature(&self) -> SingleCompleteTypeSignature {
        SingleCompleteTypeSignature::Boolean
    }
}

impl ToSignature for DBusInt16 {
    fn signature(&self) -> SingleCompleteTypeSignature {
        SingleCompleteTypeSignature::Int16
    }
}

impl ToSignature for DBusUint16 {
    fn signature(&self) -> SingleCompleteTypeSignature {
        SingleCompleteTypeSignature::Uint16
    }
}

impl ToSignature for DBusInt32 {
    fn signature(&self) -> SingleCompleteTypeSignature {
        SingleCompleteTypeSignature::Int32
    }
}

impl ToSignature for DBusUint32 {
    fn signature(&self) -> SingleCompleteTypeSignature {
        SingleCompleteTypeSignature::Uint32
    }
}

impl ToSignature for DBusInt64 {
    fn signature(&self) -> SingleCompleteTypeSignature {
        SingleCompleteTypeSignature::Int64
    }
}

impl ToSignature for DBusUint64 {
    fn signature(&self) -> SingleCompleteTypeSignature {
        SingleCompleteTypeSignature::Uint64
    }
}

impl ToSignature for DBusDouble {
    fn signature(&self) -> SingleCompleteTypeSignature {
        SingleCompleteTypeSignature::Double
    }
}

impl ToSignature for DBusString {
    fn signature(&self) -> SingleCompleteTypeSignature {
        SingleCompleteTypeSignature::String
    }
}

impl ToSignature for DBusObjectPath {
    fn signature(&self) -> SingleCompleteTypeSignature {
        SingleCompleteTypeSignature::ObjectPath
    }
}

impl ToSignature for DBusSignature {
    fn signature(&self) -> SingleCompleteTypeSignature {
        SingleCompleteTypeSignature::Signature
    }
}

impl ToSignature for DBusUnixFileDescriptor {
    fn signature(&self) -> SingleCompleteTypeSignature {
        SingleCompleteTypeSignature::UnixFileDescriptor
    }
}

impl ToSignature for DBusArray {
    fn signature(&self) -> SingleCompleteTypeSignature {
        SingleCompleteTypeSignature::Array(Box::new(self.item_type.clone()))
    }
}

impl ToSignature for DBusStruct {
    fn signature(&self) -> SingleCompleteTypeSignature {
        SingleCompleteTypeSignature::Struct {
            fields: self.fields.iter().map(|field| field.signature()).collect(),
        }
    }
}

impl ToSignature for DBusVariant {
    fn signature(&self) -> SingleCompleteTypeSignature {
        SingleCompleteTypeSignature::Variant
    }
}

impl ToSignature for DBusMap {
    fn signature(&self) -> SingleCompleteTypeSignature {
        SingleCompleteTypeSignature::Map {
            key: Box::new(self.key_type.clone()),
            value: Box::new(self.value_type.clone()),
        }
    }
}
