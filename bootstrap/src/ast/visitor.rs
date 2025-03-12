use super::*;




pub trait Visitor {

    fn visit(&mut self, ast: &Ast) where Self: Sized {
        for item in &ast.items {
            self.visit_item(item);
        }
    }

    fn visit_simple_path(&mut self, node: &AstNodeRef<SimplePath>) where Self: Sized {
    }

    fn visit_expr_path(&mut self, node: &AstNodeRef<ExprPath>) where Self: Sized {
        helpers::visit_expr_path(self, node)
    }

    fn visit_type_path(&mut self, node: &AstNodeRef<TypePath>) where Self: Sized {
        helpers::visit_type_path(self, node)
    }

    fn visit_qualified_path(&mut self, node: &AstNodeRef<QualifiedPath>) where Self: Sized {
        helpers::visit_qualified_path(self, node)
    }

// =============================================================================================================================

    fn visit_item(&mut self, item: &Item) where Self: Sized {
        helpers::visit_item(self, item);
    }

    fn visit_trait_item(&mut self, item: &TraitItem) where Self: Sized {
        helpers::visit_trait_item(self, item);
    }

    fn visit_assoc_item(&mut self, item: &ImplItem) where Self: Sized {
        helpers::visit_assoc_item(self, item);
    }

    fn visit_extern_item(&mut self, item: &ExternItem) where Self: Sized {
        helpers::visit_extern_item(self, item);
    }

    fn visit_module(&mut self, node: &AstNodeRef<ModuleItem>) where Self: Sized {
        helpers::visit_module(self, node);
    }

    fn visit_use(&mut self, node: &AstNodeRef<UseItem>) where Self: Sized {
        helpers::visit_use(self, node);
    }

    fn visit_use_path(&mut self, node: &AstNodeRef<UsePath>) where Self: Sized {
        helpers::visit_use_path(self, node);
    }

    fn visit_function(&mut self, node: &AstNodeRef<Function>) where Self: Sized {
        helpers::visit_function(self, node, true, true);
    }

    fn visit_method(&mut self, node: &AstNodeRef<Method>) where Self: Sized {
        helpers::visit_method(self, node, true, true);
    }

    fn visit_type_alias(&mut self, node: &AstNodeRef<TypeAlias>) where Self: Sized {
        helpers::visit_type_alias(self, node, true);
    }

    fn visit_distinct_type(&mut self, node: &AstNodeRef<DistinctType>) where Self: Sized {
        helpers::visit_distinct_type(self, node, true);
    }

    fn visit_opaque_type(&mut self, node: &AstNodeRef<OpaqueType>) where Self: Sized {
        helpers::visit_opaque_type(self, node);
    }

    fn visit_struct(&mut self, node: &AstNodeRef<Struct>) where Self: Sized {
        helpers::visit_struct(self, node, true);
    }

    fn visit_reg_struct_field(&mut self, field: &RegStructField) where Self: Sized {
        helpers::visit_reg_struct_field(self, field);
    }

    fn visit_tuple_struct_field(&mut self, field: &TupleStructField) where Self: Sized {
        helpers::visit_tuple_struct_field(self, field);
    }

    fn visit_union(&mut self, node: &AstNodeRef<Union>) where Self: Sized {
        helpers::visit_union(self, node, true);
    }

    fn visit_enum(&mut self, node: &AstNodeRef<Enum>) where Self: Sized {
        helpers::visit_enum(self, node, true);
    }

    fn visit_enum_variant(&mut self, variant: &EnumVariant) where Self: Sized {
        helpers::visit_enum_variant(self, variant);
    }

    fn visit_bitfield(&mut self, node: &AstNodeRef<Bitfield>) where Self: Sized {
        helpers::visit_bitfield(self, node, true);
    }
    
    fn visit_bitfield_field(&mut self, field: &BitfieldField) where Self: Sized {
        helpers::visit_bitfield_field(self, field);
    }
    
    fn visit_const(&mut self, node: &AstNodeRef<Const>) where Self: Sized {
        helpers::visit_const(self, node);
    }
    
    fn visit_static(&mut self, node: &AstNodeRef<Static>) where Self: Sized {
        helpers::visit_static(self, node);
    }
    
    fn visit_property(&mut self, node: &AstNodeRef<Property>) where Self: Sized {
        helpers::visit_property(self, node);
    }
    
//--------------------------------------------------------------

    fn visit_trait(&mut self, node: &AstNodeRef<Trait>) where Self: Sized {
        helpers::visit_trait(self, node, true);
    }
    
    fn visit_trait_function(&mut self, node: &AstNodeRef<TraitFunction>) where Self: Sized {
        helpers::visit_trait_function(self, node, true, true);
    }

    fn visit_trait_method(&mut self, node: &AstNodeRef<TraitMethod>) where Self: Sized {
        helpers::visit_trait_method(self, node, true, true);
    }

    fn visit_trait_type_alias(&mut self, node: &AstNodeRef<TraitTypeAlias>) where Self: Sized {
        helpers::visit_trait_type_alias(self, node, true);
    }

    fn visit_trait_type_alias_override(&mut self, node: &AstNodeRef<TraitTypeAliasOverride>) where Self: Sized {
        helpers::visit_trait_type_alias_override(self, node);
    }

    fn visit_trait_const(&mut self, node: &AstNodeRef<TraitConst>) where Self: Sized {
        helpers::visit_trait_const(self, node);
    }

    fn visit_trait_const_override(&mut self, node: &AstNodeRef<TraitConstOverride>) where Self: Sized {
        helpers::visit_trait_const_override(self, node);
    }

    fn visit_trait_property(&mut self, node: &AstNodeRef<TraitProperty>) where Self: Sized {
        helpers::visit_trait_property(self, node);
    }

    fn visit_trait_property_override(&mut self, node: &AstNodeRef<TraitPropertyOverride>) where Self: Sized {
        helpers::visit_trait_property_override(self, node);
    }

//--------------------------------------------------------------

    fn visit_impl(&mut self, node: &AstNodeRef<Impl>) where Self: Sized {
        helpers::visit_impl(self, node, true);
    }
    
//--------------------------------------------------------------

    fn visit_extern_block(&mut self, node: &AstNodeRef<ExternBlock>) where Self: Sized {
        helpers::visit_extern_block(self, node);
    }
    
//--------------------------------------------------------------

    fn visit_op_trait(&mut self, node: &AstNodeRef<OpTrait>) where Self: Sized {
        helpers::visit_op_trait(self, node);
    }
    
    fn visit_op_use(&mut self, node: &AstNodeRef<OpUse>) where Self: Sized {
    }

    fn visit_precedence(&mut self, node: &AstNodeRef<Precedence>) where Self: Sized {
        helpers::visit_precedence(self, node);
    }
    
    fn visit_precedence_use(&mut self, node: &AstNodeRef<PrecedenceUse>) where Self: Sized {
    }

// =============================================================================================================================

    fn visit_block(&mut self, node: &AstNodeRef<Block>) where Self: Sized {
        helpers::visit_block(self, node);
    }

// =============================================================================================================================

    fn visit_stmt(&mut self, node: &Stmt) where Self: Sized {
        helpers::visit_stmt(self, node);
    }

    fn visit_empty_stmt(&mut self, node: &AstNodeRef<EmptyStmt>) where Self: Sized {
    }

    fn visit_var_decl(&mut self, node: &AstNodeRef<VarDecl>) where Self: Sized {
        helpers::visit_var_decl(self, node);
    }

    fn visit_defer(&mut self, node: &AstNodeRef<Defer>) where Self: Sized {
        helpers::visit_defer(self, node);
    }

    fn visit_err_defer(&mut self, node: &AstNodeRef<ErrDefer>) where Self: Sized {
        helpers::visit_err_defer(self, node);
    }

    fn visit_expr_stmt(&mut self, node: &AstNodeRef<ExprStmt>) where Self: Sized {
        helpers::visit_expr_stmt(self, node);
    }

// =============================================================================================================================

    fn visit_expr(&mut self, node: &Expr) where Self: Sized {
        helpers::visit_expr(self, node);
    }

    fn visit_literal_expr(&mut self, node: &AstNodeRef<LiteralExpr>) where Self: Sized {
        
    }

    fn visit_path_expr(&mut self, node: &AstNodeRef<PathExpr>) where Self: Sized {
        helpers::visit_path_expr(self, node);
    }

    fn visit_unit_expr(&mut self, node: &AstNodeRef<UnitExpr>) where Self: Sized {
    }

    fn visit_block_expr(&mut self, node: &AstNodeRef<BlockExpr>) where Self: Sized {
        helpers::visit_block_expr(self, node);
    }

    fn visit_prefix_expr(&mut self, node: &AstNodeRef<PrefixExpr>) where Self: Sized {
        helpers::visit_prefix_expr(self, node);
    }

