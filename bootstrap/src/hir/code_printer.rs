use crate::{
    common::{IndentLogger, NameTable},
    lexer::PuncutationTable,
    literals::LiteralTable
};

use super::*;





pub struct CodePrinter<'a> {
    pub logger: IndentLogger,
    pub names:  &'a NameTable,
    pub lits:   &'a LiteralTable,
    pub puncts: &'a PuncutationTable,
}

impl<'a> CodePrinter<'a> {
    pub fn new(names: &'a NameTable, lits: &'a LiteralTable, puncts: &'a PuncutationTable) -> Self {
        let mut logger = IndentLogger::new("    ", "    ", "    ");
        logger.pop_indent();

        Self {
            logger,
            names,
            lits,
            puncts,
        }
    }

    pub fn log_fn_param(&mut self, param: &mut FnParam) {
        match param {
            FnParam::Param { span, attrs, label, pattern, ty } => {
                for attr in attrs {
                    self.visit_attribute(attr);
                }
                if let Some(label) = *label {
                    self.logger.prefixed_log_fmt(format_args!(":{} ", &self.names[label]));
                }
                self.visit_pattern(pattern);
                self.logger.log(" : ");
                self.visit_type(ty);
            },
            FnParam::Opt { span, attrs, label, pattern, ty, def } => {
                for attr in attrs {
                    self.visit_attribute(attr);
                }
                if let Some(label) = *label {
                    self.logger.prefixed_log_fmt(format_args!(":{} ", &self.names[label]));
                }
                self.visit_pattern(pattern);
                self.logger.log(" : ");
                self.visit_type(ty);
                self.logger.log(" = ");
                self.visit_expr(def);
            },
            FnParam::Variadic { span, attrs, name, ty } => {
                for attr in attrs {
                    self.visit_attribute(attr);
                }
                self.logger.prefixed_log_fmt(format_args!("{}: ", &self.names[*name]));
                self.visit_type(ty);
            },
        }
        self.logger.logln(",");
    }

    pub fn log_struct_field(&mut self, field: &mut StructField) {
        for attr in &mut field.attrs {
            self.visit_attribute(attr);
        }
        self.log_vis(&mut field.vis);
        self.logger.log_fmt(format_args!("{}{}: ", 
            if field.is_mut { "mut " } else { "" },
            &self.names[field.name]
        ));
        self.visit_type(&mut field.ty);
        if let Some(def) = &mut field.def {
            self.logger.log(" = ");
            self.visit_expr(def);
        }
        self.logger.logln(",");
    }

    pub fn log_tuple_struct_field(&mut self, field: &mut TupleStructField) {
        for attr in &mut field.attrs {
            self.visit_attribute(attr);
        }
        self.log_vis(&mut field.vis);
        self.visit_type(&mut field.ty);
        if let Some(def) = &mut field.def {
            self.logger.log(" = ");
            self.visit_expr(def);
        }
        self.logger.logln(",");
    }

    pub fn log_trait(&mut self, hir: &mut Hir, idx: usize) {
        let (trait_ref, trait_ctx) = &hir.traits[idx];

        let mut node = trait_ref.write();
        for attr in &mut node.attrs {
            self.visit_attribute(attr);
        }
        self.log_vis(&mut node.vis);
        self.logger.log_fmt(format_args!("{}{}trait {}",
            if node.is_unsafe { "unsafe " } else { "" },
            if node.is_sealed { "sealed " } else { "" },
            &self.names[node.name]
        ));
        if let Some(bounds) = &mut node.bounds {
            self.logger.log(" : ");
            self.visit_trait_bounds(bounds);
        }
        self.logger.logln("{");

        let type_aliases_count = hir.trait_type_alias.iter().filter(|(search_idx, _, _)| *search_idx == idx).count();
        let funcs_count = hir.trait_functions.iter().filter(|(search_idx, _, _)| *search_idx == idx).count();
        let consts_count = hir.trait_consts.iter().filter(|(search_idx, _, _)| *search_idx == idx).count();
        let props_count = hir.trait_properties.iter().filter(|(search_idx, _, _)| *search_idx == idx).count();

        self.logger.push_indent();
        let mut count = 0;
        for (fn_idx, node, ctx) in &mut hir.trait_type_alias {
            if *fn_idx == idx {
                self.logger.set_last_at_indent_if(count == type_aliases_count - 1 && consts_count == 0 && funcs_count == 0 && props_count == 0);
                self.visit_trait_type_alias(trait_ref.clone(), trait_ctx.clone(), node, ctx);
                count += 1;
            }
        }
        count = 0;
        for (fn_idx, node, ctx) in &mut hir.trait_consts {
            if *fn_idx == idx {
                self.logger.set_last_at_indent_if(count == consts_count - 1 && funcs_count == 0 && props_count == 0);
                self.visit_trait_const(trait_ref.clone(), trait_ctx.clone(), node, ctx);
                count += 1;
            }
        }
        
        count = 0;
        for (fn_idx, node, ctx) in &mut hir.trait_functions {
            if *fn_idx == idx {
                self.logger.set_last_at_indent_if(count == funcs_count - 1 && props_count == 0);
                self.visit_trait_function(trait_ref.clone(), trait_ctx.clone(), node, ctx);
                count += 1;
            }
        }
        
        count = 0;
        for (fn_idx, node, ctx) in &mut hir.trait_properties {
            if *fn_idx == idx {
                self.logger.set_last_at_indent_if(count == props_count - 1);
                self.visit_trait_property(trait_ref.clone(), trait_ctx.clone(), node, ctx);
                count += 1;
            }
        }
        self.logger.pop_indent();
        self.logger.prefixed_logln("}");
    }

    pub fn log_impl(&mut self, hir: &mut Hir, idx: usize) {
        let (impl_ref, impl_ctx) = &hir.impls[idx];

        let mut node = impl_ref.write();
        for attr in &mut node.attrs {
            self.visit_attribute(attr);
        }
        self.log_vis(&mut node.vis);
        self.logger.log_fmt(format_args!("{}impl",
            if node.is_unsafe { "unsafe " } else { "" }
        ));
        if let Some(generics) = &mut node.generics {
            self.visit_gen_params(generics);
        }

        self.logger.log(" ");
        self.visit_type(&mut node.ty);
        
        if let Some(impl_trait) = &mut node.impl_trait {
            self.logger.log(" as ");
            self.visit_type_path(impl_trait);
        }
        self.logger.logln("{");

        let funcs_count = hir.trait_functions.iter().filter(|(search_idx, _, _)| *search_idx == idx).count();
        let method_count = hir.methods.iter().filter(|(search_idx, _, _)| *search_idx == idx).count();
        let type_aliases_count = hir.impl_type_aliases.iter().filter(|(search_idx, _, _)| *search_idx == idx).count();
        let consts_count = hir.impl_consts.iter().filter(|(search_idx, _, _)| *search_idx == idx).count();
        let static_count = hir.impl_statics.iter().filter(|(search_idx, _, _)| *search_idx == idx).count();
        let tls_static_count = hir.impl_tls_statics.iter().filter(|(search_idx, _, _)| *search_idx == idx).count();
        let props_count = hir.properties.iter().filter(|(search_idx, _, _)| *search_idx == idx).count();

        self.logger.push_indent();
        let mut count = 0;
        for (fn_idx, node, ctx) in &mut hir.impl_functions {
            if *fn_idx == idx {
                self.logger.set_last_at_indent_if(count == funcs_count - 1 && method_count == 0 && type_aliases_count == 0 && consts_count == 0 && static_count == 0 && tls_static_count == 0 && props_count == 0);
                self.visit_impl_function(impl_ref.clone(), impl_ctx.clone(), node, ctx);
                count += 1;
            }
        }
        count = 0;
        for (fn_idx, node, ctx) in &mut hir.methods {
            if *fn_idx == idx {
                self.logger.set_last_at_indent_if(count == method_count - 1 && type_aliases_count == 0 && consts_count == 0 && static_count == 0 && tls_static_count == 0 && props_count == 0);
                self.visit_method(impl_ref.clone(), impl_ctx.clone(), node, ctx);
                count += 1;
            }
        }
        count = 0;
        for (fn_idx, node, ctx) in &mut hir.impl_type_aliases {
            if *fn_idx == idx {
                self.logger.set_last_at_indent_if(count == type_aliases_count - 1 && consts_count == 0 && static_count == 0 && tls_static_count == 0 && props_count == 0);
                self.visit_impl_type_alias(impl_ref.clone(), impl_ctx.clone(), node, ctx);
                count += 1;
            }
        }
        count = 0;
        for (fn_idx, node, ctx) in &mut hir.impl_consts {
            if *fn_idx == idx {
                self.logger.set_last_at_indent_if(count == consts_count - 1 && static_count == 0 && tls_static_count == 0 && props_count == 0);
                self.visit_impl_const(impl_ref.clone(), impl_ctx.clone(), node, ctx);
                count += 1;
            }
        }
        count = 0;
        for (fn_idx, node, ctx) in &mut hir.impl_statics {
            if *fn_idx == idx {
                self.logger.set_last_at_indent_if(count == static_count - 1 && tls_static_count == 0 && props_count == 0);
                self.visit_impl_static(impl_ref.clone(), impl_ctx.clone(), node, ctx);
                count += 1;
            }
        }
        count = 0;
        for (fn_idx, node, ctx) in &mut hir.impl_tls_statics {
            if *fn_idx == idx {
                self.logger.set_last_at_indent_if(count == tls_static_count - 1 && props_count == 0);
                self.visit_impl_tls_static(impl_ref.clone(), impl_ctx.clone(), node, ctx);
                count += 1;
            }
        }
        count = 0;
        for (fn_idx, node, ctx) in &mut hir.properties {
            if *fn_idx == idx {
                self.logger.set_last_at_indent_if(count == props_count - 1);
                self.visit_property(impl_ref.clone(), impl_ctx.clone(), node, ctx);
                count += 1;
            }
        }
        self.logger.pop_indent();
        self.logger.prefixed_logln("}");
    }

