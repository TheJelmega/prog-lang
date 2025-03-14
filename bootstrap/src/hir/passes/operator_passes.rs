use core::prelude;
use std::{collections::VecDeque, mem};

use passes::PassContext;

use crate::{
    common::{LibraryPath, NameTable, OperatorInfo, OperatorTable, PrecedenceDAG, PrecedenceOrder, RootSymbolTable, RootUseTable, Symbol, SymbolTable, UseTable},
    hir::*, lexer::PuncutationTable
};

pub struct OpPrecedenceProcessing<'a> {
    ctx: &'a PassContext
}

impl<'a> OpPrecedenceProcessing<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx,
        }
    }
}

impl Pass for OpPrecedenceProcessing<'_> {
    const NAME: &'static str = "Op trait <-> precedence processing";
}

impl Visitor for OpPrecedenceProcessing<'_> {
    fn visit(&mut self, hir: &mut Hir, _flags: VisitFlags) {
        
        let mut to_process: VecDeque<usize> = (0..hir.op_traits.len()).collect();
        

        while !to_process.is_empty() {
            // Can't fail, as we check if the array contains elements in the loop
            let idx = to_process.pop_front().unwrap();
            let (op_trait_ref, ctx_ref) = &mut hir.op_traits[idx];
            let op_trait_ref = op_trait_ref.write();

            let mut ctx_ref = ctx_ref.write();
            let mut sym_path = ctx_ref.scope.clone();
            sym_path.push(self.ctx.names.read()[op_trait_ref.name].to_string());

            // Explicit precedence
            if let Some(precedence) = op_trait_ref.precedence {
                let precedence_name = &self.ctx.names.read()[precedence];
                    match self.ctx.precedence_dag.read().get(precedence_name){
                        Some(id) => self.ctx.op_table.write().add_trait_precedence(sym_path.clone(), precedence_name.to_string(), id),
                        None => todo!("Error")
                    };
            } else if op_trait_ref.bases.is_empty() { // Default if there are no base classes, i.e. no precedence
                self.ctx.op_table.write().add_trait_precedence(sym_path, "<none>".to_string(), u16::MAX);
                continue;
            }

            // When we have a base, look it up and propagate the precedence
            
            // TODO: use paths

            for base in &op_trait_ref.bases {

                // TODO: Store this is some node context
                let mut search_sym_path = Scope::new();
                for name in &base.names {
                    search_sym_path.push(self.ctx.names.read()[*name].to_string());
                }

                let Some(sym) = self.ctx.syms.read().get_symbol_with_uses(&self.ctx.uses.read(), &ctx_ref.scope, None, &search_sym_path) else {
                    todo!("Error");
                };
                let Symbol::Trait(sym) = &*sym.read() else { 
                    todo!("Error");
                };

                let mut base_sym_path = sym.path.scope.clone();
                base_sym_path.push(sym.path.name.clone());

                let trait_precedence = self.ctx.op_table.read().get_trait_precedence(&base_sym_path).map(|(name, id)| (name.to_string(), id));
                match trait_precedence {
                    Some((prec, id)) => {
                        self.ctx.op_table.write().add_trait_precedence(sym_path.clone(), prec, id);
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

//==============================================================================================================================

pub struct OperatorCollection<'a> {
    ctx: &'a PassContext
}

impl<'a> OperatorCollection<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx
        }
    }
}

impl Pass for OperatorCollection<'_> {
    const NAME: &'static str = "Operator Collection";
}

impl Visitor for OperatorCollection<'_> {
    fn visit_op_function(&mut self, op_trait_ref: Ref<OpTrait>, op_trait_ctx: Ref<OpTraitContext>, node: &mut OpFunction, ctx: &mut OpFunctionContext) {
        let op_trait_path = ctx.scope.clone();
        
        let op_info = {
            let op_table = self.ctx.op_table.read();
            let (prec_name, prec_id) = op_table.get_trait_precedence(&op_trait_path).unwrap();

            OperatorInfo {
                op_type: node.op_ty,
                op: node.op,
                precedence_name: prec_name.to_string(),
                precedence_id: prec_id,
                library_path: self.ctx.lib_path.clone(),
                trait_path: ctx.scope.clone(),
                func_name: self.ctx.names.read()[node.name].to_string(),
            }
        };
        self.ctx.op_table.write().add_operator(op_info);  
    }
}

