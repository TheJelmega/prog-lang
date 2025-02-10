// High level intermediate representation
//
// Represented as nodes with internal tree structures per module-level item
#![allow(unused)]

use std::{fmt, sync::Arc};
use parking_lot::RwLock;

use crate::{
    common::{Abi, NameId, OpType, Scope, SpanId, SymbolRef},
    error_warning::{HirErrorCode, LexErrorCode},
    lexer::Punctuation,
    literals::LiteralId,
    ast,
    type_system,
};

mod visitor;
pub use visitor::*;

pub mod utils;

mod node_logger;
pub use node_logger::*;

mod code_printer;
pub use code_printer::*;

pub mod passes;
pub use passes::Pass;

// =============================================================================================================================

pub struct HirError {
    pub node_id: ast::NodeId,
    pub err:     HirErrorCode
}

impl fmt::Display for HirError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.node_id, self.err)
    }
}

// =============================================================================================================================

#[derive(Clone)]
pub enum TypePathSegment {
    Plain {
        span:     SpanId,
        name:     NameId
    },
    GenArg {
        span:     SpanId,
        name:     NameId,
        gen_args: Box<GenericArgs>,
    },
    Fn {
        span:     SpanId,
        name:     NameId,
        params:   Vec<Box<Type>>,
        ret:      Option<Box<Type>>,
    }
}

#[derive(Clone)]
pub struct TypePath {
    pub span:     SpanId,
    pub node_id:  ast::NodeId,
    pub segments: Vec<TypePathSegment>
}

#[derive(Clone)]
pub struct SimplePath {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub names:   Vec<NameId>,
}

#[derive(Clone)]
pub struct Identifier {
    pub span:     SpanId,
    pub name:     NameId,
    pub gen_args: Option<Box<GenericArgs>>,
}

#[derive(Clone)]
pub struct Path {
    pub span:        SpanId,
    pub node_id:     ast::NodeId,
    pub is_inferred: bool,
    pub idens:       Vec<Identifier>,
}

#[derive(Clone)]
pub struct QualifiedPath {
    pub span:     SpanId,
    pub node_id:  ast::NodeId,
    pub ty:       Box<Type>,
    pub bound:    Option<TypePath>,
    pub sub_path: Vec<Identifier>,
}

// =============================================================================================================================

pub struct Function {
    pub span:         SpanId,
    pub node_id:      ast::NodeId,
    pub attrs:        Vec<Box<Attribute>>,
    pub vis:          Visibility,
    pub is_const:     bool,
    pub is_unsafe:    bool,
    pub abi:          Abi,
    pub name:         NameId,
    pub generics:     Option<Box<GenericParams>>,
    pub params:       Vec<FnParam>,
    pub return_ty:    Option<Box<Type>>,
    pub where_clause: Option<Box<WhereClause>>,
    pub contracts:    Vec<Box<Contract>>,
    pub body:         Box<Block>,
}

pub struct Method {
    pub span:         SpanId,
    pub node_id:      ast::NodeId,
    pub attrs:        Vec<Box<Attribute>>,
    pub vis:          Visibility,
    pub is_const:     bool,
    pub is_unsafe:    bool,
    pub name:         NameId,
    pub generics:     Option<Box<GenericParams>>,
    pub receiver:     FnReceiver,
    pub params:       Vec<FnParam>,
    pub return_ty:    Option<Box<Type>>,
    pub where_clause: Option<Box<WhereClause>>,
    pub contracts:    Vec<Box<Contract>>,
    pub body:         Box<Block>,
}

pub struct ExternFunctionNoBody {
    pub span:         SpanId,
    pub node_id:      ast::NodeId,
    pub attrs:        Vec<Box<Attribute>>,
    pub vis:          Visibility,
    pub is_unsafe:    bool,
    pub abi:          Abi,
    pub name:         NameId,
    pub generics:     Option<Box<GenericParams>>,
    pub params:       Vec<FnParam>,
    pub return_ty:    Option<Box<Type>>,
    pub where_clause: Option<Box<WhereClause>>,
    pub contracts:    Vec<Box<Contract>>,
}

pub struct TraitFunction {
    pub span:         SpanId,
    pub node_id:      ast::NodeId,
    pub attrs:        Vec<Box<Attribute>>,
    pub vis:          Visibility,
    pub is_override:  bool,
    pub is_const:     bool,
    pub is_unsafe:    bool,
    pub name:         NameId,
    pub generics:     Option<Box<GenericParams>>,
    pub receiver:     FnReceiver,
    pub params:       Vec<FnParam>,
    pub return_ty:    Option<Box<Type>>,
    pub where_clause: Option<Box<WhereClause>>,
    pub contracts:    Vec<Box<Contract>>,
    pub body:         Option<Box<Block>>,
}

pub enum FnReceiver {
    None,
    SelfReceiver {
        span:   SpanId,
        is_ref: bool,
        is_mut: bool,
    },
    SelfTyped {
        span:   SpanId,
        is_mut: bool,
        ty:     Box<Type>,
    }
}

#[derive(Clone)]
pub enum FnParam {
    Param {
        span:    SpanId,
        attrs:   Vec<Box<Attribute>>,
        label:   Option<NameId>,
        pattern: Box<Pattern>,
        ty:      Box<Type>,
    },
    Opt {
        span:    SpanId,
        attrs:   Vec<Box<Attribute>>,
        label:   Option<NameId>,
        pattern: Box<Pattern>,
        ty:      Box<Type>,
        def:     Box<Expr>,
    },
    Variadic {
        span:    SpanId,
        attrs:   Vec<Box<Attribute>>,
        name:    NameId,
        ty:      Box<Type>,
    }
}

pub struct TypeAlias {
    pub span:     SpanId,
    pub node_id:  ast::NodeId,
    pub attrs:    Vec<Box<Attribute>>,
    pub vis:      Visibility,
    pub name:     NameId,
    pub generics: Option<Box<GenericParams>>,
    pub ty:       Box<Type>,
}

pub struct TraitTypeAlias {
    pub span:     SpanId,
    pub node_id:  ast::NodeId,
    pub attrs:    Vec<Box<Attribute>>,
    pub name:     NameId,
    pub generics: Option<Box<GenericParams>>,
}

