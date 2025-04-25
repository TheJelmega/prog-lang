use std::{ops::Deref, sync::Arc};

use parking_lot::RwLock;

use crate::common::{Logger, SymbolPath, SymbolRef};

use super::*;

pub struct TypeRegistry {
    prim_types:      Vec<Option<TypeHandle>>,
    str_slice_types: Vec<Option<TypeHandle>>,
    unit_ty:         Option<TypeHandle>,
    never_ty:        Option<TypeHandle>,
    path_types:      Vec<TypeHandle>,
    tuple_types:     Vec<TypeHandle>,
    array_types:     Vec<TypeHandle>,
    slice_types:     Vec<TypeHandle>,
    pointer_types:   Vec<TypeHandle>,
    reference_types: Vec<TypeHandle>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        Self {
            prim_types: Vec::new(),
            str_slice_types: Vec::new(),
            unit_ty: None,
            never_ty: None,
            path_types: Vec::new(),
            tuple_types: Vec::new(),
            array_types: Vec::new(),
            slice_types: Vec::new(),
            pointer_types: Vec::new(),
            reference_types: Vec::new(),
        }
    }

    pub fn type_count(&self) -> u32 {
        let mut count = 0;
        count += self.unit_ty.is_some() as usize;
        count += self.never_ty.is_some() as usize;

        for ty in &self.prim_types {
            count += ty.is_some() as usize;
        }
        for ty in &self.str_slice_types {
            count += ty.is_some() as usize;
        }

        count += self.path_types.len();
        count += self.tuple_types.len();
        count += self.array_types.len();
        count += self.slice_types.len();
        count += self.pointer_types.len();
        count += self.reference_types.len();

        count as u32
    }

    pub fn log(&self) {
        let logger = Logger::new();
        
        logger.logln("Type registry:");
        if self.unit_ty.is_some() {
            logger.logln("- ()");
        }
        if self.never_ty.is_some() {
            logger.logln("- !");
        }

        if !self.prim_types.is_empty() {
            logger.logln("- Primitive types:");
            for prim_ty in &self.prim_types {
                if let Some(prim_ty) = prim_ty {
                    let prim_ty = prim_ty.get();
                    logger.log_fmt(format_args!("    {prim_ty}\n"));
                }
            }
        }

        if !self.str_slice_types.is_empty() {
            logger.logln("- String slice types:");
            for str_slice_ty in &self.str_slice_types {
                if let Some(str_slice_ty) = str_slice_ty {
                    logger.log_fmt(format_args!("    {str_slice_ty}\n"));
                }
            }
        }

        if !self.path_types.is_empty() {
            logger.logln("- Path types:");
            for path_ty in &self.path_types {
                logger.log_fmt(format_args!("    {path_ty}\n"));
            }
        }

        if !self.tuple_types.is_empty() {
            logger.logln("- Tuple types:");
            for tup_ty in &self.tuple_types {
                logger.log_fmt(format_args!("    {tup_ty}\n"));
            }
        }

        if !self.array_types.is_empty() {
            logger.logln("- Array types:");
            for arr_ty in &self.array_types {
                logger.log_fmt(format_args!("    {arr_ty}\n"));
            }
        }

        if !self.slice_types.is_empty() {
            logger.logln("- Slice types:");
            for slice_ty in &self.slice_types {
                logger.log_fmt(format_args!("    {slice_ty}\n"));
            }
        }

        if !self.pointer_types.is_empty() {
            logger.logln("- Pointer types:");
            for ptr_ty in &self.pointer_types {
                logger.log_fmt(format_args!("    {ptr_ty}\n"));
            }
        }

        if !self.reference_types.is_empty() {
            logger.logln("- Reference types:");
            for ref_ty in &self.reference_types {
                logger.log_fmt(format_args!("    {ref_ty}\n"));
            }
        }
    }

    pub fn create_primitive_type(&mut self, ty: PrimitiveType) -> TypeHandle {
        let idx = ty as usize;
        if idx < self.prim_types.len() {
            if let Some(prim_ty) = &self.prim_types[idx] {
                return prim_ty.clone();
            }
        }

        let ty = TypeHandle::new(Type::Primitive(ty));
        if idx >= self.prim_types.len() {
            self.prim_types.resize(idx + 1, None);
        }
        self.prim_types[idx] = Some(ty.clone());
        ty
    }

    pub fn create_str_slice_type(&mut self, ty: StringSliceType) -> TypeHandle {
        let idx = ty as usize;
        if idx < self.str_slice_types.len() {
            if let Some(str_slice_ty) = &self.str_slice_types[idx] {
                return str_slice_ty.clone();
            }
        }

        let ty = TypeHandle::new(Type::StringSlice(ty));
        if idx >= self.str_slice_types.len() {
            self.str_slice_types.resize(idx + 1, None);
        }
        self.str_slice_types[idx] = Some(ty.clone());
        ty
    }

    pub fn create_unit_type(&mut self) -> TypeHandle {
        if let Some(ty) = &self.unit_ty {
            ty.clone()
        } else {
            let ty = TypeHandle::new(Type::Unit(UnitType));
            self.unit_ty = Some(ty.clone());
            ty
        }
    }

    pub fn create_never_type(&mut self) -> TypeHandle {
        if let Some(ty) = &self.unit_ty {
            ty.clone()
        } else {
            let ty = TypeHandle::new(Type::Never(NeverType));
            self.unit_ty = Some(ty.clone());
            ty
        }
    }

    pub fn create_path_type(&mut self, sym: SymbolRef) -> TypeHandle {
        for path_ty in &self.path_types {
            let Type::Path(PathType{ sym: inner_sym }) = &*path_ty.get() else { unreachable!() };
            if Arc::ptr_eq(inner_sym, &sym) {
                return path_ty.clone();
            }
        }

        let ty = TypeHandle::new(Type::Path(PathType { sym  }));
        self.path_types.push(ty.clone());
        ty
    }

    pub fn create_tuple_type(&mut self, types: &[TypeHandle]) -> TypeHandle {
        'outer: for tup in &self.tuple_types {
            let Type::Tuple(TupleType { types: tup_types }) = &*tup.get() else { unreachable!() };
            if tup_types.len() != types.len() {
                continue;
            }

            for (ty, tup_ty) in types.iter().zip(tup_types.iter()) {
                if !TypeHandle::ptr_eq(ty, tup_ty) {
                    continue 'outer;
                }
            }

            return tup.clone();
        }

        let ty = TypeHandle::new(Type::Tuple(TupleType { types: Vec::from(types) }));
        self.tuple_types.push(ty.clone());
        ty
    }

    pub fn create_array_type(&mut self, ty: TypeHandle, size: Option<usize>) -> TypeHandle {
        for arr_ty in &self.array_types {
            let Type::Array(ArrayType { ty: inner_ty, size: inner_size }) = &*arr_ty.get() else { unreachable!() };
            if TypeHandle::ptr_eq(inner_ty, &ty) && *inner_size == size {
                return arr_ty.clone();
            }
        }

        let ty = TypeHandle::new(Type::Array(ArrayType { ty, size }));
        self.array_types.push(ty.clone());
        ty
    }

    pub fn create_slice_type(&mut self, ty: TypeHandle) -> TypeHandle {
        for arr_ty in &self.slice_types {
            let Type::Slice(SliceType { ty: inner_ty }) = &*arr_ty.get() else { unreachable!() };
            if TypeHandle::ptr_eq(inner_ty, &ty) {
                return arr_ty.clone();
            }
        }

        let ty = TypeHandle::new(Type::Slice(SliceType { ty }));
        self.slice_types.push(ty.clone());
        ty
    }

    pub fn create_pointer_type(&mut self, ty: TypeHandle, is_multi: bool) -> TypeHandle {
        for ptr_ty in &self.pointer_types {
            let Type::Pointer(PointerType { ty: inner_ty, is_multi: inner_multi }) = &*ptr_ty.get() else { unreachable!() };
            if TypeHandle::ptr_eq(inner_ty, &ty) && *inner_multi == is_multi {
                return ptr_ty.clone();
            }
        }

        let ty = TypeHandle::new(Type::Pointer(PointerType { ty, is_multi }));
        self.pointer_types.push(ty.clone());
        ty
    }

    pub fn create_reference_type(&mut self, ty: TypeHandle, is_mut: bool) -> TypeHandle {
        for ref_ty in &self.reference_types {
            let Type::Reference(ReferenceType { ty: inner_ty, is_mut: inner_mut }) = &*ref_ty.get() else { unreachable!() };
            if TypeHandle::ptr_eq(inner_ty, &ty) && *inner_mut == is_mut {
                return ref_ty.clone();
            }
        }

        let ty = TypeHandle::new(Type::Reference(ReferenceType { ty, is_mut }));
        self.reference_types.push(ty.clone());
        ty
    }

}