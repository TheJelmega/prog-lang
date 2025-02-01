use crate::{
    ast::*,
    common::{NameTable, OpType, OperatorImportPath, OperatorInfo},
    error_warning::ErrorCode,
    lexer::PuncutationTable
};

use super::{AstError, Context, ContextNodeData};


pub struct OperatorCollection<'a> {
    ctx:    &'a Context,
    names:  &'a NameTable,
}

impl<'a> OperatorCollection<'a> {
    pub fn new(ctx: &'a Context, names: &'a NameTable) -> Self {
        Self {
            ctx,
            names,
        }
    }
}

impl Visitor for OperatorCollection<'_> {
    fn visit_op_trait(&mut self, ast: &Ast, node_id: AstNodeRef<OpTrait>) where Self: Sized {
        let node = &ast[node_id];

        let ctx_node = self.ctx.get_node_for(node_id);

        let mut trait_path = ctx_node.scope.clone();
        
        match node {
            OpTrait::Base { attrs: _, vis: _, name, precedence, elems } => {
                trait_path.push(self.names[*name].to_string());

                for elem in elems {
                    match elem {
                        OpElem::Def { op_type, op, name, .. } => {
                            let op_info = OperatorInfo {
                                op_type: *op_type,
                                op: *op,
                                precedence: precedence.map(|prec| self.names[prec].to_string()),
                                library_path: self.ctx.lib_path.clone(),
                                trait_path: trait_path.clone(),
                                func_name: self.names[*name].to_string(),
                            };
                            self.ctx.operators.write().add_operator(op_info);
                        },
                        _ => {},
                    }
                }
            },
            OpTrait::Extended { name, elems, .. } => {
                trait_path.push(self.names[*name].to_string());

                for elem in elems {
                    match elem {
                        OpElem::Def { op_type, op, name, .. } => {
                            let op_info = OperatorInfo {
                                op_type: *op_type,
                                op: *op,
                                precedence: None,
                                library_path: self.ctx.lib_path.clone(),
                                trait_path: trait_path.clone(),
                                func_name: self.names[*name].to_string(),
                            };
                            self.ctx.operators.write().add_operator(op_info);
                        },
                        _ => {},
                    }
                }
            },
            
        }
    }
}

pub struct OperatorImport<'a> {
    ctx:         &'a Context,
    names:       &'a NameTable,
    top_level:   bool,
    pub imports: Vec<OperatorImportPath>,
}

impl<'a> OperatorImport<'a> {
    pub fn new(ctx: &'a Context, names: &'a NameTable) -> Self {
        let top_level = ctx.mod_root.is_empty();
        Self {
            ctx,
            names,
            top_level,
            imports: Vec::new(),
        }
    }
}

impl Visitor for OperatorImport<'_> {
    fn visit_module(&mut self, ast: &Ast, node_id: AstNodeRef<ModuleItem>) where Self: Sized {
        self.top_level = false;
        helpers::visit_module(self, ast, node_id);
    }

    fn visit_op_use(&mut self, ast: &Ast, node_id: AstNodeRef<OpUse>) where Self: Sized {
        if !self.top_level {
            let scope = &self.ctx.get_node_for(node_id).scope;
            let path = scope.to_string();

            self.ctx.add_error(AstError {
                node_id: node_id.index(),
                err: ErrorCode::AstNotTopLevel { 
                    path,
                    info: "Operator use".to_string(),
                 }
            });
            return;
        }

        let node = &ast[node_id];

        let group = node.group.map(|group| self.names[group].to_string());
        let package = match node.package {
            Some(package) => self.names[package].to_string(),
            None          => self.ctx.lib_path.package.clone()
        };
        let library = match node.library {
            Some(library) => self.names[library].to_string(),
            None          => package.clone(),
        };

        for op in &node.operators {
            let import_path = OperatorImportPath {
                group: group.clone(),
                package: package.clone(),
                library: library.clone(),
                op: *op,
            };
            self.imports.push(import_path);
        }
    }
}






pub struct OperatorReorder<'a> {
    ctx:    &'a mut Context,
    puncts: &'a PuncutationTable,
}

