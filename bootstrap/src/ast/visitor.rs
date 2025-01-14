use super::*;




pub trait Visitor {

    fn visit(&mut self, ast: &Ast) where Self: Sized {
        for item in &ast.items {
            self.visit_item(ast, item);
        }
    }

    fn visit_simple_path(&mut self, ast: &Ast, node_id: AstNodeRef<SimplePath>) where Self: Sized {
    }

    fn visit_expr_path(&mut self, ast: &Ast, node_id: AstNodeRef<ExprPath>) where Self: Sized {
        helpers::visit_expr_path(self, ast, node_id)
    }

    fn visit_type_path(&mut self, ast: &Ast, node_id: AstNodeRef<TypePath>) where Self: Sized {
        helpers::visit_type_path(self, ast, node_id)
    }

    fn visit_qualified_path(&mut self, ast: &Ast, node_id: AstNodeRef<QualifiedPath>) where Self: Sized {
        helpers::visit_qualified_path(self, ast, node_id)
    }

// =============================================================================================================================

    fn visit_item(&mut self, ast: &Ast, item: &Item) where Self: Sized {
        helpers::visit_item(self, ast, item);
    }

    fn visit_trait_item(&mut self, ast: &Ast, item: &TraitItem) where Self: Sized {
        helpers::visit_trait_item(self, ast, item);
    }

    fn visit_assoc_item(&mut self, ast: &Ast, item: &AssocItem) where Self: Sized {
        helpers::visit_assoc_item(self, ast, item);
    }

    fn visit_extern_item(&mut self, ast: &Ast, item: &ExternItem) where Self: Sized {
        helpers::visit_extern_item(self, ast, item);
    }

    fn visit_module(&mut self, ast: &Ast, node_id: AstNodeRef<ModuleItem>) where Self: Sized {
        helpers::visit_module(self, ast, node_id);
    }

    fn visit_use(&mut self, ast: &Ast, node_id: AstNodeRef<UseItem>) where Self: Sized {
        helpers::visit_use(self, ast, node_id);
    }

    fn visit_use_path(&mut self, ast: &Ast, node_id: AstNodeRef<UsePath>) where Self: Sized {
        helpers::visit_use_path(self, ast, node_id);
    }

    fn visit_function(&mut self, ast: &Ast, node_id: AstNodeRef<Function>) where Self: Sized {
        helpers::visit_function(self, ast, node_id);
    }

    fn visit_type_alias(&mut self, ast: &Ast, node_id: AstNodeRef<TypeAlias>) where Self: Sized {
        helpers::visit_type_alias(self, ast, node_id);
    }

    fn visit_struct(&mut self, ast: &Ast, node_id: AstNodeRef<Struct>) where Self: Sized {
        helpers::visit_struct(self, ast, node_id);
    }

    fn visit_reg_struct_field(&mut self, ast: &Ast, field: &RegStructField) where Self: Sized {
        helpers::visit_reg_struct_field(self, ast, field);
    }

    fn visit_tuple_struct_field(&mut self, ast: &Ast, field: &TupleStructField) where Self: Sized {
        helpers::visit_tuple_struct_field(self, ast, field);
    }

    fn visit_union(&mut self, ast: &Ast, node_id: AstNodeRef<Union>) where Self: Sized {
        helpers::visit_union(self, ast, node_id);
    }

    fn visit_enum(&mut self, ast: &Ast, node_id: AstNodeRef<Enum>) where Self: Sized {
        helpers::visit_enum(self, ast, node_id);
    }

    fn visit_enum_variant(&mut self, ast: &Ast, variant: &EnumVariant) where Self: Sized {
        helpers::visit_enum_variant(self, ast, variant);
    }

    fn visit_bitfield(&mut self, ast: &Ast, node_id: AstNodeRef<Bitfield>) where Self: Sized {
        helpers::visit_bitfield(self, ast, node_id);
    }
    
    fn visit_bitfield_field(&mut self, ast: &Ast, field: &BitfieldField) where Self: Sized {
        helpers::visit_bitfield_field(self, ast, field);
    }
    
    fn visit_const(&mut self, ast: &Ast, node_id: AstNodeRef<Const>) where Self: Sized {
        helpers::visit_const(self, ast, node_id);
    }
    
    fn visit_static(&mut self, ast: &Ast, node_id: AstNodeRef<Static>) where Self: Sized {
        helpers::visit_static(self, ast, node_id);
    }
    
    fn visit_property(&mut self, ast: &Ast, node_id: AstNodeRef<Property>) where Self: Sized {
        helpers::visit_property(self, ast, node_id);
    }
    
    fn visit_trait(&mut self, ast: &Ast, node_id: AstNodeRef<Trait>) where Self: Sized {
        helpers::visit_trait(self, ast, node_id);
    }
    
    fn visit_impl(&mut self, ast: &Ast, node_id: AstNodeRef<Impl>) where Self: Sized {
        helpers::visit_impl(self, ast, node_id);
    }
    
    fn visit_extern_block(&mut self, ast: &Ast, node_id: AstNodeRef<ExternBlock>) where Self: Sized {
        helpers::visit_extern_block(self, ast, node_id);
    }
    
    fn visit_op_trait(&mut self, ast: &Ast, node_id: AstNodeRef<OpTrait>) where Self: Sized {
        helpers::visit_op_trait(self, ast, node_id);
    }
    
    fn visit_op_use(&mut self, ast: &Ast, node_id: AstNodeRef<OpUse>) where Self: Sized {
    }

    fn visit_precedence(&mut self, ast: &Ast, node_id: AstNodeRef<Precedence>) where Self: Sized {
        helpers::visit_precedence(self, ast, node_id);
    }
    
    fn visit_precedence_use(&mut self, ast: &Ast, node_id: AstNodeRef<PrecedenceUse>) where Self: Sized {
    }
    
// =============================================================================================================================

    fn visit_block(&mut self, ast: &Ast, node_id: AstNodeRef<Block>) where Self: Sized {
        helpers::visit_block(self, ast, node_id);
    }

// =============================================================================================================================

    fn visit_stmt(&mut self, ast: &Ast, node: &Stmt) where Self: Sized {
        helpers::visit_stmt(self, ast, node);
    }

    fn visit_var_decl(&mut self, ast: &Ast, node_id: AstNodeRef<VarDecl>) where Self: Sized {
        helpers::visit_var_decl(self, ast, node_id);
    }

    fn visit_defer(&mut self, ast: &Ast, node_id: AstNodeRef<Defer>) where Self: Sized {
        helpers::visit_defer(self, ast, node_id);
    }

    fn visit_err_defer(&mut self, ast: &Ast, node_id: AstNodeRef<ErrDefer>) where Self: Sized {
        helpers::visit_err_defer(self, ast, node_id);
    }

    fn visit_expr_stmt(&mut self, ast: &Ast, node_id: AstNodeRef<ExprStmt>) where Self: Sized {
        helpers::visit_expr_stmt(self, ast, node_id);
    }

// =============================================================================================================================

    fn visit_expr(&mut self, ast: &Ast, node: &Expr) where Self: Sized {
        helpers::visit_expr(self, ast, node);
    }

    fn visit_literal_expr(&mut self, ast: &Ast, node_id: AstNodeRef<LiteralExpr>) where Self: Sized {
        
    }

    fn visit_path_expr(&mut self, ast: &Ast, node_id: AstNodeRef<PathExpr>) where Self: Sized {
        helpers::visit_path_expr(self, ast, node_id);
    }

    fn visit_unit_expr(&mut self, ast: &Ast) where Self: Sized {
    }

    fn visit_block_expr(&mut self, ast: &Ast, node_id: AstNodeRef<BlockExpr>) where Self: Sized {
        helpers::visit_block_expr(self, ast, node_id);
    }

    fn visit_prefix_expr(&mut self, ast: &Ast, node_id: AstNodeRef<PrefixExpr>) where Self: Sized {
        helpers::visit_prefix_expr(self, ast, node_id);
    }

