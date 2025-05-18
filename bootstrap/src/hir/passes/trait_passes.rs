use std::collections::HashMap;

use crate::{
    common::{Scope, Symbol, SymbolPath, TraitItemKind},
    error_warning::HirErrorCode,
    hir::*,
};

use super::{Pass, PassContext};


pub struct TraitDagGen<'a> {
    ctx: &'a PassContext
}

impl<'a> TraitDagGen<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx,
        }
    }
}



impl Pass for TraitDagGen<'_> {
    const NAME: &'static str = "Trait DAG Generation";

    fn process(&mut self, hir: &mut Hir) {
        let mut dag = self.ctx.trait_dag.write();

        // Collect trait symbols and add them to the DAG
        for (node, ctx) in &mut hir.traits {
            let mut ctx = ctx.write();
            
            let sym = ctx.sym.as_ref().unwrap().clone();
            let idx = dag.add(sym.clone());
            ctx.dag_idx = idx;

            let mut sym = sym.write();
            let Symbol::Trait(sym) = &mut *sym else {
                self.ctx.add_error(HirError {
                    node_id: node.read().node_id,
                    err: HirErrorCode::InternalError("Trait does not have trait symbol associated with it"),
                });
                continue;
            };
            sym.dag_idx = idx;
        }

        for (node, ctx) in &mut hir.op_traits {
            let mut ctx = ctx.write();
            
            let sym = ctx.sym.as_ref().unwrap().clone();
            let idx = dag.add(sym.clone());
            ctx.dag_idx = idx;

            let mut sym = sym.write();
            let Symbol::Trait(sym) = &mut *sym else {
                self.ctx.add_error(HirError {
                    node_id: node.read().node_id,
                    err: HirErrorCode::InternalError("Trait does not have trait symbol associated with it"),
                });
                continue;
            };
            sym.dag_idx = idx;
        }

        let names = self.ctx.names.read();
        let syms = self.ctx.syms.read();
        let uses = self.ctx.uses.read();

        // Now set dependencies
        for (node, ctx) in &mut hir.traits {
            let node = node.read();
            let ctx = ctx.read();

            if let Some(bound) = &node.bounds {
                for path in &bound.bounds {
                    let scope = &path.ctx.path;
                    let Some(sym) = syms.get_symbol_with_uses(&uses, &ctx.scope, None, scope) else {
                        self.ctx.add_error(HirError {
                            node_id: bound.node_id,
                            err: HirErrorCode::UnknownSymbol { path: scope.clone() },
                        });
                        continue;
                    };
                    let mut sym = sym.write();
                    let Symbol::Trait(sym) = &mut *sym else {
                        self.ctx.add_error(HirError {
                            node_id: bound.node_id,
                            err: HirErrorCode::InternalError("Trait does not have trait symbol associated with it"),
                        });
                        continue;
                    };

                    dag.set_base_dependency(ctx.dag_idx, sym.dag_idx);
                }
            }
        }

        for (node, ctx) in &mut hir.op_traits {
            let node = node.read();
            let ctx = ctx.read();

            for path in &node.bases {
                let scope = &path.ctx.path;

                let Some(sym) = syms.get_symbol_with_uses(&uses, &ctx.scope, None, scope) else {
                    self.ctx.add_error(HirError {
                        node_id: node.node_id,
                        err: HirErrorCode::UnknownSymbol { path: scope.clone() },
                    });
                    continue;
                };
                let mut sym = sym.write();
                let Symbol::Trait(sym) = &mut *sym else {
                    self.ctx.add_error(HirError {
                        node_id: node.node_id,
                        err: HirErrorCode::InternalError("Trait does not have trait symbol associated with it"),
                    });
                    continue;
                };
            
                dag.set_base_dependency(ctx.dag_idx, sym.dag_idx);
            }
        }
    }
}

impl Visitor for TraitDagGen<'_> {

}

//==============================================================================================================================

pub struct TraitItemProcess<'a> {
    ctx: &'a PassContext,
}

impl<'a> TraitItemProcess<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx,
        }
    }
}

