use crate::{common::{IndentLogger, NameTable}, lexer::PuncutationTable, literals::LiteralTable};

use super::{
    *,
    visitor::helpers,
};


pub struct NodeLogger<'a> {
    pub logger: IndentLogger,
    pub names:  &'a NameTable,
    pub lits:   &'a LiteralTable,
    pub puncts: &'a PuncutationTable,
}

impl<'a> NodeLogger<'a> {
    pub fn new(names: &'a NameTable, lits: &'a LiteralTable, puncts: &'a PuncutationTable) -> Self {
        Self {
            logger: IndentLogger::new("    ", "|   ", "+---"),
            names,
            lits,
            puncts,
        }
    }
}

impl NodeLogger<'_> {
    pub fn log_indented<F: FnMut(&mut Self)>(&mut self, name: &str, mut f: F) {
        self.logger.prefixed_logln(name);
        self.logger.push_indent();
        f(self);
        self.logger.pop_indent();
    }

    pub fn log_single_indented<F: FnMut(&mut Self)>(&mut self, name: &str, mut f: F) {
        self.log_indented(name, |this| {
            this.logger.set_last_at_indent();
            f(this);
        })
    }

    pub fn log_opt_indented<T, F: FnMut(&mut Self, &mut T)>(&mut self, name: &str, val: &mut Option<T>, mut f: F) {
        if let Some(val) = val {
            self.logger.prefixed_logln(name);
            self.logger.push_indent();
            self.logger.set_last_at_indent();
            f(self, val);
            self.logger.pop_indent();
        }
    }

    pub fn log_slice_indented<T, F: FnMut(&mut Self, &mut T)>(&mut self, name: &str, slice: &mut [T], mut f: F) {
        if slice.is_empty() {
            return;
        }

        self.logger.prefixed_logln(name);
        self.logger.push_indent();
        let end = slice.len() - 1;
        for (idx, elem) in slice.iter_mut().enumerate() {
            if idx == end {
                self.logger.set_last_at_indent();
            }

            f(self, elem);
        }
        self.logger.pop_indent();
    }

    pub fn log_slice<T, F: FnMut(&mut Self, &mut T)>(&mut self, slice: &mut [T], mut f: F) {
        if slice.is_empty() {
            return;
        }
        let end = slice.len() - 1;
        for (idx, elem) in slice.iter_mut().enumerate() {
            if idx == end {
                self.logger.set_last_at_indent();
            }

            f(self, elem);
        }
    }

    pub fn log_node<F: FnMut(&mut Self)>(&mut self, name: &str, node_id: ast::NodeId, mut f: F) {
        self.logger.prefixed_log_fmt(format_args!("{name} (node ID: {node_id})\n"));
        self.logger.push_indent();
        f(self);
        self.logger.pop_indent();
    }




    pub fn log_visibility(&mut self, vis: &mut Visibility) {
        match vis {
            Visibility::Priv{ .. } => self.logger.prefixed_logln("Visibility: private"),
            Visibility::Pub{ .. } => self.logger.prefixed_logln("Visibility: public"),
            Visibility::Lib{ .. } => self.logger.prefixed_logln("Visibility: library"),
            Visibility::Package{ .. } => self.logger.prefixed_logln("Visibility: package"),
            Visibility::Super{ .. } => self.logger.prefixed_logln("Visibility: super"),
            Visibility::Path{ path, .. } => {
                self.log_slice_indented("Visibility: Path", &mut path.names, |this, name|
                    this.logger.prefixed_logln(&this.names[*name])
                );
            },
        }
    }

    pub fn log_fn_receiver(&mut self, receiver: &mut FnReceiver) {
        match receiver {
            FnReceiver::None => (),
            FnReceiver::SelfReceiver { span, is_ref, is_mut } => self.log_indented("Self receiver", |this| {
                this.logger.prefixed_log_fmt(format_args!("Is ref: {is_ref}\n"));
                this.logger.set_last_at_indent();
                this.logger.prefixed_log_fmt(format_args!("Is mut: {is_mut}\n"));
            }),
            FnReceiver::SelfTyped { span, is_mut, ty } => self.log_indented("Typed receiver", |this| {
                this.logger.prefixed_log_fmt(format_args!("Is mut: {is_mut}\n"));
                this.logger.set_last_at_indent();
                this.log_single_indented("Type", |this| this.visit_type(ty));
            }),
        }
    }

    pub fn log_fn_param(&mut self, param: &mut FnParam) {
        match param {
            FnParam::Param { span, attrs, label, pattern, ty } => {
                self.logger.prefixed_log_fmt(format_args!("Param: {}", label.map_or("", |label| &self.names[label])));
                self.logger.push_indent();
                self.log_slice_indented("Attributes", attrs, |this, attr| {
                    this.visit_attribute(attr);
                });
                self.log_single_indented("Pattern", |this| this.visit_pattern(pattern));
                self.log_single_indented("Type", |this| this.visit_type(ty));
                self.logger.pop_indent();
            },
            FnParam::Opt { span, attrs, label, pattern, ty, def } => {
                self.logger.prefixed_log_fmt(format_args!("Optional Param: {}", label.map_or("", |label| &self.names[label])));
                self.logger.push_indent();
                self.log_slice_indented("Attributes", attrs, |this, attr| {
                    this.visit_attribute(attr);
                });
                self.log_single_indented("Pattern", |this| this.visit_pattern(pattern));
                self.log_single_indented("Type", |this| this.visit_type(ty));
                self.log_single_indented("Default Value", |this| this.visit_expr(def));
                self.logger.pop_indent();
            },
            FnParam::Variadic { span, attrs, name, ty } => {
                self.logger.prefixed_log_fmt(format_args!("Varaidic param: {}", &self.names[*name]));
                self.logger.push_indent();
                self.log_slice_indented("Attributes", attrs, |this, attr| {
                    this.visit_attribute(attr);
                });
                self.log_single_indented("Type", |this| this.visit_type(ty));
                self.logger.pop_indent();
            },
        }
    }

    pub fn log_attr_meta(&mut self, attr_meta: &mut AttrMeta) {
        match attr_meta {
            AttrMeta::Simple { path } => self.log_indented("Simple Attribute Meta", |this| this.visit_simple_path(path)),
            AttrMeta::Expr { expr } => self.log_indented("Expr Attribute Meta", |this| this.visit_expr(expr)),
            AttrMeta::Assign { span, path, expr } => self.log_indented("Assign Attribute Meta", |this| {
                this.log_indented("Path", |this| this.visit_simple_path(path));
                this.logger.set_last_at_indent();
                this.log_indented("Expr", |this| this.visit_expr(expr));
            }),
            AttrMeta::Meta { span, path, metas } => self.log_indented("Meta Attribute Meta", |this| {
                this.log_indented("Path", |this| this.visit_simple_path(path));
                this.logger.set_last_at_indent();
                this.log_slice_indented("Metas", metas, |this, meta| this.log_attr_meta(meta));
            }),
        }
    }


    pub fn log_trait(&mut self, hir: &mut Hir, idx: usize) {
        let (trait_ref, trait_ctx) = &hir.traits[idx];
        let node_id = {
            let node = trait_ref.write();
            node.node_id
        };

        self.log_node("Trait", node_id, |this| {

            let type_aliases_count = hir.trait_type_alias.iter().filter(|(search_idx, _, _)| *search_idx == idx).count();
            let consts_count = hir.trait_consts.iter().filter(|(search_idx, _, _)| *search_idx == idx).count();
            let funcs_count = hir.trait_functions.iter().filter(|(search_idx, _, _)| *search_idx == idx).count();
            let props_count = hir.trait_properties.iter().filter(|(search_idx, _, _)| *search_idx == idx).count();

            {
                let mut node = trait_ref.write();

                let has_items = type_aliases_count == 0 && consts_count == 0 && funcs_count == 0 && props_count == 0;

                this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
                this.logger.prefixed_log_fmt(format_args!("Is unsafe: {}\n", node.is_unsafe));
                this.logger.prefixed_log_fmt(format_args!("Is sealed: {}\n", node.is_sealed));
                this.logger.set_last_at_indent_if(node.attrs.is_empty() && node.bounds.is_none() && has_items);
                this.log_visibility(&mut node.vis);
                this.logger.set_last_at_indent_if(node.bounds.is_none() && has_items);
                this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
                this.logger.set_last_at_indent_if(has_items);
                this.log_opt_indented("Bounds", &mut node.bounds, |this, bounds| this.visit_trait_bounds(bounds));
            }

            let mut count = 0;
            for (fn_idx, node, ctx) in &mut hir.trait_type_alias {
                if *fn_idx == idx {
                    this.logger.set_last_at_indent_if(count == type_aliases_count - 1 && consts_count == 0 && funcs_count == 0 && props_count == 0);
                    this.visit_trait_type_alias(trait_ref.clone(), trait_ctx.clone(), node, ctx);
                    count += 1;
                }
            }

            count = 0;
            for (fn_idx, node, ctx) in &mut hir.trait_consts {
                if *fn_idx == idx {
                    this.logger.set_last_at_indent_if(count == consts_count - 1 && funcs_count == 0 && props_count == 0);
                    this.visit_trait_const(trait_ref.clone(), trait_ctx.clone(), node, ctx);
                    count += 1;
                }
            }
            
            count = 0;
            for (fn_idx, node, ctx) in &mut hir.trait_functions {
                if *fn_idx == idx {
                    this.logger.set_last_at_indent_if(count == funcs_count - 1 && props_count == 0);
                    this.visit_trait_function(trait_ref.clone(), trait_ctx.clone(), node, ctx);
                    count += 1;
                }
            }
            
            count = 0;
            for (fn_idx, node, ctx) in &mut hir.trait_properties {
                if *fn_idx == idx {
                    this.logger.set_last_at_indent_if(count == props_count - 1);
                    this.visit_trait_property(trait_ref.clone(), trait_ctx.clone(), node, ctx);
                    count += 1;
                }
            }

        });
    }

    pub fn log_impl(&mut self, hir: &mut Hir, idx: usize) {
        let (impl_ref, impl_ctx) = &hir.impls[idx];
        let node_id = {
            let node = impl_ref.write();
            node.node_id
        };

        self.log_node("Impl", node_id, |this| {

            let funcs_count = hir.trait_functions.iter().filter(|(search_idx, _, _)| *search_idx == idx).count();
            let method_count = hir.methods.iter().filter(|(search_idx, _, _)| *search_idx == idx).count();
            let type_aliases_count = hir.impl_type_aliases.iter().filter(|(search_idx, _, _)| *search_idx == idx).count();
            let consts_count = hir.impl_consts.iter().filter(|(search_idx, _, _)| *search_idx == idx).count();
            let static_count = hir.impl_statics.iter().filter(|(search_idx, _, _)| *search_idx == idx).count();
            let tls_static_count = hir.impl_tls_statics.iter().filter(|(search_idx, _, _)| *search_idx == idx).count();
            let props_count = hir.properties.iter().filter(|(search_idx, _, _)| *search_idx == idx).count();

            {
                let mut node = impl_ref.write();

                let has_items = funcs_count == 0 && method_count == 0 && type_aliases_count == 0 && consts_count == 0 && static_count == 0 && tls_static_count == 0 && props_count == 0;

                this.logger.prefixed_log_fmt(format_args!("Is unsafe: {}\n", node.is_unsafe));
                this.log_visibility(&mut node.vis);
                this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
                this.log_opt_indented("Generics", &mut node.generics, |this, generics| this.visit_gen_params(generics));
                this.logger.set_last_at_indent_if(node.impl_trait.is_none() && node.where_clause.is_none() && has_items);
                this.log_single_indented("Type", |this| this.visit_type(&mut node.ty));
                this.logger.set_last_at_indent_if(node.where_clause.is_none() && has_items);
                this.log_opt_indented("Impl trait", &mut node.impl_trait, |this, impl_trait| this.visit_path(impl_trait));
                this.logger.set_last_at_indent_if(has_items);
                this.log_opt_indented("Where clause", &mut node.where_clause, |this, where_clause| this.visit_where_clause(where_clause));
            }

            let mut count = 0;
            for (fn_idx, node, ctx) in &mut hir.impl_functions {
                if *fn_idx == idx {
                    this.logger.set_last_at_indent_if(count == funcs_count - 1 && method_count == 0 && type_aliases_count == 0 && consts_count == 0 && static_count == 0 && tls_static_count == 0 && props_count == 0);
                    this.visit_impl_function(impl_ref.clone(), impl_ctx.clone(), node, ctx);
                    count += 1;
                }
            }

            count = 0;
            for (fn_idx, node, ctx) in &mut hir.methods {
                if *fn_idx == idx {
                    this.logger.set_last_at_indent_if(count == method_count - 1 && type_aliases_count == 0 && consts_count == 0 && static_count == 0 && tls_static_count == 0 && props_count == 0);
                    this.visit_method(impl_ref.clone(), impl_ctx.clone(), node, ctx);
                    count += 1;
                }
            }

            count = 0;
            for (fn_idx, node, ctx) in &mut hir.impl_type_aliases {
                if *fn_idx == idx {
                    this.logger.set_last_at_indent_if(count == type_aliases_count - 1 && consts_count == 0 && static_count == 0 && tls_static_count == 0 && props_count == 0);
                    this.visit_impl_type_alias(impl_ref.clone(), impl_ctx.clone(), node, ctx);
                    count += 1;
                }
            }

            count = 0;
            for (fn_idx, node, ctx) in &mut hir.impl_consts {
                if *fn_idx == idx {
                    this.logger.set_last_at_indent_if(count == consts_count - 1 && static_count == 0 && tls_static_count == 0 && props_count == 0);
                    this.visit_impl_const(impl_ref.clone(), impl_ctx.clone(), node, ctx);
                    count += 1;
                }
            }

            count = 0;
            for (fn_idx, node, ctx) in &mut hir.impl_statics {
                if *fn_idx == idx {
                    this.logger.set_last_at_indent_if(count == static_count - 1 && tls_static_count == 0 && props_count == 0);
                    this.visit_impl_static(impl_ref.clone(), impl_ctx.clone(), node, ctx);
                    count += 1;
                }
            }

            count = 0;
            for (fn_idx, node, ctx) in &mut hir.impl_tls_statics {
                if *fn_idx == idx {
                    this.logger.set_last_at_indent_if(count == tls_static_count - 1 && props_count == 0);
                    this.visit_impl_tls_static(impl_ref.clone(), impl_ctx.clone(), node, ctx);
                    count += 1;
                }
            }

            count = 0;
            for (fn_idx, node, ctx) in &mut hir.properties {
                if *fn_idx == idx {
                    this.logger.set_last_at_indent_if(count == props_count - 1);
                    this.visit_property(impl_ref.clone(), impl_ctx.clone(), node, ctx);
                    count += 1;
                }
            }
        });
    }

    pub fn log_op_set(&mut self, hir: &mut Hir, idx: usize) {
        let (trait_ref, trait_ctx) = &hir.op_sets[idx];
        let node_id = {
            let node = trait_ref.write();
            node.node_id
        };

        self.log_node("Op trait", node_id, |this| {

            let funcs_count = hir.operators.iter().filter(|(search_idx, _, _)| *search_idx == idx).count();
            let contract_count = hir.op_contracts.iter().filter(|(search_idx, _, _)| *search_idx == idx).count();

            {
                let mut node = trait_ref.write();

                let has_items = contract_count == 0 && funcs_count == 0;

                this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
                this.logger.set_last_at_indent_if(node.attrs.is_empty() && node.bases.is_empty() && has_items);
                this.log_visibility(&mut node.vis);
                this.logger.set_last_at_indent_if(node.bases.is_empty() && has_items);
                this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
                this.logger.set_last_at_indent_if(has_items);
                this.log_slice_indented("Bases", &mut node.bases, |this, (base, _)| this.logger.prefixed_logln(&self.names[*base]));
            }

            let mut count = 0;
            for (fn_idx, node, ctx) in &mut hir.operators {
                if *fn_idx == idx {
                    this.logger.set_last_at_indent_if(count == funcs_count - 1 && contract_count == 0);
                    this.visit_operator(trait_ref.clone(), trait_ctx.clone(), node, ctx);
                    count += 1;
                }
            }
            
            count = 0;
            for (fn_idx, node, ctx) in &mut hir.op_contracts {
                if *fn_idx == idx {
                    this.logger.set_last_at_indent_if(count == contract_count - 1);
                    this.visit_op_contract(trait_ref.clone(), trait_ctx.clone(), node, ctx);
                    count += 1;
                }
            }

        });
    }

    pub fn log_identifier(&mut self, iden: &mut Identifier) {
        match &mut iden.name {
            IdenName::Name { name, span } => self.logger.prefixed_log_fmt(format_args!("Identifier: {}\n", &self.names[*name])),
            IdenName::Disambig { trait_path, name, .. } => self.log_indented("Trait Disambiguation", |this| {
                this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[*name]));
                this.log_indented("Trait", |this| this.visit_path(trait_path));
            }),
        }
        if let Some(gen_args) = &mut iden.gen_args {
            self.logger.push_indent();
            self.logger.set_last_at_indent();
            self.log_single_indented("Generics", |this| this.visit_gen_args(gen_args));
            self.logger.pop_indent();
        }
    }

    pub fn log_path_start(&mut self, start: &mut PathStart) {
        match start {
            PathStart::None => (),
            PathStart::SelfTy { span } => self.logger.prefixed_logln("Start: Self type relative"),
            PathStart::Inferred { span } => self.logger.prefixed_logln("Start: Inferred"),
            PathStart::Type { ty } => self.log_indented("Start: Type relative", |this| this.visit_type(ty)),
        }
    }
}