    fn visit_postfix_expr(&mut self, ast: &Ast, node_id: AstNodeRef<PostfixExpr>) where Self: Sized {
        helpers::visit_postfix_expr(self, ast, node_id);
    }

    fn visit_binary_expr(&mut self, ast: &Ast, node_id: AstNodeRef<InfixExpr>) where Self: Sized {
        helpers::visit_binary_expr(self, ast, node_id);
    }

    fn visit_paren_expr(&mut self, ast: &Ast, node_id: AstNodeRef<ParenExpr>) where Self: Sized {
        helpers::visit_paren_expr(self, ast, node_id);
    }

    fn visit_inplace_expr(&mut self, ast: &Ast, node_id: AstNodeRef<InplaceExpr>) where Self: Sized {
        helpers::visit_inplace_expr(self, ast, node_id);
    }

    fn visit_type_cast_expr(&mut self, ast: &Ast, node_id: AstNodeRef<TypeCastExpr>) where Self: Sized {
        helpers::visit_type_cast_expr(self, ast, node_id);
    }

    fn visit_type_check_expr(&mut self, ast: &Ast, node_id: AstNodeRef<TypeCheckExpr>) where Self: Sized {
        helpers::visit_type_check_expr(self, ast, node_id);
    }

    fn visit_tuple_expr(&mut self, ast: &Ast, node_id: AstNodeRef<TupleExpr>) where Self: Sized {
        helpers::visit_tuple_expr(self, ast, node_id);
    }

    fn visit_array_expr(&mut self, ast: &Ast, node_id: AstNodeRef<ArrayExpr>) where Self: Sized {
        helpers::visit_array_expr(self, ast, node_id);
    }

    fn visit_struct_expr(&mut self, ast: &Ast, node_id: AstNodeRef<StructExpr>) where Self: Sized {
        helpers::visit_struct_expr(self, ast, node_id);
    }

    fn visit_index_expr(&mut self, ast: &Ast, node_id: AstNodeRef<IndexExpr>) where Self: Sized {
        helpers::visit_index_expr(self, ast, node_id);
    }

    fn visit_tuple_index_expr(&mut self, ast: &Ast, node_id: AstNodeRef<TupleIndexExpr>) where Self: Sized {
        helpers::visit_tuple_index_expr(self, ast, node_id);
    }

    fn visit_fn_call_expr(&mut self, ast: &Ast, node_id: AstNodeRef<FnCallExpr>) where Self: Sized {
        helpers::visit_fn_call_expr(self, ast, node_id);
    }

    fn visit_method_call_expr(&mut self, ast: &Ast, node_id: AstNodeRef<MethodCallExpr>) where Self: Sized {
        helpers::visit_method_call_expr(self, ast, node_id);
    }

    fn visit_field_access_expr(&mut self, ast: &Ast, node_id: AstNodeRef<FieldAccessExpr>) where Self: Sized {
        helpers::visit_field_access_expr(self, ast, node_id);
    }

    fn visit_closure_expr(&mut self, ast: &Ast, node_id: AstNodeRef<ClosureExpr>) where Self: Sized {
        helpers::visit_closure_expr(self, ast, node_id);
    }

    fn visit_full_range_expr(&mut self, ast: &Ast) where Self: Sized {
    }

    fn visit_let_binding_expr(&mut self, ast: &Ast, node_id: AstNodeRef<LetBindingExpr>) where Self: Sized {
        helpers::visit_let_binding_expr(self, ast, node_id);
    }

    fn visit_if_expr(&mut self, ast: &Ast, node_id: AstNodeRef<IfExpr>) where Self: Sized {
        helpers::visit_if_expr(self, ast, node_id);
    }

    fn visit_loop_expr(&mut self, ast: &Ast, node_id: AstNodeRef<LoopExpr>) where Self: Sized {
        helpers::visit_loop_expr(self, ast, node_id);
    }

    fn visit_while_expr(&mut self, ast: &Ast, node_id: AstNodeRef<WhileExpr>) where Self: Sized {
        helpers::visit_while_expr(self, ast, node_id);
    }

    fn visit_do_while_expr(&mut self, ast: &Ast, node_id: AstNodeRef<DoWhileExpr>) where Self: Sized {
        helpers::visit_do_while_expr(self, ast, node_id);
    }

    fn visit_for_expr(&mut self, ast: &Ast, node_id: AstNodeRef<ForExpr>) where Self: Sized {
        helpers::visit_for_expr(self, ast, node_id);
    }

    fn visit_match_expr(&mut self, ast: &Ast, node_id: AstNodeRef<MatchExpr>) where Self: Sized {
        helpers::visit_match_expr(self, ast, node_id);
    }

    fn visit_break_expr(&mut self, ast: &Ast, node_id: AstNodeRef<BreakExpr>) where Self: Sized {
        helpers::visit_break_expr(self, ast, node_id);
    }

    fn visit_continue_expr(&mut self, ast: &Ast, node_id: AstNodeRef<ContinueExpr>) where Self: Sized {
        
    }

    fn visit_fallthrough_expr(&mut self, ast: &Ast, node_id: AstNodeRef<FallthroughExpr>) where Self: Sized {
        
    }

    fn visit_return_expr(&mut self, ast: &Ast, node_id: AstNodeRef<ReturnExpr>) where Self: Sized {
        helpers::visit_return_expr(self, ast, node_id);
    }

    fn visit_underscore_expr(&mut self, ast: &Ast) where Self: Sized {
    }

    fn visit_throw_expr(&mut self, ast: &Ast, node_id: AstNodeRef<ThrowExpr>) where Self: Sized {
        helpers::visit_throw_expr(self, ast, node_id);
    }

    fn visit_comma_expr(&mut self, ast: &Ast, node_id: AstNodeRef<CommaExpr>) where Self: Sized {
        helpers::visit_comma_expr(self, ast, node_id);
    }

    fn visit_when_expr(&mut self, ast: &Ast, node_id: AstNodeRef<WhenExpr>) where Self: Sized {
        helpers::visit_when_expr(self, ast, node_id);
    }

// =============================================================================================================================

    fn visit_pattern(&mut self, ast: &Ast, node: &Pattern) where Self: Sized {
        helpers::visit_pattern(self, ast, node);
    }

    fn visit_literal_pattern(&mut self, ast: &Ast, node_id: AstNodeRef<LiteralPattern>) where Self: Sized {
    }

    fn visit_identifier_pattern(&mut self, ast: &Ast, node_id: AstNodeRef<IdentifierPattern>) where Self: Sized {
        helpers::visit_identifier_pattern(self, ast, node_id);
    }

    fn visit_path_pattern(&mut self, ast: &Ast, node_id: AstNodeRef<PathPattern>) where Self: Sized {
        helpers::visit_path_pattern(self, ast, node_id);
    }

    fn visit_wildcard_pattern(&mut self, ast: &Ast) where Self: Sized {
    }

    fn visit_rest_pattern(&mut self, ast: &Ast) where Self: Sized {
    }

    fn visit_range_pattern(&mut self, ast: &Ast, node_id: AstNodeRef<RangePattern>) where Self: Sized {
        helpers::visit_range_pattern(self, ast, node_id);
    }

    fn visit_reference_pattern(&mut self, ast: &Ast, node_id: AstNodeRef<ReferencePattern>) where Self: Sized {
        helpers::visit_reference_pattern(self, ast, node_id);
    }

    fn visit_struct_pattern(&mut self, ast: &Ast, node_id: AstNodeRef<StructPattern>) where Self: Sized {
        helpers::visit_struct_pattern(self, ast, node_id);
    }

    fn visit_tuple_struct_pattern(&mut self, ast: &Ast, node_id: AstNodeRef<TupleStructPattern>) where Self: Sized {
        helpers::visit_tuple_struct_pattern(self, ast, node_id);
    }

    fn visit_tuple_pattern(&mut self, ast: &Ast, node_id: AstNodeRef<TuplePattern>) where Self: Sized {
        helpers::visit_tuple_pattern(self, ast, node_id);
    }

