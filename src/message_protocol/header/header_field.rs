use crate::type_system::types::*;
use crate::type_system::*;

pub enum HeaderField {
    Path(Path),
    Interface(Interface),
    Member(Member),
    ErrorName(ErrorName),
    ReplySerial(ReplySerial),
    Destination(Destination),
    Sender(Sender),
    Signature(Signature),
    UnixFds(UnixFds),
}

pub struct Path {
    pub dbus_object_path: DBusObjectPath,
}

pub struct Interface {
    pub dbus_string: DBusString,
}

pub struct Member {
    pub dbus_string: DBusString,
}

pub struct ErrorName {
    pub dbus_string: DBusString,
}

pub struct ReplySerial {
    pub dbus_uint32: DBusUint32,
}

pub struct Destination {
    pub dbus_string: DBusString,
}

pub struct Sender {
    pub dbus_string: DBusString,
}

pub struct Signature {
    pub dbus_signature: DBusSignature,
}

pub struct UnixFds {
    pub dbus_uint32: DBusUint32,
}

impl HeaderField {
    pub fn decimal_code(&self) -> u8 {
        match self {
            Self::Path(_) => 1,
            Self::Interface(_) => 2,
            Self::Member(_) => 3,
            Self::ErrorName(_) => 4,
            Self::ReplySerial(_) => 5,
            Self::Destination(_) => 6,
            Self::Sender(_) => 7,
            Self::Signature(_) => 8,
            Self::UnixFds(_) => 9,
        }
    }
}