//==============================================================================================================================

pub struct InfixReorder<'a> {
    ctx: &'a PassContext
}

impl<'a> InfixReorder<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx
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

        let puncts = self.ctx.puncts.read();
        let op_table = self.ctx.op_table.read();

        let op = match op_table.get(OpType::Infix, node.op) {
            Some(op) => op,
            None => {
                self.ctx.add_error(HirError {
                    node_id: node.node_id,
                    err: HirErrorCode::OperatorDoesNotExist { op: node.op.as_str(&puncts).to_string() },
                });
                return;
            }
        };
        if op.precedence_id == u16::MAX {
            self.ctx.add_error(HirError {
                node_id: node.node_id,
                err: HirErrorCode::OperatorNoPrecedence { op: node.op.as_str(&puncts).to_string() },
            });
        }

        let right_op = match op_table.get(OpType::Infix, right.op) {
            Some(op) => op,
            None => {
                self.ctx.add_error(HirError {
                    node_id: right.node_id,
                    err: HirErrorCode::OperatorDoesNotExist { op: right.op.as_str(&puncts).to_string() },
                });
                return;
            }
        };
        if right_op.precedence_id == u16::MAX {
            self.ctx.add_error(HirError {
                node_id: node.node_id,
                err: HirErrorCode::OperatorNoPrecedence { op: right.op.as_str(&puncts).to_string() },
            });
        }

        match self.ctx.precedence_dag.read().get_order(op.precedence_id, right_op.precedence_id) {
            PrecedenceOrder::None => {
                let op0 = node.op.as_str(&puncts).to_string();
                let op1 = right.op.as_str(&puncts).to_string();
                self.ctx.add_error(HirError {
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

//==============================================================================================================================

pub struct OpTagging<'a> {
    ctx: &'a PassContext
}

impl<'a> OpTagging<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx,
        }
    }
}

impl Pass for OpTagging<'_> {
    const NAME: &'static str = "Operator Tagging";

    fn process(&mut self, hir: &mut Hir) {
        
        // Tag traits as using generics
        for (trait_idx, node, ctx) in &hir.op_functions {
            let mut trait_ctx = hir.op_traits[*trait_idx].1.write();
            if matches!(node.op_ty, OpType::Infix | OpType::Assign) {
                trait_ctx.has_generics = true;
            }
            if node.ret_ty.is_none() {
                trait_ctx.has_output_alias = true;
            }
        }

        let trait_dag = self.ctx.trait_dag.read();

        // Propagate generics flag to parent traits
        for idx in 0..hir.op_traits.len() {
            let dag_idx = hir.op_traits[idx].1.read().dag_idx;
            let base_ids = trait_dag.get_base_ids(dag_idx);
            for base in base_ids {
                let base_data  = trait_dag.get(*base).unwrap();
                let entry = hir.op_traits.iter().find(|entry| {
                    let ctx = entry.1.read();
                    Arc::ptr_eq(ctx.sym.as_ref().unwrap(), &base_data.symbol)
                }).unwrap();

                if entry.1.read().has_generics {
                    let mut trait_ctx = hir.op_traits[idx].1.write();
                    trait_ctx.has_generics = true;
                    break;
                }
            }
        }

    }
}

impl Visitor for OpTagging<'_> {
    
}


//==============================================================================================================================

struct TraitGenEntry {
    scope:        Scope,
    item:         Trait,
    methods:      Vec<TraitMethod>,
    output_alias: Option<TraitTypeAlias>,
}

pub struct OpTraitGen<'a> {
    ctx:    &'a PassContext,
    traits: Vec<TraitGenEntry>
}

impl<'a> OpTraitGen<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx,
            traits: Vec::new(),
        }
    }
}

impl Pass for OpTraitGen<'_> {
    const NAME: &'static str = "Operator Trait Gen";

    fn process(&mut self, hir: &mut Hir) {
        self.visit(hir, VisitFlags::OpTrait | VisitFlags::OpFunction | VisitFlags::OpSpecialization);

        for entry in mem::take(&mut self.traits) {
            let trait_name =  self.ctx.names.read()[entry.item.name].to_string();
            hir.add_trait(entry.scope.clone(), entry.item);

            let mut scope = entry.scope;
            scope.push(trait_name);
            for method in entry.methods {
                hir.add_trait_method(scope.clone(), method);
            }
            if let Some(alias) = entry.output_alias {
                hir.add_trait_type_alias(scope, alias);
            }
        }
    }
}

