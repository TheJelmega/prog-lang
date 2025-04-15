use std::{cmp::max, fmt, sync::Arc};

use parking_lot::RwLock;

use super::{Type, TypeInfo};

pub struct TupleType {
    pub types: Vec<Arc<Type>>,
}

impl fmt::Display for TupleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        for (idx, ty) in self.types.iter().enumerate() {
            if idx == 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", &ty)?;
        }
        write!(f, ")")
    }
}

impl TypeInfo for TupleType {
    fn byte_size(&self, register_byte_size: usize) -> Option<usize> {
        // TODO
        None
    }

    fn bit_size(&self, register_byte_size: usize) -> Option<usize> {
        let mut bit_size = 0;
        for ty in &self.types {
            bit_size += ty.bit_size(register_byte_size)?;
        }
        Some(bit_size)
    }

    fn byte_align(&self, register_byte_size: usize) -> Option<usize> {
        let mut align = 0;
        for ty in &self.types {
            let sub_align = ty.byte_align(register_byte_size)?;
            align = max(align, sub_align);
        }
        Some(align)
    }
}