pub struct DistinctType {
    pub span:     SpanId,
    pub node_id:  ast::NodeId,
    pub attrs:    Vec<Box<Attribute>>,
    pub vis:      Visibility,
    pub name:     NameId,
    pub generics: Option<Box<GenericParams>>,
    pub ty:       Box<Type>
}

pub struct OpaqueType {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub attrs:   Vec<Box<Attribute>>,
    pub vis:     Visibility,
    pub name:    NameId,
    pub size:    Option<Box<Expr>>
}

pub struct Struct {
    pub span:         SpanId,
    pub node_id:      ast::NodeId,
    pub attrs:        Vec<Box<Attribute>>,
    pub vis:          Visibility,
    pub is_mut:       bool,
    pub is_record:    bool,
    pub name:         NameId,
    pub generics:     Option<Box<GenericParams>>,
    pub where_clause: Option<Box<WhereClause>>,
    pub fields:       Vec<StructField>,
    pub uses:         Vec<StructUse>,
    /// Allow double underscore names (compiler reserved)
    pub allow_du:     bool,
}

#[derive(Clone)]
pub struct StructField {
    pub span:    SpanId,
    pub attrs:   Vec<Box<Attribute>>,
    pub vis:     Visibility,
    pub is_mut:  bool,
    pub name:    NameId,
    pub ty:      Box<Type>,
    pub def:     Option<Box<Expr>>,
}

#[derive(Clone)]
pub struct StructUse {
    pub span:   SpanId,
    pub attrs:  Vec<Box<Attribute>>,
    pub vis:    Visibility,
    pub is_mut: bool,
    pub path:   TypePath,
}

pub struct TupleStruct {
    pub span:         SpanId,
    pub node_id:      ast::NodeId,
    pub attrs:        Vec<Box<Attribute>>,
    pub vis:          Visibility,
    pub is_mut:       bool,
    pub is_record:    bool,
    pub name:         NameId,
    pub generics:     Option<Box<GenericParams>>,
    pub where_clause: Option<Box<WhereClause>>,
    pub fields:       Vec<TupleStructField>,
}

#[derive(Clone)]
pub struct TupleStructField {
    pub span:  SpanId,
    pub attrs: Vec<Box<Attribute>>,
    pub vis:   Visibility,
    pub ty:    Box<Type>,
    pub def:   Option<Box<Expr>>
}

pub struct UnitStruct {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub attrs:   Vec<Box<Attribute>>,
    pub vis:     Visibility,
    pub name:    NameId,
}

pub struct Union {
    pub span:         SpanId,
    pub node_id:      ast::NodeId,
    pub attrs:        Vec<Box<Attribute>>,
    pub vis:          Visibility,
    pub is_mut:       bool,
    pub name:         NameId,
    pub generics:     Option<Box<GenericParams>>,
    pub where_clause: Option<Box<WhereClause>>,
    pub fields:       Vec<UnionField>,
}

pub struct UnionField {
    pub span:   SpanId,
    pub attrs:  Vec<Box<Attribute>>,
    pub vis:    Visibility,
    pub is_mut: bool,
    pub name:   NameId,
    pub ty:     Box<Type>,
}

pub struct AdtEnum {
    pub span:         SpanId,
    pub node_id:      ast::NodeId,
    pub attrs:        Vec<Box<Attribute>>,
    pub vis:          Visibility,
    pub is_mut:       bool,
    pub is_record:    bool,
    pub name:         NameId,
    pub generics:     Option<Box<GenericParams>>,
    pub where_clause: Option<Box<WhereClause>>,
    pub variants:     Vec<AdtEnumVariant>,
    /// Allow double underscore names (compiler reserved)
    pub allow_du:     bool,
}

#[derive(Clone)]
pub enum AdtEnumVariant {
    Struct {
        span:         SpanId,
        attrs:        Vec<Box<Attribute>>,
        is_mut:       bool,
        name:         NameId,
        fields:       Vec<StructField>,
        discriminant: Option<Box<Expr>>,
    },
    Tuple {
        span:         SpanId,
        attrs:        Vec<Box<Attribute>>,
        is_mut:       bool,
        name:         NameId,
        fields:       Vec<TupleStructField>,
        discriminant: Option<Box<Expr>>,
    },
    Fieldless {
        span:         SpanId,
        attrs:        Vec<Box<Attribute>>,
        name:         NameId,
        discriminant: Option<Box<Expr>>,
    }
}

pub struct FlagEnum {
    pub span:     SpanId,
    pub node_id:  ast::NodeId,
    pub attrs:    Vec<Box<Attribute>>,
    pub vis:      Visibility,
    pub name:     NameId,
    pub variants: Vec<FlagEnumVariant>,
}

pub struct FlagEnumVariant {
    pub span:         SpanId,
    pub attrs:        Vec<Box<Attribute>>,
    pub name:         NameId,
    pub discriminant: Option<Box<Expr>>,
}

pub struct Bitfield {
    pub span:         SpanId,
    pub node_id:      ast::NodeId,
    pub attrs:        Vec<Box<Attribute>>,
    pub vis:          Visibility,
    pub is_mut:       bool,
    pub is_record:    bool,
    pub name:         NameId,
    pub generics:     Option<Box<GenericParams>>,
    pub where_clause: Option<Box<WhereClause>>,
    pub fields:       Vec<BitfieldField>,
    pub uses:         Vec<BitfieldUse>,
}

pub struct BitfieldField {
    pub span:   SpanId,
    pub attrs:  Vec<Box<Attribute>>,
    pub vis:    Visibility,
    pub is_mut: bool,
    pub name:   NameId,
    pub ty:     Box<Type>,
    pub bits:   Option<Box<Expr>>,
    pub def:    Option<Box<Expr>>
}

pub struct BitfieldUse {
    pub span:   SpanId,
    pub attrs:  Vec<Box<Attribute>>,
    pub vis:    Visibility,
    pub is_mut: bool,
    pub path:   TypePath,
    pub bits:   Option<Box<Expr>>
}

pub struct Const {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub attrs:   Vec<Box<Attribute>>,
    pub vis:     Visibility,
    pub name:    NameId,
    pub ty:      Option<Box<Type>>,
    pub val:     Box<Expr>,
}

pub struct Static {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub attrs:   Vec<Box<Attribute>>,
    pub vis:     Visibility,
    pub name:    NameId,
    pub ty:      Option<Box<Type>>,
    pub val:     Box<Expr>,
}

