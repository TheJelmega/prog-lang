#![allow(unused)]


mod primtive;
use std::fmt;

pub use primtive::*;

mod string_slice;
pub use string_slice::*;

mod unit;
pub use unit::*;

mod never;
pub use never::*;

mod path;
pub use path::*; 

mod tuple;
pub use tuple::*;

mod array;
pub use array::*;

mod slice;
pub use slice::*;

mod pointer;
pub use pointer::*;

mod reference;
pub use reference::*;



mod fn_ptr;
pub use fn_ptr::*;




mod trait_object;
pub use trait_object::*;

mod impl_trait;
pub use impl_trait::*;


mod registry;
pub use registry::*;



pub enum Type {
    Primitive(PrimitiveType),
    Unit(UnitType),
    Never(NeverType),
    Path(PathType),
    Tuple(TupleType),
    Array(ArrayType),
    Slice(SliceType),
    StringSlice(StringSliceType),
    Pointer(PointerType),
    Reference(ReferenceType),
    Optional, // ??
    Func,
    FuncPtr(FnPtrType),
    Closure,
    Inferred,
    TraitObject(TraitObjectType),
    ImplTrait(ImplTraitType),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Primitive(ty)       => write!(f, "{ty}"),
            Type::Unit(ty)            => write!(f, "{ty}"),
            Type::Never(ty)           => write!(f, "{ty}"),
            Type::Path(ty)            => write!(f, "{ty}"),
            Type::Tuple(ty)           => write!(f, "{ty}"),
            Type::Array(ty)           => write!(f, "{ty}"),
            Type::Slice(ty)           => write!(f, "{ty}"),
            Type::StringSlice(ty)     => write!(f, "{ty}"),
            Type::Pointer(ty)         => write!(f, "{ty}"),
            Type::Reference(ty)       => write!(f, "{ty}"),
            Type::Optional        => todo!(),
            Type::Func            => todo!(),
            Type::FuncPtr(ty)         => write!(f, "<inferred>"),
            Type::Closure         => todo!(),
            Type::Inferred            => write!(f, "<inferred>"),
            Type::TraitObject(ty)     => write!(f, "{ty}"),
            Type::ImplTrait(ty)       => write!(f, "{ty}"),
        }
    }
}

impl TypeInfo for Type {
    fn byte_size(&self, register_byte_size: usize) -> Option<usize> {
        todo!()
    }

    fn bit_size(&self, register_byte_size: usize) -> Option<usize> {
        todo!()
    }

    fn byte_align(&self, register_byte_size: usize) -> Option<usize> {
        todo!()
    }
}

pub trait TypeInfo {
    fn byte_size(&self, register_byte_size: usize) -> Option<usize>;
    fn bit_size(&self, register_byte_size: usize) -> Option<usize>;
    fn byte_align(&self, register_byte_size: usize) -> Option<usize>;
}