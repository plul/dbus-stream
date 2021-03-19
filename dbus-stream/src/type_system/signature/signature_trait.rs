use super::SingleCompleteTypeSignature;
/// [Signature] trait to get a [SingleCompleteTypeSignature] for a type.
use crate::type_system::types::*;
use crate::type_system::BasicType;
use crate::type_system::ContainerType;
use crate::type_system::Type;

pub trait Signature {
    fn signature(&self) -> SingleCompleteTypeSignature;
}

impl Signature for Type {
    /// Return signature for this type.
    fn signature(&self) -> SingleCompleteTypeSignature {
        match self {
            Type::Basic(inner) => inner.signature(),
            Type::Container(inner) => inner.signature(),
        }
    }
}

impl Signature for BasicType {
    /// Return signature for this basic type.
    fn signature(&self) -> SingleCompleteTypeSignature {
        match self {
            BasicType::DBusByte(inner) => inner.signature(),
            BasicType::DBusBoolean(inner) => inner.signature(),
            BasicType::DBusInt16(inner) => inner.signature(),
            BasicType::DBusUint16(inner) => inner.signature(),
            BasicType::DBusInt32(inner) => inner.signature(),
            BasicType::DBusUint32(inner) => inner.signature(),
            BasicType::DBusInt64(inner) => inner.signature(),
            BasicType::DBusUint64(inner) => inner.signature(),
            BasicType::DBusDouble(inner) => inner.signature(),
            BasicType::DBusString(inner) => inner.signature(),
            BasicType::DBusObjectPath(inner) => inner.signature(),
            BasicType::DBusSignature(inner) => inner.signature(),
            BasicType::DBusUnixFileDescriptor(inner) => inner.signature(),
        }
    }
}

impl Signature for ContainerType {
    /// Return signature for this container type.
    fn signature(&self) -> SingleCompleteTypeSignature {
        match self {
            ContainerType::DBusArray(inner) => inner.signature(),
            ContainerType::DBusStruct(inner) => inner.signature(),
            ContainerType::DBusVariant(inner) => inner.signature(),
            ContainerType::DBusDictEntry(inner) => inner.signature(),
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
