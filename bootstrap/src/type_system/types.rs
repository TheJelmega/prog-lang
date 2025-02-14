#![allow(unused)]

use std::fmt;


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PrimitiveType {
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    F16,
    F32,
    F64,
    F128,
    Bool,
    B8,
    B16,
    B32,
    B64,
    Char,
    Char7,
    Char8,
    Char16,
    Char32,
}

impl PrimitiveType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::U8 =>     "u8",
            Self::U16 =>    "u16",
            Self::U32 =>    "u32",
            Self::U64 =>    "u64",
            Self::U128 =>   "u128",
            Self::Usize =>  "usize",
            Self::I8 =>     "i8",
            Self::I16 =>    "i16",
            Self::I32 =>    "i32",
            Self::I64 =>    "i64",
            Self::I128 =>   "i128",
            Self::Isize =>  "isize",
            Self::F16 =>    "f16",
            Self::F32 =>    "f32",
            Self::F64 =>    "f64",
            Self::F128 =>   "f128",
            Self::Bool =>   "bool",
            Self::B8 =>     "b8",
            Self::B16 =>    "b16",
            Self::B32 =>    "b32",
            Self::B64 =>    "b64",
            Self::Char =>   "char",
            Self::Char7 =>  "char7",
            Self::Char8 =>  "char8",
            Self::Char16 => "char16",
            Self::Char32 => "char32",
        }
    }

    pub fn byte_size(&self, reg_byte_size: usize) -> usize {
        match self {
            Self::U8     => 1,
            Self::U16    => 2,
            Self::U32    => 4,
            Self::U64    => 8,
            Self::U128   => 16,
            Self::Usize  => reg_byte_size,
            Self::I8     => 1,
            Self::I16    => 2,
            Self::I32    => 4,
            Self::I64    => 8,
            Self::I128   => 16,
            Self::Isize  => reg_byte_size,
            Self::F16    => 2,
            Self::F32    => 4,
            Self::F64    => 8,
            Self::F128   => 16,
            Self::Bool   => 1,
            Self::B8     => 1,
            Self::B16    => 2,
            Self::B32    => 4,
            Self::B64    => 8,
            Self::Char   => 4,
            Self::Char7  => 1,
            Self::Char8  => 1,
            Self::Char16 => 2,
            Self::Char32 => 4,
        }
    }

    pub fn bit_size(&self, reg_bit_size: usize) -> usize {
        match self {
            Self::U8     => 8,
            Self::U16    => 16,
            Self::U32    => 32,
            Self::U64    => 64,
            Self::U128   => 128,
            Self::Usize  => reg_bit_size,
            Self::I8     => 8,
            Self::I16    => 16,
            Self::I32    => 32,
            Self::I64    => 64,
            Self::I128   => 128,
            Self::Isize  => reg_bit_size,
            Self::F16    => 16,
            Self::F32    => 32,
            Self::F64    => 64,
            Self::F128   => 128,
            Self::Bool   => 1,
            Self::B8     => 8,
            Self::B16    => 16,
            Self::B32    => 32,
            Self::B64    => 64,
            Self::Char   => 32,
            Self::Char7  => 7,
            Self::Char8  => 8,
            Self::Char16 => 16,
            Self::Char32 => 32,
        }
    }

    pub fn align(&self, reg_byte_size: usize) -> usize {
        self.byte_size(reg_byte_size)
    }
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

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

    pub fn byte_size(&self, reg_byte_size: usize) -> usize {
        reg_byte_size * 2
    }

    pub fn bit_size(&self, reg_bit_size: usize) -> usize {
        reg_bit_size * 2
    }

    pub fn align(&self, reg_byte_size: usize) -> usize {
        reg_byte_size
    }
}

impl fmt::Display for StringSliceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
