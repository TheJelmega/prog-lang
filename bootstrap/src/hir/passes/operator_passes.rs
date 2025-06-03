use core::prelude;
use std::{collections::{HashMap, VecDeque}, mem, sync::Weak};

use passes::PassContext;

use crate::{
    common::{LibraryPath, NameTable, OperatorInfo, OperatorTable, PathIden, PrecedenceDAG, PrecedenceOrder, RootSymbolTable, RootUseTable, Symbol, SymbolPath, SymbolTable, UseTable, WeakSymbolRef},
    hir::*, lexer::PuncutationTable
};

pub struct OperatorSymbolGen<'a> {
    ctx:         &'a PassContext,
    op_set_name: String,
}

impl<'a> OperatorSymbolGen<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self { ctx, op_set_name: String::new() }
    }
}

impl Visitor for OperatorSymbolGen<'_> {
    fn visit_op_set(&mut self, node: &mut OpSet, ctx: &mut OpSetContext) {
        let name = self.ctx.names.read()[node.name].to_string();
        let sym = self.ctx.syms.write().add_op_set(None, name.clone());
        ctx.sym = Some(sym);


        self.op_set_name = name;
    }

    fn visit_operator(&mut self, op_set_ref: Ref<OpSet>, op_set_ctx: Ref<OpSetContext>, node: &mut Operator, ctx: &mut OperatorContext) {
        let name = self.ctx.names.read()[node.name].to_string();
        let sym = self.ctx.syms.write().add_operator(None, &self.op_set_name, name, node.op_ty, node.op);
        ctx.sym = Some(sym);
    }
}

impl Pass for OperatorSymbolGen<'_> {
    const NAME: &'static str = "Operator Symbol Generation";

    fn process(&mut self, hir: &mut Hir) {
        self.visit(hir, VisitFlags::AnyOp);
    }
}

//==============================================================================================================================

pub struct OperatorSetDependencyProcess<'a> {
    ctx: &'a PassContext
}

impl<'a> OperatorSetDependencyProcess<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self { ctx }
    }
}

impl Visitor for OperatorSetDependencyProcess<'_> {
}

impl Pass for OperatorSetDependencyProcess<'_> {
    const NAME: &'static str = "Operator Set Dependency Processing";

    fn process(&mut self, hir: &mut Hir) {
        let mut to_process: VecDeque<_> = (0..hir.op_sets.len()).collect();

        let syms = self.ctx.syms.read();
        let uses = self.ctx.uses.read();
        let names = self.ctx.names.read();

        while let Some(idx) = to_process.pop_front() {
            let (op_set_ref, ctx_ref) = &mut hir.op_sets[idx];
            let op_set = op_set_ref.read();
            let mut op_ctx = ctx_ref.write();

            let op_set_sym = op_ctx.sym.as_ref().unwrap();
            let mut op_set_sym = op_set_sym.write();
            let Symbol::OpSet(op_set_sym) = &mut *op_set_sym else { unreachable!() };

            // Explicit precedence
            if let Some(precedence) = op_set.precedence {
                let precedence_name = &names[precedence];
                match syms.get_precedence(&uses, precedence_name) {
                    Ok(sym) => op_set_sym.precedence = Some(Arc::downgrade(&sym)),
                    Err(_) => {
                        let op_set_name = names[op_set.name].to_string();
                        self.ctx.add_error(HirError {
                            span: op_set.span,
                            err: HirErrorCode::OpSetUnknownPrecedence { prec: precedence_name.to_string(), op_set: op_set_name },
                        });
                    },
                }

                // We have an explicit precedence, so not base, so just continue to the next one
                continue;
            }
            
            // When we have a base, look it up and proagate the precedence
            // If there are multiple bases and their precedences differ, error (if one doesn't have a precendence, use the precedence from the one that has one)
            // If a base does not have it's precedence set yet, add this operator set's idx back to the 'to process' list
            let mut precedence: Option<WeakSymbolRef> = None;
            let mut conflicting_precedences = Vec::new();
            for (base, _) in &op_set.bases {
                let base_name = &names[*base];
                let base_sym = match syms.get_operator_set(&uses, base_name) {
                    Ok(base_sym) => base_sym,
                    Err(_) => {
                        let op_set_name = names[op_set.name].to_string();
                        self.ctx.add_error(HirError {
                            span: op_set.span,
                            err: HirErrorCode::OpSetUnknowBase { base: base_name.to_string(), op_set: op_set_name },
                        });
                        continue;
                    },
                };
                
                let base_sym = base_sym.read();
                let Symbol::OpSet(base_sym) = &*base_sym else { unreachable!() };
                if let Some(base_precedence) = &base_sym.precedence {
                    match &precedence {
                        Some(prec) => if !prec.ptr_eq(&base_precedence) {
                            if !conflicting_precedences.is_empty() {
                                let prec = prec.upgrade().unwrap();
                                let prec = prec.read();
                                let prec_name = &prec.path().iden().name;
                                conflicting_precedences.push(prec_name.clone());
                            }
                            conflicting_precedences.push(base_name.to_string());
                        },
                        None => precedence = Some(base_precedence.clone()),
                    }
                }
            }

            if !conflicting_precedences.is_empty() {
                let op_set_name = names[op_set.name].to_string();
                self.ctx.add_error(HirError {
                    span: op_set.span,
                    err: HirErrorCode::OpSetConflictBasePrec { op_set: op_set_name.clone(), precedences: conflicting_precedences }
                });
                continue;
            }

            op_set_sym.precedence = precedence;
        }
    }
}