pub struct TlsStatic {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub attrs:   Vec<Box<Attribute>>,
    pub vis:     Visibility,
    pub is_mut:  bool,
    pub name:    NameId,
    pub ty:      Option<Box<Type>>,
    pub val:     Box<Expr>,
}

pub struct ExternStatic {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub attrs:   Vec<Box<Attribute>>,
    pub vis:     Visibility,
    pub abi:     Abi,
    pub is_mut:  bool,
    pub name:    NameId,
    pub ty:      Box<Type>,
}

pub struct Property {
    pub span:      SpanId,
    pub node_id:   ast::NodeId,
    pub attrs:     Vec<Box<Attribute>>,
    pub vis:       Visibility,
    pub is_unsafe: bool,
    pub name:      NameId,
    pub get:       Option<Box<Expr>>,
    pub ref_get:   Option<Box<Expr>>,
    pub mut_get:   Option<Box<Expr>>,
    pub set:       Option<Box<Expr>>,
}

pub struct TraitProperty {
    pub span:        SpanId,
    pub node_id:     ast::NodeId,
    pub attrs:       Vec<Box<Attribute>>,
    pub vis:         Visibility,
    pub is_unsafe:   bool,
    pub name:        NameId,
    pub has_get:     bool,
    pub has_ref_get: bool,
    pub has_mut_get: bool,
    pub has_set:     bool,
}

pub struct Trait {
    pub span:       SpanId,
    pub node_id:    ast::NodeId,
    pub attrs:      Vec<Box<Attribute>>,
    pub vis:        Visibility,
    pub is_unsafe:  bool,
    pub is_sealed:  bool,
    pub name:       NameId,
    pub bounds:     Option<Box<TraitBounds>>,
}

pub struct Impl {
    pub span:         SpanId,
    pub node_id:      ast::NodeId,
    pub attrs:        Vec<Box<Attribute>>,
    pub vis:          Visibility,
    pub is_unsafe:    bool,
    pub generics:     Option<Box<GenericParams>>,
    pub ty:           Box<Type>,
    pub impl_trait:   Option<TypePath>,
    pub where_clause: Option<Box<WhereClause>>,
}

pub struct OpTrait {
    pub span:       SpanId,
    pub node_id:    ast::NodeId,
    pub attrs:      Vec<Box<Attribute>>,
    pub vis:        Visibility,
    pub name:       NameId,
    pub bases:      Vec<SimplePath>,
    pub precedence: Option<NameId>,
}

pub struct OpFunction {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub op_ty:   OpType,
    pub op:      Punctuation, 
    pub name:    NameId,
    pub ret_ty:  Option<Box<Type>>,
    pub def:     Option<Box<Expr>>,
}

pub struct OpSpecialization {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub op_ty:   OpType,
    pub op:      Punctuation,
    pub def:     Box<Expr>,
}

pub struct OpContract {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub expr:    Box<Expr>,
}

// =============================================================================================================================

#[derive(Clone)]
pub struct Block {
    pub span:  SpanId,
    pub stmts: Vec<Box<Stmt>>,
    pub expr:  Option<Box<Expr>>,
}

// =============================================================================================================================

#[derive(Clone)]
pub enum Stmt {
    VarDecl(VarDecl),
    UninitVarDecl(UninitVarDecl),
    Defer(DeferStmt),
    ErrDefer(ErrorDeferStmt),
    Expr(ExprStmt),
}

#[derive(Clone)]
pub struct VarDecl {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub attrs:   Vec<Box<Attribute>>,
    pub is_mut:  bool,
    pub name:    NameId,
    pub ty:      Option<Box<Type>>,
    pub expr:    Box<Expr>,
    /// Allow double underscore names (compiler reserved)
    pub allow_du: bool,
}

#[derive(Clone)]
pub struct UninitVarDecl {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub attrs:   Vec<Box<Attribute>>,
    pub is_mut:  bool,
    pub name:    NameId,
    pub ty:      Box<Type>,
    /// Allow double underscore names (compiler reserved)
    pub allow_du: bool,
}

#[derive(Clone)]
pub struct DeferStmt {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub attrs:   Vec<Box<Attribute>>,
    pub expr:    Box<Expr>,
}

#[derive(Clone)]
pub struct ErrorDeferStmt {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub attrs:   Vec<Box<Attribute>>,
    pub rec:     Option<ErrorDeferReceiver>,
    pub expr:    Box<Expr>,
}

#[derive(Clone)]
pub struct ErrorDeferReceiver { 
    pub span:    SpanId,
    pub is_mut:  bool,
    pub name:    NameId,
}

#[derive(Clone)]
pub struct ExprStmt {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub expr:    Box<Expr>,
}

// =============================================================================================================================


#[derive(Clone)]
pub enum Expr {
    Unit(UnitExpr),
    FullRange(FullRangeExpr),
    Underscore(UnderscoreExpr),
    Literal(LiteralExpr),
    Path(PathExpr),
    Block(BlockExpr),
    Prefix(PrefixExpr),
    Postfix(PostfixExpr),
    Infix(InfixExpr),
    Inplace(InplaceExpr),
    TypeCast(TypeCastExpr),
    TypeCheck(TypeCheckExpr),
    Tuple(TupleExpr),
    Array(ArrayExpr),
    Struct(StructExpr),
    Index(IndexExpr),
    TupleIndex(TupleIndexExpr),
    FnCall(FnCallExpr),
    MethodCall(MethodCallExpr),
    FieldAccess(FieldAccessExpr),
    Closure(ClosureExpr),
    Loop(LoopExpr),
    Match(MatchExpr),
    Break(BreakExpr),
    Continue(ContinueExpr),
    Fallthrough(FallthroughExpr),
    Return(ReturnExpr),
    Throw(ThrowExpr),
    Comma(CommaExpr),
    When(WhenExpr),

    // Special expression only uses in match expressions for let var decls
    Irrefutable,
}