    fn visit_grouped_pattern(&mut self, ast: &Ast, node_id: AstNodeRef<GroupedPattern>) where Self: Sized {
        helpers::visit_grouped_pattern(self, ast, node_id);
    }

    fn visit_slice_pattern(&mut self, ast: &Ast, node_id: AstNodeRef<SlicePattern>) where Self: Sized {
        helpers::visit_slice_pattern(self, ast, node_id);
    }

    fn visit_enum_member_pattern(&mut self, ast: &Ast, node_id: AstNodeRef<EnumMemberPattern>) where Self: Sized {
    }

    fn visit_alternative_pattern(&mut self, ast: &Ast, node_id: AstNodeRef<AlternativePattern>) where Self: Sized {
        helpers::visit_alternative_pattern(self, ast, node_id);
    }

    fn visit_type_check_pattern(&mut self, ast: &Ast, node_id: AstNodeRef<TypeCheckPattern>) where Self: Sized {
        helpers::visit_type_check_pattern(self, ast, node_id);
    }

// =============================================================================================================================

    fn visit_type(&mut self, ast: &Ast, node: &Type) where Self: Sized {
        helpers::visit_type(self, ast, node);
    }

    fn visit_paren_type(&mut self, ast: &Ast, node_id: AstNodeRef<ParenthesizedType>) where Self: Sized {
        helpers::visit_paren_type(self, ast, node_id);
    }

    fn visit_primitive_type(&mut self, ast: &Ast, node_id: AstNodeRef<PrimitiveType>) where Self: Sized {
        
    }

    fn visit_unit_type(&mut self, ast: &Ast) where Self: Sized {
    }

    fn visit_never_type(&mut self, ast: &Ast) where Self: Sized {
    }

    fn visit_path_type(&mut self, ast: &Ast, node_id: AstNodeRef<PathType>) where Self: Sized {
        helpers::visit_path_type(self, ast, node_id);
    }

    fn visit_tuple_type(&mut self, ast: &Ast, node_id: AstNodeRef<TupleType>) where Self: Sized {
        helpers::visit_tuple_type(self, ast, node_id);
    }

    fn visit_array_type(&mut self, ast: &Ast, node_id: AstNodeRef<ArrayType>) where Self: Sized {
        helpers::visit_array_type(self, ast, node_id);
    }

    fn visit_slice_type(&mut self, ast: &Ast, node_id: AstNodeRef<SliceType>) where Self: Sized {
        helpers::visit_slice_type(self, ast, node_id);
    }

    fn visit_string_slice_type(&mut self, ast: &Ast, node_id: AstNodeRef<StringSliceType>) where Self: Sized {
    }

    fn visit_pointer_type(&mut self, ast: &Ast, node_id: AstNodeRef<PointerType>) where Self: Sized {
        helpers::visit_pointer_type(self, ast, node_id);
    }

    fn visit_reference_type(&mut self, ast: &Ast, node_id: AstNodeRef<ReferenceType>) where Self: Sized {
        helpers::visit_reference_type(self, ast, node_id);
    }

    fn visit_optional_type(&mut self, ast: &Ast, node_id: AstNodeRef<OptionalType>) where Self: Sized {
        helpers::visit_optional_type(self, ast, node_id);
    }

    fn visit_fn_type(&mut self, ast: &Ast, node_id: AstNodeRef<FnType>) where Self: Sized {
        helpers::visit_fn_type(self, ast, node_id);
    }

    fn visit_record_type(&mut self, ast: &Ast, node_id: AstNodeRef<RecordType>) where Self: Sized {
        helpers::visit_record_type(self, ast, node_id);
    }

    fn visit_enum_record_type(&mut self, ast: &Ast, node_id: AstNodeRef<EnumRecordType>) where Self: Sized {
        helpers::visit_enum_record_type(self, ast, node_id);
    }

// =============================================================================================================================

    fn visit_visibility(&mut self, ast: &Ast, node_id: AstNodeRef<Visibility>) where Self: Sized {
        helpers::visit_visibility(self, ast, node_id);
    }

    fn visit_attribute(&mut self, ast: &Ast, node_id: AstNodeRef<Attribute>) where Self: Sized {
        helpers::visit_attribute(self, ast, node_id);
    }

// =============================================================================================================================

    fn visit_contract(&mut self, ast: &Ast, node_id: AstNodeRef<Contract>) where Self: Sized {
        helpers::visit_contract(self, ast, node_id);
    }

// =============================================================================================================================

    fn visit_generic_params(&mut self, ast: &Ast, node_id: AstNodeRef<GenericParams>) where Self: Sized {
        helpers::visit_generic_params(self, ast, node_id);
    }

    fn visit_generic_args(&mut self, ast: &Ast, node_id: AstNodeRef<GenericArgs>) where Self: Sized {
        helpers::visit_generic_args(self, ast, node_id);
    }

    fn visit_where_clause(&mut self, ast: &Ast, node_id: AstNodeRef<WhereClause>) where Self: Sized {
        helpers::visit_where_clause(self, ast, node_id);
    }

    fn visit_trait_bounds(&mut self, ast: &Ast, node_id: AstNodeRef<TraitBounds>) where Self: Sized {
        helpers::visit_trait_bounds(self, ast, node_id);
    }

}

pub mod helpers {
    use super::*;

    pub fn visit<T: Visitor>(visitor: &mut T, ast: &Ast) {
        for item in &ast.items {
            visitor.visit_item(ast, item);
        }
    }

    pub fn visit_expr_path<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<ExprPath>) {
        let node = &ast[node_id];
        for iden in &node.idens {
            if let Some(gen_args) = iden.gen_args {
                visitor.visit_generic_args(ast, gen_args)
            }
        }
    }

    pub fn visit_type_path<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<TypePath>) {
        let node = &ast[node_id];
        for iden in &node.idens {
            match iden {
                TypePathIdentifier::Plain { name }            => {},
                TypePathIdentifier::GenArg { name, gen_args } => visitor.visit_generic_args(ast, *gen_args),
                TypePathIdentifier::Fn { name, params, ret }  => {
                    for param_ty in params {
                        visitor.visit_type(ast, param_ty);
                    }
                    if let Some(ret) = ret {
                        visitor.visit_type(ast, ret);
                    }
                },
            }
        }
    }

    pub fn visit_qualified_path<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<QualifiedPath>) {
        let node = &ast[node_id];
        visitor.visit_type(ast, &node.ty);
        if let Some(bound) = &node.bound {
            if let Some(gen_args) = bound.gen_args {
                visitor.visit_generic_args(ast, gen_args);
            }
        }
        for iden in &node.sub_path {
            if let Some(gen_args) = iden.gen_args {
                visitor.visit_generic_args(ast, gen_args)
            }
        }
    }

