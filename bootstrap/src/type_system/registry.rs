use std::{ops::Deref, sync::Arc};

use parking_lot::RwLock;

use crate::common::{Logger, PathGeneric, Scope, SymbolPath, SymbolRef};

use super::*;

// TODO: Handle creation after replacements correctly

pub struct TypeRegistry {
    dependencies:    DependencyDag,

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
    placeholders:    Vec<TypeHandle>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        Self {
            dependencies: DependencyDag::new(),

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
            placeholders: Vec::new(),
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
                    logger.log_fmt(format_args!("    - {prim_ty}\n"));
                }
            }
        }

        if !self.str_slice_types.is_empty() {
            logger.logln("- String slice types:");
            for str_slice_ty in &self.str_slice_types {
                if let Some(str_slice_ty) = str_slice_ty {
                    logger.log_fmt(format_args!("    - {str_slice_ty}\n"));
                }
            }
        }

        if !self.path_types.is_empty() {
            logger.logln("- Path types:");
            for path_ty in &self.path_types {
                let ty = path_ty.get();
                let Type::Path(ty) = &*ty else { unreachable!() };
                let resolved = if ty.sym.is_some() {
                    "  (symbol resolved)"
                } else {
                    ""
                };
                // We first write it in a string, as rust decided to limit how you can format it, but doesn't even handle it for you.
                // So we are just gonna let the string formatting handle it for us.
                // Then why not allow completely custom format specifiers. smh.
                // And no, I'm not gonna handle all possible specifiers in each implementation of display I'm gonna make.
                let ty_str = format!("{path_ty}");
                logger.log_fmt(format_args!("    - {ty_str:96}{resolved}\n"));
            }
        }

        if !self.tuple_types.is_empty() {
            logger.logln("- Tuple types:");
            for tup_ty in &self.tuple_types {
                logger.log_fmt(format_args!("    - {tup_ty}\n"));
            }
        }

        if !self.array_types.is_empty() {
            logger.logln("- Array types:");
            for arr_ty in &self.array_types {
                logger.log_fmt(format_args!("    - {arr_ty}\n"));
            }
        }

        if !self.slice_types.is_empty() {
            logger.logln("- Slice types:");
            for slice_ty in &self.slice_types {
                logger.log_fmt(format_args!("    - {slice_ty}\n"));
            }
        }

        if !self.pointer_types.is_empty() {
            logger.logln("- Pointer types:");
            for ptr_ty in &self.pointer_types {
                logger.log_fmt(format_args!("    - {ptr_ty}\n"));
            }
        }

        if !self.reference_types.is_empty() {
            logger.logln("- Reference types:");
            for ref_ty in &self.reference_types {
                logger.log_fmt(format_args!("    - {ref_ty}\n"));
            }
        }
    }

    pub fn log_dependencies(&self) {
        self.dependencies.log_nodes();
    }

    // TODO: Add some info in the registry to fix up and propagate resolved types (if this is actually needed at some point)
    pub fn set_resolved(&mut self, ty: &TypeHandle, resolved: TypeHandle) {
        ty.handle.write().resolved = Some(resolved);
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
        self.dependencies.add(ty.clone());
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
        self.dependencies.add(ty.clone());
        ty
    }

    pub fn create_unit_type(&mut self) -> TypeHandle {
        if let Some(ty) = &self.unit_ty {
            ty.clone()
        } else {
            let ty = TypeHandle::new(Type::Unit(UnitType));
            self.unit_ty = Some(ty.clone());
            self.dependencies.add(ty.clone());
            ty
        }
    }

    pub fn create_never_type(&mut self) -> TypeHandle {
        if let Some(ty) = &self.unit_ty {
            ty.clone()
        } else {
            let ty = TypeHandle::new(Type::Never(NeverType));
            self.unit_ty = Some(ty.clone());
            self.dependencies.add(ty.clone());
            ty
        }
    }

    pub fn create_sym_path_type(&mut self, sym: SymbolRef) -> TypeHandle {
        let path = sym.read().path().clone();

        for path_ty in &self.path_types {
            let Type::Path(PathType{ sym: inner_sym, .. }) = &*path_ty.get() else { unreachable!() };
            if let Some(inner_sym) = inner_sym {   
                if Arc::ptr_eq(inner_sym, &sym) {
                    return path_ty.clone();
                }
            }
        }
        
        let ty = TypeHandle::new(Type::Path(PathType { path: path.clone(), sym: Some(sym) }));
        self.path_types.push(ty.clone());

        self.dependencies.add(ty.clone());
        let dag_idx = ty.handle.read().dag_idx();
        for segment in path.scope().idens() {
            for arg in &segment.gen_args {
                match arg {
                    PathGeneric::Type { ty } => {
                        let base_idx = ty.handle.read().dag_idx();
                        self.dependencies.set_dependency(dag_idx, base_idx);
                    },
                    _ => (),
                }
            }
        }
        for arg in &path.iden().gen_args {
            match arg {
                PathGeneric::Type { ty } => {
                    let base_idx = ty.handle.read().dag_idx();
                    self.dependencies.set_dependency(dag_idx, base_idx);
                },
                _ => (),
            }
        }
        ty
    }

    pub fn create_path_type(&mut self, path: SymbolPath) -> TypeHandle {
        // We don't have enough info to actually resolves what the path points to, i.e. don't know the full path, just the local one
        // So just create a new type, we can later on redirect it to the correct path
        // But there does need to be a better way to do it, but generics make this a harder problem to solve atm without further work on type resolution
        let ty = TypeHandle::new(Type::Path(PathType{ path: path.clone(), sym: None }));
        self.path_types.push(ty.clone());

        self.dependencies.add(ty.clone());
        let dag_idx = ty.handle.read().dag_idx();
        for segment in path.scope().idens() {
            for arg in &segment.gen_args {
                match arg {
                    PathGeneric::Type { ty } => {
                        let base_idx = ty.handle.read().dag_idx();
                        self.dependencies.set_dependency(dag_idx, base_idx);
                    },
                    _ => (),
                }
            }
        }
        for arg in &path.iden().gen_args {
            match arg {
                PathGeneric::Type { ty } => {
                    let base_idx = ty.handle.read().dag_idx();
                    self.dependencies.set_dependency(dag_idx, base_idx);
                },
                _ => (),
            }
        }
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

        self.dependencies.add(ty.clone());
        let dag_idx = ty.handle.read().dag_idx();
        for ty in types {
            let base_idx = ty.handle.read().dag_idx();
            self.dependencies.set_dependency(dag_idx, base_idx);
        }

        ty
    }

    pub fn create_array_type(&mut self, elem_ty: TypeHandle, size: Option<usize>) -> TypeHandle {
        for arr_ty in &self.array_types {
            let Type::Array(ArrayType { ty: inner_ty, size: inner_size }) = &*arr_ty.get() else { unreachable!() };
            if TypeHandle::ptr_eq(inner_ty, &elem_ty) && *inner_size == size {
                return arr_ty.clone();
            }
        }

        let ty = TypeHandle::new(Type::Array(ArrayType { ty: elem_ty.clone(), size }));
        self.array_types.push(ty.clone());

        self.dependencies.add(ty.clone());
        let dag_idx = ty.handle.read().dag_idx();
        let base_idx = elem_ty.handle.read().dag_idx();
        self.dependencies.set_dependency(dag_idx, base_idx);

        ty
    }

    pub fn create_slice_type(&mut self, elem_ty: TypeHandle) -> TypeHandle {
        for arr_ty in &self.slice_types {
            let Type::Slice(SliceType { ty: inner_ty }) = &*arr_ty.get() else { unreachable!() };
            if TypeHandle::ptr_eq(inner_ty, &elem_ty) {
                return arr_ty.clone();
            }
        }

        let ty = TypeHandle::new(Type::Slice(SliceType { ty: elem_ty.clone() }));
        self.slice_types.push(ty.clone());

        self.dependencies.add(ty.clone());
        let dag_idx = ty.handle.read().dag_idx();
        let base_idx = elem_ty.handle.read().dag_idx();
        self.dependencies.set_dependency(dag_idx, base_idx);

        ty
    }

    pub fn create_pointer_type(&mut self, elem_ty: TypeHandle, is_multi: bool) -> TypeHandle {
        for ptr_ty in &self.pointer_types {
            let Type::Pointer(PointerType { ty: inner_ty, is_multi: inner_multi }) = &*ptr_ty.get() else { unreachable!() };
            if TypeHandle::ptr_eq(inner_ty, &elem_ty) && *inner_multi == is_multi {
                return ptr_ty.clone();
            }
        }

        let ty = TypeHandle::new(Type::Pointer(PointerType { ty: elem_ty.clone(), is_multi }));
        self.pointer_types.push(ty.clone());

        self.dependencies.add(ty.clone());
        let dag_idx = ty.handle.read().dag_idx();
        let base_idx = elem_ty.handle.read().dag_idx();
        self.dependencies.set_dependency(dag_idx, base_idx);

        ty
    }

    pub fn create_reference_type(&mut self, elem_ty: TypeHandle, is_mut: bool) -> TypeHandle {
        for ref_ty in &self.reference_types {
            let Type::Reference(ReferenceType { ty: inner_ty, is_mut: inner_mut }) = &*ref_ty.get() else { unreachable!() };
            if TypeHandle::ptr_eq(inner_ty, &elem_ty) && *inner_mut == is_mut {
                return ref_ty.clone();
            }
        }

        let ty = TypeHandle::new(Type::Reference(ReferenceType { ty: elem_ty.clone(), is_mut }));
        self.reference_types.push(ty.clone());
        ty
    }

    pub fn create_placeholder_type(&mut self) -> TypeHandle {
        let ty = TypeHandle::new(Type::Placeholder);
        self.placeholders.push(ty.clone());
        self.dependencies.add(ty.clone());
        ty
    }
}