
use bootstrap_macros::flags;

use super::*;

#[flags]
pub enum VisitFlags {
    Function,
    ExternFunctionNoBody,
    TypeAlias,
    DistinctType,
    OpaqueType,
    Struct,
    TupleStruct,
    UnitStruct,
    Union,
    AdtEnum,
    FlagEnum,
    Bitfield,
    Const,
    Static,
    TlsStatic,
    ExternStatic,
    
    Trait,
    TraitFunction,
    TraitMethod,
    TraitTypeAlias,
    TraitConst,
    TraitProperty,
    AnyTrait = Trait | TraitFunction | TraitMethod | TraitTypeAlias | TraitConst | TraitConst | TraitProperty,
    
    Impl,
    ImplFunction,
    Method,
    ImplTypeAlias,
    ImplConst,
    ImplStatic,
    ImplTlsStatic,
    Property,
    AnyImpl = Impl | ImplFunction | Method | ImplTypeAlias | ImplConst | ImplStatic | ImplTlsStatic | Property,

    OpTrait,
    OpFunction,
    OpContract,
    AnyOpTrait = OpTrait | OpFunction | OpContract,

    Precedence,
}

pub trait Visitor: Sized {

    fn visit(&mut self, hir: &mut Hir, flags: VisitFlags) {
        helpers::visit(self, hir, flags)
    }

    // =============================================================

    fn visit_simple_path(&mut self, path: &mut SimplePath) {

    }

    fn visit_path(&mut self, path: &mut Path) {
        helpers::visit_path(self, path);
    }

    // =============================================================

    fn visit_function(&mut self, node: &mut Function, ctx: &mut FunctionContext) {
        helpers::visit_function(self, node);
    }

    fn visit_extern_function_no_body(&mut self, node: &mut ExternFunctionNoBody, ctx: &mut FunctionContext) {
        helpers::visit_extern_function(self, node);
    }

    fn visit_type_alias(&mut self, node: &mut TypeAlias, ctx: &mut TypeAliasContext) {
        helpers::visit_type_alias(self, node);
    }

    fn visit_distinct_type(&mut self, node: &mut DistinctType, ctx: &mut TypeAliasContext) {
        helpers::visit_distinct_type(self, node);
    }

    fn visit_opaque_type(&mut self, node: &mut OpaqueType, ctx: &mut TypeAliasContext) {
        helpers::visit_opaque_type(self, node);
    }

    fn visit_struct(&mut self, node: &mut Struct, ctx: &mut StructContext) {
        helpers::visit_struct(self, node);
    }

    fn visit_tuple_struct(&mut self, node: &mut TupleStruct, ctx: &mut StructContext) {
        helpers::visit_tuple_struct(self, node);
    }

    fn visit_unit_struct(&mut self, node: &mut UnitStruct, ctx: &mut StructContext) {
        helpers::visit_unit_struct(self, node);
    }

    fn visit_union(&mut self, node: &mut Union, ctx: &mut UnionContext) {
        helpers::visit_union(self, node);
    }

    fn visit_adt_enum(&mut self, node: &mut AdtEnum, ctx: &mut AdtEnumContext) {
        helpers::visit_adt_enum(self, node);
    }

    fn visit_flag_enum(&mut self, node: &mut FlagEnum, ctx: &mut FlagEnumContext) {
        helpers::visit_flag_enum(self, node);
    }

    fn visit_bitfield(&mut self, node: &mut Bitfield, ctx: &mut BitfieldContext) {
        helpers::visit_bitfield(self, node);
    }

    fn visit_const(&mut self, node: &mut Const, ctx: &mut ConstContext) {
        helpers::visit_const(self, node);
    }

    fn visit_static(&mut self, node: &mut Static, ctx: &mut StaticContext) {
        helpers::visit_static(self, node);
    }

    fn visit_tls_static(&mut self, node: &mut TlsStatic, ctx: &mut StaticContext) {
        helpers::visit_tls_static(self, node);
    }

    fn visit_extern_static(&mut self, node: &mut ExternStatic, ctx: &mut StaticContext) {
        helpers::visit_extern_static(self, node);
    }

    //--------------------------------------------------------------

    fn visit_trait(&mut self, node: &mut Trait, ctx: &mut TraitContext) {
        helpers::visit_trait(self, node);
    }

    fn visit_trait_function(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitFunction, ctx: &mut FunctionContext) {
        helpers::visit_trait_function(self, node);
    }

    fn visit_trait_method(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitMethod, ctx: &mut FunctionContext) {
        helpers::visit_trait_method(self, node);
    }

    fn visit_trait_type_alias(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitTypeAlias, ctx: &mut TypeAliasContext) {
        helpers::visit_trait_type_alias(self, node);
    }

    fn visit_trait_const(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitConst, ctx: &mut ConstContext) {
        helpers::visit_trait_const(self, node);
    }

    fn visit_trait_property(&mut self, trait_ref: Ref<Trait>, trait_ctx: Ref<TraitContext>, node: &mut TraitProperty, ctx: &mut PropertyContext) {
        helpers::visit_trait_property(self, node);
    }

    //--------------------------------------------------------------

    fn visit_impl(&mut self, node: &mut Impl, ctx: &mut ImplContext) {
        helpers::visit_impl(self, node);
    }

    fn visit_impl_function(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut Function, ctx: &mut FunctionContext) {
        helpers::visit_function(self, node);
    }

    fn visit_method(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut Method, ctx: &mut FunctionContext) {
        helpers::visit_method(self, node);
    }

    fn visit_impl_type_alias(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut TypeAlias, ctx: &mut TypeAliasContext) {
        helpers::visit_type_alias(self, node);
    }

    fn visit_impl_const(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut Const, ctx: &mut ConstContext) {
        helpers::visit_const(self, node);
    }

    fn visit_impl_static(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut Static, ctx: &mut StaticContext) {
        helpers::visit_static(self, node);
    }

    fn visit_impl_tls_static(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut TlsStatic, ctx: &mut StaticContext) {
        helpers::visit_tls_static(self, node);
    }

    fn visit_property(&mut self, impl_ref: Ref<Impl>, impl_ctx: Ref<ImplContext>, node: &mut Property, ctx: &mut PropertyContext) {
        helpers::visit_propety(self, node);
    }

    //--------------------------------------------------------------

    fn visit_op_trait(&mut self, node: &mut OpTrait, ctx: &mut OpTraitContext) {
        helpers::visit_op_trait(self, node, ctx);
    }

    fn visit_op_function(&mut self, op_trait_ref: Ref<OpTrait>, op_trait_ctx: Ref<OpTraitContext>, node: &mut OpFunction, ctx: &mut OpFunctionContext) {
        helpers::visit_op_function(self, node, ctx);
    }

    fn visit_op_contract(&mut self, op_trait_ref: Ref<OpTrait>, op_trait_ctx: Ref<OpTraitContext>, node: &mut OpContract, ctx: &mut OpContractContext) {
        helpers::visit_op_contract(self, node, ctx);
    }

    //--------------------------------------------------------------

    fn visit_precedence(&mut self, node: &mut Precedence, ctx: Ref<PrecedenceContext>) {
        helpers::visit_precedence(self, node)
    }

    // =============================================================

    fn visit_block(&mut self, node: &mut Block) {
        helpers::visit_block(self, node);
    }

    // =============================================================

    fn visit_stmt(&mut self, stmt: &mut Stmt) {
        helpers::visit_stmt(self, stmt);
    }

    fn visit_var_decl(&mut self, node: &mut VarDecl) {
        helpers::visit_var_decl(self, node);
    }

    fn visit_uninit_var_decl(&mut self, node: &mut UninitVarDecl) {
        helpers::visit_uninit_var_decl(self, node);
    }