// =============================================================================================================================

    pub fn visit_item<T: Visitor>(visitor: &mut T, ast: &Ast, item: &Item) {
        match item {
            Item::Module(node_id)        => visitor.visit_module(ast, *node_id),
            Item::Use(node_id)           => visitor.visit_use(ast, *node_id),
            Item::Function(node_id)      => visitor.visit_function(ast, *node_id),
            Item::TypeAlias(node_id)     => visitor.visit_type_alias(ast, *node_id),
            Item::Struct(node_id)        => visitor.visit_struct(ast, *node_id),
            Item::Union(node_id)         => visitor.visit_union(ast, *node_id),
            Item::Enum(node_id)          => visitor.visit_enum(ast, *node_id),
            Item::Bitfield(node_id)      => visitor.visit_bitfield(ast, *node_id),
            Item::Const(node_id)         => visitor.visit_const(ast, *node_id),
            Item::Static(node_id)        => visitor.visit_static(ast, *node_id),
            Item::Property(node_id)      => visitor.visit_property(ast, *node_id),
            Item::Trait(node_id)         => visitor.visit_trait(ast, *node_id),
            Item::Impl(node_id)          => visitor.visit_impl(ast, *node_id),
            Item::Extern(node_id)        => visitor.visit_extern_block(ast, *node_id),
            Item::OpTrait(node_id)       => visitor.visit_op_trait(ast, *node_id),
            Item::OpUse(node_id)         => visitor.visit_op_use(ast, *node_id),
            Item::Precedence(node_id)    => visitor.visit_precedence(ast, *node_id),
            Item::PrecedenceUse(node_id) => visitor.visit_precedence_use(ast, *node_id),
        }
    }

    pub fn visit_trait_item<T: Visitor>(visitor: &mut T, ast: &Ast, item: &TraitItem) {
        match item {
            TraitItem::Function(node_id)  => visitor.visit_function(ast, *node_id),
            TraitItem::TypeAlias(node_id) => visitor.visit_type_alias(ast, *node_id),
            TraitItem::Const(node_id)     => visitor.visit_const(ast, *node_id),
            TraitItem::Property(node_id)  => visitor.visit_property(ast, *node_id),
        }
    }

    pub fn visit_assoc_item<T: Visitor>(visitor: &mut T, ast: &Ast, item: &AssocItem) {
        match item {
            AssocItem::Function(node_id)  => visitor.visit_function(ast, *node_id),
            AssocItem::TypeAlias(node_id) => visitor.visit_type_alias(ast, *node_id),
            AssocItem::Const(node_id)     => visitor.visit_const(ast, *node_id),
            AssocItem::Static(node_id)    => visitor.visit_static(ast, *node_id),
            AssocItem::Property(node_id)  => visitor.visit_property(ast, *node_id),
        }
    }

    pub fn visit_extern_item<T: Visitor>(visitor: &mut T, ast: &Ast, item: &ExternItem) {
        match item {
            ExternItem::Function(node_id) => visitor.visit_function(ast, *node_id),
            ExternItem::Static(node_id)   => visitor.visit_static(ast, *node_id),
        }
    }

    pub fn visit_module<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<ModuleItem>) {
        let node = &ast[node_id];
        for attr in &node.attrs {
            visitor.visit_attribute(ast, *attr);
        }

        if let Some(vis) = node.vis {
            visitor.visit_visibility(ast, vis);
        }
        if let Some(block) = node.block {
            visitor.visit_block(ast, block);
        }
    }

    pub fn visit_use<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<UseItem>) {
        let node = &ast[node_id];
        for attr in &node.attrs {
            visitor.visit_attribute(ast, *attr);
        }
        if let Some(vis) = node.vis {
            visitor.visit_visibility(ast, vis);
        }
        visitor.visit_use_path(ast, node.path);
    }

    pub fn visit_use_path<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<UsePath>) {
        let node = &ast[node_id];
        match node {
            UsePath::SelfPath { alias } => {},
            UsePath::SubPaths { segments, sub_paths } => for path in sub_paths {
                visitor.visit_use_path(ast, *path);
            },
            UsePath::Alias { segments, alias } => {},
        }
    }

    pub fn visit_function<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<Function>) {
        let node = &ast[node_id];
        for attr in &node.attrs {
            visitor.visit_attribute(ast, *attr);
        }
        if let Some(vis) = node.vis {
            visitor.visit_visibility(ast, vis);
        }
        if let Some(generics) = node.generics {
            visitor.visit_generic_params(ast, generics);
        }

        if let Some(FnReceiver::SelfTyped { is_mut, ty }) = &node.receiver {
            visitor.visit_type(ast, ty);
        }

        for param in &node.params {
            visit_fn_param(visitor, ast, param);
        }

        if let Some(ret) = &node.returns {
            visit_fn_return(visitor, ast, ret);
        }

        if let Some(where_clause) = &node.where_clause {
            visitor.visit_where_clause(ast, *where_clause);
        }
        for contract in &node.contracts {
            visitor.visit_contract(ast,  *contract);
        }
        visitor.visit_block(ast, node.body);
    }

    pub fn visit_fn_param<T: Visitor>(visitor: &mut T, ast: &Ast, param: &FnParam) {
        for name in &param.names {
            for attr in &name.attrs {
                visitor.visit_attribute(ast, *attr);
            }
            visitor.visit_pattern(ast, &name.pattern);
        }

        visitor.visit_type(ast, &param.ty);
        if let Some(expr) = &param.def_val {
            visitor.visit_expr(ast, expr);
        }
    }

    pub fn visit_fn_return<T: Visitor>(visitor: &mut T, ast: &Ast, ret: &FnReturn) {
        match ret {
            FnReturn::Type(ty) => visitor.visit_type(ast, &ty),
            FnReturn::Named(pairs) => for (_, ty) in pairs {
                visitor.visit_type(ast, &ty);
            },
        }
    }

    pub fn visit_type_alias<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<TypeAlias>) {
        match &ast[node_id] {
            TypeAlias::Normal { attrs, vis, name, generics, ty } => {
                for attr in attrs {
                    visitor.visit_attribute(ast, *attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(ast, *vis);
                }
                if let Some(generics) = generics {
                    visitor.visit_generic_params(ast, *generics);
                }
                visitor.visit_type(ast, ty);
            },
            TypeAlias::Distinct { attrs, vis, name, generics, ty } => {
                for attr in attrs {
                    visitor.visit_attribute(ast, *attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(ast, *vis);
                }
                if let Some(generics) = generics {
                    visitor.visit_generic_params(ast, *generics);
                }
                visitor.visit_type(ast, ty);
            },
            TypeAlias::Trait { attrs, name, generics } => {
                for attr in attrs {
                    visitor.visit_attribute(ast, *attr);
                }
                if let Some(generics) = generics {
                    visitor.visit_generic_params(ast, *generics);
                }
            },
            TypeAlias::Opaque { attrs, vis, name, size } => {
                for attr in attrs {
                    visitor.visit_attribute(ast, *attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(ast, *vis);
                }
                if let Some(size) = size {
                    visitor.visit_expr(ast, size);
                }
            },
        }
    }

    pub fn visit_struct<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<Struct>) {
        match &ast[node_id] {
            Struct::Regular { attrs, vis, is_mut, is_record, name, generics, where_clause, fields } => {
                for attr in attrs {
                    visitor.visit_attribute(ast, *attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(ast, *vis);
                }
                if let Some(generics) = generics {
                    visitor.visit_generic_params(ast, *generics);
                }
                if let Some(where_clause) = where_clause {
                    visitor.visit_where_clause(ast, *where_clause);
                }
                for field in fields {
                    visitor.visit_reg_struct_field(ast, field);
                }
            },
            Struct::Tuple { attrs, vis, is_mut, is_record, name, generics, where_clause, fields } => {
                for attr in attrs {
                    visitor.visit_attribute(ast, *attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(ast, *vis);
                }
                if let Some(generics) = generics {
                    visitor.visit_generic_params(ast, *generics);
                }
                if let Some(where_clause) = where_clause {
                    visitor.visit_where_clause(ast, *where_clause);
                }
                for field in fields {
                    visitor.visit_tuple_struct_field(ast, field);
                }
            },
            Struct::Unit { attrs, vis, name } => {
                for attr in attrs {
                    visitor.visit_attribute(ast, *attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(ast, *vis);
                }
            },
        }
    }

    pub fn visit_reg_struct_field<T: Visitor>(visitor: &mut T, ast: &Ast, field: &RegStructField) {
        match field {
            RegStructField::Field { attrs, vis, is_mut, names, ty, def } => {
                for attr in attrs {
                    visitor.visit_attribute(ast, *attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(ast, *vis);
                }
                visitor.visit_type(ast, ty);
                if let Some(expr) = def {
                    visitor.visit_expr(ast, expr);
                }
            },
            RegStructField::Use { attrs, vis, is_mut, path } => {
                for attr in attrs {
                    visitor.visit_attribute(ast, *attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(ast, *vis);
                }
                visitor.visit_type_path(ast, *path);
            },
        }
    }

    pub fn visit_tuple_struct_field<T: Visitor>(visitor: &mut T, ast: &Ast, field: &TupleStructField) {
        for attr in &field.attrs {
            visitor.visit_attribute(ast, *attr);
        }
        if let Some(vis) = &field.vis {
            visitor.visit_visibility(ast, *vis);
        }
        visitor.visit_type(ast, &field.ty);
        if let Some(expr) = &field.def {
            visitor.visit_expr(ast, expr);
        }
    }

    pub fn visit_union<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<Union>) {
        let node = &ast[node_id];
        for attr in &node.attrs {
            visitor.visit_attribute(ast, *attr);
        }
        if let Some(vis) = &node.vis {
            visitor.visit_visibility(ast, *vis);
        }
        if let Some(generics) = &node.generics {
            visitor.visit_generic_params(ast, *generics);
        }
        if let Some(where_clause) = &node.where_clause {
            visitor.visit_where_clause(ast, *where_clause);
        }
        for field in &node.fields {
            for attr in &field.attrs {
                visitor.visit_attribute(ast, *attr);
            }
            if let Some(vis) = field.vis {
                visitor.visit_visibility(ast, vis);
            }
            visitor.visit_type(ast, &field.ty);
        }
    }

    pub fn visit_enum<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<Enum>) {
        match &ast[node_id] {
            Enum::Adt { attrs, vis, is_mut, is_record, name, generics, where_clause, variants } => {
                for attr in attrs {
                    visitor.visit_attribute(ast, *attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(ast, *vis);
                }
                if let Some(generics) = &generics {
                    visitor.visit_generic_params(ast, *generics);
                }
                if let Some(where_clause) = &where_clause {
                    visitor.visit_where_clause(ast, *where_clause);
                }
                for variant in variants {
                    visitor.visit_enum_variant(ast, variant);
                }
            },
            Enum::Flag { attrs, vis, name, variants } => {
                for attr in attrs {
                    visitor.visit_attribute(ast, *attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(ast, *vis);
                }
                for variant in variants {
                    for attr in &variant.attrs {
                        visitor.visit_attribute(ast, *attr);
                    }
                    if let Some(discriminant) = &variant.discriminant {
                        visitor.visit_expr(ast, discriminant);
                    }
                }
            },
        }
    }

    pub fn visit_enum_variant<T: Visitor>(visitor: &mut T, ast: &Ast, variant: &EnumVariant) {
        match variant {
            EnumVariant::Struct { attrs, is_mut, name, fields, discriminant } => {
                for attr in attrs {
                    visitor.visit_attribute(ast, *attr);
                }
                for field in fields {
                    visitor.visit_reg_struct_field(ast, field);
                }
                if let Some(discriminant) = discriminant {
                    visitor.visit_expr(ast, discriminant);
                }
            },
            EnumVariant::Tuple { attrs, is_mut, name, fields, discriminant } => {
                for attr in attrs {
                    visitor.visit_attribute(ast, *attr);
                }
                for field in fields {
                    visitor.visit_tuple_struct_field(ast, field);
                }
                if let Some(discriminant) = discriminant {
                    visitor.visit_expr(ast, discriminant);
                }
            },
            EnumVariant::Fieldless { attrs, name, discriminant } => {
                for attr in attrs {
                    visitor.visit_attribute(ast, *attr);
                }
                if let Some(discriminant) = discriminant {
                    visitor.visit_expr(ast, discriminant);
                }
            },
        }
    }

    pub fn visit_bitfield<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<Bitfield>) {
        let node = &ast[node_id];
        for attr in &node.attrs {
            visitor.visit_attribute(ast, *attr);
        }
        if let Some(vis) = &node.vis {
            visitor.visit_visibility(ast, *vis);
        }
        if let Some(generics) = &node.generics {
            visitor.visit_generic_params(ast, *generics);
        }
        if let Some(where_clause) = &node.where_clause {
            visitor.visit_where_clause(ast, *where_clause);
        }
        for field in &node.fields {
            visitor.visit_bitfield_field(ast, field);
        }
    }

    pub fn visit_bitfield_field<T: Visitor>(visitor: &mut T, ast: &Ast, field: &BitfieldField) {
        match field {
            BitfieldField::Field { attrs, vis, is_mut, names, ty, bits, def } => {
                for attr in attrs {
                    visitor.visit_attribute(ast, *attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(ast, *vis);
                }
                if let Some(bits) = bits {
                    visitor.visit_expr(ast, bits);
                }
                if let Some(def) = def {
                    visitor.visit_expr(ast, def);
                }
            },
            BitfieldField::Use { attrs, vis, is_mut, path, bits } => {
                for attr in attrs {
                    visitor.visit_attribute(ast, *attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(ast, *vis);
                }
                visitor.visit_type_path(ast, *path);
                if let Some(bits) = bits {
                    visitor.visit_expr(ast, bits);
                }
            },
        }
    }

    pub fn visit_const<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<Const>) {
        let node = &ast[node_id];
        for attr in &node.attrs {
            visitor.visit_attribute(ast, *attr);
        }
        if let Some(vis) = &node.vis {
            visitor.visit_visibility(ast, *vis);
        }
        if let Some(ty) = &node.ty {
            visitor.visit_type(ast, ty);
        }
        visitor.visit_expr(ast, &node.val)
    }

    pub fn visit_static<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<Static>) {
        match &ast[node_id] {
            Static::Static { attrs, vis, name, ty, val } => {
                for attr in attrs {
                    visitor.visit_attribute(ast, *attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(ast, *vis);
                }
                visitor.visit_type(ast, ty);
                visitor.visit_expr(ast, val);
            },
            Static::Tls { attrs, vis, is_mut, name, ty, val } => {
                for attr in attrs {
                    visitor.visit_attribute(ast, *attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(ast, *vis);
                }
                visitor.visit_type(ast, ty);
                visitor.visit_expr(ast, val);
            },
            Static::Extern { attrs, vis, abi, is_mut, name, ty } => {
                for attr in attrs {
                    visitor.visit_attribute(ast, *attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(ast, *vis);
                }
                visitor.visit_type(ast, ty);
            },
        }
    }

    pub fn visit_property<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<Property>) {
        let node = &ast[node_id];
        for attr in &node.attrs {
            visitor.visit_attribute(ast, *attr);
        }
        if let Some(vis) = &node.vis {
            visitor.visit_visibility(ast, *vis);
        }
        match &node.body {
            PropertyBody::Assoc { get, ref_get, mut_get, set } => {
                if let Some(get) = get {
                    visitor.visit_expr(ast, get);
                }
                if let Some(ref_get) = ref_get {
                    visitor.visit_expr(ast, ref_get);
                }
                if let Some(mut_get) = mut_get {
                    visitor.visit_expr(ast, mut_get);
                }
                if let Some(set) = set {
                    visitor.visit_expr(ast, set);
                }
            },
            PropertyBody::Trait { has_get, has_ref_get, has_mut_get, has_set } => {},
        }
    }

    pub fn visit_trait<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<Trait>) {
        let node = &ast[node_id];
        for attr in &node.attrs {
            visitor.visit_attribute(ast, *attr);
        }
        if let Some(vis) = &node.vis {
            visitor.visit_visibility(ast, *vis);
        }
        if let Some(bounds) = node.bounds {
            visitor.visit_trait_bounds(ast, bounds);
        }
        for item in &node.assoc_items {
            visitor.visit_trait_item(ast, item);
        }
    }

    pub fn visit_impl<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<Impl>) {
        let node = &ast[node_id];
        for attr in &node.attrs {
            visitor.visit_attribute(ast, *attr);
        }
        if let Some(vis) = &node.vis {
            visitor.visit_visibility(ast, *vis);
        }
        if let Some(generics) = node.generics {
            visitor.visit_generic_params(ast, generics)
        }
        visitor.visit_type(ast, &node.ty);
        if let Some(impl_trait) = node.impl_trait {
            visitor.visit_type_path(ast, impl_trait);
        }
        if let Some(where_clause) = &node.where_clause {
            visitor.visit_where_clause(ast, *where_clause);
        }
        for item in &node.assoc_items {
            visitor.visit_assoc_item(ast, item);
        }
    }

    pub fn visit_extern_block<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<ExternBlock>) {
        let node = &ast[node_id];
        for attr in &node.attrs {
            visitor.visit_attribute(ast, *attr);
        }
        if let Some(vis) = &node.vis {
            visitor.visit_visibility(ast, *vis);
        }
        for item in &node.items {
            visitor.visit_extern_item(ast, item);
        }
    }

    pub fn visit_op_trait<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<OpTrait>) {
        match &ast[node_id] {
            OpTrait::Base { attrs, vis, name, precedence, elems } => {
                for attr in attrs {
                    visitor.visit_attribute(ast, *attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(ast, *vis);
                }
                for elem in elems {
                    visit_op_elem(visitor, ast, elem);
                }
            },
            OpTrait::Extended { attrs, vis, name, bases, elems } => {
                for attr in attrs {
                    visitor.visit_attribute(ast, *attr);
                }
                if let Some(vis) = vis {
                    visitor.visit_visibility(ast, *vis);
                }
                for base in bases {
                    visitor.visit_simple_path(ast, *base);
                }
                for elem in elems {
                    visit_op_elem(visitor, ast, elem);
                }
            },
        }
    }

    pub fn visit_op_elem<T: Visitor>(visitor: &mut T, ast: &Ast, elem: &OpElem) {
        match elem {
            OpElem::Def { op_type, op, name, ret, def } => {
                if let Some(ret) = ret {
                    visitor.visit_type(ast, ret);
                }
                if let Some(def) = def {
                    visitor.visit_expr(ast, def);
                }
            },
            OpElem::Extend { op_type, op, def } => {
                visitor.visit_expr(ast, def);
            },
            OpElem::Contract { expr } => {
                visitor.visit_block_expr(ast, *expr);
            },
        }
    }

    pub fn visit_precedence<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<Precedence>) {
        let node = &ast[node_id];
        for attr in &node.attrs {
            visitor.visit_attribute(ast, *attr);
        }
        if let Some(vis) = &node.vis {
            visitor.visit_visibility(ast, *vis);
        }
    }
    
// =============================================================================================================================

    pub fn visit_block<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<Block>) {
        let node = &ast[node_id];
        for stmt in &node.stmts {
            visitor.visit_stmt(ast, stmt);
        }
        if let Some(final_expr) = &node.final_expr {
            //visitor.visit_expr_stmt(ast, final_expr);
        }
    }

// =============================================================================================================================

    pub fn visit_stmt<T: Visitor>(visitor: &mut T, ast: &Ast, node: &Stmt) {
        match node {
            Stmt::Empty             => {},
            Stmt::Item(item)        => visitor.visit_item(ast, item),
            Stmt::VarDecl(node_id)  => visitor.visit_var_decl(ast, *node_id),
            Stmt::Defer(node_id)    => visitor.visit_defer(ast, *node_id),
            Stmt::ErrDefer(node_id) => visitor.visit_err_defer(ast, *node_id),
            Stmt::Expr(node_id)     => visitor.visit_expr_stmt(ast, *node_id),
        }
    }

    pub fn visit_var_decl<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<VarDecl>) {
        match &ast[node_id] {
            VarDecl::Named { attrs, names, expr } => {
                for attr in attrs {
                    visitor.visit_attribute(ast, *attr);
                }
                visitor.visit_expr(ast, expr);
            },
            VarDecl::Let { attrs, pattern, ty, expr, else_block } => {
                for attr in attrs {
                    visitor.visit_attribute(ast, *attr);
                }
                visitor.visit_pattern(ast, pattern);
                if let Some(ty) = ty {
                    visitor.visit_type(ast, ty);
                }
                if let Some(expr) = expr {
                    visitor.visit_expr(ast, expr);
                }
                if let Some(else_block) = else_block {
                    visitor.visit_block_expr(ast, *else_block);
                }
            },
        }
    }

    pub fn visit_defer<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<Defer>) {
        let node = &ast[node_id];
        for attr in &node.attrs {
            visitor.visit_attribute(ast, *attr);
        }
        visitor.visit_expr(ast, &node.expr);
    }

    pub fn visit_err_defer<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<ErrDefer>) {
        let node = &ast[node_id];
        for attr in &node.attrs {
            visitor.visit_attribute(ast, *attr);
        }
        visitor.visit_expr(ast, &node.expr);
    }

    pub fn visit_expr_stmt<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<ExprStmt>) {
        let node = &ast[node_id];
        for attr in &node.attrs {
            visitor.visit_attribute(ast, *attr);
        }
        visitor.visit_expr(ast, &node.expr);
    }


