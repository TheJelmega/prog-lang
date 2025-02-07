use std::mem;

use crate::{
    ast::*,
    common::{uses, Abi, LibraryPath, NameTable, Scope, SpanRegistry, UseTable},
    error_warning::AstErrorCode,
    hir::{self, Identifier, Visitor as _},
    literals::{LiteralId, LiteralTable},
};

use super::{AstError, Context};

// TODO: node tracking for nodes that don't have a `node_id`

pub struct AstToHirLowering<'a> {
    ctx:                &'a mut Context,
    names:              &'a mut NameTable,
    literals:           &'a LiteralTable,
    spans:              &'a SpanRegistry,

    num_nodes_gen:      usize,
    comp_gen_attr:      Box<hir::Attribute>,
    named_ret_expr:     Option<Box<hir::Expr>>,
    in_trait:           bool,
    in_impl:            bool,

    default_vis:        hir::Visibility,
    extern_attrs:       Vec<Box<hir::Attribute>>,

    attr_stack:         Vec<Box<hir::Attribute>>,
    vis_stack:          Vec<hir::Visibility>,
    block_stack:        Vec<hir::Block>,
    stmt_stack:         Vec<Box<hir::Stmt>>,
    expr_stack:         Vec<Box<hir::Expr>>,
    pattern_stack:      Vec<Box<hir::Pattern>>,
    type_stack:         Vec<Box<hir::Type>>,

    gen_args_stack:     Vec<Box<hir::GenericArgs>>,
    gen_params_stack:   Vec<Box<hir::GenericParams>>,
    gen_where_stack:    Vec<Box<hir::WhereClause>>,
    trait_bounds_stack: Vec<Box<hir::TraitBounds>>,

    #[allow(unused)]
    contract_stack:     Vec<Box<hir::Contract>>,

    path_stack:         Vec<hir::Path>,
    simple_path_stack:  Vec<hir::SimplePath>,
    type_path_stack:    Vec<hir::TypePath>,
    qual_path_stack:    Vec<hir::QualifiedPath>,

    hir:                &'a mut hir::Hir,
    use_table:          &'a mut UseTable,

    lib_path:           LibraryPath,
}

impl<'a> AstToHirLowering<'a> {
    pub fn new(ctx: &'a mut Context, names: &'a mut NameTable, literals: &'a LiteralTable, spans: &'a SpanRegistry, hir: &'a mut hir::Hir, use_table: &'a mut UseTable, lib_path: LibraryPath) -> Self {

        let comp_gen_name = names.add("compiler_generated");

        let comp_gen_attr = Box::new(hir::Attribute {
            node_id: u32::MAX,
            path: vec![comp_gen_name],
            meta: hir::AttrMeta::None,
        });

        Self {
            ctx,
            names,
            literals,
            spans,

            num_nodes_gen:      0,
            comp_gen_attr,
            named_ret_expr:     None,
            in_trait:           false,
            in_impl:            false,

            default_vis:        hir::Visibility::Priv,
            extern_attrs:       Vec::new(),

            attr_stack:         Vec::new(),
            vis_stack:          Vec::new(),
            block_stack:        Vec::new(),
            expr_stack:         Vec::new(),
            stmt_stack:         Vec::new(),
            pattern_stack:      Vec::new(),
            type_stack:         Vec::new(),

            gen_args_stack:     Vec::new(),
            gen_params_stack:   Vec::new(),
            gen_where_stack:    Vec::new(),
            trait_bounds_stack: Vec::new(),

            contract_stack:     Vec::new(),

            path_stack:         Vec::new(),
            simple_path_stack:  Vec::new(),
            type_path_stack:    Vec::new(),
            qual_path_stack:    Vec::new(),

            hir,
            use_table,
            lib_path,
        }
    }
}
    
impl AstToHirLowering<'_> {
    fn push_stmt(&mut self, node: hir::Stmt) {
        self.stmt_stack.push(Box::new(node));
        self.num_nodes_gen += 1;
    }

    fn push_expr(&mut self, node: hir::Expr) {
        self.expr_stack.push(Box::new(node));
        self.num_nodes_gen += 1;
    }

    fn push_pattern(&mut self, node: hir::Pattern) {
        self.pattern_stack.push(Box::new(node));
        self.num_nodes_gen += 1;
    }

    fn push_type(&mut self, ty: hir::Type) {
        self.type_stack.push(Box::new(ty));
        self.num_nodes_gen += 1;
    }

    fn convert_fn_params(&mut self, ast_params: &[FnParam], node_id: NodeId) -> Vec<hir::FnParam> {
        
        let mut has_opt = false;
        let mut has_variadic = false;
        let mut params = Vec::new();
        for param in ast_params {
            if has_variadic {
                self.ctx.add_error(AstError {
                    node_id,
                    err: AstErrorCode::VariadicMultiple,
                });
                continue;
            }

            if let Some(_) = param.def_val {
                if param.names.len() != 1 {
                    self.ctx.add_error(AstError {
                        node_id,
                        err: AstErrorCode::ParamMultipleNamesWithDefVal,
                    });
                    continue;
                }

                let def = self.expr_stack.pop().unwrap();
                let ty = self.type_stack.pop().unwrap();
                let pattern = self.pattern_stack.pop().unwrap();
                let attrs = self.get_attribs(&param.names[0].attrs);

                params.push(hir::FnParam::Opt {
                    attrs,
                    label: param.names[0].label,
                    pattern,
                    ty,
                    def,
                });
                has_opt = true;
            } else if param.is_variadic {
                if param.names.len() != 1 {
                    self.ctx.add_error(AstError {
                        node_id,
                        err: AstErrorCode::VariadicMultipleNames,
                    });
                    continue;
                }
                let ty = self.type_stack.pop().unwrap();
                let pattern = self.pattern_stack.pop().unwrap();
                
                let name = if let hir::Pattern::Iden(hir::IdenPattern { node_id: _, is_ref, is_mut, name, bound }) = *pattern {
                    if is_ref {
                        self.ctx.add_error(AstError {
                            node_id,
                            err: AstErrorCode::VariadicInvalidPattern { info: "Variadic Identifier parameters may not have the `ref` modifier".to_string() },
                        });
                        continue;
                    }
                    if is_mut {
                        self.ctx.add_error(AstError {
                            node_id,
                            err: AstErrorCode::VariadicInvalidPattern { info: "Variadic Identifier parameters may not have the `mut` modifier".to_string() },
                        });
                        continue;
                    }
                    if bound.is_some() {
                        self.ctx.add_error(AstError {
                            node_id,
                            err: AstErrorCode::VariadicInvalidPattern { info: "Variadic Identifier parameters may not have a bound".to_string() },
                        });
                        continue;
                    }

                    name
                } else {
                    self.ctx.add_error(AstError {
                        node_id,
                        err: AstErrorCode::VariadicInvalidPattern { info: "Only identifier patterns are alloowed".to_string() },
                    });
                    continue;
                };
                let attrs = self.get_attribs(&param.names[0].attrs);

                params.push(hir::FnParam::Variadic {
                    attrs,
                    name,
                    ty,
                });
                has_variadic = true;

            } else {
                if has_opt {
                    self.ctx.add_error(AstError {
                        node_id,
                        err: AstErrorCode::ParamReqAfterOpt,
                    });
                    continue;
                }

                let ty = self.type_stack.pop().unwrap();

                for name in &param.names {
                    let pattern = self.pattern_stack.pop().unwrap();
                    let attrs = self.get_attribs(&param.names[0].attrs);
                    
                    params.push(hir::FnParam::Param {
                        attrs,
                        label: name.label,
                        pattern,
                        ty: ty.clone(),
                    })
                }
            }
        }
        params.reverse();
        params
    }

    fn convert_reg_struct_field(&mut self, field: &RegStructField) -> (Vec<hir::StructField>, Vec<hir::StructUse>) {
        let mut fields = Vec::new();
        let mut uses = Vec::new();

        match field {
            RegStructField::Field { span, attrs, vis, is_mut, names, ty: _, def } => {
                let hir_attrs = self.attr_stack.split_off(self.attr_stack.len() - attrs.len());

                let hir_vis = self.get_vis(vis.as_ref());
                let hir_ty = self.type_stack.pop().unwrap();

                let hir_def = def.as_ref().map(|_| self.expr_stack.pop().unwrap());

                if let Some(hir_def) = hir_def {
                    if let hir::Expr::Tuple(exprs) = *hir_def {
                        for (name, expr) in names.iter().zip(exprs.exprs.into_iter()).rev() {
                            fields.push(hir::StructField {
                                attrs: hir_attrs.clone(),
                                vis: hir_vis.clone(),
                                is_mut: *is_mut,
                                name: *name,
                                ty: hir_ty.clone(),
                                def: Some(expr),
                            });
                        }
                    } else {
                        for name in names.iter().rev() {
                            fields.push(hir::StructField {
                                attrs: hir_attrs.clone(),
                                vis: hir_vis.clone(),
                                is_mut: *is_mut,
                                name: *name,
                                ty: hir_ty.clone(),
                                def: Some(hir_def.clone()),
                            });
                        }
                    }
                } else {
                    for name in names.iter().rev() {
                        fields.push(hir::StructField {
                            attrs: hir_attrs.clone(),
                            vis: hir_vis.clone(),
                            is_mut: *is_mut,
                            name: *name,
                            ty: hir_ty.clone(),
                            def: hir_def.clone(),
                        });
                    }
                }

                
            },
            RegStructField::Use { span, attrs, vis, is_mut, path: _ } => {
                let attrs = self.get_attribs(attrs);
                let vis = self.get_vis(vis.as_ref());
                let path = self.type_path_stack.pop().unwrap();

                uses.push(hir::StructUse {
                    attrs,
                    vis,
                    is_mut: *is_mut,
                    path,
                })
            },
        };


        (fields, uses)
    }

    fn convert_tuple_struct_field(&mut self, field: &TupleStructField) -> hir::TupleStructField {
        let attrs = self.attr_stack.split_off(self.attr_stack.len() - field.attrs.len());
        let vis = self.get_vis(field.vis.as_ref());
        let ty = self.type_stack.pop().unwrap();
        let def = field.def.as_ref().map(|_| self.expr_stack.pop().unwrap());

        hir::TupleStructField {
            attrs,
            vis,
            ty,
            def,
        }
    }

    fn convert_adt_enum_variant(&mut self, variant: &EnumVariant) -> hir::AdtEnumVariant {
        match variant {
            EnumVariant::Struct { span, attrs, is_mut, name, fields, discriminant } => {
                let hir_attrs = self.attr_stack.split_off(self.attr_stack.len() - attrs.len());
                let hir_dicriminant = discriminant.as_ref().map(|_| self.expr_stack.pop().unwrap());

                let mut hir_fields = Vec::new();
                for field in fields.iter().rev() {
                    let (tmp_field, _) = self.convert_reg_struct_field(field);
                    hir_fields.extend(tmp_field);
                }
                hir_fields.reverse();

                hir::AdtEnumVariant::Struct {
                    attrs: hir_attrs,
                    is_mut: *is_mut,
                    name: *name,
                    fields: hir_fields,
                    discriminant: hir_dicriminant,
                }
            },
            EnumVariant::Tuple { span, attrs, is_mut, name, fields, discriminant } => {
                let hir_attrs = self.attr_stack.split_off(self.attr_stack.len() - attrs.len());
                let hir_dicriminant = discriminant.as_ref().map(|_| self.expr_stack.pop().unwrap());

                let mut hir_fields = Vec::new();
                for field in fields.iter().rev() {
                    hir_fields.push(self.convert_tuple_struct_field(field));
                }
                hir_fields.reverse();

                hir::AdtEnumVariant::Tuple {
                    attrs: hir_attrs,
                    is_mut: *is_mut,
                    name: *name,
                    fields: hir_fields,
                    discriminant: hir_dicriminant,
                }
            },
            EnumVariant::Fieldless { span, attrs, name, discriminant } => {
                let hir_attrs = self.attr_stack.split_off(self.attr_stack.len() - attrs.len());
                let hir_dicriminant = discriminant.as_ref().map(|_| self.expr_stack.pop().unwrap());

                hir::AdtEnumVariant::Fieldless {
                    attrs: hir_attrs,
                    name: *name,
                    discriminant: hir_dicriminant
                }
            },
        }
    }

    fn convert_abi(&mut self, abi: Option<LiteralId>, node_id: NodeId) -> Abi {
        match abi {
            Some(lit_id) => match &self.literals[lit_id] {
                crate::literals::Literal::String(s) => match s.as_str() {
                    "C" => Abi::C,
                    "contextless" => Abi::Contextless,
                    "xenon" => Abi::Xenon,
                    _ => {
                        self.ctx.add_error(AstError{
                            node_id: node_id,
                            err: AstErrorCode::InvalidAbiLiteral { lit: s.clone(), info: "Unknown ABI".to_string() },
                        });
                        Abi::Xenon
                    },
                },
                _ => {
                    let lit = self.literals[lit_id].to_string();
                    self.ctx.add_error(AstError{
                        node_id: node_id,
                        err: AstErrorCode::InvalidAbiLiteral { lit, info: "ABI need to be a string literal".to_string() },
                    });
                    Abi::Xenon
                }
            },
            None => Abi::Xenon,
        }
    }

    fn convert_op_elem(&mut self, op_elem: &OpElem, scope: Scope, node_id: u32) {
        helpers::visit_op_elem(self, op_elem);

        match op_elem {
            OpElem::Def { span, op_type, op, name, ret, def } => {
                let def = def.as_ref().map(|_| self.expr_stack.pop().unwrap());
                let ret_ty = ret.as_ref().map(|_| self.type_stack.pop().unwrap());

                self.hir.add_op_function(scope, hir::OpFunction {
                    node_id,
                    op_ty: *op_type,
                    op: *op,
                    name: *name,
                    ret_ty,
                    def,
                });
            },
            OpElem::Extend { span, op_type, op, def: _ } => {
                let def = self.expr_stack.pop().unwrap();
                
                self.hir.add_op_specialization(scope, hir::OpSpecialization {
                    node_id,
                    op_ty: *op_type,
                    op: *op,
                    def,
                });
            },
            OpElem::Contract { span, expr: _ } => {
                let expr = self.expr_stack.pop().unwrap();
                
                self.hir.add_op_contract(scope, hir::OpContract {
                    node_id,
                    expr,
                });
            },
        }
    }

    fn create_true_false_patterns(&mut self, node_id: u32) -> (Box<hir::Pattern>, Box<hir::Pattern>) {
        let true_pat = Box::new(hir::Pattern::Literal(hir::LiteralPattern {
            node_id: node_id,
            literal: hir::LiteralValue::Bool(true),
            lit_op: None,
        }));
        let false_pat = Box::new(hir::Pattern::Literal(hir::LiteralPattern {
            node_id: node_id,
            literal: hir::LiteralValue::Bool(true),
            lit_op: None,
        }));
        self.num_nodes_gen += 1;
        (true_pat, false_pat)
    }

    
    // TODO: AST attribs don't map fully to HIR attribs
    fn get_attribs(&mut self, attrs: &[AstNodeRef<Attribute>]) -> Vec<Box<hir::Attribute>> {
        let mut hir_attrs = Vec::new();
        for attr in attrs.iter().rev() {
            for _ in attr.metas.iter().rev() {
                hir_attrs.push(self.attr_stack.pop().unwrap());
            }
        }

        for attr in self.extern_attrs.iter().rev() {
            hir_attrs.push(attr.clone());
        }

        hir_attrs.reverse();
        hir_attrs
    }

    fn get_vis(&mut self, vis: Option<&AstNodeRef<Visibility>>) -> hir::Visibility {
        vis.map_or(self.default_vis.clone(), |_| self.vis_stack.pop().unwrap())
    }

    fn get_use_subpaths(&mut self, use_path: &AstNodeRef<UsePath>, lib_path: LibraryPath, base_scope: Scope, paths: &mut Vec<uses::UsePath>) {
        match &**use_path {
            UsePath::SelfPath { span, node_id, alias } => {
                paths.push(uses::UsePath {
                    lib_path: lib_path.clone(),
                    path: base_scope.clone(),
                    wildcard: true,
                    alias: alias.map(|name| self.names[name].to_string()),
                });
            },
            UsePath::SubPaths { span, node_id, segments, sub_paths } => {
                let mut path = base_scope.clone();
                for segment in segments {
                    path.push(self.names[*segment].to_string());
                }
                for sub_path in sub_paths {
                    self.get_use_subpaths(sub_path, lib_path.clone(), path.clone(), paths);
                }

            },
            UsePath::Alias { span, node_id, segments, alias } => {
                let mut path = base_scope.clone();
                for segment in segments {
                    path.push(self.names[*segment].to_string());
                }
                paths.push(uses::UsePath {
                    lib_path: lib_path.clone(),
                    path,
                    wildcard: false,
                    alias: alias.map(|name| self.names[name].to_string()),
                });
            },
        }
    }
}

