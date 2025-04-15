use std::{fmt, sync::Arc};

use super::{Type, TypeInfo};

pub struct FnPtrType {
    pub params: Vec<Arc<Type>>,
    pub ret:    Option<Arc<Type>>,
}

impl fmt::Display for FnPtrType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fn(")?;
        for (idx, ty) in self.params.iter().enumerate() {
            if idx != 0 {
                write!(f, ", ")?;
            }
            write!(f, "{ty}")?;
        }
        write!(f, ")")?;
        if let Some(ret) = &self.ret {
            write!(f, " -> {ret}")?;
        }
        Ok(())
    }
}

impl TypeInfo for FnPtrType {
    fn byte_size(&self, register_byte_size: usize) -> Option<usize> {
        Some(register_byte_size)
    }

    fn bit_size(&self, register_byte_size: usize) -> Option<usize> {
        Some(register_byte_size)
    }

    fn byte_align(&self, register_byte_size: usize) -> Option<usize> {
        Some(register_byte_size)
    }
}