    fn visit_postfix_expr(&mut self, node: &AstNodeRef<PostfixExpr>) where Self: Sized {
        helpers::visit_postfix_expr(self, node);
    }

    fn visit_binary_expr(&mut self, node: &AstNodeRef<InfixExpr>) where Self: Sized {
        helpers::visit_binary_expr(self, node);
    }

    fn visit_paren_expr(&mut self, node: &AstNodeRef<ParenExpr>) where Self: Sized {
        helpers::visit_paren_expr(self, node);
    }

    fn visit_inplace_expr(&mut self, node: &AstNodeRef<InplaceExpr>) where Self: Sized {
        helpers::visit_inplace_expr(self, node);
    }

    fn visit_type_cast_expr(&mut self, node: &AstNodeRef<TypeCastExpr>) where Self: Sized {
        helpers::visit_type_cast_expr(self, node);
    }

    fn visit_type_check_expr(&mut self, node: &AstNodeRef<TypeCheckExpr>) where Self: Sized {
        helpers::visit_type_check_expr(self, node);
    }

    fn visit_tuple_expr(&mut self, node: &AstNodeRef<TupleExpr>) where Self: Sized {
        helpers::visit_tuple_expr(self, node);
    }

    fn visit_array_expr(&mut self, node: &AstNodeRef<ArrayExpr>) where Self: Sized {
        helpers::visit_array_expr(self, node);
    }

    fn visit_struct_expr(&mut self, node: &AstNodeRef<StructExpr>) where Self: Sized {
        helpers::visit_struct_expr(self, node);
    }

    fn visit_index_expr(&mut self, node: &AstNodeRef<IndexExpr>) where Self: Sized {
        helpers::visit_index_expr(self, node);
    }

    fn visit_tuple_index_expr(&mut self, node: &AstNodeRef<TupleIndexExpr>) where Self: Sized {
        helpers::visit_tuple_index_expr(self, node);
    }

    fn visit_fn_call_expr(&mut self, node: &AstNodeRef<FnCallExpr>) where Self: Sized {
        helpers::visit_fn_call_expr(self, node);
    }

    fn visit_method_call_expr(&mut self, node: &AstNodeRef<MethodCallExpr>) where Self: Sized {
        helpers::visit_method_call_expr(self, node);
    }

    fn visit_field_access_expr(&mut self, node: &AstNodeRef<FieldAccessExpr>) where Self: Sized {
        helpers::visit_field_access_expr(self, node);
    }

    fn visit_closure_expr(&mut self, node: &AstNodeRef<ClosureExpr>) where Self: Sized {
        helpers::visit_closure_expr(self, node);
    }

    fn visit_full_range_expr(&mut self, node: &AstNodeRef<FullRangeExpr>) where Self: Sized {
    }

    fn visit_let_binding_expr(&mut self, node: &AstNodeRef<LetBindingExpr>) where Self: Sized {
        helpers::visit_let_binding_expr(self, node);
    }

    fn visit_if_expr(&mut self, node: &AstNodeRef<IfExpr>) where Self: Sized {
        helpers::visit_if_expr(self, node);
    }

    fn visit_loop_expr(&mut self, node: &AstNodeRef<LoopExpr>) where Self: Sized {
        helpers::visit_loop_expr(self, node);
    }

    fn visit_while_expr(&mut self, node: &AstNodeRef<WhileExpr>) where Self: Sized {
        helpers::visit_while_expr(self, node);
    }

    fn visit_do_while_expr(&mut self, node: &AstNodeRef<DoWhileExpr>) where Self: Sized {
        helpers::visit_do_while_expr(self, node);
    }

    fn visit_for_expr(&mut self, node: &AstNodeRef<ForExpr>) where Self: Sized {
        helpers::visit_for_expr(self, node);
    }

    fn visit_match_expr(&mut self, node: &AstNodeRef<MatchExpr>) where Self: Sized {
        helpers::visit_match_expr(self, node);
    }

    fn visit_break_expr(&mut self, node: &AstNodeRef<BreakExpr>) where Self: Sized {
        helpers::visit_break_expr(self, node);
    }

    fn visit_continue_expr(&mut self, node: &AstNodeRef<ContinueExpr>) where Self: Sized {
        
    }

    fn visit_fallthrough_expr(&mut self, node: &AstNodeRef<FallthroughExpr>) where Self: Sized {
        
    }

    fn visit_return_expr(&mut self, node: &AstNodeRef<ReturnExpr>) where Self: Sized {
        helpers::visit_return_expr(self, node);
    }

    fn visit_underscore_expr(&mut self, node: &AstNodeRef<UnderscoreExpr>) where Self: Sized {
    }

    fn visit_throw_expr(&mut self, node: &AstNodeRef<ThrowExpr>) where Self: Sized {
        helpers::visit_throw_expr(self, node);
    }

    fn visit_comma_expr(&mut self, node: &AstNodeRef<CommaExpr>) where Self: Sized {
        helpers::visit_comma_expr(self, node);
    }

    fn visit_when_expr(&mut self, node: &AstNodeRef<WhenExpr>) where Self: Sized {
        helpers::visit_when_expr(self, node);
    }

// =============================================================================================================================

    fn visit_pattern(&mut self, node: &Pattern) where Self: Sized {
        helpers::visit_pattern(self, node);
    }

    fn visit_literal_pattern(&mut self, node: &AstNodeRef<LiteralPattern>) where Self: Sized {
    }

    fn visit_identifier_pattern(&mut self, node: &AstNodeRef<IdentifierPattern>) where Self: Sized {
        helpers::visit_identifier_pattern(self, node);
    }

    fn visit_path_pattern(&mut self, node: &AstNodeRef<PathPattern>) where Self: Sized {
        helpers::visit_path_pattern(self, node);
    }

    fn visit_wildcard_pattern(&mut self, node: &AstNodeRef<WildcardPattern>) where Self: Sized {
    }

    fn visit_rest_pattern(&mut self, node: &AstNodeRef<RestPattern>) where Self: Sized {
    }

    fn visit_range_pattern(&mut self, node: &AstNodeRef<RangePattern>) where Self: Sized {
        helpers::visit_range_pattern(self, node);
    }

    fn visit_reference_pattern(&mut self, node: &AstNodeRef<ReferencePattern>) where Self: Sized {
        helpers::visit_reference_pattern(self, node);
    }

    fn visit_struct_pattern(&mut self, node: &AstNodeRef<StructPattern>) where Self: Sized {
        helpers::visit_struct_pattern(self, node);
    }

    fn visit_tuple_struct_pattern(&mut self, node: &AstNodeRef<TupleStructPattern>) where Self: Sized {
        helpers::visit_tuple_struct_pattern(self, node);
    }

    fn visit_tuple_pattern(&mut self, node: &AstNodeRef<TuplePattern>) where Self: Sized {
        helpers::visit_tuple_pattern(self, node);
    }

    fn visit_grouped_pattern(&mut self, node: &AstNodeRef<GroupedPattern>) where Self: Sized {
        helpers::visit_grouped_pattern(self, node);
    }

    fn visit_slice_pattern(&mut self, node: &AstNodeRef<SlicePattern>) where Self: Sized {
        helpers::visit_slice_pattern(self, node);
    }

    fn visit_enum_member_pattern(&mut self, node: &AstNodeRef<EnumMemberPattern>) where Self: Sized {
    }

    fn visit_alternative_pattern(&mut self, node: &AstNodeRef<AlternativePattern>) where Self: Sized {
        helpers::visit_alternative_pattern(self, node);
    }

    fn visit_type_check_pattern(&mut self, node: &AstNodeRef<TypeCheckPattern>) where Self: Sized {
        helpers::visit_type_check_pattern(self, node);
    }

// =============================================================================================================================

    fn visit_type(&mut self, node: &Type) where Self: Sized {
        helpers::visit_type(self, node);
    }

    fn visit_paren_type(&mut self, node: &AstNodeRef<ParenthesizedType>) where Self: Sized {
        helpers::visit_paren_type(self, node);
    }

    fn visit_primitive_type(&mut self, node: &AstNodeRef<PrimitiveType>) where Self: Sized {
        
    }

    fn visit_unit_type(&mut self, node: &AstNodeRef<UnitType>) where Self: Sized {
    }

    fn visit_never_type(&mut self, node: &AstNodeRef<NeverType>) where Self: Sized {
    }

    fn visit_path_type(&mut self, node: &AstNodeRef<PathType>) where Self: Sized {
        helpers::visit_path_type(self, node);
    }

    fn visit_tuple_type(&mut self, node: &AstNodeRef<TupleType>) where Self: Sized {
        helpers::visit_tuple_type(self, node);
    }

    fn visit_array_type(&mut self, node: &AstNodeRef<ArrayType>) where Self: Sized {
        helpers::visit_array_type(self, node);
    }

    fn visit_slice_type(&mut self, node: &AstNodeRef<SliceType>) where Self: Sized {
        helpers::visit_slice_type(self, node);
    }