//==============================================================================================================================

pub struct OpSetConnect<'a> {
    ctx: &'a PassContext,
}

impl<'a> OpSetConnect<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self { ctx }
    }
}

impl Visitor for OpSetConnect<'_> {
    fn visit_op_set(&mut self, node: &mut OpSet, ctx: &mut OpSetContext) {
        if node.bases.is_empty() {
            return;
        }

        let mut sym = ctx.sym.as_ref().unwrap().write();
        let Symbol::OpSet(sym) = &mut *sym else { unreachable!() };

        let names = self.ctx.names.read();
        let syms = self.ctx.syms.read();
        let uses = self.ctx.uses.read();

        for (base_name, span) in &node.bases {
            let base_name = &names[*base_name];
            let base_sym = match syms.get_operator_set(&uses, base_name) {
                Ok(sym) => sym,
                Err(err) => {
                    let op_set_name = names[node.name].to_string();
                    self.ctx.add_error(HirError {
                        span: *span,
                        err: HirErrorCode::OpSetUnknowBase { base: base_name.to_string(), op_set: op_set_name },
                    });
                    continue;
                },
            };

            sym.bases.push(Arc::downgrade(&base_sym));
        }
    }
}

impl Pass for OpSetConnect<'_> {
    const NAME: &'static str = "Operator Set Connecting";
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

impl Visitor for OpTagging<'_> {
}

impl Pass for OpTagging<'_> {
    const NAME: &'static str = "Operator Tagging";

    fn process(&mut self, hir: &mut Hir) {
        let names = self.ctx.names.read();
        let mut op_set_idx_mapping = HashMap::with_capacity(hir.op_sets.len());

        // Initial pass
        for (op_set_idx, (op_set, op_set_ctx)) in hir.op_sets.iter().enumerate() {
            let mut has_generics = false;
            let mut has_outuput_alias = false;

            for (idx, node, ctx) in &hir.operators {
                if *idx != op_set_idx {
                    continue;
                }

                if node.op_ty.has_generics() {
                    has_generics = true;
                }
                if node.ret_ty.is_none() && node.op_ty.has_output() {
                    has_outuput_alias = true;
                }
            }

            let op_set_ctx = op_set_ctx.read();
            let sym_ref = op_set_ctx.sym.as_ref().unwrap();
            let mut sym = sym_ref.write();
            let Symbol::OpSet(sym) = &mut *sym else { unreachable!() };

            sym.has_generics = has_generics;
            sym.has_output_alias = has_outuput_alias;

            let name = names[op_set.read().name].to_string();
            op_set_idx_mapping.insert(name, op_set_idx);
        }

        // Propagate to bases
        let mut to_process: VecDeque<_> = (0..hir.op_sets.len()).collect();
        let mut possible_output_conflicts = HashMap::new();
        
        'main: while let Some(idx) = to_process.pop_front() {
            let (node, ctx) = &hir.op_sets[idx];
            let mut ctx = ctx.write();

            let sym = ctx.sym.clone().unwrap();
            let mut sym = sym.write();
            let Symbol::OpSet(sym) = &mut *sym else { unreachable!() };

            if !sym.bases.is_empty() {
                for base in &sym.bases {
                    let base = base.upgrade().unwrap();
                    let base = base.read();
                    let base_name = &base.path().iden().name;
                    let Symbol::OpSet(base) = &*base else { unreachable!() };
                    
                    match op_set_idx_mapping.get(base_name) {
                        Some(base_idx) => if hir.op_sets[*base_idx].1.read().tagging_done {
                            sym.has_generics |= base.has_generics;
                            sym.has_output_alias &= !base.has_output_alias;

                            if base.parent_has_output || base.has_output_alias {
                                let set_name = names[node.read().name].to_string();
                                let entry = possible_output_conflicts.entry(set_name).or_insert((Vec::new(), SpanId::INVALID));
                                entry.0.push(base_name.clone());
                                entry.1 = node.read().span;
                            }
                            sym.parent_has_output |= base.parent_has_output | base.has_output_alias;

                        } else {
                            to_process.push_back(idx);
                            continue 'main;
                        },
                        None => {
                            sym.has_generics |= base.has_generics;
                            sym.has_output_alias &= !base.has_output_alias;
                        }
                    }
                }
            }
            ctx.tagging_done = true;
        };

        for (name, (bases, span)) in possible_output_conflicts {
            if bases.len() > 1 {
                self.ctx.add_error(HirError {
                    span,
                    err: HirErrorCode::OpSetConflictBaseOutput { op_set: name, bases: bases },
                })
            }
        }
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
            VisitFlags::Operator | VisitFlags::OpContract;
        
        self.visit(hir, flags);
    }
}

