// High level intermediate representation
//
// Represented as nodes with internal tree structures per module-level item
#![allow(unused)]

use std::{fmt, sync::Arc};
use parking_lot::RwLock;

use crate::{
    ast::{self, NodeId}, common::{Abi, NameId, OpType, PrecedenceAssocKind, Scope, SpanId, SymbolRef, TraitItemRecord}, error_warning::{HirErrorCode, LexErrorCode}, lexer::Punctuation, literals::LiteralId, type_system
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

#[derive(Clone)]
pub struct HirError {
    pub node_id: ast::NodeId,
    pub err:     HirErrorCode
}

impl fmt::Display for HirError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.node_id == NodeId::INVALID {
            write!(f, "{}", self.err)
        } else {
            write!(f, "{}: {}", self.node_id, self.err)
        }
    }
}

// =============================================================================================================================

#[derive(Clone)]
pub struct PathCtx {
    pub path: Scope,
}

impl PathCtx {
    pub fn new() -> Self {
        Self {
            path: Scope::new(),
        }
    }
}


#[derive(Clone)]
pub struct SimplePath {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub names:   Vec<NameId>,
    pub ctx:     PathCtx,
}

#[derive(Clone)]
pub enum IdenName {
    Name{
        name: NameId,
        span: SpanId,
    },
    Disambig{
        span:       SpanId,
        trait_path: Box<Path>,
        name:       NameId,
        name_span:  SpanId,
    }
}

#[derive(Clone)]
pub struct Identifier {
    pub name:     IdenName,
    pub gen_args: Option<Box<GenericArgs>>,
    pub span:     SpanId,
}

#[derive(Clone)]
pub enum PathStart {
    None,
    SelfTy {
        span: SpanId,
    },
    Inferred {
        span: SpanId,
    },
    Type {
        ty: Box<Type>,
    }
}

#[derive(Clone)]
pub struct PathFnEnd {
    pub span:   SpanId,
    pub name:   NameId,
    pub params: Vec<(NameId, Box<Type>)>,
    pub ret_ty: Option<Box<Type>>,
}

