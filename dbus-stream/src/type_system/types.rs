use super::signature::*;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Type {
    Byte(DBusByte),
    Boolean(DBusBoolean),
    Int16(DBusInt16),
    Uint16(DBusUint16),
    Int32(DBusInt32),
    Uint32(DBusUint32),
    Int64(DBusInt64),
    Uint64(DBusUint64),
    Double(DBusDouble),
    String(DBusString),
    ObjectPath(DBusObjectPath),
    Signature(DBusSignature),
    UnixFileDescriptor(DBusUnixFileDescriptor),
    Array(DBusArray),
    Struct(DBusStruct),
    Variant(DBusVariant),
    DictEntry(DBusDictEntry),
}

/// Macro to create the primitive (basic) DBus types, that are modelled as just simple wrappers over a native Rust type.
macro_rules! basic_type {
    (
        // Repetition
        $(
            [$name:ident, $field_name:ident, $inner_type:ty]
        )
        // ...separated by commas...
        ,
        // ...zero or more times
        *
    ) => {
        // Create the individual wrapper types
        $(
            #[derive(Debug, Clone, PartialEq, PartialOrd)]
            pub struct $name {
                pub $field_name: $inner_type
            }

            impl<T> From<T> for $name where $inner_type: From<T> {
                fn from(x: T) -> Self {
                    Self { $field_name: <$inner_type>::from(x) }
                }
            }
        )*
    };
}

// Define the basic types.
basic_type!(
    [DBusByte, u8, u8],
    [DBusBoolean, bool, bool],
    [DBusInt16, i16, i16],
    [DBusUint16, u16, u16],
    [DBusInt32, i32, i32],
    [DBusUint32, u32, u32],
    [DBusInt64, i64, i64],
    [DBusUint64, u64, u64],
    [DBusDouble, f64, f64],
    [DBusString, string, String],
    [DBusObjectPath, dbus_string, DBusString],
    [DBusSignature, vec, Vec<SingleCompleteTypeSignature>],
    [DBusUnixFileDescriptor, u32, u32]
);


#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct DBusDictEntry {
    /// Key must be a basic type, not a container type.
    pub key: SingleCompleteTypeSignature,

    pub value: Box<Type>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct DBusArray {
    pub item_type: SingleCompleteTypeSignature,
    pub items: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct DBusStruct {
    pub fields: Vec<Type>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct DBusVariant {
    pub variant: Box<Type>,
}

impl DBusStruct {
    pub fn new<T: Into<Vec<Type>>>(fields: T) -> Self {
        Self {
            fields: fields.into(),
        }
    }
}

impl DBusVariant {
    pub fn new<T: Into<Type>>(variant: T) -> Self {
        Self {
            variant: Box::new(variant.into()),
        }
    }
}

impl DBusArray {
    pub fn new(item_type: SingleCompleteTypeSignature) -> Self {
        Self {
            item_type,
            items: Vec::new(),
        }
    }
}


macro_rules! impl_from_type_variant {
    ($name:ident, $type_variant:ident) => {
        impl From<$name> for Type {
            fn from(x: $name) -> Type {
                Type::$type_variant(x)
            }
        }
    }
}

impl_from_type_variant!(DBusByte, Byte);
impl_from_type_variant!(DBusBoolean, Boolean);
impl_from_type_variant!(DBusInt16, Int16);
impl_from_type_variant!(DBusUint16, Uint16);
impl_from_type_variant!(DBusInt32, Int32);
impl_from_type_variant!(DBusUint32, Uint32);
impl_from_type_variant!(DBusInt64, Int64);
impl_from_type_variant!(DBusUint64, Uint64);
impl_from_type_variant!(DBusDouble, Double);
impl_from_type_variant!(DBusString, String);
impl_from_type_variant!(DBusObjectPath, ObjectPath);
impl_from_type_variant!(DBusSignature, Signature);
impl_from_type_variant!(DBusUnixFileDescriptor, UnixFileDescriptor);
impl_from_type_variant!(DBusArray, Array);
impl_from_type_variant!(DBusStruct, Struct);
impl_from_type_variant!(DBusVariant, Variant);
impl_from_type_variant!(DBusDictEntry, DictEntry);
