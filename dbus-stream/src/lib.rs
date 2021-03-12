//! DBus Specification:
//! https://dbus.freedesktop.org/doc/dbus-specification.html#message-protocol-marshaling

pub(crate) mod connection;
pub(crate) mod message_protocol;
pub(crate) mod type_system;

pub use connection::Connection;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    TryFromIntError(#[from] std::num::TryFromIntError),

    #[error(transparent)]
    TryFromSliceError(#[from] std::array::TryFromSliceError),

    #[error(transparent)]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    #[error("Parse error")]
    ParseError,

    #[error("Failed AUTH")]
    FailedAuth,
}

pub type Result<T> = std::result::Result<T, Error>;

pub const MAJOR_PROTOCOL_VERSION: u8 = 1;

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         assert_eq!(2 + 2, 4);
//     }
// }
