use crate::type_system::signature::SingleCompleteTypeSignature;
use crate::type_system::types::*;

#[derive(Debug, Default)]
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
    pub fn signature(&self) -> DBusSignature {
        let vec: Vec<SingleCompleteTypeSignature> =
            self.arguments.iter().map(|arg| arg.signature()).collect();

        DBusSignature { vec }
    }
}
