use crate::{
    ast::*, common::{NameId, NameTable}, error_warning::{AstErrorCode, LexErrorCode}, literals::{Literal, LiteralTable}
};

use super::{AstError, Context, ContextNodeData};

pub struct ModuleAttributeResolver<'a> {
    ctx:          &'a mut Context,
    lit_table:    &'a LiteralTable,
    path_name_id: NameId,
}

impl<'a> ModuleAttributeResolver<'a> {
    pub fn new(ctx: &'a mut Context, name_table: &NameTable, lit_table: &'a LiteralTable) -> Self {
        let path_name_id = name_table.get_id_for_str("path");
        Self {
            ctx,
            lit_table,
            path_name_id,
        }
    }
}

impl Visitor for ModuleAttributeResolver<'_> {

    // Limit visiting only to root accessible items
    fn visit_item(&mut self, item: &Item) where Self: Sized {
        if let Item::Module(module) = item {
            self.visit_module(module);
        }
    }

    fn visit_module(&mut self, node: &AstNodeRef<ModuleItem>) where Self: Sized {
        for attr in &node.attrs {
            for meta in &attr.metas {
                match meta {
                    AttribMeta::Simple { .. } => {
                        self.ctx.add_error(AstError {
                            node_id: node.node_id(),
                            err: AstErrorCode::InvalidAttribute { info: format!("Modules may not have simple attributes") },
                        })
                    },
                    AttribMeta::Expr { .. } => {
                        self.ctx.add_error(AstError {
                            node_id: node.node_id(),
                            err: AstErrorCode::InvalidAttribute { info: format!("Modules may not have expression-only attributes") },
                        })
                    },
                    AttribMeta::Assign { span, node_id, path, expr } => {
                        if path.names.len() == 1 || path.names[0].0 == self.path_name_id {
                            let Expr::Literal(lit_node) = expr else { 
                                self.ctx.add_error(AstError {
                                    node_id: node.node_id(),
                                    err: AstErrorCode::InvalidAttributeData { info: format!("Path attribute only accepts string literals") },
                                });
                                continue;
                            };

                            let LiteralValue::Lit(lit_id) = lit_node.literal else { 
                                self.ctx.add_error(AstError {
                                    node_id: node.node_id(),
                                    err: AstErrorCode::InvalidAttributeData { info: format!("Path attribute only accepts string literals") },
                                });
                                continue;
                            };

                            let path = {
                                let lit = &self.lit_table[lit_id];
                                match lit {
                                    Literal::String(path) => Some(path.into()),
                                    _ => {
                                        self.ctx.add_error(AstError {
                                            node_id: node.node_id(),
                                            err: AstErrorCode::InvalidAttributeData { info: format!("Path attribute only accepts string literals") },
                                        });
                                        None
                                    },
                                }
                            };
                            
                            let ctx_node = self.ctx.get_node_for_mut(node);
                            let ContextNodeData::Module(module_data) = &mut ctx_node.data else { unreachable!() };
                            module_data.path = path;
                        }
                        
                    },
                    AttribMeta::Meta { .. } => {
                        self.ctx.add_error(AstError {
                            node_id: node.node_id(),
                            err: AstErrorCode::InvalidAttribute { info: format!("Modules may not have nested attributes") },
                        })
                    },
                }
            }

            helpers::visit_module(self, node);
        }
    }
}