    fn visit_string_slice_type(&mut self, node: &AstNodeRef<StringSliceType>) where Self: Sized {
    }

    fn visit_pointer_type(&mut self, node: &AstNodeRef<PointerType>) where Self: Sized {
        helpers::visit_pointer_type(self, node);
    }

    fn visit_reference_type(&mut self, node: &AstNodeRef<ReferenceType>) where Self: Sized {
        helpers::visit_reference_type(self, node);
    }

    fn visit_optional_type(&mut self, node: &AstNodeRef<OptionalType>) where Self: Sized {
        helpers::visit_optional_type(self, node);
    }

    fn visit_fn_type(&mut self, node: &AstNodeRef<FnType>) where Self: Sized {
        helpers::visit_fn_type(self, node);
    }

    fn visit_record_type(&mut self, node: &AstNodeRef<RecordType>) where Self: Sized {
        helpers::visit_record_type(self, node);
    }

    fn visit_enum_record_type(&mut self, node: &AstNodeRef<EnumRecordType>) where Self: Sized {
        helpers::visit_enum_record_type(self, node);
    }

// =============================================================================================================================

    fn visit_visibility(&mut self, node: &AstNodeRef<Visibility>) where Self: Sized {
        helpers::visit_visibility(self, node);
    }

    fn visit_attribute(&mut self, node: &AstNodeRef<Attribute>) where Self: Sized {
        helpers::visit_attribute(self, node);
    }

// =============================================================================================================================

    fn visit_contract(&mut self, node: &AstNodeRef<Contract>) where Self: Sized {
        helpers::visit_contract(self, node);
    }

// =============================================================================================================================

    fn visit_generic_params(&mut self, node: &AstNodeRef<GenericParams>) where Self: Sized {
        helpers::visit_generic_params(self, node);
    }

    fn visit_generic_type_param(&mut self, node: &AstNodeRef<GenericTypeParam>) where Self: Sized {
        helpers::visit_generic_type_param(self, node)
    }

    fn visit_generic_type_spec(&mut self, node: &AstNodeRef<GenericTypeSpec>) where Self: Sized {
        helpers::visit_generic_type_spec(self, node)
    }

    fn visit_generic_const_param(&mut self, node: &AstNodeRef<GenericConstParam>) where Self: Sized {
        helpers::visit_generic_const_param(self, node)
    }

    fn visit_generic_const_spec(&mut self, node: &AstNodeRef<GenericConstSpec>) where Self: Sized {
        helpers::visit_generic_const_spec(self, node)
    }

