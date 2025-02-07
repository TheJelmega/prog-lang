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
    fn visit_item(&mut self, item: &Item) where Self: Sized {
        let ctx = self.ctx.get_node_for_index_mut(item.node_id().index());
        ctx.scope = self.scope.clone();
        helpers::visit_item(self, item);
    }

    fn visit_trait_item(&mut self, item: &TraitItem) where Self: Sized {
        let ctx = self.ctx.get_node_for_index_mut(item.node_id().index());
        ctx.scope = self.scope.clone();
        helpers::visit_trait_item(self, item);
    }

    fn visit_assoc_item(&mut self, item: &AssocItem) where Self: Sized {
        let ctx = self.ctx.get_node_for_index_mut(item.node_id().index());
        ctx.scope = self.scope.clone();
        helpers::visit_assoc_item(self, item);
    }

    fn visit_extern_item(&mut self, item: &ExternItem) where Self: Sized {
        let ctx = self.ctx.get_node_for_index_mut(item.node_id().index());
        ctx.scope = self.scope.clone();
        helpers::visit_extern_item(self, item);
    }

    fn visit_module(&mut self, node: &AstNodeRef<ModuleItem>) where Self: Sized {
        let name = &self.names[node.name];
        self.scope.push(name.to_string());
        helpers::visit_module(self, node);
        self.scope.pop();
    }

    // Only item that can have nested items
    fn visit_function(&mut self, node: &AstNodeRef<Function>) where Self: Sized {
        let name = self.names[node.name].to_string();

        let mut param_names = Vec::new();
        let mut anon_idx = 0;
        for param in &node.params {
            for name in &param.names {
                if let Some(label) = name.label {
                    param_names.push(self.names[label].to_string())
                } else if let Pattern::Identifier(pattern) = &name.pattern {
                    param_names.push(self.names[pattern.name].to_string());
                } else {
                    param_names.push(format!("__anon_{anon_idx}"));
                    anon_idx += 1;
                }
            }
        }

        self.scope.push_with_params(name, param_names);
        helpers::visit_function(self, node, true);
        self.scope.pop();
    }
}