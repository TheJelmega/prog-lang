use crate::{
    common::{NameTable, PrecedenceDAG, Symbol},
    hir::*, literals::{Literal, LiteralTable},
};

use super::Pass;


pub struct PrecedenceAttrib<'a> {
    names:  &'a NameTable,
    lits:   &'a LiteralTable,
    ctx:    Option<Ref<PrecedenceContext>>,
    errors: &'a mut Vec<HirError>,
}

impl<'a> PrecedenceAttrib<'a> {
    pub fn new(names: &'a NameTable, lits: &'a LiteralTable, errors: &'a mut Vec<HirError>) -> Self {
        Self {
            names,
            lits,
            ctx: None,
            errors,
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
        self.ctx = Some(ctx.clone());
        helpers::visit_precedence(self, node);
        self.ctx = None;
    }

    fn visit_attribute(&mut self, node: &mut Attribute) {
        if node.path.names.len() != 1 {
            let mut path = String::new();
            for name in &node.path.names {
                path.push_str(&self.names[*name]);
            }

            self.errors.push(HirError {
                node_id: node.node_id,
                err: HirErrorCode::PrecedenceUnsupportedAttrib { info: format!("Unsupported path: {path}, only `builtin` is supported") },
            });
            return;
        }

        if &self.names[node.path.names[0]] != "builtin" {
            self.errors.push(HirError {
                node_id: node.node_id,
                err: HirErrorCode::PrecedenceUnsupportedAttrib { info: format!("Unsupported path: {}, only `builtin` is supported", &self.names[node.path.names[0]]) },
            });
            return;
        }

        if node.metas.len() != 1 {
            self.errors.push(HirError {
                node_id: node.node_id,
                err: HirErrorCode::PrecedenceUnsupportedAttrib { info: format!("Only 1 meta element is supported") },
            });
            return;
        }

        match &node.metas[0] {
            AttrMeta::Simple { path } => {
                if path.names.len() != 1 {
                    self.errors.push(HirError {
                        node_id: node.node_id,
                        err: HirErrorCode::PrecedenceUnsupportedAttrib { info: format!("Only `builtin(highest_precedence)` and `builtin(lowest_precedence)` are supported") },
                    });
                    return;
                }

                match &self.names[path.names[0]] {
                    "lowest_precedence" => {
                        let mut ctx = self.ctx.as_ref().unwrap().write();
                        if ctx.is_highest {
                            self.errors.push(HirError {
                                node_id: path.node_id,
                                err: HirErrorCode::PrecedenceUnsupportedAttrib { info: format!("A precedence cannot be both `highest_precedence` and `lowest_precedence` at the same time") },
                            });
                        }
                        ctx.is_lowest = true;
                    },
                    "highest_precedence" => {
                        let mut ctx = self.ctx.as_ref().unwrap().write();
                        if ctx.is_lowest {
                            self.errors.push(HirError {
                                node_id: path.node_id,
                                err: HirErrorCode::PrecedenceUnsupportedAttrib { info: format!("A precedence cannot be both `highest_precedence` and `lowest_precedence` at the same time") },
                            });
                        }
                        ctx.is_highest = true;
                    },
                    _ => {
                        self.errors.push(HirError {
                            node_id: path.node_id,
                            err: HirErrorCode::PrecedenceUnsupportedAttrib { info: format!("Only `builtin(highest_precedence)` and `builtin(lowest_precedence)` are supported") },
                        });
                        return;
                    }
                }
            },
            _ => {
                self.errors.push(HirError {
                    node_id: node.node_id,
                    err: HirErrorCode::PrecedenceUnsupportedAttrib { info: format!("Only `builtin(highest_precedence)` and `builtin(lowest_precedence)` are supported") },
                });
            },
        }
    }
}

pub struct PrecedenceCollection<'a> {
    dag:   &'a mut PrecedenceDAG,
    names: &'a NameTable, 
}

impl<'a> PrecedenceCollection<'a> {
    pub fn new(dag: &'a mut PrecedenceDAG, names: &'a NameTable) -> Self {
        Self {
            dag,
            names
        }
    }
}

impl Pass for PrecedenceCollection<'_> {
    const NAME: &'static str = "Precedence Collection";
    
    fn process(&mut self, hir: &mut Hir) {
        self.visit(hir, VisitFlags::Precedence);
    }
}

impl Visitor for PrecedenceCollection<'_> {
    fn visit_precedence(&mut self, node: &mut Precedence, ctx: Ref<PrecedenceContext>) {
        let ctx = ctx.read();
        let mut sym = ctx.sym.as_ref().unwrap().write();
        let Symbol::Precedence(sym) = &mut *sym else { unreachable!("Precedence HIR nodes should always have Precedence symbols") };

        let id = self.dag.add_precedence(self.names[node.name].to_string());
        sym.id = id;

        if ctx.is_lowest {
            self.dag.set_lowest(id);
        } else if ctx.is_highest {
            self.dag.set_highest(id);
        }
    }
}

pub struct PrecedenceConnect<'a> {
    names:  &'a NameTable,
    dag:    &'a mut PrecedenceDAG,
    errors: &'a mut Vec<HirError>,
}

impl<'a> PrecedenceConnect<'a> {
    pub fn new(names: &'a NameTable, dag: &'a mut PrecedenceDAG, errors: &'a mut Vec<HirError>) -> Self {
        Self {
            names,
            dag,
            errors,
        }
    }
}

impl Pass for PrecedenceConnect<'_> {
    const NAME: &'static str = "Precedence Connecting";
}

impl Visitor for PrecedenceConnect<'_> {
    fn visit_precedence(&mut self, node: &mut Precedence, ctx: Ref<PrecedenceContext>) {
        let ctx = ctx.read();
        let mut sym = ctx.sym.as_ref().unwrap().write();
        let Symbol::Precedence(sym) = &mut *sym else { unreachable!("Precedence HIR nodes should always have Precedence symbols") };

        if let Some((lower_than, _)) = node.lower_than {
            if ctx.is_highest {
                self.errors.push(HirError {
                    node_id: node.node_id,
                    err: HirErrorCode::PrecedenceInvalidOrder { info: "Highest precedence cannot be lower than other precedences".to_string() },
                });
            } else {
                let higher = self.dag.get_id(&self.names[lower_than]);
                self.dag.set_order(sym.id, higher);
            }
        }

        if let Some((higher_than, _)) = node.higher_than {
            if ctx.is_highest {
                self.errors.push(HirError {
                    node_id: node.node_id,
                    err: HirErrorCode::PrecedenceInvalidOrder { info: "Lowest precedence cannot be higher than other precedences".to_string() },
                });
            } else {
                let lower = self.dag.get_id(&self.names[higher_than]);
                self.dag.set_order(lower, sym.id)
            }
        }
    }
}
