use crate::{
    ast::*, common::{NameId, NameTable}, error_warning::ErrorCode, literals::{Literal, LiteralTable}
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
    fn visit_item(&mut self, ast: &Ast, item: &Item) where Self: Sized {
        if let Item::Module(module) = item {
            self.visit_module(ast, *module);
        }
    }

    fn visit_module(&mut self, ast: &Ast, node_id: AstNodeRef<ModuleItem>) where Self: Sized {
        let node = &ast[node_id];
        for attr in &node.attrs {
            for meta in &ast[*attr].metas {
                match meta {
                    AttribMeta::Simple { .. } => {
                        self.ctx.add_error(AstError {
                            node_id: node_id.index(),
                            err: ErrorCode::AstInvalidAttribute { info: format!("Modules may not have simple attributes") },
                        })
                    },
                    AttribMeta::Expr { .. } => {
                        self.ctx.add_error(AstError {
                            node_id: node_id.index(),
                            err: ErrorCode::AstInvalidAttribute { info: format!("Modules may not have expression-only attributes") },
                        })
                    },
                    AttribMeta::Assign { path, expr } => {
                        let path = &ast[*path];
                        
                        if path.names.len() == 1 || path.names[0] == self.path_name_id {
                            let Expr::Literal(lit_node_id) = expr else { 
                                self.ctx.add_error(AstError {
                                    node_id: node_id.index(),
                                    err: ErrorCode::AstInvalidAttributeData { info: format!("Path attribute only accepts string literals") },
                                });
                                continue;
                            };

                            let LiteralValue::Lit(lit_id) = ast[*lit_node_id].literal else { 
                                self.ctx.add_error(AstError {
                                    node_id: node_id.index(),
                                    err: ErrorCode::AstInvalidAttributeData { info: format!("Path attribute only accepts string literals") },
                                });
                                continue;
                            };

                            let path = {
                                let lit = &self.lit_table[lit_id];
                                match lit {
                                    Literal::String(path) => Some(path.into()),
                                    _ => {
                                        self.ctx.add_error(AstError {
                                            node_id: node_id.index(),
                                            err: ErrorCode::AstInvalidAttributeData { info: format!("Path attribute only accepts string literals") },
                                        });
                                        None
                                    },
                                }
                            };
                            
                            let ctx_node = self.ctx.get_node_for_mut(node_id);
                            let ContextNodeData::Module(module_data) = &mut ctx_node.data else { unreachable!() };
                            module_data.path = path;
                        }
                        
                    },
                    AttribMeta::Meta { .. } => {
                        self.ctx.add_error(AstError {
                            node_id: node_id.index(),
                            err: ErrorCode::AstInvalidAttribute { info: format!("Modules may not have nested attributes") },
                        })
                    },
                }
            }

            helpers::visit_module(self, ast, node_id);
        }
    }
}