impl Expr {
    pub fn span(&self) -> SpanId {
        match self {
            Expr::Unit(node) => node.span,
            Expr::FullRange(node) => node.span,
            Expr::Underscore(node) => node.span,
            Expr::Literal(node) => node.span,
            Expr::Path(node) => node.span(),
            Expr::Block(node) => node.span,
            Expr::Prefix(node) => node.span,
            Expr::Postfix(node) => node.span,
            Expr::Infix(node) => node.span,
            Expr::Inplace(node) => node.span,
            Expr::TypeCast(node) => node.span,
            Expr::TypeCheck(node) => node.span,
            Expr::Tuple(node) => node.span,
            Expr::Array(node) => node.span,
            Expr::Struct(node) => node.span,
            Expr::Index(node) => node.span,
            Expr::TupleIndex(node) => node.span,
            Expr::FnCall(node) => node.span,
            Expr::MethodCall(node) => node.span,
            Expr::FieldAccess(node) => node.span,
            Expr::Closure(node) => node.span,
            Expr::Loop(node) => node.span,
            Expr::Match(node) => node.span,
            Expr::Break(node) => node.span,
            Expr::Continue(node) => node.span,
            Expr::Fallthrough(node) => node.span,
            Expr::Return(node) => node.span,
            Expr::Throw(node) => node.span,
            Expr::Comma(node) => node.span,
            Expr::When(node) => node.span,
            Expr::Irrefutable => todo!(),
        }
    }
}

#[derive(Clone)]
pub struct UnitExpr {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
}

#[derive(Clone)]
pub struct FullRangeExpr {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
}

#[derive(Clone)]
pub struct UnderscoreExpr {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
}

#[derive(Clone)]
pub enum LiteralValue {
    Lit(LiteralId),
    Bool(bool),
}

#[derive(Clone)]
pub enum LiteralOp {
    Name(NameId),
    Primitive(type_system::PrimitiveType),
    StringSlice(type_system::StringSliceType),
}

#[derive(Clone)]
pub struct LiteralExpr {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub literal: LiteralValue,
    pub lit_op:  Option<LiteralOp>,
}

#[derive(Clone)]
pub enum PathExpr {
    Named {
        span:    SpanId,
        node_id: ast::NodeId,
        iden:    Identifier,
    },
    Inferred {
        span:    SpanId,
        node_id: ast::NodeId,
        iden:    Identifier,
    },
    SelfPath {
        span:    SpanId,
        node_id: ast::NodeId,
    },
    Qualified {
        span:    SpanId,
        node_id: ast::NodeId,
        path:    QualifiedPath,
    }
}