// =============================================================================================================================

    pub fn visit_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node: &Expr) {
        match node {
            Expr::Literal(node_id)        => visitor.visit_literal_expr(ast, *node_id),
            Expr::Path(node_id)           => visitor.visit_path_expr(ast, *node_id),
            Expr::Unit                    => visitor.visit_unit_expr(ast),
            Expr::Block(node_id)          => visitor.visit_block_expr(ast, *node_id),
            Expr::Prefix(node_id)         => visitor.visit_prefix_expr(ast, *node_id),
            Expr::Postfix(node_id)        => visitor.visit_postfix_expr(ast, *node_id),
            Expr::Infix(node_id)          => visitor.visit_binary_expr(ast, *node_id),
            Expr::Paren(node_id)          => visitor.visit_paren_expr(ast, *node_id),
            Expr::Inplace(node_id)        => visitor.visit_inplace_expr(ast, *node_id),
            Expr::TypeCast(node_id)       => visitor.visit_type_cast_expr(ast, *node_id),
            Expr::TypeCheck(node_id)      => visitor.visit_type_check_expr(ast, *node_id),
            Expr::Tuple(node_id)          => visitor.visit_tuple_expr(ast, *node_id),
            Expr::Array(node_id)          => visitor.visit_array_expr(ast, *node_id),
            Expr::Struct(node_id)         => visitor.visit_struct_expr(ast, *node_id),
            Expr::Index(node_id)          => visitor.visit_index_expr(ast, *node_id),
            Expr::TupleIndex(node_id)     => visitor.visit_tuple_index_expr(ast, *node_id),
            Expr::FnCall(node_id)         => visitor.visit_fn_call_expr(ast, *node_id),
            Expr::Method(node_id)         => visitor.visit_method_call_expr(ast, *node_id),
            Expr::FieldAccess(node_id)    => visitor.visit_field_access_expr(ast, *node_id),
            Expr::Closure(node_id)        => visitor.visit_closure_expr(ast, *node_id),
            Expr::FullRange               => visitor.visit_full_range_expr(ast),
            Expr::If(node_id)             => visitor.visit_if_expr(ast, *node_id),
            Expr::Let(node_id)            => visitor.visit_let_binding_expr(ast, *node_id),
            Expr::Loop(node_id)           => visitor.visit_loop_expr(ast, *node_id),
            Expr::While(node_id)          => visitor.visit_while_expr(ast, *node_id),
            Expr::DoWhile(node_id)        => visitor.visit_do_while_expr(ast, *node_id),
            Expr::For(node_id)            => visitor.visit_for_expr(ast, *node_id),
            Expr::Match(node_id)          => visitor.visit_match_expr(ast, *node_id),
            Expr::Break(node_id)          => visitor.visit_break_expr(ast, *node_id),
            Expr::Continue(node_id)       => visitor.visit_continue_expr(ast, *node_id),
            Expr::Fallthrough(node_id)    => visitor.visit_fallthrough_expr(ast, *node_id),
            Expr::Return(node_id)         => visitor.visit_return_expr(ast, *node_id),
            Expr::Underscore              => visitor.visit_underscore_expr(ast),
            Expr::Throw(node_id)          => visitor.visit_throw_expr(ast, *node_id),
            Expr::Comma(node_id)          => visitor.visit_comma_expr(ast, *node_id),
            Expr::When(node_id)           => visitor.visit_when_expr(ast, *node_id),
        }
    }

    pub fn visit_path_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<PathExpr>) {
    }

    pub fn visit_block_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<BlockExpr>) {
        let node = &ast[node_id];
        visitor.visit_block(ast, node.block);
    }

    pub fn visit_prefix_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<PrefixExpr>) {
        let node = &ast[node_id];
        visitor.visit_expr(ast, &node.expr);
    }

    pub fn visit_postfix_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<PostfixExpr>) {
        let node = &ast[node_id];
        visitor.visit_expr(ast, &node.expr);
    }

    pub fn visit_binary_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<InfixExpr>) {
        let node = &ast[node_id];
        visitor.visit_expr(ast, &node.left);
        visitor.visit_expr(ast, &node.right);
    }

    pub fn visit_paren_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<ParenExpr>) {
        let node = &ast[node_id];
        visitor.visit_expr(ast, &node.expr);
    }

    pub fn visit_inplace_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<InplaceExpr>) {
        let node = &ast[node_id];
        visitor.visit_expr(ast, &node.left);
        visitor.visit_expr(ast, &node.right);
    }

    pub fn visit_type_cast_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<TypeCastExpr>) {
        let node = &ast[node_id];
        visitor.visit_expr(ast, &node.expr);
        visitor.visit_type(ast, &node.ty);
    }

    pub fn visit_type_check_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<TypeCheckExpr>) {
        let node = &ast[node_id];
        visitor.visit_expr(ast, &node.expr);
        visitor.visit_type(ast, &node.ty);
    }

    pub fn visit_tuple_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<TupleExpr>) {
        let node = &ast[node_id];
        for expr in &node.exprs {
            visitor.visit_expr(ast, expr);
        }
    }

    pub fn visit_array_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<ArrayExpr>) {
        let node = &ast[node_id];
        for expr in &node.exprs {
            visitor.visit_expr(ast, expr);
        }
    }

    pub fn visit_struct_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<StructExpr>) {
        let node = &ast[node_id];
        visitor.visit_expr(ast, &node.path);
        for arg in &node.args {
            match arg {
                StructArg::Expr(_, expr)  => visitor.visit_expr(ast, expr),
                StructArg::Name(_)        => {},
                StructArg::Complete(expr) => visitor.visit_expr(ast, expr),
            }
        }
    }

    pub fn visit_index_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<IndexExpr>) {
        let node = &ast[node_id];
        visitor.visit_expr(ast, &node.expr);
        visitor.visit_expr(ast, &node.index);
    }

    pub fn visit_tuple_index_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<TupleIndexExpr>) {
        let node = &ast[node_id];
        visitor.visit_expr(ast, &node.expr);
    }

    pub fn visit_fn_call_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<FnCallExpr>) {
        match &ast[node_id] {
            FnCallExpr::Expr { expr, args } => {
                visitor.visit_expr(ast, expr);
                for arg in args {
                    match arg {
                        FnArg::Expr(expr)              => visitor.visit_expr(ast, expr),
                        FnArg::Labeled { label, expr } => visitor.visit_expr(ast, expr),
                    }
                }
            },
            FnCallExpr::Qual { path, args } => {
                visitor.visit_qualified_path(ast, *path);
                for arg in args {
                    match arg {
                        FnArg::Expr(expr)              => visitor.visit_expr(ast, expr),
                        FnArg::Labeled { label, expr } => visitor.visit_expr(ast, expr),
                    }
                }
            },
        }
    }

    pub fn visit_method_call_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<MethodCallExpr>) {
        let node = &ast[node_id];
        visitor.visit_expr(ast, &node.receiver);
        if let Some(gen_args) = node.gen_args {
            visitor.visit_generic_args(ast, gen_args);
        }
        for arg in &node.args {
            match arg {
                FnArg::Expr(expr)              => visitor.visit_expr(ast, expr),
                FnArg::Labeled { label, expr } => visitor.visit_expr(ast, expr),
            }
        }
    }

    pub fn visit_field_access_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<FieldAccessExpr>) {
        let node = &ast[node_id];
        visitor.visit_expr(ast, &node.expr);
    }
    
    pub fn visit_closure_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<ClosureExpr>) {
        let node = &ast[node_id];
        for param in &node.params {
            visit_fn_param(visitor, ast, param);
        }
        if let Some(ret) = &node.ret {
            visit_fn_return(visitor, ast, ret);
        }
        visitor.visit_expr(ast, &node.body);
    }

    pub fn visit_let_binding_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<LetBindingExpr>) {
        let node = &ast[node_id];
        visitor.visit_pattern(ast, &node.pattern);
        visitor.visit_expr(ast, &node.scrutinee);
    }
    
    pub fn visit_if_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<IfExpr>) {
        let node = &ast[node_id];
        visitor.visit_expr(ast, &node.cond);
        visitor.visit_block_expr(ast, node.body);
        if let Some(else_body) = &node.else_body {
            visitor.visit_expr(ast, &else_body);
        }
    }   

    pub fn visit_loop_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<LoopExpr>) {
        let node = &ast[node_id];
        visitor.visit_block_expr(ast, node.body);
    }   

    pub fn visit_while_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<WhileExpr>) {
        let node = &ast[node_id];
        visitor.visit_expr(ast, &node.cond);
        if let Some(inc) = &node.inc {
            visitor.visit_expr(ast, &inc);
        }
        visitor.visit_block_expr(ast, node.body);
        if let Some(else_body) = node.else_body {
            visitor.visit_block_expr(ast, else_body);
        }
    }

    pub fn visit_do_while_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<DoWhileExpr>) {
        let node = &ast[node_id];
        visitor.visit_block_expr(ast, node.body);
        visitor.visit_expr(ast, &node.cond);
    }

    pub fn visit_for_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<ForExpr>) {
        let node = &ast[node_id];
        visitor.visit_pattern(ast, &node.pattern);
        visitor.visit_expr(ast, &node.src);
        visitor.visit_block_expr(ast, node.body);
        if let Some(else_body) = node.else_body {
            visitor.visit_block_expr(ast, else_body);
        }
    }

    pub fn visit_match_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<MatchExpr>) {
        let node = &ast[node_id];
        visitor.visit_expr(ast, &node.scrutinee);
        for branch in &node.branches {
            visitor.visit_pattern(ast, &branch.pattern);
            if let Some(guard) = &branch.guard {
                visitor.visit_expr(ast, guard);
            }
            visitor.visit_expr(ast, &branch.body);
        }
    }

    pub fn visit_break_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<BreakExpr>) {
        let node = &ast[node_id];
        if let Some(expr) = &node.value {
            visitor.visit_expr(ast, expr);
        }
    }   

    pub fn visit_return_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<ReturnExpr>) {
        let node = &ast[node_id];
        if let Some(expr) = &node.value {
            visitor.visit_expr(ast, expr);
        }
    }

    pub fn visit_throw_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<ThrowExpr>) {
        let node = &ast[node_id];
        visitor.visit_expr(ast, &node.expr);
    }
    
    pub fn visit_comma_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<CommaExpr>) {
        let node = &ast[node_id];
        for expr in &node.exprs {
            visitor.visit_expr(ast, expr);
        }
    }

    pub fn visit_when_expr<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<WhenExpr>) {
        let node = &ast[node_id];
        visitor.visit_expr(ast, &node.cond);
        visitor.visit_block_expr(ast, node.body);
        if let Some(else_body) = &node.else_body {
            visitor.visit_expr(ast, else_body);
        }
    }


