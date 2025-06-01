use passes::PassContext;

use crate::{
    common::{NameTable, PrecedenceDAG, PrecedenceOrderKind, Symbol},
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

        let kind = if ctx.is_highest {
            PrecedenceOrderKind::Highest
        } else if ctx.is_lowest {
            PrecedenceOrderKind::Lowest
        } else {
            PrecedenceOrderKind::User
        };
        let assoc = node.assoc.as_ref().map_or(PrecedenceAssocKind::None, |assoc| assoc.kind);

        let sym = self.ctx.syms.write().add_precedence(None, &ctx.scope, name, kind, assoc);
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
    sym: Option<SymbolRef>,
}

impl<'a> PrecedenceAttrib<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx,
            sym: None,
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
        self.sym = Some(ctx.read().sym.clone().unwrap());
        helpers::visit_precedence(self, node);
        self.sym = None;
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

                let kind = match &self.ctx.names.read()[path.names[0]] {
                    "lowest_precedence"  => PrecedenceOrderKind::Lowest,
                    "highest_precedence" => PrecedenceOrderKind::Highest,
                    _ => {
                        self.ctx.add_error(HirError {
                            span: path.span,
                            err: HirErrorCode::PrecedenceUnsupportedAttrib { info: format!("Only `builtin(highest_precedence)` and `builtin(lowest_precedence)` are supported") },
                        });
                        return;
                    }
                };

                let mut sym = self.sym.as_ref().unwrap().write();
                let Symbol::Precedence(sym) = &mut *sym else { unreachable!() };
                if sym.order_kind != PrecedenceOrderKind::User {
                    self.ctx.add_error(HirError {
                        span: path.span,
                        err: HirErrorCode::PrecedenceUnsupportedAttrib { info: format!("A precedence cannot be both `highest_precedence` and `lowest_precedence` at the same time") },
                    });
                }
                sym.order_kind = kind;
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

        let syms = self.ctx.syms.read();
        let uses = self.ctx.uses.read();
        let names = self.ctx.names.read();
        
        if let Some((lower_than, span)) = node.lower_than {
            if sym.order_kind == PrecedenceOrderKind::Highest {
                // Pretty niche error that only really happens for the core libary, as it contains the corresponding language item
                self.ctx.add_error(HirError {
                    span,
                    err: HirErrorCode::PrecedenceInvalidOrder { info: "Highest precedence cannot be lower than other precedences".to_string() },
                })
            } else {
                let lower_than_name = &names[lower_than];
                match syms.get_precedence(&uses, lower_than_name) {
                    Ok(lower) => sym.lower_than = Some(Arc::downgrade(&lower)),
                    Err(err) => self.ctx.add_error(HirError {
                        span,
                        err: HirErrorCode::UnknownSymbol { err },
                    }),
                };
            }
        }

        if let Some((higher_than, span)) = node.higher_than {
            if sym.order_kind == PrecedenceOrderKind::Lowest {
                // Pretty niche error that only really happens for the core libary, as it contains the corresponding language item
                self.ctx.add_error(HirError {
                    span,
                    err: HirErrorCode::PrecedenceInvalidOrder { info: "Lowest precedence cannot be higher than other precedences".to_string() },
                })
            } else {
                let higher_than_name = &names[higher_than];
                match syms.get_precedence(&uses, higher_than_name) {
                    Ok(lower) => sym.higher_than = Some(Arc::downgrade(&lower)),
                    Err(err) => self.ctx.add_error(HirError {
                        span,
                        err: HirErrorCode::UnknownSymbol { err },
                    }),
                };
            }
        }
    }
}

impl Pass for PrecedenceConnect<'_> {
    const NAME: &'static str = "Precedence Connecting";
}
