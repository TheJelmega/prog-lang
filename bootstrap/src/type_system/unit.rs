use std::fmt;

use super::TypeInfo;

#[derive(Debug)]
pub struct UnitType;

impl fmt::Display for UnitType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "()")
    }
}

impl TypeInfo for UnitType {
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