// =============================================================================================================================

    pub fn visit_pattern<T: Visitor>(visitor: &mut T, ast: &Ast, pattern: &Pattern) {
        match pattern {
            Pattern::Literal(node_id)     => visitor.visit_literal_pattern(ast, *node_id),
            Pattern::Identifier(node_id)  => visitor.visit_identifier_pattern(ast, *node_id),
            Pattern::Path(node_id)        => visitor.visit_path_pattern(ast, *node_id),
            Pattern::Wildcard             => visitor.visit_wildcard_pattern(ast),
            Pattern::Rest                 => visitor.visit_rest_pattern(ast),
            Pattern::Range(node_id)       => visitor.visit_range_pattern(ast, *node_id),
            Pattern::Reference(node_id)   => visitor.visit_reference_pattern(ast, *node_id),
            Pattern::Struct(node_id)      => visitor.visit_struct_pattern(ast, *node_id),
            Pattern::TupleStruct(node_id) => visitor.visit_tuple_struct_pattern(ast, *node_id),
            Pattern::Tuple(node_id)       => visitor.visit_tuple_pattern(ast, *node_id),
            Pattern::Grouped(node_id)     => visitor.visit_grouped_pattern(ast, *node_id),
            Pattern::Slice(node_id)       => visitor.visit_slice_pattern(ast, *node_id),
            Pattern::EnumMember(node_id)  => visitor.visit_enum_member_pattern(ast, *node_id),
            Pattern::Alternative(node_id) => visitor.visit_alternative_pattern(ast, *node_id),
            Pattern::TypeCheck(node_id)   => visitor.visit_type_check_pattern(ast, *node_id),
        }
    }

    pub fn visit_identifier_pattern<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<IdentifierPattern>) {
        let node = &ast[node_id];
        if let Some(bound) = &node.bound {
            visitor.visit_pattern(ast, bound);
        }
    }

    pub fn visit_path_pattern<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<PathPattern>) {
        let node = &ast[node_id];
        visitor.visit_expr_path(ast, node.path);
    }

    pub fn visit_range_pattern<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<RangePattern>) {
        match &ast[node_id] {
            RangePattern::Exclusive { begin, end } => {
                visitor.visit_pattern(ast, begin);
                visitor.visit_pattern(ast, end);
            },
            RangePattern::Inclusive { begin, end } => {
                visitor.visit_pattern(ast, begin);
                visitor.visit_pattern(ast, end);
            },
            RangePattern::From { begin } => {
                visitor.visit_pattern(ast, begin);
            },
            RangePattern::To { end } => {
                visitor.visit_pattern(ast, end);
            },
            RangePattern::InclusiveTo { end } => {
                visitor.visit_pattern(ast, end);
            },
        }
    }

    pub fn visit_reference_pattern<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<ReferencePattern>) {
        let node = &ast[node_id];
        visitor.visit_pattern(ast, &node.pattern);
    }

    pub fn visit_struct_pattern<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<StructPattern>) {
        let node = &ast[node_id];
        for field in &node.fields {
            match field {
                StructPatternField::Named { name, pattern }       => visitor.visit_pattern(ast, pattern),
                StructPatternField::TupleIndex { idx, pattern }   => visitor.visit_pattern(ast, pattern),
                StructPatternField::Iden { is_ref, is_mut, iden } => {},
                StructPatternField::Rest                          => {},
            }
        }
    }

    pub fn visit_tuple_struct_pattern<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<TupleStructPattern>) {
        let node = &ast[node_id];
        for pattern in &node.patterns {
            visitor.visit_pattern(ast, pattern);
        }
    }

    pub fn visit_tuple_pattern<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<TuplePattern>) {
        let node = &ast[node_id];
        for pattern in &node.patterns {
            visitor.visit_pattern(ast, pattern);
        }
    }

    pub fn visit_grouped_pattern<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<GroupedPattern>) {
        let node = &ast[node_id];
        visitor.visit_pattern(ast, &node.pattern);
    }

    pub fn visit_slice_pattern<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<SlicePattern>) {
        let node = &ast[node_id];
        for pattern in &node.patterns {
            visitor.visit_pattern(ast, pattern);
        }
    }

    pub fn visit_alternative_pattern<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<AlternativePattern>) {
        let node = &ast[node_id];
        for pattern in &node.patterns {
            visitor.visit_pattern(ast, pattern);
        }
    }

    pub fn visit_type_check_pattern<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<TypeCheckPattern>) {
        let node = &ast[node_id];
        visitor.visit_type(ast, &node.ty);
    }

