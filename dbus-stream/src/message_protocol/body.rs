use crate::type_system::signature::SingleCompleteTypeSignature;
use crate::type_system::types::*;

#[derive(Debug, Default)]
pub struct Body {
    pub arguments: Vec<Type>,
}

impl Body {
    /// The body is made up of zero or more [single complete types](crate::type_system::signature::SingleCompleteTypeSignature).
    pub fn signature(&self) -> DBusSignature {
        let vec: Vec<SingleCompleteTypeSignature> =
            self.arguments.iter().map(|arg| arg.signature()).collect();

        DBusSignature { vec }
    }
}