// =============================================================================================================================

impl Visitor for AstToHirLowering<'_> {
    fn visit(&mut self, ast: &Ast) where Self: Sized {
        for item in &ast.items {
            self.visit_item(item);
        }
    }

    fn visit_simple_path(&mut self, node: &AstNodeRef<SimplePath>) where Self: Sized {
        let mut names = Vec::new();
        for (name, _) in &node.names {
            names.push(*name);
        }

        self.simple_path_stack.push(hir::SimplePath {
            node_id: node.node_id().index() as u32,
            names,
        })
    }

    fn visit_expr_path(&mut self, node: &AstNodeRef<ExprPath>) where Self: Sized {
        helpers::visit_expr_path(self, node);

        let mut idens = Vec::new();
        for iden in node.idens.iter().rev() {
            let gen_args = iden.gen_args.as_ref().map(|_| self.gen_args_stack.pop().unwrap());
            idens.push(hir::Identifier {
                name: iden.name,
                gen_args,
            });
        }
        idens.reverse();


        self.path_stack.push(hir::Path {
            node_id: node.node_id().index() as u32,
            is_inferred: node.inferred,
            idens,
        })
    }

    fn visit_type_path(&mut self, node: &AstNodeRef<TypePath>) where Self: Sized {
        helpers::visit_type_path(self, node);

        let mut segments = Vec::new();
        for iden in node.idens.iter().rev() {
            match iden {
                TypePathIdentifier::Plain { span:_, name } => segments.push(hir::TypePathSegment::Plain { name: *name }),
                TypePathIdentifier::GenArg { name, .. } => {
                    let gen_args = self.gen_args_stack.pop().unwrap();
                    segments.push(hir::TypePathSegment::GenArg { name: *name, gen_args });
                },
                TypePathIdentifier::Fn { span:_, name, params, ret } => {
                    let ret = ret.as_ref().map(|_| self.type_stack.pop().unwrap());
                    let mut hir_params = Vec::new();
                    for _param in params.iter().rev() {
                        hir_params.push(self.type_stack.pop().unwrap());
                    };
                    hir_params.reverse();

                    segments.push(hir::TypePathSegment::Fn {
                        name: *name,
                        params: hir_params,
                        ret,
                    })
                },
            }
        }
        segments.reverse();


        self.type_path_stack.push(hir::TypePath {
            node_id: node.node_id().index() as u32,
            segments,
        })
    }

    fn visit_qualified_path(&mut self, node: &AstNodeRef<QualifiedPath>) where Self: Sized {
        helpers::visit_qualified_path(self, node);

        let sub_gen_args = node.sub_path.gen_args.as_ref().map(|_| self.gen_args_stack.pop().unwrap());
        let sub_path = vec![hir::Identifier {
            name: node.sub_path.name,
            gen_args: sub_gen_args,
        }];

        let bound = node.bound.as_ref().map(|_| self.type_path_stack.pop().unwrap());
        let ty = self.type_stack.pop().unwrap();

        self.qual_path_stack.push(hir::QualifiedPath {
            node_id: node.node_id().index() as u32,
            ty,
            bound,
            sub_path,
        })
    }

    // =============================================================

    fn visit_item(&mut self, item: &Item) where Self: Sized {
        helpers::visit_item(self, item);

        // Don't have to do anything here
    }

    fn visit_trait_item(&mut self, item: &TraitItem) where Self: Sized {
        helpers::visit_trait_item(self, item);

        // Don't have to do anything here
    }

    fn visit_assoc_item(&mut self, item: &AssocItem) where Self: Sized {
        helpers::visit_assoc_item(self, item);

        // Don't have to do anything here
    }

    fn visit_extern_item(&mut self, item: &ExternItem) where Self: Sized {
        helpers::visit_extern_item(self, item);

        // Don't have to do anything here
    }

    fn visit_module(&mut self, node: &AstNodeRef<ModuleItem>) where Self: Sized {
        helpers::visit_module(self, node);

        // Don't have to do anything here
    }

    fn visit_use(&mut self, node: &AstNodeRef<UseItem>) where Self: Sized {
        helpers::visit_use(self, node);

        let ast_ctx = self.ctx.get_node_for(node);
        let scope = ast_ctx.scope.clone();

        let mut paths = Vec::new();
        self.get_use_subpaths(&node.path, self.lib_path.clone(), Scope::new(), &mut paths);

        for path in paths {
            self.use_table.add_use(&scope, path);
        }
    }

    fn visit_use_path(&mut self, node: &AstNodeRef<UsePath>) where Self: Sized {
        helpers::visit_use_path(self, node);

        // Don't have to do anything here
    }

    fn visit_function(&mut self, node: &AstNodeRef<Function>) where Self: Sized {
        helpers::visit_function(self, node, false);
        
        let mut contracts = Vec::new();
        contracts.reverse();

        let where_clause = node.where_clause.as_ref().map(|_| self.gen_where_stack.pop().unwrap());
        let return_ty = node.returns.as_ref().map(|rets| match rets {
            FnReturn::Type{ span, ty:_ } => self.type_stack.pop().unwrap(),
            FnReturn::Named{ span, vars } => {
                let mut types = Vec::new();
                for _ in vars {
                    types.push(self.type_stack.pop().unwrap());
                }
                Box::new(hir::Type::Tuple(hir::TupleType {
                    node_id: node.node_id().index() as u32,
                    types,
                }))
            },
        });

        let params = self.convert_fn_params(&node.params, node.node_id());

        let receiver = node.receiver.as_ref().map_or(hir::FnReceiver::None, |rec| match rec {
            FnReceiver::SelfReceiver { span, node_id, is_ref, is_mut } => {
                hir::FnReceiver::SelfReceiver {
                    is_ref: *is_ref,
                    is_mut: *is_mut,
                }
            },
            FnReceiver::SelfTyped { span, node_id, is_mut, ty: _ } => {
                let ty = self.type_stack.pop().unwrap();
                hir::FnReceiver::SelfTyped {
                    is_mut: *is_mut,
                    ty,
                }
            },
        });

        let generics = node.generics.as_ref().map(|_| self.gen_params_stack.pop().unwrap());

        let abi = self.convert_abi(node.abi, node.node_id());

        let vis = self.get_vis(node.vis.as_ref());
        let attrs = self.get_attribs(&node.attrs);


        let body = node.body.as_ref().map(|body| if let Some(FnReturn::Named{ span, vars }) = &node.returns {
            // convert:
            //
            // ```
            // fn foo() -> (a, b: u32, c: f32) { ... /* body */ }
            // ```
            // to: 
            // ```
            // fn foo() -> (u32, u32, f32) {
            //     let mut a: u32;
            //     let mut b: u32;
            //     let mut c: f32;
            //     ... // body
            //      (a, b, c)
            // }
            // ```

            let Some(ret_ty) = &return_ty else { unreachable!() };
            let hir::Type::Tuple(hir::TupleType{ node_id: _, types }) = ret_ty.as_ref() else { unreachable!() };

            let mut ret_exprs = Vec::new();
            for (names, _) in vars {
                for name in names {
                    ret_exprs.push(Box::new(hir::Expr::Path(hir::PathExpr::Named {
                        iden: Identifier {
                            name: *name,
                            gen_args: None,
                        },
                    })));
                }
            }
            let ret_tup_expr = Box::new(hir::Expr::Tuple(hir::TupleExpr {
                node_id: node.node_id().index() as u32,
                exprs: ret_exprs,
            }));
            self.named_ret_expr = Some(ret_tup_expr.clone());
            self.visit_block(body);
            self.named_ret_expr = None;

            let mut block = self.block_stack.pop().unwrap();
            
            for (idx, (names, _)) in vars.iter().enumerate() {
                let ty = &types[idx];
                for name in names {
                    block.stmts.push(Box::new(hir::Stmt::UninitVarDecl(hir::UninitVarDecl {
                        node_id: node.node_id().index() as u32,
                        attrs: Vec::new(),
                        is_mut: true,
                        name: *name,
                        ty: ty.clone(),
                    })));
                }
            }

            if block.expr.is_none() {
                block.expr = Some(ret_tup_expr)
            }

            Box::new(block)
        } else {
            self.visit_block(body);
            Box::new(self.block_stack.pop().unwrap())
        });

        let node_ctx = self.ctx.get_node_for(node);

        if self.in_trait {
            self.hir.add_trait_function(node_ctx.scope.clone(), hir::TraitFunction {
                node_id: node.node_id().index() as u32,
                attrs,
                vis,
                is_override: node.is_override,
                is_const: node.is_const,
                is_unsafe: node.is_unsafe,
                name: node.name,
                generics,
                receiver,
                params,
                return_ty,
                where_clause,
                contracts,
                body,
            });
        } else if let Some(body) = body {
            if let hir::FnReceiver::None = receiver {
                // function, external function
                self.hir.add_function(self.in_impl, node_ctx.scope.clone(), hir::Function {
                    node_id: node.node_id().index() as u32,
                    attrs,
                    vis,
                    is_const: node.is_const,
                    is_unsafe: node.is_unsafe,
                    abi,
                    name: node.name,
                    generics,
                    params,
                    return_ty,
                    where_clause,
                    contracts,
                    body,
                });
            } else {
                // method
                self.hir.add_method(node_ctx.scope.clone(), hir::Method {
                    node_id: node.node_id().index() as u32,
                    attrs,
                    vis,
                    is_const: node.is_const,
                    is_unsafe: node.is_unsafe,
                    name: node.name,
                    generics,
                    receiver,
                    params,
                    return_ty,
                    where_clause,
                    contracts,
                    body,
                })
            }
        } else {
            // extern
            self.hir.add_extern_function(node_ctx.scope.clone(), hir::ExternFunctionNoBody {
                node_id: node.node_id().index() as u32,
                attrs,
                vis,
                is_unsafe: node.is_unsafe,
                abi,
                name: node.name,
                generics,
                params,
                return_ty,
                where_clause,
                contracts,
            });
        }
    }

    fn visit_type_alias(&mut self, node: &AstNodeRef<TypeAlias>) where Self: Sized {
        helpers::visit_type_alias(self, node);

        let node_ctx = self.ctx.get_node_for(node);
        let scope = node_ctx.scope.clone();
        match &**node {
            TypeAlias::Normal { span, node_id, attrs, vis, name, generics, ty: _ } => {
                let ty = self.type_stack.pop().unwrap();
                let generics = generics.as_ref().map(|_| self.gen_params_stack.pop().unwrap());
                let vis = self.get_vis(vis.as_ref());
                let attrs = self.get_attribs(attrs);

                self.hir.add_type_alias(scope, hir::TypeAlias {
                    node_id: node.node_id().index() as u32,
                    attrs,
                    vis,
                    name: *name,
                    generics,
                    ty,
                });
            },
            TypeAlias::Distinct { span, node_id, attrs, vis, name, generics, ty: _ } => {
                let ty = self.type_stack.pop().unwrap();
                let generics = generics.as_ref().map(|_| self.gen_params_stack.pop().unwrap());
                let vis = self.get_vis(vis.as_ref());
                let attrs = self.get_attribs(attrs);

                self.hir.add_distinct_type(scope, hir::DistinctType {
                    node_id: node.node_id().index() as u32,
                    attrs,
                    vis,
                    name: *name,
                    generics,
                    ty,
                });
            },
            TypeAlias::Trait { span, node_id, attrs, name, generics } => {
                let generics = generics.as_ref().map(|_| self.gen_params_stack.pop().unwrap());
                let attrs = self.get_attribs(attrs);

                self.hir.add_trait_type_alias(scope, hir::TraitTypeAlias {
                    node_id: node.node_id().index() as u32,
                    attrs,
                    name: *name,
                    generics,
                })
            },
            TypeAlias::Opaque { span, node_id, attrs, vis, name, size } => {
                let size = size.as_ref().map(|_| self.expr_stack.pop().unwrap());
                let vis = self.get_vis(vis.as_ref());
                let attrs = self.get_attribs(attrs);

                self.hir.add_opaque_type(scope, hir::OpaqueType {
                    node_id: node.node_id().index() as u32,
                    attrs,
                    vis,
                    name: *name,
                    size,
                })
            },
        }
    }

    fn visit_struct(&mut self, node: &AstNodeRef<Struct>) where Self: Sized {
        helpers::visit_struct(self, node);

        let node_ctx = self.ctx.get_node_for(node);
        let scope = node_ctx.scope.clone();

        match &**node {
            Struct::Regular { span, node_id, attrs, vis, is_mut, is_record, name, generics, where_clause, fields } => {
                let mut hir_fields = Vec::new();
                let mut uses = Vec::new();
        
                for field in fields.iter().rev() {
                    let (tmp_field, tmp_uses) = self.convert_reg_struct_field(field);
                    hir_fields.extend(tmp_field);
                    uses.extend(tmp_uses);
                }
                hir_fields.reverse();
                uses.reverse();

                let where_clause = where_clause.as_ref().map(|_| self.gen_where_stack.pop().unwrap());
                let generics = generics.as_ref().map(|_| self.gen_params_stack.pop().unwrap());
                let vis = self.get_vis(vis.as_ref());
                let attrs = self.get_attribs(attrs);

                self.hir.add_struct(scope, hir::Struct {
                    node_id: node.node_id().index() as u32,
                    attrs,
                    vis,
                    is_mut: *is_mut,
                    is_record: *is_record,
                    name: *name,
                    generics,
                    where_clause,
                    fields: hir_fields,
                    uses,
                })
            },
            Struct::Tuple { span, node_id, attrs, vis, is_mut, is_record, name, generics, where_clause, fields } => {
                let mut hir_fields = Vec::new();
                for field in fields.iter().rev() {
                    hir_fields.push(self.convert_tuple_struct_field(field));
                }
                hir_fields.reverse();

                let where_clause = where_clause.as_ref().map(|_| self.gen_where_stack.pop().unwrap());
                let generics = generics.as_ref().map(|_| self.gen_params_stack.pop().unwrap());
                let vis = self.get_vis(vis.as_ref());
                let attrs = self.get_attribs(attrs);

                self.hir.add_tuple_struct(scope, hir::TupleStruct {
                    node_id: node.node_id().index() as u32,
                    attrs,
                    vis,
                    is_mut: *is_mut,
                    is_record: *is_record,
                    name: *name,
                    generics,
                    where_clause,
                    fields: hir_fields,
                })
            },
            Struct::Unit { span, node_id, attrs, vis, name } => {
                let vis = self.get_vis(vis.as_ref());
                let attrs = self.get_attribs(attrs);

                self.hir.add_unit_struct(scope, hir::UnitStruct {
                    node_id: node.node_id().index() as u32,
                    attrs,
                    vis,
                    name: *name,
                })
            },
        }
    }

    fn visit_reg_struct_field(&mut self, field: &RegStructField) where Self: Sized {
        helpers::visit_reg_struct_field(self, field);

        // Don't have to do anything here, as it's done in convert_*
    }

    fn visit_tuple_struct_field(&mut self, field: &TupleStructField) where Self: Sized {
        helpers::visit_tuple_struct_field(self, field);

        // Don't have to do anything here, as it's done in convert_*
    }

    fn visit_union(&mut self, node: &AstNodeRef<Union>) where Self: Sized {
        helpers::visit_union(self, node);

        let mut fields = Vec::new();
        for field in node.fields.iter().rev() {
            let ty = self.type_stack.pop().unwrap();
            let vis = self.get_vis(field.vis.as_ref());
            let attrs = self.get_attribs(&node.attrs);

            fields.push(hir::UnionField {
                attrs,
                vis,
                is_mut: field.is_mut,
                name: field.name,
                ty,
            });
        }
        fields.reverse();

        let where_clause = node.where_clause.as_ref().map(|_| self.gen_where_stack.pop().unwrap());
        let generics = node.generics.as_ref().map(|_| self.gen_params_stack.pop().unwrap());
        let vis = self.get_vis(node.vis.as_ref());
        let attrs = self.get_attribs(&node.attrs);

        let node_ctx = self.ctx.get_node_for(node);

        self.hir.add_union(node_ctx.scope.clone(), hir::Union {
            node_id: node.node_id().index() as u32,
            attrs,
            vis,
            is_mut: node.is_mut,
            name: node.name,
            generics,
            where_clause,
            fields,
        });
    }

    fn visit_enum(&mut self, node: &AstNodeRef<Enum>) where Self: Sized {
        helpers::visit_enum(self, node);
        let node_ctx = self.ctx.get_node_for(node);
        let scope = node_ctx.scope.clone();

        match &**node {
            Enum::Adt { span, node_id, attrs, vis, is_mut, is_record, name, generics, where_clause, variants } => {
                let mut hir_variants = Vec::new();
                for variant in variants {
                    hir_variants.push(self.convert_adt_enum_variant(variant));
                }
                hir_variants.reverse();

                let where_clause = where_clause.as_ref().map(|_| self.gen_where_stack.pop().unwrap());
                let generics = generics.as_ref().map(|_| self.gen_params_stack.pop().unwrap());
                let vis = self.get_vis(vis.as_ref());
                let attrs = self.get_attribs(attrs);

                self.hir.add_adt_enum(scope, hir::AdtEnum {
                    node_id: node.node_id().index() as u32,
                    attrs,
                    vis,
                    is_mut: *is_mut,
                    is_record: *is_record,
                    name: *name,
                    generics,
                    where_clause,
                    variants: hir_variants,
                });
            },
            Enum::Flag { span, node_id, attrs, vis, name, variants } => {
                let mut hir_variants = Vec::new();
                for variant in variants.iter().rev() {
                    let discriminant = variant.discriminant.as_ref().map(|_| self.expr_stack.pop().unwrap());
                    let attrs = self.get_attribs(attrs);
                    
                    hir_variants.push(hir::FlagEnumVariant {
                        attrs,
                        name: *name,
                        discriminant,
                    })
                }
                hir_variants.reverse();

                let vis = self.get_vis(vis.as_ref());
                let attrs = self.get_attribs(attrs);

                self.hir.add_flag_enum(scope, hir::FlagEnum {
                    node_id: node.node_id().index() as u32,
                    attrs,
                    vis,
                    name: *name,
                    variants: hir_variants,
                });
            },
        }
    }

    fn visit_enum_variant(&mut self, variant: &EnumVariant) where Self: Sized {
        helpers::visit_enum_variant(self, variant);

        // Don't have to do anything here, as it's done in convert_*
    }

    fn visit_bitfield(&mut self, node: &AstNodeRef<Bitfield>) where Self: Sized {
        helpers::visit_bitfield(self, node);

        let mut fields = Vec::new();
        let mut uses = Vec::new();
        for field in node.fields.iter().rev() {
            match field {
                BitfieldField::Field { span, attrs, vis, is_mut, names, ty:_, bits, def } => {
                    let def = def.as_ref().map(|_| self.expr_stack.pop().unwrap());
                    let bits = bits.as_ref().map(|_| self.expr_stack.pop().unwrap());
                    let ty = self.type_stack.pop().unwrap();
                    let vis = self.get_vis(vis.as_ref());
                    let attrs = self.get_attribs(attrs);

                    for name in names.iter().rev() {
                        fields.push(hir::BitfieldField {
                            attrs: attrs.clone(),
                            vis: vis.clone(),
                            is_mut: *is_mut,
                            name: *name,
                            ty: ty.clone(),
                            bits: bits.clone(),
                            def: def.clone(),
                        });
                    }
                },
                BitfieldField::Use { span, attrs, vis, is_mut, path:_, bits } => {
                    let bits = bits.as_ref().map(|_| self.expr_stack.pop().unwrap());
                    let path = self.type_path_stack.pop().unwrap();
                    let vis = self.get_vis(vis.as_ref());
                    let attrs = self.get_attribs(attrs);
                    
                    uses.push(hir::BitfieldUse {
                        attrs,
                        vis,
                        is_mut: *is_mut,
                        path,
                        bits,
                    });
                },
            }
        }
        fields.reverse();
        uses.reverse();

        let where_clause = node.where_clause.as_ref().map(|_| self.gen_where_stack.pop().unwrap());
        let generics = node.generics.as_ref().map(|_| self.gen_params_stack.pop().unwrap());
        let vis = self.get_vis(node.vis.as_ref());
        let attrs = self.get_attribs(&node.attrs);

        let ast_ctx = self.ctx.get_node_for(node);
        self.hir.add_bitfield(ast_ctx.scope.clone(), hir::Bitfield {
            node_id: node.node_id().index() as u32,
            attrs,
            vis,
            is_mut: node.is_mut,
            is_record: node.is_record,
            name: node.name,
            generics,
            where_clause,
            fields,
            uses,
        })
    }

    fn visit_bitfield_field(&mut self, field: &BitfieldField) where Self: Sized {
        helpers::visit_bitfield_field(self, field);

        // Don't have to do anything here, as it's done when handling the bitfield itself
    }

    fn visit_const(&mut self, node: &AstNodeRef<Const>) where Self: Sized {
        helpers::visit_const(self, node);

        let val = self.expr_stack.pop().unwrap();
        let ty = node.ty.as_ref().map(|_| self.type_stack.pop().unwrap());
        let vis = self.get_vis(node.vis.as_ref());
        let attrs = self.get_attribs(&node.attrs);


        let ast_ctx = self.ctx.get_node_for(node );
        let item = hir::Const {
            node_id: node.node_id().index() as u32,
            attrs,
            vis,
            name: node.name,
            ty,
            val,
        };
        if self.in_trait {
            self.hir.add_trait_const(ast_ctx.scope.clone(), item);
        } else {
            self.hir.add_const(self.in_impl, ast_ctx.scope.clone(), item);
        }
    }

    fn visit_static(&mut self, node: &AstNodeRef<Static>) where Self: Sized {
        helpers::visit_static(self, node);

        let ast_ctx = self.ctx.get_node_for(node);
        let scope = ast_ctx.scope.clone();

        match &**node {
            Static::Static { span, node_id, attrs, vis, name, ty, val:_ } => {
                let val = self.expr_stack.pop().unwrap();
                let ty = ty.as_ref().map(|_| self.type_stack.pop().unwrap());
                let vis = self.get_vis(vis.as_ref());
                let attrs = self.get_attribs(attrs);

                self.hir.add_static(self.in_impl, scope, hir::Static {
                    node_id: node.node_id().index() as u32,
                    attrs,
                    vis,
                    name: *name,
                    ty,
                    val,
                })
            },
            Static::Tls { span, node_id, attrs, vis, is_mut, name, ty, val:_ } => {
                let val = self.expr_stack.pop().unwrap();
                let ty = ty.as_ref().map(|_| self.type_stack.pop().unwrap());
                let vis = self.get_vis(vis.as_ref());
                let attrs = self.get_attribs(attrs);

                self.hir.add_tls_static(self.in_impl, scope, hir::TlsStatic {
                    node_id: node.node_id().index() as u32,
                    attrs,
                    vis,
                    is_mut: *is_mut,
                    name: *name,
                    ty,
                    val,
                })
            },
            Static::Extern { span, node_id, attrs, vis, abi, is_mut, name, ty:_ } => {
                let ty = self.type_stack.pop().unwrap();
                let abi = self.convert_abi(Some(*abi), node.node_id());
                let vis = self.get_vis(vis.as_ref());
                let attrs = self.get_attribs(attrs);

                self.hir.add_extern_static(scope, hir::ExternStatic {
                    node_id: node.node_id().index() as u32,
                    attrs,
                    vis,
                    abi,
                    is_mut: *is_mut,
                    name: *name,
                    ty,
                });
            },
        }
    }

    fn visit_property(&mut self, node: &AstNodeRef<Property>) where Self: Sized {
        helpers::visit_property(self, node);

        let ast_ctx = self.ctx.get_node_for(node);
        let scope = ast_ctx.scope.clone();

        match &node.body {
            PropertyBody::Assoc { get, ref_get, mut_get, set } => {
                let set = set.as_ref().map(|_| self.expr_stack.pop().unwrap());
                let mut_get = mut_get.as_ref().map(|_| self.expr_stack.pop().unwrap());
                let ref_get = ref_get.as_ref().map(|_| self.expr_stack.pop().unwrap());
                let get = get.as_ref().map(|_| self.expr_stack.pop().unwrap());

                let vis = self.get_vis(node.vis.as_ref());
                let attrs = self.get_attribs(&node.attrs);

                self.hir.add_property(scope, hir::Property {
                    node_id: node.node_id().index() as u32,
                    attrs,
                    vis,
                    is_unsafe: node.is_unsafe,
                    name: node.name,
                    get,
                    ref_get,
                    mut_get,
                    set,
                });
            },
            PropertyBody::Trait { has_get, has_ref_get, has_mut_get, has_set } => {
                let vis = self.get_vis(node.vis.as_ref());
                let attrs = self.get_attribs(&node.attrs);

                self.hir.add_trait_property(scope, hir::TraitProperty {
                    node_id: node.node_id().index() as u32,
                    attrs,
                    vis,
                    is_unsafe: node.is_unsafe,
                    name: node.name,
                    has_get: has_get.is_some(),
                    has_ref_get: has_ref_get.is_some(),
                    has_mut_get: has_mut_get.is_some(),
                    has_set: has_set.is_some(),
                })
            },
        }
    }

    fn visit_trait(&mut self, node: &AstNodeRef<Trait>) where Self: Sized {
        for attr in &node.attrs {
            self.visit_attribute(attr);
        }
        if let Some(vis) = &node.vis {
            self.visit_visibility(vis);
        }
        if let Some(bounds) = &node.bounds {
            self.visit_trait_bounds(bounds);
        }

        let bounds = node.bounds.as_ref().map(|_| self.trait_bounds_stack.pop().unwrap());
        let vis = self.get_vis(node.vis.as_ref());
        let attrs = self.get_attribs(&node.attrs);

        let ast_ctx = self.ctx.get_node_for(node);
        self.hir.add_trait(ast_ctx.scope.clone(), hir::Trait {
            node_id: node.node_id().index() as u32,
            attrs,
            vis,
            is_unsafe: node.is_unsafe,
            is_sealed: node.is_sealed,
            name: node.name,
            bounds,
        });

        self.in_trait = true;
        for item in &node.assoc_items {
            self.visit_trait_item(item);
        }
        self.in_trait = false;
    }

    fn visit_impl(&mut self, node: &AstNodeRef<Impl>) where Self: Sized {
        for attr in &node.attrs {
            self.visit_attribute(attr);
        }
        if let Some(vis) = &node.vis {
            self.visit_visibility(vis);
        }
        if let Some(generics) = &node.generics {
            self.visit_generic_params(generics)
        }
        self.visit_type(&node.ty);
        if let Some(impl_trait) = &node.impl_trait {
            self.visit_type_path(impl_trait);
        }
        if let Some(where_clause) = &node.where_clause {
            self.visit_where_clause(where_clause);
        }

        let where_clause = node.where_clause.as_ref().map(|_| self.gen_where_stack.pop().unwrap());
        let impl_trait = node.impl_trait.as_ref().map(|_| self.type_path_stack.pop().unwrap());
        let ty = self.type_stack.pop().unwrap();
        let generics = node.generics.as_ref().map(|_| self.gen_params_stack.pop().unwrap());
        let vis = self.get_vis(node.vis.as_ref());
        let attrs = self.get_attribs(&node.attrs);

        let ast_ctx = self.ctx.get_node_for(node);
        self.hir.add_impl(ast_ctx.scope.clone(), hir::Impl {
            node_id: node.node_id().index() as u32,
            attrs,
            vis,
            is_unsafe: node.is_unsafe,
            generics,
            ty,
            impl_trait,
            where_clause,
        });

        self.in_impl = true;
        for item in &node.assoc_items {
            self.visit_assoc_item(item);
        }
        self.in_impl = false;
    }

    fn visit_extern_block(&mut self, node: &AstNodeRef<ExternBlock>) where Self: Sized {
        helpers::visit_extern_block(self, node);
        
        for attr in &node.attrs {
            self.visit_attribute(attr);
        }
        self.extern_attrs = self.get_attribs(&node.attrs);

        self.default_vis = if let Some(vis) = &node.vis {
            self.visit_visibility(vis);
            self.vis_stack.pop().unwrap()
        } else {
            hir::Visibility::Priv
        };

        for item in &node.items {
            self.visit_extern_item(item);
        }

        self.default_vis = hir::Visibility::Priv;
        self.extern_attrs.clear();
    }

    fn visit_op_trait(&mut self, node: &AstNodeRef<OpTrait>) where Self: Sized {
        //helpers::visit_op_trait(self, node);

        let ast_ctx = self.ctx.get_node_for(node);
        let mut scope = ast_ctx.scope.clone();

        match &**node {
            OpTrait::Base { span, node_id, attrs, vis, name, precedence, elems } => {
                for attr in attrs {
                    self.visit_attribute(attr);
                }
                let attrs = self.get_attribs(&attrs);

                if let Some(vis) = vis {
                    self.visit_visibility(vis);
                }
                let vis = self.get_vis(vis.as_ref());

                self.hir.add_op_trait(scope.clone(), hir::OpTrait {
                    node_id: node.node_id().index() as u32,
                    attrs,
                    vis,
                    name: *name,
                    bases: Vec::new(),
                    precedence: *precedence,
                });

                scope.push(self.names[*name].to_string());
                for op_elem in elems {
                    self.convert_op_elem(op_elem, scope.clone(), node.node_id().index() as u32);
                }
            },
            OpTrait::Extended { span, node_id, attrs, vis, name, bases, elems } => {
                for attr in attrs {
                    self.visit_attribute(attr);
                }
                let attrs = self.get_attribs(attrs);

                if let Some(vis) = vis {
                    self.visit_visibility(vis);
                }
                let vis = self.get_vis(vis.as_ref());

                let mut hir_bases = Vec::new();
                for base in bases {
                    self.visit_simple_path(base);
                    hir_bases.push(self.simple_path_stack.pop().unwrap());    
                }
                hir_bases.reverse();

                self.hir.add_op_trait(scope.clone(), hir::OpTrait {
                    node_id: node.node_id().index() as u32,
                    attrs,
                    vis,
                    name: *name,
                    bases: hir_bases,
                    precedence: None,
                });

                scope.push(self.names[*name].to_string());
                for op_elem in elems {
                    self.convert_op_elem(op_elem, scope.clone(), node.node_id().index() as u32);
                }
            },
        }


    }


    fn visit_op_use(&mut self, _node: &AstNodeRef<OpUse>) where Self: Sized {
    }

    fn visit_precedence(&mut self, node: &AstNodeRef<Precedence>) where Self: Sized {
        helpers::visit_precedence(self, node);
    }

    fn visit_precedence_use(&mut self, _node: &AstNodeRef<PrecedenceUse>) where Self: Sized {
    }

    // =============================================================
  
    

    // =============================================================

    fn visit_block(&mut self, node: &AstNodeRef<Block>) where Self: Sized {
        let pre_stmt_count = self.stmt_stack.len();
        helpers::visit_block(self, node);

        let expr = node.final_expr.as_ref().map(|_| self.expr_stack.pop().unwrap());

        let mut stmts = Vec::new();
        for _ in pre_stmt_count..self.stmt_stack.len() {
            stmts.push(self.stmt_stack.pop().unwrap());
        }
        stmts.reverse();

        self.block_stack.push(hir::Block {
            stmts,
            expr,
        });
    }

    // =============================================================

    fn visit_stmt(&mut self, node: &Stmt) where Self: Sized {
        helpers::visit_stmt(self, node);

        // Don't have to do anything here
    }

    fn visit_var_decl(&mut self, node: &AstNodeRef<VarDecl>) where Self: Sized {
        helpers::visit_var_decl(self, node);

        match &**node {
            VarDecl::Named { span, node_id, attrs, names, expr: _ } => {
                let expr = self.expr_stack.pop().unwrap();
                let attrs = self.get_attribs(attrs);

                if names.len() == 1 {
                    let (is_mut, name) = names[0];
                    
                    self.push_stmt(hir::Stmt::VarDecl(hir::VarDecl {
                        node_id: node.node_id().index() as u32,
                        attrs,
                        is_mut,
                        name,
                        ty: None,
                        expr,
                    }));
                } else {
                    match *expr {
                        hir::Expr::Comma(comma_expr) => {
                            for ((is_mut, name), expr) in names.iter().zip(comma_expr.exprs.into_iter()) {
                                self.push_stmt(hir::Stmt::VarDecl(hir::VarDecl {
                                    node_id: node.node_id().index() as u32,
                                    attrs: attrs.clone(),
                                    is_mut: *is_mut,
                                    name: *name,
                                    ty: None    ,
                                    expr,
                                }));
                            }
                        },
                        hir::Expr::Tuple(tuple_expr) => {
                            for ((is_mut, name), expr) in names.iter().zip(tuple_expr.exprs.into_iter()) {
                                self.push_stmt(hir::Stmt::VarDecl(hir::VarDecl {
                                    node_id: node.node_id().index() as u32,
                                    attrs: attrs.clone(),
                                    is_mut: *is_mut,
                                    name: *name,
                                    ty: None    ,
                                    expr,
                                }));
                            }
                        },
                        _ => {
                            // rewrite
                            // 
                            // ```
                            // a, b := c;
                            // ```
                            // 
                            // as"
                            // 
                            // ```
                            // let tmp_0_0 = c;
                            // let a = tmp_0_0.0;
                            // let b = tmp_0_0.1;
                            // ```

                            let span = self.spans[node.span()];

                            let tmp_name = format!("__tmp_{}_{}", span.row, span.column);
                            let tmp_name = self.names.add(&tmp_name);

                            self.push_stmt(hir::Stmt::VarDecl(hir::VarDecl {
                                node_id: node.node_id().index() as u32,
                                attrs: Vec::new(),
                                is_mut: false,
                                name: tmp_name,
                                ty: None,
                                expr,
                            }));
                        
                            for (index, (is_mut, name)) in names.iter().enumerate() {
                                let path_expr = Box::new(hir::Expr::Path(hir::PathExpr::Named {
                                    iden: hir::Identifier {
                                        name: *name,
                                        gen_args: None,
                                    }
                                }));
                            
                                let tup_index = Box::new(hir::Expr::TupleIndex(hir::TupleIndexExpr {
                                    node_id: node.node_id().index() as u32,
                                    expr: path_expr,
                                    index,
                                }));
                            
                                self.push_stmt(hir::Stmt::VarDecl(hir::VarDecl {
                                    node_id: node.node_id().index() as u32,
                                    attrs: attrs.clone(),
                                    is_mut: *is_mut,
                                    name: *name,
                                    ty: None,
                                    expr: tup_index,
                                }))
                            }
                        }
                    }
                }
            },
            VarDecl::Let { span, node_id, attrs, pattern: _, ty, expr, else_block } => {
                let else_block = else_block.as_ref().map(|_| self.expr_stack.pop().unwrap());
                let expr = expr.as_ref().map(|_| self.expr_stack.pop().unwrap());
                let ty = ty.as_ref().map(|_| self.type_stack.pop().unwrap());
                let mut pattern = self.pattern_stack.pop().unwrap();
                let attrs = self.get_attribs(attrs);

                // Special case for unititialized assignments, i.e.
                // `let a: ty;` or `let (b, c): (ty0, ty1);`
                let expr = if let Some(expr) = expr {
                    expr
                } else {
                    let ty = match ty {
                        Some(ty) => ty,
                        None => {
                            self.ctx.add_error(AstError {
                                node_id: node.node_id(),
                                err: AstErrorCode::InvalidUninitVarDecl { info: "Missing type".to_string() },
                            });
                            return;
                        },
                    };

                    match *pattern {
                        // `let a: ty;`
                        hir::Pattern::Iden(hir::IdenPattern { is_ref, is_mut, name, bound, .. }) => {
                            if is_ref {
                                self.ctx.add_error(AstError {
                                    node_id: node.node_id(),
                                    err: AstErrorCode::InvalidUninitVarDecl { info: "Identifiers cannot be prefixed with 'ref'".to_string() },
                                });
                                return;
                            }
                            if bound.is_some() {
                                self.ctx.add_error(AstError {
                                    node_id: node.node_id(),
                                    err: AstErrorCode::InvalidUninitVarDecl { info: "Identifiers cannot have a bound".to_string() },
                                });
                                return;
                            }

                            self.push_stmt(hir::Stmt::UninitVarDecl(hir::UninitVarDecl {
                                node_id: node.node_id().index() as u32,
                                attrs,
                                is_mut,
                                name,
                                ty,
                            }))
                        },
                        // `let (b, mut c): (ty0, ty1);`
                        hir::Pattern::Tuple(hir::TuplePattern { patterns, .. }) => {
                            let types = match *ty {
                                hir::Type::Tuple(hir::TupleType{ types, .. }) => {
                                    types
                                },
                                _ => {
                                    self.ctx.add_error(AstError {
                                        node_id: node.node_id(),
                                        err: AstErrorCode::InvalidUninitVarDecl { info: "Expected a tuple type".to_string() },
                                    });
                                    return;
                                }
                            };

                            for (pattern, ty) in patterns.into_iter().zip(types.into_iter()) {
                                match *pattern {
                                    hir::Pattern::Iden(hir::IdenPattern{ is_ref, is_mut, name, bound, ..  }) => {
                                        if is_ref {
                                            self.ctx.add_error(AstError {
                                                node_id: node.node_id(),
                                                err: AstErrorCode::InvalidUninitVarDecl { info: "Identifiers cannot be prefixed with 'ref'".to_string() },
                                            });
                                            return;
                                        }
                                        if bound.is_some() {
                                            self.ctx.add_error(AstError {
                                                node_id: node.node_id(),
                                                err: AstErrorCode::InvalidUninitVarDecl { info: "Identifiers cannot have a bound".to_string() },
                                            });
                                            return;
                                        }

                                        self.push_stmt(hir::Stmt::UninitVarDecl(hir::UninitVarDecl {
                                            node_id: node.node_id().index() as u32,
                                            attrs: attrs.clone(),
                                            is_mut,
                                            name,
                                            ty,
                                        }))
                                    },
                                    _ => {
                                        self.ctx.add_error(AstError {
                                            node_id: node.node_id(),
                                            err: AstErrorCode::InvalidUninitVarDecl { info: "Only identifiers within tuple patterns are allowed".to_string() },
                                        });
                                    }
                                }
                            }
                        },
                        _ => {
                            self.ctx.add_error(AstError {
                                node_id: node.node_id(),
                                err: AstErrorCode::InvalidUninitVarDecl { info: "Only identifiers are allowed".to_string() },
                            });
                        }
                    }
                    return;
                };

                // Special case for simple assignment, i.e.:
                // `let a: T = expr;`
                if let hir::Pattern::Iden(hir::IdenPattern { is_ref, is_mut, name, bound, .. }) = *pattern {
                    if is_ref {
                        self.ctx.add_error(AstError {
                            node_id: node.node_id(),
                            err: AstErrorCode::InvalidUninitVarDecl { info: "Identifiers cannot be prefixed with 'ref'".to_string() },
                        });
                        return;
                    }
                    if bound.is_some() {
                        self.ctx.add_error(AstError {
                            node_id: node.node_id(),
                            err: AstErrorCode::InvalidUninitVarDecl { info: "Identifiers cannot have a bound".to_string() },
                        });
                        return;
                    }

                    self.push_stmt(hir::Stmt::VarDecl(hir::VarDecl {
                        node_id: node.node_id().index() as u32,
                        attrs,
                        is_mut,
                        name,
                        ty,
                        expr,
                    }));
                    return;
                }

                let span = self.spans[node.span()];

                let tmp0_name = format!("__tmp0_{}_{}", span.row, span.column);
                let tmp0_name = self.names.add(&tmp0_name);

                // Assignment for type check
                self.push_stmt(hir::Stmt::VarDecl(hir::VarDecl {
                    node_id: node.node_id().index() as u32,
                    attrs: Vec::new(),
                    is_mut: false,
                    name: tmp0_name,
                    ty,
                    expr,
                }));


                let mut pattern_iden_collect = hir::utils::PatternIdenCollection::new();
                pattern_iden_collect.visit_pattern(&mut pattern);
                let bind_names = pattern_iden_collect.is_mut_and_names;
                    
                let mut tup_exprs = Vec::new();
                for name in &bind_names {
                    tup_exprs.push(Box::new(hir::Expr::Path(hir::PathExpr::Named {
                        iden: Identifier {
                            name: name.name,
                            gen_args: None,
                        }
                    })));
                }
                let tup_expr = Box::new(hir::Expr::Tuple(hir::TupleExpr {
                    node_id: node.node_id().index() as u32,
                    exprs: tup_exprs,
                })); 

                let scrutinee = Box::new(hir::Expr::Path(hir::PathExpr::Named {
                    iden: hir::Identifier {
                        name: tmp0_name,
                        gen_args: None,
                    }
                }));
                let match_expr = Box::new(hir::Expr::Match(hir::MatchExpr {
                    node_id: node.node_id().index() as u32,
                    label: None,
                    scrutinee,
                    branches: vec![
                    hir::MatchBranch {
                        label: None,
                        pattern,
                        guard: None,
                        body: tup_expr
                    },
                    hir::MatchBranch {
                        label: None,
                        pattern: Box::new(hir::Pattern::Wildcard),
                        guard: None,
                        body: else_block.unwrap_or(Box::new(hir::Expr::Irrefutable)),
                    }
                    ],
                }));
                
                let span = self.spans[node.span()];

                let tmp1_name = if bind_names.len() == 1 {
                    bind_names[0].name
                } else {
                    let tmp_name = format!("__tmp_{}_{}", span.row, span.column);
                    self.names.add(&tmp_name)
                };

                self.push_stmt(hir::Stmt::VarDecl(hir::VarDecl {
                    node_id: node.node_id().index() as u32,
                    attrs: Vec::new(),
                    is_mut: false,
                    name: tmp1_name,
                    ty: None,
                    expr: match_expr,
                }));

                let index_src = Box::new(hir::Expr::Path(hir::PathExpr::Named {
                    iden: hir::Identifier {
                        name: tmp1_name,
                        gen_args: None,
                    }
                }));
                
                if bind_names.len() > 0 {
                    for (index, name) in bind_names.iter().enumerate() {
                        let index_expr = Box::new(hir::Expr::TupleIndex(hir::TupleIndexExpr {
                            node_id: node.node_id().index() as u32,
                            expr: index_src.clone(),
                            index,
                        }));

                        self.push_stmt(hir::Stmt::VarDecl(hir::VarDecl {
                            node_id: node.node_id().index() as u32,
                            attrs: attrs.clone(),
                            is_mut: name.is_mut,
                            name: name.name,
                            ty: None,
                            expr: index_expr,
                        }));
                    }
                }
            },
        }
    }

    fn visit_defer(&mut self, node: &AstNodeRef<Defer>) where Self: Sized {
        helpers::visit_defer(self, node);

        let expr = self.expr_stack.pop().unwrap();
        let attrs = self.get_attribs(&node.attrs);

        self.push_stmt(hir::Stmt::Defer(hir::DeferStmt {
            node_id: node.node_id().index() as u32,
            attrs,
            expr,
        }))
    }

    fn visit_err_defer(&mut self, node: &AstNodeRef<ErrDefer>) where Self: Sized {
        helpers::visit_err_defer(self, node);

        let expr = self.expr_stack.pop().unwrap();
        let rec = node.receiver.as_ref().map(|rec| hir::ErrorDeferReceiver {
            is_mut: rec.is_mut,
            name: rec.name,
        });
        let attrs = self.get_attribs(&node.attrs);

        self.push_stmt(hir::Stmt::ErrDefer(hir::ErrorDeferStmt {
            node_id: node.node_id().index() as u32,
            attrs,
            rec,
            expr,
        }));
    }

    fn visit_expr_stmt(&mut self, node: &AstNodeRef<ExprStmt>) where Self: Sized {
        helpers::visit_expr_stmt(self, node);

        let expr = self.expr_stack.pop().unwrap(); 

        self.push_stmt(hir::Stmt::Expr(hir::ExprStmt {
            node_id: node.node_id().index() as u32,
            expr,
        }));
    }

    // =============================================================

    fn visit_expr(&mut self, node: &Expr) where Self: Sized {
        helpers::visit_expr(self, node);

        // Don't have to do anything here
    }

    fn visit_literal_expr(&mut self, node: &AstNodeRef<LiteralExpr>) where Self: Sized {
        let literal = match node.literal {
            LiteralValue::Lit(lit_id) => hir::LiteralValue::Lit(lit_id),
            LiteralValue::Bool(val)   => hir::LiteralValue::Bool(val),
        };

        let lit_op = node.lit_op.as_ref().map(|lit_op| match lit_op {
            LiteralOp::Name(name_id)   => hir::LiteralOp::Name(*name_id),
            LiteralOp::Primitive(ty)   => hir::LiteralOp::Primitive(ty.ty),
            LiteralOp::StringSlice(ty) => hir::LiteralOp::StringSlice(ty.ty), 
        });

        let lit_expr = hir::LiteralExpr {
            node_id: node.node_id().index() as u32,
            literal,
            lit_op,
        };
        
        self.push_expr(hir::Expr::Literal(lit_expr));
    }

    fn visit_path_expr(&mut self, node: &AstNodeRef<PathExpr>) where Self: Sized {
        helpers::visit_path_expr(self, node);

        let expr = match &**node {
            PathExpr::Named { span, node_id, iden } => {
                let gen_args = iden.gen_args.as_ref().map(|_| self.gen_args_stack.pop().unwrap());

                hir::PathExpr::Named {
                    iden: hir::Identifier {
                        name: iden.name,
                        gen_args,
                    }
                }
            },
            PathExpr::Inferred { span, node_id, iden } => {
                let gen_args = iden.gen_args.as_ref().map(|_| self.gen_args_stack.pop().unwrap());

                hir::PathExpr::Inferred {
                    iden: hir::Identifier {
                        name: iden.name,
                        gen_args,
                    }
                }
            },
            PathExpr::SelfPath => hir::PathExpr::SelfPath,
            PathExpr::Qualified { span, node_id, path: _ } => {
                let path = self.qual_path_stack.pop().unwrap();
                hir::PathExpr::Qualified { path }
            },
        };
        self.push_expr(hir::Expr::Path(expr)); 
    }

    fn visit_unit_expr(&mut self) where Self: Sized {
        self.push_expr(hir::Expr::Unit);
    }

    fn visit_block_expr(&mut self, node: &AstNodeRef<BlockExpr>) where Self: Sized {
        helpers::visit_block_expr(self, node);

        let kind = match node.kind {
            BlockExprKind::Normal            => hir::BlockKind::Normal,
            BlockExprKind::Unsafe            => hir::BlockKind::Unsafe,
            BlockExprKind::Const             => hir::BlockKind::Const,
            BlockExprKind::Try               => hir::BlockKind::Try,
            BlockExprKind::TryUnwrap         => hir::BlockKind::TryUnwrap,
            BlockExprKind::Labeled { label } => hir::BlockKind::Labeled(label),
        };

        let block = self.block_stack.pop().unwrap();
        self.push_expr(hir::Expr::Block(hir::BlockExpr {
            node_id: node.node_id().index() as u32,
            kind,
            block,
        }))
    }

    fn visit_prefix_expr(&mut self, node: &AstNodeRef<PrefixExpr>) where Self: Sized {
        helpers::visit_prefix_expr(self, node);

        let expr = self.expr_stack.pop().unwrap();

        self.push_expr(hir::Expr::Prefix(hir::PrefixExpr{
            node_id: node.node_id().index() as u32,
            op: node.op,
            expr,
        }));
    }

    fn visit_postfix_expr(&mut self, node: &AstNodeRef<PostfixExpr>) where Self: Sized {
        helpers::visit_postfix_expr(self, node);

        let expr = self.expr_stack.pop().unwrap();

        self.push_expr(hir::Expr::Postfix(hir::PostfixExpr{
            node_id: node.node_id().index() as u32,
            op: node.op,
            expr,
        }));
    }

    fn visit_binary_expr(&mut self, node: &AstNodeRef<InfixExpr>) where Self: Sized {
        helpers::visit_binary_expr(self, node);

        let right = self.expr_stack.pop().unwrap();
        let left = self.expr_stack.pop().unwrap();
        let can_reorder = matches!(node.right, Expr::Infix(_));

        self.push_expr(hir::Expr::Infix(hir::InfixExpr {
            node_id: node.node_id().index() as u32,
            left,
            op: node.op,
            right,
            can_reorder,
        }));
    }

    fn visit_paren_expr(&mut self, node: &AstNodeRef<ParenExpr>) where Self: Sized {
        helpers::visit_paren_expr(self, node);

        // Don't have this is hir, so just fall through
    }

    fn visit_inplace_expr(&mut self, node: &AstNodeRef<InplaceExpr>) where Self: Sized {
        helpers::visit_inplace_expr(self, node);

        let right = self.expr_stack.pop().unwrap();
        let left = self.expr_stack.pop().unwrap();

        self.push_expr(hir::Expr::Inplace(hir::InplaceExpr {
            node_id: node.node_id().index() as u32,
            left,
            right,
        }));
    }

    fn visit_type_cast_expr(&mut self, node: &AstNodeRef<TypeCastExpr>) where Self: Sized {
        helpers::visit_type_cast_expr(self, node);

        let ty = self.type_stack.pop().unwrap();
        let expr = self.expr_stack.pop().unwrap();

        self.push_expr(hir::Expr::TypeCast(hir::TypeCastExpr {
            node_id: node.node_id().index() as u32,
            expr,
            ty,
        }));
    }

    fn visit_type_check_expr(&mut self, node: &AstNodeRef<TypeCheckExpr>) where Self: Sized {
        helpers::visit_type_check_expr(self, node);

        let ty = self.type_stack.pop().unwrap();
        let expr = self.expr_stack.pop().unwrap();

        self.push_expr(hir::Expr::TypeCheck(hir::TypeCheckExpr {
            node_id: node.node_id().index() as u32,
            expr,
            ty,
        }));
    }

    fn visit_tuple_expr(&mut self, node: &AstNodeRef<TupleExpr>) where Self: Sized {
        helpers::visit_tuple_expr(self, node);

        let mut exprs = Vec::new();
        for _ in node.exprs.iter().rev() {
            exprs.push(self.expr_stack.pop().unwrap());
        }
        exprs.reverse();

        self.push_expr(hir::Expr::Tuple(hir::TupleExpr {
            node_id: node.node_id().index() as u32,
            exprs,
        }))
    }

    fn visit_array_expr(&mut self, node: &AstNodeRef<ArrayExpr>) where Self: Sized {
        helpers::visit_array_expr(self, node);

        let mut exprs = Vec::new();
        for _ in node.exprs.iter().rev() {
            exprs.push(self.expr_stack.pop().unwrap());
        }
        exprs.reverse();

        self.push_expr(hir::Expr::Array(hir::ArrayExpr {
            node_id: node.node_id().index() as u32,
            exprs,
        }))
    } 

    fn visit_struct_expr(&mut self, node: &AstNodeRef<StructExpr>) where Self: Sized {
        helpers::visit_struct_expr(self, node);

        let mut args = Vec::new();
        let mut complete = None;
        for arg in node.args.iter().rev() {
            match arg {
                StructArg::Expr{ span, name, expr:_ } => {
                    let expr = self.expr_stack.pop().unwrap();

                    args.push(hir::StructArg {
                        name: *name,
                        expr,
                    });
                },
                StructArg::Name{ span ,name } => {
                    let expr = Box::new(hir::Expr::Path(hir::PathExpr::Named { 
                        iden: hir::Identifier {
                            name: *name,
                            gen_args: None,
                        }
                     }));

                     args.push(hir::StructArg {
                        name: *name,
                        expr,
                     });
                },
                StructArg::Complete{ span, expr:_ } => if complete.is_none() {
                    let expr = self.expr_stack.pop().unwrap();
                    complete = Some(expr);
                } else {
                    self.ctx.add_error(AstError {
                        node_id: node.node_id(),
                        err: AstErrorCode::MultipleStructComplete,
                    })
                },
            };
        }
        args.reverse();

        let path = self.expr_stack.pop().unwrap();

        self.push_expr(hir::Expr::Struct(hir::StructExpr {
            node_id: node.node_id().index() as u32,
            path,
            args,
            complete
        }))
    }

    fn visit_index_expr(&mut self, node: &AstNodeRef<IndexExpr>) where Self: Sized {
        helpers::visit_index_expr(self, node);

        let index = self.expr_stack.pop().unwrap();
        let expr = self.expr_stack.pop().unwrap();

        self.push_expr(hir::Expr::Index(hir::IndexExpr {
            node_id: node.node_id().index() as u32,
            is_opt: node.is_opt,
            expr,
            index,
        }))
    }

    fn visit_tuple_index_expr(&mut self, node: &AstNodeRef<TupleIndexExpr>) where Self: Sized {
        helpers::visit_tuple_index_expr(self, node);

        let expr = self.expr_stack.pop().unwrap();

        let index = match &self.literals[node.index] {
            crate::literals::Literal::Decimal { int_digits, frac_digits, .. } => {
                if !frac_digits.is_empty() {
                    self.ctx.add_error(AstError{
                        node_id: node.node_id(),
                        err: AstErrorCode::InvalidLiteral{ lit: self.literals[node.index].to_string(), info: "Only interger literals are allowed for a tuple index".to_string() },
                    });
                }

                let mut index = 0;
                for digit in int_digits {
                    index *= 10;
                    index += *digit as usize;
                }
                index
            },
            _ => {
                self.ctx.add_error(AstError{
                    node_id: node.node_id(),
                    err: AstErrorCode::InvalidLiteral{ lit: self.literals[node.index].to_string(), info: "Only interger literals are allowed for a tuple index".to_string() },
                });
                0
            },
        };

        self.push_expr(hir::Expr::TupleIndex(hir::TupleIndexExpr {
            node_id: node.node_id().index() as u32,
            expr,
            index,
        }))
    }

    fn visit_fn_call_expr(&mut self, node: &AstNodeRef<FnCallExpr>) where Self: Sized {
        helpers::visit_fn_call_expr(self, node);

        let mut args = Vec::new();
        for arg in &node.args {
            let (label, expr) = match arg {
                FnArg::Expr{ span, expr } => {
                    let expr = self.expr_stack.pop().unwrap();
                    (None, expr)
                },
                FnArg::Labeled { span, label, expr: _ } => {
                    let expr = self.expr_stack.pop().unwrap();
                    (Some(*label), expr)
                },
            };
             args.push(hir::FnArg {
                label,
                expr,
            })
        }
        args.reverse();

        let func = self.expr_stack.pop().unwrap();

        self.push_expr(hir::Expr::FnCall(hir::FnCallExpr {
            node_id: node.node_id().index() as u32,
            func,
            args,
        }))
    }

    fn visit_method_call_expr(&mut self, node: &AstNodeRef<MethodCallExpr>) where Self: Sized {
        helpers::visit_method_call_expr(self, node);

        let mut args = Vec::new();
        for arg in &node.args {
            let (label, expr) = match arg {
                FnArg::Expr{ span, expr } => {
                    let expr = self.expr_stack.pop().unwrap();
                    (None, expr)
                },
                FnArg::Labeled { span, label, expr: _ } => {
                    let expr = self.expr_stack.pop().unwrap();
                    (Some(*label), expr)
                },
            };
             args.push(hir::FnArg {
                label,
                expr,
            })
        }
        args.reverse();

        let gen_args = node.gen_args.as_ref().map(|_| self.gen_args_stack.pop().unwrap());
        let receiver = self.expr_stack.pop().unwrap();

        self.push_expr(hir::Expr::MethodCall(hir::MethodCallExpr {
            node_id: node.node_id().index() as u32,
            receiver,
            method: node.method,
            gen_args,
            args,
            is_propagating: node.is_propagating,
        }))

    }

    fn visit_field_access_expr(&mut self, node: &AstNodeRef<FieldAccessExpr>) where Self: Sized {
        helpers::visit_field_access_expr(self, node);

        let gen_args = node.gen_args.as_ref().map(|_| self.gen_args_stack.pop().unwrap());
        let expr = self.expr_stack.pop().unwrap();

        self.push_expr(hir::Expr::FieldAccess(hir::FieldAccessExpr {
            node_id: node.node_id().index() as u32,
            expr,
            field: node.field,
            gen_args,
            is_propagating: node.is_propagating,
        }))
    }

    fn visit_closure_expr(&mut self, node: &AstNodeRef<ClosureExpr>) where Self: Sized {
        helpers::visit_closure_expr(self, node);

        let body = self.expr_stack.pop().unwrap();

        // TODO``

        self.push_expr(hir::Expr::Closure(hir::ClosureExpr {
            node_id: node.node_id().index() as u32,
            is_moved: node.is_moved,
            params: todo!(),
            ret: todo!(),
            body: todo!(),
        }))
    }

    fn visit_full_range_expr(&mut self) where Self: Sized {
        self.push_expr(hir::Expr::FullRange)
    }

    fn visit_let_binding_expr(&mut self, node: &AstNodeRef<LetBindingExpr>) where Self: Sized {
        helpers::visit_let_binding_expr(self, node);


        // TODO
    }

    fn visit_if_expr(&mut self, node: &AstNodeRef<IfExpr>) where Self: Sized {
        helpers::visit_if_expr(self, node);

        let else_body = if node.else_body.is_some() {
            self.expr_stack.pop().unwrap()
        } else {
            Box::new(hir::Expr::Unit)
        };
        let body = self.expr_stack.pop().unwrap();

        let branches = vec![
            hir::MatchBranch {
                label: None,
                pattern: Box::new(hir::Pattern::Literal(hir::LiteralPattern {
                    node_id: node.node_id().index() as u32,
                    literal: hir::LiteralValue::Bool(true),
                    lit_op: None
                })),
                guard: None,
                body
            },
            hir::MatchBranch {
                label: None,
                pattern: Box::new(hir::Pattern::Literal(hir::LiteralPattern {
                    node_id: node.node_id().index() as u32,
                    literal: hir::LiteralValue::Bool(false),
                    lit_op: None
                })),
                guard: None,
                body: else_body
            }
        ];

        let cond = self.expr_stack.pop().unwrap();

        self.push_expr(hir::Expr::Match(hir::MatchExpr {
            node_id: node.node_id().index() as u32,
            label: None,
            scrutinee: cond,
            branches,
        }))
    }

    fn visit_loop_expr(&mut self, node: &AstNodeRef<LoopExpr>) where Self: Sized {
        helpers::visit_loop_expr(self, node);  
        
        let hir::Expr::Block(hir::BlockExpr{ kind, block, .. }) = *self.expr_stack.pop().unwrap() else { unreachable!() };
        assert!(kind == hir::BlockKind::Normal);
        let body = Box::new(block);

        self.push_expr(hir::Expr::Loop(hir::LoopExpr {
            node_id: node.node_id().index() as u32,
            label: node.label,
            body,
        }))
    }

    fn visit_while_expr(&mut self, node: &AstNodeRef<WhileExpr>) where Self: Sized {
        helpers::visit_while_expr(self, node);

        // rewrite:
        //
        // ```
        // :label: while cond; inc {
        //     ... // body
        // } else {
        //     ... // else
        // }
        // ```
        //
        // as:
        //
        // ```
        // match cond { // (1)
        //     true => :label: loop { // (2)
        //         { ... } // body (3)
        //         match cond { // (4)
        //             true  => (),
        //             false => break :label; // (5)
        //         }
        //         inc; // (6)
        //     },
        //     false => ..., // else // (7)
        // }
        // ```

        let else_expr = node.else_body.as_ref().map(|_| self.expr_stack.pop().unwrap());
        let body = self.expr_stack.pop().unwrap();
        let inc = node.inc.as_ref().map(|_| self.expr_stack.pop().unwrap());
        let cond = self.expr_stack.pop().unwrap();

        let (true_pat, false_pat) = self.create_true_false_patterns(node.node_id().index() as u32);

        // (3)
        let hir::Expr::Block(mut body) = *body else { unreachable!() };
        let end_expr = mem::take(&mut body.block.expr);
        if let Some(expr) = end_expr {
            body.block.stmts.push(Box::new(hir::Stmt::Expr(hir::ExprStmt {
                node_id: node.node_id().index() as u32,
                expr,
            })));
        }
        let body = Box::new(hir::Expr::Block(body));

        let label = if let Some(label) = node.label {
            label
        } else {
            let span = self.spans[node.span()];

            let label_name = format!("__label_{}_{}", span.row, span.column);
            self.names.add(&label_name)
        };

        // (5)
        let loop_break = hir::BreakExpr {
            node_id: node.node_id().index() as u32,
            label: Some(label),
            value: None,
        };

        // (4)
        let end_cond = hir::Expr::Match(hir::MatchExpr {
            node_id: node.node_id().index() as u32,
            label: None,
            scrutinee: cond.clone(),
            branches: vec![
                hir::MatchBranch {
                    label: None,
                    pattern: true_pat.clone(),
                    guard: None,
                    body: Box::new(hir::Expr::Unit)
                },
                // (5)
                hir::MatchBranch {
                    label: None,
                    pattern: false_pat.clone(),
                    guard: None,
                    body: Box::new(hir::Expr::Break(loop_break))
                },
            ],
        });
        let end_cond = Box::new(end_cond);

        
        let mut loop_body =  hir::Block {
            stmts: vec![
                Box::new(hir::Stmt::Expr(hir::ExprStmt {
                    node_id: node.node_id().index() as u32,
                    expr: body
                })),
                Box::new(hir::Stmt::Expr(hir::ExprStmt {
                    node_id: node.node_id().index() as u32,
                    expr: end_cond,
                }))
            ],
            expr: None,
        };

        // (6)
        if let Some(inc) = inc {
            loop_body.stmts.push(Box::new(hir::Stmt::Expr(hir::ExprStmt {
                node_id: node.node_id().index() as u32,
                expr: inc,
            })));
        }

        // (2)
        let loop_expr = hir::Expr::Loop(hir::LoopExpr {
            node_id: node.node_id().index() as u32,
            label: Some(label),
            body: Box::new(loop_body),
        });
        let loop_expr = Box::new(loop_expr);

        // (7)
        let else_body = else_expr.unwrap_or(Box::new(hir::Expr::Unit));

        // (1)
        self.push_expr(hir::Expr::Match(hir::MatchExpr {
            node_id: node.node_id().index() as u32,
            label: None,
            scrutinee: cond,
            branches: vec![
                hir::MatchBranch {
                    label: None,
                    pattern: true_pat,
                    guard: None,
                    body: loop_expr
                },
                // (7)
                hir::MatchBranch {
                    label: None,
                    pattern: false_pat,
                    guard: None,
                    body: else_body,
                },
            ],
        }));
    }

    fn visit_do_while_expr(&mut self, node: &AstNodeRef<DoWhileExpr>) where Self: Sized {
        helpers::visit_do_while_expr(self, node);
        
        // rewrite:
        //
        // ```
        // :label: do {
        //     ... // body
        // } while cond;
        //
        // ```
        //
        // as:
        //
        // ```
        // :label: loop { // (1)
        //     { ... } // body // (2)
        //     match cond { // (3)
        //         true  => (),
        //         false => {
        //             break :label; // (4)
        //         }
        //     }
        // }
        // ```

        let body = self.expr_stack.pop().unwrap();
        let cond = self.expr_stack.pop().unwrap();

        let (true_pat, false_pat) = self.create_true_false_patterns(node.node_id().index() as u32);

        // (3)
        let hir::Expr::Block(mut body) = *body else { unreachable!() };
        let end_expr = mem::take(&mut body.block.expr);
        if let Some(expr) = end_expr {
            body.block.stmts.push(Box::new(hir::Stmt::Expr(hir::ExprStmt {
                node_id: node.node_id().index() as u32,
                expr,
            })));
        }
        let body = Box::new(hir::Expr::Block(body));

        let label = if let Some(label) = node.label {
            label
        } else {
            let span = self.spans[node.span()];

            let label_name = format!("__label_{}_{}", span.row, span.column);
            self.names.add(&label_name)
        };

        // (4)
        let loop_break = hir::BreakExpr {
            node_id: node.node_id().index() as u32,
            label: Some(label),
            value: None,
        };

        // (3)
        let end_cond = hir::Expr::Match(hir::MatchExpr {
            node_id: node.node_id().index() as u32,
            label: None,
            scrutinee: cond.clone(),
            branches: vec![
                hir::MatchBranch {
                    label: None,
                    pattern: true_pat.clone(),
                    guard: None,
                    body: Box::new(hir::Expr::Unit)
                },
                // (5)
                hir::MatchBranch {
                    label: None,
                    pattern: false_pat.clone(),
                    guard: None,
                    body: Box::new(hir::Expr::Break(loop_break))
                },
            ],
        });
        let end_cond = Box::new(end_cond);

        
        let loop_body =  hir::Block {
            stmts: vec![
                Box::new(hir::Stmt::Expr(hir::ExprStmt {
                    node_id: node.node_id().index() as u32,
                    expr: body
                })),
                Box::new(hir::Stmt::Expr(hir::ExprStmt {
                    node_id: node.node_id().index() as u32,
                    expr: end_cond,
                }))
            ],
            expr: None,
        };
        // (1)
        self.push_expr(hir::Expr::Loop(hir::LoopExpr {
            node_id: node.node_id().index() as u32,
            label: Some(label),
            body: Box::new(loop_body),
        }));
    }

    fn visit_for_expr(&mut self, node: &AstNodeRef<ForExpr>) where Self: Sized {
        helpers::visit_for_expr(self, node);

        // TODO: figure out iterator interface

        // rewrite:
        //
        // :label: for pat in src {
        //     ... // body
        // } else {
        //     ... // else_body
        // }
        //
        // as:
        //
        // {
        //     let iter = src. ();
        //     match iter. () {
        //         Some(val) => {
        //             
        //             :label: loop {
        //             
        //             
        //             
        //             }
        //             
        //         },
        //         None => {
        //             ... // else_body
        //         }
        // }
    }

    fn visit_match_expr(&mut self, node: &AstNodeRef<MatchExpr>) where Self: Sized {
        helpers::visit_match_expr(self, node);

        let mut branches = Vec::new();
        for branch in node.branches.iter().rev() {
            let body = self.expr_stack.pop().unwrap();
            let guard = branch.guard.as_ref().map(|_| self.expr_stack.pop().unwrap());
            let pattern = self.pattern_stack.pop().unwrap();

            branches.push(hir::MatchBranch {
                label: branch.label,
                pattern,
                guard,
                body,
            });
        }
        branches.reverse();

        let scrutinee = self.expr_stack.pop().unwrap();

        self.push_expr(hir::Expr::Match(hir::MatchExpr {
            node_id: node.node_id().index() as u32,
            label: node.label,
            scrutinee,
            branches,
        }))
    }

    fn visit_break_expr(&mut self, node: &AstNodeRef<BreakExpr>) where Self: Sized {
        helpers::visit_break_expr(self, node);

        let value = node.value.as_ref().map(|_| self.expr_stack.pop().unwrap());

        self.push_expr(hir::Expr::Break(hir::BreakExpr {
            node_id: node.node_id().index() as u32,
            label: node.label,
            value,
        }));
    }

    fn visit_continue_expr(&mut self, node: &AstNodeRef<ContinueExpr>) where Self: Sized {
        self.push_expr(hir::Expr::Continue(hir::ContinueExpr {
            node_id: node.node_id().index() as u32,
            label: node.label,
        }));
    }

    fn visit_fallthrough_expr(&mut self, node: &AstNodeRef<FallthroughExpr>) where Self: Sized {
        self.push_expr(hir::Expr::Fallthrough(hir::FallthroughExpr {
            node_id: node.node_id().index() as u32,
            label: node.label,
        }));
    }

    fn visit_return_expr(&mut self, node: &AstNodeRef<ReturnExpr>) where Self: Sized {
        helpers::visit_return_expr(self, node);

        let value = match node.value {
            Some(_) => Some(self.expr_stack.pop().unwrap()),
            None => self.named_ret_expr.clone()
        };

        self.push_expr(hir::Expr::Return(hir::ReturnExpr {
            node_id: node.node_id().index() as u32,
            value,
        }));
    }

    fn visit_underscore_expr(&mut self) where Self: Sized {
        self.push_expr(hir::Expr::Underscore);
    }

    fn visit_throw_expr(&mut self, node: &AstNodeRef<ThrowExpr>) where Self: Sized {
        helpers::visit_throw_expr(self, node);

        let expr = self.expr_stack.pop().unwrap();

        self.push_expr(hir::Expr::Throw(hir::ThrowExpr {
            node_id: node.node_id().index() as u32,
            expr,
        }));
    }

    fn visit_comma_expr(&mut self, node: &AstNodeRef<CommaExpr>) where Self: Sized {
        helpers::visit_comma_expr(self, node);

        let mut exprs = Vec::new();
        for _ in node.exprs.iter().rev() {
            exprs.push(self.expr_stack.pop().unwrap());
        }
        exprs.reverse();

        self.push_expr(hir::Expr::Comma(hir::CommaExpr {
            node_id: node.node_id().index() as u32,
            exprs,
        }))
    }

    fn visit_when_expr(&mut self, node: &AstNodeRef<WhenExpr>) where Self: Sized {
        helpers::visit_when_expr(self, node);

        let else_body = node.else_body.as_ref().map(|_| {
            let expr = self.expr_stack.pop().unwrap();
            let hir::Expr::Block(hir::BlockExpr{ block, .. }) = *expr else { unreachable!() };
            let body = Box::new(block);
            body
        });

        let body = self.expr_stack.pop().unwrap();
        let hir::Expr::Block(hir::BlockExpr{ block, .. }) = *body else { unreachable!() };
        let body = Box::new(block);

        let cond = self.expr_stack.pop().unwrap();

        self.push_expr(hir::Expr::When(hir::WhenExpr {
            node_id: node.node_id().index() as u32,
            cond,
            body,
            else_body,
        }))
    }

    // =============================================================

    fn visit_pattern(&mut self, node: &Pattern) where Self: Sized {
        helpers::visit_pattern(self, node);

        // Don't have to do anything here
    }

    fn visit_literal_pattern(&mut self, node: &AstNodeRef<LiteralPattern>) where Self: Sized {
        let literal = match node.literal {
            LiteralValue::Lit(lit)  => hir::LiteralValue::Lit(lit),
            LiteralValue::Bool(val) => hir::LiteralValue::Bool(val),
        };
        let lit_op = node.lit_op.as_ref().map(|lit_op| {
            match lit_op {
                LiteralOp::Name(name)         => hir::LiteralOp::Name(*name),
                LiteralOp::Primitive(prim)    => hir::LiteralOp::Primitive(prim.ty),
                LiteralOp::StringSlice(slice) => hir::LiteralOp::StringSlice(slice.ty),
            }
        });

        self.push_pattern(hir::Pattern::Literal(hir::LiteralPattern {
            node_id: node.node_id().index() as u32,
            literal,
            lit_op,
        }))
    }

    fn visit_identifier_pattern(&mut self, node: &AstNodeRef<IdentifierPattern>) where Self: Sized {
        helpers::visit_identifier_pattern(self, node);

        let bound = node.bound.as_ref().map(|_| self.pattern_stack.pop().unwrap());

        self.push_pattern(hir::Pattern::Iden(hir::IdenPattern {
            node_id: node.node_id().index() as u32,
            is_ref: node.is_ref,
            is_mut: node.is_mut,
            name: node.name,
            bound,
        }))
    }

    fn visit_path_pattern(&mut self, node: &AstNodeRef<PathPattern>) where Self: Sized {
        helpers::visit_path_pattern(self, node);

        let path = self.path_stack.pop().unwrap();

        self.push_pattern(hir::Pattern::Path(hir::PathPattern {
            node_id: node.node_id().index() as u32,
            path,
        }));
    }

    fn visit_wildcard_pattern(&mut self) where Self: Sized {
        self.push_pattern(hir::Pattern::Wildcard);
    }

    fn visit_rest_pattern(&mut self) where Self: Sized {
        self.push_pattern(hir::Pattern::Rest);
    }

    fn visit_range_pattern(&mut self, node: &AstNodeRef<RangePattern>) where Self: Sized {
        helpers::visit_range_pattern(self, node);

        let pattern = match &**node {
            RangePattern::Exclusive { .. } => {
                let node_id = node.node_id().index() as u32;
                let end = self.pattern_stack.pop().unwrap();
                let begin = self.pattern_stack.pop().unwrap();
                hir::RangePattern::Exclusive { node_id, begin, end }
            },
            RangePattern::Inclusive { .. } => {
                let node_id = node.node_id().index() as u32;
                let end = self.pattern_stack.pop().unwrap();
                let begin = self.pattern_stack.pop().unwrap();
                hir::RangePattern::Inclusive { node_id, begin, end }
            },
            RangePattern::From { .. } => {
                let node_id = node.node_id().index() as u32;
                let begin = self.pattern_stack.pop().unwrap();
                hir::RangePattern::From { node_id, begin }
            },
            RangePattern::To { .. } => {
                let node_id = node.node_id().index() as u32;
                let end = self.pattern_stack.pop().unwrap();
                hir::RangePattern::To { node_id, end }
            },
            RangePattern::InclusiveTo { .. } => {
                let node_id = node.node_id().index() as u32;
                let end = self.pattern_stack.pop().unwrap();
                hir::RangePattern::InclusiveTo { node_id, end }
            },
        };
        self.push_pattern(hir::Pattern::Range(pattern));
    }

    fn visit_reference_pattern(&mut self, node: &AstNodeRef<ReferencePattern>) where Self: Sized {
        helpers::visit_reference_pattern(self, node);

        let pattern = self.pattern_stack.pop().unwrap();

        self.push_pattern(hir::Pattern::Reference(hir::ReferencePattern {
            node_id: node.node_id().index() as u32,
            is_mut: node.is_mut,
            pattern,
        }));
    }

    fn visit_struct_pattern(&mut self, node: &AstNodeRef<StructPattern>) where Self: Sized {
        helpers::visit_struct_pattern(self, node);

        let (path, ast_fields) = match &**node {
            StructPattern::Inferred { span, node_id, fields } => (None, fields),
            StructPattern::Path { span, node_id, path: _, fields } => {
                let path = self.path_stack.pop().unwrap();
                (Some(path), fields)
            },
        };

        let mut fields = Vec::new();
        for field in ast_fields.iter().rev() {
            match field {
                StructPatternField::Named { span, name, pattern: _ } => {
                    let pattern = self.pattern_stack.pop().unwrap();
                    fields.push(hir::StructPatternField::Named {
                        node_id: node.node_id().index() as u32,
                        name: *name,
                        pattern,
                    });
                },
                StructPatternField::TupleIndex { span, idx, pattern: _ } => {
                    let pattern = self.pattern_stack.pop().unwrap();

                    let index = match &self.literals[*idx] {
                        crate::literals::Literal::Decimal { int_digits, frac_digits, .. } => {
                            if !frac_digits.is_empty() {
                                self.ctx.add_error(AstError{
                                    node_id: node.node_id(),
                                    err: AstErrorCode::InvalidLiteral{ lit: self.literals[*idx].to_string(), info: "Only interger literals are allowed for a tuple index".to_string() },
                                });
                            }

                            let mut index = 0;
                            for digit in int_digits {
                                index *= 10;
                                index += *digit as usize;
                            }
                            index
                        },
                        _ => {
                            self.ctx.add_error(AstError{
                                node_id: node.node_id(),
                                err: AstErrorCode::InvalidLiteral{ lit: self.literals[*idx].to_string(), info: "Only interger literals are allowed for a tuple index".to_string() },
                            });
                            0
                        },
                    };

                    fields.push(hir::StructPatternField::TupleIndex {
                        node_id: node.node_id().index() as u32,
                        index,
                        pattern,
                    })
                },
                StructPatternField::Iden { span, is_ref, is_mut, iden, bound } => {
                    let bound = bound.as_ref().map(|_| self.pattern_stack.pop().unwrap());

                    fields.push(hir::StructPatternField::Iden {
                        node_id: node.node_id().index() as u32,
                        is_ref: *is_ref,
                        is_mut: *is_mut,
                        iden: *iden,
                        bound
                    })
                },
                StructPatternField::Rest => fields.push(hir::StructPatternField::Rest),
            }
        }
        fields.reverse();

        self.push_pattern(hir::Pattern::Struct(hir::StructPattern {
            node_id: node.node_id().index() as u32,
            path,
            fields,
        }));
    }

    fn visit_tuple_struct_pattern(&mut self, node: &AstNodeRef<TupleStructPattern>) where Self: Sized {
        helpers::visit_tuple_struct_pattern(self, node);

        let (path, ast_patterns) = match &**node {
            TupleStructPattern::Inferred { span, node_id, patterns } => (None, patterns),
            TupleStructPattern::Named { span, node_id, path: _, patterns } => {
                let path = self.path_stack.pop().unwrap();
                (Some(path), patterns)
            },
        };

        let mut patterns = Vec::new();
        for _ in ast_patterns.iter().rev() {
            patterns.push(self.pattern_stack.pop().unwrap());
        }
        patterns.reverse();

        self.push_pattern(hir::Pattern::TupleStruct(hir::TupleStructPattern {
            node_id: node.node_id().index() as u32,
            path,
            patterns,
        }));
    }

    fn visit_tuple_pattern(&mut self, node: &AstNodeRef<TuplePattern>) where Self: Sized {
        helpers::visit_tuple_pattern(self, node);

        let mut patterns = Vec::new();
        for _ in node.patterns.iter().rev() {
            patterns.push(self.pattern_stack.pop().unwrap());
        }
        patterns.reverse();

        self.push_pattern(hir::Pattern::Tuple(hir::TuplePattern {
            node_id: node.node_id().index() as u32,
            patterns,
        }));
    }

    fn visit_grouped_pattern(&mut self, node: &AstNodeRef<GroupedPattern>) where Self: Sized {
        helpers::visit_grouped_pattern(self, node);

        // Don't have this is hir, so just fall through
    }

    fn visit_slice_pattern(&mut self, node: &AstNodeRef<SlicePattern>) where Self: Sized {
        helpers::visit_slice_pattern(self, node);

        let mut patterns = Vec::new();
        for _ in node.patterns.iter().rev() {
            patterns.push(self.pattern_stack.pop().unwrap());
        }
        patterns.reverse();

        self.push_pattern(hir::Pattern::Slice(hir::SlicePattern {
            node_id: node.node_id().index() as u32,
            patterns,
        }));
    }

    fn visit_enum_member_pattern(&mut self, node: &AstNodeRef<EnumMemberPattern>) where Self: Sized {
        self.push_pattern(hir::Pattern::EnumMember(hir::EnumMemberPattern {
            node_id: node.node_id().index() as u32,
            name: node.name,
        }));
    }

    fn visit_alternative_pattern(&mut self, node: &AstNodeRef<AlternativePattern>) where Self: Sized {
        helpers::visit_alternative_pattern(self, node);

        let mut patterns = Vec::new();
        for _ in node.patterns.iter().rev() {
            patterns.push(self.pattern_stack.pop().unwrap());
        }
        patterns.reverse();

        self.push_pattern(hir::Pattern::Alternative(hir::AlternativePattern {
            node_id: node.node_id().index() as u32,
            patterns,
        }));
    }

    fn visit_type_check_pattern(&mut self, node: &AstNodeRef<TypeCheckPattern>) where Self: Sized {
        helpers::visit_type_check_pattern(self, node);

        let ty = self.type_stack.pop().unwrap();

        self.push_pattern(hir::Pattern::TypeCheck(hir::TypeCheckPattern {
            node_id: node.node_id().index() as u32,
            ty,
        }));
    }

    // =============================================================

    fn visit_type(&mut self, node: &Type) where Self: Sized {
        helpers::visit_type(self, node);

        // Don't have to do anything here
    }

    fn visit_paren_type(&mut self, node: &AstNodeRef<ParenthesizedType>) where Self: Sized {
        helpers::visit_paren_type(self, node);

        // Don't have this is hir, so just fall through
    }

    fn visit_primitive_type(&mut self, node: &AstNodeRef<PrimitiveType>) where Self: Sized {
        self.push_type(hir::Type::Primitive(hir::PrimitiveType {
            node_id: node.node_id().index() as u32,
            ty: node.ty
        }));
    }

    fn visit_unit_type(&mut self) where Self: Sized {
        self.push_type(hir::Type::Unit);
    }

    fn visit_never_type(&mut self) where Self: Sized {
        self.push_type(hir::Type::Never);
    }

    fn visit_path_type(&mut self, node: &AstNodeRef<PathType>) where Self: Sized {
        helpers::visit_path_type(self, node);

        let path = self.type_path_stack.pop().unwrap();
        
        self.push_type(hir::Type::Path(hir::PathType {
            node_id: node.node_id().index() as u32,
            path,
        }));
    }

    fn visit_tuple_type(&mut self, node: &AstNodeRef<TupleType>) where Self: Sized {
        helpers::visit_tuple_type(self, node);

        let mut types: Vec<Box<hir::Type>> = (0..node.types.len())
            .map(|_| self.type_stack.pop().unwrap())
            .collect();
        types.reverse();

        self.push_type(hir::Type::Tuple(hir::TupleType {
            node_id: node.node_id().index() as u32,
            types
        }));
    }

    fn visit_array_type(&mut self, node: &AstNodeRef<ArrayType>) where Self: Sized {
        helpers::visit_array_type(self, node);

        let ty = self.type_stack.pop().unwrap();
        let sentinel = node.sentinel.as_ref().map(|_| self.expr_stack.pop().unwrap());
        let size = self.expr_stack.pop().unwrap();

        self.push_type(hir::Type::Array(hir::ArrayType {
            node_id: node.node_id().index() as u32,
            size,
            sentinel,
            ty,
        }))
    }

    fn visit_slice_type(&mut self, node: &AstNodeRef<SliceType>) where Self: Sized {
        helpers::visit_slice_type(self, node);

        let ty = self.type_stack.pop().unwrap();
        let sentinel = node.sentinel.as_ref().map(|_| self.expr_stack.pop().unwrap());

        self.push_type(hir::Type::Slice(hir::SliceType {
            node_id: node.node_id().index() as u32,
            sentinel,
            ty,
        }));
    }

    fn visit_string_slice_type(&mut self, node: &AstNodeRef<StringSliceType>) where Self: Sized {
        let slice_ty = node.ty;
        self.push_type(hir::Type::StringSlice(hir::StringSliceType {
            node_id: node.node_id().index() as u32,
            ty: slice_ty,
        }));
    }

    fn visit_pointer_type(&mut self, node: &AstNodeRef<PointerType>) where Self: Sized {
        helpers::visit_pointer_type(self, node);

        let ty = self.type_stack.pop().unwrap();
        let sentinel = node.sentinel.as_ref().map(|_| self.expr_stack.pop().unwrap());

        self.push_type(hir::Type::Pointer(hir::PointerType {
            node_id: node.node_id().index() as u32,
            is_multi: node.is_multi,
            is_mut: node.is_mut,
            ty,
            sentinel,
        }));
    }

    fn visit_reference_type(&mut self, node: &AstNodeRef<ReferenceType>) where Self: Sized {
        helpers::visit_reference_type(self, node);

        let ty = self.type_stack.pop().unwrap();

        self.push_type(hir::Type::Reference(hir::ReferenceType {
            node_id: node.node_id().index() as u32,
            is_mut: node.is_mut,
            ty,
        }))
    }

    fn visit_optional_type(&mut self, node: &AstNodeRef<OptionalType>) where Self: Sized {
        helpers::visit_optional_type(self, node);

        let ty = self.type_stack.pop().unwrap();

        self.push_type(hir::Type::Optional(hir::OptionalType {
            node_id: node.node_id().index() as u32,
            ty,
        }));
    }

    fn visit_fn_type(&mut self, node: &AstNodeRef<FnType>) where Self: Sized {
        helpers::visit_fn_type(self, node);

        let return_ty = node.return_ty.as_ref().map(|_| self.type_stack.pop().unwrap());
        let mut params = Vec::new();
        node.params.iter().rev().for_each(|(names, _)| {
            let ty = self.type_stack.pop().unwrap();
            for name in names {
                params.push((*name, ty.clone()))
            }
        });
        params.reverse();

        let abi = match node.abi {
            Some(lit_id) => match &self.literals[lit_id] {
                crate::literals::Literal::String(s) => match s.as_str() {
                    "C" => Abi::C,
                    "contextless" => Abi::Contextless,
                    "xenon" => Abi::Xenon,
                    _ => {
                        self.ctx.add_error(AstError{
                            node_id: node.node_id(),
                            err: AstErrorCode::InvalidAbiLiteral { lit: s.clone(), info: "Unknown ABI".to_string() },
                        });
                        Abi::Xenon
                    },
                },
                _ => {
                    let lit = self.literals[lit_id].to_string();
                    self.ctx.add_error(AstError{
                        node_id: node.node_id(),
                        err: AstErrorCode::InvalidAbiLiteral { lit, info: "ABI need to be a string literal".to_string() },
                    });
                    Abi::Xenon
                }
            },
            None => Abi::Xenon,
        };

        self.push_type(hir::Type::Fn(hir::FnType {
            node_id: node.node_id().index() as u32,
            is_unsafe: node.is_unsafe,
            abi,
            params,
            return_ty,
        }))
    }

    // Should generate struct, not record type
    fn visit_record_type(&mut self, node: &AstNodeRef<RecordType>) where Self: Sized {
        helpers::visit_record_type(self, node);

        let mut fields = Vec::new();
        let mut uses = Vec::new();
        node.fields.iter().rev().for_each(|field| {
            let (tmp_fields, tmp_uses) = self.convert_reg_struct_field(field);
            fields.extend(tmp_fields);
            uses.extend(tmp_uses);
        });
        fields.reverse();
        uses.reverse();

        let span = self.spans[node.span()];
        let file_name = self.spans.get_file(span.file_id);

        let name = format!("__anon_record_{file_name}_{}_{}", span.row, span.column);
        let name = self.names.add(&name);

        let ast_ctx = self.ctx.get_node_for(node);

        self.hir.add_struct(ast_ctx.scope.clone(), hir::Struct {
            node_id: node.node_id().index() as u32,
            attrs: vec![self.comp_gen_attr.clone()],
            vis: hir::Visibility::Priv,
            is_mut: true,
            is_record: true,
            name,
            generics: None,
            where_clause: None,
            fields,
            uses,
        });

        let mut path = base_type_path_from_scope(&ast_ctx.scope, &mut self.names, node.node_id().index() as u32);
        path.segments.push(hir::TypePathSegment::Plain {
            name
        });

        self.push_type(hir::Type::Path(hir::PathType {
            node_id: node.node_id().index() as u32,
            path,
        }))
    }

    fn visit_enum_record_type(&mut self, node: &AstNodeRef<EnumRecordType>) where Self: Sized {
        helpers::visit_enum_record_type(self, node);

        let mut variants = Vec::new();
        for variant in node.variants.iter().rev() {
            variants.push(self.convert_adt_enum_variant(variant));
        }
        variants.reverse();

        let span = self.spans[node.span()];
        let file_name = self.spans.get_file(span.file_id);

        let name = format!("__anon_record_enum_{file_name}_{}_{}", span.row, span.column);
        let name = self.names.add(&name);

        let ast_ctx = self.ctx.get_node_for(node);

        self.hir.add_adt_enum(ast_ctx.scope.clone(), hir::AdtEnum {
            node_id: node.node_id().index() as u32,
            attrs: vec![self.comp_gen_attr.clone()],
            vis: hir::Visibility::Priv,
            is_mut: true,
            is_record: true,
            name,
            generics: None,
            where_clause: None,
            variants,
        });

        let mut path = base_type_path_from_scope(&ast_ctx.scope, &mut self.names, node.node_id().index() as u32);
        path.segments.push(hir::TypePathSegment::Plain {
            name
        });

        self.push_type(hir::Type::Path(hir::PathType {
            node_id: node.node_id().index() as u32,
            path,
        }))
    }

    // =============================================================

    fn visit_visibility(&mut self, node: &AstNodeRef<Visibility>) where Self: Sized {
        helpers::visit_visibility(self, node);

        let vis = match &**node {
            Visibility::Pub{ .. }     => hir::Visibility::Pub,
            Visibility::Super{ .. }   => hir::Visibility::Super,
            Visibility::Lib{ .. }     => hir::Visibility::Lib,
            Visibility::Package{ .. } => hir::Visibility::Package,
            Visibility::Path{ .. }    => {
                let path = self.simple_path_stack.pop().unwrap();
                hir::Visibility::Path(path)
            },
        };
        self.vis_stack.push(vis);
    }

    fn visit_attribute(&mut self, node: &AstNodeRef<Attribute>) where Self: Sized {
        helpers::visit_attribute(self, node);
    }

    fn visit_contract(&mut self, node: &AstNodeRef<Contract>) where Self: Sized {
        helpers::visit_contract(self, node);
    }

    fn visit_generic_params(&mut self, node: &AstNodeRef<GenericParams>) where Self: Sized {
        helpers::visit_generic_params(self, node);
    }

    fn visit_generic_args(&mut self, node: &AstNodeRef<GenericArgs>) where Self: Sized {
        helpers::visit_generic_args(self, node);
    }

    fn visit_where_clause(&mut self, node: &AstNodeRef<WhereClause>) where Self: Sized {
        helpers::visit_where_clause(self, node);
    }

    fn visit_trait_bounds(&mut self, node: &AstNodeRef<TraitBounds>) where Self: Sized {
        helpers::visit_trait_bounds(self, node);
    }
}

fn base_type_path_from_scope(scope: &Scope, names: &mut NameTable, node_id: u32) -> hir::TypePath {
    let mut segments = Vec::new();
    for segment in scope.segments() {
        let name = names.add(&segment.name);
        segments.push(hir::TypePathSegment::Plain {
            name
        });
    }

    hir::TypePath {
        node_id,
        segments,
    }
}