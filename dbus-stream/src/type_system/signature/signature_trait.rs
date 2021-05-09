//! [Signature] trait to get a [SingleCompleteTypeSignature] for a type.

use super::SingleCompleteTypeSignature;
use crate::type_system::types::*;
use crate::type_system::Type;

pub trait Signature {
    fn signature(&self) -> SingleCompleteTypeSignature;
}

impl Signature for Type {
    /// Return signature for this type.
    fn signature(&self) -> SingleCompleteTypeSignature {
        match self {
            Type::Byte(inner) => inner.signature(),
            Type::Boolean(inner) => inner.signature(),
            Type::Int16(inner) => inner.signature(),
            Type::Uint16(inner) => inner.signature(),
            Type::Int32(inner) => inner.signature(),
            Type::Uint32(inner) => inner.signature(),
            Type::Int64(inner) => inner.signature(),
            Type::Uint64(inner) => inner.signature(),
            Type::Double(inner) => inner.signature(),
            Type::String(inner) => inner.signature(),
            Type::ObjectPath(inner) => inner.signature(),
            Type::Signature(inner) => inner.signature(),
            Type::UnixFileDescriptor(inner) => inner.signature(),
            Type::Array(inner) => inner.signature(),
            Type::Struct(inner) => inner.signature(),
            Type::Variant(inner) => inner.signature(),
            Type::DictEntry(inner) => inner.signature(),
        }
    }
}


/// Implement [Signature] for simple types.
macro_rules! impl_to_signature {
    ($name:ident) => {
        impl Signature for $name {
            fn signature(&self) -> SingleCompleteTypeSignature {
                SingleCompleteTypeSignature::$name
            }
        }
    };
}

impl_to_signature!(DBusByte);
impl_to_signature!(DBusBoolean);
impl_to_signature!(DBusInt16);
impl_to_signature!(DBusUint16);
impl_to_signature!(DBusInt32);
impl_to_signature!(DBusUint32);
impl_to_signature!(DBusInt64);
impl_to_signature!(DBusUint64);
impl_to_signature!(DBusDouble);
impl_to_signature!(DBusString);
impl_to_signature!(DBusObjectPath);
impl_to_signature!(DBusSignature);
impl_to_signature!(DBusUnixFileDescriptor);
impl_to_signature!(DBusVariant);

impl Signature for DBusArray {
    fn signature(&self) -> SingleCompleteTypeSignature {
        SingleCompleteTypeSignature::DBusArray(Box::new(self.item_type.clone()))
    }
}

impl Signature for DBusStruct {
    fn signature(&self) -> SingleCompleteTypeSignature {
        SingleCompleteTypeSignature::DBusStruct {
            fields: self.fields.iter().map(|field| field.signature()).collect(),
        }
    }
}

impl Signature for DBusDictEntry {
    fn signature(&self) -> SingleCompleteTypeSignature {
        SingleCompleteTypeSignature::DBusDictEntry {
            key: Box::new(self.key.clone()),
            value: Box::new(self.value.signature()),
        }
    }
}
