use core::prelude;
use std::{collections::VecDeque, mem};

use crate::{
    common::{LibraryPath, NameTable, OperatorInfo, OperatorTable, PrecedenceDAG, PrecedenceOrder, RootSymbolTable, Symbol, SymbolTable, UseTable},
    hir::*, lexer::PuncutationTable
};

pub struct PrecedenceProcessing<'a> {
    names:          &'a NameTable,
    precedence_dag: &'a PrecedenceDAG,
    sym_table:      &'a RootSymbolTable,
    op_table:       &'a mut OperatorTable,
    use_table:      &'a UseTable,
}

impl<'a> PrecedenceProcessing<'a> {
    pub fn new(names: &'a NameTable, precedence_dag: &'a PrecedenceDAG, sym_table: &'a RootSymbolTable, op_table: &'a mut OperatorTable, use_table: &'a UseTable) -> Self {
        Self {
            names,
            precedence_dag,
            sym_table,
            op_table,
            use_table,
        }
    }
}

impl Pass for PrecedenceProcessing<'_> {
    const NAME: &'static str = "Op trait <-> precedence processing";
}

impl Visitor for PrecedenceProcessing<'_> {
    fn visit(&mut self, hir: &mut Hir, _flags: VisitFlags) {
        
        let mut to_process: VecDeque<usize> = (0..hir.op_traits.len()).collect();
        

        while !to_process.is_empty() {
            // Can't fail, as we check if the array contains elements in the loop
            let idx = to_process.pop_front().unwrap();
            let (op_trait_ref, ctx_ref) = &mut hir.op_traits[idx];
            let op_trait_ref = op_trait_ref.write();

            let mut ctx_ref = ctx_ref.write();
            let mut sym_path = ctx_ref.scope.clone();
            sym_path.push(self.names[op_trait_ref.name].to_string());

            // Explicit precedence
            if let Some(precedence) = op_trait_ref.precedence {
                let precedence_name = &self.names[precedence];
                    match self.precedence_dag.get(precedence_name){
                        Some(id) => self.op_table.add_trait_precedence(sym_path.clone(), precedence_name.to_string(), id),
                        None => todo!("Error")
                    };
            } else if op_trait_ref.bases.is_empty() { // Default if there are no base classes, i.e. no precedence
                self.op_table.add_trait_precedence(sym_path, "<none>".to_string(), u16::MAX);
                continue;
            }

            // When we have a base, look it up and propagate the precedence
            
            // TODO: use paths

            for base in &op_trait_ref.bases {

                // TODO: Store this is some node context
                let mut search_sym_path = Scope::new();
                for name in &base.names {
                    search_sym_path.push(self.names[*name].to_string());
                }

                let sub_scope = Scope::new();
                let Some(sym) = self.sym_table.get_symbol_with_uses(self.use_table, &ctx_ref.scope, &sub_scope, &search_sym_path) else {
                    todo!("Error");
                };
                let Symbol::Trait(sym) = &*sym.read() else { 
                    todo!("Error");
                };

                let mut base_sym_path = sym.scope.clone();
                base_sym_path.push(sym.name.clone());

                let trait_precedence = self.op_table.get_trait_precedence(&base_sym_path).map(|(name, id)| (name.to_string(), id));
                match trait_precedence {
                    Some((prec, id)) => {
                        self.op_table.add_trait_precedence(sym_path.clone(), prec, id);
                        break;
                    },
                    None    => {
                        to_process.push_back(idx);
                    },
                }
            }
        }

    }
}

pub struct OperatorCollection<'a> {
    names:    &'a NameTable,
    op_table: &'a mut OperatorTable,

    lib_path: LibraryPath,
}

impl<'a> OperatorCollection<'a> {
    pub fn new(names: &'a NameTable, op_table: &'a mut OperatorTable, lib_path: LibraryPath) -> Self {
        Self {
            names,
            op_table,
            lib_path,
        }
    }
}

impl Pass for OperatorCollection<'_> {
    const NAME: &'static str = "Operator Collection";
}

impl Visitor for OperatorCollection<'_> {
    fn visit_op_function(&mut self, op_trait_ref: Ref<OpTrait>, op_trait_ctx: Ref<OpTraitContext>, node: &mut OpFunction, ctx: &mut OpFunctionContext) {
        let op_trait_path = ctx.scope.clone();
        let (prec_name, prec_id) = self.op_table.get_trait_precedence(&op_trait_path).unwrap();

        let op_info = OperatorInfo {
            op_type: node.op_ty,
            op: node.op,
            precedence_name: prec_name.to_string(),
            precedence_id: prec_id,
            library_path: self.lib_path.clone(),
            trait_path: ctx.scope.clone(),
            func_name: self.names[node.name].to_string(),
        };
        self.op_table.add_operator(op_info);  
    }
}



