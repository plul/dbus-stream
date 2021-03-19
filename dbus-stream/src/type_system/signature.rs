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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
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
    DictEntry {
        /// Key may only be a basic type, not a container type
        key: Box<SingleCompleteTypeSignature>,

        value: Box<SingleCompleteTypeSignature>,
    },
}

impl SingleCompleteTypeSignature {
    pub fn is_basic_type(&self) -> bool {
        match self {
            Self::Byte => true,
            Self::Boolean => true,
            Self::Int16 => true,
            Self::Uint16 => true,
            Self::Int32 => true,
            Self::Uint32 => true,
            Self::Int64 => true,
            Self::Uint64 => true,
            Self::Double => true,
            Self::String => true,
            Self::ObjectPath => true,
            Self::Signature => true,
            Self::UnixFileDescriptor => true,
            Self::Array(_) => false,
            Self::Struct { fields: _ } => false,
            Self::Variant => false,
            Self::DictEntry { key: _, value: _ } => false,
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
            Self::Variant => 1,
            Self::DictEntry { key: _, value: _ } => 8,
        }
    }

    /// Return the signature as an ASCII string.
    ///
    /// For marshalling and transmitting on the wire, LV encoding must be taken into
    /// account. This method merely returns the ASCII such as "ai" for an array of INT32.
    ///
    /// TODO: enforce max depths
    pub fn serialize(&self) -> Vec<u8> {
        match self {
            Self::Byte => {
                vec![b'y']
            }
            Self::Boolean => {
                vec![b'b']
            }
            Self::Int16 => {
                vec![b'n']
            }
            Self::Uint16 => {
                vec![b'q']
            }
            Self::Int32 => {
                vec![b'i']
            }
            Self::Uint32 => {
                vec![b'u']
            }
            Self::Int64 => {
                vec![b'x']
            }
            Self::Uint64 => {
                vec![b't']
            }
            Self::Double => {
                vec![b'd']
            }
            Self::String => {
                vec![b's']
            }
            Self::ObjectPath => {
                vec![b'o']
            }
            Self::Signature => {
                vec![b'g']
            }
            Self::UnixFileDescriptor => {
                vec![b'h']
            }
            Self::Array(inner) => {
                let mut v = vec![b'a'];
                v.extend(inner.serialize());
                v
            }
            Self::Struct { fields } => {
                let mut v = Vec::new();
                v.push(b'(');
                for field in fields {
                    v.extend(field.serialize());
                }
                v.push(b')');
                v
            }
            Self::Variant => {
                vec![b'v']
            }
            Self::DictEntry { key, value } => {
                // Assert that key is basic type.
                // Maybe this should be more than a debug assertion, not sure.
                debug_assert!(key.is_basic_type());

                let mut v = Vec::new();
                v.push(b'a');
                v.push(b'{');
                v.extend(key.serialize());
                v.extend(value.serialize());
                v.push(b'}');
                v
            }
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
            BasicType::DBusByte(_) => SingleCompleteTypeSignature::Byte,
            BasicType::DBusBoolean(_) => SingleCompleteTypeSignature::Boolean,
            BasicType::DBusInt16(_) => SingleCompleteTypeSignature::Int16,
            BasicType::DBusUint16(_) => SingleCompleteTypeSignature::Uint16,
            BasicType::DBusInt32(_) => SingleCompleteTypeSignature::Int32,
            BasicType::DBusUint32(_) => SingleCompleteTypeSignature::Uint32,
            BasicType::DBusInt64(_) => SingleCompleteTypeSignature::Int64,
            BasicType::DBusUint64(_) => SingleCompleteTypeSignature::Uint64,
            BasicType::DBusDouble(_) => SingleCompleteTypeSignature::Double,
            BasicType::DBusString(_) => SingleCompleteTypeSignature::String,
            BasicType::DBusObjectPath(_) => SingleCompleteTypeSignature::ObjectPath,
            BasicType::DBusSignature(_) => SingleCompleteTypeSignature::Signature,
            BasicType::DBusUnixFileDescriptor(_) => SingleCompleteTypeSignature::UnixFileDescriptor,
        }
    }
}

impl ToSignature for ContainerType {
    /// Return signature for this container type.
    fn signature(&self) -> SingleCompleteTypeSignature {
        match self {
            ContainerType::Array(dbus_array) => {
                SingleCompleteTypeSignature::Array(Box::new(dbus_array.item_type.clone()))
            }
            ContainerType::Struct(dbus_struct) => SingleCompleteTypeSignature::Struct {
                fields: dbus_struct
                    .fields
                    .iter()
                    .map(|field_type| field_type.signature())
                    .collect(),
            },
            ContainerType::Variant(_dbus_variant) => SingleCompleteTypeSignature::Variant,
            ContainerType::DictEntry(dbus_dict_entry) => SingleCompleteTypeSignature::DictEntry {
                key: Box::new(dbus_dict_entry.key.clone()),
                value: Box::new(dbus_dict_entry.value.signature()),
            },
        }
    }
}

// TODO replace all these impls with a macro

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

impl ToSignature for DBusDictEntry {
    fn signature(&self) -> SingleCompleteTypeSignature {
        SingleCompleteTypeSignature::DictEntry {
            key: Box::new(self.key.clone()),
            value: Box::new(self.value.signature()),
        }
    }
}
