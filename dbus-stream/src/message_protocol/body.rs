use crate::type_system::signature::Signature;
use crate::type_system::types::*;

pub struct Body {
    pub arguments: Vec<Type>,
}

impl Body {
    pub fn marshall_be(&self) -> crate::Result<Vec<u8>> {
        let mut vec: Vec<u8> = Vec::new();
        for arg in &self.arguments {
            vec.extend(arg.marshall_be()?);
        }
        Ok(vec)
    }

    /// The body is made up of zero or more [single complete types](crate::type_system::signature::SingleCompleteTypeSignature).
    pub fn signature(&self) -> Signature {
        self.arguments.iter().map(|arg| arg.signature()).collect()
    }
}