impl Visitor for TraitItemProcess<'_> {
    fn visit_trait_function(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitFunction, ctx: &mut FunctionContext) {
        let mut trait_ctx = trait_ctx.write();

        trait_ctx.items.push(TraitItemRecord {
            name: self.ctx.names.read()[node.name].to_string(),
            kind: TraitItemKind::Function,
            has_default: node.body.is_some(),
            idx: ctx.idx,
        });
    }

    fn visit_trait_method(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitMethod, ctx: &mut FunctionContext) {
        let mut trait_ctx = trait_ctx.write();
        trait_ctx.items.push(TraitItemRecord {
            name: self.ctx.names.read()[node.name].to_string(),
            kind: TraitItemKind::Method,
            has_default: node.body.is_some(),
            idx: ctx.idx,
        });
    }

    fn visit_trait_type_alias(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitTypeAlias, ctx: &mut TypeAliasContext) {
        let mut trait_ctx = trait_ctx.write();
        trait_ctx.items.push(TraitItemRecord {
            name: self.ctx.names.read()[node.name].to_string(),
            kind: TraitItemKind::TypeAlias,
            has_default: node.def.is_some(),
            idx: ctx.idx,
        });
    }

    fn visit_trait_const(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitConst, ctx: &mut ConstContext) {
        let mut trait_ctx = trait_ctx.write();
        trait_ctx.items.push(TraitItemRecord {
            name: self.ctx.names.read()[node.name].to_string(),
            kind: TraitItemKind::Const,
            has_default: node.def.is_some(),
            idx: ctx.idx,
        });
    }

    fn visit_trait_property(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitProperty, ctx: &mut PropertyContext) {
        let mut trait_ctx = trait_ctx.write();

        let (kind, has_default) = match &node.members {
            TraitPropMembers::Req { get, ref_get, mut_get, set } => (
                TraitItemKind::Property { get: get.is_some(), ref_get: ref_get.is_some(), mut_set: mut_get.is_some(), set: set.is_some() },
                false
            ),
            TraitPropMembers::Def { get, ref_get, mut_get, set } => (
                TraitItemKind::Property { get: get.is_some(), ref_get: ref_get.is_some(), mut_set: mut_get.is_some(), set: set.is_some() },
                false
            ),
        };

        trait_ctx.items.push(TraitItemRecord {
            name: self.ctx.names.read()[node.name].to_string(),
            kind,
            has_default,
            idx: ctx.idx,
        });
    }
}

impl Pass for TraitItemProcess<'_> {
    const NAME: &'static str = "Trait Item processing";

    fn process(&mut self, hir: &mut Hir) {
        self.visit(hir, VisitFlags::AnyTrait);

        // TODO: Do this while adding the items
        for (_, ctx) in &mut hir.traits {
            let ctx = ctx.read();
            let sym = ctx.sym.clone().unwrap();
            let mut sym = sym.write();
            let Symbol::Trait(sym) = &mut *sym else { unreachable!() };

            sym.items = ctx.items.clone();
        }
    }
}

//==============================================================================================================================

pub struct ImplTraitSymResolve<'a> {
    ctx: &'a PassContext,
}

impl<'a> ImplTraitSymResolve<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx,
        }
    }
}

impl Visitor for ImplTraitSymResolve<'_> {
    fn visit_impl(&mut self, node: &mut Impl, ctx: &mut ImplContext) {
        let Some(impl_trait_path) = &node.impl_trait else { return; };

        let sym_path = &impl_trait_path.ctx.path;

        let syms = self.ctx.syms.read();
        let uses = self.ctx.uses.read();
        let sym = syms.get_symbol_with_uses(&uses, &ctx.scope, None, sym_path);
        
        ctx.trait_sym = sym;
    }
}

impl Pass for ImplTraitSymResolve<'_> {
    const NAME: &'static str = "Impl Trait symbol resolve";

    fn process(&mut self, hir: &mut Hir) {
        self.visit(hir, VisitFlags::Impl);
    }
}

//==============================================================================================================================

pub struct ImplTraitItemCollection<'a> {
    ctx:       &'a PassContext,
}

impl<'a> ImplTraitItemCollection<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx,
        }
    }
}

impl Visitor for ImplTraitItemCollection<'_> {}

impl Pass for ImplTraitItemCollection<'_> {
    const NAME: &'static str = "Impl trait item collection";

    fn process(&mut self, hir: &mut Hir) {
        helpers::visit_impl_cond(self, hir, VisitFlags::Impl, |this, node, ctx| {
            let Some(trait_sym) = ctx.trait_sym.clone() else { return false; };

            let sym = trait_sym.read();
            let Symbol::Trait(sym) = &*sym else { unreachable!() };
            let mut items = Vec::with_capacity(sym.items.len());
            for item in &sym.items {
                items.push((item.clone(), false));
            }
            ctx.trait_items = items;

            false
        })
    }
}