#[derive(Clone)]
pub struct Path {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub start:   PathStart,
    pub idens:   Vec<Identifier>,
    pub fn_end:  Option<PathFnEnd>,
    pub ctx:     PathCtx,
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

#[derive(Clone)]
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
    pub path:   Path,
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
    pub path:   Path,
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

//--------------------------------------------------------------

pub struct Trait {
    pub span:         SpanId,
    pub node_id:      ast::NodeId,
    pub attrs:        Vec<Box<Attribute>>,
    pub vis:          Visibility,
    pub is_unsafe:    bool,
    pub is_sealed:    bool,
    pub name:         NameId,
    pub generics:     Option<Box<GenericParams>>,
    pub bounds:       Option<Box<TraitBounds>>,
    pub where_clause: Option<Box<WhereClause>>
}

pub struct TraitFunction {
    pub span:         SpanId,
    pub node_id:      ast::NodeId,
    pub attrs:        Vec<Box<Attribute>>,
    pub is_const:     bool,
    pub is_unsafe:    bool,
    pub name:         NameId,
    pub generics:     Option<Box<GenericParams>>,
    pub params:       Vec<FnParam>,
    pub return_ty:    Option<Box<Type>>,
    pub where_clause: Option<Box<WhereClause>>,
    pub contracts:    Vec<Box<Contract>>,
    pub body:         Option<Box<Block>>,
}

pub struct TraitMethod {
    pub span:         SpanId,
    pub node_id:      ast::NodeId,
    pub attrs:        Vec<Box<Attribute>>,
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

pub struct TraitTypeAlias {
    pub span:         SpanId,
    pub node_id:      ast::NodeId,
    pub attrs:        Vec<Box<Attribute>>,
    pub name:         NameId,
    pub generics:     Option<Box<GenericParams>>,
    pub where_clause: Option<Box<WhereClause>>,
    pub def:          Option<Box<Type>>,
}

pub struct TraitConst {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub attrs:   Vec<Box<Attribute>>,
    pub name:    NameId,
    pub ty:      Box<Type>,
    pub def:     Option<Box<Expr>>,
}

pub enum TraitPropMembers {
    Req {
        get:     Option<SpanId>,
        ref_get: Option<SpanId>,
        mut_get: Option<SpanId>,
        set:     Option<SpanId>,
    },
    Def {
        get:     Option<(SpanId, Box<Expr>)>,
        ref_get: Option<(SpanId, Box<Expr>)>,
        mut_get: Option<(SpanId, Box<Expr>)>,
        set:     Option<(SpanId, Box<Expr>)>,
    }
}

pub struct TraitProperty {
    pub span:      SpanId,
    pub node_id:   ast::NodeId,
    pub attrs:     Vec<Box<Attribute>>,
    pub is_unsafe: bool,
    pub name:      NameId,
    pub ty:        Box<Type>,
    pub members:   TraitPropMembers,
}

//--------------------------------------------------------------

pub struct Impl {
    pub span:         SpanId,
    pub node_id:      ast::NodeId,
    pub attrs:        Vec<Box<Attribute>>,
    pub vis:          Visibility,
    pub is_unsafe:    bool,
    pub generics:     Option<Box<GenericParams>>,
    pub ty:           Box<Type>,
    pub impl_trait:   Option<Path>,
    pub where_clause: Option<Box<WhereClause>>,
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

//--------------------------------------------------------------

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

pub struct OpContract {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub expr:    Box<Expr>,
}

//--------------------------------------------------------------

pub struct PrecedenceAssoc {
    pub span: SpanId,
    pub kind: PrecedenceAssocKind,
}

pub struct Precedence {
    pub span:        SpanId,
    pub node_id:     ast::NodeId,
    pub attrs:       Vec<Box<Attribute>>,
    pub vis:         Visibility,
    pub name:        NameId,
    pub higher_than: Option<(NameId, SpanId)>,
    pub lower_than:  Option<(NameId, SpanId)>,
    pub assoc:       Option<PrecedenceAssoc>,
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
    Slice(SliceExpr),
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
            Expr::Slice(node) => node.span, 
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
            Expr::Irrefutable => SpanId::INVALID,
        }
    }

    pub fn node_id(&self) -> NodeId {
        match self {
            Expr::Unit(node) => node.node_id,
            Expr::FullRange(node) => node.node_id,
            Expr::Underscore(node) => node.node_id,
            Expr::Literal(node) => node.node_id,
            Expr::Path(node) => node.node_id(),
            Expr::Block(node) => node.node_id,
            Expr::Prefix(node) => node.node_id,
            Expr::Postfix(node) => node.node_id,
            Expr::Infix(node) => node.node_id,
            Expr::Inplace(node) => node.node_id,
            Expr::TypeCast(node) => node.node_id,
            Expr::TypeCheck(node) => node.node_id,
            Expr::Tuple(node) => node.node_id,
            Expr::Array(node) => node.node_id,
            Expr::Slice(node) => node.node_id,
            Expr::Struct(node) => node.node_id,
            Expr::Index(node) => node.node_id,
            Expr::TupleIndex(node) => node.node_id,
            Expr::FnCall(node) => node.node_id,
            Expr::MethodCall(node) => node.node_id,
            Expr::FieldAccess(node) => node.node_id,
            Expr::Closure(node) => node.node_id,
            Expr::Loop(node) => node.node_id,
            Expr::Match(node) => node.node_id,
            Expr::Break(node) => node.node_id,
            Expr::Continue(node) => node.node_id,
            Expr::Fallthrough(node) => node.node_id,
            Expr::Return(node) => node.node_id,
            Expr::Throw(node) => node.node_id,
            Expr::Comma(node) => node.node_id,
            Expr::When(node) => node.node_id,
            Expr::Irrefutable => NodeId::INVALID,
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
        start:   PathStart,
        iden:    Identifier,
    },
    SelfPath {
        span:    SpanId,
        node_id: ast::NodeId,
    },
    Expanded {
        path:    Path,
    }
}

impl PathExpr {
    pub fn span(&self) -> SpanId {
        match self {
            PathExpr::Named { span, .. } => *span,
            PathExpr::SelfPath { span, .. } => *span,
            PathExpr::Expanded { path } => path.span,
        }
    }