    fn visit_generic_param_pack(&mut self, node: &AstNodeRef<GenericParamPack>) where Self: Sized {
        helpers::visit_generic_param_pack(self, node)
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

pub mod helpers {
    use super::*;

    pub fn visit<T: Visitor>(visitor: &mut T, ast: &Ast) {
        for item in &ast.items {
            visitor.visit_item(item);
        }
    }

    pub fn visit_expr_path<T: Visitor>(visitor: &mut T, node: &AstNodeRef<ExprPath>) {
        for iden in &node.idens {
            if let Some(gen_args) = &iden.gen_args {
                visitor.visit_generic_args(gen_args)
            }
        }
    }

    pub fn visit_type_path<T: Visitor>(visitor: &mut T, node: &AstNodeRef<TypePath>) {
        for iden in &node.idens {
            match iden {
                TypePathIdentifier::Plain { span, name }            => {},
                TypePathIdentifier::GenArg { span, name, gen_args } => visitor.visit_generic_args(gen_args),
                TypePathIdentifier::Fn { span, name, params, ret }  => {
                    for param_ty in params {
                        visitor.visit_type(param_ty);
                    }
                    if let Some(ret) = ret {
                        visitor.visit_type(ret);
                    }
                },
            }
        }
    }

    pub fn visit_qualified_path<T: Visitor>(visitor: &mut T, node: &AstNodeRef<QualifiedPath>) {
        visitor.visit_type(&node.ty);
        if let Some(bound) = &node.bound {
            visitor.visit_type_path(bound);
        }

        if let Some(gen_args) = &node.sub_path.gen_args {
            visitor.visit_generic_args(gen_args)
        }
    }

// =============================================================================================================================

    pub fn visit_item<T: Visitor>(visitor: &mut T, item: &Item) {
        match item {
            Item::Module(node)        => visitor.visit_module(node),
            Item::Use(node)           => visitor.visit_use(node),
            Item::Function(node)      => visitor.visit_function(node),
            Item::TypeAlias(node)     => visitor.visit_type_alias(node),
            Item::DistinctType(node)  => visitor.visit_distinct_type(node),
            Item::OpaqueType(node)    => visitor.visit_opaque_type(node),
            Item::Struct(node)        => visitor.visit_struct(node),
            Item::Union(node)         => visitor.visit_union(node),
            Item::Enum(node)          => visitor.visit_enum(node),
            Item::Bitfield(node)      => visitor.visit_bitfield(node),
            Item::Const(node)         => visitor.visit_const(node),
            Item::Static(node)        => visitor.visit_static(node),
            Item::Property(node)      => visitor.visit_property(node),
            Item::Trait(node)         => visitor.visit_trait(node),
            Item::Impl(node)          => visitor.visit_impl(node),
            Item::Extern(node)        => visitor.visit_extern_block(node),
            Item::OpTrait(node)       => visitor.visit_op_trait(node),
            Item::OpUse(node)         => visitor.visit_op_use(node),
            Item::Precedence(node)    => visitor.visit_precedence(node),
            Item::PrecedenceUse(node) => visitor.visit_precedence_use(node),
        }
    }

    pub fn visit_trait_item<T: Visitor>(visitor: &mut T, item: &TraitItem) {
        match item {
            TraitItem::Function(node)          => visitor.visit_trait_function(node),
            TraitItem::Method(node)            => visitor.visit_trait_method(node),
            TraitItem::TypeAlias(node)         => visitor.visit_trait_type_alias(node),
            TraitItem::TypeAliasOverride(node) => visitor.visit_trait_type_alias_override(node),
            TraitItem::Const(node)             => visitor.visit_trait_const(node),
            TraitItem::ConstOverride(node)     => visitor.visit_trait_const_override(node),
            TraitItem::Property(node)          => visitor.visit_trait_property(node),
            TraitItem::PropertyOverride(node)  => visitor.visit_trait_property_override(node),
        }
    }

    pub fn visit_assoc_item<T: Visitor>(visitor: &mut T, item: &ImplItem) {
        match item {
            ImplItem::Function(node)  => visitor.visit_function(node),
            ImplItem::Method(node)    => visitor.visit_method(node),
            ImplItem::TypeAlias(node) => visitor.visit_type_alias(node),
            ImplItem::Const(node)     => visitor.visit_const(node),
            ImplItem::Static(node)    => visitor.visit_static(node),
            ImplItem::Property(node)  => visitor.visit_property(node),
        }
    }

    pub fn visit_extern_item<T: Visitor>(visitor: &mut T, item: &ExternItem) {
        match item {
            ExternItem::Function(node) => visitor.visit_function(node),
            ExternItem::Static(node)   => visitor.visit_static(node),
        }
    }

    pub fn visit_module<T: Visitor>(visitor: &mut T, node: &AstNodeRef<ModuleItem>) {
        for attr in &node.attrs {
            visitor.visit_attribute(attr);
        }

        if let Some(vis) = &node.vis {
            visitor.visit_visibility(vis);
        }
        if let Some(block) = &node.block {
            visitor.visit_block(block);
        }
    }

    pub fn visit_use<T: Visitor>(visitor: &mut T, node: &AstNodeRef<UseItem>) {
        for attr in &node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(vis) = &node.vis {
            visitor.visit_visibility(vis);
        }
        visitor.visit_use_path(&node.path);
    }

    pub fn visit_use_path<T: Visitor>(visitor: &mut T, node: &AstNodeRef<UsePath>) {
        match &**node {
            UsePath::SelfPath { span, node_id, alias } => {},
            UsePath::SubPaths { span, node_id, segments, sub_paths } => for path in sub_paths {
                visitor.visit_use_path(path);
            },
            UsePath::Alias { span, node_id, segments, alias } => {},
        }
    }

    pub fn visit_function<T: Visitor>(visitor: &mut T, node: &AstNodeRef<Function>, do_generics: bool, do_body: bool) {
        for attr in &node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(vis) = &node.vis {
            visitor.visit_visibility(vis);
        }
        if do_generics {
            if let Some(generics) = &node.generics {
                visitor.visit_generic_params(generics);
            }
        }

        for param in &node.params {
            visit_fn_param(visitor, param);
        }

        if let Some(ret) = &node.returns {
            visit_fn_return(visitor, ret);
        }

        if do_generics {
            if let Some(where_clause) = &node.where_clause {
                visitor.visit_where_clause(where_clause);
            }
        }

        for contract in &node.contracts {
            visitor.visit_contract( contract);
        }

        if do_body {
            if let Some(body) = &node.body {
                visitor.visit_block(body);
            }
        }
    }

    pub fn visit_fn_param<T: Visitor>(visitor: &mut T, param: &FnParam) {
        for name in &param.names {
            for attr in &name.attrs {
                visitor.visit_attribute(attr);
            }
            visitor.visit_pattern(&name.pattern);
        }

        visitor.visit_type(&param.ty);
        if let Some(expr) = &param.def_val {
            visitor.visit_expr(expr);
        }
    }

    pub fn visit_fn_return<T: Visitor>(visitor: &mut T, ret: &FnReturn) {
        match ret {
            FnReturn::Type{ span, ty } => visitor.visit_type(&ty),
            FnReturn::Named{ span, vars } => for (_, ty) in vars {
                visitor.visit_type(&ty);
            },
        }
    }

    pub fn visit_method<T: Visitor>(visitor: &mut T, node: &AstNodeRef<Method>, do_generics: bool, do_body: bool) {
        for attr in &node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(vis) = &node.vis {
            visitor.visit_visibility(vis);
        }
        if do_generics {
            if let Some(generics) = &node.generics {
                visitor.visit_generic_params(generics);
            }
        }

        if let FnReceiver::SelfTyped { span, is_mut, ty } = &node.receiver {
            visitor.visit_type(ty);
        }

        for param in &node.params {
            visit_fn_param(visitor, param);
        }

        if let Some(ret) = &node.returns {
            visit_fn_return(visitor, ret);
        }

        if do_generics {
            if let Some(where_clause) = &node.where_clause {
                visitor.visit_where_clause(where_clause);
            }
        }

        for contract in &node.contracts {
            visitor.visit_contract( contract);
        }

        if do_body {
            visitor.visit_block(&node.body);
        }
    }

    pub fn visit_type_alias<T: Visitor>(visitor: &mut T, node: &AstNodeRef<TypeAlias>, do_generics: bool) {
        for attr in &node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(vis) = &node.vis {
            visitor.visit_visibility(vis);
        }
        if do_generics {
            if let Some(generics) = &node.generics {
                visitor.visit_generic_params(generics);
            }
        }
        visitor.visit_type(&node.ty);
    }

    pub fn visit_distinct_type<T: Visitor>(visitor: &mut T, node: &AstNodeRef<DistinctType>, do_generics: bool) {
        for attr in &node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(vis) = &node.vis {
            visitor.visit_visibility(vis);
        }
        if do_generics {
            if let Some(generics) = &node.generics {
                visitor.visit_generic_params(generics);
            }
        }
        visitor.visit_type(&node.ty);
    }

    pub fn visit_opaque_type<T: Visitor>(visitor: &mut T, node: &AstNodeRef<OpaqueType>) {
        for attr in &node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(vis) = &node.vis {
            visitor.visit_visibility(vis);
        }
        if let Some(size) = &node.size {
            visitor.visit_expr(size);
        }
    }

    pub fn visit_struct<T: Visitor>(visitor: &mut T, node: &AstNodeRef<Struct>, do_generics: bool) {
        match &**node {
            Struct::Regular { span, node_id, attrs, vis, is_mut, is_record, name, generics, where_clause, fields } => {
                for attr in attrs {
                    visitor.visit_attribute(attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(vis);
                }
                if do_generics {
                    if let Some(generics) = generics {
                        visitor.visit_generic_params(generics);
                    }
                    if let Some(where_clause) = where_clause {
                        visitor.visit_where_clause(where_clause);
                    }
                }
                for field in fields {
                    visitor.visit_reg_struct_field(field);
                }
            },
            Struct::Tuple { span, node_id, attrs, vis, is_mut, is_record, name, generics, where_clause, fields } => {
                for attr in attrs {
                    visitor.visit_attribute(attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(vis);
                }
                if do_generics {
                    if let Some(generics) = generics {
                        visitor.visit_generic_params(generics);
                    }
                    if let Some(where_clause) = where_clause {
                        visitor.visit_where_clause(where_clause);
                    }
                }
                for field in fields {
                    visitor.visit_tuple_struct_field(field);
                }
            },
            Struct::Unit { span, node_id, attrs, vis, name } => {
                for attr in attrs {
                    visitor.visit_attribute(attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(vis);
                }
            },
        }
    }

    pub fn visit_reg_struct_field<T: Visitor>(visitor: &mut T, field: &RegStructField) {
        match field {
            RegStructField::Field { span, attrs, vis, is_mut, names, ty, def } => {
                for attr in attrs {
                    visitor.visit_attribute(attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(vis);
                }
                visitor.visit_type(ty);
                if let Some(expr) = def {
                    visitor.visit_expr(expr);
                }
            },
            RegStructField::Use { span, attrs, vis, is_mut, path } => {
                for attr in attrs {
                    visitor.visit_attribute(attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(vis);
                }
                visitor.visit_type_path(path);
            },
        }
    }

    pub fn visit_tuple_struct_field<T: Visitor>(visitor: &mut T, field: &TupleStructField) {
        for attr in &field.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(vis) = &field.vis {
            visitor.visit_visibility(vis);
        }
        visitor.visit_type(&field.ty);
        if let Some(expr) = &field.def {
            visitor.visit_expr(expr);
        }
    }

    pub fn visit_union<T: Visitor>(visitor: &mut T, node: &AstNodeRef<Union>, do_generics: bool) {
        for attr in &node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(vis) = &node.vis {
            visitor.visit_visibility(vis);
        }
        if do_generics {
            if let Some(generics) = &node.generics {
                visitor.visit_generic_params(generics);
            }
            if let Some(where_clause) = &node.where_clause {
                visitor.visit_where_clause(where_clause);
            }
        }
        for field in &node.fields {
            for attr in &field.attrs {
                visitor.visit_attribute(attr);
            }
            if let Some(vis) = &field.vis {
                visitor.visit_visibility(vis);
            }
            visitor.visit_type(&field.ty);
        }
    }

    pub fn visit_enum<T: Visitor>(visitor: &mut T, node: &AstNodeRef<Enum>, do_generics: bool) {
        match &**node {
            Enum::Adt { span, node_id, attrs, vis, is_mut, is_record, name, generics, where_clause, variants } => {
                for attr in attrs {
                    visitor.visit_attribute(attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(vis);
                }
                if do_generics {
                    if let Some(generics) = &generics {
                        visitor.visit_generic_params(generics);
                    }
                    if let Some(where_clause) = &where_clause {
                        visitor.visit_where_clause(where_clause);
                    }
                }
                for variant in variants {
                    visitor.visit_enum_variant(variant);
                }
            },
            Enum::Flag { span, node_id, attrs, vis, name, variants } => {
                for attr in attrs {
                    visitor.visit_attribute(attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(vis);
                }
                for variant in variants {
                    for attr in &variant.attrs {
                        visitor.visit_attribute(attr);
                    }
                    if let Some(discriminant) = &variant.discriminant {
                        visitor.visit_expr(discriminant);
                    }
                }
            },
        }
    }

    pub fn visit_enum_variant<T: Visitor>(visitor: &mut T, variant: &EnumVariant) {
        match variant {
            EnumVariant::Struct { span, attrs, is_mut, name, fields, discriminant } => {
                for attr in attrs {
                    visitor.visit_attribute(attr);
                }
                for field in fields {
                    visitor.visit_reg_struct_field(field);
                }
                if let Some(discriminant) = discriminant {
                    visitor.visit_expr(discriminant);
                }
            },
            EnumVariant::Tuple { span, attrs, is_mut, name, fields, discriminant } => {
                for attr in attrs {
                    visitor.visit_attribute(attr);
                }
                for field in fields {
                    visitor.visit_tuple_struct_field(field);
                }
                if let Some(discriminant) = discriminant {
                    visitor.visit_expr(discriminant);
                }
            },
            EnumVariant::Fieldless { span, attrs, name, discriminant } => {
                for attr in attrs {
                    visitor.visit_attribute(attr);
                }
                if let Some(discriminant) = discriminant {
                    visitor.visit_expr(discriminant);
                }
            },
        }
    }

    pub fn visit_bitfield<T: Visitor>(visitor: &mut T, node: &AstNodeRef<Bitfield>, do_generics: bool) {
        for attr in &node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(vis) = &node.vis {
            visitor.visit_visibility(vis);
        }
        if do_generics {
            if let Some(generics) = &node.generics {
                visitor.visit_generic_params(generics);
            }
            if let Some(where_clause) = &node.where_clause {
                visitor.visit_where_clause(where_clause);
            }
        }
        for field in &node.fields {
            visitor.visit_bitfield_field(field);
        }
    }

    pub fn visit_bitfield_field<T: Visitor>(visitor: &mut T, field: &BitfieldField) {
        match field {
            BitfieldField::Field { span, attrs, vis, is_mut, names, ty, bits, def } => {
                for attr in attrs {
                    visitor.visit_attribute(attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(vis);
                }
                if let Some(bits) = bits {
                    visitor.visit_expr(bits);
                }
                if let Some(def) = def {
                    visitor.visit_expr(def);
                }
            },
            BitfieldField::Use { span, attrs, vis, is_mut, path, bits } => {
                for attr in attrs {
                    visitor.visit_attribute(attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(vis);
                }
                visitor.visit_type_path(path);
                if let Some(bits) = bits {
                    visitor.visit_expr(bits);
                }
            },
        }
    }

    pub fn visit_const<T: Visitor>(visitor: &mut T, node: &AstNodeRef<Const>) {
        for attr in &node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(vis) = &node.vis {
            visitor.visit_visibility(vis);
        }
        if let Some(ty) = &node.ty {
            visitor.visit_type(ty);
        }
        visitor.visit_expr(&node.val)
    }

    pub fn visit_static<T: Visitor>(visitor: &mut T, node: &AstNodeRef<Static>) {
        match &**node {
            Static::Static { span, node_id, attrs, vis, name, ty, val } => {
                for attr in attrs {
                    visitor.visit_attribute(attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(vis);
                }
                if let Some(ty) = ty {
                    visitor.visit_type(ty);
                }
                visitor.visit_expr(val);
            },
            Static::Tls { span, node_id, attrs, vis, is_mut, name, ty, val } => {
                for attr in attrs {
                    visitor.visit_attribute(attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(vis);
                }
                if let Some(ty) = ty {
                    visitor.visit_type(ty);
                }
                visitor.visit_expr(val);
            },
            Static::Extern { span, node_id, attrs, vis, abi, is_mut, name, ty } => {
                for attr in attrs {
                    visitor.visit_attribute(attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(vis);
                }
                visitor.visit_type(ty);
            },
        }
    }

    pub fn visit_property<T: Visitor>(visitor: &mut T, node: &AstNodeRef<Property>) {
        for attr in &node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(vis) = &node.vis {
            visitor.visit_visibility(vis);
        }
        if let Some((_, get)) = &node.get {
            visitor.visit_expr(get);
        }
        if let Some((_, ref_get)) = &node.ref_get {
            visitor.visit_expr(ref_get);
        }
        if let Some((_, mut_get)) = &node.mut_get {
            visitor.visit_expr(mut_get);
        }
        if let Some((_, set)) = &node.set {
            visitor.visit_expr(set);
        }
    }

//--------------------------------------------------------------
// <T: Visitor>(visitor: &mut T, node: &AstNodeRef<>)

    pub fn visit_trait<T: Visitor>(visitor: &mut T, node: &AstNodeRef<Trait>, do_generics: bool) {
        for attr in &node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(vis) = &node.vis {
            visitor.visit_visibility(vis);
        }
        if do_generics {
            if let Some(generics) = &node.generics {
                visitor.visit_generic_params(generics);
            }
        }
        if let Some(bounds) = &node.bounds {
            visitor.visit_trait_bounds(bounds);
        }
        if do_generics {
            if let Some(where_clause) = &node.where_clause {
                visitor.visit_where_clause(where_clause);
            }
        }
        for item in &node.assoc_items {
            visitor.visit_trait_item(item);
        }
    }

    pub fn visit_trait_function<T: Visitor>(visitor: &mut T, node: &AstNodeRef<TraitFunction>, do_generics: bool, do_body: bool) {
        for attr in &node.attrs {
            visitor.visit_attribute(attr);
        }

        if do_generics {
            if let Some(generics) = &node.generics {
                visitor.visit_generic_params(generics);
            }
        }

        for param in &node.params {
            visit_fn_param(visitor, param);
        }

        if let Some(ret) = &node.returns {
            visit_fn_return(visitor, ret);
        }

        if do_generics {
            if let Some(where_clause) = &node.where_clause {
                visitor.visit_where_clause(where_clause);
            }
        }

        for contract in &node.contracts {
            visitor.visit_contract( contract);
        }

        if do_body {
            if let Some(body) = &node.body {
                visitor.visit_block(body);
            }
        }
    }

    pub fn visit_trait_method<T: Visitor>(visitor: &mut T, node: &AstNodeRef<TraitMethod>, do_generics: bool, do_body: bool) {
        for attr in &node.attrs {
            visitor.visit_attribute(attr);
        }
        if do_generics {
            if let Some(generics) = &node.generics {
                visitor.visit_generic_params(generics);
            }
        }
            
        if let FnReceiver::SelfTyped { ty, .. } = &node.receiver {
            visitor.visit_type(ty);
        }

        for param in &node.params {
            visit_fn_param(visitor, param);
        }

        if let Some(ret) = &node.returns {
            visit_fn_return(visitor, ret);
        }

        if do_generics {
            if let Some(where_clause) = &node.where_clause {
                visitor.visit_where_clause(where_clause);
            }
        }

        for contract in &node.contracts {
            visitor.visit_contract( contract);
        }

        if do_body {
            if let Some(body) = &node.body {
                visitor.visit_block(body);
            }
        }
    }

    pub fn visit_trait_type_alias<T: Visitor>(visitor: &mut T, node: &AstNodeRef<TraitTypeAlias>, do_generics: bool) {
        for attr in &node.attrs {
            visitor.visit_attribute(attr);
        }
        if do_generics {
            if let Some(generics) = &node.generics {
                visitor.visit_generic_params(generics);
            }
            for bound in &node.bounds {
                match bound {
                    GenericTypeBound::Type(path) => visitor.visit_type_path(path),
                }
            }
            if let Some(where_clause) = &node.where_clause {
                visitor.visit_where_clause(where_clause);
            }
        }
        if let Some(def) = &node.def {
            visitor.visit_type(def);
        }
    }

    pub fn visit_trait_type_alias_override<T: Visitor>(visitor: &mut T, node: &AstNodeRef<TraitTypeAliasOverride>) {
        visitor.visit_type(&node.ty);
    }

    pub fn visit_trait_const<T: Visitor>(visitor: &mut T, node: &AstNodeRef<TraitConst>) {
        for attr in &node.attrs {
            visitor.visit_attribute(attr);
        }
        visitor.visit_type(&node.ty);
        if let Some(def) = &node.def {
            visitor.visit_expr(def);
        }
    }

    pub fn visit_trait_const_override<T: Visitor>(visitor: &mut T, node: &AstNodeRef<TraitConstOverride>) {
        visitor.visit_expr(&node.expr);
    }

    pub fn visit_trait_property<T: Visitor>(visitor: &mut T, node: &AstNodeRef<TraitProperty>) {
        for attr in &node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some((_, Some(expr))) = &node.get {
            visitor.visit_expr(expr);
        }
        if let Some((_, Some(expr))) = &node.ref_get {
            visitor.visit_expr(expr);
        }
        if let Some((_, Some(expr))) = &node.mut_get {
            visitor.visit_expr(expr);
        }
        if let Some((_, Some(expr))) = &node.set {
            visitor.visit_expr(expr);
        }
    }

    pub fn visit_trait_property_override<T: Visitor>(visitor: &mut T, node: &AstNodeRef<TraitPropertyOverride>) {
        if let Some(expr) = &node.get {
            visitor.visit_expr(expr);
        }
        if let Some(expr) = &node.ref_get {
            visitor.visit_expr(expr);
        }
        if let Some(expr) = &node.mut_get {
            visitor.visit_expr(expr);
        }
        if let Some(expr) = &node.set {
            visitor.visit_expr(expr);
        }
    }

//--------------------------------------------------------------

    pub fn visit_impl<T: Visitor>(visitor: &mut T, node: &AstNodeRef<Impl>, do_generics: bool) {
        for attr in &node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(vis) = &node.vis {
            visitor.visit_visibility(vis);
        }
        if do_generics {
            if let Some(generics) = &node.generics {
                visitor.visit_generic_params(generics)
            }
        }
        visitor.visit_type(&node.ty);
        if let Some(impl_trait) = &node.impl_trait {
            visitor.visit_type_path(impl_trait);
        }
        if do_generics {
            if let Some(where_clause) = &node.where_clause {
                visitor.visit_where_clause(where_clause);
            }
        }
        for item in &node.assoc_items {
            visitor.visit_assoc_item(item);
        }
    }

//--------------------------------------------------------------

    pub fn visit_extern_block<T: Visitor>(visitor: &mut T, node: &AstNodeRef<ExternBlock>) {
        let node = &**node;
        for attr in &node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(vis) = &node.vis {
            visitor.visit_visibility(vis);
        }
        for item in &node.items {
            visitor.visit_extern_item(item);
        }
    }

//--------------------------------------------------------------

    pub fn visit_op_trait<T: Visitor>(visitor: &mut T, node: &AstNodeRef<OpTrait>) {
        match &**node {
            OpTrait::Base { span, node_id, attrs, vis, name, precedence, elems } => {
                for attr in attrs {
                    visitor.visit_attribute(attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(vis);
                }
                for elem in elems {
                    visit_op_elem(visitor, elem);
                }
            },
            OpTrait::Extended { span, node_id, attrs, vis, name, bases, elems } => {
                for attr in attrs {
                    visitor.visit_attribute(attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(vis);
                }
                for base in bases {
                    visitor.visit_simple_path(base);
                }
                for elem in elems {
                    visit_op_elem(visitor, elem);
                }
            },
        }
    }

    pub fn visit_op_elem<T: Visitor>(visitor: &mut T, elem: &OpElem) {
        match elem {
            OpElem::Def { span,op_type, op, name, ret, def } => {
                if let Some(ret) = ret {
                    visitor.visit_type(ret);
                }
                if let Some(def) = def {
                    visitor.visit_expr(def);
                }
            },
            OpElem::Extend { span, op_type, op, def } => {
                visitor.visit_expr(def);
            },
            OpElem::Contract { span, expr } => {
                visitor.visit_block_expr(expr);
            },
        }
    }

    pub fn visit_precedence<T: Visitor>(visitor: &mut T, node: &AstNodeRef<Precedence>) {
        for attr in &node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(vis) = &node.vis {
            visitor.visit_visibility(vis);
        }
    }

// =============================================================================================================================

    pub fn visit_block<T: Visitor>(visitor: &mut T, node: &AstNodeRef<Block>) {
        for stmt in &node.stmts {
            visitor.visit_stmt(stmt);
        }
        if let Some(final_expr) = &node.final_expr {
            //visitor.visit_expr_stmt(final_expr);
        }
    }

// =============================================================================================================================

    pub fn visit_stmt<T: Visitor>(visitor: &mut T, node: &Stmt) {
        match node {
            Stmt::Empty(item)    => visitor.visit_empty_stmt(item),
            Stmt::Item(item)     => visitor.visit_item(item),
            Stmt::VarDecl(node)  => visitor.visit_var_decl(node),
            Stmt::Defer(node)    => visitor.visit_defer(node),
            Stmt::ErrDefer(node) => visitor.visit_err_defer(node),
            Stmt::Expr(node)     => visitor.visit_expr_stmt(node),
        }
    }

    pub fn visit_var_decl<T: Visitor>(visitor: &mut T, node: &AstNodeRef<VarDecl>) {
        match &**node {
            VarDecl::Named { span, node_id, attrs, names, expr } => {
                for attr in attrs {
                    visitor.visit_attribute(attr);
                }
                visitor.visit_expr(expr);
            },
            VarDecl::Let { span, node_id, attrs, pattern, ty, expr, else_block } => {
                for attr in attrs {
                    visitor.visit_attribute(attr);
                }
                visitor.visit_pattern(pattern);
                if let Some(ty) = ty {
                    visitor.visit_type(ty);
                }
                if let Some(expr) = expr {
                    visitor.visit_expr(expr);
                }
                if let Some(else_block) = else_block {
                    visitor.visit_block_expr(else_block);
                }
            },
        }
    }

    pub fn visit_defer<T: Visitor>(visitor: &mut T, node: &AstNodeRef<Defer>) {
        for attr in &node.attrs {
            visitor.visit_attribute(attr);
        }
        visitor.visit_expr(&node.expr);
    }

    pub fn visit_err_defer<T: Visitor>(visitor: &mut T, node: &AstNodeRef<ErrDefer>) {
        for attr in &node.attrs {
            visitor.visit_attribute(attr);
        }
        visitor.visit_expr(&node.expr);
    }

    pub fn visit_expr_stmt<T: Visitor>(visitor: &mut T, node: &AstNodeRef<ExprStmt>) {
        for attr in &node.attrs {
            visitor.visit_attribute(attr);
        }
        visitor.visit_expr(&node.expr);
    }


// =============================================================================================================================

    pub fn visit_expr<T: Visitor>(visitor: &mut T, node: &Expr) {
        match node {
            Expr::Literal(node)        => visitor.visit_literal_expr(node),
            Expr::Path(node)           => visitor.visit_path_expr(node),
            Expr::Unit(node)           => visitor.visit_unit_expr(node),
            Expr::Block(node)          => visitor.visit_block_expr(node),
            Expr::Prefix(node)         => visitor.visit_prefix_expr(node),
            Expr::Postfix(node)        => visitor.visit_postfix_expr(node),
            Expr::Infix(node)          => visitor.visit_binary_expr(node),
            Expr::Paren(node)          => visitor.visit_paren_expr(node),
            Expr::Inplace(node)        => visitor.visit_inplace_expr(node),
            Expr::TypeCast(node)       => visitor.visit_type_cast_expr(node),
            Expr::TypeCheck(node)      => visitor.visit_type_check_expr(node),
            Expr::Tuple(node)          => visitor.visit_tuple_expr(node),
            Expr::Array(node)          => visitor.visit_array_expr(node),
            Expr::Struct(node)         => visitor.visit_struct_expr(node),
            Expr::Index(node)          => visitor.visit_index_expr(node),
            Expr::TupleIndex(node)     => visitor.visit_tuple_index_expr(node),
            Expr::FnCall(node)         => visitor.visit_fn_call_expr(node),
            Expr::Method(node)         => visitor.visit_method_call_expr(node),
            Expr::FieldAccess(node)    => visitor.visit_field_access_expr(node),
            Expr::Closure(node)        => visitor.visit_closure_expr(node),
            Expr::FullRange(node)      => visitor.visit_full_range_expr(node),
            Expr::If(node)             => visitor.visit_if_expr(node),
            Expr::Let(node)            => visitor.visit_let_binding_expr(node),
            Expr::Loop(node)           => visitor.visit_loop_expr(node),
            Expr::While(node)          => visitor.visit_while_expr(node),
            Expr::DoWhile(node)        => visitor.visit_do_while_expr(node),
            Expr::For(node)            => visitor.visit_for_expr(node),
            Expr::Match(node)          => visitor.visit_match_expr(node),
            Expr::Break(node)          => visitor.visit_break_expr(node),
            Expr::Continue(node)       => visitor.visit_continue_expr(node),
            Expr::Fallthrough(node)    => visitor.visit_fallthrough_expr(node),
            Expr::Return(node)         => visitor.visit_return_expr(node),
            Expr::Underscore(node)     => visitor.visit_underscore_expr(node),
            Expr::Throw(node)          => visitor.visit_throw_expr(node),
            Expr::Comma(node)          => visitor.visit_comma_expr(node),
            Expr::When(node)           => visitor.visit_when_expr(node),
        }
    }

    pub fn visit_path_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<PathExpr>) {
    }

    pub fn visit_block_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<BlockExpr>) {
        visitor.visit_block(&node.block);
    }

    pub fn visit_prefix_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<PrefixExpr>) {
        visitor.visit_expr(&node.expr);
    }

    pub fn visit_postfix_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<PostfixExpr>) {
        visitor.visit_expr(&node.expr);
    }

    pub fn visit_binary_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<InfixExpr>) {
        visitor.visit_expr(&node.left);
        visitor.visit_expr(&node.right);
    }

    pub fn visit_paren_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<ParenExpr>) {
        visitor.visit_expr(&node.expr);
    }

    pub fn visit_inplace_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<InplaceExpr>) {
        visitor.visit_expr(&node.left);
        visitor.visit_expr(&node.right);
    }

    pub fn visit_type_cast_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<TypeCastExpr>) {
        visitor.visit_expr(&node.expr);
        visitor.visit_type(&node.ty);
    }

    pub fn visit_type_check_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<TypeCheckExpr>) {
        visitor.visit_expr(&node.expr);
        visitor.visit_type(&node.ty);
    }

    pub fn visit_tuple_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<TupleExpr>) {
        for expr in &node.exprs {
            visitor.visit_expr(expr);
        }
    }

    pub fn visit_array_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<ArrayExpr>) {
        match &**node {
            ArrayExpr::Slice { span, node_id, exprs } => {
                for expr in exprs {
                    visitor.visit_expr(expr);
                }
            },
            ArrayExpr::Count { span, node_id, val, count } => {
                visitor.visit_expr(val);
                visitor.visit_expr(count);
            },
        }
    }

    pub fn visit_struct_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<StructExpr>) {
        visitor.visit_expr(&node.path);
        for arg in &node.args {
            match arg {
                StructArg::Expr{ span:_, name: _, expr }  => visitor.visit_expr(expr),
                StructArg::Name{ span:_, name:_ }        => {},
                StructArg::Complete{ span:_, expr } => visitor.visit_expr(expr),
            }
        }
    }

    pub fn visit_index_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<IndexExpr>) {
        visitor.visit_expr(&node.expr);
        visitor.visit_expr(&node.index);
    }

    pub fn visit_tuple_index_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<TupleIndexExpr>) {
        visitor.visit_expr(&node.expr);
    }

    pub fn visit_fn_call_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<FnCallExpr>) {
        visitor.visit_expr(&node.expr);
        for arg in &node.args {
            match arg {
                FnArg::Expr{ span, expr }            => visitor.visit_expr(expr),
                FnArg::Labeled { span, label, expr } => visitor.visit_expr(expr),
            }
        }
    }

    pub fn visit_method_call_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<MethodCallExpr>) {
        visitor.visit_expr(&node.receiver);
        if let Some(gen_args) = &node.gen_args {
            visitor.visit_generic_args(gen_args);
        }
        for arg in &node.args {
            match arg {
                FnArg::Expr{ span, expr }            => visitor.visit_expr(expr),
                FnArg::Labeled { span, label, expr } => visitor.visit_expr(expr),
            }
        }
    }

