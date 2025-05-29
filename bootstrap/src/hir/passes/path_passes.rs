use crate::{
    ast::NodeId,
    common::{self, LibraryPath, Scope, ScopeGenArg, ScopeSegment, Symbol},
    hir::*,
};

use super::{Pass, PassContext};

pub struct SimplePathGen<'a> {
    ctx: &'a PassContext
}

impl<'a> SimplePathGen<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx,
        }
    }
}

impl Visitor for SimplePathGen<'_> {
    fn visit_simple_path(&mut self, node: &mut SimplePath) {
        if !node.ctx.path.is_empty() {
            return;
        }

        let mut path = Scope::new();
        let names = self.ctx.names.read();

        for name in &node.names {
            path.push(names[*name].to_string());
        }
        node.ctx.path = path;
    }
}

impl Pass for SimplePathGen<'_> {
    const NAME: &'static str = "Simple Path Generation";
}

//==============================================================================================================================

pub struct ImplTraitPathGen<'a> {
    ctx: &'a PassContext
}

impl<'a> ImplTraitPathGen<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx
        }
    }
}

impl Visitor for ImplTraitPathGen<'_> {
    fn visit_path(&mut self, node: &mut Path) {
        let mut path = Scope::new();
        let names = self.ctx.names.read();

        for iden in &node.idens {
            match &iden.name {
                IdenName::Name { name, span } => path.push(names[*name].to_string()),
                IdenName::Disambig { span, trait_path, name, name_span } => unreachable!("Disambiguation in trait path are disallowed in the parser"),
            }
        }

        node.ctx.path = path;
    }

    fn visit_impl(&mut self, node: &mut Impl, ctx: &mut ImplContext) {
        if let Some(path) = &mut node.impl_trait {
            self.visit_path(path);
        }
    }
}

impl Pass for ImplTraitPathGen<'_> {
    const NAME: &'static str = "Implementation Trait Path Generation";

    fn process(&mut self, hir: &mut Hir) {
        self.visit(hir, VisitFlags::Impl);
    }
}

//==============================================================================================================================

pub struct PathGen<'a> {
    ctx: &'a PassContext,
    var_info_id: VarInfoId,
    cur_scope:   Scope,
}

impl<'a> PathGen<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx,
            var_info_id: VarInfoId::INVALID,
            cur_scope: Scope::new(),
        }
    }
}

impl Visitor for PathGen<'_> {
    fn set_cur_scope(&mut self, scope: &Scope, item_name: NameId) {
        self.cur_scope = scope.clone();
    }
    
    fn set_cur_var_info_id(&mut self, id: VarInfoId) {
        self.var_info_id = id;
    }

    fn visit_path(&mut self, node: &mut Path) {
        let mut path = Scope::new();
        let names = self.ctx.names.read();

        for iden in &mut node.idens {
            let name = match &iden.name {
                IdenName::Name { name, span } => names[*name].to_string(),
                IdenName::Disambig { span, trait_path, name, name_span } => todo!(),
            };

            let mut args = Vec::new();
            if let Some(gen_args) = &mut iden.gen_args {
                for arg in &mut gen_args.args {
                    match arg {
                        GenericArg::Type(ty) => {
                            self.visit_type(ty);

                            let mut type_reg = self.ctx.type_reg.write();
                            let ty = type_reg.create_placeholder_type();
                            args.push(ScopeGenArg::Type { ty });
                        },
                        GenericArg::Value(_) => todo!(),
                        GenericArg::Name(span, name) => {
                            // First check if there is a variable (We don't resolve this yet in the HIR level, but we use this to determine if this is a value or a type generic)
                            let var_infos = self.ctx.var_infos.read();
                            let var_info = var_infos.get(self.var_info_id);
                            let var_info = var_info.read();
                            let span_registry = self.ctx.spans.read();
                            let local_var = var_info.get_var(node.ctx.var_scope, *name, *span, &span_registry);
                            if local_var.is_some() {
                                args.push(ScopeGenArg::Value);
                                continue;
                            }
                            
                            // Not a local var, so needs to be a symbol accessible to the current scope (which does not use any generics)
                            let syms = self.ctx.syms.write();
                            let uses = self.ctx.uses.read();
                            let names = self.ctx.names.read();

                            let mut sym_path = Scope::new();
                            sym_path.push(names[*name].to_string());

                            let sym = syms.get_symbol_with_uses(&uses, &self.cur_scope, None, &sym_path);
                            match sym {
                                Ok(sym) => {
                                    let sym = sym.read();
                                    if matches!(&*sym,
                                        Symbol::Const(_) |
                                        Symbol::Property(_) |
                                        Symbol::Static(_) |
                                        Symbol::ValueGeneric(_)
                                    ) {
                                        args.push(ScopeGenArg::Value);
                                        continue;
                                    }
                                },
                                Err(err) => {
                                    self.ctx.add_error(HirError {
                                        span: *span,
                                        err: HirErrorCode::UnknownSymbolOrVar { name: names[*name].to_string(), err }, 
                                    });
                                },
                            }

                            // No matter if the name doesn't correspond to a value or type, just add a type placeholder so we can at least process it until we report the error
                            let mut type_reg = self.ctx.type_reg.write();
                            let ty = type_reg.create_placeholder_type();
                            args.push(ScopeGenArg::Type { ty });
                        },
                    }
                }
            }

            path.push_segment(ScopeSegment::new(name, Vec::new(), args));
        }

        node.ctx.path = path;
    }
}

impl Pass for PathGen<'_> {
    const NAME: &'static str = "Path Generation";
}