    pub fn node_id(&self) -> NodeId {
        match self {
            PathExpr::Named { node_id, .. } => *node_id,
            PathExpr::SelfPath { node_id, .. } => *node_id,
            PathExpr::Expanded { .. } => NodeId::INVALID,
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
    pub node_id: NodeId,
    pub value:   Box<Expr>,
    pub count:   Box<Expr>,
}

#[derive(Clone)]
pub struct SliceExpr {
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
    pub method:         Identifier,
    pub args:           Vec<FnArg>,
    pub is_propagating: bool,
}

#[derive(Clone)]
pub struct FieldAccessExpr {
    pub span:           SpanId,
    pub node_id:        ast::NodeId,
    pub expr:           Box<Expr>,
    pub field:          Identifier,
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
pub struct TypeContext {
    ty: Option<type_system::TypeHandle>,
}

impl TypeContext {
    pub fn new() -> Self {
        Self {
            ty: None,
        }
    }
}

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

impl Type {
    pub fn ctx(&self) -> &TypeContext {
        match self {
            Type::Unit(ty) => &ty.ctx,
            Type::Never(ty) => &ty.ctx,
            Type::Primitive(ty) => &ty.ctx,
            Type::Path(ty) => &ty.ctx,
            Type::Tuple(ty) => &ty.ctx,
            Type::Array(ty) => &ty.ctx,
            Type::Slice(ty) => &ty.ctx,
            Type::StringSlice(ty) => &ty.ctx,
            Type::Pointer(ty) => &ty.ctx,
            Type::Reference(ty) => &ty.ctx,
            Type::Optional(ty) => &ty.ctx,
            Type::Fn(ty) => &ty.ctx,
        }
    }
}

#[derive(Clone)]
pub struct UnitType {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub ctx:     TypeContext,
}

#[derive(Clone)]
pub struct NeverType {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub ctx:     TypeContext,
}


#[derive(Clone)]
pub struct PrimitiveType {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub ty:      type_system::PrimitiveType,
    pub ctx:     TypeContext,
}

#[derive(Clone)]
pub struct PathType {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub path:    Path,
    pub ctx:     TypeContext,
}

impl PathType {
    pub fn from_name(name: NameId, span: SpanId, node_id: NodeId) -> Type {
        Type::Path(PathType {
            span,
            node_id,
            path: Path {
                span,
                node_id,
                start: PathStart::None,
                idens: vec![
                    Identifier {
                        name: IdenName::Name { name, span },
                        gen_args: None,
                        span
                    }
                ],
                fn_end: None,
                ctx: PathCtx::new(),
            },
            ctx: TypeContext::new(),
        })
    }

    pub fn self_ty(span: SpanId, node_id: NodeId) -> Type {
        Type::Path(PathType {
            span,
            node_id,
            path: Path {
                span,
                node_id,
                start: PathStart::SelfTy { span },
                idens: Vec::new(),
                fn_end: None,
                ctx: PathCtx::new(),
            },
            ctx: TypeContext::new(),
        })
    }
}

#[derive(Clone)]
pub struct TupleType {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub types:   Vec<Box<Type>>,
    pub ctx:     TypeContext,
}

#[derive(Clone)]
pub struct ArrayType {
    pub span:     SpanId,
    pub node_id:  ast::NodeId,
    pub size:     Box<Expr>,
    pub sentinel: Option<Box<Expr>>,
    pub ty:       Box<Type>,
    pub ctx:     TypeContext,
}

#[derive(Clone)]
pub struct SliceType {
    pub span:     SpanId,
    pub node_id:  ast::NodeId,
    pub sentinel: Option<Box<Expr>>,
    pub ty:       Box<Type>,
    pub ctx:     TypeContext,
}

#[derive(Clone)]
pub struct StringSliceType {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub ty:      type_system::StringSliceType,
    pub ctx:     TypeContext,
}

#[derive(Clone)]
pub struct PointerType {
    pub span:     SpanId,
    pub node_id:  ast::NodeId,
    pub is_multi: bool,
    pub is_mut:   bool,
    pub ty:       Box<Type>,
    pub sentinel: Option<Box<Expr>>,
    pub ctx:     TypeContext,
}

#[derive(Clone)]
pub struct ReferenceType {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub is_mut:  bool,
    pub ty:      Box<Type>,
    pub ctx:     TypeContext,
}

#[derive(Clone)]
pub struct OptionalType {
    pub span:    SpanId,
    pub node_id: ast::NodeId,
    pub ty:      Box<Type>,
    pub ctx:     TypeContext,
}

#[derive(Clone)]
pub struct FnType {
    pub span:      SpanId,
    pub node_id:   ast::NodeId,
    pub is_unsafe: bool,
    pub abi:       Abi,
    pub params:    Vec<(NameId, Box<Type>)>,
    pub return_ty: Option<Box<Type>>,
    pub ctx:       TypeContext,
}

// =============================================================================================================================

#[derive(Clone)]
pub struct GenericParams {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub params:  Vec<GenericParam>,
    pub pack:    Option<GenericParamPack>,
}

#[derive(Clone)]
pub enum GenericParam {
    Type(GenericTypeParam),
    TypeSpec(GenericTypeSpec),
    Const(GenericConstParam),
    ConstSpec(GenericConstSpec),
}

#[derive(Clone)]
pub struct GenericTypeParam {
    pub span: SpanId,
    pub name: NameId,
    pub def:  Option<Box<Type>>,
}

#[derive(Clone)]
pub struct GenericTypeSpec {
    pub span: SpanId,
    pub ty:   Box<Type>,
}

#[derive(Clone)]
pub struct GenericConstParam {
    pub span: SpanId,
    pub name: NameId,
    pub ty:   Box<Type>,
    pub def:  Option<Box<Expr>>,
}

#[derive(Clone)]
pub struct GenericConstSpec {
    pub span: SpanId,
    pub expr: Box<Block>,
}


#[derive(Clone)]
pub struct GenericParamPack {
    pub span:  SpanId,
    pub elems: Vec<GenericParamPackElem>,
}

#[derive(Clone)]
pub enum GenericParamPackElem {
    Type {
        name:      NameId,
        name_span: SpanId,
        ty_span:   SpanId,
        defs:      Vec<Box<Type>>,
    },
    Const {
        name:      NameId,
        name_span: SpanId,
        ty:        Box<Type>,
        defs:      Vec<Box<Expr>>,
    }
}

#[derive(Clone)]
pub struct GenericArgs {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub args:    Vec<GenericArg>,
}

#[derive(Clone)]
pub enum GenericArg {
    Type(Box<Type>),
    Value(Box<Expr>),
    Name(SpanId, NameId)
}

#[derive(Clone)]
pub struct WhereClause {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub bounds:  Vec<WhereBound>,
}

#[derive(Clone)]
pub enum WhereBound {
    Type {
        span:   SpanId,
        ty:     Box<Type>,
        bounds: Vec<Box<Path>>,
    },
    Explicit {
        span:   SpanId,
        ty:     Box<Type>,
        bounds: Vec<Box<Type>>,
    },
    Expr {
        expr: Box<Expr>,
    },
}


#[derive(Clone)]
pub struct TraitBounds {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub bounds:  Vec<Box<Path>>,
}

// =============================================================================================================================

pub struct Contract {

}

// =============================================================================================================================

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
    pub path:    SimplePath,
    pub metas:   Vec<AttrMeta>,
}

#[derive(Clone)]
pub enum AttrMeta {
    Simple {
        path: SimplePath,
    },
    Expr{
        expr: Box<Expr>
    },
    Assign {
        span: SpanId,
        path: SimplePath,
        expr: Box<Expr>
    },
    Meta {
        span:  SpanId,
        path:  SimplePath,
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

//----------------------------------------------

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

//----------------------------------------------

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

//----------------------------------------------

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

//----------------------------------------------

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

//----------------------------------------------

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

//----------------------------------------------

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

//----------------------------------------------

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

//----------------------------------------------

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

//----------------------------------------------

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

//----------------------------------------------

pub struct TraitContext {
    pub scope:   Scope,
    pub sym:     Option<SymbolRef>,
    pub dag_idx: u32,
}

impl TraitContext {
    pub fn new(scope: Scope) -> Self {
        Self {
            scope,
            sym: None,
            dag_idx: u32::MAX,
        }
    }
}

//----------------------------------------------

pub struct ImplContext {
    pub name:        NameId,
    pub scope:       Scope,
    pub sym:         Option<SymbolRef>,
    pub trait_sym:   Option<SymbolRef>,
    pub trait_items: Vec<(TraitItemRecord, bool)>,
}

impl ImplContext {
    pub fn new(name: NameId, scope: Scope) -> Self {
        Self {
            name,
            scope,
            sym: None,
            trait_sym: None,
            trait_items: Vec::new(),
        }
    }
}

//----------------------------------------------

pub struct OpTraitContext {
    pub scope:            Scope,
    pub sym:              Option<SymbolRef>,
    pub has_generics:     bool,
    pub has_output_alias: bool,
    pub dag_idx:          u32,
}

impl OpTraitContext {
    pub fn new(scope: Scope) -> Self {
        Self {
            scope,
            sym: None,
            has_generics: false,
            has_output_alias: false,
            dag_idx: u32::MAX,
        }
    }
}

//----------------------------------------------

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

//----------------------------------------------

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

//----------------------------------------------

pub struct PrecedenceContext {
    pub scope:      Scope,
    pub sym:        Option<SymbolRef>,
    pub is_highest: bool,
    pub is_lowest:  bool,
}

impl PrecedenceContext {
    pub fn new(scope: Scope) -> Self {
        Self {
            scope,
            sym: None,
            is_highest: false,
            is_lowest: false,
        }
    }
}

// =============================================================================================================================

pub type Ref<T> = Arc<RwLock<T>>;

pub struct Hir {
    pub functions:                 Vec<(Function, FunctionContext)>,
    pub extern_functions_no_body:  Vec<(ExternFunctionNoBody, FunctionContext)>,
    pub type_aliases:              Vec<(TypeAlias, TypeAliasContext)>,
    pub distinct_types:            Vec<(DistinctType, TypeAliasContext)>,
    pub opaque_types:              Vec<(OpaqueType, TypeAliasContext)>,
    pub structs:                   Vec<(Struct, StructContext)>,
    pub tuple_structs:             Vec<(TupleStruct, StructContext)>,
    pub unit_structs:              Vec<(UnitStruct, StructContext)>,
    pub unions:                    Vec<(Union, UnionContext)>,
    pub adt_enums:                 Vec<(AdtEnum, AdtEnumContext)>,
    pub flag_enums:                Vec<(FlagEnum, FlagEnumContext)>,
    pub bitfields:                 Vec<(Bitfield, BitfieldContext)>,
    pub consts:                    Vec<(Const, ConstContext)>,
    pub statics:                   Vec<(Static, StaticContext)>,
    pub tls_statics:               Vec<(TlsStatic, StaticContext)>,
    pub extern_statics:            Vec<(ExternStatic, StaticContext)>,
    
    // trait items store the index into the traits array, as it cannot have any traits removed
    pub traits:                    Vec<(Ref<Trait>, Ref<TraitContext>)>,
    pub trait_functions:           Vec<(usize, TraitFunction, FunctionContext)>,
    pub trait_methods:             Vec<(usize, TraitMethod, FunctionContext)>,
    pub trait_type_alias:          Vec<(usize, TraitTypeAlias, TypeAliasContext)>,
    pub trait_consts:              Vec<(usize, TraitConst, ConstContext)>,
    pub trait_properties:          Vec<(usize, TraitProperty, PropertyContext)>,
    
    // assoc items store the index into the impls array, as it cannot have any impl removed
    pub impls:                     Vec<(Ref<Impl>, Ref<ImplContext>)>,
    pub impl_functions:            Vec<(usize, Function, FunctionContext)>,
    pub methods:                   Vec<(usize, Method, FunctionContext)>,
    pub impl_type_aliases:         Vec<(usize, TypeAlias, TypeAliasContext)>,
    pub impl_consts:               Vec<(usize, Const, ConstContext)>,
    pub impl_statics:              Vec<(usize, Static, StaticContext)>,
    pub impl_tls_statics:          Vec<(usize, TlsStatic, StaticContext)>,
    pub properties:                Vec<(usize, Property, PropertyContext)>,

    // op items store the index into the op_traits array, as it cannot have any op_trait removed
    pub op_traits:                 Vec<(Ref<OpTrait>, Ref<OpTraitContext>)>,
    pub op_functions:              Vec<(usize, OpFunction, OpFunctionContext)>,
    pub op_contracts:              Vec<(usize, OpContract, OpContractContext)>,

    pub precedences:               Vec<(Precedence, Ref<PrecedenceContext>)>,
}

impl Hir {
    pub fn new() -> Self {
        Self {
            functions:                 Vec::new(),
            extern_functions_no_body:  Vec::new(),
            type_aliases:              Vec::new(),
            distinct_types:            Vec::new(),
            opaque_types:              Vec::new(),
            structs:                   Vec::new(),
            tuple_structs:             Vec::new(),
            unit_structs:              Vec::new(),
            unions:                    Vec::new(),
            adt_enums:                 Vec::new(),
            flag_enums:                Vec::new(),
            bitfields:                 Vec::new(),
            consts:                    Vec::new(),
            statics:                   Vec::new(),
            tls_statics:               Vec::new(),
            extern_statics:            Vec::new(),
            
            traits:                    Vec::new(),
            trait_functions:           Vec::new(),
            trait_methods:             Vec::new(),
            trait_type_alias:          Vec::new(),
            trait_consts:              Vec::new(),
            trait_properties:          Vec::new(),
            
            impls:                     Vec::new(),
            impl_functions:            Vec::new(),
            methods:                   Vec::new(),
            impl_type_aliases:         Vec::new(),
            impl_consts:               Vec::new(),
            impl_statics:              Vec::new(),
            impl_tls_statics:          Vec::new(),
            properties:                Vec::new(),

            op_traits:                 Vec::new(),
            op_functions:              Vec::new(),
            op_contracts:              Vec::new(),

            precedences:               Vec::new(),
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
        let impl_idx = self.impls.len() - 1;
        self.methods.push((impl_idx, item, ctx));
    }

    pub fn add_type_alias(&mut self, in_impl: bool, scope: Scope, item: TypeAlias) {
        let ctx = TypeAliasContext::new(scope);
        if in_impl {
            let impl_idx = self.impls.len() - 1;
            self.impl_type_aliases.push((impl_idx, item, ctx));
        } else {
            self.type_aliases.push((item, ctx));
        }
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

    //--------------------------------------------------------------

    pub fn add_trait(&mut self, scope: Scope, item: Trait) {
        let item = Arc::new(RwLock::new(item));
        let ctx = Arc::new(RwLock::new(TraitContext::new(scope)));
        self.traits.push((item, ctx));
    }

    pub fn add_trait_function(&mut self, scope: Scope, item: TraitFunction) {
        let ctx = FunctionContext::new(scope);
        let trait_idx = self.traits.len() - 1;
        self.trait_functions.push((trait_idx, item, ctx));
    }

    pub fn add_trait_method(&mut self, scope: Scope, item: TraitMethod) {
        let ctx = FunctionContext::new(scope);
        let trait_idx = self.traits.len() - 1;
        self.trait_methods.push((trait_idx, item, ctx));
    }

    pub fn add_trait_type_alias(&mut self, scope: Scope, item: TraitTypeAlias) {
        let ctx = TypeAliasContext::new(scope);
        let trait_idx = self.traits.len() - 1;
        self.trait_type_alias.push((trait_idx, item, ctx));
    }

    pub fn add_trait_const(&mut self, scope: Scope, item: TraitConst) {
        let ctx = ConstContext::new(scope);
        let trait_idx = self.traits.len() - 1;
        self.trait_consts.push((trait_idx, item, ctx));
    }

    pub fn add_trait_property(&mut self, scope: Scope, item: TraitProperty) {
        let trait_idx = self.traits.len() - 1;
        let ctx = PropertyContext::new(scope);
        self.trait_properties.push((trait_idx, item, ctx));
    }

    //--------------------------------------------------------------
    
    pub fn add_impl(&mut self, name: NameId, scope: Scope, item: Impl) {
        let item = Arc::new(RwLock::new(item));
        let ctx = Arc::new(RwLock::new(ImplContext::new(name, scope)));
        self.impls.push((item, ctx));
    }

    fn find_impl_def_insert_loc<T0, T1>(arr: &[(usize, T0, T1)], impl_idx: usize) -> usize {
        match arr.binary_search_by(|(idx, _, _)| idx.cmp(&impl_idx)) {
            Ok(mut idx) => {
                // Make sure we insert it at the end, while not really necessary, makes it easier to reason about
                // If they would make a significant enough impact in the future, this could be changed
                while idx + 1 < arr.len() && arr[idx + 1].0 == impl_idx {
                    idx += 1;
                }
                idx
            },
            Err(idx) => idx,
        }
    }

    pub fn add_impl_def_function(&mut self, impl_idx: usize, scope: Scope, item: Function, symbol: SymbolRef) {
        let idx = Self::find_impl_def_insert_loc(&self.impl_functions, impl_idx);

        let mut ctx = FunctionContext::new(scope);
        ctx.sym = Some(symbol);

        self.impl_functions.insert(idx, (impl_idx, item, ctx));
    }

    pub fn add_impl_def_method(&mut self, impl_idx: usize, scope: Scope, item: Method, symbol: SymbolRef) {
        let idx = Self::find_impl_def_insert_loc(&self.methods, impl_idx);

        let mut ctx = FunctionContext::new(scope);
        ctx.sym = Some(symbol);

        self.methods.insert(idx, (impl_idx, item, ctx));
    }

    pub fn add_impl_def_type_alias(&mut self, impl_idx: usize, scope: Scope, item: TypeAlias, symbol: SymbolRef) {
        let idx = Self::find_impl_def_insert_loc(&self.impl_type_aliases, impl_idx);

        let mut ctx = TypeAliasContext::new(scope);
        ctx.sym = Some(symbol);

        self.impl_type_aliases.insert(idx, (impl_idx, item, ctx));
    }

    pub fn add_impl_def_const(&mut self, impl_idx: usize, scope: Scope, item: Const, symbol: SymbolRef) {
        let idx = Self::find_impl_def_insert_loc(&self.impl_consts, impl_idx);

        let mut ctx = ConstContext::new(scope);
        ctx.sym = Some(symbol);

        self.impl_consts.insert(idx, (impl_idx, item, ctx));
    }

    pub fn add_impl_def_property(&mut self, impl_idx: usize, scope: Scope, item: Property, symbol: SymbolRef) {
        let idx = Self::find_impl_def_insert_loc(&self.properties, impl_idx);

        let mut ctx = PropertyContext::new(scope);
        ctx.sym = Some(symbol);

        self.properties.insert(idx, (impl_idx, item, ctx));
    }

    //--------------------------------------------------------------
    
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

    pub fn add_op_contract(&mut self, scope: Scope, item: OpContract) {
        let op_idx = self.op_traits.len() - 1;
        let ctx = OpContractContext::new(scope);
        self.op_contracts.push((op_idx, item, ctx));
    }

    //--------------------------------------------------------------
    
    pub fn add_precedence(&mut self, scope: Scope, item: Precedence) {
        let ctx = Arc::new(RwLock::new(PrecedenceContext::new(scope)));
        self.precedences.push((item, ctx));
    }

}