use std::{fmt, sync::Arc};

use super::{Type, TypeHandle, TypeInfo};

#[derive(Debug)]
pub struct SliceType {
    pub ty: TypeHandle,
    // TODO: sentinel
}

impl fmt::Display for SliceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[]{}", self.ty)
    }
}

impl TypeInfo for SliceType {
    fn byte_size(&self, register_byte_size: usize) -> Option<usize> {
        Some(register_byte_size * 2)
    }

    fn bit_size(&self, register_byte_size: usize) -> Option<usize> {
        Some(register_byte_size * 2)
    }

    fn byte_align(&self, register_byte_size: usize) -> Option<usize> {
        Some(register_byte_size)
    }
}