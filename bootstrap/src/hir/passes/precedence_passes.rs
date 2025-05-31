use passes::PassContext;

use crate::{
    common::{NameTable, PrecedenceDAG, Symbol},
    hir::*, literals::{Literal, LiteralTable},
};

use super::Pass;

pub struct PrecedenceSymGen<'a> {
    ctx: &'a PassContext,
}

impl<'a> PrecedenceSymGen<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self { ctx, }
    }
}

impl Visitor for PrecedenceSymGen<'_> {
    fn visit_precedence(&mut self, node: &mut Precedence, ctx: Ref<PrecedenceContext>) {
        let name = self.ctx.names.read()[node.name].to_string();
        let mut ctx = ctx.write();
        let sym = self.ctx.syms.write().add_precedence(None, &ctx.scope, name);
        ctx.sym = Some(sym);
    }
}

impl Pass for PrecedenceSymGen<'_> {
    const NAME: &'static str = "Precedence Symbol Generation";

    fn process(&mut self, hir: &mut Hir) {
        self.visit(hir, VisitFlags::Precedence);
    }
}

//==============================================================================================================================

pub struct PrecedenceAttrib<'a> {
    ctx:            &'a PassContext,
    precedence_ctx: Option<Ref<PrecedenceContext>>,
}

impl<'a> PrecedenceAttrib<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx,
            precedence_ctx: None,
        }
    }
}

impl Pass for PrecedenceAttrib<'_> {
    const NAME: &'static str = "Precedence Attribute Processing";

    fn process(&mut self, hir: &mut Hir) {
        self.visit(hir, VisitFlags::Precedence);
    }
}

impl Visitor for PrecedenceAttrib<'_> {
    fn visit_precedence(&mut self, node: &mut Precedence, ctx: Ref<PrecedenceContext>) {
        self.precedence_ctx = Some(ctx.clone());
        helpers::visit_precedence(self, node);
        self.precedence_ctx = None;
    }

    fn visit_attribute(&mut self, node: &mut Attribute) {
        if node.path.names.len() != 1 {
            let mut path = String::new();
            for name in &node.path.names {
                path.push_str(&self.ctx.names.read()[*name]);
            }

            self.ctx.add_error(HirError {
                span: node.span,
                err: HirErrorCode::PrecedenceUnsupportedAttrib { info: format!("Unsupported path: {path}, only `builtin` is supported") },
            });
            return;
        }

        if &self.ctx.names.read()[node.path.names[0]] != "builtin" {
            self.ctx.add_error(HirError {
                span: node.span,
                err: HirErrorCode::PrecedenceUnsupportedAttrib { info: format!("Unsupported path: {}, only `builtin` is supported", &self.ctx.names.read()[node.path.names[0]]) },
            });
            return;
        }

        if node.metas.len() != 1 {
            self.ctx.add_error(HirError {
                span: node.span,
                err: HirErrorCode::PrecedenceUnsupportedAttrib { info: format!("Only 1 meta element is supported") },
            });
            return;
        }

        match &node.metas[0] {
            AttrMeta::Simple { path } => {
                if path.names.len() != 1 {
                    self.ctx.add_error(HirError {
                        span: node.span,
                        err: HirErrorCode::PrecedenceUnsupportedAttrib { info: format!("Only `builtin(highest_precedence)` and `builtin(lowest_precedence)` are supported") },
                    });
                    return;
                }

                match &self.ctx.names.read()[path.names[0]] {
                    "lowest_precedence" => {
                        let mut ctx = self.precedence_ctx.as_ref().unwrap().write();
                        if ctx.is_highest {
                            self.ctx.add_error(HirError {
                                span: path.span,
                                err: HirErrorCode::PrecedenceUnsupportedAttrib { info: format!("A precedence cannot be both `highest_precedence` and `lowest_precedence` at the same time") },
                            });
                        }
                        ctx.is_lowest = true;
                    },
                    "highest_precedence" => {
                        let mut ctx = self.precedence_ctx.as_ref().unwrap().write();
                        if ctx.is_lowest {
                            self.ctx.add_error(HirError {
                                span: path.span,
                                err: HirErrorCode::PrecedenceUnsupportedAttrib { info: format!("A precedence cannot be both `highest_precedence` and `lowest_precedence` at the same time") },
                            });
                        }
                        ctx.is_highest = true;
                    },
                    _ => {
                        self.ctx.add_error(HirError {
                            span: path.span,
                            err: HirErrorCode::PrecedenceUnsupportedAttrib { info: format!("Only `builtin(highest_precedence)` and `builtin(lowest_precedence)` are supported") },
                        });
                        return;
                    }
                }
            },
            _ => {
                self.ctx.add_error(HirError {
                    span: node.span,
                    err: HirErrorCode::PrecedenceUnsupportedAttrib { info: format!("Only `builtin(highest_precedence)` and `builtin(lowest_precedence)` are supported") },
                });
            },
        }
    }
}