    fn visit_defer_stmt(&mut self, node: &mut DeferStmt) {
        helpers::visit_defer_stmt(self, node);
    }

    fn visit_err_defer_stmt(&mut self, node: &mut ErrorDeferStmt) {
        helpers::visit_err_defer_stmt(self, node);
    }

    fn visit_expr_stmt(&mut self, node: &mut ExprStmt) {
        helpers::visit_expr_stmt(self, node);
    }

    // =============================================================

    fn visit_expr(&mut self, expr: &mut Expr) {
        helpers::visit_expr(self, expr);
    }

    fn visit_unit_expr(&mut self, node: &mut UnitExpr) {}
    fn visit_fullrange_expr(&mut self, node: &mut FullRangeExpr) {}
    fn visit_underscore_expr(&mut self, node: &mut UnderscoreExpr) {}

    fn visit_literal_expr(&mut self, node: &mut LiteralExpr) {}

    fn visit_path_expr(&mut self, node: &mut PathExpr) {
        helpers::visit_path_expr(self, node);
    }

    fn visit_block_expr(&mut self, node: &mut BlockExpr) {
        helpers::visit_block_expr(self, node);
    }

    fn visit_prefix_expr(&mut self, node: &mut PrefixExpr) {
        helpers::visit_prefix_expr(self, node);
    }

    fn visit_postfix_expr(&mut self, node: &mut PostfixExpr) {
        helpers::visit_postfix_expr(self, node);
    }

    fn visit_infix_expr(&mut self, node: &mut InfixExpr) {
        helpers::visit_infix_expr(self, node);
    }

    fn visit_inplace_expr(&mut self, node: &mut InplaceExpr) {
        helpers::visit_inplace_expr(self, node);
    }

    fn visit_type_cast_expr(&mut self, node: &mut TypeCastExpr) {
        helpers::visit_type_cast_expr(self, node);
    }

    fn visit_type_check_expr(&mut self, node: &mut TypeCheckExpr) {
        helpers::visit_type_check_expr(self, node);
    }

    fn visit_tuple_expr(&mut self, node: &mut TupleExpr) {
        helpers::visit_tuple_expr(self, node);
    }

    fn visit_slice_expr(&mut self, node: &mut SliceExpr) {
        helpers::visit_slice_expr(self, node);
    }

    fn visit_array_expr(&mut self, node: &mut ArrayExpr) {
        helpers::visit_array_expr(self, node);
    }

    fn visit_struct_expr(&mut self, node: &mut StructExpr) {
        helpers::visit_struct_expr(self, node);
    }

    fn visit_index_expr(&mut self, node: &mut IndexExpr) {
        helpers::visit_index_expr(self, node);
    }

    fn visit_tuple_index_expr(&mut self, node: &mut TupleIndexExpr) {
        helpers::visit_tuple_index_expr(self, node);
    }

    fn visit_fn_call_expr(&mut self, node: &mut FnCallExpr) {
        helpers::visit_fn_call_expr(self, node);
    }

    fn visit_method_call_expr(&mut self, node: &mut MethodCallExpr) {
        helpers::visit_method_call_expr(self, node);
    }

    fn visit_field_access_expr(&mut self, node: &mut FieldAccessExpr) {
        helpers::visit_field_access_expr(self, node);
    }

    fn visit_closure_expr(&mut self, node: &mut ClosureExpr) {
        helpers::visit_closure_expr(self, node);
    }

    fn visit_loop_expr(&mut self, node: &mut LoopExpr) {
        helpers::visit_loop_expr(self, node);
    }

    fn visit_match_expr(&mut self, node: &mut MatchExpr) {
        helpers::visit_match_expr(self, node);
    }

    fn visit_break_expr(&mut self, node: &mut BreakExpr) {
        helpers::visit_break_expr(self, node);
    }

    fn visit_continue_expr(&mut self, node: &mut ContinueExpr) {}
    fn visit_fallthrough_expr(&mut self, node: &mut FallthroughExpr) {}

    fn visit_return_expr(&mut self, node: &mut ReturnExpr) {
        helpers::visit_return_expr(self, node);
    }

    fn visit_throw_expr(&mut self, node: &mut ThrowExpr) {
        helpers::visit_throw_expr(self, node);
    }

    fn visit_comma_expr(&mut self, node: &mut CommaExpr) {
        helpers::visit_comma_expr(self, node);
    }

    fn visit_when_expr(&mut self, node: &mut WhenExpr) {
        helpers::visit_when_expr(self, node);
    }

    fn visit_irrefutable_expr(&mut self) {}

    // =============================================================

    fn visit_pattern(&mut self, node: &mut Pattern) {
        helpers::visit_pattern(self, node);
    }

    fn visit_wildcard_pattern(&mut self, node: &mut WildcardPattern) {}
    fn visit_rest_pattern(&mut self, node: &mut RestPattern) {}

    fn visit_literal_pattern(&mut self, node: &mut LiteralPattern) {}
    fn visit_iden_pattern(&mut self, node: &mut IdenPattern) {}

    fn visit_path_pattern(&mut self, node: &mut PathPattern) {
        helpers::visit_path_pattern(self, node);
    }

    fn visit_range_pattern(&mut self, node: &mut RangePattern) {
        helpers::visit_range_pattern(self, node);
    }

    fn visit_reference_pattern(&mut self, node: &mut ReferencePattern) {
        helpers::visit_reference_pattern(self, node);
    }

    fn visit_struct_pattern(&mut self, node: &mut StructPattern) {
        helpers::visit_struct_pattern(self, node);
    }

    fn visit_tuple_struct_pattern(&mut self, node: &mut TupleStructPattern) {
        helpers::visit_tuple_struct_pattern(self, node);
    }

    fn visit_tuple_pattern(&mut self, node: &mut TuplePattern) {
        helpers::visit_tuple_pattern(self, node);
    }

    fn visit_slice_pattern(&mut self, node: &mut SlicePattern) {
        helpers::visit_slice_pattern(self, node);
    }

    fn visit_enum_member_pattern(&mut self, node: &mut EnumMemberPattern) {}

    fn visit_alternative_pattern(&mut self, node: &mut AlternativePattern) {
        helpers::visit_alternative_pattern(self, node);
    }

    fn visit_type_check_pattern(&mut self, node: &mut TypeCheckPattern) {
        helpers::visit_type_check_pattern(self, node);
    }

    // =============================================================

    fn visit_type(&mut self, node: &mut Type) {
        helpers::visit_type(self, node)
    }

    fn visit_unit_type(&mut self, node: &mut UnitType) {}
    fn visit_never_type(&mut self, node: &mut NeverType) {}

    fn visit_primitive_type(&mut self, node: &mut PrimitiveType) {}

    fn visit_path_type(&mut self, node: &mut PathType) {
        helpers::visit_path_type(self, node);
    }

    fn visit_tuple_type(&mut self, node: &mut TupleType) {
        helpers::visit_tuple_type(self, node)
    }

    fn visit_array_type(&mut self, node: &mut ArrayType) {
        helpers::visit_array_type(self, node)
    }

    fn visit_slice_type(&mut self, node: &mut SliceType) {
        helpers::visit_slice_type(self, node)
    }

    fn visit_string_slice_type(&mut self, node: &mut StringSliceType) {}

    fn visit_pointer_type(&mut self, node: &mut PointerType) {
        helpers::visit_pointer_type(self, node);
    }

    fn visit_reference_type(&mut self, node: &mut ReferenceType) {
        helpers::visit_reference_type(self, node);
    }

    fn visit_optional_type(&mut self, node: &mut OptionalType) {
        helpers::visit_optional_type(self, node);
    }

