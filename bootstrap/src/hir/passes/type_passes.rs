use crate::{common::Symbol, hir::*};

use super::{type_pass_utils::TypeGenUtils, Pass, PassContext};




pub struct ItemLevelTypeGen<'a> {
    ctx:    &'a PassContext,
    helper: TypeGenUtils<'a>,
}

impl<'a> ItemLevelTypeGen<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx,
            helper: TypeGenUtils::new(ctx),
        }
    }
}

impl Visitor for ItemLevelTypeGen<'_> {
    fn visit_function(&mut self, node: &mut Function, ctx: &mut FunctionContext) {
        if let Some(generics) = &mut node.generics {
            self.helper.visit_gen_params(generics);
        }
        if let Some(where_clause) = &mut node.where_clause {
            self.helper.visit_where_clause(where_clause);
        }

        for param in &mut node.params {
            match param {
                FnParam::Param { span, attrs, label, pattern, ty } => self.visit_type(ty),
                FnParam::Opt { span, attrs, label, pattern, ty, def } => self.visit_type(ty),
                FnParam::Variadic { span, attrs, name, ty } => self.visit_type(ty),
            }
        }
        if let Some(ret_ty) = &mut node.return_ty {
            self.helper.visit_type(ret_ty);
        }
        // TODO: Func type
    }

    fn visit_extern_function_no_body(&mut self, node: &mut ExternFunctionNoBody, ctx: &mut FunctionContext) {
        for param in &mut node.params {
            match param {
                FnParam::Param { span, attrs, label, pattern, ty } => self.visit_type(ty),
                FnParam::Opt { span, attrs, label, pattern, ty, def } => self.visit_type(ty),
                FnParam::Variadic { span, attrs, name, ty } => self.visit_type(ty),
            }
        }
        if let Some(ret_ty) = &mut node.return_ty {
            self.helper.visit_type(ret_ty);
        }
        // TODO: Func type
    } 

    fn visit_type_alias(&mut self, node: &mut TypeAlias, ctx: &mut TypeAliasContext) {
        if let Some(generics) = &mut node.generics {
            self.helper.visit_gen_params(generics);
        }
        self.helper.visit_type(&mut node.ty);

        let mut ty_reg = self.ctx.type_reg.write();
        let sym = ctx.sym.clone().unwrap();
        let ty = ty_reg.create_sym_path_type(sym.clone());

        let mut sym = sym.write();
        let Symbol::TypeAlias(sym) = &mut *sym else { unreachable!() };
        sym.ty = Some(ty);
    }

    fn visit_distinct_type(&mut self, node: &mut DistinctType, ctx: &mut TypeAliasContext) {
        if let Some(generics) = &mut node.generics {
            self.helper.visit_gen_params(generics);
        }
        self.helper.visit_type(&mut node.ty);

        let mut ty_reg = self.ctx.type_reg.write();
        let sym = ctx.sym.clone().unwrap();
        let ty = ty_reg.create_sym_path_type(sym.clone());

        let mut sym = sym.write();
        let Symbol::DistinctType(sym) = &mut *sym else { unreachable!() };
        sym.ty = Some(ty);
    }

    fn visit_opaque_type(&mut self, node: &mut OpaqueType, ctx: &mut TypeAliasContext) {
        let mut ty_reg = self.ctx.type_reg.write();
        let sym = ctx.sym.clone().unwrap();
        let ty = ty_reg.create_sym_path_type(sym.clone());

        let mut sym = sym.write();
        let Symbol::OpaqueType(sym) = &mut *sym else { unreachable!() };
        sym.ty = Some(ty);
    }

    fn visit_struct(&mut self, node: &mut Struct, ctx: &mut StructContext) {
        if let Some(generics) = &mut node.generics {
            self.helper.visit_gen_params(generics);
        }
        if let Some(where_clause) = &mut node.where_clause {
            self.helper.visit_where_clause(where_clause);
        }
        
        for field in &mut node.fields {
            self.helper.visit_type(&mut field.ty);
        }

        let mut ty_reg = self.ctx.type_reg.write();
        let sym = ctx.sym.clone().unwrap();
        let ty = ty_reg.create_sym_path_type(sym.clone());

        let mut sym = sym.write();
        let Symbol::Struct(sym) = &mut *sym else { unreachable!() };
        sym.ty = Some(ty);
    }

    fn visit_tuple_struct(&mut self, node: &mut TupleStruct, ctx: &mut StructContext) {
        if let Some(generics) = &mut node.generics {
            self.helper.visit_gen_params(generics);
        }
        if let Some(where_clause) = &mut node.where_clause {
            self.helper.visit_where_clause(where_clause);
        }
        
        for field in &mut node.fields {
            self.helper.visit_type(&mut field.ty);
        }
        
        let mut ty_reg = self.ctx.type_reg.write();
        let sym = ctx.sym.clone().unwrap();
        let ty = ty_reg.create_sym_path_type(sym.clone());

        let mut sym = sym.write();
        let Symbol::Struct(sym) = &mut *sym else { unreachable!() };
        sym.ty = Some(ty);
    }

    fn visit_unit_struct(&mut self, node: &mut UnitStruct, ctx: &mut StructContext) {
        let mut ty_reg = self.ctx.type_reg.write();
        let sym = ctx.sym.clone().unwrap();
        let ty = ty_reg.create_sym_path_type(sym.clone());

        let mut sym = sym.write();
        let Symbol::Struct(sym) = &mut *sym else { unreachable!() };
        sym.ty = Some(ty);
    }

    fn visit_union(&mut self, node: &mut Union, ctx: &mut UnionContext) {
        if let Some(generics) = &mut node.generics {
            self.helper.visit_gen_params(generics);
        }
        if let Some(where_clause) = &mut node.where_clause {
            self.helper.visit_where_clause(where_clause);
        }
        for field in &mut node.fields {
            self.helper.visit_type(&mut field.ty);
        }
    }

    fn visit_adt_enum(&mut self, node: &mut AdtEnum, ctx: &mut AdtEnumContext) {
        if let Some(generics) = &mut node.generics {
            self.helper.visit_gen_params(generics);
        }
        if let Some(where_clause) = &mut node.where_clause {
            self.helper.visit_where_clause(where_clause);
        }
        for variant in &mut node.variants {
            match variant {
                AdtEnumVariant::Struct { span, attrs, is_mut, name, fields, discriminant } => {
                    for field in fields {
                        self.helper.visit_type(&mut field.ty);
                    }
                },
                AdtEnumVariant::Tuple { span, attrs, is_mut, name, fields, discriminant } => {
                    for field in fields {
                        self.helper.visit_type(&mut field.ty);
                    }
                },
                AdtEnumVariant::Fieldless { span, attrs, name, discriminant } => (),
            }
        }

        let mut ty_reg = self.ctx.type_reg.write();
        let sym = ctx.sym.clone().unwrap();
        let ty = ty_reg.create_sym_path_type(sym.clone());

        let mut sym = sym.write();
        let Symbol::AdtEnum(sym) = &mut *sym else { unreachable!() };
        sym.ty = Some(ty);
    }

    fn visit_flag_enum(&mut self, node: &mut FlagEnum, ctx: &mut FlagEnumContext) {
        let mut ty_reg = self.ctx.type_reg.write();
        let sym = ctx.sym.clone().unwrap();
        let ty = ty_reg.create_sym_path_type(sym.clone());

        let mut sym = sym.write();
        let Symbol::FlagEnum(sym) = &mut *sym else { unreachable!() };
        sym.ty = Some(ty);
    }

    fn visit_bitfield(&mut self, node: &mut Bitfield, ctx: &mut BitfieldContext) {
        if let Some(generics) = &mut node.generics {
            self.helper.visit_gen_params(generics);
        }

        for field in &mut node.fields {
            self.helper.visit_type(&mut field.ty);
        }
        
        let mut ty_reg = self.ctx.type_reg.write();
        let sym = ctx.sym.clone().unwrap();
        let ty = ty_reg.create_sym_path_type(sym.clone());

        let mut sym = sym.write();
        let Symbol::Bitfield(sym) = &mut *sym else { unreachable!() };
        sym.ty = Some(ty);
    }

    fn visit_const(&mut self, node: &mut Const, ctx: &mut ConstContext) {
        if let Some(ty) = &mut node.ty {
            self.helper.visit_type(ty);
            let ty = ty.ctx().ty.clone();
            let mut sym = ctx.sym.as_ref().unwrap().write();
            let Symbol::Const(sym) = &mut *sym else { unreachable!() };
            sym.ty = ty;
        }
    }

    fn visit_static(&mut self, node: &mut Static, ctx: &mut StaticContext) {
        if let Some(ty) = &mut node.ty {
            self.helper.visit_type(ty);
            let ty = ty.ctx().ty.clone();
            let mut sym = ctx.sym.as_ref().unwrap().write();
            let Symbol::Const(sym) = &mut *sym else { unreachable!() };
            sym.ty = ty;
        }
    }

    fn visit_tls_static(&mut self, node: &mut TlsStatic, ctx: &mut StaticContext) {
        if let Some(ty) = &mut node.ty {
            self.helper.visit_type(ty);
            let ty = ty.ctx().ty.clone();
            let mut sym = ctx.sym.as_ref().unwrap().write();
            let Symbol::Const(sym) = &mut *sym else { unreachable!() };
            sym.ty = ty;
        }
    }

    fn visit_extern_static(&mut self, node: &mut ExternStatic, ctx: &mut StaticContext) {
        self.helper.visit_type(&mut node.ty);
        let ty = node.ty.ctx().ty.clone();
        let mut sym = ctx.sym.as_ref().unwrap().write();
        let Symbol::Const(sym) = &mut *sym else { unreachable!() };
        sym.ty = ty;
    }

    fn visit_trait(&mut self, node: &mut Trait, ctx: &mut TraitContext) {
        let mut ty_reg = self.ctx.type_reg.write();
        let sym = ctx.sym.clone().unwrap();
        let ty = ty_reg.create_sym_path_type(sym.clone());

        let mut sym = sym.write();
        let Symbol::Trait(sym) = &mut *sym else { unreachable!() };
        sym.ty = Some(ty);
    }

    fn visit_trait_function(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitFunction, ctx: &mut FunctionContext) {
        if let Some(generics) = &mut node.generics {
            self.helper.visit_gen_params(generics);
        }
        if let Some(where_clause) = &mut node.where_clause {
            self.helper.visit_where_clause(where_clause);
        }

        for param in &mut node.params {
            match param {
                FnParam::Param { span, attrs, label, pattern, ty } => self.visit_type(ty),
                FnParam::Opt { span, attrs, label, pattern, ty, def } => self.visit_type(ty),
                FnParam::Variadic { span, attrs, name, ty } => self.visit_type(ty),
            }
        }
        if let Some(ret_ty) = &mut node.return_ty {
            self.helper.visit_type(ret_ty);
        }
        // TODO: Func type
    }

    fn visit_trait_method(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitMethod, ctx: &mut FunctionContext) {
        if let Some(generics) = &mut node.generics {
            self.helper.visit_gen_params(generics);
        }
        if let Some(where_clause) = &mut node.where_clause {
            self.helper.visit_where_clause(where_clause);
        }
        match &mut node.receiver {
            FnReceiver::None => (),
            FnReceiver::SelfReceiver { span, is_ref, is_mut } => {
                // TODO
            },
            FnReceiver::SelfTyped { span, is_mut, ty } => {
                self.helper.visit_type(ty);
            },
        }
        for param in &mut node.params {
            match param {
                FnParam::Param { span, attrs, label, pattern, ty } => self.visit_type(ty),
                FnParam::Opt { span, attrs, label, pattern, ty, def } => self.visit_type(ty),
                FnParam::Variadic { span, attrs, name, ty } => self.visit_type(ty),
            }
        }
        if let Some(ret_ty) = &mut node.return_ty {
            self.helper.visit_type(ret_ty);
        }
        // TODO: Func type
    }

    fn visit_trait_type_alias(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitTypeAlias, ctx: &mut TypeAliasContext) {
        let mut ty_reg = self.ctx.type_reg.write();
        let sym = ctx.sym.clone().unwrap();
        let ty = ty_reg.create_sym_path_type(sym.clone());

        let mut sym = sym.write();
        let Symbol::TypeAlias(sym) = &mut *sym else { unreachable!() };
        sym.ty = Some(ty);
    }

    fn visit_trait_const(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitConst, ctx: &mut ConstContext) {
        self.helper.visit_type(&mut node.ty);
    }

    fn visit_trait_property(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitProperty, ctx: &mut PropertyContext) {
        self.helper.visit_type(&mut node.ty);
    }

    fn visit_impl(&mut self, node: &mut Impl, ctx: &mut ImplContext) {
        self.helper.visit_type(&mut node.ty);
    }

    fn visit_impl_function(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut Function, ctx: &mut FunctionContext) {
        if let Some(generics) = &mut node.generics {
            self.helper.visit_gen_params(generics);
        }
        if let Some(where_clause) = &mut node.where_clause {
            self.helper.visit_where_clause(where_clause);
        }

        for param in &mut node.params {
            match param {
                FnParam::Param { span, attrs, label, pattern, ty } => self.visit_type(ty),
                FnParam::Opt { span, attrs, label, pattern, ty, def } => self.visit_type(ty),
                FnParam::Variadic { span, attrs, name, ty } => self.visit_type(ty),
            }
        }
        if let Some(ret_ty) = &mut node.return_ty {
            self.helper.visit_type(ret_ty);
        }
        // TODO: Func type
    }

    fn visit_method(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut Method, ctx: &mut FunctionContext) {
        if let Some(generics) = &mut node.generics {
            self.helper.visit_gen_params(generics);
        }
        if let Some(where_clause) = &mut node.where_clause {
            self.helper.visit_where_clause(where_clause);
        }
        match &mut node.receiver {
            FnReceiver::None => (),
            FnReceiver::SelfReceiver { span, is_ref, is_mut } => {
                // TODO
            },
            FnReceiver::SelfTyped { span, is_mut, ty } => {
                self.helper.visit_type(ty);
            },
        }
        for param in &mut node.params {
            match param {
                FnParam::Param { span, attrs, label, pattern, ty } => self.visit_type(ty),
                FnParam::Opt { span, attrs, label, pattern, ty, def } => self.visit_type(ty),
                FnParam::Variadic { span, attrs, name, ty } => self.visit_type(ty),
            }
        }
        if let Some(ret_ty) = &mut node.return_ty {
            self.helper.visit_type(ret_ty);
        }
        // TODO: Func type
    }

    fn visit_impl_type_alias(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut TypeAlias, ctx: &mut TypeAliasContext) {
        self.helper.visit_type(&mut node.ty);

        let mut ty_reg = self.ctx.type_reg.write();
        let sym = ctx.sym.clone().unwrap();
        let ty = ty_reg.create_sym_path_type(sym.clone());

        let mut sym = sym.write();
        let Symbol::TypeAlias(sym) = &mut *sym else { unreachable!() };
        sym.ty = Some(ty);
    }

    fn visit_impl_const(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut Const, ctx: &mut ConstContext) {
        if let Some(ty) = &mut node.ty {
            self.helper.visit_type(ty);
        }
    }

    fn visit_impl_static(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut Static, ctx: &mut StaticContext) {
        if let Some(ty) = &mut node.ty {
            self.helper.visit_type(ty);
        }
    }

    fn visit_impl_tls_static(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut TlsStatic, ctx: &mut StaticContext) {
        if let Some(ty) = &mut node.ty {
            self.helper.visit_type(ty);
        }
    }

    fn visit_property(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut Property, ctx: &mut PropertyContext) {
        if let Some(ty) = &mut node.ty {
            self.helper.visit_type(ty);
        }
    }
}

impl Pass for ItemLevelTypeGen<'_> {
    const NAME: &'static str = "Item Level Type Generation";
}