//==============================================================================================================================

pub struct TraitDefImplInfo {
    trait_sym: Symbol,
    item_name: String,
}

pub struct TraitImpl<'a> {
    ctx:        &'a PassContext,
    impl_ctx:   Option<Ref<ImplContext>>,
    trait_name: SymbolPath,
    impl_idx:   usize,
}

impl<'a> TraitImpl<'a> {
    pub fn new(ctx: &'a PassContext) -> Self {
        Self {
            ctx,
            impl_ctx: None,
            trait_name: SymbolPath::new(),
            impl_idx: 0,
        }
    }

    fn item_check(&mut self, name: NameId, node_id: NodeId) {
        let names = self.ctx.names.read();
        let name = &names[name];

        let impl_ctx = self.impl_ctx.clone().unwrap();
        let mut impl_ctx = impl_ctx.write();


        let idx = impl_ctx.trait_items.iter().enumerate().find_map(|(idx, item)| {
            if item.0.name == name {
                Some(idx)
            } else {
                None
            }
        });

        match idx {
            // Set as found, so we won't need to add a default later on
            Some(idx) => impl_ctx.trait_items[idx].1 = true,
            // otherwise we will report errors later on
            None => self.ctx.add_error(HirError {
                node_id: node_id,
                err: HirErrorCode::ImplTraitNoMatchingItem {
                    item: name.to_string(),
                    trait_name: self.trait_name.clone(),
                    info: "No function with this name exists",
                },
            }),
        }
    }
}

impl Visitor for TraitImpl<'_> {
    fn visit_impl_function(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut Function, ctx: &mut FunctionContext) {
        self.item_check(node.name, node.node_id);
    }

    fn visit_method(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut Method, ctx: &mut FunctionContext) {
        self.item_check(node.name, node.node_id);
    }

    fn visit_impl_type_alias(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut TypeAlias, ctx: &mut TypeAliasContext) {
        self.item_check(node.name, node.node_id);
    }

    fn visit_impl_const(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut Const, ctx: &mut ConstContext) {
        self.item_check(node.name, node.node_id);
    }

    fn visit_property(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut Property, ctx: &mut PropertyContext) {
        self.item_check(node.name, node.node_id);
    }
}

