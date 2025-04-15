use std::fmt;

use super::TypeInfo;

pub struct NeverType;

impl fmt::Display for NeverType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "!")
    }
}

impl TypeInfo for NeverType {
    fn byte_size(&self, register_byte_size: usize) -> Option<usize> {
        Some(0)
    }

    fn bit_size(&self, register_byte_size: usize) -> Option<usize> {
        Some(0)
    }

    fn byte_align(&self, register_byte_size: usize) -> Option<usize> {
        Some(0)
    }
}