pub struct InfixReorder<'a> {
    puncts:         &'a PuncutationTable,
    op_table:       &'a OperatorTable,
    precedence_dag: &'a PrecedenceDAG,
    errors:         &'a mut Vec<HirError>,
}

impl<'a> InfixReorder<'a> {
    pub fn new(puncts: &'a PuncutationTable, op_table: &'a OperatorTable, precedence_dag: &'a PrecedenceDAG, errors: &'a mut Vec<HirError>) -> Self {
        Self {
            puncts,
            op_table,
            precedence_dag,
            errors,
        }
    }
}

impl Pass for InfixReorder<'_> {
    const NAME: &'static str = "Infix Reordering";

    fn process(&mut self, hir: &mut Hir) {
        let flags = VisitFlags::Function | VisitFlags::TraitFunction | VisitFlags::Method |
            VisitFlags::OpFunction | VisitFlags::OpSpecialization | VisitFlags::OpContract;
        
        self.visit(hir, flags);
    }
}

impl Visitor for InfixReorder<'_> {
    fn visit_infix_expr(&mut self, node: &mut InfixExpr) {
        helpers::visit_infix_expr(self, node);

        if !node.can_reorder {
            return;
        }

        let Expr::Infix(right) = &*node.right else { unreachable!("Internal Compiler error here!") };

        let op = match self.op_table.get(OpType::Infix, node.op) {
            Some(op) => op,
            None => {
                self.errors.push(HirError {
                    node_id: node.node_id,
                    err: HirErrorCode::OperatorDoesNotExist { op: node.op.as_str(self.puncts).to_string() },
                });
                return;
            }
        };
        if op.precedence_id == u16::MAX {
            self.errors.push(HirError {
                node_id: node.node_id,
                err: HirErrorCode::OperatorNoPrecedence { op: node.op.as_str(self.puncts).to_string() },
            });
        }

        let right_op = match self.op_table.get(OpType::Infix, right.op) {
            Some(op) => op,
            None => {
                self.errors.push(HirError {
                    node_id: right.node_id,
                    err: HirErrorCode::OperatorDoesNotExist { op: right.op.as_str(self.puncts).to_string() },
                });
                return;
            }
        };
        if right_op.precedence_id == u16::MAX {
            self.errors.push(HirError {
                node_id: node.node_id,
                err: HirErrorCode::OperatorNoPrecedence { op: right.op.as_str(self.puncts).to_string() },
            });
        }

        match self.precedence_dag.get_order(op.precedence_id, right_op.precedence_id) {
            PrecedenceOrder::None => {
                let op0 = node.op.as_str(self.puncts).to_string();
                let op1 = right.op.as_str(self.puncts).to_string();
                self.errors.push(HirError {
                    node_id: node.node_id,
                    err: HirErrorCode::OperatorNoOrder { op0, op1 },
                });
            },
            PrecedenceOrder::Higher => { // the current precedence is higher, so re-order (higher is inner)
                // temporary dummy used when switching nodes around
                let dummy = Box::new(Expr::Unit(UnitExpr {
                    span: SpanId::INVALID,
                    node_id: ast::NodeId::INVALID,
                }));

                let right_node_id = right.node_id;
                let right_span = right.span;

                let op = node.op;
                let right_op = right.op;

                let right = mem::replace(&mut node.right, dummy.clone());
                let Expr::Infix(right) = *right else { unreachable!("Internal Compiler error here!") };
                let middle = right.left;
                let right = right.right;
                let left = mem::replace(&mut node.left, dummy);

                // Update current node
                node.left = Box::new(Expr::Infix(InfixExpr {
                    span: node.span,
                    node_id: node.node_id,
                    left,
                    op,
                    right: middle,
                    can_reorder: false, // doens't matter after this point + already in the correct order, so don't even need todo this
                }));
                node.span = right_span;
                node.node_id = right_node_id;
                node.op = right_op;
                node.right = right;
                
            },
            PrecedenceOrder::Same => (), // TODO: reorder based on associativity
            PrecedenceOrder::Lower => (), // Nothing to do if already in the correct order
        }
    }
}