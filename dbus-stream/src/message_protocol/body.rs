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
}
