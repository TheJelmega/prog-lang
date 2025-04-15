use std::{fmt, sync::Arc};

use super::{Type, TypeInfo};

pub struct ReferenceType {
    pub ty:     Arc<Type>,
    pub is_mut: bool,
}

impl fmt::Display for ReferenceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "&{}{}",
            if self.is_mut { "mut " } else { "" },
            self.ty
        )
    }
}

// TODO: Fat pointer when reference to trait
impl TypeInfo for ReferenceType {
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