    pub fn visit_field_access_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<FieldAccessExpr>) {
        visitor.visit_expr(&node.expr);
    }
    
    pub fn visit_closure_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<ClosureExpr>) {
        for param in &node.params {
            visit_fn_param(visitor, param);
        }
        if let Some(ret) = &node.ret {
            visit_fn_return(visitor, ret);
        }
        visitor.visit_expr(&node.body);
    }

    pub fn visit_let_binding_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<LetBindingExpr>) {
        visitor.visit_pattern(&node.pattern);
        visitor.visit_expr(&node.scrutinee);
    }
    
    pub fn visit_if_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<IfExpr>) {
        visitor.visit_expr(&node.cond);
        visitor.visit_block_expr(&node.body);
        if let Some(else_body) = &node.else_body {
            visitor.visit_expr(&else_body);
        }
    }   

    pub fn visit_loop_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<LoopExpr>) {
        visitor.visit_block_expr(&node.body);
    }   

    pub fn visit_while_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<WhileExpr>) {
        visitor.visit_expr(&node.cond);
        if let Some(inc) = &node.inc {
            visitor.visit_expr(&inc);
        }
        visitor.visit_block_expr(&node.body);
        if let Some(else_body) = &node.else_body {
            visitor.visit_block_expr(else_body);
        }
    }

    pub fn visit_do_while_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<DoWhileExpr>) {
        visitor.visit_block_expr(&node.body);
        visitor.visit_expr(&node.cond);
    }

    pub fn visit_for_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<ForExpr>) {
        visitor.visit_pattern(&node.pattern);
        visitor.visit_expr(&node.src);
        visitor.visit_block_expr(&node.body);
        if let Some(else_body) = &node.else_body {
            visitor.visit_block_expr(else_body);
        }
    }

    pub fn visit_match_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<MatchExpr>) {
        visitor.visit_expr(&node.scrutinee);
        for branch in &node.branches {
            visitor.visit_pattern(&branch.pattern);
            if let Some(guard) = &branch.guard {
                visitor.visit_expr(guard);
            }
            visitor.visit_expr(&branch.body);
        }
    }

    pub fn visit_break_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<BreakExpr>) {
        if let Some(expr) = &node.value {
            visitor.visit_expr(expr);
        }
    }   

    pub fn visit_return_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<ReturnExpr>) {
        if let Some(expr) = &node.value {
            visitor.visit_expr(expr);
        }
    }

    pub fn visit_throw_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<ThrowExpr>) {
        visitor.visit_expr(&node.expr);
    }
    
    pub fn visit_comma_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<CommaExpr>) {
        for expr in &node.exprs {
            visitor.visit_expr(expr);
        }
    }

    pub fn visit_when_expr<T: Visitor>(visitor: &mut T, node: &AstNodeRef<WhenExpr>) {
        visitor.visit_expr(&node.cond);
        visitor.visit_block_expr(&node.body);
        if let Some(else_body) = &node.else_body {
            visitor.visit_expr(else_body);
        }
    }