impl Visitor for InfixReorder<'_> {
    fn visit_infix_expr(&mut self, node: &mut InfixExpr) {
        helpers::visit_infix_expr(self, node);

        let Expr::Infix(right) = &*node.right else { return; };

        let puncts = self.ctx.puncts.read();
        let op_table = self.ctx.op_table.read();

        let op = match op_table.get(OpType::Infix, node.op) {
            Some(op) => op,
            None => {
                self.ctx.add_error(HirError {
                    span: node.span,
                    err: HirErrorCode::OperatorDoesNotExist { op: node.op.as_str(&puncts).to_string() },
                });
                return;
            }
        };
        if op.precedence_id == u16::MAX {
            self.ctx.add_error(HirError {
                span: node.span,
                err: HirErrorCode::OperatorNoPrecedence { op: node.op.as_str(&puncts).to_string() },
            });
            return;
        }

        let right_op = match op_table.get(OpType::Infix, right.op) {
            Some(op) => op,
            None => {
                self.ctx.add_error(HirError {
                    span: right.span,
                    err: HirErrorCode::OperatorDoesNotExist { op: right.op.as_str(&puncts).to_string() },
                });
                return;
            }
        };
        if right_op.precedence_id == u16::MAX {
            self.ctx.add_error(HirError {
                span: node.span,
                err: HirErrorCode::OperatorNoPrecedence { op: right.op.as_str(&puncts).to_string() },
            });
            return;
        }

        match self.ctx.precedence_dag.read().get_order(op.precedence_id, right_op.precedence_id) {
            PrecedenceOrder::None => {
                let op0 = node.op.as_str(&puncts).to_string();
                let op1 = right.op.as_str(&puncts).to_string();
                self.ctx.add_error(HirError {
                    span: node.span,
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

struct TraitGenEntry {
    scope:        Scope,
    file_scope:   Scope,
    item:         Trait,
    methods:      Vec<TraitMethod>,
    output_alias: Option<TraitTypeAlias>,
}

pub struct OpTraitGen<'a> {
    ctx:    &'a PassContext,
    gen_entries: Vec<TraitGenEntry>,
}

impl<'a> OpTraitGen<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx,
            gen_entries: Vec::new(),
        }
    }
}

impl Pass for OpTraitGen<'_> {
    const NAME: &'static str = "Operator Trait Gen";

    fn process(&mut self, hir: &mut Hir) {
        self.visit(hir, VisitFlags::OpSet | VisitFlags::Operator);

        for (op_set_idx, entry) in mem::take(&mut self.gen_entries).into_iter().enumerate() {
            let trait_name =  self.ctx.names.read()[entry.item.name].to_string();
            hir.add_trait(entry.scope.clone(), entry.file_scope.clone(), entry.item);

            let mut scope = entry.scope;
            scope.push(trait_name);
            
            for method in entry.methods {
                hir.add_trait_method(scope.clone(), entry.file_scope.clone(), method);
            }

            let mut hir_method_idx = usize::MAX;
            if let Some(alias) = entry.output_alias {
                hir.add_trait_type_alias(scope, entry.file_scope, alias);

                if hir_method_idx == usize::MAX {
                    hir_method_idx = hir.trait_methods.len() - 1;
                }
            }
            let op_set_ctx = &hir.op_sets[op_set_idx].1;
            op_set_ctx.write().trait_idx = hir.traits.len() - 1;

            if hir_method_idx != usize::MAX {
                for (set_idx, _, op_ctx) in &mut hir.operators {
                    if op_set_idx != *set_idx {
                        continue;
                    }
                    op_ctx.trait_method_idx = hir_method_idx;
                    hir_method_idx += 1;
                }
            }
        }
    }
}

