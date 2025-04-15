use std::{fmt, sync::Arc};

use super::{Type, TypeInfo};

pub struct PointerType {
    pub ty:    Arc<Type>,
    pub is_multi: bool,
    // TODO: sentinel
}

impl fmt::Display for PointerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_multi {
            write!(f, "[^]{}", self.ty)
        } else {
            write!(f, "^{}", self.ty)
        }
    }
}

// TODO: Fat pointer when reference to trait
impl TypeInfo for PointerType {
    fn byte_size(&self, register_byte_size: usize) -> Option<usize> {
        if matches!(&*self.ty, Type::TraitObject(_) | Type::ImplTrait(_)) {
            Some(register_byte_size * 2)
        } else {
            Some(register_byte_size)
        }
    }

    fn bit_size(&self, register_byte_size: usize) -> Option<usize> {
        if matches!(&*self.ty, Type::TraitObject(_) | Type::ImplTrait(_)) {
            Some(register_byte_size * 2)
        } else {
            Some(register_byte_size)
        }
    }

    fn byte_align(&self, register_byte_size: usize) -> Option<usize> {
        Some(register_byte_size)
    }
}