    fn visit_fn_type(&mut self, node: &mut FnType) {
        helpers::visit_fn_type(self, node);
    }
    
    // =============================================================
    
    fn visit_gen_params(&mut self, node: &mut GenericParams) {
        helpers::visit_gen_params(self, node);
    }

    fn visit_gen_args(&mut self, node: &mut GenericArgs) {
        helpers::visit_gen_args(self, node);
    }

    fn visit_where_clause(&mut self, node: &mut WhereClause) {
        helpers::visit_where_clause(self, node);
    }

    fn visit_trait_bounds(&mut self, node: &mut TraitBounds) {
        helpers::visit_trait_bounds(self, node);
    }
    
    // =============================================================

    fn visit_contract(&mut self, node: &mut Contract) {

    }

    // =============================================================


    fn visit_attribute(&mut self, node: &mut Attribute) {

    }
}

pub(crate) mod helpers {
    use super::*;

    pub fn visit<T: Visitor>(visitor: &mut T, hir: &mut Hir, flags: VisitFlags) {
        if flags.contains(VisitFlags::Function) {
            for (node, ctx) in &mut hir.functions {
                visitor.visit_function(node, ctx);
            }
        }

        if flags.contains(VisitFlags::ExternFunctionNoBody) {
            for (node, ctx) in &mut hir.extern_functions_no_body {
                visitor.visit_extern_function_no_body(node, ctx);
            }
        }

        if flags.contains(VisitFlags::TypeAlias) {
            for (node, ctx) in &mut hir.type_aliases {
                visitor.visit_type_alias(node, ctx);
            }
        }

        if flags.contains(VisitFlags::DistinctType) {
            for (node, ctx) in &mut hir.distinct_types {
                visitor.visit_distinct_type(node, ctx);
            }
        }

        if flags.contains(VisitFlags::OpaqueType) {
            for (node, ctx) in &mut hir.opaque_types {
                visitor.visit_opaque_type(node, ctx);
            }
        }

        if flags.contains(VisitFlags::Struct) {
            for (node, ctx) in &mut hir.structs {
                visitor.visit_struct(node, ctx);
            }
        }

        if flags.contains(VisitFlags::TupleStruct) {
            for (node, ctx) in &mut hir.tuple_structs {
                visitor.visit_tuple_struct(node, ctx);
            }
        }

        if flags.contains(VisitFlags::UnitStruct) {
            for (node, ctx) in &mut hir.unit_structs {
                visitor.visit_unit_struct(node, ctx);
            }
        }

        if flags.contains(VisitFlags::Union) {
            for (node, ctx) in &mut hir.unions {
                visitor.visit_union(node, ctx);
            }
        }

        if flags.contains(VisitFlags::AdtEnum) {
            for (node, ctx) in &mut hir.adt_enums {
                visitor.visit_adt_enum(node, ctx);
            }
        }

        if flags.contains(VisitFlags::FlagEnum) {
            for (node, ctx) in &mut hir.flag_enums {
                visitor.visit_flag_enum(node, ctx);
            }
        }

        if flags.contains(VisitFlags::Bitfield) {
            for (node, ctx) in &mut hir.bitfields {
                visitor.visit_bitfield(node, ctx);
            }
        }

        if flags.contains(VisitFlags::Const) {
            for (node, ctx) in &mut hir.consts {
                visitor.visit_const(node, ctx);
            }
        }

        if flags.contains(VisitFlags::Static) {
            for (node, ctx) in &mut hir.statics {
                visitor.visit_static(node, ctx);
            }
        }

        if flags.contains(VisitFlags::TlsStatic) {
            for (node, ctx) in &mut hir.tls_statics {
                visitor.visit_tls_static(node, ctx);
            }
        }

        if flags.contains(VisitFlags::ExternStatic) {
            for (node, ctx) in &mut hir.extern_statics {
                visitor.visit_extern_static(node, ctx);
            }
        }

        // TODO: Visit trait, then all associate elements (same for similar types of things)

    
        if flags.intersects(VisitFlags::AnyTrait) {
            let mut func_offset = 0;
            let mut method_offset = 0;
            let mut type_alias_offset = 0;
            let mut const_offset = 0;
            let mut prop_offset = 0;

            for (idx, (trait_ref, trait_ctx)) in hir.traits.iter_mut().enumerate() {
                if flags.contains(VisitFlags::Trait) {
                    let mut node = trait_ref.write();
                    let mut ctx = trait_ctx.write();
                    visitor.visit_trait(&mut node, &mut ctx);
                }
                
                if flags.contains(VisitFlags::TraitFunction) {
                    for (trait_idx, node, ctx) in &mut hir.trait_functions[func_offset..] {
                        if *trait_idx != idx {
                            break;
                        }
                        visitor.visit_trait_function(trait_ref.clone(), trait_ctx.clone(), node, ctx);
                        func_offset += 1;
                    }
                }
                
                if flags.contains(VisitFlags::TraitMethod) {
                    for (trait_idx, node, ctx) in &mut hir.trait_methods[method_offset..] {
                        if *trait_idx != idx {
                            break;
                        }
                        visitor.visit_trait_method(trait_ref.clone(), trait_ctx.clone(), node, ctx);
                        method_offset += 1;
                    }
                }
                
                if flags.contains(VisitFlags::TraitTypeAlias) {
                    for (trait_idx, node, ctx) in &mut hir.trait_type_alias[type_alias_offset..] {
                        if *trait_idx != idx {
                            break;
                        }
                        visitor.visit_trait_type_alias(trait_ref.clone(), trait_ctx.clone(), node, ctx);
                        type_alias_offset += 1;
                    }
                }
                
                if flags.contains(VisitFlags::TraitConst) {
                    for (trait_idx, node, ctx) in &mut hir.trait_consts[const_offset..] {
                        if *trait_idx != idx {
                            break;
                        }
                        visitor.visit_trait_const(trait_ref.clone(), trait_ctx.clone(), node, ctx);
                        const_offset += 1;
                    }
                }
                
                if flags.contains(VisitFlags::TraitProperty) {
                    for (trait_idx, node, ctx) in &mut hir.trait_properties[prop_offset..] {
                        if *trait_idx != idx {
                            break;
                        }
                        visitor.visit_trait_property(trait_ref.clone(), trait_ctx.clone(), node, ctx);
                        prop_offset += 1;
                    }
                }
            }
        }

        if flags.intersects(VisitFlags::AnyImpl) {
            let mut func_offset = 0;
            let mut method_offset = 0;
            let mut type_alias_offset = 0;
            let mut const_offset = 0;
            let mut static_offset = 0;
            let mut tls_static_offset = 0;
            let mut prop_offset = 0;

            for (idx, (impl_ref, impl_ctx)) in hir.impls.iter_mut().enumerate() {
                if flags.contains(VisitFlags::Impl) {
                    let mut node = impl_ref.write();
                    let mut ctx = impl_ctx.write();
                    visitor.visit_impl(&mut node, &mut ctx);
                }
                
                if flags.contains(VisitFlags::ImplFunction) {
                    for (impl_idx, node, ctx) in &mut hir.impl_functions[func_offset..] {
                        if *impl_idx != idx {
                            break;
                        }
                        visitor.visit_impl_function(impl_ref.clone(), impl_ctx.clone(), node, ctx);
                        func_offset += 1;
                    }
                }

                if flags.contains(VisitFlags::Method) {
                    for (impl_idx, node, ctx) in &mut hir.methods[method_offset..] {
                        if *impl_idx != idx {
                            break;
                        }
                        visitor.visit_method(impl_ref.clone(), impl_ctx.clone(), node, ctx);
                        method_offset += 1;
                    }
                }

                if flags.contains(VisitFlags::ImplTypeAlias) {
                    for (impl_idx, node, ctx) in &mut hir.impl_type_aliases[type_alias_offset..] {
                        if *impl_idx != idx {
                            break;
                        }
                        visitor.visit_impl_type_alias(impl_ref.clone(), impl_ctx.clone(), node, ctx);
                        type_alias_offset += 1;
                    }
                }
            
                if flags.contains(VisitFlags::ImplConst) {
                    for (impl_idx, node, ctx) in &mut hir.impl_consts[const_offset..] {
                        if *impl_idx != idx {
                            break;
                        }
                        visitor.visit_impl_const(impl_ref.clone(), impl_ctx.clone(), node, ctx);
                        const_offset += 1;
                    }
                }

                if flags.contains(VisitFlags::ImplStatic) {
                    for (impl_idx, node, ctx) in &mut hir.impl_statics[static_offset..] {
                        if *impl_idx != idx {
                            break;
                        }
                        visitor.visit_impl_static(impl_ref.clone(), impl_ctx.clone(), node, ctx);
                        static_offset += 1;
                    }
                }

                if flags.contains(VisitFlags::ImplTlsStatic) {
                    for (impl_idx, node, ctx) in &mut hir.impl_tls_statics[tls_static_offset..] {
                        if *impl_idx != idx {
                            break;
                        }
                        visitor.visit_impl_tls_static(impl_ref.clone(), impl_ctx.clone(), node, ctx);
                        tls_static_offset += 1;
                    }
                }
            
                if flags.contains(VisitFlags::Property) {
                    for (impl_idx, node, ctx) in &mut hir.properties[prop_offset..] {
                        if *impl_idx != idx {
                            break;
                        }
                        visitor.visit_property(impl_ref.clone(), impl_ctx.clone(), node, ctx);
                        prop_offset += 1;
                    }
                }
            }
        }

        if flags.intersects(VisitFlags::AnyOpTrait) {
            let mut func_offset = 0;
            let mut spec_offset = 0;
            let mut contract_offset = 0;

            for (idx, (trait_ref, trait_ctx)) in hir.op_traits.iter_mut().enumerate() {
                if flags.contains(VisitFlags::OpTrait) {
                    let mut node = trait_ref.write();
                    let mut ctx = trait_ctx.write();
                    visitor.visit_op_trait(&mut node, &mut ctx);
                }
                
                if flags.contains(VisitFlags::OpFunction) {
                    for (trait_idx, node, ctx) in &mut hir.op_functions[func_offset..] {
                        if *trait_idx != idx {
                            break;
                        }
                        visitor.visit_op_function(trait_ref.clone(), trait_ctx.clone(), node, ctx);
                        func_offset += 1;
                    }
                }
                
                if flags.contains(VisitFlags::OpContract) {
                    for (trait_idx, node, ctx) in &mut hir.op_contracts[contract_offset..] {
                        if *trait_idx != idx {
                            break;
                        }
                        visitor.visit_op_contract(trait_ref.clone(), trait_ctx.clone(), node, ctx);
                        contract_offset += 1;
                    }
                }
            }
        }
        
        if flags.contains(VisitFlags::Precedence) {
            for (node, ctx) in &mut hir.precedences {
                visitor.visit_precedence(node, ctx.clone());
            }
        }
    }

