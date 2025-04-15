use std::fmt;

use crate::common::SymbolPath;

use super::TypeInfo;

pub struct TraitObjectType {
    pub traits: Vec<SymbolPath>    
}

impl fmt::Display for TraitObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "dyn ")?;
        for (idx, path) in self.traits.iter().enumerate() {
            if idx != 0 {
                write!(f, " & ")?;
            }
            write!(f, "{path}")?;
        }
        Ok(())
    }
}

impl TypeInfo for TraitObjectType {
    fn byte_size(&self, register_byte_size: usize) -> Option<usize> {
        None
    }

    fn bit_size(&self, register_byte_size: usize) -> Option<usize> {
        None
    }

    fn byte_align(&self, register_byte_size: usize) -> Option<usize> {
        None
    }
}