// =============================================================================================================================

    pub fn visit_pattern<T: Visitor>(visitor: &mut T, pattern: &Pattern) {
        match pattern {
            Pattern::Literal(node)     => visitor.visit_literal_pattern(node),
            Pattern::Identifier(node)  => visitor.visit_identifier_pattern(node),
            Pattern::Path(node)        => visitor.visit_path_pattern(node),
            Pattern::Wildcard(node)    => visitor.visit_wildcard_pattern(node),
            Pattern::Rest(node)        => visitor.visit_rest_pattern(node),
            Pattern::Range(node)       => visitor.visit_range_pattern(node),
            Pattern::Reference(node)   => visitor.visit_reference_pattern(node),
            Pattern::Struct(node)      => visitor.visit_struct_pattern(node),
            Pattern::TupleStruct(node) => visitor.visit_tuple_struct_pattern(node),
            Pattern::Tuple(node)       => visitor.visit_tuple_pattern(node),
            Pattern::Grouped(node)     => visitor.visit_grouped_pattern(node),
            Pattern::Slice(node)       => visitor.visit_slice_pattern(node),
            Pattern::EnumMember(node)  => visitor.visit_enum_member_pattern(node),
            Pattern::Alternative(node) => visitor.visit_alternative_pattern(node),
            Pattern::TypeCheck(node)   => visitor.visit_type_check_pattern(node),
        }
    }

    pub fn visit_identifier_pattern<T: Visitor>(visitor: &mut T, node: &AstNodeRef<IdentifierPattern>) {
        if let Some(bound) = &node.bound {
            visitor.visit_pattern(bound);
        }
    }

    pub fn visit_path_pattern<T: Visitor>(visitor: &mut T, node: &AstNodeRef<PathPattern>) {
        visitor.visit_expr_path(&node.path);
    }

    pub fn visit_range_pattern<T: Visitor>(visitor: &mut T, node: &AstNodeRef<RangePattern>) {
        match &**node {
            RangePattern::Exclusive { span, node_id, begin, end } => {
                visitor.visit_pattern(begin);
                visitor.visit_pattern(end);
            },
            RangePattern::Inclusive { span, node_id, begin, end } => {
                visitor.visit_pattern(begin);
                visitor.visit_pattern(end);
            },
            RangePattern::From { span, node_id, begin } => {
                visitor.visit_pattern(begin);
            },
            RangePattern::To { span, node_id, end } => {
                visitor.visit_pattern(end);
            },
            RangePattern::InclusiveTo { span, node_id, end } => {
                visitor.visit_pattern(end);
            },
        }
    }

    pub fn visit_reference_pattern<T: Visitor>(visitor: &mut T, node: &AstNodeRef<ReferencePattern>) {
        visitor.visit_pattern(&node.pattern);
    }

    pub fn visit_struct_pattern<T: Visitor>(visitor: &mut T, node: &AstNodeRef<StructPattern>) {
        match &**node {
            StructPattern::Inferred { span, node_id, fields } => for field in fields {
                match field {
                    StructPatternField::Named { span, name, pattern }              => visitor.visit_pattern(pattern),
                    StructPatternField::TupleIndex { span, idx, pattern }          => visitor.visit_pattern(pattern),
                    StructPatternField::Iden { span, is_ref, is_mut, iden, bound } => if let Some(bound) = bound {
                        visitor.visit_pattern(bound);
                    },
                    StructPatternField::Rest                                 => {},
                }
            },
            StructPattern::Path { span, node_id, path, fields } => {
                visitor.visit_expr_path(path);
                for field in fields {
                    match field {
                        StructPatternField::Named { span, name, pattern }              => visitor.visit_pattern(pattern),
                        StructPatternField::TupleIndex { span, idx, pattern }          => visitor.visit_pattern(pattern),
                        StructPatternField::Iden { span, is_ref, is_mut, iden, bound } => if let Some(bound) = bound {
                            visitor.visit_pattern(bound);
                        },
                        StructPatternField::Rest                                 => {},
                    }
                }
            },
        }
    }

    pub fn visit_tuple_struct_pattern<T: Visitor>(visitor: &mut T, node: &AstNodeRef<TupleStructPattern>) {
        match &**node {
            TupleStructPattern::Named { span, node_id, path, patterns } => {
                visitor.visit_expr_path(path);
                for pattern in patterns {
                    visitor.visit_pattern(pattern);
                }
            },
            TupleStructPattern::Inferred { span, node_id, patterns } => for pattern in patterns {
                visitor.visit_pattern(pattern);
            },
        }
    }

    pub fn visit_tuple_pattern<T: Visitor>(visitor: &mut T, node: &AstNodeRef<TuplePattern>) {
        for pattern in &node.patterns {
            visitor.visit_pattern(pattern);
        }
    }

    pub fn visit_grouped_pattern<T: Visitor>(visitor: &mut T, node: &AstNodeRef<GroupedPattern>) {
        visitor.visit_pattern(&node.pattern);
    }

    pub fn visit_slice_pattern<T: Visitor>(visitor: &mut T, node: &AstNodeRef<SlicePattern>) {
        for pattern in &node.patterns {
            visitor.visit_pattern(pattern);
        }
    }

    pub fn visit_alternative_pattern<T: Visitor>(visitor: &mut T, node: &AstNodeRef<AlternativePattern>) {
        for pattern in &node.patterns {
            visitor.visit_pattern(pattern);
        }
    }

    pub fn visit_type_check_pattern<T: Visitor>(visitor: &mut T, node: &AstNodeRef<TypeCheckPattern>) {
        visitor.visit_type(&node.ty);
    }