impl Visitor for OpTraitGen<'_> {
    fn visit_op_trait(&mut self, node: &mut OpTrait, ctx: &mut OpTraitContext) {
        let mut names = self.ctx.names.write();

        let generics = if ctx.has_generics {
            let name = names.add("Rhs");
            let self_ty_name = names.add("Self");
            let def = Some(Box::new(PathType::from_name(self_ty_name, node.span, node.node_id)));

            let param = GenericParam::Type(GenericTypeParam {
                span: node.span,
                name,
                def,
            });

            Some(Box::new(GenericParams {
                span: node.span,
                node_id: node.node_id,
                params: vec![
                    param
                ],
                pack: None,
            }))
        } else {
            None
        };

        let output_alias = if ctx.has_output_alias {
            let output_ty_name = names.add("Output");

            Some(TraitTypeAlias {
                span: node.span,
                node_id: node.node_id,
                attrs: Vec::new(),
                name: output_ty_name,
                generics: None,
                where_clause: None,
                def: None,
            })
        } else {
            None
        };

        self.traits.push(TraitGenEntry {
            scope: ctx.scope.clone(),
            item: Trait {
                span: node.span,
                node_id: node.node_id,
                attrs: node.attrs.clone(),
                vis: node.vis.clone(),
                is_unsafe: false,
                is_sealed: false,
                name: node.name,
                generics,
                bounds: None,
                where_clause: None,
            },
            methods: Vec::new(),
            output_alias
        });
    }

    // TODO: const
    fn visit_op_function(&mut self, op_trait_ref: Ref<OpTrait>, op_trait_ctx: Ref<OpTraitContext>, node: &mut OpFunction, ctx: &mut OpFunctionContext) {
        let mut names = self.ctx.names.write();

        let self_name = names.add("Self");
        let output_ty_name = names.add("Output");


        let receiver = FnReceiver::SelfReceiver {
            span: node.span,
            is_ref: false,
            is_mut: false
        };

        let return_ty = if let Some(ret) = &node.ret_ty {
            Some(ret.clone())
        } else {
            Some(Box::new(Type::Path(PathType {
                span: node.span,
                node_id: node.node_id,
                path: TypePath {
                    span: node.span,
                    node_id: node.node_id,
                    segments: vec![
                        TypePathSegment::Plain {
                            span: node.span,
                            name: self_name,
                        },
                        TypePathSegment::Plain {
                            span: node.span,
                            name: output_ty_name,
                        }
                    ],
                },
            })))
        };
        
        let params =  if node.op_ty == OpType::Infix {
            let label = Some(names.add("_"));
            let rhs_name = names.add("rhs");
            let rhs_ty_name = names.add("Rhs");

            let pattern = Box::new(Pattern::Iden(IdenPattern {
                span: node.span,
                node_id: node.node_id,
                is_ref: false,
                is_mut: false,
                name: rhs_name,
                bound: None,
            }));
            let ty = Box::new(PathType::from_name(rhs_ty_name, node.span, node.node_id));

            vec![
                FnParam::Param {
                    span: node.span,
                    attrs: Vec::new(),
                    label,
                    pattern,
                    ty,
                }
            ]
        } else {
            Vec::new()
        };

        let body = node.def.as_ref().map(|expr| {
            Box::new(Block {
                span: node.span,
                stmts: Vec::new(),
                expr: Some(expr.clone()),
            })
        });

        let method = TraitMethod {
            span: node.span,
            node_id: node.node_id,
            attrs: Vec::new(),
            is_override: false,
            is_const: false,
            is_unsafe: false,
            name: node.name,
            generics: None,
            receiver,
            params,
            return_ty,
            where_clause: None,
            contracts: Vec::new(),
            body,
        };

        let op_trait = op_trait_ref.read();
        let Some(entry) = self.traits.iter_mut().find(|entry| entry.item.node_id == op_trait.node_id) else {
            self.ctx.add_error(HirError {
                node_id: node.node_id,
                err: HirErrorCode::InternalError("Processing function for op trait that was not generated"),
            });
            return;
        };
        entry.methods.push(method);     
    }
}