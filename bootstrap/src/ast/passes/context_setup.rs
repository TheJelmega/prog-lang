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
    fn visit_module(&mut self, node_id: &AstNodeRef<ModuleItem>) where Self: Sized {
        let node = self.ctx.get_node_for_mut(node_id);
        node.data = ContextNodeData::Module(ModuleContextData { 
            path: None,
            sym_path: Scope::new(),
        })
    }
}