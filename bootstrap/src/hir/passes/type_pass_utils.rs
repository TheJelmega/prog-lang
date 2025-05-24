use passes::PassContext;

use crate::hir::*;


pub struct TypeGenUtils<'a> {
    ctx:           &'a PassContext,
    pub cur_scope: Scope,
}


impl<'a> TypeGenUtils<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx,
            cur_scope: Scope::new(),
        }
    }
}


impl Visitor for TypeGenUtils<'_> {
    fn visit_unit_type(&mut self, node: &mut UnitType) {
        if node.ctx.ty.is_some() {
            return;
        }

        let mut registry = self.ctx.type_reg.write();
        let ty = registry.create_unit_type();
        node.ctx.ty = Some(ty);
    }

    fn visit_never_type(&mut self, node: &mut NeverType) {
        if node.ctx.ty.is_some() {
            return;
        }
        
        let mut registry = self.ctx.type_reg.write();
        let ty = registry.create_never_type();
        node.ctx.ty = Some(ty);
    }

    fn visit_primitive_type(&mut self, node: &mut PrimitiveType) {
        if node.ctx.ty.is_some() {
            return;
        }

        let mut registry = self.ctx.type_reg.write();
        let ty = registry.create_primitive_type(node.ty);
        node.ctx.ty = Some(ty);
    }

    fn visit_path_type(&mut self, node: &mut PathType) {
        if node.ctx.ty.is_some() {
            return;
        }

        let mut registry = self.ctx.type_reg.write();
        let ty = registry.create_path_type(node.path.ctx.path.clone());
        node.ctx.ty = Some(ty);
    }

    fn visit_tuple_type(&mut self, node: &mut TupleType) {
        if node.ctx.ty.is_some() {
            return;
        }

        helpers::visit_tuple_type(self, node);

        let mut types = Vec::with_capacity(node.types.len());
        for ty in &node.types {
            types.push(ty.ctx().ty.as_ref().unwrap().clone());
        }

        let mut registry = self.ctx.type_reg.write();
        let ty = registry.create_tuple_type(&types);
        node.ctx.ty = Some(ty);
    }

    fn visit_array_type(&mut self, node: &mut ArrayType) {
        if node.ctx.ty.is_some() {
            return;
        }

        helpers::visit_array_type(self, node);

        let inner_ty = node.ty.ctx().ty.as_ref().unwrap().clone();

        let mut registry = self.ctx.type_reg.write();
        let ty = registry.create_array_type(inner_ty, None);
        node.ctx.ty = Some(ty);
    }

    fn visit_slice_type(&mut self, node: &mut SliceType) {
        if node.ctx.ty.is_some() {
            return;
        }

        helpers::visit_slice_type(self, node);

        let inner_ty = node.ty.ctx().ty.as_ref().unwrap().clone();

        let mut registry = self.ctx.type_reg.write();
        let ty = registry.create_slice_type(inner_ty);
        node.ctx.ty = Some(ty);
    }

    fn visit_string_slice_type(&mut self, node: &mut StringSliceType) {
        if node.ctx.ty.is_some() {
            return;
        }

        let mut registry = self.ctx.type_reg.write();
        let ty = registry.create_str_slice_type(node.ty);
        node.ctx.ty = Some(ty);
    }

    fn visit_pointer_type(&mut self, node: &mut PointerType) {
        if node.ctx.ty.is_some() {
            return;
        }

        helpers::visit_pointer_type(self, node);

        let inner_ty = node.ty.ctx().ty.as_ref().unwrap().clone();

        let mut registry = self.ctx.type_reg.write();
        let ty = registry.create_pointer_type(inner_ty, node.is_multi);
        node.ctx.ty = Some(ty);
    }

    fn visit_reference_type(&mut self, node: &mut ReferenceType) {
        if node.ctx.ty.is_some() {
            return;
        }

        helpers::visit_reference_type(self, node);

        let inner_ty = node.ty.ctx().ty.as_ref().unwrap().clone();

        let mut registry = self.ctx.type_reg.write();
        let ty = registry.create_reference_type(inner_ty, node.is_mut);
        node.ctx.ty = Some(ty);
    }

    fn visit_optional_type(&mut self, node: &mut OptionalType) {
        if node.ctx.ty.is_some() {
            return;
        }

        // TODO
        let mut registry = self.ctx.type_reg.write();
        let ty = registry.create_unit_type();
        node.ctx.ty = Some(ty);
    }

    fn visit_fn_type(&mut self, node: &mut FnType) {
        if node.ctx.ty.is_some() {
            return;
        }

        // TODO
        let mut registry = self.ctx.type_reg.write();
        let ty = registry.create_unit_type();
        node.ctx.ty = Some(ty);
    }
}