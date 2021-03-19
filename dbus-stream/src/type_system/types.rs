use super::signature::*;

#[derive(Debug, Clone)]
pub enum Type {
    Basic(BasicType),
    Container(ContainerType),
}

#[derive(Debug, Clone)]
pub enum BasicType {
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
}

#[derive(Debug, Clone)]
pub enum ContainerType {
    Array(DBusArray),
    Struct(DBusStruct),
    Variant(DBusVariant),
    DictEntry(DBusDictEntry),
}

#[derive(Debug, Clone)]
pub struct DBusDictEntry {
    /// Key must be a basic type, not a container type.
    pub key: SingleCompleteTypeSignature,

    pub value: Box<Type>,
}

#[derive(Debug, Clone)]
pub struct DBusArray {
    pub item_type: SingleCompleteTypeSignature,
    pub items: Vec<Type>,
}

#[derive(Debug, Clone)]
pub struct DBusStruct {
    pub fields: Vec<Type>,
}

#[derive(Debug, Clone)]
pub struct DBusVariant {
    pub variant: Box<Type>,
}

#[derive(Debug, Clone)]
pub struct DBusByte {
    pub u8: u8,
}

#[derive(Debug, Clone)]
pub struct DBusBoolean {
    pub bool: bool,
}

#[derive(Debug, Clone)]
pub struct DBusInt16 {
    pub i16: i16,
}

#[derive(Debug, Clone)]
pub struct DBusUint16 {
    pub u16: u16,
}

#[derive(Debug, Clone)]
pub struct DBusInt32 {
    pub i32: i32,
}

#[derive(Debug, Clone)]
pub struct DBusUint32 {
    pub u32: u32,
}

#[derive(Debug, Clone)]
pub struct DBusInt64 {
    pub i64: i64,
}

#[derive(Debug, Clone)]
pub struct DBusUint64 {
    pub u64: u64,
}

#[derive(Debug, Clone)]
pub struct DBusDouble {
    pub f64: f64,
}

#[derive(Debug, Clone)]
pub struct DBusString {
    pub string: String,
}

#[derive(Debug, Clone)]
pub struct DBusObjectPath {
    pub dbus_string: DBusString,
}

#[derive(Debug, Clone)]
pub struct DBusSignature {
    pub vec: Vec<SingleCompleteTypeSignature>,
}

#[derive(Debug, Clone)]
pub struct DBusUnixFileDescriptor {
    // Todo
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

macro_rules! impl_from_basictype {
    ($dbustype:ident, $variant:expr) => {
        impl From<$dbustype> for BasicType {
            fn from(dbustype: $dbustype) -> BasicType {
                $variant(dbustype)
            }
        }
        impl From<$dbustype> for Type {
            fn from(dbustype: $dbustype) -> Type {
                Type::Basic($variant(dbustype))
            }
        }
    };
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

impl_from_basictype!(DBusByte, BasicType::Byte);
impl_from_basictype!(DBusBoolean, BasicType::Boolean);
impl_from_basictype!(DBusInt16, BasicType::Int16);
impl_from_basictype!(DBusUint16, BasicType::Uint16);
impl_from_basictype!(DBusInt32, BasicType::Int32);
impl_from_basictype!(DBusUint32, BasicType::Uint32);
impl_from_basictype!(DBusInt64, BasicType::Int64);
impl_from_basictype!(DBusUint64, BasicType::Uint64);
impl_from_basictype!(DBusDouble, BasicType::Double);
impl_from_basictype!(DBusString, BasicType::String);
impl_from_basictype!(DBusObjectPath, BasicType::ObjectPath);
impl_from_basictype!(DBusSignature, BasicType::Signature);
impl_from_basictype!(DBusUnixFileDescriptor, BasicType::UnixFileDescriptor);

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
