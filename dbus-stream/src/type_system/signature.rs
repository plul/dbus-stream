/// Signature for a "Single Complete Type".
#[derive(Debug)]
pub enum Signature {
    Basic(BasicSignature),
    Container(Box<ContainerSignature>),
}

#[derive(Debug)]
pub enum ContainerSignature {
    Array(Signature),
    Struct {
        fields: Vec<Signature>,
    },
    Variant(Signature),
    Map {
        key: BasicSignature,
        value: Signature,
    },
}

#[derive(Debug)]
pub enum BasicSignature {
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

impl Signature {
    pub fn code(&self) -> u8 {
        match self {
            Self::Basic(basic_signature) => basic_signature.code(),
            Self::Container(container_signature) => container_signature.code(),
        }
    }
}

impl BasicSignature {
    pub fn code(&self) -> u8 {
        match self {
            Self::Byte => b'y',
            Self::Boolean => b'b',
            Self::Int16 => b'n',
            Self::Uint16 => b'q',
            Self::Int32 => b'i',
            Self::Uint32 => b'u',
            Self::Int64 => b'x',
            Self::Uint64 => b't',
            Self::Double => b'd',
            Self::String => b's',
            Self::ObjectPath => b'o',
            Self::Signature => b'g',
            Self::UnixFileDescriptor => b'h',
        }
    }
}

impl ContainerSignature {
    pub fn code(&self) -> u8 {
        match self {
            Self::Array(signature) => todo!(),
            Self::Struct { fields } => todo!(),
            Self::Variant(signature) => b'v',
            Self::Map { key, value } => todo!(),
        }
    }
}
