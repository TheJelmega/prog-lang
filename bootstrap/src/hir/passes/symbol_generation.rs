use passes::PassContext;

use crate::{
    common::{NameTable, RootSymbolTable, StaticKind, StructKind, SymbolTable},
    hir::*,
};


pub struct SymbolGeneration<'a> {
    ctx:           &'a PassContext,
    generic_scope: Scope,
}

impl<'a> SymbolGeneration<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx,
            generic_scope: Scope::new(),
        }
    }
}

impl Pass for SymbolGeneration<'_> {
    const NAME: &'static str = "Symbol Generation";
}

impl Visitor for SymbolGeneration<'_> {

    fn visit_function(&mut self, node: &mut Function, ctx: &mut FunctionContext) {
        let name = &self.ctx.names.read()[node.name];

        let sym = self.ctx.syms.write().add_function(None, &ctx.scope, name);
        
        if let Some(generics) = &mut node.generics {
            let sym = sym.read();
            self.generic_scope = sym.path().get_full_scope();
            let mut  uses = self.ctx.uses.write();
            uses.add_generic_use(self.generic_scope.clone());
            self.visit_gen_params(generics);
        }

        ctx.sym = Some(sym);
    }

    fn visit_extern_function_no_body(&mut self, node: &mut ExternFunctionNoBody, ctx: &mut FunctionContext) {
        let name = &self.ctx.names.read()[node.name];

        let sym = self.ctx.syms.write().add_function(None, &ctx.scope, name);
        ctx.sym = Some(sym);
    }

    fn visit_type_alias(&mut self, node: &mut TypeAlias, ctx: &mut TypeAliasContext) {
        let name = &self.ctx.names.read()[node.name];
        
        let sym = self.ctx.syms.write().add_type_alias(None, &ctx.scope, name);
        
        if let Some(generics) = &mut node.generics {
            let sym = sym.read();
            self.generic_scope = sym.path().get_full_scope();
            let mut  uses = self.ctx.uses.write();
            uses.add_generic_use(self.generic_scope.clone());
            self.visit_gen_params(generics);
        }

        ctx.sym = Some(sym);
    }

    fn visit_distinct_type(&mut self, node: &mut DistinctType, ctx: &mut TypeAliasContext) {
        let name = &self.ctx.names.read()[node.name];
        
        let sym = self.ctx.syms.write().add_distinct_type(None, &ctx.scope, name);
        
        if let Some(generics) = &mut node.generics {
            let sym = sym.read();
            self.generic_scope = sym.path().get_full_scope();
            let mut  uses = self.ctx.uses.write();
            uses.add_generic_use(self.generic_scope.clone());
            self.visit_gen_params(generics);
        }

        ctx.sym = Some(sym);
    }

    fn visit_opaque_type(&mut self, node: &mut OpaqueType, ctx: &mut TypeAliasContext) {
        let name = &self.ctx.names.read()[node.name];
        
        let sym = self.ctx.syms.write().add_opaque_type(None, &ctx.scope, name);
        ctx.sym = Some(sym);
    }

    fn visit_struct(&mut self, node: &mut Struct, ctx: &mut StructContext) {
        let name = &self.ctx.names.read()[node.name];
        
        let sym = self.ctx.syms.write().add_struct(None, &ctx.scope, name, StructKind::Normal);
        
        if let Some(generics) = &mut node.generics {
            let sym = sym.read();
            self.generic_scope = sym.path().get_full_scope();
            let mut  uses = self.ctx.uses.write();
            uses.add_generic_use(self.generic_scope.clone());
            self.visit_gen_params(generics);
        }

        ctx.sym = Some(sym);
    }

    fn visit_tuple_struct(&mut self, node: &mut TupleStruct, ctx: &mut StructContext) {
        let name = &self.ctx.names.read()[node.name];
        
        let sym = self.ctx.syms.write().add_struct(None, &ctx.scope, name, StructKind::Tuple);
        ctx.sym = Some(sym);

        if let Some(generics) = &mut node.generics {
            self.visit_gen_params(generics);
        }
    }

    fn visit_unit_struct(&mut self, node: &mut UnitStruct, ctx: &mut StructContext) {
        let name = &self.ctx.names.read()[node.name];
        
        let sym = self.ctx.syms.write().add_struct(None, &ctx.scope, name, StructKind::Unit);
        ctx.sym = Some(sym);
    }

    fn visit_union(&mut self, node: &mut Union, ctx: &mut UnionContext) {
        let name = &self.ctx.names.read()[node.name];
        
        let sym = self.ctx.syms.write().add_union(None, &ctx.scope, name);
        
        if let Some(generics) = &mut node.generics {
            let sym = sym.read();
            self.generic_scope = sym.path().get_full_scope();
            let mut  uses = self.ctx.uses.write();
            uses.add_generic_use(self.generic_scope.clone());
            self.visit_gen_params(generics);
        }

        ctx.sym = Some(sym);
    }

    fn visit_adt_enum(&mut self, node: &mut AdtEnum, ctx: &mut AdtEnumContext) {
        let name = &self.ctx.names.read()[node.name];
        
        let sym = self.ctx.syms.write().add_adt_enum(None, &ctx.scope, name);
        
        if let Some(generics) = &mut node.generics {
            let sym = sym.read();
            self.generic_scope = sym.path().get_full_scope();
            let mut  uses = self.ctx.uses.write();
            uses.add_generic_use(self.generic_scope.clone());
            self.visit_gen_params(generics);
        }

        ctx.sym = Some(sym);
    }

    fn visit_flag_enum(&mut self, node: &mut FlagEnum, ctx: &mut FlagEnumContext) {
        let name = &self.ctx.names.read()[node.name];
        
        let sym = self.ctx.syms.write().add_flag_enum(None, &ctx.scope, name);
        ctx.sym = Some(sym);
    }

    fn visit_bitfield(&mut self, node: &mut Bitfield, ctx: &mut BitfieldContext) {
        let name = &self.ctx.names.read()[node.name];
        
        let sym = self.ctx.syms.write().add_bitfield(None, &ctx.scope, name);
        
        if let Some(generics) = &mut node.generics {
            let sym = sym.read();
            self.generic_scope = sym.path().get_full_scope();
            let mut  uses = self.ctx.uses.write();
            uses.add_generic_use(self.generic_scope.clone());
            self.visit_gen_params(generics);
        }

        ctx.sym = Some(sym);
    }

    fn visit_const(&mut self, node: &mut Const, ctx: &mut ConstContext) {
        let name = &self.ctx.names.read()[node.name];
        
        let sym = self.ctx.syms.write().add_const(None, &ctx.scope, name);
        ctx.sym = Some(sym);
    }

    fn visit_static(&mut self, node: &mut Static, ctx: &mut StaticContext) {
        let name = &self.ctx.names.read()[node.name];
        
        let sym = self.ctx.syms.write().add_static(None, &ctx.scope, name, StaticKind::Normal);
        ctx.sym = Some(sym);
    }

    fn visit_tls_static(&mut self, node: &mut TlsStatic, ctx: &mut StaticContext) {
        let name = &self.ctx.names.read()[node.name];
        
        let sym = self.ctx.syms.write().add_static(None, &ctx.scope, name, StaticKind::Tls);
        ctx.sym = Some(sym);
    }

    fn visit_extern_static(&mut self, node: &mut ExternStatic, ctx: &mut StaticContext) {
        let name = &self.ctx.names.read()[node.name];
        
        let sym = self.ctx.syms.write().add_static(None, &ctx.scope, name, StaticKind::Extern);
        ctx.sym = Some(sym);
    }

    fn visit_trait(&mut self, node: &mut Trait, ctx: &mut TraitContext) {
        let name = &self.ctx.names.read()[node.name];
        
        let sym = self.ctx.syms.write().add_trait(None, &ctx.scope, name);
        
        if let Some(generics) = &mut node.generics {
            let sym = sym.read();
            self.generic_scope = sym.path().get_full_scope();
            let mut  uses = self.ctx.uses.write();
            uses.add_generic_use(self.generic_scope.clone());
            self.visit_gen_params(generics);
        }

        ctx.sym = Some(sym);
    }

    fn visit_trait_function(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitFunction, ctx: &mut FunctionContext) {
        let name = &self.ctx.names.read()[node.name];
        
        let sym = self.ctx.syms.write().add_function(None, &ctx.scope, name);
        
        if let Some(generics) = &mut node.generics {
            let sym = sym.read();
            self.generic_scope = sym.path().get_full_scope();
            let mut  uses = self.ctx.uses.write();
            uses.add_generic_use(self.generic_scope.clone());
            self.visit_gen_params(generics);
        }

        ctx.sym = Some(sym);
    }

    fn visit_trait_method(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitMethod, ctx: &mut FunctionContext) {
        let name = &self.ctx.names.read()[node.name];
        
        let sym = self.ctx.syms.write().add_function(None, &ctx.scope, name);
        
        if let Some(generics) = &mut node.generics {
            let sym = sym.read();
            self.generic_scope = sym.path().get_full_scope();
            let mut  uses = self.ctx.uses.write();
            uses.add_generic_use(self.generic_scope.clone());
            self.visit_gen_params(generics);
        }

        ctx.sym = Some(sym);
    }

    fn visit_trait_type_alias(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitTypeAlias, ctx: &mut TypeAliasContext) {
        let name = &self.ctx.names.read()[node.name];
        
        let sym = self.ctx.syms.write().add_type_alias(None, &ctx.scope, name);
        
        if let Some(generics) = &mut node.generics {
            let sym = sym.read();
            self.generic_scope = sym.path().get_full_scope();
            let mut  uses = self.ctx.uses.write();
            uses.add_generic_use(self.generic_scope.clone());
            self.visit_gen_params(generics);
        }

        ctx.sym = Some(sym);
    }

    fn visit_trait_const(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitConst, ctx: &mut ConstContext) {
        let name = &self.ctx.names.read()[node.name];
        
        let sym = self.ctx.syms.write().add_const(None, &ctx.scope, name);
        ctx.sym = Some(sym);
    }

    fn visit_trait_property(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitProperty, ctx: &mut PropertyContext) {
        let name = &self.ctx.names.read()[node.name];
        
        let sym = self.ctx.syms.write().add_property(None, &ctx.scope, name);
        ctx.sym = Some(sym);
    }

    fn visit_impl(&mut self, node: &mut Impl, ctx: &mut ImplContext) {
        let name = &self.ctx.names.read()[ctx.name];

        let sym = self.ctx.syms.write().add_impl(None, &ctx.scope, name);
        
        if let Some(generics) = &mut node.generics {
            let sym = sym.read();
            self.generic_scope = sym.path().get_full_scope();
            let mut  uses = self.ctx.uses.write();
            uses.add_generic_use(self.generic_scope.clone());
            self.visit_gen_params(generics);
        }

        ctx.sym = Some(sym);
    }

    fn visit_impl_function(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut Function, ctx: &mut FunctionContext) {
        let name = &self.ctx.names.read()[node.name];
        
        let sym = self.ctx.syms.write().add_function(None, &ctx.scope, name);
        
        if let Some(generics) = &mut node.generics {
            let sym = sym.read();
            self.generic_scope = sym.path().get_full_scope();
            let mut  uses = self.ctx.uses.write();
            uses.add_generic_use(self.generic_scope.clone());
            self.visit_gen_params(generics);
        }

        ctx.sym = Some(sym);
    }

    fn visit_method(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut Method, ctx: &mut FunctionContext) {
        let name = &self.ctx.names.read()[node.name];
        
        let sym = self.ctx.syms.write().add_function(None, &ctx.scope, name);
        
        if let Some(generics) = &mut node.generics {
            let sym = sym.read();
            self.generic_scope = sym.path().get_full_scope();
            let mut  uses = self.ctx.uses.write();
            uses.add_generic_use(self.generic_scope.clone());
            self.visit_gen_params(generics);
        }

        ctx.sym = Some(sym);
    }

    fn visit_impl_type_alias(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut TypeAlias, ctx: &mut TypeAliasContext) {
        let name = &self.ctx.names.read()[node.name];
        
        let sym = self.ctx.syms.write().add_type_alias(None, &ctx.scope, name);
        
        if let Some(generics) = &mut node.generics {
            let sym = sym.read();
            self.generic_scope = sym.path().get_full_scope();
            let mut  uses = self.ctx.uses.write();
            uses.add_generic_use(self.generic_scope.clone());
            self.visit_gen_params(generics);
        }

        ctx.sym = Some(sym);
    }

    fn visit_impl_const(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut Const, ctx: &mut ConstContext) {
        let name = &self.ctx.names.read()[node.name];
        
        let sym = self.ctx.syms.write().add_const(None, &ctx.scope, name);
        ctx.sym = Some(sym);
    }

    fn visit_impl_static(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut Static, ctx: &mut StaticContext) {
        let name = &self.ctx.names.read()[node.name];
        
        let sym = self.ctx.syms.write().add_static(None, &ctx.scope, name, StaticKind::Normal);
        ctx.sym = Some(sym);
    }

    fn visit_impl_tls_static(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut TlsStatic, ctx: &mut StaticContext) {
        let name = &self.ctx.names.read()[node.name];
        
        let sym = self.ctx.syms.write().add_static(None, &ctx.scope, name, StaticKind::Tls);
        ctx.sym = Some(sym);
    }

    fn visit_property(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut Property, ctx: &mut PropertyContext) {
        let name = &self.ctx.names.read()[node.name];
        
        let sym = self.ctx.syms.write().add_property(None, &ctx.scope, name);
        ctx.sym = Some(sym);
    }

    fn visit_op_trait(&mut self, node: &mut OpTrait, ctx: &mut OpTraitContext) {
        let name = &self.ctx.names.read()[node.name];

        let sym = self.ctx.syms.write().add_trait(None, &ctx.scope, name);
        ctx.sym = Some(sym);
    }

    fn visit_op_function(&mut self, op_trait_ref: Ref<OpTrait>, op_trait_ctx: Ref<OpTraitContext>, node: &mut OpFunction, ctx: &mut OpFunctionContext) {
        let name = &self.ctx.names.read()[node.name];

        let sym = self.ctx.syms.write().add_function(None, &ctx.scope, name);
        ctx.sym = Some(sym);
    }

    fn visit_precedence(&mut self, node: &mut Precedence, ctx: Ref<PrecedenceContext>) {
        let name = &self.ctx.names.read()[node.name];

        let mut ctx = ctx.write();
        let sym = self.ctx.syms.write().add_precedence(None, &ctx.scope, name);
        ctx.sym = Some(sym);
    }

    fn visit_gen_params(&mut self, node: &mut GenericParams) {
        for param in &mut node.params {
            match param {
                GenericParam::Type(param) => {
                    let names = self.ctx.names.read();
                    let mut syms = self.ctx.syms.write();
                    let sym = syms.add_type_generic(None, &self.generic_scope, &names[param.name], false);

                    param.ctx.sym = Some(sym);
                },
                GenericParam::TypeSpec(_) => (),
                GenericParam::Const(param) => {
                    let names = self.ctx.names.read();
                    let mut syms = self.ctx.syms.write();
                    let sym = syms.add_value_generic(None, &self.generic_scope, &names[param.name], false);

                    param.ctx.sym = Some(sym);

                },
                GenericParam::ConstSpec(_) => (),
            }
        }

        // TODO: Param Pack
        if let Some(pack) = &mut node.pack {
            for elem in &mut pack.elems {
                match elem {
                    GenericParamPackElem::Type { name, name_span, ty_span, defs, ctx } => {
                        let names = self.ctx.names.read();
                        let mut syms = self.ctx.syms.write();
                        let sym = syms.add_type_generic(None, &self.generic_scope, &names[*name], true);
                    },
                    GenericParamPackElem::Const { name, name_span, ty, defs, ctx } => {
                        let names = self.ctx.names.read();
                        let mut syms = self.ctx.syms.write();
                        let sym = syms.add_value_generic(None, &self.generic_scope, &names[*name], true);
                    },
                }
            }
        }
    }


    fn visit_stmt(&mut self, stmt: &mut Stmt) {
        // Skip
    }

    fn visit_expr(&mut self, expr: &mut Expr) {
        // Skip
    }

    fn visit_type(&mut self, node: &mut Type) {
        // Skip
    }
}

pub struct TypeImplSymbolAssoc<'a> {
    ctx: &'a PassContext
}

impl<'a> TypeImplSymbolAssoc<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx,
        }
    }
}


impl Visitor for TypeImplSymbolAssoc<'_> {
    fn visit_impl(&mut self, node: &mut Impl, ctx: &mut ImplContext) {
        let sym = ctx.sym.as_ref().unwrap().clone();
        let ty = node.ty.ctx().ty.as_ref().unwrap().clone();
        
        let mut sym_table = self.ctx.syms.write();
        sym_table.associate_impl_with_ty(ty, sym);
    }
}

impl Pass for TypeImplSymbolAssoc<'_> {
    const NAME: &'static str = "Type <-> Impl Symbol Association Pass";

    fn process(&mut self, hir: &mut Hir) {
        self.visit(hir, VisitFlags::Impl);
    }
}