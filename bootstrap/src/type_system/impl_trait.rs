use std::fmt;

use crate::common::SymbolPath;

use super::TypeInfo;

pub struct ImplTraitType {
    pub traits: Vec<SymbolPath>    
}

impl fmt::Display for ImplTraitType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "impl ")?;
        for (idx, path) in self.traits.iter().enumerate() {
            if idx != 0 {
                write!(f, " & ")?;
            }
            write!(f, "{path}")?;
        }
        Ok(())
    }
}

impl TypeInfo for ImplTraitType {
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