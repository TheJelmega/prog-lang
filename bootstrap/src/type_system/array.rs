use std::{fmt, sync::Arc};

use super::{Type, TypeHandle, TypeInfo};

pub struct ArrayType {
    pub ty:   TypeHandle,
    pub size: Option<usize>,
    // TODO: sentinel
}

impl fmt::Display for ArrayType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.size {
            Some(size) => write!(f, "[{size}]{}", self.ty),
            None => write!(f, "[<unknown>]{}", self.ty),
        }
    }
}

impl TypeInfo for ArrayType {
    fn byte_size(&self, register_byte_size: usize) -> Option<usize> {
        let elem_size = self.ty.get().byte_size(register_byte_size);

        match self.size {
            Some(size) => elem_size.map(|elem_size| size * elem_size),
            None => None,
        }
    }

    fn bit_size(&self, register_byte_size: usize) -> Option<usize> {
        let elem_size = self.ty.get().bit_size(register_byte_size);
        match self.size {
            Some(size) => elem_size.map(|elem_size| size * elem_size),
            None => None,
        }
    }

    fn byte_align(&self, register_byte_size: usize) -> Option<usize> {
        self.ty.get().byte_align(register_byte_size)
    }
}