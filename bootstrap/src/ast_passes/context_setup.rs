use crate::{ast::*, common::Scope};

use super::{Context, ContextNodeData, ModuleContextData};




pub struct ContextSetup<'a> {
    ctx: &'a mut Context
}

impl<'a> ContextSetup<'a> {
    pub fn new(ctx: &'a mut Context) -> Self {
        ContextSetup {
            ctx
        }
    }
}

impl Visitor for ContextSetup<'_> {
    fn visit_module(&mut self, _ast: &Ast, node_id: AstNodeRef<ModuleItem>) where Self: Sized {
        let node = self.ctx.get_node_for_mut(node_id);
        node.data = ContextNodeData::Module(ModuleContextData { 
            path: None,
            sym_path: Scope::new(),
        })
    }

    fn visit_precedence(&mut self, _ast: &Ast, node_id: AstNodeRef<Precedence>) where Self: Sized {
        let node = self.ctx.get_node_for_mut(node_id);
        node.data = ContextNodeData::Precedence(u16::MAX);
    }

    fn visit_binary_expr(&mut self, ast: &Ast, node_id: AstNodeRef<InfixExpr>) where Self: Sized {
        let node = self.ctx.get_node_for_mut(node_id);
        node.data = ContextNodeData::Infix { reorder: false };

        helpers::visit_binary_expr(self, ast, node_id);
    }
}