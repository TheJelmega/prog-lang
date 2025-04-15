use std::fmt;

use super::TypeInfo;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StringSliceType {
    Str,
    Str7,
    Str8,
    Str16,
    Str32,
    CStr,
}

impl StringSliceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Str   => "str",
            Self::Str7  => "str7",
            Self::Str8  => "str8",
            Self::Str16 => "str16",
            Self::Str32 => "str32",
            Self::CStr  => "cstr",
        }
    }
}

impl fmt::Display for StringSliceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TypeInfo for StringSliceType {
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