// =============================================================================================================================

    pub fn visit_type<T: Visitor>(visitor: &mut T, node: &Type) {
        match node {
            Type::Paren(ty)       => visitor.visit_paren_type(ty),
            Type::Primitive(ty)   => visitor.visit_primitive_type(ty),
            Type::Unit(ty)        => visitor.visit_unit_type(ty),
            Type::Never(ty)       => visitor.visit_never_type(ty),
            Type::Path(ty)        => visitor.visit_path_type(ty),
            Type::Tuple(ty)       => visitor.visit_tuple_type(ty),
            Type::Array(ty)       => visitor.visit_array_type(ty),
            Type::Slice(ty)       => visitor.visit_slice_type(ty),
            Type::StringSlice(ty) => visitor.visit_string_slice_type(ty),
            Type::Pointer(ty)     => visitor.visit_pointer_type(ty),
            Type::Ref(ty)         => visitor.visit_reference_type(ty),
            Type::Optional(ty)    => visitor.visit_optional_type(ty),
            Type::Fn(ty)          => visitor.visit_fn_type(ty),
            Type::Record(ty)      => visitor.visit_record_type(ty),
            Type::EnumRecord(ty)  => visitor.visit_enum_record_type(ty),
        }
    }

    pub fn visit_paren_type<T: Visitor>(visitor: &mut T, node: &AstNodeRef<ParenthesizedType>) {
        visitor.visit_type(&node.ty);
    }

    pub fn visit_path_type<T: Visitor>(visitor: &mut T, node: &AstNodeRef<PathType>) {
        visitor.visit_type_path(&node.path);
    }

    pub fn visit_tuple_type<T: Visitor>(visitor: &mut T, node: &AstNodeRef<TupleType>) {
        for ty in &node.types {
            visitor.visit_type(ty);
        }
    }

    pub fn visit_array_type<T: Visitor>(visitor: &mut T, node: &AstNodeRef<ArrayType>) {
        visitor.visit_expr(&node.size);
        if let Some(sentinel) = &node.sentinel {
            visitor.visit_expr(sentinel);
        }
        visitor.visit_type(&node.ty);
    }

    pub fn visit_slice_type<T: Visitor>(visitor: &mut T, node: &AstNodeRef<SliceType>) {
        if let Some(sentinel) = &node.sentinel {
            visitor.visit_expr(sentinel);
        }
        visitor.visit_type(&node.ty);
    }

    pub fn visit_pointer_type<T: Visitor>(visitor: &mut T, node: &AstNodeRef<PointerType>) {
        if let Some(sentinel) = &node.sentinel {
            visitor.visit_expr(sentinel);
        }
        visitor.visit_type(&node.ty);
    }

    pub fn visit_reference_type<T: Visitor>(visitor: &mut T, node: &AstNodeRef<ReferenceType>) {
        visitor.visit_type(&node.ty);
    }

    pub fn visit_optional_type<T: Visitor>(visitor: &mut T, node: &AstNodeRef<OptionalType>) {
        visitor.visit_type(&node.ty);
    }

    pub fn visit_fn_type<T: Visitor>(visitor: &mut T, node: &AstNodeRef<FnType>) {
        for (_, ty) in &node.params {
            visitor.visit_type(ty);
        }
        if let Some(ret_ty) = &node.return_ty {
            visitor.visit_type(ret_ty);
        }
    }

    pub fn visit_record_type<T: Visitor>(visitor: &mut T, node: &AstNodeRef<RecordType>) {
        for field in &node.fields {
            visit_reg_struct_field(visitor, field);
        }
    }

    pub fn visit_enum_record_type<T: Visitor>(visitor: &mut T, node: &AstNodeRef<EnumRecordType>) {
        for variant in &node.variants {
            visit_enum_variant(visitor, variant);
        }
    }

