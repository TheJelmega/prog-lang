use super::Context;
use crate::{ast::*, common::NameTable};



pub struct ModuleScopePass<'a> {
    ctx: &'a mut Context,
    scope: Vec<String>,
    names: &'a NameTable,
}

impl<'a> ModuleScopePass<'a> {
    pub fn new(ctx: &'a mut Context, base_scope: Vec<String>, names: &'a NameTable) -> Self {
        Self {
            ctx,
            scope: base_scope,
            names, 
        }
    }
}

impl Visitor for ModuleScopePass<'_> {
    fn visit_item(&mut self, ast: &Ast, item: &Item) where Self: Sized {
        let ctx = self.ctx.get_node_for_index_mut(item.node_id());
        ctx.scope = self.scope.clone();
        helpers::visit_item(self, ast, item);
    }

    fn visit_trait_item(&mut self, ast: &Ast, item: &TraitItem) where Self: Sized {
        let ctx = self.ctx.get_node_for_index_mut(item.node_id());
        ctx.scope = self.scope.clone();
        helpers::visit_trait_item(self, ast, item);
    }

    fn visit_assoc_item(&mut self, ast: &Ast, item: &AssocItem) where Self: Sized {
        let ctx = self.ctx.get_node_for_index_mut(item.node_id());
        ctx.scope = self.scope.clone();
        helpers::visit_assoc_item(self, ast, item);
    }

    fn visit_extern_item(&mut self, ast: &Ast, item: &ExternItem) where Self: Sized {
        let ctx = self.ctx.get_node_for_index_mut(item.node_id());
        ctx.scope = self.scope.clone();
        helpers::visit_extern_item(self, ast, item);
    }

    fn visit_module(&mut self, ast: &Ast, node_id: AstNodeRef<ModuleItem>) where Self: Sized {
        let node = &ast[node_id];
        let name = &self.names[node.name];
        self.scope.push(name.to_string());
        helpers::visit_module(self, ast, node_id);
        self.scope.pop();
    }






    fn visit_stmt(&mut self, _ast: &Ast, _node: &Stmt) where Self: Sized {
        // Ignore
    }

    fn visit_expr(&mut self, _ast: &Ast, _node: &Expr) where Self: Sized {
        // Ignore
    }

    fn visit_pattern(&mut self, ast: &Ast, node: &Pattern) where Self: Sized {
        // Ignore
    }

    fn visit_type(&mut self, ast: &Ast, node: &Type) where Self: Sized {
        // Ignore
    }
}