    pub fn log_op_trait(&mut self, hir: &mut Hir, idx: usize) {
        let (trait_ref, trait_ctx) = &hir.op_traits[idx];

        let mut node = trait_ref.write();
        for attr in &mut node.attrs {
            self.visit_attribute(attr);
        }
        self.log_vis(&mut node.vis);
        self.logger.log_fmt(format_args!("op trait {}", &self.names[node.name] ));
        if !node.bases.is_empty() {
            self.logger.log(" : ");
            for (idx, base) in node.bases.iter_mut().enumerate() {
                if idx != 0 {
                    self.logger.log(" + ");
                }
                self.visit_simple_path(base)
            }
        }
        self.logger.logln(" {");

        let funcs_count = hir.op_functions.iter().filter(|(search_idx, _, _)| *search_idx == idx).count();
        let spec_count = hir.op_specializations.iter().filter(|(search_idx, _, _)| *search_idx == idx).count();
        let contract_count = hir.op_contracts.iter().filter(|(search_idx, _, _)| *search_idx == idx).count();

        self.logger.push_indent();
        let mut count = 0;
        for (fn_idx, node, ctx) in &mut hir.op_functions {
            if *fn_idx == idx {
                self.logger.set_last_at_indent_if(count == funcs_count - 1 && spec_count == 0 && contract_count == 0);
                self.visit_op_function(trait_ref.clone(), trait_ctx.clone(), node, ctx);
                count += 1;
            }
        }
        count = 0;
        for (fn_idx, node, ctx) in &mut hir.op_specializations {
            if *fn_idx == idx {
                self.logger.set_last_at_indent_if(count == spec_count - 1 && funcs_count == 0 && contract_count == 0);
                self.visit_op_specialization(trait_ref.clone(), trait_ctx.clone(), node, ctx);
                count += 1;
            }
        }
        
        count = 0;
        for (fn_idx, node, ctx) in &mut hir.op_contracts {
            if *fn_idx == idx {
                self.logger.set_last_at_indent_if(count == contract_count - 1);
                self.visit_op_contract(trait_ref.clone(), trait_ctx.clone(), node, ctx);
                count += 1;
            }
        }
        self.logger.pop_indent();
        self.logger.prefixed_logln("}");
    }
}

impl CodePrinter<'_> {
    pub fn log_vis(&mut self, vis: &mut Visibility) {
        match vis {
            Visibility::Priv =>    self.logger.prefixed_log(""),
            Visibility::Pub{ .. } =>     self.logger.prefixed_log("pub "),
            Visibility::Lib{ .. } =>     self.logger.prefixed_log("pub(lib) "),
            Visibility::Package{ .. } => self.logger.prefixed_log("pub(lib) "),
            Visibility::Super{ .. } =>   self.logger.prefixed_log("pub(lib) "),
            Visibility::Path{ path, .. } => {
                self.logger.prefixed_log("pub(");
                self.visit_simple_path(path);
                self.logger.prefixed_log(") ");
            },
        }
    }
}

