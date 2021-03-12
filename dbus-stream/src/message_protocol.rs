pub mod body;
pub mod header;

use self::body::Body;
use crate::type_system::types::*;

pub enum MessageType {
    MethodCall,
    MethodReturn,
    Error,
    Signal,
}

pub struct MethodCall {
    /// The name of the connection this message is intended for.
    pub destination: Option<DBusString>,

    /// The object to send a call to.
    pub path: DBusObjectPath,

    /// The interface to invoke a method call on.
    pub interface: Option<DBusString>,

    /// The member name (the name of the method).
    pub member: DBusString,

    /// Method arguments.
    pub body: Body,
}

impl MessageType {
    pub fn decimal_value(&self) -> u8 {
        match self {
            Self::MethodCall => 1,
            Self::MethodReturn => 2,
            Self::Error => 3,
            Self::Signal => 4,
        }
    }
}