impl Visitor for OpTraitGen<'_> {
    fn visit_op_set(&mut self, node: &mut OpSet, ctx: &mut OpSetContext) {
        let mut names = self.ctx.names.write();
        let op_set = ctx.sym.clone().unwrap();
        let op_set = op_set.read();
        let Symbol::OpSet(op_set) = &*op_set else { unreachable!() };

        let generics = if op_set.has_generics {
            let name = names.add("Rhs");
            let def = Some(Box::new(PathType::self_ty(node.span, node.node_id)));

            let param = GenericParam::Type(GenericTypeParam {
                span: node.span,
                name,
                def,
                ctx: GenericParamContext {
                    sym: None,
                },
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

        let output_alias = if op_set.has_output_alias {
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

        self.gen_entries.push(TraitGenEntry {
            scope: ctx.scope.clone(),
            file_scope: ctx.file_scope.clone(),
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
            output_alias,
        });
    }

    // TODO: const
    fn visit_operator(&mut self, op_set_ref: Ref<OpSet>, op_set_ctx: Ref<OpSetContext>, node: &mut Operator, ctx: &mut OperatorContext) {
        let mut names = self.ctx.names.write();

        let output_ty_name = names.add("Output");

        let is_assign = node.op_ty == OpType::Assign;
        let receiver = FnReceiver::SelfReceiver {
            span: node.span,
            is_ref: is_assign,
            is_mut: is_assign
        };

        let return_ty = if let Some(ret) = &node.ret_ty {
            Some(ret.clone())
        } else if node.op_ty.has_output() {
            Some(Box::new(Type::Path(PathType {
                span: node.span,
                node_id: node.node_id,
                path: Path {
                    span: node.span,
                    node_id: node.node_id,
                    start: PathStart::SelfTy { span: node.span },
                    idens: vec![
                        Identifier {
                            name: IdenName::Name { name: output_ty_name, span: node.span },
                            gen_args: None,
                            span: node.span
                        }
                    ],
                    fn_end: None,
                    ctx: PathCtx::new(),
                },
                ctx: TypeContext::new(),
            })))
        } else {
            None
        };
        
        let params =  if node.op_ty.is_binary() {
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
                    label: None,
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
                ctx: BlockContext::new(),
            })
        });

        let method = TraitMethod {
            span: node.span,
            node_id: node.node_id,
            attrs: Vec::new(),
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

        let op_set = op_set_ref.read();
        let Some(entry) = self.gen_entries.iter_mut().find(|entry| entry.item.node_id == op_set.node_id) else {
            self.ctx.add_error(HirError {
                span: node.span,
                err: HirErrorCode::InternalError("Processing function for op trait that was not generated"),
            });
            return;
        };

        entry.methods.push(method);     
    }
}

//==============================================================================================================================

pub struct OpSetTraitAssociation<'a> {
    ctx: &'a PassContext,
}

impl<'a> OpSetTraitAssociation<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self { ctx }
    }
}

impl Visitor for OpSetTraitAssociation<'_> {
}

impl Pass for OpSetTraitAssociation<'_> {
    const NAME: &'static str = "Op Set <-> Trait Association";

    fn process(&mut self, hir: &mut Hir) {
        for (_, ctx) in &hir.op_sets {
            let ctx = ctx.read();
            let (_, trait_ctx) = &hir.traits[ctx.trait_idx];

            let sym = ctx.sym.as_ref().unwrap();
            let mut sym = sym.write();
            let Symbol::OpSet(sym) = &mut *sym else { unreachable!() };
            sym.assoc_trait = Some(Arc::downgrade(trait_ctx.read().sym.as_ref().unwrap()));
        }

        for (_, _, ctx) in &hir.operators {
            if ctx.trait_method_idx == usize::MAX {
                continue;
            }

            let (_, _, method_ctx) = &hir.trait_methods[ctx.trait_method_idx];

            let sym = ctx.sym.as_ref().unwrap();
            let mut sym = sym.write();
            let Symbol::Operator(sym) = &mut *sym else { unreachable!() };
            sym.assoc_method = Some(Arc::downgrade(method_ctx.sym.as_ref().unwrap()));
        }
    }
}