// =============================================================================================================================

    pub fn visit_type<T: Visitor>(visitor: &mut T, ast: &Ast, node: &Type) {

    }

    pub fn visit_paren_type<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<ParenthesizedType>) {
        let node = &ast[node_id];
        visitor.visit_type(ast, &node.ty);
    }

    pub fn visit_path_type<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<PathType>) {
        let node = &ast[node_id];
        visitor.visit_type_path(ast, node.path);
    }

    pub fn visit_tuple_type<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<TupleType>) {
        let node = &ast[node_id];
        for ty in &node.types {
            visitor.visit_type(ast, ty);
        }
    }

    pub fn visit_array_type<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<ArrayType>) {
        let node = &ast[node_id];
        visitor.visit_expr(ast, &node.size);
        if let Some(sentinel) = &node.sentinel {
            visitor.visit_expr(ast, sentinel);
        }
        visitor.visit_type(ast, &node.ty);
    }

    pub fn visit_slice_type<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<SliceType>) {
        let node = &ast[node_id];
        if let Some(sentinel) = &node.sentinel {
            visitor.visit_expr(ast, sentinel);
        }
        visitor.visit_type(ast, &node.ty);
    }

    pub fn visit_pointer_type<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<PointerType>) {
        let node = &ast[node_id];
        if let Some(sentinel) = &node.sentinel {
            visitor.visit_expr(ast, sentinel);
        }
        visitor.visit_type(ast, &node.ty);
    }

    pub fn visit_reference_type<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<ReferenceType>) {
        let node = &ast[node_id];
        visitor.visit_type(ast, &node.ty);
    }

    pub fn visit_optional_type<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<OptionalType>) {
        let node = &ast[node_id];
        visitor.visit_type(ast, &node.ty);
    }

    pub fn visit_fn_type<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<FnType>) {
        let node = &ast[node_id];
        for (_, ty) in &node.params {
            visitor.visit_type(ast, ty);
        }
        if let Some(ret_ty) = &node.return_ty {
            visitor.visit_type(ast, ret_ty);
        }
    }

    pub fn visit_record_type<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<RecordType>) {
        let node = &ast[node_id];
        for field in &node.fields {
            visit_reg_struct_field(visitor, ast, field);
        }
    }

    pub fn visit_enum_record_type<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<EnumRecordType>) {
        let node = &ast[node_id];
        for variant in &node.variants {
            visit_enum_variant(visitor, ast, variant);
        }
    }

