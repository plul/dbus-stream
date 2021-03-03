use crate::type_system::types::*;

pub struct Body {
    pub arguments: Vec<Type>,
}

impl Body {
    pub fn marshall_be(&self) -> Vec<u8> {
        todo!()
    }
}