    pub fn visit_fn_param<T: Visitor>(visitor: &mut T, param: &mut FnParam) {
        match param {
            FnParam::Param { attrs, label, pattern, ty, .. } => {
                for attr in attrs {
                    visitor.visit_attribute(attr);
                }
                visitor.visit_pattern(pattern);
                visitor.visit_type(ty);
            },
            FnParam::Opt { attrs, label, pattern, ty, def, .. } => {
                for attr in attrs {
                    visitor.visit_attribute(attr);
                }
                visitor.visit_pattern(pattern);
                visitor.visit_type(ty);
                visitor.visit_expr(def);
            },
            FnParam::Variadic { attrs, name, ty, .. } => {
                for attr in attrs {
                    visitor.visit_attribute(attr);
                }
                visitor.visit_type(ty);
            },
        }
    }

    pub fn visit_path<T: Visitor>(visitor: &mut T, path: &mut Path) {
        if let PathStart::Type { ty } = &mut path.start {
            visitor.visit_type(ty);
        }
        for iden in &mut path.idens {
            visit_iden(visitor, iden);
        }
        if let Some(fn_end) = &mut path.fn_end {
            for (_, ty) in &mut fn_end.params {
                visitor.visit_type(ty);
            }
            if let Some(ret_ty) = &mut fn_end.ret_ty {
                visitor.visit_type(ret_ty);
            }
        }
    }

    pub fn visit_iden<T: Visitor>(visitor: &mut T, iden: &mut Identifier) {
        if let IdenName::Disambig { trait_path, .. } = &mut iden.name {
            visitor.visit_path(trait_path);
        }

        if let Some(gen_args) = &mut iden.gen_args {
            visitor.visit_gen_args(gen_args);
        }
    }

    // =============================================================