// =============================================================================================================================

    pub fn visit_visibility<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<Visibility>) {
        match &ast[node_id] {
            Visibility::Path(path) => visitor.visit_simple_path(ast, *path),
            _                      => {},
        }
    }

    pub fn visit_attribute<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<Attribute>) {
        let node = &ast[node_id];
        for meta in &node.metas {
            visit_attribute_meta(visitor, ast, meta);
        }
    }

    pub fn visit_attribute_meta<T: Visitor>(visitor: &mut T, ast: &Ast, meta: &AttribMeta) {
        match meta {
            AttribMeta::Simple { path }       => visitor.visit_simple_path(ast, *path),
            AttribMeta::Expr { expr }         => visitor.visit_expr(ast, expr),
            AttribMeta::Assign { path, expr } => {
                visitor.visit_simple_path(ast, *path);
                visitor.visit_expr(ast, expr);
            },
            AttribMeta::Meta { path, metas }  => {
                visitor.visit_simple_path(ast, *path);
                for meta in metas {
                    visit_attribute_meta(visitor, ast, meta);
                }
            },
        }
    }

// =============================================================================================================================

    pub fn visit_contract<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<Contract>) {

    }

// =============================================================================================================================

    pub fn visit_generic_params<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<GenericParams>) {

    }

    pub fn visit_generic_args<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<GenericArgs>) {

    }

    pub fn visit_where_clause<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<WhereClause>) {

    }

    pub fn visit_trait_bounds<T: Visitor>(visitor: &mut T, ast: &Ast, node_id: AstNodeRef<TraitBounds>) {

    }
}