impl Pass for TraitImpl<'_> {
    const NAME: &'static str = "Trait Impl Check And Default Impl";

    fn process(&mut self, hir: &mut Hir) {
        // Collect all implemented items, and then mark the ones that are implemented
        helpers::visit_impl_cond_unlocked(self, hir, VisitFlags::all(), |this, node, ctx| {
            this.impl_ctx = Some(ctx.clone());

            let ctx = ctx.read();
            ctx.trait_sym.is_some()
        });

        let mut def_fns = Vec::new();
        let mut def_methods = Vec::new();
        let mut def_type_aliases = Vec::new();
        let mut def_consts = Vec::new();
        let mut def_properties = Vec::new();

        // Now add default for items that don't exist yet
        for (impl_idx, (node, ctx)) in hir.impls.iter().enumerate() {
            let ctx = ctx.read();

            let trait_sym_ref = match &ctx.trait_sym {
                Some(sym) => sym,
                None => continue,
            };
            let trait_sym = trait_sym_ref.read();

            let Some(trait_entry) = hir.traits.iter().find(|(_, ctx)| Arc::ptr_eq(ctx.read().sym.as_ref().unwrap(), &trait_sym_ref)) else {
                self.ctx.add_error(HirError {
                    node_id: node.read().node_id,
                    err: HirErrorCode::NoHirItemForSymbol { kind: "trait" },
                });
                continue;
            };
            let vis = trait_entry.0.read().vis.clone();

            for (item, exists) in &ctx.trait_items {
                if *exists {
                    continue;
                }
                
                if !item.has_default {
                    self.ctx.add_error(HirError {
                        node_id: node.read().node_id,
                        err: HirErrorCode::ImplNoDefault { item: item.name.clone() },
                    });
                    continue;
                }

                let syms = self.ctx.syms.read();

                let trait_path = trait_sym.path();
                let mut scope = trait_path.scope.clone();
                scope.push(trait_path.name.clone());
                let item_sym = syms.get_symbol(Some(&trait_path.lib), &scope, &item.name).unwrap();

                // We just find the local hir implementation
                if trait_path.lib == self.ctx.lib_path {
                    match item.kind {
                        TraitItemKind::Function => {
                            let Some(entry) = hir.trait_functions.iter().find(|(_, _, ctx)| Arc::ptr_eq(ctx.sym.as_ref().unwrap(), &item_sym)) else {
                                self.ctx.add_error(HirError {
                                    node_id: node.read().node_id,
                                    err: HirErrorCode::NoHirItemForSymbol { kind: "trait function" },
                                });
                                continue;
                            };

                            let trait_fn = &entry.1;
                            // Exists, otherwise `item.has_default` would have been false
                            let body = trait_fn.body.clone().unwrap();
                            
                            let def_fn = Function {
                                span: trait_fn.span,
                                node_id: trait_fn.node_id,
                                attrs: Vec::new(),
                                vis: vis.clone(),
                                is_const: trait_fn.is_const,
                                is_unsafe: trait_fn.is_unsafe,
                                abi: Abi::Xenon,
                                name: trait_fn.name,
                                generics: trait_fn.generics.clone(),
                                params: trait_fn.params.clone(),
                                return_ty: trait_fn.return_ty.clone(),
                                where_clause: trait_fn.where_clause.clone(),
                                contracts: Vec::new(),
                                body,
                            };
                            def_fns.push((impl_idx, ctx.scope.clone(), def_fn));

                        },
                        TraitItemKind::Method => {
                            let Some(entry) = hir.trait_methods.iter().find(|(_, _, ctx)| Arc::ptr_eq(ctx.sym.as_ref().unwrap(), &item_sym)) else {
                                self.ctx.add_error(HirError {
                                    node_id: node.read().node_id,
                                    err: HirErrorCode::NoHirItemForSymbol { kind: "trait function" },
                                });
                                continue;
                            };

                            let trait_fn = &entry.1;
                            // Exists, otherwise `item.has_default` would have been false
                            let body = trait_fn.body.clone().unwrap();

                            let def_method = Method {
                                span: trait_fn.span,
                                node_id: trait_fn.node_id,
                                attrs: Vec::new(),
                                vis: vis.clone(),
                                is_const: trait_fn.is_const,
                                is_unsafe: trait_fn.is_unsafe,
                                name: trait_fn.name,
                                generics: trait_fn.generics.clone(),
                                receiver: trait_fn.receiver.clone(),
                                params: trait_fn.params.clone(),
                                return_ty: trait_fn.return_ty.clone(),
                                where_clause: trait_fn.where_clause.clone(),
                                contracts: Vec::new(),
                                body: body,
                            };
                            def_methods.push((impl_idx, ctx.scope.clone(), def_method));
                        },
                        TraitItemKind::TypeAlias => {
                            let Some(entry) = hir.trait_type_alias.iter().find(|(_, _, ctx)| Arc::ptr_eq(ctx.sym.as_ref().unwrap(), &item_sym)) else {
                                self.ctx.add_error(HirError {
                                    node_id: node.read().node_id,
                                    err: HirErrorCode::NoHirItemForSymbol { kind: "trait function" },
                                });
                                continue;
                            };

                            let trait_alias = &entry.1;

                            // Exists, otherwise `item.has_default` would have been false
                            let ty = trait_alias.def.clone().unwrap();

                            let def_alias = TypeAlias {
                                span: trait_alias.span,
                                node_id: trait_alias.node_id,
                                attrs: Vec::new(),
                                vis: vis.clone(),
                                name: trait_alias.name,
                                generics: trait_alias.generics.clone(),
                                ty,
                            };
                            def_type_aliases.push((impl_idx, ctx.scope.clone(), def_alias));
                        },
                        TraitItemKind::Const => {
                            let Some(entry) = hir.trait_consts.iter().find(|(_, _,  ctx)| Arc::ptr_eq(ctx.sym.as_ref().unwrap(), &item_sym)) else {
                                self.ctx.add_error(HirError {
                                    node_id: node.read().node_id,
                                    err: HirErrorCode::NoHirItemForSymbol { kind: "trait function" },
                                });
                                continue;
                            };

                            let trait_const = &entry.1;
                            // Exists, otherwise `item.has_default` would have been false
                            let val = trait_const.def.clone().unwrap();

                            let def_const = Const {
                                span: trait_const.span,
                                node_id: trait_const.node_id,
                                attrs: Vec::new(),
                                vis: vis.clone(),
                                name: trait_const.name,
                                ty: Some(trait_const.ty.clone()),
                                val,
                            };
                            def_consts.push((impl_idx, ctx.scope.clone(), def_const));
                        },
                        TraitItemKind::Property { get, ref_get, mut_set, set } => {
                            let Some(trait_entry) = hir.trait_properties.iter().find(|(_, _,  ctx)| Arc::ptr_eq(ctx.sym.as_ref().unwrap(), &item_sym)) else {
                                self.ctx.add_error(HirError {
                                    node_id: node.read().node_id,
                                    err: HirErrorCode::NoHirItemForSymbol { kind: "trait function" },
                                });
                                continue;
                            };

                            let trait_prop = &trait_entry.1;
                            // Exists, otherwise `item.has_default` would have been false
                            let TraitPropMembers::Def { get, ref_get, mut_get, set } = &trait_prop.members else { unreachable!() };

                            let def_prop = Property {
                                span: trait_prop.span,
                                node_id: trait_prop.node_id,
                                attrs: Vec::new(),
                                vis: vis.clone(),
                                is_unsafe: trait_prop.is_unsafe,
                                name: trait_prop.name,
                                get: get.clone().map(|(_, expr)| expr),
                                ref_get: ref_get.clone().map(|(_, expr)| expr),
                                mut_get: mut_get.clone().map(|(_, expr)| expr),
                                set: set.clone().map(|(_, expr)| expr),
                            };
                            def_properties.push((impl_idx, ctx.scope.clone(), def_prop));
                        },
                    }

                } else {
                    // We now need to somehow get this from an external library
                    self.ctx.add_error(HirError {
                        node_id: NodeId::INVALID,
                        err: HirErrorCode::NotSupportedYet { info: "Retrieving default implementations from external libraries" }, 
                    });
                }
            }
        }

        // Now insert each item
        let mut insert_idx = 0;
        for (impl_idx, scope, item) in def_fns {
            while insert_idx < hir.impl_functions.len() && hir.impl_functions[insert_idx].0 <= insert_idx {
                insert_idx += 1;
            }

            let ctx = FunctionContext::new(scope, insert_idx);
            let tup = (impl_idx, item, ctx);
            if insert_idx == hir.impl_functions.len() {
                hir.impl_functions.push(tup)
            } else {
                hir.impl_functions.insert(insert_idx, tup);
            }
        }
        for (impl_idx, scope, item) in def_methods {
            while insert_idx < hir.methods.len() && hir.methods[insert_idx].0 <= insert_idx {
                insert_idx += 1;
            }

            let ctx = FunctionContext::new(scope, insert_idx);
            let tup = (impl_idx, item, ctx);
            if insert_idx == hir.methods.len() {
                hir.methods.push(tup)
            } else {
                hir.methods.insert(insert_idx, tup);
            }
        }
        for (impl_idx, scope, item) in def_type_aliases {
            while insert_idx < hir.impl_type_aliases.len() && hir.impl_type_aliases[insert_idx].0 <= insert_idx {
                insert_idx += 1;
            }

            let ctx = TypeAliasContext::new(scope, insert_idx);
            let tup = (impl_idx, item, ctx);
            if insert_idx == hir.impl_type_aliases.len() {
                hir.impl_type_aliases.push(tup)
            } else {
                hir.impl_type_aliases.insert(insert_idx, tup);
            }
        }
        for (impl_idx, scope, item) in def_consts {
            while insert_idx < hir.impl_consts.len() && hir.impl_consts[insert_idx].0 <= insert_idx {
                insert_idx += 1;
            }

            let ctx = ConstContext::new(scope, insert_idx);
            let tup = (impl_idx, item, ctx);
            if insert_idx == hir.impl_consts.len() {
                hir.impl_consts.push(tup)
            } else {
                hir.impl_consts.insert(insert_idx, tup);
            }
        }
        for (impl_idx, scope, item) in def_properties {
            while insert_idx < hir.properties.len() && hir.properties[insert_idx].0 <= insert_idx {
                insert_idx += 1;
            }

            let ctx = PropertyContext::new(scope, insert_idx);
            let tup = (impl_idx, item, ctx);
            if insert_idx == hir.properties.len() {
                hir.properties.push(tup)
            } else {
                hir.properties.insert(insert_idx, tup);
            }
        }
    }
}