mod signature_trait;

pub use signature_trait::Signature;

lazy_static::lazy_static! {
    /// Signature of a header field, which is always STRUCT of (BYTE,VARIANT).
    pub static ref HEADER_FIELD_SIGNATURE: SingleCompleteTypeSignature =
        SingleCompleteTypeSignature::DBusStruct {
            fields: vec![
                SingleCompleteTypeSignature::DBusByte,
                SingleCompleteTypeSignature::DBusVariant,
            ],
        };
}

/// Signature for a "Single Complete Type".
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum SingleCompleteTypeSignature {
    DBusByte,
    DBusBoolean,
    DBusInt16,
    DBusUint16,
    DBusInt32,
    DBusUint32,
    DBusInt64,
    DBusUint64,
    DBusDouble,
    DBusString,
    DBusObjectPath,
    DBusSignature,
    DBusUnixFileDescriptor,
    DBusArray(Box<SingleCompleteTypeSignature>),
    DBusStruct {
        fields: Vec<SingleCompleteTypeSignature>,
    },
    DBusVariant,
    DBusDictEntry {
        /// Key may only be a basic type, not a container type
        key: Box<SingleCompleteTypeSignature>,

        value: Box<SingleCompleteTypeSignature>,
    },
}

impl SingleCompleteTypeSignature {
    pub fn is_basic_type(&self) -> bool {
        match self {
            Self::DBusByte => true,
            Self::DBusBoolean => true,
            Self::DBusInt16 => true,
            Self::DBusUint16 => true,
            Self::DBusInt32 => true,
            Self::DBusUint32 => true,
            Self::DBusInt64 => true,
            Self::DBusUint64 => true,
            Self::DBusDouble => true,
            Self::DBusString => true,
            Self::DBusObjectPath => true,
            Self::DBusSignature => true,
            Self::DBusUnixFileDescriptor => true,
            Self::DBusArray(_) => false,
            Self::DBusStruct { fields: _ } => false,
            Self::DBusVariant => false,
            Self::DBusDictEntry { key: _, value: _ } => false,
        }
    }

    /// Global boundary.
    ///
    /// For example, 4 byte values are aligned to a 4-byte boundary, calculated globally.
    pub fn marshalling_boundary(&self) -> usize {
        match self {
            Self::DBusByte => 1,
            Self::DBusBoolean => 4,
            Self::DBusInt16 => 2,
            Self::DBusUint16 => 2,
            Self::DBusInt32 => 4,
            Self::DBusUint32 => 4,
            Self::DBusInt64 => 8,
            Self::DBusUint64 => 8,
            Self::DBusDouble => 8,
            Self::DBusString => 4,
            Self::DBusObjectPath => 4,
            Self::DBusSignature => 1,
            Self::DBusUnixFileDescriptor => 4,
            Self::DBusArray(_) => 4,
            Self::DBusStruct { fields: _ } => 8,
            Self::DBusVariant => 1,
            Self::DBusDictEntry { key: _, value: _ } => 8,
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
            Self::DBusByte => {
                vec![b'y']
            }
            Self::DBusBoolean => {
                vec![b'b']
            }
            Self::DBusInt16 => {
                vec![b'n']
            }
            Self::DBusUint16 => {
                vec![b'q']
            }
            Self::DBusInt32 => {
                vec![b'i']
            }
            Self::DBusUint32 => {
                vec![b'u']
            }
            Self::DBusInt64 => {
                vec![b'x']
            }
            Self::DBusUint64 => {
                vec![b't']
            }
            Self::DBusDouble => {
                vec![b'd']
            }
            Self::DBusString => {
                vec![b's']
            }
            Self::DBusObjectPath => {
                vec![b'o']
            }
            Self::DBusSignature => {
                vec![b'g']
            }
            Self::DBusUnixFileDescriptor => {
                vec![b'h']
            }
            Self::DBusArray(inner) => {
                let mut v = vec![b'a'];
                v.extend(inner.serialize());
                v
            }
            Self::DBusStruct { fields } => {
                let mut v = Vec::new();
                v.push(b'(');
                for field in fields {
                    v.extend(field.serialize());
                }
                v.push(b')');
                v
            }
            Self::DBusVariant => {
                vec![b'v']
            }
            Self::DBusDictEntry { key, value } => {
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