impl PathExpr {
    pub fn span(&self) -> SpanId {
        match self {
            PathExpr::Named { span, .. } => *span,
            PathExpr::Inferred { span, .. } => *span,
            PathExpr::SelfPath { span, .. } => *span,
            PathExpr::Qualified { span, .. } => *span,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum BlockKind {
    Normal,
    Unsafe,
    Const,
    Try,
    TryUnwrap,
    Labeled(NameId)
}

#[derive(Clone)]
pub struct BlockExpr {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub kind:    BlockKind,
    pub block:   Block,
}

#[derive(Clone)]
pub struct PrefixExpr {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub op:      Punctuation,
    pub expr:    Box<Expr>,
}

#[derive(Clone)]
pub struct PostfixExpr {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub op:      Punctuation,
    pub expr:    Box<Expr>,
}

#[derive(Clone)]
pub struct InfixExpr {
    pub span:        SpanId,
    pub node_id:     ast::NodeId,
    pub left:        Box<Expr>,
    pub op:          Punctuation,
    pub right:       Box<Expr>,
    pub can_reorder: bool,
}

#[derive(Clone)]
pub struct InplaceExpr {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub left:    Box<Expr>,
    pub right:   Box<Expr>,
}

#[derive(Clone)]
pub struct TypeCastExpr {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub expr:    Box<Expr>,
    pub ty:      Box<Type>,
}

#[derive(Clone)]
pub struct TypeCheckExpr {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub expr:    Box<Expr>,
    pub ty:      Box<Type>,
}

#[derive(Clone)]
pub struct TupleExpr {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub exprs:   Vec<Box<Expr>>,
}

#[derive(Clone)]
pub struct ArrayExpr {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub exprs:   Vec<Box<Expr>>
}

#[derive(Clone)]
pub struct StructArg {
    pub span: SpanId,
    pub name: NameId,
    pub expr: Box<Expr>,
}

#[derive(Clone)]
pub struct StructExpr {
    pub span:     SpanId,
    pub node_id:  ast::NodeId,
    pub path:     Box<Expr>,
    pub args:     Vec<StructArg>,
    pub complete: Option<Box<Expr>>,
}

#[derive(Clone)]
pub struct IndexExpr {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub is_opt:  bool,
    pub expr:    Box<Expr>,
    pub index:   Box<Expr>,
}

#[derive(Clone)]
pub struct TupleIndexExpr {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub expr:    Box<Expr>,
    pub index:   usize,
}

#[derive(Clone)]
pub struct FnArg {
    pub span:  SpanId,
    pub label: Option<NameId>,
    pub expr:  Box<Expr>,
}

#[derive(Clone)]
pub struct FnCallExpr {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub func:    Box<Expr>,
    pub args:    Vec<FnArg>,
}

#[derive(Clone)]
pub struct MethodCallExpr { 
    pub span:           SpanId,
    pub node_id:        ast::NodeId,
    pub receiver:       Box<Expr>,
    pub method:         NameId,
    pub gen_args:       Option<Box<GenericArgs>>,
    pub args:           Vec<FnArg>,
    pub is_propagating: bool,
}

#[derive(Clone)]
pub struct FieldAccessExpr {
    pub span:           SpanId,
    pub node_id:        ast::NodeId,
    pub expr:           Box<Expr>,
    pub field:          NameId,
    pub gen_args:       Option<Box<GenericArgs>>,
    pub is_propagating: bool,
}

// TODO: This is not correct yet
#[derive(Clone)]
pub struct ClosureExpr {
    pub span:     SpanId,
    pub node_id:  ast::NodeId,
    pub is_moved: bool,
    pub params:   Vec<FnParam>,
    pub ret:      Option<Box<Type>>,
    pub body:     Box<Expr>,
}

#[derive(Clone)]
pub struct LoopExpr {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub label:   Option<NameId>,
    pub body:    Box<Block>,
}

#[derive(Clone)]
pub struct MatchBranch {
    pub label:     Option<NameId>,
    pub pattern:   Box<Pattern>,
    pub guard:     Option<Box<Expr>>,
    pub body:      Box<Expr>,
}

#[derive(Clone)]
pub struct MatchExpr {
    pub span:      SpanId,
    pub node_id:   ast::NodeId,
    pub label:     Option<NameId>,
    pub scrutinee: Box<Expr>,
    pub branches:  Vec<MatchBranch>,
    // Does this match require a bool, i.e. converted `if` or `while`
    pub bool_cond: bool,
}

#[derive(Clone)]
pub struct BreakExpr {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub label:   Option<NameId>,
    pub value:   Option<Box<Expr>>,
}

#[derive(Clone)]
pub struct ContinueExpr {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub label:   Option<NameId>
}

#[derive(Clone)]
pub struct FallthroughExpr {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub label:   Option<NameId>,
}

#[derive(Clone)]
pub struct ReturnExpr {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub value:   Option<Box<Expr>>,
}

#[derive(Clone)]
pub struct ThrowExpr {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub expr:    Box<Expr>,
}

#[derive(Clone)]
pub struct CommaExpr {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub exprs:   Vec<Box<Expr>>,
}

#[derive(Clone)]
pub struct WhenExpr {
    pub span:      SpanId,
    pub node_id:   ast::NodeId,
    pub cond:      Box<Expr>,
    pub body:      Box<Block>,
    pub else_body: Option<Box<Block>>
}

// =============================================================================================================================

#[derive(Clone)]
pub enum Pattern {
    Wildcard(WildcardPattern),
    Rest(RestPattern),
    Literal(LiteralPattern),
    Iden(IdenPattern),
    Path(PathPattern),
    Range(RangePattern),
    Reference(ReferencePattern),
    Struct(StructPattern),
    TupleStruct(TupleStructPattern),
    Tuple(TuplePattern),
    Slice(SlicePattern),
    EnumMember(EnumMemberPattern),
    Alternative(AlternativePattern),
    TypeCheck(TypeCheckPattern),
}

impl Pattern {
    pub fn span(&self) -> SpanId {
        match self {
            Pattern::Wildcard(node) => node.span,
            Pattern::Rest(node) => node.span,
            Pattern::Literal(node) => node.span,
            Pattern::Iden(node) => node.span,
            Pattern::Path(node) => node.span,
            Pattern::Range(node) => node.span(),
            Pattern::Reference(node) => node.span,
            Pattern::Struct(node) => node.span,
            Pattern::TupleStruct(node) => node.span,
            Pattern::Tuple(node) => node.span,
            Pattern::Slice(node) => node.span,
            Pattern::EnumMember(node) => node.span,
            Pattern::Alternative(node) => node.span,
            Pattern::TypeCheck(node) => node.span,
        }
    }
}

#[derive(Clone)]
pub struct WildcardPattern {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
}

#[derive(Clone)]
pub struct RestPattern {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
}

#[derive(Clone)]
pub struct LiteralPattern {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub literal: LiteralValue,
    pub lit_op:  Option<LiteralOp>,
}

#[derive(Clone)]
pub struct IdenPattern {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub is_ref:  bool,
    pub is_mut:  bool,
    pub name:    NameId,
    pub bound:   Option<Box<Pattern>>
}

#[derive(Clone)]
pub struct PathPattern {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub path:    Path,
}

#[derive(Clone)]
pub enum RangePattern {
    Exclusive{
        span: SpanId,
        node_id: ast::NodeId,
        begin: Box<Pattern>,
        end: Box<Pattern>
    },
    Inclusive{
        span: SpanId,
        node_id: ast::NodeId,
        begin: Box<Pattern>,
        end: Box<Pattern>
    },
    From{
        span: SpanId,
        node_id: ast::NodeId,
        begin: Box<Pattern>
    },
    To{
        span: SpanId,
        node_id: ast::NodeId,
        end: Box<Pattern>
    },
    InclusiveTo{
        span: SpanId,
        node_id: ast::NodeId,
        end: Box<Pattern>
    },
}

impl RangePattern {
    pub fn span(&self) -> SpanId {
        match self {
            RangePattern::Exclusive { span, .. } => *span,
            RangePattern::Inclusive { span, .. } => *span,
            RangePattern::From { span, .. } => *span,
            RangePattern::To { span, .. } => *span,
            RangePattern::InclusiveTo { span, .. } => *span,
        }
    }
}

#[derive(Clone)]
pub struct ReferencePattern {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub is_mut:  bool,
    pub pattern: Box<Pattern>,
}

#[derive(Clone)]
pub enum StructPatternField {
    Named {
        span:    SpanId,
        node_id: ast::NodeId,
        name:    NameId,
        pattern: Box<Pattern>,
    },
    TupleIndex {
        span:    SpanId,
        node_id: ast::NodeId,
        index:   usize,
        pattern: Box<Pattern>,
    },
    Iden {
        span:    SpanId,
        node_id: ast::NodeId,
        is_ref:  bool,
        is_mut:  bool,
        iden:    NameId,
        bound:   Option<Box<Pattern>>,
    },
    Rest,
}

#[derive(Clone)]
pub struct StructPattern {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub path:    Option<Path>,
    pub fields:  Vec<StructPatternField>,
}

#[derive(Clone)]
pub struct TupleStructPattern {
    pub span:     SpanId,
    pub node_id:  ast::NodeId,
    pub path:     Option<Path>,
    pub patterns: Vec<Box<Pattern>>,
}

#[derive(Clone)]
pub struct TuplePattern {
    pub span:     SpanId,
    pub node_id:  ast::NodeId,
    pub patterns: Vec<Box<Pattern>>,
}

#[derive(Clone)]
pub struct SlicePattern {
    pub span:     SpanId,
    pub node_id:  ast::NodeId,
    pub patterns: Vec<Box<Pattern>>,
}

#[derive(Clone)]
pub struct EnumMemberPattern {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub name:    NameId
}

#[derive(Clone)]
pub struct AlternativePattern {
    pub span:     SpanId,
    pub node_id:  ast::NodeId,
    pub patterns: Vec<Box<Pattern>>,
}

#[derive(Clone)]
pub struct TypeCheckPattern {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub ty:      Box<Type>,
}

// =============================================================================================================================

#[derive(Clone)]
pub enum Type {
    Unit(UnitType),
    Never(NeverType),
    Primitive(PrimitiveType),
    Path(PathType),
    Tuple(TupleType),
    Array(ArrayType),
    Slice(SliceType),
    StringSlice(StringSliceType),
    Pointer(PointerType),
    Reference(ReferenceType),
    Optional(OptionalType),
    Fn(FnType),
}

#[derive(Clone)]
pub struct UnitType {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
}

#[derive(Clone)]
pub struct NeverType {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
}


#[derive(Clone)]
pub struct PrimitiveType {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub ty:      type_system::PrimitiveType,
}

#[derive(Clone)]
pub struct PathType {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub path:    TypePath,
}

#[derive(Clone)]
pub struct TupleType {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub types:   Vec<Box<Type>>
}

#[derive(Clone)]
pub struct ArrayType {
    pub span:     SpanId,
    pub node_id:  ast::NodeId,
    pub size:     Box<Expr>,
    pub sentinel: Option<Box<Expr>>,
    pub ty:       Box<Type>,
}

#[derive(Clone)]
pub struct SliceType {
    pub span:     SpanId,
    pub node_id:  ast::NodeId,
    pub sentinel: Option<Box<Expr>>,
    pub ty:       Box<Type>,
}

#[derive(Clone)]
pub struct StringSliceType {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub ty:      type_system::StringSliceType
}

#[derive(Clone)]
pub struct PointerType {
    pub span:     SpanId,
    pub node_id:  ast::NodeId,
    pub is_multi: bool,
    pub is_mut:   bool,
    pub ty:       Box<Type>,
    pub sentinel: Option<Box<Expr>>,
}

#[derive(Clone)]
pub struct ReferenceType {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub is_mut:  bool,
    pub ty:      Box<Type>,
}

#[derive(Clone)]
pub struct OptionalType {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub ty:      Box<Type>,
}

#[derive(Clone)]
pub struct FnType {
    pub span:      SpanId,
    pub node_id:   ast::NodeId,
    pub is_unsafe: bool,
    pub abi:       Abi,
    pub params:    Vec<(NameId, Box<Type>)>,
    pub return_ty: Option<Box<Type>>,
}

// =============================================================================================================================

#[derive(Clone)]
pub struct GenericParams {

}

#[derive(Clone)]
pub struct GenericArgs {

}

#[derive(Clone)]
pub struct WhereClause {

}

#[derive(Clone)]
pub struct TraitBounds {

}

// =============================================================================================================================

pub struct Contract {

}

// =============================================================================================================================

// TODO: Vec to SimplePath
#[derive(Clone)]
pub enum Visibility {
    Priv,
    Pub {
        span:    SpanId,
        node_id: ast::NodeId,
    },
    Lib {
        span:    SpanId,
        node_id: ast::NodeId,
    },
    Package {
        span:    SpanId,
        node_id: ast::NodeId,
    },
    Super {
        span:    SpanId,
        node_id: ast::NodeId,
    },
    Path {
        span:    SpanId,
        node_id: ast::NodeId,
        path:    SimplePath,
    },
}

// =============================================================================================================================

#[derive(Clone)]
pub struct Attribute {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub path:    Vec<NameId>,
    pub meta:    AttrMeta,
}

#[derive(Clone)]
pub enum AttrMeta {
    None,
    Expr{
        span: SpanId,
        expr: Expr
    },
    Assign {
        span: SpanId,
        expr: Expr
    },
    Meta {
        span: SpanId,
        metas: Vec<AttrMeta>,
    }
}

// =============================================================================================================================

pub struct FunctionContext {
    pub scope: Scope,
    pub sym:   Option<SymbolRef>,
}

impl FunctionContext {
    fn new(scope: Scope) -> Self {
        Self {
            scope,
            sym: None,
        }
    }
}

pub struct TypeAliasContext {
    pub scope: Scope,
    pub sym:   Option<SymbolRef>,
}

impl TypeAliasContext {
    fn new(scope: Scope) -> Self {
        Self {
            scope,
            sym: None,
        }
    }
}

pub struct StructContext {
    pub scope: Scope,
    pub sym:   Option<SymbolRef>,
}

impl StructContext {
    pub fn new(scope: Scope) -> Self {
        Self {
            scope,
            sym: None,
        }
    }
}

pub struct UnionContext {
    pub scope: Scope,
    pub sym:   Option<SymbolRef>,
}

impl UnionContext {
    pub fn new(scope: Scope) -> Self {
        Self {
            scope,
            sym: None,
        }
    }
}


pub struct AdtEnumContext {
    pub scope: Scope,
    pub sym:   Option<SymbolRef>,
}

impl AdtEnumContext {
    pub fn new(scope: Scope) -> Self {
        Self {
            scope,
            sym: None,
        }
    }
}

pub struct FlagEnumContext {
    pub scope: Scope,
    pub sym:   Option<SymbolRef>,
}

impl FlagEnumContext {
    pub fn new(scope: Scope) -> Self {
        Self {
            scope,
            sym: None,
        }
    }
}

pub struct BitfieldContext {
    pub scope: Scope,
    pub sym:   Option<SymbolRef>,
}

impl BitfieldContext {
    pub fn new(scope: Scope) -> Self {
        Self {
            scope,
            sym: None,
        }
    }
}

pub struct ConstContext {
    pub scope: Scope,
    pub sym:   Option<SymbolRef>,
}

impl ConstContext {
    pub fn new(scope: Scope) -> Self {
        Self {
            scope,
            sym: None,
        }
    }
}

pub struct StaticContext {
    pub scope: Scope,
    pub sym:   Option<SymbolRef>,
}

impl StaticContext {
    pub fn new(scope: Scope) -> Self {
        Self {
            scope,
            sym: None,
        }
    }
}

pub struct PropertyContext {
    pub scope: Scope,
    pub sym:   Option<SymbolRef>,
}

impl PropertyContext {
    pub fn new(scope: Scope) -> Self {
        Self {
            scope,
            sym: None,
        }
    }
}

pub struct TraitContext {
    pub scope: Scope,
    pub sym:   Option<SymbolRef>,
}

impl TraitContext {
    pub fn new(scope: Scope) -> Self {
        Self {
            scope,
            sym: None,
        }
    }
}

pub struct ImplContext {
    pub scope: Scope,
    pub sym:   Option<SymbolRef>,
}

impl ImplContext {
    pub fn new(scope: Scope) -> Self {
        Self {
            scope,
            sym: None,
        }
    }
}

pub struct OpTraitContext {
    pub scope: Scope,
    pub sym:   Option<SymbolRef>,
}

impl OpTraitContext {
    pub fn new(scope: Scope) -> Self {
        Self {
            scope,
            sym: None,
        }
    }
}

pub struct OpFunctionContext {
    pub scope: Scope,
    pub sym:   Option<SymbolRef>,
}

impl OpFunctionContext {
    pub fn new(scope: Scope) -> Self {
        Self {
            scope,
            sym: None,
        }
    }
}

pub struct OpSpecializationContext {
    pub scope: Scope,
    pub sym:   Option<SymbolRef>,
}

impl OpSpecializationContext {
    pub fn new(scope: Scope) -> Self {
        Self {
            scope,
            sym: None,
        }
    }
}

pub struct OpContractContext {
    pub scope: Scope,
    pub sym:   Option<SymbolRef>,
}

impl OpContractContext {
    pub fn new(scope: Scope) -> Self {
        Self {
            scope,
            sym: None,
        }
    }
}

// =============================================================================================================================

pub type TraitRef = Arc<RwLock<Trait>>;
pub type TraitContextRef = Arc<RwLock<TraitContext>>;
pub type ImplRef = Arc<RwLock<Impl>>;
pub type ImplContextRef = Arc<RwLock<ImplContext>>;
pub type Ref<T> = Arc<RwLock<T>>;

pub struct Hir {
    pub functions:                Vec<(Function, FunctionContext)>,
    pub extern_functions_no_body: Vec<(ExternFunctionNoBody, FunctionContext)>,
    pub type_aliases:             Vec<(TypeAlias, TypeAliasContext)>,
    pub distinct_types:           Vec<(DistinctType, TypeAliasContext)>,
    pub opaque_types:             Vec<(OpaqueType, TypeAliasContext)>,
    pub structs:                  Vec<(Struct, StructContext)>,
    pub tuple_structs:            Vec<(TupleStruct, StructContext)>,
    pub unit_structs:             Vec<(UnitStruct, StructContext)>,
    pub unions:                   Vec<(Union, UnionContext)>,
    pub adt_enums:                Vec<(AdtEnum, AdtEnumContext)>,
    pub flag_enums:               Vec<(FlagEnum, FlagEnumContext)>,
    pub bitfields:                Vec<(Bitfield, BitfieldContext)>,
    pub consts:                   Vec<(Const, ConstContext)>,
    pub statics:                  Vec<(Static, StaticContext)>,
    pub tls_statics:              Vec<(TlsStatic, StaticContext)>,
    pub extern_statics:           Vec<(ExternStatic, StaticContext)>,
    
    // trait items store the index into the traits array, as it cannot have any traits removed
    pub traits:                   Vec<(Ref<Trait>, Ref<TraitContext>)>,
    pub trait_type_alias:         Vec<(usize, TraitTypeAlias, TypeAliasContext)>,
    pub trait_consts:             Vec<(usize, Const, ConstContext)>,
    pub trait_functions:          Vec<(usize, TraitFunction, FunctionContext)>,
    pub trait_properties:         Vec<(usize, TraitProperty, PropertyContext)>,
    
    // assoc items store the index into the impls array, as it cannot have any impl removed
    pub impls:                    Vec<(Ref<Impl>, Ref<ImplContext>)>,
    pub impl_functions:           Vec<(usize, Function, FunctionContext)>,
    pub methods:                  Vec<(usize, Method, FunctionContext)>,
    pub impl_type_aliases:        Vec<(usize, TypeAlias, TypeAliasContext)>,
    pub impl_consts:              Vec<(usize, Const, ConstContext)>,
    pub impl_statics:             Vec<(usize, Static, StaticContext)>,
    pub impl_tls_statics:         Vec<(usize, TlsStatic, StaticContext)>,
    pub properties:               Vec<(usize, Property, PropertyContext)>,

    // op items store the index into the op_traits array, as it cannot have any op_trait removed
    pub op_traits:                Vec<(Ref<OpTrait>, Ref<OpTraitContext>)>,
    pub op_functions:             Vec<(usize, OpFunction, OpFunctionContext)>,
    pub op_specializations:       Vec<(usize, OpSpecialization, OpSpecializationContext)>,
    pub op_contracts:             Vec<(usize, OpContract, OpContractContext)>,
}

impl Hir {
    pub fn new() -> Self {
        Self {
            functions:                Vec::new(),
            extern_functions_no_body: Vec::new(),
            type_aliases:             Vec::new(),
            distinct_types:           Vec::new(),
            opaque_types:             Vec::new(),
            structs:                  Vec::new(),
            tuple_structs:            Vec::new(),
            unit_structs:             Vec::new(),
            unions:                   Vec::new(),
            adt_enums:                Vec::new(),
            flag_enums:               Vec::new(),
            bitfields:                Vec::new(),
            consts:                   Vec::new(),
            statics:                  Vec::new(),
            tls_statics:              Vec::new(),
            extern_statics:           Vec::new(),
            
            traits:                   Vec::new(),
            trait_type_alias:         Vec::new(),
            trait_consts:             Vec::new(),
            trait_functions:          Vec::new(),
            trait_properties:         Vec::new(),
            
            impls:                    Vec::new(),
            impl_functions:           Vec::new(),
            methods:                  Vec::new(),
            impl_type_aliases:        Vec::new(),
            impl_consts:              Vec::new(),
            impl_statics:             Vec::new(),
            impl_tls_statics:         Vec::new(),
            properties:               Vec::new(),

            op_traits:                Vec::new(),
            op_functions:             Vec::new(),
            op_specializations:       Vec::new(),
            op_contracts:             Vec::new(),
        }
    }

    pub fn add_function(&mut self, in_impl: bool, scope: Scope, item: Function) {
        let ctx = FunctionContext::new(scope);
        if in_impl {
            let impl_idx = self.impls.len() - 1;
            self.impl_functions.push((impl_idx, item, ctx));
        } else {
            self.functions.push((item, ctx));
        }
    }

    pub fn add_extern_function(&mut self, scope: Scope, item: ExternFunctionNoBody) {
        let ctx = FunctionContext::new(scope);
        self.extern_functions_no_body.push((item, ctx));
    }

    pub fn add_method(&mut self, scope: Scope, item: Method) {
        let ctx = FunctionContext::new(scope);
        todo!();
    }

    pub fn add_trait_function(&mut self, scope: Scope, item: TraitFunction) {
        let ctx = FunctionContext::new(scope);
        let trait_idx = self.traits.len() - 1;
        self.trait_functions.push((trait_idx, item, ctx));
    }

    pub fn add_type_alias(&mut self, scope: Scope, item: TypeAlias) {
        let ctx = TypeAliasContext::new(scope);
        self.type_aliases.push((item, ctx));
    }

    pub fn add_trait_type_alias(&mut self, scope: Scope, item: TraitTypeAlias) {
        let ctx = TypeAliasContext::new(scope);
        let trait_idx = self.traits.len() - 1;
        self.trait_type_alias.push((trait_idx, item, ctx));
    }

    pub fn add_distinct_type(&mut self, scope: Scope, item: DistinctType) {
        let ctx = TypeAliasContext::new(scope);
        self.distinct_types.push((item, ctx));
    }

    pub fn add_opaque_type(&mut self, scope: Scope, item: OpaqueType) {
        let ctx = TypeAliasContext::new(scope);
        self.opaque_types.push((item, ctx));
    }

    pub fn add_struct(&mut self, scope: Scope, item: Struct) {
        let ctx = StructContext::new(scope);
        self.structs.push((item, ctx));
    }

    pub fn add_tuple_struct(&mut self, scope: Scope, item: TupleStruct) {
        let ctx = StructContext::new(scope);
        self.tuple_structs.push((item, ctx));
    }

    pub fn add_unit_struct(&mut self, scope: Scope, item: UnitStruct) {
        let ctx = StructContext::new(scope);
        self.unit_structs.push((item, ctx));
    }

    pub fn add_union(&mut self, scope: Scope, item: Union) {
        let ctx = UnionContext::new(scope);
        self.unions.push((item, ctx));
    }

    pub fn add_adt_enum(&mut self, scope: Scope, item: AdtEnum) {
        let ctx = AdtEnumContext::new(scope);
        self.adt_enums.push((item, ctx))
    }

    pub fn add_flag_enum(&mut self, scope: Scope, item: FlagEnum) {
        let ctx = FlagEnumContext::new(scope);
        self.flag_enums.push((item, ctx));
    }

    pub fn add_bitfield(&mut self, scope: Scope, item: Bitfield) {
        let ctx = BitfieldContext::new(scope);
        self.bitfields.push((item, ctx));
    }

    pub fn add_const(&mut self, in_impl: bool, scope: Scope, item: Const) {
        let ctx = ConstContext::new(scope);
        if in_impl {
            let impl_idx = self.impls.len() - 1;
            self.impl_consts.push((impl_idx, item, ctx));
        } else {
            self.consts.push((item, ctx));
        }
    }

    pub fn add_trait_const(&mut self, scope: Scope, item: Const) {
        let ctx = ConstContext::new(scope);
        let trait_idx = self.traits.len() - 1;
        self.trait_consts.push((trait_idx, item, ctx));
    }

    pub fn add_static(&mut self, in_impl: bool, scope: Scope, item: Static) {
        let ctx = StaticContext::new(scope);
        if in_impl {
            let impl_idx = self.impls.len() - 1;
            self.impl_statics.push((impl_idx, item, ctx));
        } else {
            self.statics.push((item, ctx));
        }
    }

    pub fn add_tls_static(&mut self, in_impl: bool, scope: Scope, item: TlsStatic) {
        let ctx = StaticContext::new(scope);
        if in_impl {
            let impl_idx = self.impls.len() - 1;
            self.impl_tls_statics.push((impl_idx, item, ctx));
        } else {
            self.tls_statics.push((item, ctx));
        }
    }

    pub fn add_extern_static(&mut self, scope: Scope, item: ExternStatic) {
        let ctx = StaticContext::new(scope);
        self.extern_statics.push((item, ctx));
    }

    // TODO: Properties are associated to an impl
    pub fn add_property(&mut self, scope: Scope, item: Property) {
        let impl_idx = self.impls.len() - 1;
        let ctx = PropertyContext::new(scope);
        self.properties.push((impl_idx, item, ctx));
    }

    pub fn add_trait_property(&mut self, scope: Scope, item: TraitProperty) {
        let trait_idx = self.traits.len() - 1;
        let ctx = PropertyContext::new(scope);
        self.trait_properties.push((trait_idx, item, ctx));
    }

    pub fn add_trait(&mut self, scope: Scope, item: Trait) {
        let item = Arc::new(RwLock::new(item));
        let ctx = Arc::new(RwLock::new(TraitContext::new(scope)));
        self.traits.push((item, ctx));
    }

    pub fn add_impl(&mut self, scope: Scope, item: Impl) {
        let item = Arc::new(RwLock::new(item));
        let ctx = Arc::new(RwLock::new(ImplContext::new(scope)));
        self.impls.push((item, ctx));
    }

    pub fn add_op_trait(&mut self, scope: Scope, item: OpTrait) {
        let item = Arc::new(RwLock::new(item));
        let ctx = Arc::new(RwLock::new(OpTraitContext::new(scope)));
        self.op_traits.push((item, ctx));
    }

    pub fn add_op_function(&mut self, scope: Scope, item: OpFunction) {
        let op_idx = self.op_traits.len() - 1;
        let ctx = OpFunctionContext::new(scope);
        self.op_functions.push((op_idx, item, ctx));
    }

    pub fn add_op_specialization(&mut self, scope: Scope, item: OpSpecialization) {
        let op_idx = self.op_traits.len() - 1;
        let ctx = OpSpecializationContext::new(scope);
        self.op_specializations.push((op_idx, item, ctx));
    }

    pub fn add_op_contract(&mut self, scope: Scope, item: OpContract) {
        let op_idx = self.op_traits.len() - 1;
        let ctx = OpContractContext::new(scope);
        self.op_contracts.push((op_idx, item, ctx));
    }
}