impl Visitor for CodePrinter<'_> {
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
            VisitFlags::OpTrait |
            VisitFlags::OpFunction |
            VisitFlags::OpSpecialization |
            VisitFlags::OpContract;

        let ignore_flags = ignore_trait_flags | ignore_impl_flags | ignore_op_flags;
        helpers::visit(self, hir, flags & !ignore_flags);

        for idx in 0..hir.traits.len() {
            self.log_trait(hir, idx);
        }

        for idx in 0..hir.impls.len() {
            self.log_impl(hir, idx);
        }

        for idx in 0..hir.op_traits.len() {
            self.log_op_trait(hir, idx);
        }
    }

    fn visit_type_path(&mut self, path: &mut TypePath) {
        for (idx, segment) in path.segments.iter_mut().enumerate() {
            if idx != 0 {
                self.logger.log(".");
            }
            match segment {
                TypePathSegment::Plain { name, .. } => self.logger.log(&self.names[*name]),
                TypePathSegment::GenArg { name, gen_args, .. } => {
                    self.logger.log(&self.names[*name]);
                    self.visit_gen_args(gen_args);
                },
                TypePathSegment::Fn { name, params, ret, .. } => {
                    self.logger.log_fmt(format_args!("{}(", &self.names[*name]));
                    let end = params.len() - 1;
                    for (idx, ty) in params.iter_mut().enumerate() {
                        self.visit_type(ty);
                        if idx != end {
                            self.logger.log(", ")
                        }
                    }
                    self.logger.log("(");
                    if let Some(ty) = ret {
                        self.logger.log("-> ");
                        self.visit_type(ty);
                    }
                },
            }
        }
    }

    fn visit_simple_path(&mut self, path: &mut SimplePath) {
        for (idx, name) in path.names.iter().enumerate() {
            if idx != 0 {
                self.logger.log(".");
            }
            self.logger.log(&self.names[*name]);
        }
    }

    fn visit_path(&mut self, path: &mut Path) {
        if path.is_inferred {
            self.logger.log(".");
        }
        for (idx, iden) in path.idens.iter_mut().enumerate() {
            if idx != 0 {
                self.logger.log(".");
            }
            self.logger.log(&self.names[iden.name]);
            if let Some(gen_args) = &mut iden.gen_args {
                self.visit_gen_args(gen_args);
            }
        }
    }

    fn visit_qual_path(&mut self, path: &mut QualifiedPath) {
        self.logger.log("(:");
        self.visit_type(&mut path.ty);
        if let Some(bound) = &mut path.bound {
            self.logger.log(" as ");
            self.visit_type_path(bound);
        }
        self.logger.log(":).");
        for iden in &mut path.sub_path {
            self.logger.log_fmt(format_args!(".{}", &self.names[iden.name]));
            if let Some(gen_args) = &mut iden.gen_args {
                self.visit_gen_args(gen_args);
            }
        }
    }

    // =============================================================

    fn visit_function(&mut self, node: &mut Function, ctx: &mut FunctionContext) {
        for attr in &mut node.attrs {
            self.visit_attribute(attr);
        }
        self.log_vis(&mut node.vis);
        self.logger.log_fmt(format_args!("{}{}extern \"{}\" fn {}", 
            if node.is_const { "const " } else { "" },
            if node.is_unsafe { "unsafe " } else { "" },
            node.abi,
            &self.names[node.name]
        ));
        if let Some(generics) = &mut node.generics {
            self.visit_gen_params(generics);
        }
        if node.params.is_empty() {
            self.logger.log("()");
        } else {
            self.logger.logln("(");
            self.logger.push_indent();
            for param in &mut node.params {
                self.log_fn_param(param);
            }
            self.logger.pop_indent();
            self.logger.prefixed_log("(");
        }
        if let Some(where_clause) = &mut node.where_clause {
            self.visit_where_clause(where_clause);
        }
        if !node.contracts.is_empty() {
            for contract in &mut node.contracts {
                self.visit_contract(contract);
            }
            self.logger.write_prefix();
        } else {
            self.logger.logln(" ");
        }
        self.visit_block(&mut node.body);
        self.logger.logln("");
    }

    fn visit_extern_function_no_body(&mut self, node: &mut ExternFunctionNoBody, ctx: &mut FunctionContext) {
        for attr in &mut node.attrs {
            self.visit_attribute(attr);
        }
        self.log_vis(&mut node.vis);
        self.logger.log_fmt(format_args!("{}extern \"{}\" fn {}",
            if node.is_unsafe { "unsafe " } else { "" },
            node.abi,
            &self.names[node.name]
        ));
        if let Some(generics) = &mut node.generics {
            self.visit_gen_params(generics);
        }
        if let Some(where_clause) = &mut node.where_clause {
            self.visit_where_clause(where_clause);
        }
        if node.params.is_empty() {
            self.logger.log("()");
        } else {
            self.logger.logln("(");
            self.logger.push_indent();
            for param in &mut node.params {
                self.log_fn_param(param);
            }
            self.logger.pop_indent();
            self.logger.prefixed_log("(");
        }
        self.logger.logln(";");
    }

    fn visit_type_alias(&mut self, node: &mut TypeAlias, ctx: &mut TypeAliasContext) {
        for attr in &mut node.attrs {
            self.visit_attribute(attr);
        }
        self.log_vis(&mut node.vis);
        self.logger.log_fmt(format_args!("type {}", &self.names[node.name]));
        if let Some(generics) = &mut node.generics {
            self.visit_gen_params(generics);
        }
        self.logger.log(" = ");
        self.visit_type(&mut node.ty);
        self.logger.log(";");
    }

    fn visit_distinct_type(&mut self, node: &mut DistinctType, ctx: &mut TypeAliasContext) {
        for attr in &mut node.attrs {
            self.visit_attribute(attr);
        }
        self.log_vis(&mut node.vis);
        self.logger.log_fmt(format_args!("type {}", &self.names[node.name]));
        if let Some(generics) = &mut node.generics {
            self.visit_gen_params(generics);
        }
        self.logger.log(" = ");
        self.visit_type(&mut node.ty);
        self.logger.log(";");
    }

    fn visit_opaque_type(&mut self, node: &mut OpaqueType, ctx: &mut TypeAliasContext) {
        for attr in &mut node.attrs {
            self.visit_attribute(attr);
        }
        self.log_vis(&mut node.vis);
        self.logger.log_fmt(format_args!("type {} = opaque", &self.names[node.name]));
        if let Some(size) = &mut node.size {
            self.logger.log("[");
            self.visit_expr(size);
            self.logger.log("]");
        };
        self.logger.logln("");
    }

    fn visit_struct(&mut self, node: &mut Struct, ctx: &mut StructContext) {
        for attr in &mut node.attrs {
            self.visit_attribute(attr);
        }
        self.log_vis(&mut node.vis);
        self.logger.log_fmt(format_args!("{}{}struct {}", 
            if node.is_mut { "mut " } else { "" },
            if node.is_record { "record " } else { "" },
            &self.names[node.name]
        ));
        if let Some(generics) = &mut node.generics {
            self.visit_gen_params(generics);
        }
        if let Some(where_clause) = &mut node.where_clause {
            self.visit_where_clause(where_clause);
        }
        self.logger.logln(" {");
        self.logger.push_indent();
        for field in &mut node.fields {
            self.log_struct_field(field);
        }
        for struct_use in &mut node.uses {
            for attr in &mut struct_use.attrs {
                self.visit_attribute(attr);
            }
            self.log_vis(&mut struct_use.vis);
            if struct_use.is_mut {
                self.logger.prefixed_log("mut")
            }
            self.visit_type_path(&mut struct_use.path);
            self.logger.log(",");
        }
        self.logger.pop_indent();
        self.logger.prefixed_logln("}");
    }

    fn visit_tuple_struct(&mut self, node: &mut TupleStruct, ctx: &mut StructContext) {
        for attr in &mut node.attrs {
            self.visit_attribute(attr);
        }
        self.log_vis(&mut node.vis);
        self.logger.log_fmt(format_args!("{}{}struct {}", 
            if node.is_mut { "mut " } else { "" },
            if node.is_record { "record " } else { "" },
            &self.names[node.name]
        ));
        if let Some(generics) = &mut node.generics {
            self.visit_gen_params(generics);
        }
        if let Some(where_clause) = &mut node.where_clause {
            self.visit_where_clause(where_clause);
        }
        self.logger.logln(" (");
        self.logger.push_indent();
        for field in &mut node.fields {
            self.log_tuple_struct_field(field);
        }
        self.logger.pop_indent();
        self.logger.prefixed_logln(" );");
    }

    fn visit_unit_struct(&mut self, node: &mut UnitStruct, ctx: &mut StructContext) {
        for attr in &mut node.attrs {
            self.visit_attribute(attr);
        }
        self.log_vis(&mut node.vis);
        self.logger.log_fmt(format_args!("struct {};",
            &self.names[node.name]
        ));
        self.logger.prefixed_logln(";");
    }

    fn visit_union(&mut self, node: &mut Union, ctx: &mut UnionContext) {
        for attr in &mut node.attrs {
            self.visit_attribute(attr);
        }
        self.log_vis(&mut node.vis);
        self.logger.log_fmt(format_args!("{}union {}", 
            if node.is_mut { "mut " } else { "" },
            &self.names[node.name]
        ));
        if let Some(generics) = &mut node.generics {
            self.visit_gen_params(generics);
        }
        if let Some(where_clause) = &mut node.where_clause {
            self.visit_where_clause(where_clause);
        }
        self.logger.logln(" {");
        self.logger.push_indent();
        for field in &mut node.fields {
            for attr in &mut field.attrs {
                self.visit_attribute(attr);
            }
            self.log_vis(&mut field.vis);
            self.logger.log_fmt(format_args!("{}{}: ", 
                if field.is_mut { "mut " } else { "" },
                &self.names[field.name]
            ));
            self.visit_type(&mut field.ty);
            self.logger.logln(",");
        }
        self.logger.pop_indent();
        self.logger.prefixed_logln("}");
    }

    fn visit_adt_enum(&mut self, node: &mut AdtEnum, ctx: &mut AdtEnumContext) {
        for attr in &mut node.attrs {
            self.visit_attribute(attr);
        }
        self.log_vis(&mut node.vis);
        self.logger.log_fmt(format_args!("{}{}enum {}", 
            if node.is_mut { "mut " } else { "" },
            if node.is_record { "record " } else { "" },
            &self.names[node.name]
        ));
        if let Some(generics) = &mut node.generics {
            self.visit_gen_params(generics);
        }
        if let Some(where_clause) = &mut node.where_clause {
            self.visit_where_clause(where_clause);
        }
        
        self.logger.logln(" {");
        self.logger.push_indent();
        for variant in &mut node.variants {
            match variant {
                AdtEnumVariant::Struct { span, attrs, is_mut, name, fields, discriminant } => {
                    for attr in attrs {
                        self.visit_attribute(attr);
                    }
                    self.logger.prefixed_log_fmt(format_args!("{}{}{{", 
                        if node.is_mut { "mut " } else { "" },
                        &self.names[*name]
                    ));
                    self.logger.push_indent();
                    for field in fields {
                        self.log_struct_field(field);
                    }
                    self.logger.pop_indent();
                    self.logger.prefixed_log("}");
                    if let Some(discriminant) = discriminant {
                        self.logger.log(" = ");
                        self.visit_expr(discriminant);
                    }
                    self.logger.prefixed_logln(",");
                },
                AdtEnumVariant::Tuple { span, attrs, is_mut, name, fields, discriminant } => {
                    for attr in attrs {
                        self.visit_attribute(attr);
                    }
                    self.logger.prefixed_log_fmt(format_args!("{}{}(", 
                        if node.is_mut { "mut " } else { "" },
                        &self.names[*name]
                    ));
                    self.logger.push_indent();
                    for field in fields {
                        self.log_tuple_struct_field(field);
                    }
                    self.logger.pop_indent();
                    self.logger.prefixed_log(")");
                    if let Some(discriminant) = discriminant {
                        self.logger.log(" = ");
                        self.visit_expr(discriminant);
                    }
                    self.logger.prefixed_logln(",");
                },
                AdtEnumVariant::Fieldless { span, attrs, name, discriminant } => {
                    for attr in attrs {
                        self.visit_attribute(attr);
                    }
                    self.logger.prefixed_log_fmt(format_args!("{}{}", 
                        if node.is_mut { "mut " } else { "" },
                        &self.names[*name]
                    ));
                    if let Some(discriminant) = discriminant {
                        self.logger.log(" = ");
                        self.visit_expr(discriminant);
                    }
                    self.logger.logln(",");
                },
            }
        }
        self.logger.pop_indent();
        self.logger.prefixed_logln("}");
    }

    fn visit_flag_enum(&mut self, node: &mut FlagEnum, ctx: &mut FlagEnumContext) {
        for attr in &mut node.attrs {
            self.visit_attribute(attr);
        }
        self.log_vis(&mut node.vis);
        self.logger.log_fmt(format_args!("flag enum {}", &self.names[node.name]));
        self.logger.logln(" {");
        self.logger.push_indent();
        for variant in &mut node.variants {
            for attr in &mut variant.attrs {
                self.visit_attribute(attr);
            }
            self.logger.prefixed_log_fmt(format_args!("{}", 
                &self.names[variant.name]
            ));
            if let Some(discriminant) = &mut variant.discriminant {
                self.logger.log(" = ");
                self.visit_expr(discriminant);
            }
            self.logger.logln(",");
        }
        self.logger.pop_indent();
        self.logger.prefixed_logln("}");
    }

    fn visit_bitfield(&mut self, node: &mut Bitfield, ctx: &mut BitfieldContext) {
        for attr in &mut node.attrs {
            self.visit_attribute(attr);
        }
        self.log_vis(&mut node.vis);
        self.logger.log_fmt(format_args!("{}{}bitfield {}", 
            if node.is_mut { "mut " } else { "" },
            if node.is_record { "record " } else { "" },
            &self.names[node.name]
        ));
        if let Some(generics) = &mut node.generics {
            self.visit_gen_params(generics);
        }
        if let Some(where_clause) = &mut node.where_clause {
            self.visit_where_clause(where_clause);
        }
        self.logger.logln(" {");
        self.logger.push_indent();
        for field in &mut node.fields {
            for attr in &mut field.attrs {
                self.visit_attribute(attr);
            }
            self.log_vis(&mut field.vis);
            self.logger.log_fmt(format_args!("{}{}: ", 
                if field.is_mut { "mut " } else { "" },
                &self.names[field.name]
            ));
            self.visit_type(&mut field.ty);
            if let Some(bits) = &mut field.bits {
                self.logger.log(" | ");
                self.visit_expr(bits);
            }
            if let Some(def) = &mut field.def {
                self.logger.log(" = ");
                self.visit_expr(def);
            }
            self.logger.logln(",");
        }
        for struct_use in &mut node.uses {
            for attr in &mut struct_use.attrs {
                self.visit_attribute(attr);
            }
            self.log_vis(&mut struct_use.vis);
            if struct_use.is_mut {
                self.logger.prefixed_log("mut")
            }
            self.visit_type_path(&mut struct_use.path);
            self.logger.log(",");
        }
        self.logger.pop_indent();
        self.logger.prefixed_logln("}");
    }

    fn visit_const(&mut self, node: &mut Const, ctx: &mut ConstContext) {
        for attr in &mut node.attrs {
            self.visit_attribute(attr);
        }
        self.log_vis(&mut node.vis);
        self.logger.prefixed_log_fmt(format_args!("const {}", &self.names[node.name]));
        if let Some(ty) = &mut node.ty {
            self.logger.log(" : ");
            self.visit_type(ty);
        }
        self.logger.log(" = ");
        self.visit_expr(&mut node.val);
        self.logger.logln(";");
    }

    fn visit_static(&mut self, node: &mut Static, ctx: &mut StaticContext) {
        for attr in &mut node.attrs {
            self.visit_attribute(attr);
        }
        self.log_vis(&mut node.vis);
        self.logger.prefixed_log_fmt(format_args!("static {}", &self.names[node.name]));
        if let Some(ty) = &mut node.ty {
            self.logger.log(" : ");
            self.visit_type(ty);
        }
        self.logger.log(" = ");
        self.visit_expr(&mut node.val);
        self.logger.logln(";");
    }

    fn visit_tls_static(&mut self, node: &mut TlsStatic, ctx: &mut StaticContext) {
        for attr in &mut node.attrs {
            self.visit_attribute(attr);
        }
        self.log_vis(&mut node.vis);
        self.logger.prefixed_log_fmt(format_args!("{}tls static {}",
            if node.is_mut { "mut " } else { "" },
            &self.names[node.name]
        ));
        if let Some(ty) = &mut node.ty {
            self.logger.log(" : ");
            self.visit_type(ty);
        }
        self.logger.log(" = ");
        self.visit_expr(&mut node.val);
        self.logger.logln(";");
    }

    fn visit_extern_static(&mut self, node: &mut ExternStatic, ctx: &mut StaticContext) {
        for attr in &mut node.attrs {
            self.visit_attribute(attr);
        }
        self.log_vis(&mut node.vis);
        self.logger.prefixed_log_fmt(format_args!("{}extern {} static {}",
            if node.is_mut { "mut " } else { "" },
            node.abi,
            &self.names[node.name]
        ));
        self.logger.log(" : ");
        self.visit_type(&mut node.ty);
        self.logger.logln(";");
    }

    fn visit_trait(&mut self, node: &mut Trait, ctx: &mut TraitContext) {
        // handled in log_trait
    }

    fn visit_trait_function(&mut self, trait_ref: TraitRef, trait_ctx: TraitContextRef, node: &mut TraitFunction, ctx: &mut FunctionContext) {
        for attr in &mut node.attrs {
            self.visit_attribute(attr);
        }
        self.log_vis(&mut node.vis);
        self.logger.log_fmt(format_args!("{}{}{}fn {}", 
            if node.is_override { "override " } else { "" },
            if node.is_const { "const " } else { "" },
            if node.is_unsafe { "unsafe " } else { "" },
            &self.names[node.name]
        ));
        if let Some(generics) = &mut node.generics {
            self.visit_gen_params(generics);
        }
        self.logger.log("(");
        match &mut node.receiver {
            FnReceiver::None => (),
            FnReceiver::SelfReceiver { span, is_ref, is_mut } => self.logger.log_fmt(format_args!(
                "{}{}self",
                if *is_ref { "&" } else { "" },
                if *is_mut { "mut "} else { "" },
            )),
            FnReceiver::SelfTyped { span, is_mut, ty } => {
                self.logger.log_fmt(format_args!(
                    "{}self : ",
                    if *is_mut { "mut "} else { "" },
                ));
                self.visit_type(ty);
            },
        }
        if !node.params.is_empty() {
            self.logger.logln("");
            self.logger.push_indent();

            for param in &mut node.params {
                self.log_fn_param(param);
            }
            self.logger.pop_indent();
        }
        self.logger.log(")");
        if let Some(where_clause) = &mut node.where_clause {
            self.visit_where_clause(where_clause);
        }
        if !node.contracts.is_empty() {
            for contract in &mut node.contracts {
                self.visit_contract(contract);
            }
            self.logger.write_prefix();
        } else {
            self.logger.logln(" ");
        }
        if let Some(body) = &mut node.body {
            self.visit_block(body);
            self.logger.logln("");
        } else {
            self.logger.logln(";");
        }
    }

    fn visit_trait_type_alias(&mut self, trait_ref: TraitRef, trait_ctx: TraitContextRef, node: &mut TraitTypeAlias, ctx: &mut TypeAliasContext) {
        for attr in &mut node.attrs {
            self.visit_attribute(attr);
        }
        self.logger.log_fmt(format_args!("type {}", &self.names[node.name]));
        if let Some(generics) = &mut node.generics {
            self.visit_gen_params(generics);
        }
        self.logger.log(";");
    }

    fn visit_trait_const(&mut self, trait_ref: TraitRef, trait_ctx: TraitContextRef, node: &mut Const, ctx: &mut ConstContext) {
        // Reuse visit_const
        self.visit_const(node, ctx);
    }

    fn visit_trait_static(&mut self, trait_ref: TraitRef, trait_ctx: TraitContextRef, node: &mut Static, ctx: &mut StaticContext) {
        // Reuse visit_static
        self.visit_static(node, ctx);
    }

    fn visit_trait_property(&mut self, trait_ref: TraitRef, trait_ctx: TraitContextRef, node: &mut TraitProperty, ctx: &mut PropertyContext) {
        for attr in &mut node.attrs {
            self.visit_attribute(attr);
        }
        self.log_vis(&mut node.vis);
        self.logger.log_fmt(format_args!("{}property {} {{\n", 
            if node.is_unsafe { "unsafe " } else { "" },
            &self.names[node.name]
        ));
        if node.has_get {
            self.logger.prefixed_logln("get;");
        }
        if node.has_ref_get {
            self.logger.prefixed_logln("ref get;");
        }
        if node.has_mut_get {
            self.logger.prefixed_logln("mut get;");
        }
        if node.has_set {
            self.logger.prefixed_logln("set;");
        }

        self.logger.prefixed_logln("}");
    }

    fn visit_impl(&mut self, node: &mut Impl, ctx: &mut ImplContext) {
        // handled in log_impl
    }

    fn visit_impl_function(&mut self, impl_ref: ImplRef, impl_ctx: ImplContextRef, node: &mut Function, ctx: &mut FunctionContext) {
        // Reuse visit_function
        self.visit_function(node, ctx);
    }

    fn visit_method(&mut self, impl_ref: ImplRef, impl_ctx: ImplContextRef, node: &mut Method, ctx: &mut FunctionContext) {
        for attr in &mut node.attrs {
            self.visit_attribute(attr);
        }
        self.log_vis(&mut node.vis);
        self.logger.log_fmt(format_args!("{}{}fn {}", 
            if node.is_const { "const " } else { "" },
            if node.is_unsafe { "unsafe " } else { "" },
            &self.names[node.name]
        ));
        if let Some(generics) = &mut node.generics {
            self.visit_gen_params(generics);
        }
        self.logger.log("(");
        match &mut node.receiver {
            FnReceiver::None => (),
            FnReceiver::SelfReceiver { span, is_ref, is_mut } => self.logger.log_fmt(format_args!(
                "{}{}self",
                if *is_ref { "&" } else { "" },
                if *is_mut { "mut "} else { "" },
            )),
            FnReceiver::SelfTyped { span, is_mut, ty } => {
                self.logger.log_fmt(format_args!(
                    "{}self : ",
                    if *is_mut { "mut "} else { "" },
                ));
                self.visit_type(ty);
            },
        }
        if !node.params.is_empty() {
            self.logger.logln("");
            self.logger.push_indent();

            for param in &mut node.params {
                self.log_fn_param(param);
            }
            self.logger.pop_indent();
        }
        self.logger.log(")");
        if let Some(where_clause) = &mut node.where_clause {
            self.visit_where_clause(where_clause);
        }
        if !node.contracts.is_empty() {
            for contract in &mut node.contracts {
                self.visit_contract(contract);
            }
            self.logger.write_prefix();
        } else {
            self.logger.logln(" ");
        }
        self.visit_block(&mut node.body);
        self.logger.logln("");
    }

    fn visit_impl_type_alias(&mut self, impl_ref: ImplRef, impl_ctx: ImplContextRef, node: &mut TypeAlias, ctx: &mut TypeAliasContext) {
        // Reuse visit_type_alias
        self.visit_type_alias(node, ctx);
    }

    fn visit_impl_const(&mut self, impl_ref: ImplRef, impl_ctx: ImplContextRef, node: &mut Const, ctx: &mut ConstContext) {
        // Reuse visit_const
        self.visit_const(node, ctx);
    }

    fn visit_impl_static(&mut self, impl_ref: ImplRef, impl_ctx: ImplContextRef, node: &mut Static, ctx: &mut StaticContext) {
        // Reuse visit_static
        self.visit_static(node, ctx);
    }

    fn visit_impl_tls_static(&mut self, impl_ref: ImplRef, impl_ctx: ImplContextRef, node: &mut TlsStatic, ctx: &mut StaticContext) {
        // Reuse visit_tls_static
        self.visit_tls_static(node, ctx);
    }

    fn visit_property(&mut self, impl_ref: ImplRef, impl_ctx: ImplContextRef, node: &mut Property, ctx: &mut PropertyContext) {
        for attr in &mut node.attrs {
            self.visit_attribute(attr);
        }
        self.log_vis(&mut node.vis);
        self.logger.log_fmt(format_args!("{}property {} {{\n", 
            if node.is_unsafe { "unsafe " } else { "" },
            &self.names[node.name]
        ));
        if let Some(expr) = &mut node.get {
            self.logger.prefixed_log("get ");
            self.visit_expr(expr);
            if let Expr::Block(_) = **expr {
                self.logger.logln(";");
            } else {
                self.logger.logln(";");
            }
        }
        if let Some(expr) = &mut node.ref_get {
            self.logger.prefixed_log("ref get ");
            self.visit_expr(expr);
            if let Expr::Block(_) = **expr {
                self.logger.logln(";");
            } else {
                self.logger.logln(";");
            }
        }
        if let Some(expr) = &mut node.mut_get {
            self.logger.prefixed_log("mut get ");
            self.visit_expr(expr);
            if let Expr::Block(_) = **expr {
                self.logger.logln(";");
            } else {
                self.logger.logln(";");
            }
        }
        if let Some(expr) = &mut node.set {
            self.logger.prefixed_log("set ");
            self.visit_expr(expr);
            if let Expr::Block(_) = **expr {
                self.logger.logln(";");
            } else {
                self.logger.logln(";");
            }
        }

        self.logger.prefixed_logln("}");
    }

    fn visit_op_trait(&mut self, node: &mut OpTrait, ctx: &mut OpTraitContext) {
        // handled in log_op_trait
    }

    fn visit_op_function(&mut self, op_trait_ref: Ref<OpTrait>, op_trait_ctx: Ref<OpTraitContext>, node: &mut OpFunction, ctx: &mut OpFunctionContext) {
        self.logger.prefixed_log_fmt(format_args!("{} op {} : {}",
            node.op_ty,
            node.op.as_str(&self.puncts),
            &self.names[node.name]
        ));
        if let Some(ty) = &mut node.ret_ty {
            self.logger.log(" -> ");
            self.visit_type(ty);
        }
        if let Some(expr) = &mut node.def {
            self.logger.log(" = ");
            self.visit_expr(expr);
        }
        self.logger.logln(";");
    }

    fn visit_op_specialization(&mut self, op_trait_ref: Ref<OpTrait>, op_trait_ctx: Ref<OpTraitContext>, node: &mut OpSpecialization, ctx: &mut OpSpecializationContext) {
        self.logger.prefixed_log_fmt(format_args!("{} op {} :=",
            node.op_ty,
            node.op.as_str(&self.puncts)
        ));
        self.visit_expr(&mut node.def);
        self.logger.logln(";");
    }

    fn visit_op_contract(&mut self, op_trait_ref: Ref<OpTrait>, op_trait_ctx: Ref<OpTraitContext>, node: &mut OpContract, ctx: &mut OpContractContext) {
        if let Expr::Block(block_expr) = &mut *node.expr {
            self.logger.prefixed_log("invar ");
            self.visit_block_expr(block_expr);
            self.logger.prefixed_logln("");
        } else {
            self.logger.prefixed_logln("invar {");
            self.visit_expr(&mut node.expr);
            self.logger.prefixed_logln("}");
        }
    }

    // =============================================================

    fn visit_block(&mut self, node: &mut Block) {
        if node.stmts.is_empty() && node.expr.is_none() {
            self.logger.log("{}");
            return;
        }
        
        self.logger.logln("{");
        self.logger.push_indent();
        for stmt in &mut node.stmts {
            self.logger.write_prefix();
            self.visit_stmt(stmt);
            self.logger.logln("");
        }
        if let Some(expr) = &mut node.expr {
            self.logger.write_prefix();
            self.visit_expr(expr);
            self.logger.logln("");
        }
        self.logger.pop_indent();
        self.logger.prefixed_log("}");
    }

    // =============================================================

    fn visit_stmt(&mut self, stmt: &mut Stmt) {
        // Nothing to do here
        helpers::visit_stmt(self, stmt);
    }

    fn visit_var_decl(&mut self, node: &mut VarDecl) {
        for attr in &mut node.attrs {
            self.visit_attribute(attr);
        }

        self.logger.prefixed_log_fmt(format_args!("let {}{} = ",
            if node.is_mut { "mut " } else { "" },
            &self.names[node.name]
        ));
        if let Some(ty) = &mut node.ty {
            self.logger.log(" : ");
            self.visit_type(ty);
        }

        self.visit_expr(&mut node.expr);
        self.logger.log(";");
    }

    fn visit_uninit_var_decl(&mut self, node: &mut UninitVarDecl) {
        for attr in &mut node.attrs {
            self.visit_attribute(attr);
        }

        self.logger.prefixed_log_fmt(format_args!("let {}{} : ",
            if node.is_mut { "mut " } else { "" },
            &self.names[node.name]
        ));
        self.visit_type(&mut node.ty);
        self.logger.log(";");
    }

    fn visit_defer_stmt(&mut self, node: &mut DeferStmt) {
        for attr in &mut node.attrs {
            self.visit_attribute(attr);
        }
        self.logger.prefixed_log("defer");
        if let Expr::Block(block_expr) = &mut *node.expr {
            self.visit_block_expr(block_expr);
            self.logger.log("");
        } else {
            self.visit_expr(&mut node.expr);
            self.logger.log(";");
        }
    }

    fn visit_err_defer_stmt(&mut self, node: &mut ErrorDeferStmt) {
        for attr in &mut node.attrs {
            self.visit_attribute(attr);
        }
        if let Some(rec) = &mut node.rec {
            self.logger.prefixed_log_fmt(format_args!("errdefer({}{})",
                if rec.is_mut { "mut " } else { "" },
                &self.names[rec.name]
            ));
        } else {
            self.logger.prefixed_log("errdefer");
        };

        if let Expr::Block(block_expr) = &mut *node.expr {
            self.visit_block_expr(block_expr);
            self.logger.log("");
        } else {
            self.visit_expr(&mut node.expr);
            self.logger.log(";");
        }
    }

    fn visit_expr_stmt(&mut self, node: &mut ExprStmt) {
        // Nothing to do here
        helpers::visit_expr_stmt(self, node);
    }

    // =============================================================

    fn visit_expr(&mut self, expr: &mut Expr) {
        // Nothing to do here
        helpers::visit_expr(self, expr);
    }

    fn visit_unit_expr(&mut self, node: &mut UnitExpr) {
        self.logger.log("()");
    }
    
    fn visit_fullrange_expr(&mut self, node: &mut FullRangeExpr) {
        self.logger.log("..");
    }
    
    fn visit_underscore_expr(&mut self, node: &mut UnderscoreExpr) {
        self.logger.log("_");
    }

    fn visit_literal_expr(&mut self, node: &mut LiteralExpr) {
        match node.literal {
            LiteralValue::Lit(lit) => self.logger.log_fmt(format_args!("{}", &self.lits[lit].to_string())),
            LiteralValue::Bool(lit) => self.logger.log_fmt(format_args!("{lit}")),
        }
        if let Some(lit_op) = &mut node.lit_op {
            self.logger.log(":");
            match lit_op {
                LiteralOp::Name(name) => self.logger.log_fmt(format_args!("{}", &self.names[*name])),
                LiteralOp::Primitive(ty) => self.logger.log_fmt(format_args!("{ty}")),
                LiteralOp::StringSlice(ty) => self.logger.log_fmt(format_args!("{ty}")),
            }
        }
    }

    fn visit_path_expr(&mut self, node: &mut PathExpr) {
        match node {
            PathExpr::Named { iden, .. } => {
                self.logger.log_fmt(format_args!("{}", &self.names[iden.name]));
                if let Some(gen_args) = &mut iden.gen_args {
                    self.visit_gen_args(gen_args);
                }
            },
            PathExpr::Inferred { iden, .. } => {
                self.logger.log_fmt(format_args!(".{}", &self.names[iden.name]));
                if let Some(gen_args) = &mut iden.gen_args {
                    self.visit_gen_args(gen_args);
                }
            },
            PathExpr::SelfPath{ .. } => self.logger.log("self"),
            PathExpr::Qualified { path, .. } => self.visit_qual_path(path),
        }
    }

    fn visit_block_expr(&mut self, node: &mut BlockExpr) {
        helpers::visit_block_expr(self, node);
    }

    fn visit_prefix_expr(&mut self, node: &mut PrefixExpr) {
        self.logger.log(node.op.as_str(&self.puncts));
        self.logger.log("(");
        self.visit_expr(&mut node.expr);
        self.logger.log(")");
    }

    fn visit_postfix_expr(&mut self, node: &mut PostfixExpr) {
        self.logger.log("(");
        self.visit_expr(&mut node.expr);
        self.logger.log(")");
        self.logger.log(node.op.as_str(&self.puncts));
    }

    fn visit_infix_expr(&mut self, node: &mut InfixExpr) {
        self.logger.log("(");
        self.visit_expr(&mut node.left);
        self.logger.log(") ");
        self.logger.log(node.op.as_str(&self.puncts));
        self.logger.log(" (");
        self.visit_expr(&mut node.right);
        self.logger.log(")");
    }

    fn visit_inplace_expr(&mut self, node: &mut InplaceExpr) {
        self.logger.log("(");
        self.visit_expr(&mut node.left);
        self.logger.log(") <- (");
        self.visit_expr(&mut node.right);
        self.logger.log(")");
    }

    fn visit_type_cast_expr(&mut self, node: &mut TypeCastExpr) {
        self.logger.log("(");
        self.visit_expr(&mut node.expr);
        self.logger.log(") as (");
        self.visit_type(&mut node.ty);
        self.logger.log(")");
    }

    fn visit_type_check_expr(&mut self, node: &mut TypeCheckExpr) {
        self.logger.log("(");
        self.visit_expr(&mut node.expr);
        self.logger.log(") is (");
        self.visit_type(&mut node.ty);
        self.logger.log(")");
    }

    fn visit_tuple_expr(&mut self, node: &mut TupleExpr) {
        self.logger.log("(");
        for expr in &mut node.exprs {
            self.visit_expr(expr);
            self.logger.log(", ");
        }
        self.logger.log(")");
    }

    fn visit_array_expr(&mut self, node: &mut ArrayExpr) {
        self.logger.log("[");
        for expr in &mut node.exprs {
            self.visit_expr(expr);
            self.logger.log(", ");
        }
        self.logger.log("]");
    }

    fn visit_struct_expr(&mut self, node: &mut StructExpr) {
        self.visit_expr(&mut node.path);
        self.logger.logln("{");
        self.logger.push_indent();
        for arg in &mut node.args {
            self.logger.prefixed_log_fmt(format_args!("{} : ", &self.names[arg.name]));
            self.visit_expr(&mut arg.expr);
            self.logger.logln(",");
        }
        self.logger.pop_indent();
        self.logger.log("}");
    }

    fn visit_index_expr(&mut self, node: &mut IndexExpr) {
        self.visit_expr(&mut node.expr);
        self.logger.log("[");
        self.visit_expr(&mut node.index);
        self.logger.log("]");
    }

    fn visit_tuple_index_expr(&mut self, node: &mut TupleIndexExpr) {
        self.visit_expr(&mut node.expr);
        self.logger.log_fmt(format_args!(".{}", node.index));
    }

    fn visit_fn_call_expr(&mut self, node: &mut FnCallExpr) {
        self.visit_expr(&mut node.func);
        if node.args.is_empty() {
            self.logger.log("()");
        } else {
            self.logger.logln("(");
            self.logger.push_indent();
            for arg in &mut node.args {
                if let Some(label) = arg.label {
                    self.logger.prefixed_log_fmt(format_args!("{}:", &self.names[label]));
                } else {
                    self.logger.write_prefix();
                }
                self.visit_expr(&mut arg.expr);
            }
            self.logger.pop_indent();
            self.logger.prefixed_log(")");
        }
    }

    fn visit_method_call_expr(&mut self, node: &mut MethodCallExpr) {
        self.visit_expr(&mut node.receiver);
        self.logger.log_fmt(format_args!(".{}", &self.names[node.method]));
        if let Some(gen_args) = &mut node.gen_args {
            self.visit_gen_args(gen_args);
        }
        if node.args.is_empty() {
            self.logger.log("()");
        } else {
            self.logger.logln("(");
            self.logger.push_indent();
            for arg in &mut node.args {
                if let Some(label) = arg.label {
                    self.logger.prefixed_log_fmt(format_args!("{}:", &self.names[label]));
                } else {
                    self.logger.write_prefix();
                }
                self.visit_expr(&mut arg.expr);
            }
            self.logger.pop_indent();
            self.logger.prefixed_log(")");
        }
    }

    fn visit_field_access_expr(&mut self, node: &mut FieldAccessExpr) {
        self.visit_expr(&mut node.expr);
        self.logger.log_fmt(format_args!("{}.{}",
            if node.is_propagating { "?" } else { "" },
            &self.names[node.field],
        ));
        if let Some(gen_args) = &mut node.gen_args {
            self.visit_gen_args(gen_args);
        }
    }

    fn visit_closure_expr(&mut self, node: &mut ClosureExpr) {
        // TODO
        self.logger.log("<closure>");
    }

    fn visit_loop_expr(&mut self, node: &mut LoopExpr) {
        self.logger.log("loop");
        self.visit_block(&mut node.body);
    }

    fn visit_match_expr(&mut self, node: &mut MatchExpr) {
        self.logger.log("match ");
        self.visit_expr(&mut node.scrutinee);
        self.logger.logln("{");
        self.logger.push_indent();
        for branch in &mut node.branches {
            if let Some(label) = node.label {
                self.logger.log_fmt(format_args!("{}: ", &self.names[label]));
                self.visit_pattern(&mut branch.pattern);
                if let Some(guard) = &mut branch.guard {
                    self.logger.log(" if ");
                    self.visit_expr(guard);
                }
                self.logger.log(" => ");
                self.visit_expr(&mut branch.body);
                self.logger.logln(",");
            }
        }
        self.logger.pop_indent();
        self.logger.log("}");
    }

    fn visit_break_expr(&mut self, node: &mut BreakExpr) {
        self.logger.log("break");
        if let Some(label) = node.label {
            self.logger.log_fmt(format_args!(" :{}", &self.names[label]));
        }
        if let Some(expr) = &mut node.value {
            self.logger.log(" ");
            self.visit_expr(expr);
        }
    }

    fn visit_continue_expr(&mut self, node: &mut ContinueExpr) {
        self.logger.log("continue");
        if let Some(label) = node.label {
            self.logger.log_fmt(format_args!(" :{}", &self.names[label]));
        }
    }

    fn visit_fallthrough_expr(&mut self, node: &mut FallthroughExpr) {
        self.logger.log("continue");
        if let Some(label) = node.label {
            self.logger.log_fmt(format_args!(" :{}", &self.names[label]));
        }
    }

    fn visit_return_expr(&mut self, node: &mut ReturnExpr) {
        self.logger.log("return");
        if let Some(expr) = &mut node.value {
            self.logger.log(" ");
            self.visit_expr(expr);
        }
    }

    fn visit_throw_expr(&mut self, node: &mut ThrowExpr) {
        self.logger.log("throw ");
        self.visit_expr(&mut node.expr);
    }

    fn visit_comma_expr(&mut self, node: &mut CommaExpr) {
        let end = node.exprs.len() - 1;
        for (idx, expr) in node.exprs.iter_mut().enumerate() {
            self.visit_expr(expr);
            if idx != end {
                self.logger.log(", ");
            }
        }
    }

    fn visit_when_expr(&mut self, node: &mut WhenExpr) {
        self.logger.log("when ");
        self.visit_expr(&mut node.cond);
        self.visit_block(&mut node.body);
        if let Some(else_body) = &mut node.else_body {
            self.logger.log(" else ");
            self.visit_block(else_body);
        }
    }

    fn visit_irrefutable_expr(&mut self) {
        self.logger.log("<irrefutable>");
    }

    fn visit_pattern(&mut self, node: &mut Pattern) {
        helpers::visit_pattern(self, node);
    }

    fn visit_wildcard_pattern(&mut self, node: &mut WildcardPattern) {
        self.logger.log("_");
    }
    
    fn visit_rest_pattern(&mut self, node: &mut RestPattern) {
        self.logger.log("..");
    }

    fn visit_literal_pattern(&mut self, node: &mut LiteralPattern) {
        match node.literal {
            LiteralValue::Lit(lit) => self.logger.log_fmt(format_args!("{}", &self.lits[lit].to_string())),
            LiteralValue::Bool(lit) => self.logger.log_fmt(format_args!("{lit}")),
        }
        if let Some(lit_op) = &mut node.lit_op {
            self.logger.log(":");
            match lit_op {
                LiteralOp::Name(name) => self.logger.log_fmt(format_args!("{}", &self.names[*name])),
                LiteralOp::Primitive(ty) => self.logger.log_fmt(format_args!("{ty}")),
                LiteralOp::StringSlice(ty) => self.logger.log_fmt(format_args!("{ty}")),
            }
        }
    }

    fn visit_iden_pattern(&mut self, node: &mut IdenPattern) {
        self.logger.log_fmt(format_args!("{}{}{}",
            if node.is_ref { "ref " } else { "" },
            if node.is_mut { "mut " } else { "" },
            &self.names[node.name]
        ));
        if let Some(bound) = &mut node.bound {
            self.logger.log(" @ ");
            self.visit_pattern(bound);
        }
    }

    fn visit_path_pattern(&mut self, node: &mut PathPattern) {
        // Nothing to do here
        helpers::visit_path_pattern(self, node);
    }

    fn visit_range_pattern(&mut self, node: &mut RangePattern) {
        match node {
            RangePattern::Exclusive { begin, end, .. } => {
                self.logger.log("(");
                self.visit_pattern(begin);
                self.logger.log(") .. (");
                self.visit_pattern(end);
                self.logger.log(")");
            },
            RangePattern::Inclusive { begin, end, .. } => {
                self.logger.log("(");
                self.visit_pattern(begin);
                self.logger.log(") ..= (");
                self.visit_pattern(end);
                self.logger.log(")");
            },
            RangePattern::From { begin, .. } => {
                self.logger.log("(");
                self.visit_pattern(begin);
                self.logger.log(") ..");
            },
            RangePattern::To { end, .. } => {
                self.logger.log(") .. (");
                self.visit_pattern(end);
                self.logger.log(")");
            },
            RangePattern::InclusiveTo { end, .. } => {
                self.logger.log("..= (");
                self.visit_pattern(end);
                self.logger.log(")");
            },
        }
    }

    fn visit_reference_pattern(&mut self, node: &mut ReferencePattern) {
        self.logger.log("&");
        self.visit_pattern(&mut node.pattern);
    }

    fn visit_struct_pattern(&mut self, node: &mut StructPattern) {
        if let Some(path) = &mut node.path {
            self.visit_path(path);
            self.logger.logln("{");
        } else {
            self.logger.logln(".{");
        }
        self.logger.push_indent();
        for field in &mut node.fields {
            match field {
                StructPatternField::Named { name, pattern, .. } => {
                    self.logger.prefixed_log_fmt(format_args!("{}: ", &self.names[*name]));
                    self.visit_pattern(pattern);
                },
                StructPatternField::TupleIndex { index, pattern, .. } => {
                    self.logger.prefixed_log_fmt(format_args!("{index}: "));
                    self.visit_pattern(pattern);
                },
                StructPatternField::Iden { is_ref, is_mut, iden, bound, .. } => {
                    self.logger.log_fmt(format_args!("{}{}{}",
                        if *is_ref { "ref " } else { "" },
                        if *is_mut { "mut " } else { "" },
                        &self.names[*iden]
                    ));
                    if let Some(bound) = bound {
                        self.logger.log(" @ ");
                        self.visit_pattern(bound);
                    }
                },
                StructPatternField::Rest => self.logger.prefixed_log(".."),
            }
            self.logger.logln("");
        }
        self.logger.pop_indent();
        self.logger.log("}");
    }

    fn visit_tuple_struct_pattern(&mut self, node: &mut TupleStructPattern) {
        if let Some(path) = &mut node.path {
            self.visit_path(path);
            self.logger.logln("(");
        } else {
            self.logger.logln(".(");
        }
        self.logger.push_indent();
        for pattern in &mut node.patterns {
            self.logger.write_prefix();
            self.visit_pattern(pattern);
            self.logger.logln("");
        }
        self.logger.pop_indent();
        self.logger.log(")");
    }

    fn visit_tuple_pattern(&mut self, node: &mut TuplePattern) {
        self.logger.logln("(");
        self.logger.push_indent();
        for pattern in &mut node.patterns {
            self.logger.write_prefix();
            self.visit_pattern(pattern);
            self.logger.logln("");
        }
        self.logger.pop_indent();
        self.logger.log(")");
    }

    fn visit_slice_pattern(&mut self, node: &mut SlicePattern) {
        self.logger.logln("[");
        self.logger.push_indent();
        for pattern in &mut node.patterns {
            self.logger.write_prefix();
            self.visit_pattern(pattern);
            self.logger.logln("");
        }
        self.logger.pop_indent();
        self.logger.log("]");
    }

    fn visit_enum_member_pattern(&mut self, node: &mut EnumMemberPattern) {
        self.logger.log_fmt(format_args!(".{}", &self.names[node.name]));
    }

    fn visit_alternative_pattern(&mut self, node: &mut AlternativePattern) {
        let end = node.patterns.len() - 1;
        for (idx, pattern) in node.patterns.iter_mut().enumerate() {
            self.visit_pattern(pattern);
            if idx == end {
                self.logger.logln("");
            } else {
                self.logger.logln("|");
            }
        }
    }

    fn visit_type_check_pattern(&mut self, node: &mut TypeCheckPattern) {
        self.logger.log("is ");
        self.visit_type(&mut node.ty);
    }

    fn visit_type(&mut self, node: &mut Type) {
        helpers::visit_type(self, node)
    }

    fn visit_unit_type(&mut self, node: &mut UnitType) {
        self.logger.log("()");
    }
    
    fn visit_never_type(&mut self, node: &mut NeverType) {
        self.logger.log("!");
    }

    fn visit_primitive_type(&mut self, node: &mut PrimitiveType) {
        self.logger.log(node.ty.as_str());
    }

    fn visit_path_type(&mut self, node: &mut PathType) {
        self.visit_type_path(&mut node.path);
    }

    fn visit_tuple_type(&mut self, node: &mut TupleType) {
        self.logger.log("(");
        let end = node.types.len() - 1;
        for (idx, ty) in node.types.iter_mut().enumerate() {
            self.visit_type(ty);
            if idx != end {
                self.logger.log(", ");
            }
        }
        self.logger.log(")");
    }

    fn visit_array_type(&mut self, node: &mut ArrayType) {
        self.logger.log("[");
        self.visit_expr(&mut node.size);
        if let Some(sentinel) = &mut node.sentinel {
            self.logger.log("; ");
            self.visit_expr(sentinel);
        }
        self.logger.log("]");
        self.visit_type(&mut node.ty);
    }

    fn visit_slice_type(&mut self, node: &mut SliceType) {
        self.logger.log("[");
        if let Some(sentinel) = &mut node.sentinel {
            self.logger.log("; ");
            self.visit_expr(sentinel);
        }
        self.logger.log("]");
        self.visit_type(&mut node.ty);
    }

    fn visit_string_slice_type(&mut self, node: &mut StringSliceType) {
        self.logger.log(node.ty.as_str());
    }

    fn visit_pointer_type(&mut self, node: &mut PointerType) {
        if node.is_multi {
            self.logger.log("[^");
            if let Some(sentinel) = &mut node.sentinel {
                self.logger.log("; ");
                self.visit_expr(sentinel);
            }
            self.logger.log("]");
        } else {
            self.logger.log("^");
        }
        self.visit_type(&mut node.ty);
    }

    fn visit_reference_type(&mut self, node: &mut ReferenceType) {
        self.logger.log_fmt(format_args!("?{}", if node.is_mut { "mut " } else { "" } ));
        self.visit_type(&mut node.ty);
    }

    fn visit_optional_type(&mut self, node: &mut OptionalType) {
        self.logger.log("?");
        self.visit_type(&mut node.ty);
    }

    fn visit_fn_type(&mut self, node: &mut FnType) {
        self.logger.log("fn(");
        let end = node.params.len() - 1;
        for (idx, (name, ty)) in node.params.iter_mut().enumerate() {
            self.logger.log_fmt(format_args!("{}: ", &self.names[*name]));
            self.visit_type(ty);
            if idx != end {
                self.logger.log(", ")
            }
        }
        self.logger.log("(");
        if let Some(ty) = &mut node.return_ty {
            self.logger.log("-> ");
            self.visit_type(ty);
        }
    }

    fn visit_gen_params(&mut self, node: &mut GenericParams) {
        todo!()
    }

    fn visit_gen_args(&mut self, node: &mut GenericArgs) {
        todo!()
    }

    fn visit_where_clause(&mut self, node: &mut WhereClause) {
        todo!()
    }

    fn visit_trait_bounds(&mut self, node: &mut TraitBounds) {
        todo!()
    }

    fn visit_contract(&mut self, node: &mut Contract) {

    }

    fn visit_attribute(&mut self, node: &mut Attribute) {

    }
}