impl Visitor for NodeLogger<'_> {
    fn visit(&mut self, hir: &mut Hir, flags: VisitFlags) {
        let ignore_trait_flags =
            VisitFlags::Trait |
            VisitFlags::TraitFunction |
            VisitFlags::TraitTypeAlias |
            VisitFlags::TraitConst |
            VisitFlags::TraitProperty;

        let ignore_impl_flags =
            VisitFlags::Impl |
            VisitFlags::ImplFunction |
            VisitFlags::Method |
            VisitFlags::ImplTypeAlias |
            VisitFlags::ImplConst |
            VisitFlags::ImplStatic |
            VisitFlags::ImplTlsStatic |
            VisitFlags::Property;

        let ignore_op_flags =
            VisitFlags::OpSet |
            VisitFlags::Operator |
            VisitFlags::OpContract;

        let ignore_flags = ignore_trait_flags | ignore_impl_flags | ignore_op_flags;
        helpers::visit(self, hir, flags & !ignore_flags);

        for idx in 0..hir.traits.len() {
            self.log_trait(hir, idx);
        }

        for idx in 0..hir.impls.len() {
            self.log_impl(hir, idx);
        }

        for idx in 0..hir.op_sets.len() {
            self.log_op_set(hir, idx);
        }
    }

    // =============================================================

    fn visit_path(&mut self, path: &mut Path) {
        self.log_node("Path", path.node_id, |this| {
            this.log_path_start(&mut path.start);

            this.logger.set_last_at_indent_if(path.fn_end.is_none());
            this.log_slice_indented("Identifiers", &mut path.idens, |this, iden| {
                this.log_identifier(iden);
            });

            this.logger.set_last_at_indent();
            if let Some(fn_end) = &mut path.fn_end {
                this.log_indented("Trait Path Function End", |this| {
                    this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[fn_end.name]));
                    this.log_slice_indented("Parameters", &mut fn_end.params, |this, (name, ty)| {
                        this.logger.log_fmt(format_args!("Name: {}", &this.names[*name]));
                        this.visit_type(ty);
                    })
                });
                this.log_opt_indented("Return Type", &mut fn_end.ret_ty, |this, ty| this.visit_type(ty));
            }
            
        });
    }

    // =============================================================

    fn visit_function(&mut self, node: &mut Function, ctx: &mut FunctionContext) {
        self.log_node("Function", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.logger.prefixed_log_fmt(format_args!("Is const: {}\n", node.is_const));
            this.logger.prefixed_log_fmt(format_args!("Is unsafe: {}\n", node.is_unsafe));
            this.logger.prefixed_log_fmt(format_args!("ABI: {}\n", node.abi));
            this.log_visibility(&mut node.vis);
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.log_opt_indented("Generics", &mut node.generics, |this, generics| this.visit_gen_params(generics));
            this.log_slice_indented("Params", &mut node.params, |this, param| this.log_fn_param(param));
            this.log_opt_indented("Return Type", &mut node.return_ty, |this, ty| this.visit_type(ty));
            this.log_opt_indented("Where Clause", &mut node.where_clause, |this, where_clause| this.visit_where_clause(where_clause));
            this.log_slice_indented("Contracts", &mut node.contracts, |this, contract| this.visit_contract(contract));
            this.logger.set_last_at_indent();
            this.log_single_indented("Body", |this| this.visit_block(&mut node.body));
        });
    }

    fn visit_extern_function_no_body(&mut self, node: &mut ExternFunctionNoBody, ctx: &mut FunctionContext) {
        self.log_node("Extern Function", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.logger.prefixed_log_fmt(format_args!("Is unsafe: {}\n", node.is_unsafe));
            this.logger.prefixed_log_fmt(format_args!("ABI: {}\n", node.abi));
            this.logger.set_last_at_indent_if(node.attrs.is_empty() && node.params.is_empty() && node.return_ty.is_none() && node.contracts.is_empty());
            this.log_visibility(&mut node.vis);
            this.logger.set_last_at_indent_if(node.params.is_empty() && node.return_ty.is_none() && node.contracts.is_empty());
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.logger.set_last_at_indent_if(node.return_ty.is_none() && node.contracts.is_empty());
            this.log_slice_indented("Params", &mut node.params, |this, param| this.log_fn_param(param));
            this.logger.set_last_at_indent_if(node.contracts.is_empty());
            this.log_opt_indented("Return Type", &mut node.return_ty, |this, ty| this.visit_type(ty));
            this.logger.set_last_at_indent();
            this.log_slice_indented("Contracts", &mut node.contracts, |this, contract| this.visit_contract(contract));
        });
    }

    fn visit_type_alias(&mut self, node: &mut TypeAlias, ctx: &mut TypeAliasContext) {
        self.log_node("Type Alias", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.log_visibility(&mut node.vis);
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.log_opt_indented("Generics", &mut node.generics, |this, generics| this.visit_gen_params(generics));
            this.logger.set_last_at_indent();
            this.log_single_indented("Type", |this| this.visit_type(&mut node.ty));
        });
    }

    fn visit_distinct_type(&mut self, node: &mut DistinctType, ctx: &mut TypeAliasContext) {
        self.log_node("Distinct Type", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.log_visibility(&mut node.vis);
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.log_opt_indented("Generics", &mut node.generics, |this, generics| this.visit_gen_params(generics));
            this.logger.set_last_at_indent();
            this.log_single_indented("Type", |this| this.visit_type(&mut node.ty));
        });
    }

    fn visit_opaque_type(&mut self, node: &mut OpaqueType, ctx: &mut TypeAliasContext) {
        self.log_node("Opaque Type", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.logger.set_last_at_indent_if(node.attrs.is_empty() && node.size.is_none());
            this.log_visibility(&mut node.vis);
            this.logger.set_last_at_indent_if(node.size.is_none());
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.logger.set_last_at_indent();
            this.log_opt_indented("Size", &mut node.size, |this, size| this.visit_expr(size));
        });
    }

    fn visit_struct(&mut self, node: &mut Struct, ctx: &mut StructContext) {
        self.log_node("Struct", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.logger.prefixed_log_fmt(format_args!("Is mut: {}\n", node.is_mut));
            this.logger.prefixed_log_fmt(format_args!("Is record: {}\n", node.is_record));
            this.logger.set_last_at_indent_if(node.attrs.is_empty() && node.generics.is_none() && node.where_clause.is_none() && node.fields.is_empty());
            this.log_visibility(&mut node.vis);
            this.logger.set_last_at_indent_if(node.generics.is_none() && node.where_clause.is_none() && node.fields.is_empty());
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.logger.set_last_at_indent_if(node.where_clause.is_none() && node.fields.is_empty());
            this.log_opt_indented("Generics", &mut node.generics, |this, generics| this.visit_gen_params(generics));
            this.logger.set_last_at_indent_if(node.fields.is_empty());
            this.log_opt_indented("Where Clause", &mut node.where_clause, |this, where_clause| this.visit_where_clause(where_clause));
            this.logger.set_last_at_indent();
            this.log_slice_indented("Fields", &mut node.fields, |this, field| {
                this.logger.prefixed_log_fmt(format_args!("Param: {}\n", &this.names[field.name]));
                this.logger.push_indent();
                this.logger.prefixed_log_fmt(format_args!("Is mut: {}\n", field.is_mut));
                this.log_visibility(&mut node.vis);
                this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
                this.logger.set_last_at_indent_if(field.def.is_none());
                this.log_single_indented("Type", |this| this.visit_type(&mut field.ty));
                this.logger.set_last_at_indent();
                this.log_opt_indented("Default value", &mut field.def, |this, def| this.visit_expr(def));
                this.logger.pop_indent();
            });
            this.log_slice_indented("Uses", &mut node.uses, |this, struct_use| {
                this.logger.prefixed_logln("Use");
                this.logger.push_indent();
                this.logger.prefixed_log_fmt(format_args!("Is mut: {}\n", struct_use.is_mut));
                this.log_visibility(&mut struct_use.vis);
                this.log_slice_indented("Attributes", &mut struct_use.attrs, |this, attr| this.visit_attribute(attr));
                this.logger.set_last_at_indent();
                this.log_single_indented("Path", |this| this.visit_path(&mut struct_use.path));
                this.logger.pop_indent();
            });
            
        });
    }

    fn visit_tuple_struct(&mut self, node: &mut TupleStruct, ctx: &mut StructContext) {
        self.log_node("Tuple Struct", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.logger.prefixed_log_fmt(format_args!("Is mut: {}\n", node.is_mut));
            this.logger.prefixed_log_fmt(format_args!("Is record: {}\n", node.is_record));
            this.logger.set_last_at_indent_if(node.attrs.is_empty() && node.generics.is_none() && node.where_clause.is_none() && node.fields.is_empty());
            this.log_visibility(&mut node.vis);
            this.logger.set_last_at_indent_if(node.generics.is_none() && node.where_clause.is_none() && node.fields.is_empty());
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.logger.set_last_at_indent_if(node.where_clause.is_none() && node.fields.is_empty());
            this.log_opt_indented("Generics", &mut node.generics, |this, generics| this.visit_gen_params(generics));
            this.logger.set_last_at_indent_if(node.fields.is_empty());
            this.log_opt_indented("Where Clause", &mut node.where_clause, |this, where_clause| this.visit_where_clause(where_clause));
            this.logger.set_last_at_indent();
            this.log_slice_indented("Fields", &mut node.fields, |this, field| {
                this.logger.prefixed_logln("Field");
                this.logger.push_indent();
                this.log_visibility(&mut field.vis);
                this.log_slice_indented("Attributes", &mut field.attrs, |this, attr| this.visit_attribute(attr));
                this.logger.set_last_at_indent_if(field.def.is_none());
                this.log_single_indented("Type", |this| this.visit_type(&mut field.ty));
                this.logger.set_last_at_indent();
                this.log_opt_indented("Default value", &mut field.def, |this, def| this.visit_expr(def));
                this.logger.pop_indent();
            });
        });
    }

    fn visit_unit_struct(&mut self, node: &mut UnitStruct, ctx: &mut StructContext) {
        self.log_node("Tuple Struct", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.logger.set_last_at_indent_if(node.attrs.is_empty());
            this.log_visibility(&mut node.vis);
            this.logger.set_last_at_indent();
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
        });
    }

    fn visit_union(&mut self, node: &mut Union, ctx: &mut UnionContext) {
        self.log_node("Union", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.logger.prefixed_log_fmt(format_args!("Is mut: {}\n", node.is_mut));
            this.logger.set_last_at_indent_if(node.attrs.is_empty() && node.generics.is_none() && node.where_clause.is_none() && node.fields.is_empty());
            this.log_visibility(&mut node.vis);
            this.logger.set_last_at_indent_if(node.generics.is_none() && node.where_clause.is_none() && node.fields.is_empty());
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.logger.set_last_at_indent_if(node.where_clause.is_none() && node.fields.is_empty());
            this.log_opt_indented("Generics", &mut node.generics, |this, generics| this.visit_gen_params(generics));
            this.logger.set_last_at_indent_if(node.fields.is_empty());
            this.log_opt_indented("Where Clause", &mut node.where_clause, |this, where_clause| this.visit_where_clause(where_clause));
            this.logger.set_last_at_indent();
            this.log_slice_indented("Fields", &mut node.fields, |this, field| {
                this.logger.prefixed_log_fmt(format_args!("Param: {}\n", &this.names[field.name]));
                this.logger.push_indent();
                this.logger.prefixed_log_fmt(format_args!("Is mut: {}\n", field.is_mut));
                this.log_visibility(&mut node.vis);
                this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
                this.log_single_indented("Type", |this| this.visit_type(&mut field.ty));
                this.logger.pop_indent();
            })
        });
    }

    fn visit_adt_enum(&mut self, node: &mut AdtEnum, ctx: &mut AdtEnumContext) {
        self.log_node("Adt Enum", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.logger.prefixed_log_fmt(format_args!("Is mut: {}\n", node.is_mut));
            this.logger.prefixed_log_fmt(format_args!("Is record: {}\n", node.is_record));
            this.logger.set_last_at_indent_if(node.attrs.is_empty() && node.generics.is_none() && node.where_clause.is_none() && node.variants.is_empty());
            this.log_visibility(&mut node.vis);
            this.logger.set_last_at_indent_if(node.generics.is_none() && node.where_clause.is_none() && node.variants.is_empty());
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.logger.set_last_at_indent_if(node.where_clause.is_none() && node.variants.is_empty());
            this.log_opt_indented("Generics", &mut node.generics, |this, generics| this.visit_gen_params(generics));
            this.logger.set_last_at_indent_if(node.variants.is_empty());
            this.log_opt_indented("Where Clause", &mut node.where_clause, |this, where_clause| this.visit_where_clause(where_clause));
            this.logger.set_last_at_indent();
            this.log_slice_indented("Variants", &mut node.variants, |this, variant| {
                match variant {
                    AdtEnumVariant::Struct { span, attrs, is_mut, name, fields, discriminant } => this.log_indented("Struct variant", |this| {
                        this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[*name]));
                        this.logger.set_last_at_indent_if(attrs.is_empty() && fields.is_empty() && discriminant.is_none());
                        this.logger.prefixed_log_fmt(format_args!("Is mut: {}\n", is_mut));
                        this.logger.set_last_at_indent_if(fields.is_empty() && discriminant.is_none());
                        this.log_slice_indented("Attributes", attrs, |this, attr| this.visit_attribute(attr));
                        this.logger.set_last_at_indent_if(discriminant.is_none());
                        this.log_slice_indented("Fields", fields, |this, field| {
                            this.logger.prefixed_logln("Field");
                            this.logger.push_indent();
                            this.log_slice_indented("Attributes", &mut field.attrs, |this, attr| this.visit_attribute(attr));
                            this.logger.set_last_at_indent_if(field.def.is_none());
                            this.log_single_indented("Type", |this| this.visit_type(&mut field.ty));
                            this.logger.set_last_at_indent();
                            this.log_opt_indented("Default value", &mut field.def, |this, def| this.visit_expr(def));
                            this.logger.pop_indent();
                        });
                        this.logger.set_last_at_indent();
                        this.log_opt_indented("Discriminant", discriminant, |this, discriminant| this.visit_expr(discriminant));
                    }),
                    AdtEnumVariant::Tuple { span, attrs, is_mut, name, fields, discriminant } => this.log_indented("Tuple struct variant", |this| {
                        this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[*name]));
                        this.logger.set_last_at_indent_if(attrs.is_empty() && fields.is_empty() && discriminant.is_none());
                        this.logger.prefixed_log_fmt(format_args!("Is mut: {}\n", is_mut));
                        this.logger.set_last_at_indent_if(fields.is_empty() && discriminant.is_none());
                        this.log_slice_indented("Attributes", attrs, |this, attr| this.visit_attribute(attr));
                        this.logger.set_last_at_indent_if(discriminant.is_none());
                        this.log_slice_indented("Fields", fields, |this, field| {
                            this.logger.prefixed_logln("Field");
                            this.logger.push_indent();
                            this.log_slice_indented("Attributes", &mut field.attrs, |this, attr| this.visit_attribute(attr));
                            this.logger.set_last_at_indent_if(field.def.is_none());
                            this.log_single_indented("Type", |this| this.visit_type(&mut field.ty));
                            this.logger.set_last_at_indent();
                            this.log_opt_indented("Default value", &mut field.def, |this, def| this.visit_expr(def));
                            this.logger.pop_indent();
                        });
                        this.logger.set_last_at_indent();
                        this.log_opt_indented("Discriminant", discriminant, |this, discriminant| this.visit_expr(discriminant));
                    }),
                    AdtEnumVariant::Fieldless { span, attrs, name, discriminant } => this.log_indented("Fieldless variant", |this| {
                        this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[*name]));
                        this.logger.set_last_at_indent_if(attrs.is_empty() && discriminant.is_none());
                        this.logger.set_last_at_indent_if(discriminant.is_none());
                        this.log_slice_indented("Attributes", attrs, |this, attr| this.visit_attribute(attr));
                        this.logger.set_last_at_indent();
                        this.log_opt_indented("Discriminant", discriminant, |this, discriminant| this.visit_expr(discriminant));
                    }),
                }
            });
        });
    }

    fn visit_flag_enum(&mut self, node: &mut FlagEnum, ctx: &mut FlagEnumContext) {
        self.log_node("Adt Enum", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.log_visibility(&mut node.vis);
            this.logger.set_last_at_indent_if(node.attrs.is_empty() && node.variants.is_empty());
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.logger.set_last_at_indent();
            this.log_slice_indented("Variants", &mut node.variants, |this, variant| {
                this.log_indented("Fieldless variant", |this| {
                    this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[variant.name]));
                    this.logger.set_last_at_indent_if(variant.attrs.is_empty() && variant.discriminant.is_none());
                    this.logger.set_last_at_indent_if(variant.discriminant.is_none());
                    this.log_slice_indented("Attributes", &mut variant.attrs, |this, attr| this.visit_attribute(attr));
                    this.logger.set_last_at_indent();
                    this.log_opt_indented("Discriminant", &mut variant.discriminant, |this, discriminant| this.visit_expr(discriminant));
                });
            });
        });
    }

    fn visit_bitfield(&mut self, node: &mut Bitfield, ctx: &mut BitfieldContext) {
        self.log_node("Bitfield", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.logger.prefixed_log_fmt(format_args!("Is mut: {}\n", node.is_mut));
            this.logger.prefixed_log_fmt(format_args!("Is record: {}\n", node.is_record));
            this.logger.set_last_at_indent_if(node.attrs.is_empty() && node.generics.is_none() && node.where_clause.is_none() && node.fields.is_empty() && node.uses.is_empty());
            this.log_visibility(&mut node.vis);
            this.logger.set_last_at_indent_if(node.generics.is_none() && node.where_clause.is_none() && node.fields.is_empty() && node.uses.is_empty());
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.logger.set_last_at_indent_if(node.where_clause.is_none() && node.fields.is_empty() && node.uses.is_empty());
            this.log_opt_indented("Generics", &mut node.generics, |this, generics| this.visit_gen_params(generics));
            this.logger.set_last_at_indent_if(node.fields.is_empty() && node.uses.is_empty());
            this.log_opt_indented("Where Clause", &mut node.where_clause, |this, where_clause| this.visit_where_clause(where_clause));
            this.logger.set_last_at_indent_if(node.uses.is_empty());
            this.log_slice_indented("Fields", &mut node.fields, |this, field| {
                this.logger.prefixed_logln("Field");
                this.logger.push_indent();
                this.log_visibility(&mut node.vis);
                this.log_slice_indented("Attributes", &mut field.attrs, |this, attr| this.visit_attribute(attr));
                this.logger.set_last_at_indent_if(field.bits.is_none() && field.def.is_none());
                this.log_single_indented("Type", |this| this.visit_type(&mut field.ty));
                this.logger.set_last_at_indent_if(field.def.is_none());
                this.log_opt_indented("Bits", &mut field.bits, |this, bits| this.visit_expr(bits));
                this.logger.set_last_at_indent();
                this.log_opt_indented("Default value", &mut field.def, |this, def| this.visit_expr(def));
                this.logger.pop_indent();
            });
            this.logger.set_last_at_indent();
            this.log_slice_indented("Uses", &mut node.uses, |this, bf_use| {
                this.logger.prefixed_logln("Use");
                this.logger.push_indent();
                this.logger.prefixed_log_fmt(format_args!("Is mut: {}\n", bf_use.is_mut));
                this.log_visibility(&mut bf_use.vis);
                this.log_slice_indented("Attributes", &mut bf_use.attrs, |this, attr| this.visit_attribute(attr));
                this.logger.set_last_at_indent_if(bf_use.bits.is_none());
                this.log_single_indented("Path", |this| this.visit_path(&mut bf_use.path));
                this.logger.set_last_at_indent();
                this.log_opt_indented("Bits", &mut bf_use.bits, |this, bits| this.visit_expr(bits));
                this.logger.pop_indent();
            });
        })
    }

    fn visit_const(&mut self, node: &mut Const, ctx: &mut ConstContext) {
        self.log_node("Const", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.log_visibility(&mut node.vis);
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.log_opt_indented("Type", &mut node.ty, |this, ty| this.visit_type(ty));
            this.logger.set_last_at_indent();
            this.log_single_indented("Value", |this| this.visit_expr(&mut node.val));
        });
    }

    fn visit_static(&mut self, node: &mut Static, ctx: &mut StaticContext) {
        self.log_node("Static", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.log_visibility(&mut node.vis);
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.log_opt_indented("Type", &mut node.ty, |this, ty| this.visit_type(ty));
            this.logger.set_last_at_indent();
            this.log_single_indented("Value", |this| this.visit_expr(&mut node.val));
        });
    }

    fn visit_tls_static(&mut self, node: &mut TlsStatic, ctx: &mut StaticContext) {
        self.log_node("TLS Static", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.logger.prefixed_log_fmt(format_args!("Is mut: {}\n", node.is_mut));
            this.log_visibility(&mut node.vis);
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.log_opt_indented("Type", &mut node.ty, |this, ty| this.visit_type(ty));
            this.logger.set_last_at_indent();
            this.log_single_indented("Value", |this| this.visit_expr(&mut node.val));
        });
    }

    fn visit_extern_static(&mut self, node: &mut ExternStatic, ctx: &mut StaticContext) {
        self.log_node("Static", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.logger.prefixed_log_fmt(format_args!("Abi: {}\n", node.abi));
            this.log_visibility(&mut node.vis);
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.logger.set_last_at_indent();
            this.log_single_indented("Value", |this| this.visit_type(&mut node.ty));
        });
    }

    //--------------------------------------------------------------

    fn visit_trait(&mut self, node: &mut Trait, ctx: &mut TraitContext) {
        // handled in log_trait
    }

    fn visit_trait_function(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitFunction, ctx: &mut FunctionContext) {
        self.log_node("Trait Function", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.logger.prefixed_log_fmt(format_args!("Is const: {}\n", node.is_const));
            this.logger.prefixed_log_fmt(format_args!("Is unsafe: {}\n", node.is_unsafe));
            this.logger.set_last_at_indent_if(node.generics.is_none() && node.params.is_empty() && node.return_ty.is_none() && node.return_ty.is_none() && node.contracts.is_empty() && node.body.is_none());
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.logger.set_last_at_indent_if(node.params.is_empty() && node.return_ty.is_none() && node.return_ty.is_none() && node.contracts.is_empty() && node.body.is_none());
            this.log_opt_indented("Generics", &mut node.generics, |this, generics| this.visit_gen_params(generics));
            this.logger.set_last_at_indent_if(node.params.is_empty() && node.return_ty.is_none() && node.return_ty.is_none() && node.contracts.is_empty() && node.body.is_none());
            this.logger.set_last_at_indent_if(node.return_ty.is_none() && node.return_ty.is_none() && node.contracts.is_empty() && node.body.is_none());
            this.log_slice_indented("Params", &mut node.params, |this, param| this.log_fn_param(param));
            this.logger.set_last_at_indent_if(node.return_ty.is_none() && node.contracts.is_empty() && node.body.is_none());
            this.log_opt_indented("Return Type", &mut node.return_ty, |this, ty| this.visit_type(ty));
            this.logger.set_last_at_indent_if(node.contracts.is_empty() && node.body.is_none());
            this.log_opt_indented("Where Clause", &mut node.where_clause, |this, where_clause| this.visit_where_clause(where_clause));
            this.logger.set_last_at_indent_if(node.body.is_none());
            this.log_slice_indented("Contracts", &mut node.contracts, |this, contract| this.visit_contract(contract));
            this.logger.set_last_at_indent();
            this.log_opt_indented("Body", &mut node.body, |this, body| this.visit_block(body));
        });
    }

    fn visit_trait_method(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitMethod, ctx: &mut FunctionContext) {
        self.log_node("Trait Function", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.logger.prefixed_log_fmt(format_args!("Is const: {}\n", node.is_const));
            this.logger.prefixed_log_fmt(format_args!("Is unsafe: {}\n", node.is_unsafe));
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.log_opt_indented("Generics", &mut node.generics, |this, generics| this.visit_gen_params(generics));
            this.logger.set_last_at_indent_if(node.params.is_empty() && node.return_ty.is_none() && node.where_clause.is_none() && node.contracts.is_empty() && node.body.is_none());
            this.log_fn_receiver(&mut node.receiver);
            this.logger.set_last_at_indent_if(node.return_ty.is_none() && node.where_clause.is_none() && node.contracts.is_empty() && node.body.is_none());
            this.log_slice_indented("Params", &mut node.params, |this, param| this.log_fn_param(param));
            this.logger.set_last_at_indent_if(node.where_clause.is_none() && node.contracts.is_empty() && node.body.is_none());
            this.log_opt_indented("Return Type", &mut node.return_ty, |this, ty| this.visit_type(ty));
            this.logger.set_last_at_indent_if(node.contracts.is_empty() && node.body.is_none());
            this.log_opt_indented("Where Clause", &mut node.where_clause, |this, where_clause| this.visit_where_clause(where_clause));
            this.logger.set_last_at_indent_if(node.body.is_none());
            this.log_slice_indented("Contracts", &mut node.contracts, |this, contract| this.visit_contract(contract));
            this.logger.set_last_at_indent();
            this.log_opt_indented("Body", &mut node.body, |this, body| this.visit_block(body));
        });
    }

    fn visit_trait_type_alias(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitTypeAlias, ctx: &mut TypeAliasContext) {
        self.log_node("Trait Type Alias", node.node_id, |this| {
            this.logger.set_last_at_indent_if(node.attrs.is_empty() && node.generics.is_none() && node.def.is_none());
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.logger.set_last_at_indent_if(node.generics.is_none() && node.def.is_none());
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.logger.set_last_at_indent_if(node.def.is_none());
            this.log_opt_indented("Generics", &mut node.generics, |this, generics| this.visit_gen_params(generics));
            this.logger.set_last_at_indent();
            this.log_opt_indented("Default Type", &mut node.def, |logger, ty| logger.visit_type(ty));
        });
    }
 
    fn visit_trait_const(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitConst, ctx: &mut ConstContext) {
        self.log_node("Trait const", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.logger.set_last_at_indent();
            this.log_indented("Type", |this| this.visit_type(&mut node.ty));
        });
    }

    fn visit_trait_property(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitProperty, ctx: &mut PropertyContext) {
        self.log_node("Trait Property", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.logger.prefixed_log_fmt(format_args!("Is unsafe: {}\n", node.is_unsafe));
            this.logger.set_last_at_indent_if(node.attrs.is_empty());
            this.visit_type(&mut node.ty);
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));

            match &mut node.members {
                TraitPropMembers::Req { get, ref_get, mut_get, set } => {
                    this.logger.set_last_at_indent_if(ref_get.is_none() && mut_get.is_none() && set.is_none());
                    if get.is_some() {
                        this.logger.logln("Get");
                    }
                    this.logger.set_last_at_indent_if(mut_get.is_none() && set.is_none());
                    if ref_get.is_some() {
                        this.logger.logln("Get");
                    }
                    this.logger.set_last_at_indent_if(set.is_none());
                    if mut_get.is_some() {
                        this.logger.logln("Get");
                    }
                    this.logger.set_last_at_indent();
                    if set.is_some() {
                        this.logger.logln("Get");
                    }
                },
                TraitPropMembers::Def { get, ref_get, mut_get, set } => {
                    this.logger.set_last_at_indent_if(ref_get.is_none() && mut_get.is_none() && set.is_none());
                    this.log_opt_indented("Ref Get", ref_get, |this, (_, expr)| this.visit_expr(expr));
                    this.logger.set_last_at_indent_if(mut_get.is_none() && set.is_none());
                    this.log_opt_indented("Mut Get", mut_get, |this, (_, expr)| this.visit_expr(expr));
                    this.logger.set_last_at_indent_if(set.is_none());
                    this.log_opt_indented("Set", set, |this, (_, expr)| this.visit_expr(expr));
                    this.logger.set_last_at_indent();
                },
            }
        });
    }

    //--------------------------------------------------------------

    fn visit_impl(&mut self, node: &mut Impl, ctx: &mut ImplContext) {
        // handled in log_impl
    }

    fn visit_impl_function(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut Function, ctx: &mut FunctionContext) {
        self.log_node("Impl function", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.logger.prefixed_log_fmt(format_args!("Is const: {}\n", node.is_const));
            this.logger.prefixed_log_fmt(format_args!("Is unsafe: {}\n", node.is_unsafe));
            this.logger.prefixed_log_fmt(format_args!("ABI: {}\n", node.abi));
            this.log_visibility(&mut node.vis);
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.log_opt_indented("Generics", &mut node.generics, |this, generics| this.visit_gen_params(generics));
            this.log_slice_indented("Params", &mut node.params, |this, param| this.log_fn_param(param));
            this.log_opt_indented("Return Type", &mut node.return_ty, |this, ty| this.visit_type(ty));
            this.log_opt_indented("Where Clause", &mut node.where_clause, |this, where_clause| this.visit_where_clause(where_clause));
            this.log_slice_indented("Contracts", &mut node.contracts, |this, contract| this.visit_contract(contract));
            this.logger.set_last_at_indent();
            this.log_single_indented("Body", |this| this.visit_block(&mut node.body));
        });
    }

    fn visit_method(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut Method, ctx: &mut FunctionContext) {
        self.log_node("Method", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.logger.prefixed_log_fmt(format_args!("Is const: {}\n", node.is_const));
            this.logger.prefixed_log_fmt(format_args!("Is unsafe: {}\n", node.is_unsafe));
            this.log_visibility(&mut node.vis);
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.log_opt_indented("Generics", &mut node.generics, |this, generics| this.visit_gen_params(generics));
            this.log_fn_receiver(&mut node.receiver);
            this.log_slice_indented("Params", &mut node.params, |this, param| this.log_fn_param(param));
            this.log_opt_indented("Return Type", &mut node.return_ty, |this, ty| this.visit_type(ty));
            this.log_opt_indented("Where Clause", &mut node.where_clause, |this, where_clause| this.visit_where_clause(where_clause));
            this.log_slice_indented("Contracts", &mut node.contracts, |this, contract| this.visit_contract(contract));
            this.logger.set_last_at_indent();
            this.log_single_indented("Body", |this| this.visit_block(&mut node.body));
        });
    }

    fn visit_impl_type_alias(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut TypeAlias, ctx: &mut TypeAliasContext) {
        self.log_node("Type Alias", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.log_visibility(&mut node.vis);
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.log_opt_indented("Generics", &mut node.generics, |this, generics| this.visit_gen_params(generics));
            this.logger.set_last_at_indent();
            this.log_single_indented("Type", |this| this.visit_type(&mut node.ty));
        });
    }

    fn visit_impl_const(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut Const, ctx: &mut ConstContext) {
        self.log_node("Impl const", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.log_visibility(&mut node.vis);
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.log_opt_indented("Type", &mut node.ty, |this, ty| this.visit_type(ty));
            this.logger.set_last_at_indent();
            this.log_single_indented("Value", |this| this.visit_expr(&mut node.val));
        });
    }

    fn visit_impl_static(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut Static, ctx: &mut StaticContext) {
        self.log_node("Impl static", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.log_visibility(&mut node.vis);
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.log_opt_indented("Type", &mut node.ty, |this, ty| this.visit_type(ty));
            this.logger.set_last_at_indent();
            this.log_single_indented("Value", |this| this.visit_expr(&mut node.val));
        });
    }

    fn visit_impl_tls_static(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut TlsStatic, ctx: &mut StaticContext) {
        self.log_node("Impl TLS static", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.logger.prefixed_log_fmt(format_args!("Is mut: {}\n", node.is_mut));
            this.log_visibility(&mut node.vis);
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.log_opt_indented("Type", &mut node.ty, |this, ty| this.visit_type(ty));
            this.logger.set_last_at_indent();
            this.log_single_indented("Value", |this| this.visit_expr(&mut node.val));
        });
    }

    fn visit_property(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut Property, ctx: &mut PropertyContext) {
        self.log_node("Impl Property", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.logger.prefixed_log_fmt(format_args!("Is unsafe: {}\n", node.is_unsafe));
            this.log_visibility(&mut node.vis);
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            
            // One of these should exist, so no need to tell we are at the end of an index before this
            this.logger.set_last_at_indent_if(node.ref_get.is_none() && node.mut_get.is_none() && node.set.is_none());
            this.log_opt_indented("Get", &mut node.get, |this, get| this.visit_expr(get));
            this.logger.set_last_at_indent_if(node.mut_get.is_none() && node.set.is_none());
            this.log_opt_indented("Ref get", &mut node.ref_get, |this, ref_get| this.visit_expr(ref_get));
            this.logger.set_last_at_indent_if(node.set.is_none());
            this.log_opt_indented("Mut get", &mut node.mut_get, |this, mut_get| this.visit_expr(mut_get));
            this.logger.set_last_at_indent();
            this.log_opt_indented("Set", &mut node.set, |this, set| this.visit_expr(set));
        });
    }

    //--------------------------------------------------------------

    fn visit_op_set(&mut self, node: &mut OpSet, ctx: &mut OpSetContext) {
        // handled in log_op_set
    }

    fn visit_operator(&mut self, op_set_ref: Ref<OpSet>, op_set_ctx: Ref<OpSetContext>, node: &mut Operator, ctx: &mut OperatorContext) {
        self.log_node("Op function", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Op type: {}\n", node.op_ty));
            this.logger.prefixed_log_fmt(format_args!("Op: {}\n", node.op.as_str(this.puncts)));
            this.logger.set_last_at_indent_if(node.ret_ty.is_none() && node.def.is_none());
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.logger.set_last_at_indent_if(node.def.is_none());
            this.log_opt_indented("Return Type", &mut node.ret_ty, |this, ty| this.visit_type(ty));
            this.logger.set_last_at_indent();
            this.log_opt_indented("Default impl", &mut node.def, |this, def| this.visit_expr(def));
        });
    }

    fn visit_op_contract(&mut self, op_set_ref: Ref<OpSet>, op_set_ctx: Ref<OpSetContext>, node: &mut OpContract, ctx: &mut OpContractContext) {
        self.log_node("Op contract", node.node_id, |this| {
            this.logger.set_last_at_indent();
            this.visit_expr(&mut node.expr);
        })
    }

    //--------------------------------------------------------------

    fn visit_precedence(&mut self, node: &mut Precedence, ctx: Ref<PrecedenceContext>) {
        self.log_node("Precedence", node.node_id, |this| {
            this.log_visibility(&mut node.vis);
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &self.names[node.name]));
            this.log_visibility(&mut node.vis);
            this.logger.set_last_at_indent_if(node.higher_than.is_none() && node.lower_than.is_none() && node.assoc.is_none());
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.logger.set_last_at_indent_if(node.lower_than.is_none() && node.assoc.is_none());
            if let Some((higher_than, _)) = node.higher_than {
                this.logger.prefixed_log_fmt(format_args!("Higher than: {}\n", &self.names[higher_than]))
            }
            this.logger.set_last_at_indent_if(node.assoc.is_none());
            if let Some((lower_than, _)) = node.lower_than {
                this.logger.prefixed_log_fmt(format_args!("Higher than: {}\n", &self.names[lower_than]))
            }
            this.logger.set_last_at_indent();
            if let Some(assoc) = &node.assoc {
                this.logger.prefixed_log_fmt(format_args!("Associativity: {}\n", assoc.kind))
            }
        })
    }

    // =============================================================

    fn visit_block(&mut self, node: &mut Block) {
        self.log_indented("Block", |this| {
            if !node.stmts.is_empty() {
                let end = node.stmts.len() - 1;
                for (idx, stmt) in node.stmts.iter_mut().enumerate() {
                    if idx == end {
                        this.logger.set_last_at_indent_if(node.expr.is_none());
                    }
                    this.visit_stmt(stmt);
                }
            }
            this.logger.set_last_at_indent();
            this.log_opt_indented("Returning Expression", &mut node.expr, |this, expr| this.visit_expr(expr));
        });
    }

    // =============================================================

    fn visit_stmt(&mut self, stmt: &mut Stmt) {
        helpers::visit_stmt(self, stmt);
    }

    // =============================================================

    fn visit_var_decl(&mut self, node: &mut VarDecl) {
        self.log_node("Var decl", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.logger.prefixed_log_fmt(format_args!("Is mut: {}\n", node.is_mut));
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.log_opt_indented("Type", &mut node.ty, |this, ty| this.visit_type(ty));
            this.logger.set_last_at_indent();
            this.log_single_indented("Value", |this| this.visit_expr(&mut node.expr));
        });
    }

    fn visit_uninit_var_decl(&mut self, node: &mut UninitVarDecl) {
        self.log_node("Var decl", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.logger.prefixed_log_fmt(format_args!("Is mut: {}\n", node.is_mut));
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.logger.set_last_at_indent();
            this.log_single_indented("Type", |this| this.visit_type(&mut node.ty));
        });
    }

    fn visit_defer_stmt(&mut self, node: &mut DeferStmt) {
        self.log_node("Defer", node.node_id, |this| {
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.logger.set_last_at_indent();
            this.log_single_indented("Expr", |this| this.visit_expr(&mut node.expr));
        });
    }

    fn visit_err_defer_stmt(&mut self, node: &mut ErrorDeferStmt) {
        self.log_node("Error defer", node.node_id, |this| {
            this.log_slice_indented("Attributes", &mut node.attrs, |this, attr| this.visit_attribute(attr));
            this.log_opt_indented("Receiver", &mut node.rec, |this, rec| {
                this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[rec.name]));
                this.logger.prefixed_log_fmt(format_args!("Is mut {}\n", rec.is_mut));
            });
            this.logger.set_last_at_indent();
            this.log_single_indented("Expr", |this| this.visit_expr(&mut node.expr));
        });
    }

    fn visit_expr_stmt(&mut self, node: &mut ExprStmt) {
        self.log_node("Expression statement", node.node_id, |this| {
            this.logger.set_last_at_indent();
            this.visit_expr(&mut node.expr);
        })
    }

    // =============================================================

    fn visit_expr(&mut self, expr: &mut Expr) {
        helpers::visit_expr(self, expr);
    }

    fn visit_unit_expr(&mut self, node: &mut UnitExpr) {
        self.log_node("Unit Expr", node.node_id, |_|());
    }

    fn visit_fullrange_expr(&mut self, node: &mut FullRangeExpr) {
        self.log_node("Fullrange Expr", node.node_id, |_|());
    }

    fn visit_underscore_expr(&mut self, node: &mut UnderscoreExpr) {
        self.log_node("Underscore Expr", node.node_id, |_|());
    }

    fn visit_literal_expr(&mut self, node: &mut LiteralExpr) { 
        self.log_node("Literal Expr", node.node_id, |this| {
            this.logger.set_last_at_indent_if(node.lit_op.is_none());
            match node.literal {
                LiteralValue::Lit(lit_id) => this.logger.prefixed_log_fmt(format_args!("Value: {}\n", this.lits[lit_id])),
                LiteralValue::Bool(val) => this.logger.prefixed_log_fmt(format_args!("Value: {val}\n")),
            }
            this.logger.set_last_at_indent();
            if let Some(lit_op) = &node.lit_op {
                match lit_op {
                    LiteralOp::Name(name) => this.logger.prefixed_log_fmt(format_args!("Value: {}\n", &this.names[*name])),
                    LiteralOp::Primitive(ty) => this.logger.prefixed_log_fmt(format_args!("Lit Op: {ty}\n")),
                    LiteralOp::StringSlice(ty) => this.logger.prefixed_log_fmt(format_args!("Lit Op: {ty}\n")),
                }
            }
        });
    }

    fn visit_path_expr(&mut self, node: &mut PathExpr) {
        match node {
            PathExpr::Named { start, iden, .. } => {
                self.log_indented("Named Path Expression", |this| {
                    this.log_path_start(start);
                    this.log_identifier(iden)
                })
            },
            PathExpr::SelfPath { span, node_id } => self.logger.prefixed_logln("`self` Path Expression"),
            PathExpr::Expanded { path } => self.log_indented("Expanded path expression", |this| this.visit_path(path)),
        }
    }

    fn visit_block_expr(&mut self, node: &mut BlockExpr) {
        self.log_node("Block expression", node.node_id, |this| {
            this.logger.prefixed_log("Block kind: ");
            match node.kind {
                BlockKind::Normal => this.logger.logln("Normal"),
                BlockKind::Unsafe => this.logger.logln("Unsafe"),
                BlockKind::Const => this.logger.logln("Const"),
                BlockKind::Try => this.logger.logln("Try"),
                BlockKind::TryUnwrap => this.logger.logln("TryUnwrap"),
                BlockKind::Labeled(label) => this.logger.log_fmt(format_args!("Label: {}", &this.names[label])),
            }
            this.logger.set_last_at_indent();
            this.visit_block(&mut node.block);
        });
    }

    fn visit_prefix_expr(&mut self, node: &mut PrefixExpr) {
        self.log_node("Prefix expression", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Op: {}\n", node.op.as_str(&this.puncts)));
            this.logger.set_last_at_indent();
            this.visit_expr(&mut node.expr);
        });
    }

    fn visit_postfix_expr(&mut self, node: &mut PostfixExpr) {
        self.log_node("Post expression", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Op: {}\n", node.op.as_str(&this.puncts)));
            this.logger.set_last_at_indent();
            this.visit_expr(&mut node.expr);
        });
    }

    fn visit_infix_expr(&mut self, node: &mut InfixExpr) {
        self.log_node("Infix expression", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Op: {}\n", node.op.as_str(&this.puncts)));
            this.visit_expr(&mut node.left);
            this.logger.set_last_at_indent();
            this.visit_expr(&mut node.right);
        });
    }

    fn visit_inplace_expr(&mut self, node: &mut InplaceExpr) {
        self.log_node("Inplace expression", node.node_id, |this| {
            this.visit_expr(&mut node.left);
            this.logger.set_last_at_indent();
            this.visit_expr(&mut node.right);
        });
    }

    fn visit_type_cast_expr(&mut self, node: &mut TypeCastExpr) {
        self.log_node("Type cast expression", node.node_id, |this| {
            this.visit_expr(&mut node.expr);
            this.logger.set_last_at_indent();
            this.visit_type(&mut node.ty);
        });
    }

    fn visit_type_check_expr(&mut self, node: &mut TypeCheckExpr) {
        self.log_node("Type check expression", node.node_id, |this| {
            this.visit_expr(&mut node.expr);
            this.logger.set_last_at_indent();
            this.visit_type(&mut node.ty);
        });
    }

    fn visit_tuple_expr(&mut self, node: &mut TupleExpr) {
        self.log_node("Tuple expression", node.node_id, |this| {
            this.logger.set_last_at_indent();
            this.log_slice(&mut node.exprs, |this, expr| this.visit_expr(expr));
        });
    }

    fn visit_slice_expr(&mut self, node: &mut SliceExpr) {
        self.log_node("Slice expression", node.node_id, |this| {
            this.logger.set_last_at_indent();
            this.log_slice(&mut node.exprs, |this, expr| this.visit_expr(expr));
        });
    }

    fn visit_array_expr(&mut self, node: &mut ArrayExpr) {
        self.log_node("Array expression", node.node_id, |this| {
            this.log_indented("Value", |this| this.visit_expr(&mut node.value));
            this.logger.set_last_at_indent();
            this.log_indented("Value", |this| this.visit_expr(&mut node.count));
        });
    }

    fn visit_struct_expr(&mut self, node: &mut StructExpr) {
        self.log_node("Struct expression", node.node_id, |this| {
            this.logger.set_last_at_indent();
            this.log_single_indented("Struct path", |this| this.visit_expr(&mut node.path))
        });
    }

    fn visit_index_expr(&mut self, node: &mut IndexExpr) {
        self.log_node("Index expression", node.node_id, |this| {
            this.visit_expr(&mut node.expr);
            this.logger.set_last_at_indent();
            this.visit_expr(&mut node.index);
        });
    }

    fn visit_tuple_index_expr(&mut self, node: &mut TupleIndexExpr) {
        self.log_node("Tuple index expression", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Index: {}\n", node.index));
            this.logger.set_last_at_indent();
            this.visit_expr(&mut node.expr);
        });
    }

    fn visit_fn_call_expr(&mut self, node: &mut FnCallExpr) {
        self.log_node("Fn call expression", node.node_id, |this| {
            this.logger.set_last_at_indent_if(node.args.is_empty());
            this.log_single_indented("Function", |this| this.visit_expr(&mut node.func));
            this.logger.set_last_at_indent();
            this.log_slice_indented("Arguments", &mut node.args, |this, arg| {
                this.logger.prefixed_log_fmt(format_args!("Argument {}\n", arg.label.map_or("", |label| &this.names[label])));
                this.logger.push_indent();
                this.logger.set_last_at_indent();
                this.visit_expr(&mut arg.expr);
                this.logger.pop_indent();
            });
        });
    }

    fn visit_method_call_expr(&mut self, node: &mut MethodCallExpr) {
        self.log_node("Method call expression", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Is propagating: {}\n", node.is_propagating));
            this.log_single_indented("Receiver", |this| this.visit_expr(&mut node.receiver));
            this.log_indented("Method", |this| this.log_identifier(&mut node.method));
            this.logger.set_last_at_indent();
            this.log_slice_indented("Arguments", &mut node.args, |this, arg| {
                this.logger.prefixed_log_fmt(format_args!("Argument {}\n", arg.label.map_or("", |label| &this.names[label])));
                this.logger.push_indent();
                this.logger.set_last_at_indent();
                this.visit_expr(&mut arg.expr);
                this.logger.pop_indent();
            });
        });
    }

    fn visit_field_access_expr(&mut self, node: &mut FieldAccessExpr) {
        self.log_node("Field access expressions", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Is propagating: {}\n", node.is_propagating));
            this.log_indented("Field", |this| this.log_identifier(&mut node.field));
            this.logger.set_last_at_indent();
            this.log_single_indented("Expr", |this| this.visit_expr(&mut node.expr));
        });
    }

    fn visit_closure_expr(&mut self, node: &mut ClosureExpr) {
        self.log_node("Closure expression", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Is moved: {}\n", node.is_moved));
            // TODO
        });
    }

    fn visit_loop_expr(&mut self, node: &mut LoopExpr) {
        self.log_node("Loop expression", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Label: {}\n", node.label.map_or("<none>", |label| &this.names[label])));
            this.visit_block(&mut node.body);
        });
    }

    fn visit_match_expr(&mut self, node: &mut MatchExpr) {
        self.log_node("Match expression", node.node_id, |this| {
            this.log_indented("Scrutinee", |this| this.visit_expr(&mut node.scrutinee));
            this.log_slice_indented("Branches", &mut node.branches, |this, branch| {
                this.log_indented("Branch", |this| {
                    this.log_single_indented("Pattern", |this| this.visit_pattern(&mut branch.pattern));
                    this.log_opt_indented("Guard", &mut branch.guard, |this, guard| this.visit_expr(guard));
                    this.log_single_indented("Body", |this| this.visit_expr(&mut branch.body));
                });
            });
        });
    }

    fn visit_break_expr(&mut self, node: &mut BreakExpr) {
        self.log_node("Break expression", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Label: {}\n", node.label.map_or("<none>", |label| &this.names[label])));
            if let Some(expr) = &mut node.value {
                this.visit_expr(expr);
            }
        });
    }

    fn visit_continue_expr(&mut self, node: &mut ContinueExpr) {
        self.log_node("Continue expression", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Label: {}\n", node.label.map_or("<none>", |label| &this.names[label])));
        });
    }

    fn visit_fallthrough_expr(&mut self, node: &mut FallthroughExpr) {
        self.log_node("Fallthrough expression", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Label: {}\n", node.label.map_or("<none>", |label| &this.names[label])));
        });
    }

    fn visit_return_expr(&mut self, node: &mut ReturnExpr) {
        self.log_node("Return expression", node.node_id, |this| {
            if let Some(expr) = &mut node.value {
                this.visit_expr(expr);
            }
        })
    }

    fn visit_throw_expr(&mut self, node: &mut ThrowExpr) {
        self.log_node("Throw expression", node.node_id, |this| {
            this.visit_expr(&mut node.expr);
        });
    }

    fn visit_comma_expr(&mut self, node: &mut CommaExpr) {
        self.log_node("Comma expression", node.node_id, |this| {
            this.log_slice(&mut node.exprs, |this, expr| this.visit_expr(expr));
        })
    }

    fn visit_when_expr(&mut self, node: &mut WhenExpr) {
        self.log_node("When expression", node.node_id, |this| {
            this.log_single_indented("Condition", |this| this.visit_expr(&mut node.cond));
            this.logger.set_last_at_indent_if(node.else_body.is_none());
            this.log_single_indented("Body", |this| this.visit_block(&mut node.body));
            this.logger.set_last_at_indent();
            this.log_opt_indented("Else body", &mut node.else_body, |this, body| this.visit_block(body));
        });
    }

    fn visit_irrefutable_expr(&mut self) {
        self.logger.prefixed_logln("Irrifutable expression");
    }

    // =============================================================

    fn visit_pattern(&mut self, node: &mut Pattern) {
        helpers::visit_pattern(self, node);
    }

    fn visit_wildcard_pattern(&mut self, node: &mut WildcardPattern) {
        self.log_node("Wildcard pattern", node.node_id, |_|());
    }

    fn visit_rest_pattern(&mut self, node: &mut RestPattern) {
        self.log_node("Rest pattern", node.node_id, |_|());
    }

    fn visit_literal_pattern(&mut self, node: &mut LiteralPattern) {
        self.log_node("Literal Pattern", node.node_id, |this| {
            this.logger.set_last_at_indent_if(node.lit_op.is_none());
            match node.literal {
                LiteralValue::Lit(lit_id) => this.logger.prefixed_log_fmt(format_args!("Value: {}\n", this.lits[lit_id])),
                LiteralValue::Bool(val) => this.logger.prefixed_log_fmt(format_args!("Value: {val}\n")),
            }
            this.logger.set_last_at_indent();
            if let Some(lit_op) = &node.lit_op {
                match lit_op {
                    LiteralOp::Name(name) => this.logger.prefixed_log_fmt(format_args!("Value: {}\n", &this.names[*name])),
                    LiteralOp::Primitive(ty) => this.logger.prefixed_log_fmt(format_args!("Lit Op: {ty}\n")),
                    LiteralOp::StringSlice(ty) => this.logger.prefixed_log_fmt(format_args!("Lit Op: {ty}\n")),
                }
            }
        });
    }

    fn visit_iden_pattern(&mut self, node: &mut IdenPattern) {
        self.log_node("Identifier pattern", node.node_id, |this| {
            this.logger.log_fmt(format_args!("Is ref: {}\n", node.is_ref));
            this.logger.log_fmt(format_args!("Is mut: {}\n", node.is_mut));
            this.logger.log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
            this.logger.set_last_at_indent();
            this.log_opt_indented("Bound", &mut node.bound, |this, bound| this.visit_pattern(bound));
        });
    }

    fn visit_path_pattern(&mut self, node: &mut PathPattern) {
        self.log_node("Path pattern", node.node_id, |this| {
            this.logger.set_last_at_indent();
            this.visit_path(&mut node.path);
        });
    }

    fn visit_range_pattern(&mut self, node: &mut RangePattern) {
        match node {
            RangePattern::Exclusive { node_id, begin, end, .. } => self.log_node("Exclusive range expression", *node_id, |this| {
                this.visit_pattern(begin);
                this.logger.set_last_at_indent();
                this.visit_pattern(end);
            }),
            RangePattern::Inclusive { node_id, begin, end, .. } => self.log_node("Inclusive range expression", *node_id, |this| {
                this.visit_pattern(begin);
                this.logger.set_last_at_indent();
                this.visit_pattern(end);
            }),
            RangePattern::From { node_id, begin, .. } => self.log_node("From range expression", *node_id, |this| {
                this.logger.set_last_at_indent();
                this.visit_pattern(begin);
            }),
            RangePattern::To { node_id, end, .. } => self.log_node("To range expression", *node_id, |this| {
                this.logger.set_last_at_indent();
                this.visit_pattern(end);
            }),
            RangePattern::InclusiveTo { node_id, end, .. } => self.log_node("Inclusive to range expression", *node_id, |this| {
                this.logger.set_last_at_indent();
                this.visit_pattern(end);
            }),
        }
    }

    fn visit_reference_pattern(&mut self, node: &mut ReferencePattern) {
        self.log_node("Reference pattern", node.node_id, |this| {
            this.logger.log_fmt(format_args!("Is mut: {}\n", node.is_mut));
            this.visit_pattern(&mut node.pattern);
        });
    }

    fn visit_struct_pattern(&mut self, node: &mut StructPattern) {
        self.log_node("Struct pattern", node.node_id, |this| {
            match &mut node.path {
                Some(path) => this.visit_path(path),
                None => this.logger.prefixed_logln("Inferred path"),
            }
            this.log_slice_indented("Fields", &mut node.fields, |this, field| {
                match field {
                    StructPatternField::Named { node_id, name, pattern, .. } => this.log_node("Named field", *node_id, |this| {
                        this.logger.log_fmt(format_args!("Name: {}\n", &this.names[*name]));
                        this.logger.set_last_at_indent();
                        this.visit_pattern(pattern);
                    }),
                    StructPatternField::TupleIndex { node_id, index, pattern, .. } => this.log_node("Tuple index field", *node_id, |this| {
                        this.logger.log_fmt(format_args!("Index: {}\n", index));
                        this.logger.set_last_at_indent();
                        this.visit_pattern(pattern);
                    }),
                    StructPatternField::Iden { node_id, is_ref, is_mut, iden, bound, .. } => this.log_node("Iden field", *node_id, |this| {
                        this.logger.log_fmt(format_args!("Is ref: {}\n", is_ref));
                        this.logger.log_fmt(format_args!("Is mut: {}\n", is_mut));
                        this.logger.log_fmt(format_args!("Name: {}\n", &this.names[*iden]));
                        this.log_opt_indented("Bound", bound, |this, bound| this.visit_pattern(bound));
                    }),
                    StructPatternField::Rest => this.logger.prefixed_logln("Rest field"),
                }
            })
        });
    }

    fn visit_tuple_struct_pattern(&mut self, node: &mut TupleStructPattern) {
        self.log_node("Tuple struct pattern", node.node_id, |this| {
            match &mut node.path {
                Some(path) => this.visit_path(path),
                None => this.logger.prefixed_logln("Inferred path"),
            }
            this.log_slice_indented("Field", &mut node.patterns, |this, pattern| this.visit_pattern(pattern))
        });
    }

    fn visit_tuple_pattern(&mut self, node: &mut TuplePattern) {
        self.log_node("Tuple pattern", node.node_id, |this| {
            this.log_slice(&mut node.patterns, |this, pattern| this.visit_pattern(pattern));
        });
    }

    fn visit_slice_pattern(&mut self, node: &mut SlicePattern) {
        self.log_node("Slice pattern", node.node_id, |this| {
            this.log_slice(&mut node.patterns, |this, pattern| this.visit_pattern(pattern));
        });
    }

    fn visit_enum_member_pattern(&mut self, node: &mut EnumMemberPattern) {
        self.log_node("Enum member pattern", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[node.name]));
        });
    }

    fn visit_alternative_pattern(&mut self, node: &mut AlternativePattern) {
        self.log_node("Alternative pattern", node.node_id, |this| {
            this.log_slice(&mut node.patterns, |this, pattern| this.visit_pattern(pattern));
        });
    }

    fn visit_type_check_pattern(&mut self, node: &mut TypeCheckPattern) {
        self.log_node("Type check pattern", node.node_id, |this| {
            this.visit_type(&mut node.ty);
        });
    }

    // =============================================================

    fn visit_type(&mut self, node: &mut Type) {
        helpers::visit_type(self, node)
    }

    fn visit_unit_type(&mut self, node: &mut UnitType) {
        self.log_node("Unit type", node.node_id, |_|());
    }

    fn visit_never_type(&mut self, node: &mut NeverType) {
        self.log_node("Never type", node.node_id, |_|());
    }

    fn visit_primitive_type(&mut self, node: &mut PrimitiveType) {
        self.log_node("Primitive type", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Type: {}\n", node.ty));
        });
    }

    fn visit_path_type(&mut self, node: &mut PathType) {
        self.log_node("Path type", node.node_id, |this| {
            this.logger.set_last_at_indent();
            this.visit_path(&mut node.path);
        });
    }

    fn visit_tuple_type(&mut self, node: &mut TupleType) {
        self.log_node("Tuple type", node.node_id, |this| {
            this.log_slice(&mut node.types, |this, ty| {
                this.visit_type(ty);
            });
        });
    }

    fn visit_array_type(&mut self, node: &mut ArrayType) {
        self.log_node("Array type", node.node_id, |this| {
            this.log_single_indented("Type", |this| this.visit_type(&mut node.ty));
            this.logger.set_last_at_indent_if(node.sentinel.is_none());
            this.log_single_indented("Size", |this| this.visit_expr(&mut node.size));
            this.logger.set_last_at_indent();
            this.log_opt_indented("Sentinel", &mut node.sentinel, |this, expr| this.visit_expr(expr));
        });
    }

    fn visit_slice_type(&mut self, node: &mut SliceType) {
        self.log_node("Slice type", node.node_id, |this| {
            this.logger.set_last_at_indent_if(node.sentinel.is_none());
            this.log_single_indented("Type", |this| this.visit_type(&mut node.ty));
            this.logger.set_last_at_indent();
            this.log_opt_indented("Sentinel", &mut node.sentinel, |this, expr| this.visit_expr(expr));
        });
    }

    fn visit_string_slice_type(&mut self, node: &mut StringSliceType) {
        self.log_node("String slice type", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Type: {}\n", node.ty));
        });
    }

    fn visit_pointer_type(&mut self, node: &mut PointerType) {
        self.log_node("Pointer type", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Is multi: {}", node.is_multi));
            this.logger.prefixed_log_fmt(format_args!("Is mut: {}", node.is_mut));
            this.logger.set_last_at_indent_if(node.sentinel.is_none());
            this.log_single_indented("Type", |this| this.visit_type(&mut node.ty));
            this.logger.set_last_at_indent();
            this.log_opt_indented("Sentinel", &mut node.sentinel, |this, expr| this.visit_expr(expr));
        });
    }

    fn visit_reference_type(&mut self, node: &mut ReferenceType) {
        self.log_node("Reference type", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Is mut: {}", node.is_mut));
            this.logger.set_last_at_indent();
            this.log_single_indented("Type", |this| this.visit_type(&mut node.ty));
        });
    }

    fn visit_optional_type(&mut self, node: &mut OptionalType) {
        self.log_node("Optional type", node.node_id, |this| {
            this.logger.set_last_at_indent();
            this.log_single_indented("Type", |this| this.visit_type(&mut node.ty));
        });
    }

    fn visit_fn_type(&mut self, node: &mut FnType) {
        self.log_node("Function type", node.node_id, |this| {
            this.logger.prefixed_log_fmt(format_args!("Is unsafe: {}", node.is_unsafe));
            this.logger.prefixed_log_fmt(format_args!("Abi: {}", node.abi));
            this.logger.set_last_at_indent_if(node.return_ty.is_none());
            this.log_slice_indented("Parameters", &mut node.params, |this, (name, ty)| {
                this.logger.prefixed_log_fmt(format_args!("Parameter: {}\n", &this.names[*name]));
                this.logger.push_indent();
                this.logger.set_last_at_indent();
                this.visit_type(ty);
                this.logger.pop_indent();
            });
            this.logger.set_last_at_indent();
            this.log_opt_indented("Return type", &mut node.return_ty, |this, ty| this.visit_type(ty));
        });
    }

    // =============================================================

    fn visit_gen_params(&mut self, node: &mut GenericParams) {
        self.log_node("Generic Parameters", node.node_id, |this| {
            let end = node.params.len() - 1;
            for (idx, param) in &mut node.params.iter_mut().enumerate() {
                this.logger.set_last_at_indent_if(idx == end && node.pack.is_none());

                match param {
                    GenericParam::Type(param) => this.log_indented("Generic Type Parameter", |this| {
                        this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &self.names[param.name]));
                        this.logger.set_last_at_indent();
                        this.log_opt_indented("Default Type", &mut param.def, |this, def| this.visit_type(def));
                    }),
                    GenericParam::TypeSpec(param) => this.log_indented("Generic Type Specialization", |this| this.visit_type(&mut param.ty)),
                    GenericParam::Const(param) => this.log_indented("Generic Constant Parameter", |this| {
                        this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[param.name]));
                        this.logger.set_last_at_indent_if(param.def.is_none());
                        this.log_indented("Type", |this| this.visit_type(&mut param.ty));
                        this.logger.set_last_at_indent();
                        this.log_opt_indented("Default Value", &mut param.def, |this, expr| this.visit_expr(expr));
                    }),
                    GenericParam::ConstSpec(param) => this.log_indented("Generic Constant Specialization", |this| this.visit_block(&mut param.expr)),
                }
            }
            if let Some(pack) = &mut node.pack {
                this.log_indented("Parameter Pack", |this| {
                    for elem in &mut pack.elems {
                        match elem {
                            GenericParamPackElem::Type { name, defs, .. } => this.log_indented("Parameter Pack Type Element", |this| {
                                this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[*name]));
                                this.logger.set_last_at_indent();
                                if !defs.is_empty() {
                                    this.log_slice_indented("Default Types", defs, |this, def| {
                                        this.visit_type(def);
                                    });
                                }
                            }),
                            GenericParamPackElem::Const { name, ty, defs, .. } => this.log_indented("Parameter Pack Expression Type Element", |this| {
                                this.logger.prefixed_log_fmt(format_args!("Name: {}\n", &this.names[*name]));
                                this.log_indented("Type", |this| this.visit_type(ty));
                                if !defs.is_empty() {
                                    this.log_slice_indented("Default Values", defs, |this, def| {
                                        this.visit_expr(def);
                                    });
                                }
                            }),
                        }
                    }
                })
            }
        })
    }

    fn visit_gen_args(&mut self, node: &mut GenericArgs) {
        self.log_node("Generic Arguments", node.node_id, |this| {
            let end = node.args.len() - 1;
            for (idx, arg) in node.args.iter_mut().enumerate() {
                this.logger.set_last_at_indent_if(idx == end);
                match arg {
                    GenericArg::Type(ty) => this.log_indented("Generic Type Argument", |this| this.visit_type(ty)),
                    GenericArg::Value(val) => this.log_indented("Generic Constant Argument", |this| this.visit_expr(val)),
                    GenericArg::Name(_, name) => this.logger.prefixed_log_fmt(format_args!("Generic Name Argument: {}\n", &this.names[*name])),
                }
            }
        })
    }

    fn visit_where_clause(&mut self, node: &mut WhereClause) {
        self.log_node("Trait Bounds", node.node_id, |this| {
            let end = node.bounds.len() - 1;
            for (idx, bound) in node.bounds.iter_mut().enumerate() {
                this.logger.set_last_at_indent_if(idx == end);
                match bound {
                    WhereBound::Type { span, ty, bounds } => this.log_indented("Type Bound", |this| {
                        this.log_indented("Type", |this| this.visit_type(ty));
                        this.log_slice_indented("Bounds", bounds, |this, bound| {
                            this.visit_path(bound);
                        });
                    }),
                    WhereBound::Explicit { span, ty, bounds } => this.log_indented("Explicit Type Bound", |this| {
                        this.log_indented("Type", |this| this.visit_type(ty));
                        this.log_slice_indented("Bounds", bounds, |this, bound| {
                            this.visit_type(bound);
                        });
                    }),
                    WhereBound::Expr { expr } => this.log_indented("Expression Bound", |this| this.visit_expr(expr)),
                }
            }
        })
    }

    fn visit_trait_bounds(&mut self, node: &mut TraitBounds) {
        self.log_node("Trait Bounds", node.node_id, |this| {
            let end = node.bounds.len() - 1;
            for (idx, bound) in node.bounds.iter_mut().enumerate() {
                this.logger.set_last_at_indent_if(idx == end);
                this.visit_path(bound);
            }
        })
    }

    fn visit_contract(&mut self, node: &mut Contract) {

    }

    fn visit_attribute(&mut self, node: &mut Attribute) {
        self.log_node("Attribute", node.node_id, |this| {
            this.logger.set_last_at_indent_if(node.metas.is_empty());
            this.log_indented("Path", |this| this.visit_simple_path(&mut node.path));
            this.logger.set_last_at_indent();
            this.log_slice_indented("Attributes", &mut node.metas, |this, meta| this.log_attr_meta(meta));
        })
    }
}