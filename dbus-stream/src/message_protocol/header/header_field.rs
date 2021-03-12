use crate::type_system::signature::Signature;
use crate::type_system::signature::SingleCompleteTypeSignature;
use crate::type_system::types::*;

pub enum HeaderField {
    Path(DBusObjectPath),
    Interface(DBusString),
    Member(DBusString),
    ErrorName(DBusString),
    ReplySerial(DBusUint32),
    Destination(DBusString),
    Sender(DBusString),
    Signature(Signature),
    UnixFds(DBusUint32),
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