    pub fn visit_function<T: Visitor>(visitor: &mut T, node: &mut Function) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }

        if let Some(generics) = &mut node.generics {
            visitor.visit_gen_params(generics);
        }

        for param in &mut node.params {
            helpers::visit_fn_param(visitor, param);
        }

        if let Some(ret_ty) = &mut node.return_ty {
            visitor.visit_type(ret_ty);
        }
        if let Some(where_clause) = &mut node.where_clause {
            visitor.visit_where_clause(where_clause);
        }

        for contract in &mut node.contracts {
            visitor.visit_contract(contract);
        }

        visitor.visit_block(&mut node.body);
    }

    pub fn visit_method<T: Visitor>(visitor: &mut T, node: &mut Method) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }

        if let Some(generics) = &mut node.generics {
            visitor.visit_gen_params(generics);
        }

        match &mut node.receiver {
            FnReceiver::None => (),
            FnReceiver::SelfReceiver { .. } => (),
            FnReceiver::SelfTyped { ty, .. } => visitor.visit_type(ty),
        }

        for param in &mut node.params {
            helpers::visit_fn_param(visitor, param);
        }

        if let Some(ret_ty) = &mut node.return_ty {
            visitor.visit_type(ret_ty);
        }
        if let Some(where_clause) = &mut node.where_clause {
            visitor.visit_where_clause(where_clause);
        }

        for contract in &mut node.contracts {
            visitor.visit_contract(contract);
        }

        visitor.visit_block(&mut node.body);
    }

    pub fn visit_extern_function<T: Visitor>(visitor: &mut T, node: &mut ExternFunctionNoBody) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }

        if let Some(generics) = &mut node.generics {
            visitor.visit_gen_params(generics);
        }

        for param in &mut node.params {
            helpers::visit_fn_param(visitor, param);
        }

        if let Some(ret_ty) = &mut node.return_ty {
            visitor.visit_type(ret_ty);
        }
        if let Some(where_clause) = &mut node.where_clause {
            visitor.visit_where_clause(where_clause);
        }

        for contract in &mut node.contracts {
            visitor.visit_contract(contract);
        }
    }

    pub fn visit_type_alias<T: Visitor>(visitor: &mut T, node: &mut TypeAlias) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(generics) = &mut node.generics {
            visitor.visit_gen_params(generics);
        }

        visitor.visit_type(&mut node.ty);
    }

    pub fn visit_distinct_type<T: Visitor>(visitor: &mut T, node: &mut DistinctType) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(generics) = &mut node.generics {
            visitor.visit_gen_params(generics);
        }
        visitor.visit_type(&mut node.ty);
    }

    pub fn visit_opaque_type<T: Visitor>(visitor: &mut T, node: &mut OpaqueType) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(size) = &mut node.size {    
            visitor.visit_expr(size);
        }
    }

    pub fn visit_struct<T: Visitor>(visitor: &mut T, node: &mut Struct) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(generics) = &mut node.generics {
            visitor.visit_gen_params(generics);
        }
        if let Some(where_clause) = &mut node.where_clause {
            visitor.visit_where_clause(where_clause);
        }

        for field in &mut node.fields {
            visit_struct_field(visitor, field);
        }

        for struct_use in &mut node.uses {
            for attr in &mut struct_use.attrs {
                visitor.visit_attribute(attr);
            }
            visitor.visit_path(&mut struct_use.path);
        }
    }

    pub fn visit_struct_field<T: Visitor>(visitor: &mut T, field: &mut StructField) {
        for attr in &mut field.attrs {
            visitor.visit_attribute(attr);
        }
        visitor.visit_type(&mut field.ty);
        if let Some(def) = &mut field.def {
            visitor.visit_expr(def);
        }
    }

    pub fn visit_tuple_struct<T: Visitor>(visitor: &mut T, node: &mut TupleStruct) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(generics) = &mut node.generics {
            visitor.visit_gen_params(generics);
        }
        if let Some(where_clause) = &mut node.where_clause {
            visitor.visit_where_clause(where_clause);
        }

        for field in &mut node.fields {
            visit_tuple_struct_field(visitor, field);
        }
    }

    pub fn visit_tuple_struct_field<T: Visitor>(visitor: &mut T, field: &mut TupleStructField) {
        for attr in &mut field.attrs {
            visitor.visit_attribute(attr);
        }
        visitor.visit_type(&mut field.ty);
        if let Some(def) = &mut field.def {
            visitor.visit_expr(def);
        }
    }

    pub fn visit_unit_struct<T: Visitor>(visitor: &mut T, node: &mut UnitStruct) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }
    }

    pub fn visit_union<T: Visitor>(visitor: &mut T, node: &mut Union) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(generics) = &mut node.generics {
            visitor.visit_gen_params(generics);
        }
        if let Some(where_clause) = &mut node.where_clause {
            visitor.visit_where_clause(where_clause);
        }

        for fields in &mut node.fields {
            for attr in &mut fields.attrs {
                visitor.visit_attribute(attr);
            }
            visitor.visit_type(&mut fields.ty);
        }
    }

    pub fn visit_adt_enum<T: Visitor>(visitor: &mut T, node: &mut AdtEnum) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(generics) = &mut node.generics {
            visitor.visit_gen_params(generics);
        }
        if let Some(where_clause) = &mut node.where_clause {
            visitor.visit_where_clause(where_clause);
        }

        for variant in &mut node.variants {
            match variant {
                AdtEnumVariant::Struct { attrs, is_mut, name, fields, discriminant, .. } => {
                    for attr in attrs {
                        visitor.visit_attribute(attr);
                    }
                    for field in fields {
                        helpers::visit_struct_field(visitor, field);
                    }
                    if let Some(discriminant) = discriminant {
                        visitor.visit_expr(discriminant);
                    }
                },
                AdtEnumVariant::Tuple { attrs, is_mut, name, fields, discriminant, .. } => {
                    for attr in attrs {
                        visitor.visit_attribute(attr);
                    }
                    for field in fields {
                        helpers::visit_tuple_struct_field(visitor, field);
                    }
                    if let Some(discriminant) = discriminant {
                        visitor.visit_expr(discriminant);
                    }
                },
                AdtEnumVariant::Fieldless { attrs, name, discriminant, .. } => {
                    for attr in attrs {
                        visitor.visit_attribute(attr);
                    }
                    if let Some(discriminant) = discriminant {
                        visitor.visit_expr(discriminant);
                    }
                },
            }
        }
    }

    pub fn visit_flag_enum<T: Visitor>(visitor: &mut T, node: &mut FlagEnum) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }
        for variant in &mut node.variants {
            for attr in &mut variant.attrs {
                visitor.visit_attribute(attr);
            }
            if let Some(discriminant) = &mut variant.discriminant {
                visitor.visit_expr(discriminant);
            }
        }
    }

    pub fn visit_bitfield<T: Visitor>(visitor: &mut T, node: &mut Bitfield) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(generics) = &mut node.generics {
            visitor.visit_gen_params(generics);
        }
        if let Some(where_clause) = &mut node.where_clause {
            visitor.visit_where_clause(where_clause);
        }

        for field in &mut node.fields {
            for attr in &mut field.attrs {
                visitor.visit_attribute(attr);
            }

            visitor.visit_type(&mut field.ty);
            if let Some(bits) = &mut field.bits {
                visitor.visit_expr(bits);
            }
            if let Some(def) = &mut field.def {
                visitor.visit_expr(def);
            }
        }

        for bf_use in &mut node.uses {
            for attr in &mut bf_use.attrs {
                visitor.visit_attribute(attr);
            }
            visitor.visit_path(&mut bf_use.path);
            if let Some(bits) = &mut bf_use.bits {
                visitor.visit_expr(bits);
            }
        }
    }

    pub fn visit_const<T: Visitor>(visitor: &mut T, node: &mut Const) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(ty) = &mut node.ty {
            visitor.visit_type(ty);
        }
        visitor.visit_expr(&mut node.val);
    }

    pub fn visit_static<T: Visitor>(visitor: &mut T, node: &mut Static) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(ty) = &mut node.ty {
            visitor.visit_type(ty);
        }
        visitor.visit_expr(&mut node.val);
    }

    pub fn visit_tls_static<T: Visitor>(visitor: &mut T, node: &mut TlsStatic) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(ty) = &mut node.ty {
            visitor.visit_type(ty);
        }
        visitor.visit_expr(&mut node.val);
    }

    pub fn visit_extern_static<T: Visitor>(visitor: &mut T, node: &mut ExternStatic) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }
        visitor.visit_type(&mut node.ty);
    }

    pub fn visit_propety<T: Visitor>(visitor: &mut T, node: &mut Property) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(expr) = &mut node.get {
            visitor.visit_expr(expr);
        }
        if let Some(expr) = &mut node.ref_get {
            visitor.visit_expr(expr);
        }
        if let Some(expr) = &mut node.mut_get {
            visitor.visit_expr(expr);
        }
        if let Some(expr) = &mut node.set {
            visitor.visit_expr(expr);
        }
    }

    //--------------------------------------------------------------

    pub fn visit_trait<T: Visitor>(visitor: &mut T, node: &mut Trait) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(generics) = &mut node.generics {
            visitor.visit_gen_params(generics);
        }
        if let Some(bounds) = &mut node.bounds {
            visitor.visit_trait_bounds(bounds);
        }
    }

    pub fn visit_trait_function<T: Visitor>(visitor: &mut T, node: &mut TraitFunction) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }

        if let Some(generics) = &mut node.generics {
            visitor.visit_gen_params(generics);
        }

        for param in &mut node.params {
            helpers::visit_fn_param(visitor, param);
        }

        if let Some(ret_ty) = &mut node.return_ty {
            visitor.visit_type(ret_ty);
        }
        if let Some(where_clause) = &mut node.where_clause {
            visitor.visit_where_clause(where_clause);
        }

        for contract in &mut node.contracts {
            visitor.visit_contract(contract);
        }

        if let Some(body) = &mut node.body {
            visitor.visit_block(body);
        }
    }

    pub fn visit_trait_method<T: Visitor>(visitor: &mut T, node: &mut TraitMethod) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }

        if let Some(generics) = &mut node.generics {
            visitor.visit_gen_params(generics);
        }

        match &mut node.receiver {
            FnReceiver::None => (),
            FnReceiver::SelfReceiver { .. } => (),
            FnReceiver::SelfTyped { ty, .. } => visitor.visit_type(ty),
        }

        for param in &mut node.params {
            helpers::visit_fn_param(visitor, param);
        }

        if let Some(ret_ty) = &mut node.return_ty {
            visitor.visit_type(ret_ty);
        }
        if let Some(where_clause) = &mut node.where_clause {
            visitor.visit_where_clause(where_clause);
        }

        for contract in &mut node.contracts {
            visitor.visit_contract(contract);
        }

        if let Some(body) = &mut node.body {
            visitor.visit_block(body);
        }
    }

    pub fn visit_trait_type_alias<T: Visitor>(visitor: &mut T, node: &mut TraitTypeAlias) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(def) = &mut node.def {
            visitor.visit_type(def);
        }
    }

    pub fn visit_trait_const<T: Visitor>(visitor: &mut T, node: &mut TraitConst) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }
        visitor.visit_type(&mut node.ty);
        if let Some(def) = &mut node.def {
            visitor.visit_expr(def);
        }
    }

    pub fn visit_trait_property<T: Visitor>(visitor: &mut T, node: &mut TraitProperty) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }

        match &mut node.get {
            TraitPropertyMember::Def(_, expr) => visitor.visit_expr(expr),
            _ => (),
        }
        match &mut node.ref_get {
            TraitPropertyMember::Def(_, expr) => visitor.visit_expr(expr),
            _ => (),
        }
        match &mut node.mut_get {
            TraitPropertyMember::Def(_, expr) => visitor.visit_expr(expr),
            _ => (),
        }
        match &mut node.set {
            TraitPropertyMember::Def(_, expr) => visitor.visit_expr(expr),
            _ => (),
        }
    }

    //--------------------------------------------------------------

    pub fn visit_impl<T: Visitor>(visitor: &mut T, node: &mut Impl) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }
    }

    //--------------------------------------------------------------

    pub fn visit_op_trait<T: Visitor>(visitor: &mut T, node: &mut OpTrait, ctx: &mut OpTraitContext) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }
        for base in &mut node.bases {
            visitor.visit_simple_path(base);
        }
    }

    pub fn visit_op_function<T: Visitor>(visitor: &mut T, node: &mut OpFunction, ctx: &mut OpFunctionContext) {
        if let Some(ret_ty) = &mut node.ret_ty {
            visitor.visit_type(ret_ty);
        }
        if let Some(def) = &mut node.def {
            visitor.visit_expr(def);
        }
    }

    pub fn visit_op_contract<T: Visitor>(visitor: &mut T, node: &mut OpContract, ctx: &mut OpContractContext) {
        visitor.visit_expr(&mut node.expr);
    }

    //--------------------------------------------------------------

    pub fn visit_precedence<T: Visitor>(visitor: &mut T, node: &mut Precedence) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }
    }

    // =============================================================

    pub fn visit_block<T: Visitor>(visitor: &mut T, node: &mut Block) {
        for stmt in &mut node.stmts {
            visitor.visit_stmt(stmt);
        }
        if let Some(expr) = &mut node.expr {
            visitor.visit_expr(expr);
        }
    }

    // =============================================================

    pub fn visit_stmt<T: Visitor>(visitor: &mut T, node: &mut Stmt) {
        match node {
            Stmt::VarDecl(stmt)       => visitor.visit_var_decl(stmt),
            Stmt::UninitVarDecl(stmt) => visitor.visit_uninit_var_decl(stmt),
            Stmt::Defer(stmt)         => visitor.visit_defer_stmt(stmt),
            Stmt::ErrDefer(stmt)      => visitor.visit_err_defer_stmt(stmt),
            Stmt::Expr(stmt)          => visitor.visit_expr_stmt(stmt),
        }
    }

    pub fn visit_var_decl<T: Visitor>(visitor: &mut T, node: &mut VarDecl) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }
        if let Some(ty) = &mut node.ty {
            visitor.visit_type(ty);
        }
        visitor.visit_expr(&mut node.expr);
    }

    pub fn visit_uninit_var_decl<T: Visitor>(visitor: &mut T, node: &mut UninitVarDecl) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }
        visitor.visit_type(&mut node.ty);
    }

    pub fn visit_defer_stmt<T: Visitor>(visitor: &mut T, node: &mut DeferStmt) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }
        visitor.visit_expr(&mut node.expr);
    }
    
    pub fn visit_err_defer_stmt<T: Visitor>(visitor: &mut T, node: &mut ErrorDeferStmt) {
        for attr in &mut node.attrs {
            visitor.visit_attribute(attr);
        }
        visitor.visit_expr(&mut node.expr);
    }

    pub fn visit_expr_stmt<T: Visitor>(visitor: &mut T, node: &mut ExprStmt) {
        visitor.visit_expr(&mut node.expr);
    }

    // =============================================================

    pub fn visit_expr<T: Visitor>(visitor: &mut T, node: &mut Expr) {
        match node {
            Expr::Unit(expr)        => visitor.visit_unit_expr(expr),
            Expr::FullRange(expr)   => visitor.visit_fullrange_expr(expr),
            Expr::Underscore(expr)  => visitor.visit_underscore_expr(expr),
            Expr::Literal(expr)     => visitor.visit_literal_expr(expr),
            Expr::Path(expr)        => visitor.visit_path_expr(expr),
            Expr::Block(expr)       => visitor.visit_block_expr(expr),
            Expr::Prefix(expr)      => visitor.visit_prefix_expr(expr),
            Expr::Postfix(expr)     => visitor.visit_postfix_expr(expr),
            Expr::Infix(expr)       => visitor.visit_infix_expr(expr),
            Expr::Inplace(expr)     => visitor.visit_inplace_expr(expr),
            Expr::TypeCast(expr)    => visitor.visit_type_cast_expr(expr),
            Expr::TypeCheck(expr)   => visitor.visit_type_check_expr(expr),
            Expr::Tuple(expr)       => visitor.visit_tuple_expr(expr),
            Expr::Slice(expr)       => visitor.visit_slice_expr(expr),
            Expr::Array(expr)       => visitor.visit_array_expr(expr),
            Expr::Struct(expr)      => visitor.visit_struct_expr(expr),
            Expr::Index(expr)       => visitor.visit_index_expr(expr),
            Expr::TupleIndex(expr)  => visitor.visit_tuple_index_expr(expr),
            Expr::FnCall(expr)      => visitor.visit_fn_call_expr(expr),
            Expr::MethodCall(expr)  => visitor.visit_method_call_expr(expr),
            Expr::FieldAccess(expr) => visitor.visit_field_access_expr(expr),
            Expr::Closure(expr)     => visitor.visit_closure_expr(expr),
            Expr::Loop(expr)        => visitor.visit_loop_expr(expr),
            Expr::Match(expr)       => visitor.visit_match_expr(expr),
            Expr::Break(expr)       => visitor.visit_break_expr(expr),
            Expr::Continue(expr)    => visitor.visit_continue_expr(expr),
            Expr::Fallthrough(expr) => visitor.visit_fallthrough_expr(expr),
            Expr::Return(expr)      => visitor.visit_return_expr(expr),
            Expr::Throw(expr)       => visitor.visit_throw_expr(expr),
            Expr::Comma(expr)       => visitor.visit_comma_expr(expr),
            Expr::When(expr)        => visitor.visit_when_expr(expr),
            Expr::Irrefutable       => visitor.visit_irrefutable_expr(),
        }
    }

    pub fn visit_path_expr<T: Visitor>(visitor: &mut T, node: &mut PathExpr) {
        match node {
            PathExpr::Named { iden, .. } => {
                if let Some(gen_args) = &mut iden.gen_args {
                    visitor.visit_gen_args(gen_args)
                }
            },
            PathExpr::SelfPath{ .. } => (),
            PathExpr::Expanded { path } => visitor.visit_path(path),
        }
    }

    pub fn visit_block_expr<T: Visitor>(visitor: &mut T, node: &mut BlockExpr) {
        visitor.visit_block(&mut node.block);
    }

    pub fn visit_prefix_expr<T: Visitor>(visitor: &mut T, node: &mut PrefixExpr) {
        visitor.visit_expr(&mut node.expr);
    }

    pub fn visit_postfix_expr<T: Visitor>(visitor: &mut T, node: &mut PostfixExpr) {
        visitor.visit_expr(&mut node.expr);
    }

    pub fn visit_infix_expr<T: Visitor>(visitor: &mut T, node: &mut InfixExpr) {
        visitor.visit_expr(&mut node.left);
        visitor.visit_expr(&mut node.right);
    }

    pub fn visit_inplace_expr<T: Visitor>(visitor: &mut T, node: &mut InplaceExpr) {
        visitor.visit_expr(&mut node.left);
        visitor.visit_expr(&mut node.right);
    }

    pub fn visit_type_cast_expr<T: Visitor>(visitor: &mut T, node: &mut TypeCastExpr) {
        visitor.visit_expr(&mut node.expr);
        visitor.visit_type(&mut node.ty);
    }

    pub fn visit_type_check_expr<T: Visitor>(visitor: &mut T, node: &mut TypeCheckExpr) {
        visitor.visit_expr(&mut node.expr);
        visitor.visit_type(&mut node.ty);
    }

    pub fn visit_tuple_expr<T: Visitor>(visitor: &mut T, node: &mut TupleExpr) {
        for expr in &mut node.exprs {
            visitor.visit_expr(expr);
        }
    }

    pub fn visit_struct_expr<T: Visitor>(visitor: &mut T, node: &mut StructExpr) {
        visitor.visit_expr(&mut node.path);
        for arg in &mut node.args {
            visitor.visit_expr(&mut arg.expr);
        }
        if let Some(complete) = &mut node.complete {
            visitor.visit_expr(complete);
        }
    }

    pub fn visit_slice_expr<T: Visitor>(visitor: &mut T, node: &mut SliceExpr) {
        for expr in &mut node.exprs {
            visitor.visit_expr(expr);
        }
    }

    pub fn visit_array_expr<T: Visitor>(visitor: &mut T, node: &mut ArrayExpr) {
        visitor.visit_expr(&mut node.value);
        visitor.visit_expr(&mut node.count);
    }

    pub fn visit_index_expr<T: Visitor>(visitor: &mut T, node: &mut IndexExpr) {
        visitor.visit_expr(&mut node.expr);
        visitor.visit_expr(&mut node.index);
    }

    pub fn visit_tuple_index_expr<T: Visitor>(visitor: &mut T, node: &mut TupleIndexExpr) {
        visitor.visit_expr(&mut node.expr);
    }

    pub fn visit_fn_call_expr<T: Visitor>(visitor: &mut T, node: &mut FnCallExpr) {
        visitor.visit_expr(&mut node.func);
        for arg in &mut node.args {
            visitor.visit_expr(&mut arg.expr);
        }
    }

    pub fn visit_method_call_expr<T: Visitor>(visitor: &mut T, node: &mut MethodCallExpr) {
        visitor.visit_expr(&mut node.receiver);
        visit_iden(visitor, &mut node.method);
        for arg in &mut node.args {
            visitor.visit_expr(&mut arg.expr);
        }
    }

    pub fn visit_field_access_expr<T: Visitor>(visitor: &mut T, node: &mut FieldAccessExpr) {
        visitor.visit_expr(&mut node.expr);
        visit_iden(visitor, &mut node.field);
    }

    pub fn visit_closure_expr<T: Visitor>(visitor: &mut T, node: &mut ClosureExpr) {
        for param in &mut node.params {
            visit_fn_param(visitor, param);
        }
        if let Some(ret) = &mut node.ret {
            visitor.visit_type(ret);
        }
        visitor.visit_expr(&mut node.body);
    }

    pub fn visit_loop_expr<T: Visitor>(visitor: &mut T, node: &mut LoopExpr) {
        visitor.visit_block(&mut node.body);
    }

    pub fn visit_match_expr<T: Visitor>(visitor: &mut T, node: &mut MatchExpr) {
        visitor.visit_expr(&mut node.scrutinee);
        for branch in &mut node.branches {
            visitor.visit_pattern(&mut branch.pattern);
            if let Some(guard) = &mut branch.guard {
                visitor.visit_expr(guard);
            }
            visitor.visit_expr(&mut branch.body);
        }
    }

    pub fn visit_break_expr<T: Visitor>(visitor: &mut T, node: &mut BreakExpr) {
        if let Some(value) = &mut node.value {
            visitor.visit_expr(value);
        }
    }

    pub fn visit_return_expr<T: Visitor>(visitor: &mut T, node: &mut ReturnExpr) {
        if let Some(value) = &mut node.value {
            visitor.visit_expr(value);
        }
    }

    pub fn visit_throw_expr<T: Visitor>(visitor: &mut T, node: &mut ThrowExpr) {
        visitor.visit_expr(&mut node.expr);
    }

    pub fn visit_comma_expr<T: Visitor>(visitor: &mut T, node: &mut CommaExpr) {
        for expr in &mut node.exprs {
            visitor.visit_expr(expr);
        }
    }

    pub fn visit_when_expr<T: Visitor>(visitor: &mut T, node: &mut WhenExpr) {
        visitor.visit_expr(&mut node.cond);
        visitor.visit_block(&mut node.body);
        if let Some(else_body) = &mut node.else_body {
            visitor.visit_block(else_body);
        }
    }


    // =============================================================

    pub fn visit_pattern<T: Visitor>(visitor: &mut T, node: &mut Pattern) {
        match node {
            Pattern::Wildcard(pattern)    => visitor.visit_wildcard_pattern(pattern),
            Pattern::Rest(pattern)        => visitor.visit_rest_pattern(pattern),
            Pattern::Literal(pattern)     => visitor.visit_literal_pattern(pattern),
            Pattern::Iden(pattern)        => visitor.visit_iden_pattern(pattern),
            Pattern::Path(pattern)        => visitor.visit_path_pattern(pattern),
            Pattern::Range(pattern)       => visitor.visit_range_pattern(pattern),
            Pattern::Reference(pattern)   => visitor.visit_reference_pattern(pattern),
            Pattern::Struct(pattern)      => visitor.visit_struct_pattern(pattern),
            Pattern::TupleStruct(pattern) => visitor.visit_tuple_struct_pattern(pattern),
            Pattern::Tuple(pattern)       => visitor.visit_tuple_pattern(pattern),
            Pattern::Slice(pattern)       => visitor.visit_slice_pattern(pattern),
            Pattern::EnumMember(pattern)  => visitor.visit_enum_member_pattern(pattern),
            Pattern::Alternative(pattern) => visitor.visit_alternative_pattern(pattern),
            Pattern::TypeCheck(pattern)   => visitor.visit_type_check_pattern(pattern),
        }
    }

    pub fn visit_iden_pattern<T: Visitor>(visitor: &mut T, node: &mut IdenPattern) {
        if let Some(bound) = &mut node.bound {
            visitor.visit_pattern(bound);
        }
    }

    pub fn visit_path_pattern<T: Visitor>(visitor: &mut T, node: &mut PathPattern) {
        visitor.visit_path(&mut node.path);
    }

    pub fn visit_range_pattern<T: Visitor>(visitor: &mut T, node: &mut RangePattern) {
        match node {
            RangePattern::Exclusive { begin, end, .. } => {
                visitor.visit_pattern(begin);
                visitor.visit_pattern(end);
            },
            RangePattern::Inclusive { begin, end, .. } => {
                visitor.visit_pattern(begin);
                visitor.visit_pattern(end);
            },
            RangePattern::From { begin, .. } => {
                visitor.visit_pattern(begin);
            },
            RangePattern::To { end, .. } => {
                visitor.visit_pattern(end);
            },
            RangePattern::InclusiveTo { end, .. } => {
                visitor.visit_pattern(end);
            },
        }
    }

    pub fn visit_reference_pattern<T: Visitor>(visitor: &mut T, node: &mut ReferencePattern) {
        visitor.visit_pattern(&mut node.pattern);
    }

    pub fn visit_struct_pattern<T: Visitor>(visitor: &mut T, node: &mut StructPattern) {
        if let Some(path) = &mut node.path {
            visitor.visit_path(path);
        }
        for field in &mut node.fields {
            match field {
                StructPatternField::Named { pattern, .. } => {
                    visitor.visit_pattern(pattern);
                },
                StructPatternField::TupleIndex { pattern, .. } => {
                    visitor.visit_pattern(pattern);
                },
                StructPatternField::Iden { bound, .. } => if let Some(bound) = bound {
                    visitor.visit_pattern(bound);
                },
                StructPatternField::Rest => {},
            }
        }
    }

    pub fn visit_tuple_struct_pattern<T: Visitor>(visitor: &mut T, node: &mut TupleStructPattern) {
        if let Some(path) = &mut node.path {
            visitor.visit_path(path);
        }
        for pattern in &mut node.patterns {
            visitor.visit_pattern(pattern);
        }
    }

    pub fn visit_tuple_pattern<T: Visitor>(visitor: &mut T, node: &mut TuplePattern) {
        for pattern in &mut node.patterns {
            visitor.visit_pattern(pattern);
        }
    }

    pub fn visit_slice_pattern<T: Visitor>(visitor: &mut T, node: &mut SlicePattern) {
        for pattern in &mut node.patterns {
            visitor.visit_pattern(pattern);
        }
    }

    pub fn visit_alternative_pattern<T: Visitor>(visitor: &mut T, node: &mut AlternativePattern) {
        for pattern in &mut node.patterns {
            visitor.visit_pattern(pattern);
        }
    }

    pub fn visit_type_check_pattern<T: Visitor>(visitor: &mut T, node: &mut TypeCheckPattern) {
        visitor.visit_type(&mut node.ty);
    }

    // =============================================================

    pub fn visit_type<T: Visitor>(visitor: &mut T, node: &mut Type) {
        match node {
            Type::Unit(ty)        => visitor.visit_unit_type(ty),
            Type::Never(ty)       => visitor.visit_never_type(ty),
            Type::Primitive(ty)   => visitor.visit_primitive_type(ty),
            Type::Path(ty)        => visitor.visit_path_type(ty),
            Type::Tuple(ty)       => visitor.visit_tuple_type(ty),
            Type::Array(ty)       => visitor.visit_array_type(ty),
            Type::Slice(ty)       => visitor.visit_slice_type(ty),
            Type::StringSlice(ty) => visitor.visit_string_slice_type(ty),
            Type::Pointer(ty)     => visitor.visit_pointer_type(ty),
            Type::Reference(ty)   => visitor.visit_reference_type(ty),
            Type::Optional(ty)    => visitor.visit_optional_type(ty),
            Type::Fn(ty)          => visitor.visit_fn_type(ty),
        }
    }

    pub fn visit_path_type<T: Visitor>(visitor: &mut T, node: &mut PathType) {
        visitor.visit_path(&mut node.path);
    }

    pub fn visit_tuple_type<T: Visitor>(visitor: &mut T, node: &mut TupleType) {
        for ty in &mut node.types {
            visitor.visit_type(ty);
        }
    }

    pub fn visit_array_type<T: Visitor>(visitor: &mut T, node: &mut ArrayType) {
        visitor.visit_expr(&mut node.size);
        if let Some(sentinel) = &mut node.sentinel {
            visitor.visit_expr(sentinel);
        }
        visitor.visit_type(&mut node.ty);
    } 

    pub fn visit_slice_type<T: Visitor>(visitor: &mut T, node: &mut SliceType) {
        if let Some(sentinel) = &mut node.sentinel {
            visitor.visit_expr(sentinel);
        }
        visitor.visit_type(&mut node.ty);
    }
    
    pub fn visit_pointer_type<T: Visitor>(visitor: &mut T, node: &mut PointerType) {
        visitor.visit_type(&mut node.ty);
        if let Some(sentinel) = &mut node.sentinel {
            visitor.visit_expr(sentinel);
        }
    }

    pub fn visit_reference_type<T: Visitor>(visitor: &mut T, node: &mut ReferenceType) {
        visitor.visit_type(&mut node.ty);
    }

    pub fn visit_optional_type<T: Visitor>(visitor: &mut T, node: &mut OptionalType) {
        visitor.visit_type(&mut node.ty);
    }

    pub fn visit_fn_type<T: Visitor>(visitor: &mut T, node: &mut FnType) {
        for (_, ty) in &mut node.params {
            visitor.visit_type(ty);
        }
        if let Some(ty) = &mut node.return_ty {
            visitor.visit_type(ty);
        }
    }
    
    // =============================================================

    pub fn visit_gen_params<T: Visitor>(visitor: &mut T, node: &mut GenericParams) {
        for param in &mut node.params {
            match param {
                GenericParam::Type(param) => {
                    if let Some(def) = &mut param.def {
                        visitor.visit_type(def);
                    }
                },
                GenericParam::TypeSpec(param) => {
                    visitor.visit_type(&mut param.ty)
                },
                GenericParam::Const(param) => {
                    visitor.visit_type(&mut param.ty);
                    if let Some(def) = &mut param.def {
                        visitor.visit_expr(def);
                    }
                },
                GenericParam::ConstSpec(param) => {
                    visitor.visit_block (&mut param.expr);
                },
            }
        }
        if let Some(pack) = &mut node.pack {
            for elem in &mut pack.elems {
                match elem {
                    GenericParamPackElem::Type { defs, ..} => {
                        for def in defs {
                            visitor.visit_type(def);
                        }
                    },
                    GenericParamPackElem::Const { ty, defs, .. } => {
                        visitor.visit_type(ty);
                        for def in defs {
                            visitor.visit_expr(def);
                        }
                    },
                }
            }
        }
    }

    pub fn visit_gen_args<T: Visitor>(visitor: &mut T, node: &mut GenericArgs) {
        for arg in &mut node.args {
            match arg {
                GenericArg::Type(ty) => visitor.visit_type(ty),
                GenericArg::Value(expr) => visitor.visit_expr(expr),
                GenericArg::Name(_, _) => (),
            }
        }
    }

    pub fn visit_where_clause<T: Visitor>(visitor: &mut T, node: &mut WhereClause) {
        for where_bound in &mut node.bounds {
            match where_bound {
                WhereBound::Type { span, ty, bounds } => {
                    visitor.visit_type(ty);
                    for bound in bounds {
                        visitor.visit_path(bound);
                    }
                },
                WhereBound::Explicit { span, ty, bounds } => {
                    visitor.visit_type(ty);
                    for bound in bounds {
                        visitor.visit_type(bound);
                    }
                },
                WhereBound::Expr { expr } => visitor.visit_expr(expr),
            }
        }
    }

    pub fn visit_trait_bounds<T: Visitor>(visitor: &mut T, node: &mut TraitBounds) {
        for bound in &mut node.bounds {
            visitor.visit_path(bound);
        }
    }
    
    // =============================================================

}