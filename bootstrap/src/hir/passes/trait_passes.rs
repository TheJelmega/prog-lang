use std::collections::HashMap;

use crate::{
    common::{Scope, Symbol},
    error_warning::HirErrorCode,
    hir::{HirError, Visitor}
};

use super::{Pass, PassContext};


pub struct TraitDagGen<'a> {
    ctx: &'a PassContext
}

impl<'a> TraitDagGen<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx,
        }
    }
}



impl Pass for TraitDagGen<'_> {
    const NAME: &'static str = "Trait DAG Generation";

    fn process(&mut self, hir: &mut crate::hir::Hir) {
        let mut dag = self.ctx.trait_dag.write();

        // Collect trait symbols and add them to the DAG
        for (node, ctx) in &mut hir.traits {
            let mut ctx = ctx.write();
            
            let sym = ctx.sym.as_ref().unwrap().clone();
            let idx = dag.add(sym.clone());
            ctx.dag_idx = idx;

            let mut sym = sym.write();
            let Symbol::Trait(sym) = &mut *sym else {
                self.ctx.add_error(HirError {
                    node_id: node.read().node_id,
                    err: HirErrorCode::InternalError("Trait does not have trait symbol associated with it"),
                });
                continue;
            };
            sym.dag_idx = idx;
        }

        for (node, ctx) in &mut hir.op_traits {
            let mut ctx = ctx.write();
            
            let sym = ctx.sym.as_ref().unwrap().clone();
            let idx = dag.add(sym.clone());
            ctx.dag_idx = idx;

            let mut sym = sym.write();
            let Symbol::Trait(sym) = &mut *sym else {
                self.ctx.add_error(HirError {
                    node_id: node.read().node_id,
                    err: HirErrorCode::InternalError("Trait does not have trait symbol associated with it"),
                });
                continue;
            };
            sym.dag_idx = idx;
        }

        let names = self.ctx.names.read();
        let syms = self.ctx.syms.read();
        let uses = self.ctx.uses.read();

        // Now set dependencies
        for (node, ctx) in &mut hir.traits {
            let node = node.read();
            let ctx = ctx.read();

            if let Some(bound) = &node.bounds {
                for path in &bound.bounds {
                    let scope = &path.ctx.path;
                    let Some(sym) = syms.get_symbol_with_uses(&uses, &ctx.scope, None, scope) else {
                        self.ctx.add_error(HirError {
                            node_id: bound.node_id,
                            err: HirErrorCode::UnknownSymbol { path: scope.clone() },
                        });
                        continue;
                    };
                    let mut sym = sym.write();
                    let Symbol::Trait(sym) = &mut *sym else {
                        self.ctx.add_error(HirError {
                            node_id: bound.node_id,
                            err: HirErrorCode::InternalError("Trait does not have trait symbol associated with it"),
                        });
                        continue;
                    };

                    dag.set_base_dependency(ctx.dag_idx, sym.dag_idx);
                }
            }
        }

        for (node, ctx) in &mut hir.op_traits {
            let node = node.read();
            let ctx = ctx.read();

            for path in &node.bases {
                let scope = &path.ctx.path;

                let Some(sym) = syms.get_symbol_with_uses(&uses, &ctx.scope, None, scope) else {
                    self.ctx.add_error(HirError {
                        node_id: node.node_id,
                        err: HirErrorCode::UnknownSymbol { path: scope.clone() },
                    });
                    continue;
                };
                let mut sym = sym.write();
                let Symbol::Trait(sym) = &mut *sym else {
                    self.ctx.add_error(HirError {
                        node_id: node.node_id,
                        err: HirErrorCode::InternalError("Trait does not have trait symbol associated with it"),
                    });
                    continue;
                };
            
                dag.set_base_dependency(ctx.dag_idx, sym.dag_idx);
            }
        }
    }
}

impl Visitor for TraitDagGen<'_> {

}