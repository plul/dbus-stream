use super::signature::*;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Type {
    Basic(BasicType),
    Container(ContainerType),
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ContainerType {
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
        #[derive(Debug, Clone, PartialEq, PartialOrd)]
        pub enum BasicType {
            // Loop over the variants to build the enum:
            $(
                $name($name),
            )*
        }

        // Create the individual wrapper types
        $(
            #[derive(Debug, Clone, PartialEq, PartialOrd)]
            pub struct $name {
                pub $field_name: $inner_type
            }

            /// Wrap it in [BasicType].
            impl From<$name> for BasicType {
                fn from(x: $name) -> BasicType {
                    BasicType::$name(x)
                }
            }

            /// Wrap it in [Type].
            impl From<$name> for Type {
                fn from(x: $name) -> Type {
                    Type::Basic(BasicType::from(x))
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

impl DBusString {
    pub fn new<T>(t: T) -> crate::Result<Self>
    where
        T: Into<String>,
    {
        // TODO: As soon as there are more stringent checking done for this type, this may need to change.
        let s = Self { string: t.into() };
        Ok(s)
    }
}

impl DBusObjectPath {
    pub fn new<T>(t: T) -> crate::Result<Self>
    where
        T: Into<String>,
    {
        // TODO: As soon as there are more stringent checking done for this type, this may need to change.
        let s = Self {
            dbus_string: DBusString::new(t)?,
        };
        Ok(s)
    }
}

impl From<BasicType> for Type {
    fn from(basic_type: BasicType) -> Type {
        Type::Basic(basic_type)
    }
}

impl From<ContainerType> for Type {
    fn from(container_type: ContainerType) -> Type {
        Type::Container(container_type)
    }
}

macro_rules! impl_from_containertype {
    ($dbustype:ident, $variant:expr) => {
        impl From<$dbustype> for ContainerType {
            fn from(dbustype: $dbustype) -> ContainerType {
                $variant(dbustype)
            }
        }
        impl From<$dbustype> for Type {
            fn from(dbustype: $dbustype) -> Type {
                Type::Container($variant(dbustype))
            }
        }
    };
}

impl_from_containertype!(DBusArray, ContainerType::Array);
impl_from_containertype!(DBusStruct, ContainerType::Struct);
impl_from_containertype!(DBusVariant, ContainerType::Variant);
impl_from_containertype!(DBusDictEntry, ContainerType::DictEntry);

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

impl DBusByte {
    pub fn new(u8: u8) -> Self {
        Self { u8 }
    }
}

impl DBusArray {
    pub fn new(item_type: SingleCompleteTypeSignature) -> Self {
        Self {
            item_type,
            items: Vec::new(),
        }
    }

    pub fn push<T: Into<Type>>(&mut self, item: T) {
        self.items.push(item.into());
    }
}