// =============================================================================================================================

    pub fn visit_visibility<T: Visitor>(visitor: &mut T, node: &AstNodeRef<Visibility>) {
        match &**node {
            Visibility::Path{ span:_, node_id, path } => visitor.visit_simple_path(path),
            _                                         => {},
        }
    }

    pub fn visit_attribute<T: Visitor>(visitor: &mut T, node: &AstNodeRef<Attribute>) {
        visitor.visit_simple_path(&node.path);
        for meta in &node.metas {
            visit_attribute_meta(visitor, meta);
        }
    }

    pub fn visit_attribute_meta<T: Visitor>(visitor: &mut T, meta: &AttribMeta) {
        match meta {
            AttribMeta::Simple { path }             => visitor.visit_simple_path(path),
            AttribMeta::Expr { expr }               => visitor.visit_expr(expr),
            AttribMeta::Assign { span, path, expr } => {
                visitor.visit_simple_path(path);
                visitor.visit_expr(expr);
            },
            AttribMeta::Meta { span, path, metas }  => {
                visitor.visit_simple_path(path);
                for meta in metas {
                    visit_attribute_meta(visitor, meta);
                }
            },
        }
    }

// =============================================================================================================================

    pub fn visit_contract<T: Visitor>(visitor: &mut T, node: &AstNodeRef<Contract>) {

    }

// =============================================================================================================================

    pub fn visit_generic_params<T: Visitor>(visitor: &mut T, node: &AstNodeRef<GenericParams>) {
        for param in &node.params {
            match param {
                GenericParam::Type(param) => visitor.visit_generic_type_param(param),
                GenericParam::TypeSpec(param) => visitor.visit_generic_type_spec(param),
                GenericParam::Const(param) => visitor.visit_generic_const_param(param),
                GenericParam::ConstSpec(param) => visitor.visit_generic_const_spec(param),
                GenericParam::Pack(param) => visitor.visit_generic_param_pack(param),
            }
        }
    }

    pub fn visit_generic_type_param<T: Visitor>(visitor: &mut T, node: &AstNodeRef<GenericTypeParam>) {
        for bound in &node.bounds {
            match bound {
                GenericTypeBound::Type(path) => visitor.visit_type_path(path),
            }
        }
        if let Some(def) = &node.def {
            visitor.visit_type(def);
        }
    }

    pub fn visit_generic_type_spec<T: Visitor>(visitor: &mut T, node: &AstNodeRef<GenericTypeSpec>) {
        visitor.visit_type(&node.ty);
    }

    pub fn visit_generic_const_param<T: Visitor>(visitor: &mut T, node: &AstNodeRef<GenericConstParam>) {
        visitor.visit_type(&node.ty);
        if let Some(def) = &node.def {
            visitor.visit_expr(def);
        }
    }

    pub fn visit_generic_const_spec<T: Visitor>(visitor: &mut T, node: &AstNodeRef<GenericConstSpec>) {
        visitor.visit_block_expr(&node.expr);
    }

    pub fn visit_generic_param_pack<T: Visitor>(visitor: &mut T, node: &AstNodeRef<GenericParamPack>) {
        for desc in &node.descs {
            match desc {
                GenericParamPackDesc::Type(_) => (),
                GenericParamPackDesc::TypeBounds(_, bounds) => for bound in bounds {
                    match bound {
                        GenericTypeBound::Type(path) => visitor.visit_type_path(path),
                    }
                },
                GenericParamPackDesc::Expr(ty) => visitor.visit_type(ty),
            }
        }
    }

    pub fn visit_generic_args<T: Visitor>(visitor: &mut T, node: &AstNodeRef<GenericArgs>) {
        for arg in &node.args {
            match arg {
                GenericArg::Type(ty) => visitor.visit_type(ty),
                GenericArg::Value(expr) => visitor.visit_block_expr(expr),
                GenericArg::TypeOrValue(_) => (),
            }
        }
    }

    pub fn visit_where_clause<T: Visitor>(visitor: &mut T, node: &AstNodeRef<WhereClause>) {
        for bound in &node.bounds {
            match bound {
                WhereBound::Type { ty, bounds, .. } => {
                    visitor.visit_type(ty);
                    for bound in bounds {
                        match bound {
                            GenericTypeBound::Type(path) => visitor.visit_type_path(path),
                        }
                    }
                },
                WhereBound::ExplicitType { ty, bounds, .. } => {
                    visitor.visit_type(ty);
                    for bound in bounds {
                        visitor.visit_type(bound);
                    }
                },
                WhereBound::Value { bound } => visitor.visit_block_expr(bound),
            }
        }
    }

    pub fn visit_trait_bounds<T: Visitor>(visitor: &mut T, node: &AstNodeRef<TraitBounds>) {
        for bound in &node.bounds {
            visitor.visit_type_path(bound);
        }
    }
}