impl<'a> OperatorReorder<'a> {
    pub fn new(ctx: &'a mut Context, puncts: &'a PuncutationTable) -> Self {
        Self {
            ctx,
            puncts,
        }
    }

    // In its own funtion 
    fn process(&mut self, ast: &Ast, node_id: AstNodeRef<InfixExpr>) -> Result<bool, ( )> {
        // first resolve inner nodes
        helpers::visit_binary_expr(self, ast, node_id);

        let node = &ast[node_id];

        // When we can reorder, we need to have another binary expression on the right side, otherwise we are done here
        let Expr::Infix(right_node_id) = node.right else {
            return Ok(false);
        };
        let right = &ast[right_node_id];

        let operators = self.ctx.operators.read();
        let Some(right_info) = operators.get(OpType::Infix, right.op) else {
            let op = right.op.as_str(&self.puncts).to_string();
            self.ctx.add_error(AstError {
                node_id: right_node_id.index(),
                err: ErrorCode::AstOperatorDoesNotExist { op },
            });
            return Err(());
        };
        
        // We can't reorder if the right side has no precedence
        if right_info.precedence.is_none() {
            let op = right.op.as_str(&self.puncts).to_string();
            self.ctx.add_error(AstError {
                node_id: right_node_id.index(),
                err: ErrorCode::AstOperatorNoPrecedence { op },
            });
            return Err(());
        }

        let Some(op_info) = operators.get(OpType::Infix, node.op) else {
            let op = right.op.as_str(&self.puncts).to_string();
            self.ctx.add_error(AstError {
                node_id: node_id.index(),
                err: ErrorCode::AstOperatorDoesNotExist { op },
            });
            return Err(());
        };

        // We can't reorder if the operator has no precedence
        if op_info.precedence.is_none() {
            let op = right.op.as_str(&self.puncts).to_string();
            self.ctx.add_error(AstError {
                node_id: right_node_id.index(),
                err: ErrorCode::AstOperatorNoPrecedence { op },
            });
            return Err(());
        }

        let precedences = self.ctx.precedences.read();
    
        let left_pred = match &op_info.precedence {
            Some(pred) => match precedences.get(&pred) {
                Some(id) => id,
                None => {
                    self.ctx.add_error(AstError {
                        node_id: right_node_id.index(),
                        err: ErrorCode::AstPrecedenceDoesNotExist { precedence: pred.to_string() },
                    });
                    return Err(());
                },
            },
            None => {
                let op = right.op.as_str(&self.puncts).to_string();
                self.ctx.add_error(AstError {
                    node_id: right_node_id.index(),
                    err: ErrorCode::AstOperatorNoPrecedence { op },
                });
                return Err(());
            },
        };

        let right_pred = match &right_info.precedence {
            Some(pred) => match precedences.get(&pred) {
                Some(id) => id,
                None => {
                    self.ctx.add_error(AstError {
                        node_id: right_node_id.index(),
                        err: ErrorCode::AstPrecedenceDoesNotExist { precedence: pred.to_string() },
                    });
                    return Err(());
                },
            },
            None => {
                let op = right.op.as_str(&self.puncts).to_string();
                self.ctx.add_error(AstError {
                    node_id: right_node_id.index(),
                    err: ErrorCode::AstOperatorNoPrecedence { op },
                });
                return Err(());
            },
        };

        match precedences.get_order(left_pred, right_pred) {
            Some(needs_reorder) => {
                Ok(needs_reorder)
            },
            None => {
                let op0 = op_info.op.as_str(&self.puncts).to_string();
                let op1 = right_info.op.as_str(&self.puncts).to_string();
                self.ctx.add_error(AstError {
                    node_id: node_id.index(),
                    err: ErrorCode::AstOperatorNoOrder { op0, op1 },
                });
                Err(())
            },
        }
    }
}

impl Visitor for OperatorReorder<'_> {
    fn visit_binary_expr(&mut self, ast: &Ast, node_id: AstNodeRef<InfixExpr>) where Self: Sized {
        if let Ok(needs_reorder) = self.process(ast, node_id) {
            let node_ctx = self.ctx.get_node_for_mut(node_id);
            let ContextNodeData::Infix{ reorder } = &mut node_ctx.data else { unreachable!() };
            *reorder = needs_reorder;
        }
    }
}