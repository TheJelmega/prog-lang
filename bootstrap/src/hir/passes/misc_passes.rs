use crate::{common::Scope, hir::{VisitFlags, Visitor}};

use super::{Pass, PassContext};

pub struct SimplePathGenPass<'a> {
    ctx: &'a PassContext
}

impl<'a> SimplePathGenPass<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx,
        }
    }
}

impl Visitor for SimplePathGenPass<'_> {
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
}

impl Pass for SimplePathGenPass<'_> {
    const NAME: &'static str = "Path Generation";
}


pub struct PostOpPathGenPass<'a> {
    pass: SimplePathGenPass<'a>,
}

impl<'a> PostOpPathGenPass<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            pass: SimplePathGenPass::new(ctx),
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