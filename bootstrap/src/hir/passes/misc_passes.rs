use crate::{common::Scope, hir::{TypePathSegment, VisitFlags, Visitor}};

use super::{Pass, PassContext};

pub struct PathGenPass<'a> {
    ctx: &'a PassContext
}

impl<'a> PathGenPass<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx,
        }
    }
}

impl Visitor for PathGenPass<'_> {
    fn visit_type_path(&mut self, node: &mut crate::hir::TypePath) {
        if !node.ctx.path.is_empty() {
            return;
        }

        let mut path = Scope::new();
        let names = self.ctx.names.read();

        for segment in &node.segments {
            match segment {
                TypePathSegment::Plain { name, .. } => path.push(names[*name].to_string()),
                TypePathSegment::GenArg { span, name, gen_args } => todo!(),
                TypePathSegment::Fn { span, name, params, ret } => todo!(),
            }
        }
        node.ctx.path = path;
    }

    fn visit_simple_path(&mut self, node: &mut crate::hir::SimplePath) {
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

    fn visit_path(&mut self, node: &mut crate::hir::Path) {
        if !node.ctx.path.is_empty() {
            return;
        }

        // TODO
    }

    fn visit_qual_path(&mut self, node: &mut crate::hir::QualifiedPath) {
        if !node.ctx.path.is_empty() {
            return;
        }

        // TODO
    }
}

impl Pass for PathGenPass<'_> {
    const NAME: &'static str = "Path Generation";
}



pub struct PostOpPathGenPass<'a> {
    pass: PathGenPass<'a>,
}

impl<'a> PostOpPathGenPass<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            pass: PathGenPass::new(ctx),
        }
    }
}

impl Visitor for PostOpPathGenPass<'_> {}
impl Pass for PostOpPathGenPass<'_> {
    const NAME: &'static str = "Post Operator Path Gen";

    fn process(&mut self, hir: &mut crate::hir::Hir) {
        self.pass.visit(hir, VisitFlags::Trait | VisitFlags::TraitMethod | VisitFlags::TraitTypeAlias);
    }
}