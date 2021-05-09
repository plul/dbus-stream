pub mod body;

use std::convert::TryFrom;
use std::num::NonZeroU32;

use self::body::Body;
use crate::type_system::marshal::Marshal;
use crate::type_system::marshal::Encoder;
use crate::type_system::signature::HEADER_FIELD_SIGNATURE;
use crate::type_system::types::*;
use crate::type_system::Endianness;

#[derive(Debug, Clone)]
enum HeaderField {
    Path(DBusObjectPath),
    Interface(DBusString),
    Member(DBusString),
    ErrorName(DBusString),
    ReplySerial(DBusUint32),
    Destination(DBusString),
    Sender(DBusString),
    Signature(DBusSignature),
    UnixFds(DBusUint32),
}

#[derive(Debug)]
pub struct Message {
    pub flag_no_reply_expected: bool,
    pub flag_no_auto_start: bool,
    pub flag_allow_interactive_authorization: bool,

    /// The serial of this message, used as a cookie by the sender to identify the reply corresponding to this request.
    pub serial: NonZeroU32,

    pub message_type_param: MessageTypeParam,

    /// The name of the connection this message is intended for.
    pub destination: Option<DBusString>,

    /// Body
    pub body: Body,
}

#[derive(Debug)]
pub enum MessageType {
    MethodCall,
    MethodReturn,
    Error,
    Signal,
}

#[derive(Debug)]
pub enum MessageTypeParam {
    MethodCall(MethodCall),
    MethodReturn,
    Error,
    Signal(Signal),
}

#[derive(Debug)]
pub struct MethodCall {
    /// The object to send a call to.
    pub path: DBusObjectPath,

    /// The interface to invoke a method call on.
    pub interface: Option<DBusString>,

    /// The member name (the name of the method).
    pub member: DBusString,
}

#[derive(Debug)]
pub struct Signal {
    /// The object to send a call to.
    pub path: DBusObjectPath,

    /// The interface to invoke a method call on.
    pub interface: DBusString,

    /// The member name (the name of the method).
    pub member: DBusString,
}

/// Prepare header fields to be marshalled.
///
/// Header fields are an Array of Struct(Byte, Variant).
fn prepare_header_fields<T: IntoIterator<Item = HeaderField>>(header_fields: T) -> DBusArray {
    let mut array = DBusArray::new(HEADER_FIELD_SIGNATURE.clone());

    for header_field in header_fields {
        let byte: DBusByte = DBusByte::from(header_field.decimal_code());
        let variant: DBusVariant = header_field.inner_into_variant();
        let header_field_struct = DBusStruct::new(vec![byte.into(), variant.into()]);

        array.items.push(Type::from(header_field_struct));
    }

    array
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

impl Message {
    pub fn marshal_be(&self) -> crate::Result<Vec<u8>> {
        let endianness = Endianness::BigEndian;

        let marshalled_body: Vec<u8> = self
            .body
            .arguments
            .iter()
            .try_fold(Encoder::default(), |mut m, arg| {
                match m.marshal_be(arg) {
                    Ok(()) => Ok(m),
                    Err(err) => Err(err),
                }
            })?
            .finish();

        let mut header: Vec<u8> = Vec::new();

        // 1st byte: Endianness
        header.push(endianness.ascii_code());

        // 2nd byte: Message Type
        header.push(self.message_type_param.message_type().decimal_value());

        // 3rd byte: Bitwise OR flags
        let mut flags = 0;
        if self.flag_no_reply_expected {
            flags |= 0x1;
        }
        if self.flag_no_auto_start {
            flags |= 0x2;
        }
        if self.flag_allow_interactive_authorization {
            flags |= 0x4;
        }
        header.push(flags);

        // 4th byte: Major protocol version
        header.push(crate::MAJOR_PROTOCOL_VERSION);

        // 5th to 8th byte: Length in bytes of message body
        let length_in_bytes_of_message_body = u32::try_from(marshalled_body.len())?;
        header.extend_from_slice(&length_in_bytes_of_message_body.to_be_bytes());

        // 9th to 12th byte: Serial
        header.extend_from_slice(&self.serial.get().to_be_bytes());

        // Header fields.
        //
        // TODO:
        // The way these header fields are first converted to an intermediate layout, requires cloning.
        // To improve performance, either change the intermediary to work with references, or use COW, Rc or something like that.
        let mut header_fields: Vec<HeaderField> = Vec::new();

        // Header field: Signature
        // This _can_ be omitted if the body is empty, but here we  always include it.
        header_fields.push(HeaderField::Signature(self.body.signature()));

        // Header field: Destination (optional).
        if let Some(destination) = &self.destination {
            header_fields.push(HeaderField::Destination(destination.clone()));
        }

        // NOTE:
        // No current handling of header fields: UNIX_FDS, SENDER,

        // Message type specific header fields
        match &self.message_type_param {
            MessageTypeParam::MethodCall(method_call) => {
                // Path and Member are mandatory.
                header_fields.push(HeaderField::Path(method_call.path.clone()));
                header_fields.push(HeaderField::Member(method_call.member.clone()));

                // Interface is optional.
                if let Some(interface) = &method_call.interface {
                    header_fields.push(HeaderField::Interface(interface.clone()));
                }
            }
            _ => todo!("Header fields for other message types"),
        };

        let mut header = Encoder { buf: header };

        // Convert header fields enums to a DBus Array of Struct of (Byte, Variant), and marshal that.
        header.marshal_be(&prepare_header_fields(header_fields))?;

        // Header must be 8-aligned with null bytes
        header.align(8);

        // Finalize marshalled message by appending body.
        let mut message = header.finish();
        message.extend(marshalled_body);
        Ok(message)
    }
}

impl HeaderField {
    fn decimal_code(&self) -> u8 {
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

    fn inner_into_variant(self) -> DBusVariant {
        match self {
            Self::Path(inner) => DBusVariant::new(inner),
            Self::Interface(inner) => DBusVariant::new(inner),
            Self::Member(inner) => DBusVariant::new(inner),
            Self::ErrorName(inner) => DBusVariant::new(inner),
            Self::ReplySerial(inner) => DBusVariant::new(inner),
            Self::Destination(inner) => DBusVariant::new(inner),
            Self::Sender(inner) => DBusVariant::new(inner),
            Self::Signature(inner) => DBusVariant::new(inner),
            Self::UnixFds(inner) => DBusVariant::new(inner),
        }
    }
}

impl MessageTypeParam {
    fn message_type(&self) -> MessageType {
        match self {
            MessageTypeParam::MethodCall(_) => MessageType::MethodCall,
            MessageTypeParam::MethodReturn => MessageType::MethodReturn,
            MessageTypeParam::Error => MessageType::Error,
            MessageTypeParam::Signal(_) => MessageType::Signal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn message_marshalling() -> crate::Result<()> {
        let message = Message {
            flag_no_reply_expected: true,
            flag_no_auto_start: true,
            flag_allow_interactive_authorization: true,
            serial: NonZeroU32::new(1).unwrap(),
            message_type_param: MessageTypeParam::MethodCall(MethodCall {
                path: DBusObjectPath::from("path"),
                interface: Some(DBusString::from("interface")),
                member: DBusString::from("member"),
            }),
            destination: None,
            body: Body::default(),
        };

        let marshalled = message.marshal_be()?;

        // Check first 8 bytes of header:
        // endianness, message type, flags, major protocol version, and finally 4 bytes for body length.
        assert_eq!(&marshalled[0..8], &[b'B', 1, 0x7, 1, 0, 0, 0, 0]);

        Ok(())
    }
}