//==============================================================================================================================

pub struct PrecedenceCollection<'a> {
    ctx: &'a PassContext
}

impl<'a> PrecedenceCollection<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx
        }
    }
}

impl Visitor for PrecedenceCollection<'_> {
    fn visit_precedence(&mut self, node: &mut Precedence, ctx: Ref<PrecedenceContext>) {
        let ctx = ctx.read();
        let mut sym = ctx.sym.as_ref().unwrap().write();
        let Symbol::Precedence(sym) = &mut *sym else { unreachable!("Precedence HIR nodes should always have Precedence symbols") };

        let mut dag = self.ctx.precedence_dag.write();

        let id = dag.add_precedence(self.ctx.names.read()[node.name].to_string());
        sym.id = id;

        if ctx.is_lowest {
            dag.set_lowest(id);
        } else if ctx.is_highest {
            dag.set_highest(id);
        }
    }
}

impl Pass for PrecedenceCollection<'_> {
    const NAME: &'static str = "Precedence Collection";
    
    fn process(&mut self, hir: &mut Hir) {
        self.visit(hir, VisitFlags::Precedence);
    }
}

//==============================================================================================================================

pub struct PrecedenceConnect<'a> {
    ctx: &'a PassContext
}

impl<'a> PrecedenceConnect<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx
        }
    }
}

impl Visitor for PrecedenceConnect<'_> {
    fn visit_precedence(&mut self, node: &mut Precedence, ctx: Ref<PrecedenceContext>) {
        let ctx = ctx.read();
        let mut sym = ctx.sym.as_ref().unwrap().write();
        let Symbol::Precedence(sym) = &mut *sym else { unreachable!("Precedence HIR nodes should always have Precedence symbols") };

        let mut dag = self.ctx.precedence_dag.write();

        if let Some((lower_than, _)) = node.lower_than {
            if ctx.is_highest {
                self.ctx.add_error(HirError {
                    span: node.span,
                    err: HirErrorCode::PrecedenceInvalidOrder { info: "Highest precedence cannot be lower than other precedences".to_string() },
                });
            } else {
                let higher = dag.get(&self.ctx.names.read()[lower_than]).unwrap();
                dag.set_order(sym.id, higher);
            }
        }

        if let Some((higher_than, _)) = node.higher_than {
            if ctx.is_highest {
                self.ctx.add_error(HirError {
                    span: node.span,
                    err: HirErrorCode::PrecedenceInvalidOrder { info: "Lowest precedence cannot be higher than other precedences".to_string() },
                });
            } else {
                let lower = dag.get(&self.ctx.names.read()[higher_than]).unwrap();
                dag.set_order(lower, sym.id)
            }
        }
    }
}

impl Pass for PrecedenceConnect<'_> {
    const NAME: &'static str = "Precedence Connecting";
}
