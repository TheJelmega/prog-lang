use super::Context;
use crate::{ast::*, common::{NameTable, Scope}};



pub struct ModuleScopePass<'a> {
    ctx: &'a mut Context,
    scope: Scope,
    names: &'a NameTable,
}

impl<'a> ModuleScopePass<'a> {
    pub fn new(ctx: &'a mut Context, base_scope: Scope, names: &'a NameTable) -> Self {
        Self {
            ctx,
            scope: base_scope,
            names, 
        }
    }
}

impl Visitor for ModuleScopePass<'_> {
    fn visit_item(&mut self, ast: &Ast, item: &Item) where Self: Sized {
        let ctx = self.ctx.get_node_for_index_mut(item.node_id(ast).index());
        ctx.scope = self.scope.clone();
        helpers::visit_item(self, ast, item);
    }

    fn visit_trait_item(&mut self, ast: &Ast, item: &TraitItem) where Self: Sized {
        let ctx = self.ctx.get_node_for_index_mut(item.node_id(ast).index());
        ctx.scope = self.scope.clone();
        helpers::visit_trait_item(self, ast, item);
    }

    fn visit_assoc_item(&mut self, ast: &Ast, item: &AssocItem) where Self: Sized {
        let ctx = self.ctx.get_node_for_index_mut(item.node_id(ast).index());
        ctx.scope = self.scope.clone();
        helpers::visit_assoc_item(self, ast, item);
    }

    fn visit_extern_item(&mut self, ast: &Ast, item: &ExternItem) where Self: Sized {
        let ctx = self.ctx.get_node_for_index_mut(item.node_id(ast).index());
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

    // Only item that can have nested items
    fn visit_function(&mut self, ast: &Ast, node_id: AstNodeRef<Function>) where Self: Sized {
        let node = &ast[node_id];
        let name = self.names[node.name].to_string();

        let mut param_names = Vec::new();
        let mut anon_idx = 0;
        for param in &node.params {
            for name in &param.names {
                if let Some(label) = name.label {
                    param_names.push(self.names[label].to_string())
                } else if let Pattern::Identifier(pattern_id) = name.pattern {
                    let pattern = &ast[pattern_id];
                    param_names.push(self.names[pattern.name].to_string());
                } else {
                    param_names.push(format!("__anon_{anon_idx}"));
                    anon_idx += 1;
                }
            }
        }

        self.scope.push_with_params(name, param_names);
        helpers::visit_function(self, ast, node_id, true);
        self.scope.pop();
    }
}