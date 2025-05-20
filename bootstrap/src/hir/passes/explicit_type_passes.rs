use crate::{
    common::Scope,
    error_warning::HirErrorCode,
    hir::*
};

use super::{Pass, PassContext};

pub struct ExplicitTypeGenHelper<'a> {
    ctx:           &'a PassContext,
    pub cur_scope: Scope,
}

impl<'a> ExplicitTypeGenHelper<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx,
            cur_scope: Scope::new(),
        }
    }
}

impl Visitor for ExplicitTypeGenHelper<'_> {
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

        let syms = self.ctx.syms.read();
        let uses = self.ctx.uses.read();

        let sym = match syms.get_symbol_with_uses(&uses, &self.cur_scope, None, &node.path.ctx.path) {
            Some(sym) => {
                let mut registry = self.ctx.type_reg.write();
                let ty = registry.create_path_type(sym);
                node.ctx.ty = Some(ty);
            },
            None => {
                self.ctx.add_error(HirError {
                    span: node.span,
                    err: HirErrorCode::UnknownSymbol { path: node.path.ctx.path.clone() },
                });

                let mut registry = self.ctx.type_reg.write();
                let ty = registry.create_unit_type();
                node.ctx.ty = Some(ty);
            },
        };

        
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


pub struct ImplTypeGen<'a> {
    ctx:    &'a PassContext,
    helper: ExplicitTypeGenHelper<'a>
}

impl<'a> ImplTypeGen<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx,
            helper: ExplicitTypeGenHelper::new(ctx),
        }
    }
}

impl Visitor for ImplTypeGen<'_> {
    fn visit_impl(&mut self, node: &mut Impl, ctx: &mut ImplContext) {
        self.helper.visit_type(&mut node.ty);
    }
}

impl Pass for ImplTypeGen<'_> {
    const NAME: &'static str = "Impl Type Generation";

    fn process(&mut self, hir: &mut Hir) {
        self.visit(hir, VisitFlags::Impl);
    }
}