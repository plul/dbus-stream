/// Marker for a "Single Complete Type".
#[derive(Debug)]
pub enum TypeMark {
    Basic(BasicTypeMark),
    Container(Box<ContainerTypeMark>),
}

#[derive(Debug)]
pub enum ContainerTypeMark {
    Array(TypeMark),
    Struct { fields: Vec<TypeMark> },
    Variant(TypeMark),
    Map { key: BasicTypeMark, value: TypeMark },
}

#[derive(Debug)]
pub enum BasicTypeMark {
    Byte,
    Boolean,
    Int16,
    Uint16,
    Int32,
    Uint32,
    Int64,
    Uint64,
    Double,
    String,
    ObjectPath,
    Signature,
    UnixFileDescriptor,
}

impl TypeMark {
    pub fn new_from_signature(signature: &str) -> Self {
        todo!()
    }
}
