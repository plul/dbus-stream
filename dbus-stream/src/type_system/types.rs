use std::collections::HashMap;

use super::signature::*;

#[derive(Debug)]
pub enum Type {
    Basic(BasicType),
    Container(ContainerType),
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum ContainerType {
    Array(DBusArray),
    Struct(DBusStruct),
    Variant(DBusVariant),
    Map(DBusMap),
}

#[derive(Debug)]
pub struct DBusArray {
    pub item_type: SingleCompleteTypeSignature,
    pub items: Vec<Type>,
}

#[derive(Debug)]
pub struct DBusStruct {
    pub fields: Vec<(SingleCompleteTypeSignature, Type)>,
}

#[derive(Debug)]
pub struct DBusVariant {
    pub variant: Box<Type>,
}

/// Map (Array of Dict Entries)
#[derive(Debug)]
pub struct DBusMap {
    /// Key must be a basic type, not a container type.
    pub key_type: SingleCompleteTypeSignature,

    pub value_type: SingleCompleteTypeSignature,
    pub map: HashMap<BasicType, Type>,
}

#[derive(Debug)]
pub struct DBusByte {
    pub u8: u8,
}

#[derive(Debug)]
pub struct DBusBoolean {
    pub bool: bool,
}

#[derive(Debug)]
pub struct DBusInt16 {
    pub i16: i16,
}

#[derive(Debug)]
pub struct DBusUint16 {
    pub u16: u16,
}

#[derive(Debug)]
pub struct DBusInt32 {
    pub i32: i32,
}

#[derive(Debug)]
pub struct DBusUint32 {
    pub u32: u32,
}

#[derive(Debug)]
pub struct DBusInt64 {
    pub i64: i64,
}

#[derive(Debug)]
pub struct DBusUint64 {
    pub u64: u64,
}

#[derive(Debug)]
pub struct DBusDouble {
    pub f64: f64,
}

#[derive(Debug)]
pub struct DBusString {
    pub string: String,
}

#[derive(Debug)]
pub struct DBusObjectPath {
    pub dbus_string: DBusString,
}

#[derive(Debug)]
pub struct DBusSignature {
    /// The bytes containing the signature as ASCII.
    pub vec: Vec<u8>,
}

#[derive(Debug)]
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
