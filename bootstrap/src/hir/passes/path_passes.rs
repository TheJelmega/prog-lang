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
}

impl<'a> PathGen<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx,
        }
    }
}

impl Visitor for PathGen<'_> {
    fn visit_path(&mut self, node: &mut Path) {
        let mut path = Scope::new();
        let names = self.ctx.names.read();

        for iden in &node.idens {
            let name = match &iden.name {
                IdenName::Name { name, span } => names[*name].to_string(),
                IdenName::Disambig { span, trait_path, name, name_span } => todo!(),
            };

            let mut args = Vec::new();
            if let Some(gen_args) = &iden.gen_args {
                for arg in &gen_args.args {
                    match arg {
                        GenericArg::Type(_) => {
                            let mut type_reg = self.ctx.type_reg.write();
                            let ty = type_reg.create_placeholder_type();
                            args.push(ScopeGenArg::Type { ty });
                        },
                        GenericArg::Value(_) => todo!(),
                        GenericArg::Name(_, _) => {
                            // TODO: For now, just assume a type, we'll fix this once we figured out type generics more
                            // We can do this by also collecting all possible variables withing the current scope, and then check which one it is
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