#![allow(unused)]

use std::{
    fmt::{self, write, Write as _},
    io::{Stdout, Write},
    marker::PhantomData,
    ops::{Index, IndexMut},
    path::{self, PathBuf}, sync::Arc,
};

use crate::{
    common::{IndentLogger, NameId, NameTable, OpType, SpanId},
    lexer::{Punctuation, PunctuationId, PuncutationTable, StrongKeyword, TokenStore, WeakKeyword},
    literals::{LiteralId, LiteralTable}, type_system,
};


mod parser;
pub use parser::{Parser, ParserErr};

mod visitor;
pub use visitor::{Visitor, helpers};

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct NodeId(usize);

impl NodeId {
    pub const INVALID: Self = Self(usize::MAX);

    pub fn index(self) -> usize {
        self.0
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub(super) trait AstNodeParseHelper {
    fn set_node_id(&mut self, node_id: NodeId);
}

pub trait AstNode {
    fn span(&self) -> SpanId;
    fn node_id(&self) -> NodeId;
    fn log(&self, logger: &mut AstLogger);
}

pub struct AstNodeMeta {
    pub span:      SpanId,
    pub first_tok: u32,
    pub last_tok:  u32,
}

pub struct Identifier {
    pub span:     SpanId,
    pub name:     NameId,
    pub gen_args: Option<AstNodeRef<GenericArgs>>,
}

impl Identifier {
    pub fn log(&self, logger: &mut AstLogger) {
        logger.log_indented("Identifier", |logger| {
            logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(self.name)));
            if let Some(gen_args) = &self.gen_args {
                logger.log_node_ref(gen_args);
            }
        })
    }
}

pub enum SimplePathStartKind {
    None,
    Inferred,
    SelfPath,
    Super,
}

pub struct SimplePathStart {
    pub span: SpanId,
    pub kind: SimplePathStartKind
}

impl SimplePathStart {
    fn log(&self, logger: &mut AstLogger) {
        match self.kind {
            SimplePathStartKind::None     => {},
            SimplePathStartKind::Inferred => logger.prefixed_logln("Path Start: Inferred"),
            SimplePathStartKind::SelfPath => logger.prefixed_logln("Path Start: self"),
            SimplePathStartKind::Super    => logger.prefixed_logln("Path Start: super"),
        }
    }
}

pub struct SimplePath {
    pub span:  SpanId,
    pub node_id: NodeId,
    pub start: Option<SimplePathStart>,
    pub names: Vec<(NameId, SpanId)>,
}

impl AstNode for SimplePath {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Simple Path", |logger| {
            logger.log_opt(&self.start, |logger, start| start.log(logger));
            for (idx, (name, _)) in self.names.iter().enumerate() {
                if idx == 0 {
                    logger.prefixed_log(logger.resolve_name(*name));
                } else {
                    logger.log_fmt(format_args!(".{}", logger.resolve_name(*name)));
                }
            }
            logger.logln("");
        })
    }
}

impl AstNodeParseHelper for SimplePath {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub enum TypePathIdentifier {
    Plain {
        span: SpanId,
        name: NameId
    },
    GenArg{
        span:     SpanId,
        name:     NameId,
        gen_args: AstNodeRef<GenericArgs>,
    },
    Fn {
        span:   SpanId,
        name:   NameId,
        params: Vec<Type>,
        ret:    Option<Type>
    },
}

pub struct ExprPath {
    pub span:     SpanId,
    pub node_id:  NodeId,
    pub inferred: bool,
    pub idens:    Vec<Identifier>,
}

impl AstNode for ExprPath {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Expr Path", |logger| {
            logger.prefixed_log_fmt(format_args!("Is Inferred: {}\n", self.inferred));
            logger.set_last_at_indent();
            logger.log_indented_slice("Identifiers", &self.idens, |logger, iden| iden.log(logger));
        });
    }
}

impl AstNodeParseHelper for ExprPath {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct TypePath {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub idens:   Vec<TypePathIdentifier>,
}

impl AstNode for TypePath {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Type Path", |logger| for (idx, iden) in self.idens.iter().enumerate() {
            if idx == self.idens.len() - 1 {
                logger.set_last_at_indent();
            }

            match iden {
                TypePathIdentifier::Plain { span:_, name } => {
                    logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                },
                TypePathIdentifier::GenArg { span:_, name, gen_args } => logger.log_indented("Identifier", |logger| {
                    logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                    logger.set_last_at_indent();
                    logger.log_node_ref(gen_args);
                }),
                TypePathIdentifier::Fn { span:_, name, params, ret } => logger.log_indented("Function Identifier", |logger| {
                    logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                }),
            }
        })
    }
}

impl AstNodeParseHelper for TypePath {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct QualifiedPath {
    pub span:     SpanId,
    pub node_id:  NodeId,
    pub ty:       Type,
    pub bound:    Option<AstNodeRef<TypePath>>,
    pub sub_path: Identifier,
}

impl AstNode for QualifiedPath {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id 
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_indented("Qualified Path", |logger| {
            logger.log_indented_node("Type", &self.ty);
            if let Some(bound) = &self.bound {
                logger.log_indented_node_ref("Bound", bound);
            }
            
            logger.set_last_at_indent();
            logger.log_indented("Sub Path", |logger| self.sub_path.log(logger));
        })
    }
}

impl AstNodeParseHelper for QualifiedPath {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct Block {
    pub span:       SpanId,
    pub node_id:    NodeId,
    pub stmts:      Vec<Stmt>,
    pub final_expr: Option<AstNodeRef<ExprStmt>>,
}

impl AstNode for Block {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Block", |logger| {
            logger.set_last_at_indent_if(self.final_expr.is_none());
            logger.log_indented_node_slice("Statements", &self.stmts);
            logger.set_last_at_indent();
            logger.log_indented_opt_node_ref("Final expression", &self.final_expr);
        });
    }
}

impl AstNodeParseHelper for Block {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

// =============================================================================================================================

pub enum Item {
    Module(AstNodeRef<ModuleItem>),
    Use(AstNodeRef<UseItem>),
    Function(AstNodeRef<Function>),
    TypeAlias(AstNodeRef<TypeAlias>),
    Struct(AstNodeRef<Struct>),
    Union(AstNodeRef<Union>),
    Enum(AstNodeRef<Enum>),
    Bitfield(AstNodeRef<Bitfield>),
    Const(AstNodeRef<Const>),
    Static(AstNodeRef<Static>),
    Property(AstNodeRef<Property>),
    Trait(AstNodeRef<Trait>),
    Impl(AstNodeRef<Impl>),
    Extern(AstNodeRef<ExternBlock>),
    OpTrait(AstNodeRef<OpTrait>),
    OpUse(AstNodeRef<OpUse>),
    Precedence(AstNodeRef<Precedence>),
    PrecedenceUse(AstNodeRef<PrecedenceUse>),
}

impl AstNode for Item {
    fn span(&self) -> SpanId {
        match self {
            Item::Use(item)           => item.span(),
            Item::Module(item)        => item.span(),
            Item::Function(item)      => item.span(),
            Item::TypeAlias(item)     => item.span(),
            Item::Struct(item)        => item.span(),
            Item::Union(item)         => item.span(),
            Item::Enum(item)          => item.span(),
            Item::Bitfield(item)      => item.span(),
            Item::Const(item)         => item.span(),
            Item::Static(item)        => item.span(),
            Item::Property(item)      => item.span(),
            Item::Trait(item)         => item.span(),
            Item::Impl(item)          => item.span(),
            Item::Extern(item)        => item.span(),
            Item::OpTrait(item)       => item.span(),
            Item::OpUse(item)         => item.span(),
            Item::Precedence(item)    => item.span(),
            Item::PrecedenceUse(item) => item.span(),
        }
    }
    
    fn node_id(&self) -> NodeId {
        match self {
            Item::Use(item)           => item.node_id(),
            Item::Module(item)        => item.node_id(),
            Item::Function(item)      => item.node_id(),
            Item::TypeAlias(item)     => item.node_id(),
            Item::Struct(item)        => item.node_id(),
            Item::Union(item)         => item.node_id(),
            Item::Enum(item)          => item.node_id(),
            Item::Bitfield(item)      => item.node_id(),
            Item::Const(item)         => item.node_id(),
            Item::Static(item)        => item.node_id(),
            Item::Property(item)      => item.node_id(),
            Item::Trait(item)         => item.node_id(),
            Item::Impl(item)          => item.node_id(),
            Item::Extern(item)        => item.node_id(),
            Item::OpTrait(item)       => item.node_id(),
            Item::OpUse(item)         => item.node_id(),
            Item::Precedence(item)    => item.node_id(),
            Item::PrecedenceUse(item) => item.node_id(),
        }
    }

    fn log(&self, logger: &mut AstLogger) {
        match self {
            Self::Module(item)        => logger.log_node_ref(item),
            Self::Use(item)           => logger.log_node_ref(item),
            Self::Function(item)      => logger.log_node_ref(item),
            Self::TypeAlias(item)     => logger.log_node_ref(item),
            Self::Struct(item)        => logger.log_node_ref(item),
            Self::Union(item)         => logger.log_node_ref(item),
            Self::Enum(item)          => logger.log_node_ref(item),
            Self::Bitfield(item)      => logger.log_node_ref(item),
            Self::Const(item)         => logger.log_node_ref(item),
            Self::Static(item)        => logger.log_node_ref(item),
            Self::Property(item)      => logger.log_node_ref(item),
            Self::Trait(item)         => logger.log_node_ref(item),
            Self::Impl(item)          => logger.log_node_ref(item),
            Self::Extern(item)        => logger.log_node_ref(item),
            Self::OpTrait(item)       => logger.log_node_ref(item),
            Self::OpUse(item)         => logger.log_node_ref(item),
            Self::Precedence(item)    => logger.log_node_ref(item),
            Self::PrecedenceUse(item) => logger.log_node_ref(item),
        }
    }
}

pub enum TraitItem {
    Function(AstNodeRef<Function>),
    TypeAlias(AstNodeRef<TypeAlias>),
    Const(AstNodeRef<Const>),
    Property(AstNodeRef<Property>),
}

impl AstNode for TraitItem {
    fn span(&self) -> SpanId {
        match self {
            TraitItem::Function(item)  => item.span(),
            TraitItem::TypeAlias(item) => item.span(),
            TraitItem::Const(item)     => item.span(),
            TraitItem::Property(item)  => item.span(),
        }
    }
    
    fn node_id(&self) -> NodeId {
        match self {
            TraitItem::Function(item)  => item.node_id(),
            TraitItem::TypeAlias(item) => item.node_id(),
            TraitItem::Const(item)     => item.node_id(),
            TraitItem::Property(item)  => item.node_id(),
        }
    }

    fn log(&self, logger: &mut AstLogger) {
        match self {
            Self::Function(fn_item)       => logger.log_node_ref(fn_item),
            Self::TypeAlias(type_alias)   => logger.log_node_ref(type_alias),
            Self::Const(const_item)       => logger.log_node_ref(const_item),
            Self::Property(prop_item)     => logger.log_node_ref(prop_item),
        }
    }
}


pub enum AssocItem {
    Function(AstNodeRef<Function>),
    TypeAlias(AstNodeRef<TypeAlias>),
    Const(AstNodeRef<Const>),
    Static(AstNodeRef<Static>),
    Property(AstNodeRef<Property>),
}

impl AstNode for AssocItem {
    fn span(&self) -> SpanId {
        match self {
            AssocItem::Function(item)  => item.span(),
            AssocItem::TypeAlias(item) => item.span(),
            AssocItem::Const(item)     => item.span(),
            AssocItem::Static(item)    => item.span(),
            AssocItem::Property(item)  => item.span(),
        }
    }

    fn node_id(&self) -> NodeId {
        match self {
            AssocItem::Function(item)  => item.node_id(),
            AssocItem::TypeAlias(item) => item.node_id(),
            AssocItem::Const(item)     => item.node_id(),
            AssocItem::Static(item)    => item.node_id(),
            AssocItem::Property(item)  => item.node_id(),
        }
    }

    fn log(&self, logger: &mut AstLogger) {
        match self {
            Self::Function(fn_item)       => logger.log_node_ref(fn_item),
            Self::TypeAlias(type_alias)   => logger.log_node_ref(type_alias),
            Self::Const(const_item)       => logger.log_node_ref(const_item),
            Self::Static(static_item)     => logger.log_node_ref(static_item),
            Self::Property(prop_item)     => logger.log_node_ref(prop_item),
        }
    }
}

pub enum ExternItem {
    Function(AstNodeRef<Function>),
    Static(AstNodeRef<Static>),
}

impl AstNode for ExternItem {
    fn span(&self) -> SpanId {
        match self {
            ExternItem::Function(item) => item.span(),
            ExternItem::Static(item)   => item.span(),
        }    
    }

    fn node_id(&self) -> NodeId {
        match self {
            ExternItem::Function(item) => item.node_id(),
            ExternItem::Static(item)   => item.node_id(),
        }    
    }

    fn log(&self, logger: &mut AstLogger) {
        match self {
            Self::Function(fn_item)       => logger.log_node_ref(fn_item),
            Self::Static(static_item)     => logger.log_node_ref(static_item),
        }
    }
}

pub struct ModuleItem {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub attrs:   Vec<AstNodeRef<Attribute>>,
    pub vis:     Option<AstNodeRef<Visibility>>,
    pub name:    NameId,
    pub block:   Option<AstNodeRef<Block>>,
}

impl AstNode for ModuleItem {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_indented("Module", |logger| {
            logger.log_indented_node_ref_slice("Attributes", &self.attrs);
            logger.log_opt_node_ref(&self.vis);
            logger.prefixed_log_fmt(format_args!("Module: {}\n", logger.resolve_name(self.name)));
            logger.set_last_at_indent();
            logger.log_indented_opt_node_ref("Body", &self.block);
        });
    }
}

impl AstNodeParseHelper for ModuleItem {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}


pub struct UseItem {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub attrs:   Vec<AstNodeRef<Attribute>>,
    pub vis:     Option<AstNodeRef<Visibility>>,
    pub group:   Option<NameId>,
    pub package: Option<NameId>,
    pub module:  Option<NameId>,
    pub path:    AstNodeRef<UsePath>,
}

impl AstNode for UseItem {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Use Declaration", |logger| {
            logger.log_indented_node_ref_slice("Attributes", &self.attrs);
            logger.log_opt_node_ref(&self.vis);

            if let Some(group) = &self.group {
                logger.prefixed_log_fmt(format_args!("Group: {}\n", logger.resolve_name(*group)));
            }
            if let Some(package) = &self.package {
                logger.prefixed_log_fmt(format_args!("Package: {}\n", logger.resolve_name(*package)));
            }
            if let Some(module) = &self.module {
                logger.prefixed_log_fmt(format_args!("Module: {}\n", logger.resolve_name(*module)));
            }
            
            logger.set_last_at_indent();
            logger.log_node_ref(&self.path);
        });                                         
    }
}

impl AstNodeParseHelper for UseItem {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub enum UsePath {
    SelfPath{
        span:    SpanId,
        node_id: NodeId,
        alias:   Option<NameId>
    },
    SubPaths{
        span:      SpanId,
        node_id:   NodeId,
        segments:  Vec<NameId>,
        sub_paths: Vec<AstNodeRef<UsePath>>,
    },
    Alias{
        span:     SpanId,
        node_id:  NodeId,
        segments: Vec<NameId>,
        alias:    Option<NameId>,
    }
}
impl AstNode for UsePath {
    fn span(&self) -> SpanId {
        match self {
            UsePath::SelfPath { span, .. } => *span,
            UsePath::SubPaths { span, .. } => *span,
            UsePath::Alias { span, .. }    => *span,
        }    
    }

    fn node_id(&self) -> NodeId {
        match self {
            UsePath::SelfPath { node_id, .. } => *node_id,
            UsePath::SubPaths { node_id, .. } => *node_id,
            UsePath::Alias { node_id, .. }    => *node_id,
        }    
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Use Path", |logger| {
            match self {
                UsePath::SelfPath { span, node_id, alias } => {
                    logger.set_last_at_indent();
                    logger.prefixed_log("SelfPath");
                    
                    if let Some(alias) = alias {
                        logger.log_fmt(format_args!(", alias: {}\n", logger.resolve_name(*alias)));
                    }
                    logger.logln("");
                },
                UsePath::SubPaths { span, node_id, segments, sub_paths } => {
                    logger.prefixed_log("Path: ");
                    
                    for (idx, segment) in segments.iter().enumerate() {
                        if idx != 0 {
                            logger.log(".");
                        }
                        logger.log(logger.resolve_name(*segment));
                    }
                    logger.logln("");

                    logger.set_last_at_indent();
                    logger.log_indented_node_ref_slice("Sub Paths", sub_paths);
                },
                UsePath::Alias { span, node_id, segments, alias } => {
                    logger.set_last_at_indent();
                    logger.prefixed_log("Path: ");

                    for (idx, segment) in segments.iter().enumerate() {
                        if idx != 0 {
                            logger.log(".");
                        }
                        logger.log(logger.resolve_name(*segment));
                    }

                    if let Some(alias) = alias {
                        logger.log_fmt(format_args!(", alias: {}", logger.resolve_name(*alias)));
                    }

                    logger.logln("");
                },
            }
        });
    }
}

impl AstNodeParseHelper for UsePath {
    fn set_node_id(&mut self, ast_node_id: NodeId) {
        match self {
            UsePath::SelfPath { node_id, .. } => *node_id = ast_node_id,
            UsePath::SubPaths { node_id, .. } => *node_id = ast_node_id,
            UsePath::Alias { node_id, .. }    => *node_id = ast_node_id,
        }
    }
}

pub struct Function {
    pub span:         SpanId,
    pub node_id:      NodeId,
    pub attrs:        Vec<AstNodeRef<Attribute>>,
    pub vis:          Option<AstNodeRef<Visibility>>,
    pub is_override:  bool,
    pub is_const:     bool,
    pub is_unsafe:    bool,
    pub abi:          Option<LiteralId>,
    pub name:         NameId,
    pub generics:     Option<AstNodeRef<GenericParams>>,
    pub receiver:     Option<FnReceiver>,
    pub params:       Vec<FnParam>,
    pub returns:      Option<FnReturn>,
    pub where_clause: Option<AstNodeRef<WhereClause>>,
    pub contracts:    Vec<AstNodeRef<Contract>>,
    pub body:         Option<AstNodeRef<Block>>,
}

impl AstNode for Function {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Function", |logger| {
            logger.log_indented_node_ref_slice("Attributes", &self.attrs);
            logger.log_opt_node_ref(&self.vis);

            logger.prefixed_log_fmt(format_args!("Is Override: {}\n", self.is_override));
            logger.prefixed_log_fmt(format_args!("Is Const: {}\n", self.is_const));
            logger.prefixed_log_fmt(format_args!("Is Unsafe: {}\n", self.is_unsafe));
            if let Some(abi) = self.abi {
                logger.prefixed_log_fmt(format_args!("ABI: {}\n", logger.resolve_name(self.name)));
            }
            logger.set_last_at_indent_if(self.generics.is_none() && self.generics.is_none() && self.receiver.is_none() && self.params.is_empty() && self.body.is_none());
            logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(self.name)));
            
           
            logger.set_last_at_indent_if(self.generics.is_none() && self.receiver.is_none() && self.params.is_empty() && self.body.is_none());
            logger.log_opt_node_ref(&self.generics);
            logger.set_last_at_indent_if(self.receiver.is_none() && self.params.is_empty() && self.body.is_none());
            logger.log_opt(&self.receiver, |logger, rec| rec.log(logger));
            logger.set_last_at_indent_if(self.params.is_empty() && self.body.is_none());
            logger.log_indented_slice("Params", &self.params, |logger, param| param.log(logger));
            logger.set_last_at_indent_if(self.body.is_none());
            logger.log_opt(&self.returns, |logger, ret| ret.log(logger));
            logger.set_last_at_indent();
            if let Some(body) = &self.body {
                logger.log_node_ref(body);
            }
        })
    }
}

impl AstNodeParseHelper for Function {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub enum FnReceiver {
    SelfReceiver{
        span:    SpanId,
        is_ref:  bool,
        is_mut:  bool,
    },
    SelfTyped{
        span:    SpanId,
        is_mut:  bool,
        ty:      Type,  
    },
}

impl FnReceiver {
    pub fn log(&self, logger: &mut AstLogger) {
        match self {
            FnReceiver::SelfReceiver { span, is_ref, is_mut } => logger.log_indented("Self Receiver", |logger| {
                logger.prefixed_log_fmt(format_args!("Is Ref: {is_ref}\n"));
                logger.prefixed_log_fmt(format_args!("Is Mut: {is_mut}\n"));
            }),
            FnReceiver::SelfTyped{ span, is_mut, ty } => logger.log_indented("Typed Receiver", |logger| {
                logger.prefixed_log_fmt(format_args!("Is Mut: {is_mut}\n"));
                logger.set_last_at_indent();
                logger.log_indented_node("Type", ty);
            }),
        }
    }
}

pub struct FnParam {
    pub span:        SpanId,
    pub names:       Vec<FnParamName>,
    pub ty:          Type,
    pub is_variadic: bool,
    pub def_val:     Option<Expr>,
}

impl FnParam {
    pub fn log(&self, logger: &mut AstLogger) {
        logger.log_indented("Param", |logger| {
            logger.prefixed_log_fmt(format_args!("Is Variadic: {}\n", self.is_variadic));
            logger.log_indented_slice("Names", &self.names, |logger, name| name.log(logger));
            logger.set_last_at_indent();
            logger.log_indented_node("Type", &self.ty); 
        })
    }
}

pub struct FnParamName {
    pub span:    SpanId,
    pub attrs:   Vec<AstNodeRef<Attribute>>,
    pub label:   Option<NameId>,
    pub pattern: Pattern,
}

impl FnParamName {
    pub fn log(&self, logger: &mut AstLogger) {
        logger.log_indented("Name", |logger| {
            logger.log_indented_node_ref_slice("Attributes", &self.attrs);
            if let Some(label) = &self.label {
                logger.prefixed_log_fmt(format_args!("Label: {}\n", logger.resolve_name(*label)));
            }
            logger.set_last_at_indent();
            logger.log_node(&self.pattern);
        })
    }
}

pub enum FnReturn {
    Type{
        span: SpanId,
        ty:   Type
    },
    Named{
        span: SpanId,
        vars: Vec<(Vec<(NameId, SpanId)>, Type)>
    }
}

impl FnReturn {
    pub fn log(&self, logger: &mut AstLogger) {
        match self {
            FnReturn::Type{ span, ty } => logger.log_indented_node("Typed Function Return", ty),
            FnReturn::Named{ span, vars } => logger.log_indented_slice("Named Function Return", vars, |logger, (names, ty)| {
                logger.log_indented("Return", |logger| {
                    logger.log_indented_slice("Names", &names, |logger, (name, _)| {
                        logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                    });

                    logger.set_last_at_indent();
                    logger.log_indented_node("Type", ty);
                })
            }),
        }
    }
}

pub enum TypeAlias {
    Normal {
        span:     SpanId,
        node_id:  NodeId,
        attrs:    Vec<AstNodeRef<Attribute>>,
        vis:      Option<AstNodeRef<Visibility>>,
        name:     NameId,
        generics: Option<AstNodeRef<GenericParams>>,
        ty:       Type,
    },
    Distinct {
        span:     SpanId,
        node_id:  NodeId,
        attrs:    Vec<AstNodeRef<Attribute>>,
        vis:      Option<AstNodeRef<Visibility>>,
        name:     NameId,
        generics: Option<AstNodeRef<GenericParams>>,
        ty:       Type,
    },
    Trait {
        span:     SpanId,
        node_id:  NodeId,
        attrs:    Vec<AstNodeRef<Attribute>>,
        name:     NameId,
        generics: Option<AstNodeRef<GenericParams>>,
    },
    Opaque {
        span:     SpanId,
        node_id : NodeId,
        attrs:    Vec<AstNodeRef<Attribute>>,
        vis:      Option<AstNodeRef<Visibility>>,
        name:     NameId,
        size:     Option<Expr>,
    }
}

impl AstNode for TypeAlias {
    fn span(&self) -> SpanId {
        match self {
            TypeAlias::Normal { span, .. }   => *span,
            TypeAlias::Distinct { span, .. } => *span,
            TypeAlias::Trait { span, .. }    => *span,
            TypeAlias::Opaque { span, .. }   => *span,
        }    
    }

    fn node_id(&self) -> NodeId {
        match self {
            TypeAlias::Normal { node_id, .. }   => *node_id,
            TypeAlias::Distinct { node_id, .. } => *node_id,
            TypeAlias::Trait { node_id, .. }    => *node_id,
            TypeAlias::Opaque { node_id, .. }   => *node_id,
        }    
    }

    fn log(&self, logger: &mut AstLogger) {
        match self {
            TypeAlias::Normal { span, node_id, attrs, vis, name, generics, ty } => logger.log_ast_node("Typealias", |logger| {
                logger.log_indented_node_ref_slice("Attributes", attrs);
                logger.log_opt_node_ref(vis);
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                logger.log_opt_node_ref(generics);
                logger.set_last_at_indent();
                logger.log_indented_node("Type", ty);
            }),
            TypeAlias::Distinct { span, node_id, attrs, vis, name, generics, ty } => logger.log_ast_node("Distinct Typealias", |logger| {
                logger.log_indented_node_ref_slice("Attributes", attrs);
                logger.log_opt_node_ref(vis);
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                logger.log_opt_node_ref(generics);
                logger.set_last_at_indent();
                logger.log_indented_node("Type", ty);
            }),
            TypeAlias::Trait { span, node_id, attrs, name, generics } => logger.log_ast_node("Trait Typealias", |logger| {
                logger.log_indented_node_ref_slice("Attributes", attrs);
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                logger.set_last_at_indent();
                logger.log_opt_node_ref(generics);
            }),
            TypeAlias::Opaque { span, node_id, attrs, vis, name, size } => logger.log_ast_node("Opaque Typealias", |logger| {
                logger.log_indented_node_ref_slice("Attributes", attrs);
                logger.log_opt_node_ref(vis);
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                logger.log_indented_opt_node("Size", size);
            }),
        }
    }
}

impl AstNodeParseHelper for TypeAlias {
    fn set_node_id(&mut self, ast_node_id: NodeId) {
        match self {
            TypeAlias::Normal { node_id, .. }   => *node_id = ast_node_id,
            TypeAlias::Distinct { node_id, .. } => *node_id = ast_node_id,
            TypeAlias::Trait { node_id, .. }    => *node_id = ast_node_id,
            TypeAlias::Opaque { node_id, .. }   => *node_id = ast_node_id,
        }
    }
}

pub enum Struct {
    Regular {
        span:         SpanId,
        node_id: NodeId,
        attrs:        Vec<AstNodeRef<Attribute>>,
        vis:          Option<AstNodeRef<Visibility>>,
        is_mut:       bool,
        is_record:    bool,
        name:         NameId,
        generics:     Option<AstNodeRef<GenericParams>>,
        where_clause: Option<AstNodeRef<WhereClause>>,
        fields:       Vec<RegStructField>,
    },
    Tuple {
        span:         SpanId,
        node_id: NodeId,
        attrs:        Vec<AstNodeRef<Attribute>>,
        vis:          Option<AstNodeRef<Visibility>>,
        is_mut:       bool,
        is_record:    bool,
        name:         NameId,
        generics:     Option<AstNodeRef<GenericParams>>,
        where_clause: Option<AstNodeRef<WhereClause>>,
        fields:       Vec<TupleStructField>,
    },
    Unit {
        span:         SpanId,
        node_id: NodeId,
        attrs:        Vec<AstNodeRef<Attribute>>,
        vis:          Option<AstNodeRef<Visibility>>,
        name:         NameId,
    }
}

impl AstNode for Struct {
    fn span(&self) -> SpanId {
        match self {
            Struct::Regular { span, .. } => *span,
            Struct::Tuple { span, .. }   => *span,
            Struct::Unit { span, .. }    => *span,
        }    
    }

    fn node_id(&self) -> NodeId {
        match self {
            Struct::Regular { node_id, .. } => *node_id,
            Struct::Tuple { node_id, .. }   => *node_id,
            Struct::Unit { node_id, .. }    => *node_id,
        }    
    }

    fn log(&self, logger: &mut AstLogger) {
        match self {
            Struct::Regular { span, node_id, attrs, vis, is_mut, is_record, name, generics, where_clause, fields } => logger.log_ast_node("Struct", |logger| {
                logger.log_indented_node_ref_slice("Attributes", attrs);
                logger.log_opt_node_ref(vis);

                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                logger.prefixed_log_fmt(format_args!("Is Mut: {is_mut}\n"));
                logger.prefixed_log_fmt(format_args!("Is Record: {is_record}\n"));

                logger.set_last_at_indent_if(where_clause.is_none() && fields.is_empty());
                logger.log_opt_node_ref(generics);
                
                logger.set_last_at_indent_if(fields.is_empty());
                logger.log_opt_node_ref(where_clause);

                logger.set_last_at_indent();
                logger.log_indented_slice("Fields", &fields, |logger, field| field.log(logger));
            }),
            Struct::Tuple { span, node_id, attrs, vis, is_mut, is_record, name, generics, where_clause, fields } => logger.log_ast_node("Tuple Struct", |logger| {
                logger.log_indented_node_ref_slice("Attributes", attrs);
                logger.log_opt_node_ref(vis);
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                logger.log_opt_node_ref(generics);
                logger.log_opt_node_ref(where_clause);
                logger.set_last_at_indent();
                logger.log_indented_slice("Fields", fields, |logger, field| field.log(logger));
            }),
            Struct::Unit { span, node_id, attrs, vis, name } => logger.log_ast_node("Unit Struct", |logger| {
                logger.log_indented_node_ref_slice("Attributes", attrs);
                logger.log_opt_node_ref(vis);
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
            }),
        }
    }
}

impl AstNodeParseHelper for Struct {
    fn set_node_id(&mut self, ast_node_id: NodeId) {
        match self {
            Struct::Regular { node_id, .. } => *node_id = ast_node_id,
            Struct::Tuple { node_id, .. }   => *node_id = ast_node_id,
            Struct::Unit { node_id, .. }    => *node_id = ast_node_id,
        }
    }
}

pub enum RegStructField {
    Field {
        span:   SpanId,
        attrs:  Vec<AstNodeRef<Attribute>>,
        vis:    Option<AstNodeRef<Visibility>>,
        is_mut: bool,
        names:  Vec<NameId>,
        ty:     Type,
        def:    Option<Expr>,
    },
    Use {
        span:   SpanId,
        attrs:  Vec<AstNodeRef<Attribute>>,
        vis:    Option<AstNodeRef<Visibility>>,
        is_mut: bool,
        path:   AstNodeRef<TypePath>,
    }
}

impl RegStructField {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            RegStructField::Field { span, attrs, vis, is_mut, names, ty, def } => logger.log_indented("Named Field", |logger| {
                logger.log_indented_node_ref_slice("Attributes", attrs);
                logger.log_opt_node_ref(vis);

                logger.prefixed_log_fmt(format_args!("Is Mut: {is_mut}\n"));

                logger.log_indented_slice("Names", &names, |logger, name| {
                    logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                });

                logger.set_last_at_indent_if(def.is_none());
                logger.log_indented_node("Type", ty);
                logger.set_last_at_indent();
                logger.log_indented_opt_node("Default Value", def);
            }),
            RegStructField::Use { span, attrs, vis, is_mut, path } => logger.log_indented("Use Field", |logger| {
                logger.log_indented_node_ref_slice("Attributes", attrs);
                logger.log_opt_node_ref(vis);
                logger.prefixed_log_fmt(format_args!("Is Mut: {is_mut}\n"));
                logger.set_last_at_indent();
                logger.log_node_ref(path);
            }),
        }
    }
}

pub struct TupleStructField {
    pub span:  SpanId,
    pub attrs: Vec<AstNodeRef<Attribute>>,
    pub vis:   Option<AstNodeRef<Visibility>>,
    pub ty:    Type,
    pub def:   Option<Expr>,
}

impl TupleStructField {
    pub fn log(&self, logger: &mut AstLogger) {
        logger.log_indented("Field", |logger| {
            logger.log_indented_node_ref_slice("Attributes", &self.attrs);
            logger.log_opt_node_ref(&self.vis);
            
            logger.set_last_at_indent_if(self.def.is_none());
            logger.log_indented_node("Type", &self.ty);

            logger.set_last_at_indent();
            logger.log_indented_opt_node("Default", &self.def);
        });
    }
}

pub struct Union {
    pub span:         SpanId,
    pub node_id:      NodeId,
    pub attrs:        Vec<AstNodeRef<Attribute>>,
    pub vis:          Option<AstNodeRef<Visibility>>,
    pub is_mut:       bool,
    pub name:         NameId,
    pub generics:     Option<AstNodeRef<GenericParams>>,
    pub where_clause: Option<AstNodeRef<WhereClause>>,
    pub fields:       Vec<UnionField>,
}

impl AstNodeParseHelper for Union {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

impl AstNode for Union {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Union", |logger| {
            logger.log_indented_node_ref_slice("Attributes", &self.attrs);
            logger.log_opt_node_ref(&self.vis);

            logger.prefixed_log_fmt(format_args!("Is Mut: {}\n", self.is_mut));

            logger.set_last_at_indent_if(self.where_clause.is_none() && self.fields.is_empty());
            logger.log_opt_node_ref(&self.generics);
            
            logger.set_last_at_indent_if(self.fields.is_empty());
            logger.log_opt_node_ref(&self.where_clause);

            logger.set_last_at_indent();
            logger.log_indented_slice("Fields", &self.fields, |logger, field| field.log(logger));
        })
    }
}

pub struct UnionField {
    pub span:   SpanId,
    pub attrs:  Vec<AstNodeRef<Attribute>>,
    pub vis:    Option<AstNodeRef<Visibility>>,
    pub is_mut: bool,
    pub name:   NameId,
    pub ty:     Type,
}

impl UnionField {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_indented("Field", |logger| {
            logger.log_indented_node_ref_slice("Attributes", &self.attrs);
            logger.log_opt_node_ref(&self.vis);
            logger.prefixed_log_fmt(format_args!("Is Mut: {}\n", self.is_mut));
            logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(self.name)));
            logger.set_last_at_indent(); 
            logger.log_indented_node("Type", &self.ty);
        })
    }
}

pub enum Enum {
    Adt {
        span:         SpanId,
        node_id:      NodeId,
        attrs:        Vec<AstNodeRef<Attribute>>,
        vis:          Option<AstNodeRef<Visibility>>,
        is_mut:       bool,
        is_record:    bool,
        name:         NameId,
        generics:     Option<AstNodeRef<GenericParams>>,
        where_clause: Option<AstNodeRef<WhereClause>>,
        variants:     Vec<EnumVariant>,
    },
    Flag {
        span:         SpanId,
        node_id:      NodeId,
        attrs:        Vec<AstNodeRef<Attribute>>,
        vis:          Option<AstNodeRef<Visibility>>,
        name:         NameId,
        variants:     Vec<FlagEnumVariant>,
    }
}

impl AstNode for Enum {
    fn span(&self) -> SpanId {
        match self {
            Enum::Adt { span, .. }  => *span,
            Enum::Flag { span, .. } => *span,
        }    
    }

    fn node_id(&self) -> NodeId {
        match self {
            Enum::Adt { node_id, .. }  => *node_id,
            Enum::Flag { node_id, .. } => *node_id,
        }    
    }

    fn log(&self, logger: &mut AstLogger) {
        match self {
            Enum::Adt { span, node_id, attrs, vis, is_mut, is_record, name, generics, where_clause, variants } => logger.log_ast_node("Adt Enum", |logger| {
                logger.log_indented_node_ref_slice("Attributes", &attrs);
                logger.log_opt_node_ref(vis);

                logger.prefixed_log_fmt(format_args!("Is Mut: {is_mut}\n"));
                logger.prefixed_log_fmt(format_args!("Is Record: {is_record}\n"));
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));

                logger.set_last_at_indent_if(where_clause.is_none() && variants.is_empty());
                logger.log_opt_node_ref(generics);
                
                logger.set_last_at_indent_if(variants.is_empty());
                logger.log_opt_node_ref(where_clause);
    
                logger.set_last_at_indent();
                logger.log_indented_slice("Variants", variants, |logger, variant| variant.log(logger));
            }),
            Enum::Flag { span, node_id, attrs, vis, name, variants } => logger.log_ast_node("Flag Enum", |logger| {
                logger.log_indented_node_ref_slice("Attributes", attrs);
                logger.log_opt_node_ref(vis);
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                logger.set_last_at_indent();
                logger.log_indented_slice("Variants", variants, |logger, variant| variant.log(logger));
            }),
        }
    }
}

impl AstNodeParseHelper for Enum {
    fn set_node_id(&mut self, ast_node_id: NodeId) {
        match self {
            Enum::Adt { node_id, .. }  => *node_id =  ast_node_id,
            Enum::Flag { node_id, .. } => *node_id =  ast_node_id,
        }
    }
}

pub enum EnumVariant {
    Struct {
        span:         SpanId,
        attrs:        Vec<AstNodeRef<Attribute>>,
        is_mut:       bool,
        name:         NameId,
        fields:       Vec<RegStructField>,
        discriminant: Option<Expr>,
    },
    Tuple {
        span:         SpanId,
        attrs:        Vec<AstNodeRef<Attribute>>,
        is_mut:       bool,
        name:         NameId,
        fields:       Vec<TupleStructField>,
        discriminant: Option<Expr>,
    },
    Fieldless {
        span:         SpanId,
        attrs:        Vec<AstNodeRef<Attribute>>,
        name:         NameId,
        discriminant: Option<Expr>,
    }
}

impl EnumVariant {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            EnumVariant::Struct { span, attrs, is_mut, name, fields, discriminant } => logger.log_indented("Struct Variant", |logger| {
                logger.log_indented_node_ref_slice("Attributes", &attrs);
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));

                logger.set_last_at_indent_if(discriminant.is_none());
                logger.log_indented_slice("Fields", fields, |logger, field| field.log(logger));
                logger.set_last_at_indent();
                logger.log_indented_opt_node("Discriminant", discriminant);

            }),
            EnumVariant::Tuple { span, attrs, is_mut, name, fields, discriminant } => logger.log_indented("Tuple Variant", |logger| {
                logger.log_indented_node_ref_slice("Attributes", &attrs);
                
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));

                logger.set_last_at_indent_if(discriminant.is_none());
                logger.log_indented_slice("Fields", fields, |logger, field| field.log(logger));
                logger.set_last_at_indent();
                logger.log_indented_opt_node("Discriminant", discriminant);
            }),
            EnumVariant::Fieldless { span, attrs, name, discriminant } => logger.log_indented("Fieldless Variant", |logger| {
                logger.log_indented_node_ref_slice("Attributes", &attrs);

                logger.set_last_at_indent_if(discriminant.is_none());
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));

                logger.set_last_at_indent();
                logger.log_indented_opt_node("Discriminant", discriminant);
            }),
        }
    }
}

pub struct FlagEnumVariant {
    pub span:         SpanId,
    pub attrs:        Vec<AstNodeRef<Attribute>>,
    pub name:         NameId,
    pub discriminant: Option<Expr>,
}

impl FlagEnumVariant {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_indented("Flag Variant", |logger| {
            logger.log_indented_node_ref_slice("Attributes", &self.attrs);
            logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(self.name)));
            logger.set_last_at_indent();
            logger.log_indented_opt_node("Discriminant", &self.discriminant);
        });
    }
}

pub struct Bitfield {
    pub span:         SpanId,
    pub node_id:      NodeId,
    pub attrs:        Vec<AstNodeRef<Attribute>>,
    pub vis:          Option<AstNodeRef<Visibility>>,
    pub is_mut:       bool,
    pub is_record:    bool,
    pub name:         NameId,
    pub generics:     Option<AstNodeRef<GenericParams>>,
    pub bit_count:    Option<Expr>,
    pub where_clause: Option<AstNodeRef<WhereClause>>,
    pub fields:       Vec<BitfieldField>,
}

impl AstNode for Bitfield {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Bitfield", |logger| {
            logger.log_indented_node_ref_slice("Attributes", &self.attrs);
            logger.log_opt_node_ref(&self.vis);

            logger.prefixed_log_fmt(format_args!("Is Mut: {}\n", self.is_mut));
            logger.prefixed_log_fmt(format_args!("Is Record: {}\n", self.is_record));
            logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(self.name)));

            logger.set_last_at_indent_if(self.bit_count.is_none() && self.where_clause.is_none() && self.fields.is_empty());
            logger.log_opt_node_ref(&self.generics);
            
            logger.set_last_at_indent_if(self.where_clause.is_none() && self.bit_count.is_none() && self.fields.is_empty());
            logger.log_indented_opt_node("Bits", &self.bit_count);

            logger.set_last_at_indent_if(self.fields.is_empty());
            logger.log_opt_node_ref(&self.where_clause);

            logger.set_last_at_indent();
            logger.log_indented_slice("Fields", &self.fields, |logger, field| field.log(logger))
        })
    }
}

impl AstNodeParseHelper for Bitfield {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub enum BitfieldField {
    Field {
        span:   SpanId,
        attrs:  Vec<AstNodeRef<Attribute>>,
        vis:    Option<AstNodeRef<Visibility>>,
        is_mut: bool,
        names:  Vec<NameId>,
        ty:     Type,
        bits:   Option<Expr>,
        def:    Option<Expr>,
    },
    Use {
        span:   SpanId,
        attrs:  Vec<AstNodeRef<Attribute>>,
        vis:    Option<AstNodeRef<Visibility>>,
        is_mut: bool,
        path:   AstNodeRef<TypePath>,
        bits:   Option<Expr>,
    }
}

impl BitfieldField {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            BitfieldField::Field { span, attrs, vis, is_mut, names, ty, bits, def } => logger.log_indented("Field", |logger| {
                logger.log_indented_node_ref_slice("Attributes", &attrs);
                logger.log_opt_node_ref(vis);
                logger.prefixed_log_fmt(format_args!("Is Mut: {is_mut}\n"));

                logger.log_indented_slice("Names", names, |logger, name| {
                    logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                });

                logger.set_last_at_indent_if(bits.is_none() && def.is_none());
                logger.log_indented_node("Type", ty);

                logger.set_last_at_indent_if(def.is_none());
                logger.log_indented_opt_node("Bits", bits);

                logger.set_last_at_indent();
                logger.log_indented_opt_node("Default Value", def);
            }),
            BitfieldField::Use { span, attrs, vis, is_mut, path, bits } => logger.log_indented("Use Field", |logger| {
                logger.log_indented_node_ref_slice("Attributes", &attrs);
                logger.log_opt_node_ref(vis);
                logger.prefixed_log_fmt(format_args!("Is Mut: {is_mut}\n"));
                
                if bits.is_none() {
                    logger.set_last_at_indent();
                }
                logger.set_last_at_indent_if(bits.is_none());
                logger.log_indented_node_ref("Path", path);

                logger.set_last_at_indent();
                logger.log_indented_opt_node("Bits", bits);
            }),
        }
    }
}


pub struct Const {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub attrs:   Vec<AstNodeRef<Attribute>>,
    pub vis:     Option<AstNodeRef<Visibility>>,
    pub name:    NameId,
    pub ty:      Option<Type>,
    pub val:     Expr,
}

impl AstNode for Const {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Const Item", |logger| {
            logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(self.name)));
            logger.log_indented_opt_node("Type", &self.ty);
            logger.set_last_at_indent();
            logger.log_indented_node("Value", &self.val);
        });
    }
}

impl AstNodeParseHelper for Const {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub enum Static {
    Static {
        span:    SpanId,
        node_id: NodeId,
        attrs:   Vec<AstNodeRef<Attribute>>,
        vis:     Option<AstNodeRef<Visibility>>,
        name:    NameId,
        ty:      Option<Type>,
        val:     Expr,
    },
    Tls {
        span:    SpanId,
        node_id: NodeId,
        attrs:   Vec<AstNodeRef<Attribute>>,
        vis:     Option<AstNodeRef<Visibility>>,
        is_mut:  bool,
        name:    NameId,
        ty:      Option<Type>,
        val:     Expr,
    },
    Extern {
        span:    SpanId,
        node_id: NodeId,
        attrs:   Vec<AstNodeRef<Attribute>>,
        vis:     Option<AstNodeRef<Visibility>>,
        abi:     LiteralId,
        is_mut:  bool,
        name:    NameId,
        ty:      Type,
    }
}

impl AstNode for Static {
    fn span(&self) -> SpanId {
        match self {
            Static::Static { span, .. } => *span,
            Static::Tls { span, .. }    => *span,
            Static::Extern { span, .. } => *span,
        }    
    }

    fn node_id(&self) -> NodeId {
        match self {
            Static::Static { node_id, .. } => *node_id,
            Static::Tls { node_id, .. }    => *node_id,
            Static::Extern { node_id, .. } => *node_id,
        }    
    }

    fn log(&self, logger: &mut AstLogger) {
        match self {
            Static::Static { span, node_id, attrs, vis, name, ty, val } => logger.log_ast_node("Static", |logger| {
                logger.log_indented_node_ref_slice("Attributes", &attrs);
                logger.log_opt_node_ref(vis);
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                logger.log_indented_opt_node("Type", ty);
                logger.set_last_at_indent();
                logger.log_indented_node("Val", val);
            }),
            Static::Tls { span, node_id, attrs, vis, is_mut, name, ty, val } => logger.log_ast_node("Tls Static", |logger| {
                logger.log_indented_node_ref_slice("Attributes", &attrs);
                logger.log_opt_node_ref(vis);
                logger.prefixed_log_fmt(format_args!("Is Mut: {is_mut}\n"));
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                logger.log_indented_opt_node("Type", ty);
                logger.set_last_at_indent();
                logger.log_indented_node("Val", val);
            }),
            Static::Extern { span, node_id, attrs, vis, abi, is_mut, name, ty } => logger.log_ast_node("Extern Static", |logger| {
                logger.log_indented_node_ref_slice("Attributes", &attrs);
                logger.log_opt_node_ref(vis);
                logger.prefixed_log_fmt(format_args!("ABI: {}\n", logger.resolve_literal(*abi)));
                logger.prefixed_log_fmt(format_args!("Is Mut: {is_mut}\n"));
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                logger.set_last_at_indent();
                logger.log_indented_node("Type", ty);
            }),
        }
    }
}

impl AstNodeParseHelper for Static {
    fn set_node_id(&mut self, ast_node_id: NodeId) {
        match self {
            Static::Static { node_id, .. } => *node_id = ast_node_id,
            Static::Tls { node_id, .. }    => *node_id = ast_node_id,
            Static::Extern { node_id, .. } => *node_id = ast_node_id,
        }
    }
}

pub struct Property {
    pub span:      SpanId,
    pub node_id:   NodeId,
    pub attrs:     Vec<AstNodeRef<Attribute>>,
    pub vis:       Option<AstNodeRef<Visibility>>,
    pub is_unsafe: bool,
    pub name:      NameId,
    pub body:      PropertyBody,
}


pub enum PropertyBody {
    Assoc {
        get:       Option<(SpanId, Expr)>,
        ref_get:   Option<(SpanId, Expr)>,
        mut_get:   Option<(SpanId, Expr)>,
        set:       Option<(SpanId, Expr)>,
    },
    Trait {
        has_get:     Option<SpanId>,
        has_ref_get: Option<SpanId>,
        has_mut_get: Option<SpanId>,
        has_set:     Option<SpanId>,
    }
}

impl AstNode for Property {
    fn span(&self) -> SpanId {
        self.span
    }
    
    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        let header = if self.is_trait_property() {
            "Trait Property"
        } else {
            "Assoc Property"
        };
        logger.log_ast_node(&header, |logger| {
            logger.log_indented_node_ref_slice("Attributes", &self.attrs);
            logger.log_opt_node_ref(&self.vis);
            logger.prefixed_log_fmt(format_args!("Is Unsafe: {}\n", self.is_unsafe));
            logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(self.name)));
            self.body.log(logger);
        });
    }
}

impl AstNodeParseHelper for Property {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

impl Property {
    pub fn is_trait_property(&self) -> bool {
        matches!(self.body, PropertyBody::Trait{ .. })
    }

    pub fn has_get(&self) -> bool {
        match &self.body {
            PropertyBody::Assoc { get, .. } => get.is_some(),
            PropertyBody::Trait { has_get, .. } => has_get.is_some(),
        }
    }
    pub fn has_ref_get(&self) -> bool {
        match &self.body {
            PropertyBody::Assoc { ref_get, .. } => ref_get.is_some(),
            PropertyBody::Trait { has_ref_get, .. } => has_ref_get.is_some(),
        }
    }
    pub fn has_mut_get(&self) -> bool {
        match &self.body {
            PropertyBody::Assoc { mut_get, .. } => mut_get.is_some(),
            PropertyBody::Trait { has_mut_get, .. } => has_mut_get.is_some(),
        }
    }
    pub fn has_set(&self) -> bool {
        match &self.body {
            PropertyBody::Assoc { set, .. } => set.is_some(),
            PropertyBody::Trait { has_set, .. } => has_set.is_some(),
        }
    }
}

impl PropertyBody {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            PropertyBody::Assoc { get, ref_get, mut_get, set } => {
                logger.set_last_at_indent_if(ref_get.is_none() && mut_get.is_none() && set.is_none());
                logger.log_indented_opt("Get", get, |logger, (_, get)| get.log(logger));
                logger.set_last_at_indent_if(mut_get.is_none() && set.is_none());
                logger.log_indented_opt("Ref Get", ref_get, |logger, (_, ref_get)| ref_get.log(logger));
                logger.set_last_at_indent_if(set.is_none());
                logger.log_indented_opt("Mut Get", mut_get, |logger, (_, mut_get)| mut_get.log(logger));
                logger.set_last_at_indent();
                logger.log_indented_opt("Set", set, |logger, (_, set)| set.log(logger));
            },
            PropertyBody::Trait { has_get, has_ref_get, has_mut_get, has_set } => {
                logger.prefixed_log_fmt(format_args!("Has Get: {}\n", has_get.is_some()));
                logger.prefixed_log_fmt(format_args!("Has Ref Get: {}\n", has_ref_get.is_some()));
                logger.prefixed_log_fmt(format_args!("Has Mut Get: {}\n", has_mut_get.is_some()));
                logger.prefixed_log_fmt(format_args!("Has Set: {}\n", has_set.is_some()));
            },
        }
    }
}


pub struct Trait {
    pub span:        SpanId,
    pub node_id:     NodeId,
    pub attrs:       Vec<AstNodeRef<Attribute>>,
    pub vis:         Option<AstNodeRef<Visibility>>,
    pub is_unsafe:   bool,
    pub is_sealed:   bool,
    pub name:        NameId,
    pub bounds:      Option<AstNodeRef<TraitBounds>>,
    pub assoc_items: Vec<TraitItem>,
}

impl AstNode for Trait {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Trait", |logger| {
            logger.log_indented_node_ref_slice("Attributes", &self.attrs);
            logger.log_opt_node_ref(&self.vis);
            logger.prefixed_log_fmt(format_args!("Is Unsafe: {}\n", self.is_unsafe));
            logger.prefixed_log_fmt(format_args!("Is Sealed: {}\n", self.is_sealed));
            logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(self.name)));            

            logger.set_last_at_indent_if(self.assoc_items.is_empty());
            logger.log_opt_node_ref(&self.bounds);
            logger.set_last_at_indent();
            logger.log_indented_node_slice("Associated Items", &self.assoc_items);
        })
    }
}

impl AstNodeParseHelper for Trait {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct Impl {
    pub span:         SpanId,
    pub node_id:      NodeId,
    pub attrs:        Vec<AstNodeRef<Attribute>>,
    pub vis:          Option<AstNodeRef<Visibility>>,
    pub is_unsafe:    bool,
    pub generics:     Option<AstNodeRef<GenericParams>>,
    pub ty:           Type,
    pub impl_trait:   Option<AstNodeRef<TypePath>>,
    pub where_clause: Option<AstNodeRef<WhereClause>>,
    pub assoc_items:  Vec<AssocItem>,
}

impl AstNode for Impl {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Impl", |logger| {
            logger.log_indented_node_ref_slice("Attributes", &self.attrs);
            logger.log_opt_node_ref(&self.vis);
            logger.prefixed_log_fmt(format_args!("Is Unsafe: {}\n", self.is_unsafe));

            logger.set_last_at_indent_if(self.impl_trait.is_none() && self.where_clause.is_none() && self.assoc_items.is_empty());
            logger.log_indented_node("Type", &self.ty);
            
            logger.set_last_at_indent_if(self.where_clause.is_none() && self.assoc_items.is_empty());
            logger.log_indented_opt_node_ref("Impl Trait", &self.impl_trait);

            logger.set_last_at_indent_if(self.assoc_items.is_empty());
            logger.log_opt_node_ref(&self.where_clause);

            logger.set_last_at_indent();
            logger.log_indented_node_slice("Associated Items", &self.assoc_items);
        });
    }
}

impl AstNodeParseHelper for Impl {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct ExternBlock {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub attrs:   Vec<AstNodeRef<Attribute>>,
    pub vis:     Option<AstNodeRef<Visibility>>,
    pub abi:     LiteralId,
    pub items:   Vec<ExternItem>,
}

impl AstNode for ExternBlock {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Extern Block", |logger| {
            logger.log_indented_node_ref_slice("Attributes", &self.attrs);
            logger.log_opt_node_ref(&self.vis);
            logger.prefixed_log_fmt(format_args!("ABI: {}\n", logger.resolve_literal(self.abi)));

            logger.set_last_at_indent();  
            logger.log_indented_node_slice("Extern Items", &self.items);
        })
    }
}

impl AstNodeParseHelper for ExternBlock {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub enum OpTrait {
    Base {
        span:       SpanId,
        node_id:    NodeId,
        attrs:      Vec<AstNodeRef<Attribute>>,
        vis:        Option<AstNodeRef<Visibility>>,
        name:       NameId,
        precedence: Option<NameId>,
        elems:      Vec<OpElem>,
    },
    Extended {
        span:       SpanId,
        node_id:    NodeId,
        attrs:      Vec<AstNodeRef<Attribute>>,
        vis:        Option<AstNodeRef<Visibility>>,
        name:       NameId,
        bases:      Vec<AstNodeRef<SimplePath>>,
        elems:      Vec<OpElem>,
    }
}

impl AstNode for OpTrait {
    fn span(&self) -> SpanId {
        match self {
            OpTrait::Base { span, .. }     => *span,
            OpTrait::Extended { span, .. } => *span,
        }
    }

    fn node_id(&self) -> NodeId {
        match self {
            OpTrait::Base { node_id, .. }     => *node_id,
            OpTrait::Extended { node_id, .. } => *node_id,
        }
    }
    
    fn log(&self, logger: &mut AstLogger) {
        match self {
            OpTrait::Base { span, node_id, attrs, vis, name, precedence, elems } => logger.log_ast_node("Operator Trait", |logger| {   
                logger.log_indented_node_ref_slice("Attributes", attrs);
                logger.log_opt_node_ref(vis);

                logger.set_last_at_indent_if(precedence.is_none() && elems.is_empty());
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                logger.set_last_at_indent_if(elems.is_empty());
                logger.log_opt(precedence, |logger, precedence| {
                    logger.log_fmt(format_args!("Precedence: {}", logger.resolve_name(*precedence)))
                });
                logger.set_last_at_indent();
                logger.log_indented_slice("Elements", elems, |logger, elem| elem.log(logger));
            }),
            OpTrait::Extended { span, node_id, attrs, vis, name, bases, elems } => logger.log_ast_node("Operator Extension", |logger| {
                logger.log_indented_node_ref_slice("Attributes", attrs);
                logger.log_opt_node_ref(vis);

                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                logger.set_last_at_indent_if(elems.is_empty());
                logger.log_indented_node_ref_slice("Bases", bases);
                logger.set_last_at_indent();
                logger.log_indented_slice("Elements", elems, |logger, elem| elem.log(logger));
            }),
        }
    }
}

impl AstNodeParseHelper for OpTrait {
    fn set_node_id(&mut self, ast_node_id: NodeId) {
        match self {
            OpTrait::Base { node_id, .. }     => *node_id = ast_node_id,
            OpTrait::Extended { node_id, .. } => *node_id = ast_node_id,
        }
    }
}

pub enum OpElem {
    Def {
        span:    SpanId,
        op_type: OpType,
        op:      Punctuation,
        name:    NameId,
        ret:     Option<Type>,
        def:     Option<Expr>,
    },
    Extend {
        span:    SpanId,
        op_type: OpType,
        op:      Punctuation,
        def:     Expr,
    },
    Contract {
        span:    SpanId,
        expr:    AstNodeRef<BlockExpr>
    }
}

impl OpElem {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            OpElem::Def { span, op_type, op, name, ret, def } => logger.log_indented("Operator Definition", |logger| {
                logger.prefixed_log_fmt(format_args!("Operator Type: {op_type}\n"));
                logger.prefixed_log_fmt(format_args!("Operator: {}\n", logger.resolve_punctuation(*op)));
                logger.set_last_at_indent_if(def.is_none());
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                logger.set_last_at_indent_if(def.is_none());
                logger.log_indented_opt_node("Return Type", ret);
                logger.set_last_at_indent();
                logger.log_indented_opt_node("Default Implementation", def);
            }),
            OpElem::Extend { span, op_type, op, def } => logger.log_indented("Operator Specialization", |logger| {
                logger.prefixed_log_fmt(format_args!("Operator Type: {op_type}\n"));
                logger.prefixed_log_fmt(format_args!("Operator: {}\n", logger.resolve_punctuation(*op)));
                logger.set_last_at_indent();
                logger.log_indented_node("Default Implementation", def);
            }),
            OpElem::Contract { span, expr } => logger.log_indented_node_ref("Contract", expr),
        }
    }
}

pub struct OpUse {
    pub span:      SpanId,
    pub node_id:   NodeId,
    pub group:     Option<NameId>,
    pub package:   Option<NameId>,
    pub library:   Option<NameId>,
    pub operators: Vec<Punctuation>,
}

impl AstNode for OpUse {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Operator Use", |logger| {
            logger.log_opt(&self.group, |logger, name| {
                logger.prefixed_log_fmt(format_args!("Group: {}", logger.resolve_name(*name)))
            });
            logger.set_last_at_indent_if(self.library.is_none() && self.operators.is_empty());
            logger.log_opt(&self.package, |logger, name| {
                logger.prefixed_log_fmt(format_args!("Package: {}", logger.resolve_name(*name)))
            });
            logger.set_last_at_indent_if(self.operators.is_empty());
            logger.log_opt(&self.library, |logger, name| {
                logger.prefixed_log_fmt(format_args!("Library: {}", logger.resolve_name(*name)))
            });
            logger.set_last_at_indent();
            logger.log_indented_slice("Precedences", &self.operators, |logger, punct| {
                logger.prefixed_log_fmt(format_args!("{}", logger.resolve_punctuation(*punct)))
            })
        })
    }
}

impl AstNodeParseHelper for OpUse {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

// TODO: May be moved into separate fill
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PrecedenceAssociativityKind {
    None,
    Left,
    Right,
}

pub struct PrecedenceAssociativity {
    pub span: SpanId,
    pub kind: PrecedenceAssociativityKind
}

impl fmt::Display for PrecedenceAssociativity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            PrecedenceAssociativityKind::None  => write!(f, "none"),
            PrecedenceAssociativityKind::Left  => write!(f, "left"),
            PrecedenceAssociativityKind::Right => write!(f, "right"),
        }
    }
}

pub struct Precedence {
    pub span:          SpanId,
    pub node_id:       NodeId,
    pub attrs:         Vec<AstNodeRef<Attribute>>,
    pub vis:           Option<AstNodeRef<Visibility>>,
    pub name:          NameId,
    pub higher_than:   Option<NameId>,
    pub lower_than:    Option<NameId>,
    pub associativity: Option<PrecedenceAssociativity>
}

impl AstNode for Precedence {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Precedence", |logger| {
            logger.log_indented_node_ref_slice("Attributes", &self.attrs);
            logger.log_opt_node_ref(&self.vis);

            logger.set_last_at_indent_if(self.higher_than.is_none() && self.lower_than.is_none() && self.associativity.is_none());
            logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(self.name)));
            
            logger.set_last_at_indent_if(self.lower_than.is_none() && self.associativity.is_none());
            logger.log_opt(&self.higher_than, |logger, name| {
                logger.prefixed_log_fmt(format_args!("Higher Than: {}\n", logger.resolve_name(*name)));
            });
            logger.set_last_at_indent_if(self.associativity.is_none());
            logger.log_opt(&self.lower_than, |logger, name| {
                logger.prefixed_log_fmt(format_args!("Lower Than: {}\n", logger.resolve_name(*name)));
            });
            logger.set_last_at_indent();
            logger.log_opt(&self.associativity, |logger, assoc| {
                logger.prefixed_log_fmt(format_args!("Associativity: {assoc}\n"));
            });
        });
    }
}

impl AstNodeParseHelper for Precedence {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct PrecedenceUse {
    pub span:        SpanId,
    pub node_id:     NodeId,
    pub group:       Option<NameId>,
    pub package:     Option<NameId>,
    pub library:     Option<NameId>,
    pub precedences: Vec<NameId>,
}

impl AstNode for PrecedenceUse {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Precedence Use", |logger| {
            logger.log_opt(&self.group, |logger, name| {
                logger.prefixed_log_fmt(format_args!("Group: {}", logger.resolve_name(*name)))
            });
            logger.set_last_at_indent_if(self.library.is_none() && self.precedences.is_empty());
            logger.log_opt(&self.package, |logger, name| {
                logger.prefixed_log_fmt(format_args!("Package: {}", logger.resolve_name(*name)))
            });
            logger.set_last_at_indent_if(self.precedences.is_empty());
            logger.log_opt(&self.library, |logger, name| {
                logger.prefixed_log_fmt(format_args!("Library: {}", logger.resolve_name(*name)))
            });
            logger.set_last_at_indent();
            logger.log_indented_slice("Precedences", &self.precedences, |logger, name| {
                logger.prefixed_log_fmt(format_args!("{}", logger.resolve_name(*name)))
            })
        })
    }
}

impl AstNodeParseHelper for PrecedenceUse {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

// =============================================================================================================================

pub enum Stmt {
    Empty(AstNodeRef<EmptyStmt>),
    Item(Item),
    VarDecl(AstNodeRef<VarDecl>),
    Defer(AstNodeRef<Defer>),
    ErrDefer(AstNodeRef<ErrDefer>),
    Expr(AstNodeRef<ExprStmt>),
}

impl AstNode for Stmt {
    fn span(&self) -> SpanId {
        match self {
            Stmt::Empty(item)    => item.span(),
            Stmt::Item(item)     => item.span(),
            Stmt::VarDecl(item)  => item.span(),
            Stmt::Defer(item)    => item.span(),
            Stmt::ErrDefer(item) => item.span(),
            Stmt::Expr(item)     => item.span(),
        }
    }

    fn node_id(&self) -> NodeId {
        match self {
            Stmt::Empty(item)    => item.node_id(),
            Stmt::Item(item)     => item.node_id(),
            Stmt::VarDecl(item)  => item.node_id(),
            Stmt::Defer(item)    => item.node_id(),
            Stmt::ErrDefer(item) => item.node_id(),
            Stmt::Expr(item)     => item.node_id(),
        }
    }

    fn log(&self, logger: &mut AstLogger) {
        match self {
            Self::Empty(item)        => logger.log_node_ref(item),
            Self::Item(item)         => item.log(logger),
            Self::VarDecl(var_decl)  => logger.log_node_ref(var_decl),
            Self::Defer(defer)       => logger.log_node_ref(defer),
            Self::ErrDefer(errdefer) => logger.log_node_ref(errdefer),
            Self::Expr(expr)         => logger.log_node_ref(expr),
        }
    }
}

pub struct EmptyStmt {
    pub span:    SpanId,
    pub node_id: NodeId,
}

impl AstNode for EmptyStmt {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.prefixed_logln("Empty Statement")
    }
}

impl AstNodeParseHelper for EmptyStmt {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub enum VarDecl {
    Named {
        span:    SpanId,
        node_id: NodeId,
        attrs:   Vec<AstNodeRef<Attribute>>,
        names:   Vec<(bool, NameId, SpanId)>,
        expr:    Expr,
    },
    Let {
        span:       SpanId,
        node_id:    NodeId,
        attrs:      Vec<AstNodeRef<Attribute>>,
        pattern:    Pattern,
        ty:         Option<Type>,
        expr:       Option<Expr>,
        else_block: Option<AstNodeRef<BlockExpr>>,
    }
}

impl AstNode for VarDecl {
    fn span(&self) -> SpanId {
        match self {
            VarDecl::Named { span, .. } => *span,
            VarDecl::Let { span, .. }   => *span,
        }
    }

    fn node_id(&self) -> NodeId {
        match self {
            VarDecl::Named { node_id, .. } => *node_id,
            VarDecl::Let { node_id, .. }   => *node_id,
        }
    }

    fn log(&self, logger: &mut AstLogger) {
        match self {
            VarDecl::Named { span, node_id, attrs, names, expr } => logger.log_ast_node("Named Variable Declaration", |logger| {
                logger.log_indented_node_ref_slice("Attributes", attrs);
                logger.log_indented_slice("Names", names, |logger, (is_mut, name, _)| {
                    logger.log_indented("Name", |logger| {
                        logger.prefixed_log_fmt(format_args!("Is Mut: {is_mut}\n"));
                        logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                    });
                });

                logger.set_last_at_indent();
                logger.log_indented_node("Value", expr);
            }),
            VarDecl::Let { span, node_id, attrs, pattern, ty, expr, else_block } => logger.log_ast_node("Let Variable Declaration", |logger| {
                logger.log_indented_node_ref_slice("Attributes", attrs);
                logger.log_indented_node("Pattern", pattern);
                logger.set_last_at_indent_if(expr.is_none() && else_block.is_none());
                logger.log_indented_opt_node("Type", ty);
                logger.set_last_at_indent_if(else_block.is_none());
                logger.log_indented_opt_node("Value", expr);
                logger.set_last_at_indent();
                logger.log_indented_opt_node_ref("Else Block", else_block);
            }),
        }
    }
}

impl AstNodeParseHelper for VarDecl {
    fn set_node_id(&mut self, ast_node_id: NodeId) {
        match self {
            VarDecl::Named { node_id, .. } => *node_id = ast_node_id,
            VarDecl::Let { node_id, .. }   => *node_id = ast_node_id,
        }
    }
}

pub struct Defer {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub attrs:   Vec<AstNodeRef<Attribute>>,
    pub expr:    Expr,
}

impl AstNode for Defer {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Defer Statement", |logger| {
            logger.log_indented_node_ref_slice("Attributes", &self.attrs);
            logger.set_last_at_indent();
            logger.log_indented_node("Expr", &self.expr); 
        });
    }
}

impl AstNodeParseHelper for Defer {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct ErrDefer {
    pub span:     SpanId,
    pub node_id:  NodeId,
    pub attrs:    Vec<AstNodeRef<Attribute>>,
    pub receiver: Option<ErrDeferReceiver>,
    pub expr:     Expr,

}

impl AstNode for ErrDefer {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Error Defer Statement", |logger| {
            logger.log_indented_node_ref_slice("Attributes", &self.attrs);
            logger.log_indented_opt("Receiver", &self.receiver, |logger, rec| {
                logger.prefixed_log_fmt(format_args!("Is Mut: {}\n", rec.is_mut));
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(rec.name)));
            });
            logger.set_last_at_indent();
            logger.log_indented_node("Expr", &self.expr);
        })
    }
}

impl AstNodeParseHelper for ErrDefer {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct ErrDeferReceiver {
    pub span:   SpanId,
    pub is_mut: bool,
    pub name:   NameId,
}

pub struct ExprStmt {
    pub span:     SpanId,
    pub node_id:  NodeId,
    pub attrs:    Vec<AstNodeRef<Attribute>>,
    pub expr:     Expr,
    pub has_semi: bool,
}

impl AstNode for ExprStmt {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Expression Statement", |logger| {
            logger.log_indented_node_ref_slice("Attributes", &self.attrs);
            logger.set_last_at_indent();
            logger.log_node(&self.expr);
        })
    }
}

impl AstNodeParseHelper for ExprStmt {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

// =============================================================================================================================

#[derive(Clone)]
pub enum Expr {
    Literal(AstNodeRef<LiteralExpr>),
    // Can include a sequence of field accesses
    Path(AstNodeRef<PathExpr>),
    Unit(AstNodeRef<UnitExpr>),
    Block(AstNodeRef<BlockExpr>),
    Prefix(AstNodeRef<PrefixExpr>),
    Postfix(AstNodeRef<PostfixExpr>),
    Infix(AstNodeRef<InfixExpr>),
    Paren(AstNodeRef<ParenExpr>),
    Inplace(AstNodeRef<InplaceExpr>),
    TypeCast(AstNodeRef<TypeCastExpr>),
    TypeCheck(AstNodeRef<TypeCheckExpr>),
    Tuple(AstNodeRef<TupleExpr>),
    Array(AstNodeRef<ArrayExpr>),
    Struct(AstNodeRef<StructExpr>),
    Index(AstNodeRef<IndexExpr>),
    TupleIndex(AstNodeRef<TupleIndexExpr>),
    FnCall(AstNodeRef<FnCallExpr>),
    Method(AstNodeRef<MethodCallExpr>),
    FieldAccess(AstNodeRef<FieldAccessExpr>),
    Closure(AstNodeRef<ClosureExpr>),
    FullRange(AstNodeRef<FullRangeExpr>),
    If(AstNodeRef<IfExpr>),
    Let(AstNodeRef<LetBindingExpr>),
    Loop(AstNodeRef<LoopExpr>),
    While(AstNodeRef<WhileExpr>),
    DoWhile(AstNodeRef<DoWhileExpr>),
    For(AstNodeRef<ForExpr>),
    Match(AstNodeRef<MatchExpr>),
    Break(AstNodeRef<BreakExpr>),
    Continue(AstNodeRef<ContinueExpr>),
    Fallthrough(AstNodeRef<FallthroughExpr>),
    Return(AstNodeRef<ReturnExpr>),
    Underscore(AstNodeRef<UnderscoreExpr>),
    Throw(AstNodeRef<ThrowExpr>),
    Comma(AstNodeRef<CommaExpr>),
    When(AstNodeRef<WhenExpr>),
}

impl Expr {
    pub fn has_block(&self) -> bool {
        match self {
            Self::Block(_) |
            Self::If(_)    |
            Self::Loop(_)  |
            Self::While(_) |
            Self::For(_)
            => true,
            _ => false,
        }
    }
}

impl AstNode for Expr {
    fn span(&self) -> SpanId {
        match self {
            Expr::Literal(expr)     => expr.span(),
            Expr::Path(expr)        => expr.span(),
            Expr::Unit(expr)        => expr.span(),
            Expr::Block(expr)       => expr.span(),
            Expr::Prefix(expr)      => expr.span(),
            Expr::Postfix(expr)     => expr.span(),
            Expr::Infix(expr)       => expr.span(),
            Expr::Paren(expr)       => expr.span(),
            Expr::Inplace(expr)     => expr.span(),
            Expr::TypeCast(expr)    => expr.span(),
            Expr::TypeCheck(expr)   => expr.span(),
            Expr::Tuple(expr)       => expr.span(),
            Expr::Array(expr)       => expr.span(),
            Expr::Struct(expr)      => expr.span(),
            Expr::Index(expr)       => expr.span(),
            Expr::TupleIndex(expr)  => expr.span(),
            Expr::FnCall(expr)      => expr.span(),
            Expr::Method(expr)      => expr.span(),
            Expr::FieldAccess(expr) => expr.span(),
            Expr::Closure(expr)     => expr.span(),
            Expr::FullRange(expr)   => expr.span(),
            Expr::If(expr)          => expr.span(),
            Expr::Let(expr)         => expr.span(),
            Expr::Loop(expr)        => expr.span(),
            Expr::While(expr)       => expr.span(),
            Expr::DoWhile(expr)     => expr.span(),
            Expr::For(expr)         => expr.span(),
            Expr::Match(expr)       => expr.span(),
            Expr::Break(expr)       => expr.span(),
            Expr::Continue(expr)    => expr.span(),
            Expr::Fallthrough(expr) => expr.span(),
            Expr::Return(expr)      => expr.span(),
            Expr::Underscore(expr)  => expr.span(),
            Expr::Throw(expr)       => expr.span(),
            Expr::Comma(expr)       => expr.span(),
            Expr::When(expr)        => expr.span(),
        }
    }

    fn node_id(&self) -> NodeId {
        match self {
            Expr::Literal(expr)     => expr.node_id(),
            Expr::Path(expr)        => expr.node_id(),
            Expr::Unit(expr)        => expr.node_id(),
            Expr::Block(expr)       => expr.node_id(),
            Expr::Prefix(expr)      => expr.node_id(),
            Expr::Postfix(expr)     => expr.node_id(),
            Expr::Infix(expr)       => expr.node_id(),
            Expr::Paren(expr)       => expr.node_id(),
            Expr::Inplace(expr)     => expr.node_id(),
            Expr::TypeCast(expr)    => expr.node_id(),
            Expr::TypeCheck(expr)   => expr.node_id(),
            Expr::Tuple(expr)       => expr.node_id(),
            Expr::Array(expr)       => expr.node_id(),
            Expr::Struct(expr)      => expr.node_id(),
            Expr::Index(expr)       => expr.node_id(),
            Expr::TupleIndex(expr)  => expr.node_id(),
            Expr::FnCall(expr)      => expr.node_id(),
            Expr::Method(expr)      => expr.node_id(),
            Expr::FieldAccess(expr) => expr.node_id(),
            Expr::Closure(expr)     => expr.node_id(),
            Expr::FullRange(expr)   => expr.node_id(),
            Expr::If(expr)          => expr.node_id(),
            Expr::Let(expr)         => expr.node_id(),
            Expr::Loop(expr)        => expr.node_id(),
            Expr::While(expr)       => expr.node_id(),
            Expr::DoWhile(expr)     => expr.node_id(),
            Expr::For(expr)         => expr.node_id(),
            Expr::Match(expr)       => expr.node_id(),
            Expr::Break(expr)       => expr.node_id(),
            Expr::Continue(expr)    => expr.node_id(),
            Expr::Fallthrough(expr) => expr.node_id(),
            Expr::Return(expr)      => expr.node_id(),
            Expr::Underscore(expr)  => expr.node_id(),
            Expr::Throw(expr)       => expr.node_id(),
            Expr::Comma(expr)       => expr.node_id(),
            Expr::When(expr)        => expr.node_id(),
        }
    }

    fn log(&self, logger: &mut AstLogger) {
        match self {
            Self::Literal(expr)     => logger.log_node_ref(expr),
            Self::Path(expr)        => logger.log_node_ref(expr),
            Self::Unit(expr)        => logger.log_node_ref(expr),
            Self::Block(expr)       => logger.log_node_ref(expr),
            Self::Prefix(expr)      => logger.log_node_ref(expr),
            Self::Postfix(expr)     => logger.log_node_ref(expr),
            Self::Infix(expr)       => logger.log_node_ref(expr),
            Self::Paren(expr)       => logger.log_node_ref(expr),
            Self::Inplace(expr)     => logger.log_node_ref(expr),
            Self::TypeCast(expr)    => logger.log_node_ref(expr),
            Self::TypeCheck(expr)   => logger.log_node_ref(expr),
            Self::Tuple(expr)       => logger.log_node_ref(expr),
            Self::Array(expr)       => logger.log_node_ref(expr),
            Self::Struct(expr)      => logger.log_node_ref(expr),
            Self::Index(expr)       => logger.log_node_ref(expr),
            Self::TupleIndex(expr)  => logger.log_node_ref(expr),
            Self::FnCall(expr)      => logger.log_node_ref(expr),
            Self::Method(expr)      => logger.log_node_ref(expr),
            Self::FieldAccess(expr) => logger.log_node_ref(expr),
            Self::Closure(expr)     => logger.log_node_ref(expr),
            Self::FullRange(expr)   => logger.log_node_ref(expr),
            Self::If(expr)          => logger.log_node_ref(expr),
            Self::Let(expr)         => logger.log_node_ref(expr),
            Self::Loop(expr)        => logger.log_node_ref(expr),
            Self::While(expr)       => logger.log_node_ref(expr),
            Self::DoWhile(expr)     => logger.log_node_ref(expr),
            Self::For(expr)         => logger.log_node_ref(expr),
            Self::Match(expr)       => logger.log_node_ref(expr),
            Self::Break(expr)       => logger.log_node_ref(expr),
            Self::Continue(expr)    => logger.log_node_ref(expr),
            Self::Fallthrough(expr) => logger.log_node_ref(expr),
            Self::Return(expr)      => logger.log_node_ref(expr),
            Self::Underscore(expr)  => logger.log_node_ref(expr),
            Self::Throw(expr)       => logger.log_node_ref(expr),
            Self::Comma(expr)       => logger.log_node_ref(expr),
            Self::When(expr)        => logger.log_node_ref(expr),
        }
    }
}


pub enum LiteralValue {
    Lit(LiteralId),
    Bool(bool),
}

pub enum LiteralOp {
    Name(NameId),
    Primitive(PrimitiveType),
    StringSlice(StringSliceType),
}

impl LiteralOp {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_indented("Literal Op", |logger| {
            logger.set_last_at_indent();
            match self {
                LiteralOp::Name(name) => {
                    logger.prefixed_log_fmt(format_args!("Named: {}\n", logger.resolve_name(*name)));
                },
                LiteralOp::Primitive(ty) => ty.log(logger),
                LiteralOp::StringSlice(ty) => ty.log(logger),
            }
        });
    }
}

pub struct LiteralExpr {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub literal: LiteralValue,
    pub lit_op:  Option<LiteralOp>
}

impl AstNode for LiteralExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Literal Expr", |logger| {
            match self.literal {
                LiteralValue::Lit(lit) => logger.prefixed_log_fmt(format_args!("Literal: {}\n", logger.resolve_literal(lit))),
                LiteralValue::Bool(val) => logger.prefixed_log_fmt(format_args!("Literal: {val}\n")),
            }

            logger.set_last_at_indent();
            if let Some(lit_op) = &self.lit_op {
                lit_op.log(logger);
            }
        });
    }
}

impl AstNodeParseHelper for LiteralExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub enum PathExpr {
    Named {
        span:    SpanId,
        node_id: NodeId,
        iden:    Identifier,
    },
    Inferred {
        span:    SpanId,
        node_id: NodeId,
        iden:    Identifier,
    },
    SelfPath {
        span:    SpanId,
        node_id: NodeId,
    },
    Qualified {
        span:    SpanId,
        node_id: NodeId,
        path:    AstNodeRef<QualifiedPath>,
    }
}

impl AstNode for PathExpr {
    fn span(&self) -> SpanId {
        match self {
            PathExpr::Named { span, .. }     => *span,
            PathExpr::Inferred { span, .. }  => *span,
            PathExpr::SelfPath { span, ..}   => *span,
            PathExpr::Qualified { span, .. } => *span,
        }
    }

    fn node_id(&self) -> NodeId {
        match self {
            PathExpr::Named { node_id, .. }     => *node_id,
            PathExpr::Inferred { node_id, .. }  => *node_id,
            PathExpr::SelfPath{ node_id, .. }   => *node_id,
            PathExpr::Qualified { node_id, .. } => *node_id,
        }
    }

    fn log(&self, logger: &mut AstLogger) {
        match self {
            PathExpr::Named { span, node_id, iden } => logger.log_ast_node("Path Expr", |logger| {
                logger.set_last_at_indent();
                iden.log(logger);
            }),
            PathExpr::Inferred { span, node_id, iden } => logger.log_ast_node("Inferred Path Expr", |logger| {
                logger.set_last_at_indent();
                iden.log(logger);
            }),
            PathExpr::SelfPath{ span, node_id } => {
                logger.set_last_at_indent();
                logger.prefixed_logln("Self Path Expr");
            },
            PathExpr::Qualified { span, node_id, path } => logger.log_ast_node("Qualified Path Expr", |logger| {
                logger.set_last_at_indent();
                logger.log_node_ref(path);
            })
        }
    }
}

impl AstNodeParseHelper for PathExpr {
    fn set_node_id(&mut self, ast_node_id: NodeId) {
        match self {
            PathExpr::Named { node_id, .. }     => *node_id = ast_node_id,
            PathExpr::Inferred { node_id, .. }  => *node_id = ast_node_id,
            PathExpr::SelfPath{ node_id, ..}    => *node_id = ast_node_id,
            PathExpr::Qualified { node_id, .. } => *node_id = ast_node_id,
        }
    }
}

pub struct UnitExpr {
    pub span:    SpanId,
    pub node_id: NodeId,
}

impl AstNode for UnitExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.prefixed_logln("Unit Expression")
    }
}

impl AstNodeParseHelper for UnitExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlockExprKind {
    Normal,
    Unsafe,
    Const,
    Try,
    TryUnwrap,
    Labeled{ label: NameId }
}

pub struct BlockExpr {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub kind:    BlockExprKind,
    pub block:   AstNodeRef<Block>
}

impl AstNode for BlockExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        let name = match self.kind {
            BlockExprKind::Normal         => "Block Expression",
            BlockExprKind::Unsafe         => "Unsafe Block Expression",
            BlockExprKind::Const          => "Const Block Expression",
            BlockExprKind::Try            => "Try Block Expression",
            BlockExprKind::TryUnwrap      => "Try Unwrap Block Expression",
            BlockExprKind::Labeled { .. } => "Labeled Block Expression",
        };


        logger.log_ast_node(name, |logger| {
            if let BlockExprKind::Labeled{ label } = self.kind {
                logger.prefixed_log_fmt(format_args!("Label: {}\n", logger.resolve_name(label)));
            }
            logger.set_last_at_indent();
            logger.log_node_ref(&self.block);
        });
    }
}

impl AstNodeParseHelper for BlockExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct PrefixExpr {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub op:      Punctuation,
    pub expr:    Expr,
}

impl AstNode for PrefixExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Prefix expression", |logger| {
            logger.prefixed_log_fmt(format_args!("Op: {}\n", logger.resolve_punctuation(self.op)));
            logger.set_last_at_indent();
            logger.log_node(&self.expr);
        });
    }
}

impl AstNodeParseHelper for PrefixExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct PostfixExpr {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub op:      Punctuation,
    pub expr:    Expr,
}

impl AstNode for PostfixExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Postfix expression", |logger| {
            logger.prefixed_log_fmt(format_args!("Op: {}\n", logger.resolve_punctuation(self.op)));
            logger.set_last_at_indent();
            logger.log_node(&self.expr);
        });
    }
}

impl AstNodeParseHelper for PostfixExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct InfixExpr {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub op:      Punctuation,
    pub left:    Expr,
    pub right:   Expr,
}

impl AstNode for InfixExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Infix expression", |logger| {
            logger.prefixed_log_fmt(format_args!("Op: {}\n", logger.resolve_punctuation(self.op)));
            logger.log_node(&self.left);
            logger.set_last_at_indent();
            logger.log_node(&self.right);
        });
    }
}

impl AstNodeParseHelper for InfixExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct ParenExpr {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub expr:    Expr,
}

impl AstNode for ParenExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Parenthesized Expression", |logger| {
            logger.set_last_at_indent();
            logger.log_node(&self.expr);
        });
    }
}

impl AstNodeParseHelper for ParenExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct InplaceExpr {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub left:    Expr,
    pub right:   Expr,
}

impl AstNode for InplaceExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Inplace Expession", |logger| {
            logger.log_node(&self.left);
            logger.set_last_at_indent();
            logger.log_node(&self.right);
        });
    }
}

impl AstNodeParseHelper for InplaceExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TypeCastKind {
    Normal,
    Try,
    Unwrap,
}

pub struct TypeCastExpr {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub kind:    TypeCastKind,
    pub expr:    Expr,
    pub ty:      Type,
}

impl AstNode for TypeCastExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        let name = match self.kind {
            TypeCastKind::Normal => "Type Cast Expression",
            TypeCastKind::Try    => "Try Type Cast Expression",
            TypeCastKind::Unwrap => "Unwrap Type Cast Expression",
        };

        logger.log_ast_node(name, |logger| {
            logger.log_node(&self.expr);
            logger.set_last_at_indent();
            logger.log_node(&self.ty);
        });
    }
}

impl AstNodeParseHelper for TypeCastExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct TypeCheckExpr {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub negate:  bool,
    pub expr:    Expr,
    pub ty:      Type,
}

impl AstNode for TypeCheckExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Type Check Expression", |logger| {
            logger.prefixed_log_fmt(format_args!("Negate: {}\n", self.negate));
            logger.log_node(&self.expr);
            logger.set_last_at_indent();
            logger.log_node(&self.ty);
        })
    }
}

impl AstNodeParseHelper for TypeCheckExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct TupleExpr {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub exprs:   Vec<Expr>
}

impl AstNode for TupleExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Tuple Expression", |logger| {
            for (idx, expr) in self.exprs.iter().enumerate() {
                if idx == self.exprs.len() - 1 {
                    logger.set_last_at_indent();
                }
                logger.log_node(expr);
            }
        });
    }
}

impl AstNodeParseHelper for TupleExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

// Add support for [expr; size]
pub struct ArrayExpr {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub exprs:   Vec<Expr>
}

impl AstNode for ArrayExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Array Expression", |logger| {
            for (idx, expr) in self.exprs.iter().enumerate() {
                if idx == self.exprs.len() - 1 {
                    logger.set_last_at_indent();
                }
                logger.log_node(expr);
            }
        });
    }
}

impl AstNodeParseHelper for ArrayExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub enum StructArg {
    Expr{
        span: SpanId,
        name: NameId,
        expr: Expr
    },
    Name{
        span: SpanId,
        name: NameId
    },
    Complete{
        span: SpanId,
        expr: Expr
    },
}

impl StructArg {
    pub fn log(&self, logger: &mut AstLogger) {
        match self {
            StructArg::Expr{ span, name, expr } => logger.log_indented("Named Argument", |logger| {
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                logger.set_last_at_indent();
                logger.log_node(expr);
            }),
            StructArg::Name{ span ,name }     => logger.prefixed_log_fmt(format_args!("Name-only: {}\n", logger.resolve_name(*name))),
            StructArg::Complete{ span, expr } => logger.log_indented_node("Struct Completion", expr),
        }
    }
}

pub struct StructExpr {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub path:    Expr,
    pub args:    Vec<StructArg>,
}

impl AstNode for StructExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Struct Expression", |logger| {
            logger.set_last_at_indent_if(self.args.is_empty());
            logger.log_indented_node("Path", &self.path);
            logger.set_last_at_indent();
            logger.log_indented_slice("Arguments", &self.args, |logger, arg| arg.log(logger));
        });
    }
}

impl AstNodeParseHelper for StructExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct IndexExpr {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub is_opt:  bool,
    pub expr:    Expr,
    pub index:   Expr,
}

impl AstNode for IndexExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Index Expression", |logger| {
            logger.prefixed_log_fmt(format_args!("Is Optional: {}\n", self.is_opt));
            logger.log_indented_node("Expression", &self.expr);
            logger.set_last_at_indent();
            let index_name = if let Expr::Comma(_) = &self.index { "Indices" } else { "Index" };
            logger.log_indented_node(index_name, &self.index);  
        });
    }
}

impl AstNodeParseHelper for IndexExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct TupleIndexExpr {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub expr:    Expr,
    pub index:   LiteralId
}

impl AstNode for TupleIndexExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Tuple Index Expression", |logger| {
            logger.prefixed_log_fmt(format_args!("Index: {}\n", logger.resolve_literal(self.index)));
            logger.set_last_at_indent();
            logger.log_node(&self.expr);
        });
    }
}

impl AstNodeParseHelper for TupleIndexExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub enum FnArg {
    Expr{
        span: SpanId,
        expr: Expr
    },
    Labeled{
        span:  SpanId,
        label: NameId,
        expr:  Expr,
    }
}

impl FnArg {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            FnArg::Expr{ span, expr } => logger.log_indented_node("Argument", expr),
            FnArg::Labeled { span, label, expr } => logger.log_indented("Labeled Argument", |logger| {
                logger.prefixed_log_fmt(format_args!("Label: {}\n", logger.resolve_name(*label)));
                logger.set_last_at_indent();
                logger.log_indented_node("Expression", expr)
            }),
        }
    }
}

pub struct FnCallExpr {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub expr:    Expr,
    pub args:    Vec<FnArg>,
}

impl AstNode for FnCallExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Expression Function Call", |logger| {
            logger.set_last_at_indent_if(self.args.is_empty());
            logger.log_indented_node("Function", &self.expr);
            logger.set_last_at_indent();
            logger.log_indented_slice("Arguments", &self.args, |logger, arg| arg.log(logger));
        });
    }
}

impl AstNodeParseHelper for FnCallExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct MethodCallExpr {
    pub span:           SpanId,
    pub node_id:        NodeId,
    pub receiver:       Expr,
    pub method:         NameId,
    pub gen_args:       Option<AstNodeRef<GenericArgs>>,
    pub args:           Vec<FnArg>,
    pub is_propagating: bool,
}

impl AstNode for MethodCallExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Method Call Expression", |logger| {
            logger.prefixed_log_fmt(format_args!("Is Propagating: {}\n", self.is_propagating));
            logger.prefixed_log_fmt(format_args!("Method Name: {}\n", logger.resolve_name(self.method)));
            logger.set_last_at_indent_if(self.args.is_empty());
            logger.log_indented_node("Receiver", &self.receiver);
            logger.set_last_at_indent();
            logger.log_indented_slice("Arguments", &self.args, |logger, arg| arg.log(logger));
        });
    }
}

impl AstNodeParseHelper for MethodCallExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct FieldAccessExpr {
    pub span:           SpanId,
    pub node_id:        NodeId,
    pub expr:           Expr,
    pub field:          NameId,
    pub gen_args:       Option<AstNodeRef<GenericArgs>>,
    pub is_propagating: bool
}

impl AstNode for FieldAccessExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Field Access Expression", |logger| {
            logger.prefixed_log_fmt(format_args!("Is Propagating: {}\n", self.is_propagating));
            logger.prefixed_log_fmt(format_args!("Field Name: {}\n", logger.resolve_name(self.field)));
            logger.log_indented_opt_node_ref("Generics", &self.gen_args);
            logger.set_last_at_indent();
            logger.log_node(&self.expr);
        });
    }
}

impl AstNodeParseHelper for FieldAccessExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct ClosureExpr {
    pub span:     SpanId,
    pub node_id:  NodeId,
    pub is_moved: bool,
    pub params:   Vec<FnParam>,
    pub ret:      Option<FnReturn>,
    pub body:     Expr,
}

impl AstNode for ClosureExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Closure Expression", |logger| {

        });
    }
}

impl AstNodeParseHelper for ClosureExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct FullRangeExpr {
    pub span:    SpanId,
    pub node_id: NodeId,
}

impl AstNode for FullRangeExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.prefixed_logln("Full Range Expression")
    }
}

impl AstNodeParseHelper for FullRangeExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct IfExpr {
    pub span:      SpanId,
    pub node_id:   NodeId,
    pub cond:      Expr,
    pub body:      AstNodeRef<BlockExpr>,
    pub else_body: Option<Expr>,
}

impl AstNode for IfExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("If Expression", |logger| {
            logger.log_indented_node("Condition", &self.cond);
            logger.set_last_at_indent_if(self.else_body.is_none());
            logger.log_indented_node_ref("Body", &self.body);
            logger.set_last_at_indent();
            logger.log_indented_opt_node("Else Body", &self.else_body);
        });
    }
}

impl AstNodeParseHelper for IfExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct LetBindingExpr {
    pub span:      SpanId,
    pub node_id:   NodeId,
    pub pattern:   Pattern,
    pub scrutinee: Expr,
}

impl AstNode for LetBindingExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Let Binding Expression", |logger| {
            logger.log_node(&self.pattern);
            logger.set_last_at_indent();
            logger.log_node(&self.scrutinee);
        });
    }
}

impl AstNodeParseHelper for LetBindingExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct LoopExpr {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub label:   Option<NameId>,
    pub body:    AstNodeRef<BlockExpr>,
}

impl AstNode for LoopExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Loop expression", |logger| {
            if let Some(label) = &self.label {
                logger.prefixed_log_fmt(format_args!("Label: {}\n", logger.resolve_name(*label)));
            }
            logger.set_last_at_indent();
            logger.log_node_ref(&self.body);
        });
    }
}

impl AstNodeParseHelper for LoopExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct WhileExpr {
    pub span:      SpanId,
    pub node_id:   NodeId,
    pub label:     Option<NameId>,
    pub cond:      Expr,
    pub inc:       Option<Expr>,
    pub body:      AstNodeRef<BlockExpr>,
    pub else_body: Option<AstNodeRef<BlockExpr>>,
}

impl AstNode for WhileExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("While Expression", |logger| {
            if let Some(label) = &self.label {
                logger.prefixed_log_fmt(format_args!("Label: {}\n", logger.resolve_name(*label)));
            }
            logger.log_indented_node("Condition", &self.cond);
            logger.log_indented_opt_node("Increment", &self.inc);
            logger.set_last_at_indent_if(self.else_body.is_none());
            logger.log_indented_node_ref("Body", &self.body);
            logger.set_last_at_indent();
            logger.log_indented_opt_node_ref("Else Body", &self.else_body);
        });
    }
}

impl AstNodeParseHelper for WhileExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct DoWhileExpr {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub label:   Option<NameId>,
    pub body:    AstNodeRef<BlockExpr>,
    pub cond:    Expr,
}

impl AstNode for DoWhileExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Do While Expression", |logger| {
            if let Some(label) = &self.label {
                logger.prefixed_log_fmt(format_args!("Label: {}\n", logger.resolve_name(*label)));
            }
            logger.log_indented_node_ref("Body", &self.body);
            logger.set_last_at_indent();
            logger.log_indented_node("Condition", &self.cond);
        });
    }
}

impl AstNodeParseHelper for DoWhileExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct ForExpr {
    pub span:      SpanId,
    pub node_id:   NodeId,
    pub label:     Option<NameId>,
    pub pattern:   Pattern,
    pub src:       Expr,
    pub body:      AstNodeRef<BlockExpr>,
    pub else_body: Option<AstNodeRef<BlockExpr>>,
}

impl AstNode for ForExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("For Expression", |logger| {
            if let Some(label) = &self.label {
                logger.prefixed_log_fmt(format_args!("Label: {}\n", logger.resolve_name(*label)));
            }
            logger.log_indented_node("Pattern", &self.pattern);
            logger.log_indented_node("Src", &self.src);
            logger.set_last_at_indent_if(self.else_body.is_some());
            logger.log_indented_node_ref("Body", &self.body);
            logger.set_last_at_indent();
            logger.log_indented_opt_node_ref("Else Body", &self.else_body);
        });
    }
}

impl AstNodeParseHelper for ForExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct MatchExpr {
    pub span:      SpanId,
    pub node_id:   NodeId,
    pub label:     Option<NameId>,
    pub scrutinee: Expr,
    pub branches:  Vec<MatchBranch>,
}

impl AstNode for MatchExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Match Expression", |logger| {
            if let Some(label) = &self.label {
                logger.prefixed_log_fmt(format_args!("Label: {}\n", logger.resolve_name(*label)));
            }
            logger.set_last_at_indent_if(self.branches.is_empty());
            logger.log_indented_node("Scrutinee", &self.scrutinee);
            logger.set_last_at_indent();
            logger.log_indented_slice("Branches", &self.branches, |logger, branch| branch.log(logger));
        });
    }
}

impl AstNodeParseHelper for MatchExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct MatchBranch {
    pub span:     SpanId,
    pub label:    Option<NameId>,
    pub pattern:  Pattern,
    pub guard:    Option<Expr>,
    pub body:     Expr,
}

impl MatchBranch {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_indented("Branch", |logger| {
            if let Some(label) = &self.label {
                logger.prefixed_log_fmt(format_args!("Label: {}\n", logger.resolve_name(*label)));
            }
            logger.log_indented_node("Pattern", &self.pattern);
            logger.log_indented_opt_node("Guard", &self.guard);
            logger.set_last_at_indent();
            logger.log_indented_node("Body", &self.body);
        });
    }
}

pub struct BreakExpr {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub label:   Option<NameId>,
    pub value:   Option<Expr>,
}

impl AstNode for BreakExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Continue Expr", |logger| {
            if let Some(label) = &self.label {
                logger.prefixed_log_fmt(format_args!("Label: {}\n", logger.resolve_name(*label)));
            }
            logger.set_last_at_indent();
            logger.log_opt_node(&self.value);
        });
    }
}

impl AstNodeParseHelper for BreakExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct ContinueExpr {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub label:   Option<NameId>
}

impl AstNode for ContinueExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Continue Expr", |logger| {
            if let Some(label) = &self.label {
                logger.prefixed_log_fmt(format_args!("Label: {}\n", logger.resolve_name(*label)));
            }
        });
    }
}

impl AstNodeParseHelper for ContinueExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct FallthroughExpr {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub label:   Option<NameId>
}

impl AstNode for FallthroughExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Fallthrough Expr", |logger| {
            if let Some(label) = &self.label {
                logger.prefixed_log_fmt(format_args!("Label: {}\n", logger.resolve_name(*label)));
            }
        });
    }
}

impl AstNodeParseHelper for FallthroughExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct ReturnExpr {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub value:   Option<Expr>,
}

impl AstNode for ReturnExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Continue Expr", |logger| {
            logger.set_last_at_indent();
            logger.log_opt_node(&self.value);
        });
    }
}

impl AstNodeParseHelper for ReturnExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct UnderscoreExpr {
    pub span:    SpanId,
    pub node_id: NodeId,
}

impl AstNode for UnderscoreExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.prefixed_logln("Underscore expression")
    }
}

impl AstNodeParseHelper for UnderscoreExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct ThrowExpr {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub expr:    Expr,
}

impl AstNode for ThrowExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Throw Expression", |logger| {
            logger.log_node(&self.expr);
        });
    }
}

impl AstNodeParseHelper for ThrowExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct CommaExpr {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub exprs:   Vec<Expr>,
}

impl AstNode for CommaExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Comma expression", |logger| {
            logger.log_node_slice(&self.exprs);
        })
    }
}

impl AstNodeParseHelper for CommaExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct WhenExpr {
    pub span:      SpanId,
    pub node_id:   NodeId,
    pub cond:      Expr,
    pub body:      AstNodeRef<BlockExpr>,
    pub else_body: Option<Expr>,
}

impl AstNode for WhenExpr {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("When Expression", |logger| {
            logger.log_indented_node("Condition", &self.cond);
            logger.set_last_at_indent_if(self.else_body.is_none());
            logger.log_indented_node_ref("Body", &self.body);
            logger.set_last_at_indent();
            logger.log_indented_opt_node("Else Body", &self.else_body);
        });
    }
}

impl AstNodeParseHelper for WhenExpr {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

// =============================================================================================================================

pub enum Pattern {
    Literal(AstNodeRef<LiteralPattern>),
    Identifier(AstNodeRef<IdentifierPattern>),
    Path(AstNodeRef<PathPattern>),
    Wildcard(AstNodeRef<WildcardPattern>),
    Rest(AstNodeRef<RestPattern>),
    Range(AstNodeRef<RangePattern>),
    Reference(AstNodeRef<ReferencePattern>),
    Struct(AstNodeRef<StructPattern>),
    TupleStruct(AstNodeRef<TupleStructPattern>),
    Tuple(AstNodeRef<TuplePattern>),
    Grouped(AstNodeRef<GroupedPattern>),
    Slice(AstNodeRef<SlicePattern>),
    EnumMember(AstNodeRef<EnumMemberPattern>),
    Alternative(AstNodeRef<AlternativePattern>),
    TypeCheck(AstNodeRef<TypeCheckPattern>),
}

impl AstNode for Pattern {
    fn span(&self) -> SpanId {
        match self {
            Pattern::Literal(pattern)     => pattern.span(),
            Pattern::Identifier(pattern)  => pattern.span(),
            Pattern::Path(pattern)        => pattern.span(),
            Pattern::Wildcard(pattern)    => pattern.span(),
            Pattern::Rest(pattern)        => pattern.span(),
            Pattern::Range(pattern)       => pattern.span(),
            Pattern::Reference(pattern)   => pattern.span(),
            Pattern::Struct(pattern)      => pattern.span(),
            Pattern::TupleStruct(pattern) => pattern.span(),
            Pattern::Tuple(pattern)       => pattern.span(),
            Pattern::Grouped(pattern)     => pattern.span(),
            Pattern::Slice(pattern)       => pattern.span(),
            Pattern::EnumMember(pattern)  => pattern.span(),
            Pattern::Alternative(pattern) => pattern.span(),
            Pattern::TypeCheck(pattern)   => pattern.span(),
        }
    }

    fn node_id(&self) -> NodeId {
        match self {
            Pattern::Literal(pattern)     => pattern.node_id(),
            Pattern::Identifier(pattern)  => pattern.node_id(),
            Pattern::Path(pattern)        => pattern.node_id(),
            Pattern::Wildcard(pattern)    => pattern.node_id(),
            Pattern::Rest(pattern)        => pattern.node_id(),
            Pattern::Range(pattern)       => pattern.node_id(),
            Pattern::Reference(pattern)   => pattern.node_id(),
            Pattern::Struct(pattern)      => pattern.node_id(),
            Pattern::TupleStruct(pattern) => pattern.node_id(),
            Pattern::Tuple(pattern)       => pattern.node_id(),
            Pattern::Grouped(pattern)     => pattern.node_id(),
            Pattern::Slice(pattern)       => pattern.node_id(),
            Pattern::EnumMember(pattern)  => pattern.node_id(),
            Pattern::Alternative(pattern) => pattern.node_id(),
            Pattern::TypeCheck(pattern)   => pattern.node_id(),
        }
    }

    fn log(&self, logger: &mut AstLogger) {
        match self {
            Pattern::Literal(pattern)     => logger.log_node_ref(pattern),
            Pattern::Identifier(pattern)  => logger.log_node_ref(pattern),
            Pattern::Path(pattern)        => logger.log_node_ref(pattern),
            Pattern::Wildcard(pattern)    => logger.log_node_ref(pattern),
            Pattern::Rest(pattern)        => logger.log_node_ref(pattern),
            Pattern::Range(pattern)       => logger.log_node_ref(pattern),
            Pattern::Reference(pattern)   => logger.log_node_ref(pattern),
            Pattern::Tuple(pattern)       => logger.log_node_ref(pattern),
            Pattern::Struct(pattern)      => logger.log_node_ref(pattern),
            Pattern::TupleStruct(pattern) => logger.log_node_ref(pattern),
            Pattern::Grouped(pattern)     => logger.log_node_ref(pattern),
            Pattern::Slice(pattern)       => logger.log_node_ref(pattern),
            Pattern::EnumMember(pattern)  => logger.log_node_ref(pattern),
            Pattern::Alternative(pattern) => logger.log_node_ref(pattern),
            Pattern::TypeCheck(pattern)   => logger.log_node_ref(pattern),
        }
    }
}

pub struct LiteralPattern {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub literal: LiteralValue,
    pub lit_op:  Option<LiteralOp>,
}

impl AstNode for LiteralPattern {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Literal Pattern", |logger| {
            match self.literal {
                LiteralValue::Lit(lit)  => logger.prefixed_log_fmt(format_args!("Literal: {}\n", logger.resolve_literal(lit))),
                LiteralValue::Bool(val) => logger.prefixed_log_fmt(format_args!("Literal: {val}\n")),
            }
            logger.set_last_at_indent();
            logger.log_opt(&self.lit_op, |logger, lit_op| lit_op.log(logger));
        })
    }
}

impl AstNodeParseHelper for LiteralPattern {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct IdentifierPattern {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub is_ref:  bool,
    pub is_mut:  bool,
    pub name:    NameId,
    pub bound:   Option<Pattern>,
}

impl AstNode for IdentifierPattern {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Identifier Pattern", |logger| {
            logger.prefixed_log_fmt(format_args!("Is Ref: {}\n", self.is_ref));
            logger.prefixed_log_fmt(format_args!("Is Mut: {}\n", self.is_mut));
            logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(self.name)));
            logger.set_last_at_indent();
            logger.log_indented_opt_node("Bound", &self.bound);
        })
    }
}

impl AstNodeParseHelper for IdentifierPattern {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct PathPattern {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub path:    AstNodeRef<ExprPath>,
}

impl AstNode for PathPattern {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Path Pattern", |logger| {
            logger.set_last_at_indent();
            logger.log_node_ref(&self.path);
        });
    }
}

impl AstNodeParseHelper for PathPattern {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct WildcardPattern {
    pub span:    SpanId,
    pub node_id: NodeId,
}

impl AstNode for WildcardPattern {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.prefixed_logln("Wildcard Pattern")
    }
}

impl AstNodeParseHelper for WildcardPattern {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct RestPattern {
    pub span:    SpanId,
    pub node_id: NodeId,
}

impl AstNode for RestPattern {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.prefixed_logln("Rest Pattern")
    }
}

impl AstNodeParseHelper for RestPattern {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub enum RangePattern {
    Exclusive{
        span:    SpanId,
        node_id: NodeId,
        begin:   Pattern,
        end:     Pattern
    },
    Inclusive{
        span:    SpanId,
        node_id: NodeId,
        begin:   Pattern,
        end:     Pattern
    },
    From {
        span:    SpanId,
        node_id: NodeId,
        begin:   Pattern
    },
    To {
        span:    SpanId,
        node_id: NodeId,
        end:     Pattern
    },
    InclusiveTo {
        span:    SpanId,
        node_id: NodeId,
        end:     Pattern
    }
}

impl AstNode for RangePattern {
    fn span(&self) -> SpanId {
        match self {
            RangePattern::Exclusive { span, .. }   => *span,
            RangePattern::Inclusive { span, .. }   => *span,
            RangePattern::From { span, .. }        => *span,
            RangePattern::To { span, .. }          => *span,
            RangePattern::InclusiveTo { span, .. } => *span,
        }    
    }

    fn node_id(&self) -> NodeId {
        match self {
            RangePattern::Exclusive { node_id, .. }   => *node_id,
            RangePattern::Inclusive { node_id, .. }   => *node_id,
            RangePattern::From { node_id, .. }        => *node_id,
            RangePattern::To { node_id, .. }          => *node_id,
            RangePattern::InclusiveTo { node_id, .. } => *node_id,
        }    
    }

    fn log(&self, logger: &mut AstLogger) {
        match self {
            RangePattern::Exclusive { span, node_id, begin, end } => logger.log_indented("Exclusive Range Pattern", |logger| {
                logger.log_indented_node("Begin", begin);
                logger.set_last_at_indent();
                logger.log_indented_node("End", end);
            }),
            RangePattern::Inclusive { span, node_id, begin, end } => logger.log_indented("Inclusive Range Pattern", |logger| {
                logger.log_indented_node("Begin", begin);
                logger.set_last_at_indent();
                logger.log_indented_node("End", end);
            }),
            RangePattern::From { span, node_id, begin } => logger.log_indented("From Range Pattern", |logger| {
                logger.set_last_at_indent();
                logger.log_indented_node("Begin", begin);
            }),
            RangePattern::To { span, node_id, end } => logger.log_indented("To Range Pattern", |logger| {
                logger.set_last_at_indent();
                logger.log_indented_node("End", end);
            }),
            RangePattern::InclusiveTo { span, node_id, end } => logger.log_indented("Inclusive To Range Pattern", |logger| {
                logger.set_last_at_indent();
                logger.log_indented_node("End", end);
            }),
        }
    }
}

impl AstNodeParseHelper for RangePattern {
    fn set_node_id(&mut self, ast_node_id: NodeId) {
        match self {
            RangePattern::Exclusive { node_id, .. }   => *node_id = ast_node_id,
            RangePattern::Inclusive { node_id, .. }   => *node_id = ast_node_id,
            RangePattern::From { node_id, .. }        => *node_id = ast_node_id,
            RangePattern::To { node_id, .. }          => *node_id = ast_node_id,
            RangePattern::InclusiveTo { node_id, .. } => *node_id = ast_node_id,
        }
    }
}

pub struct ReferencePattern {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub is_mut:  bool,
    pub pattern: Pattern,
}

impl AstNode for ReferencePattern {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Reference Pattern", |logger| {
            logger.prefixed_log_fmt(format_args!("Is Mut: {}\n", self.is_mut));
            logger.set_last_at_indent();
            logger.log_node(&self.pattern);
        })
    }
}

impl AstNodeParseHelper for ReferencePattern {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub enum StructPattern {
    Inferred {
        span:    SpanId,
        node_id: NodeId,
        fields:  Vec<StructPatternField>,
    },
    Path {
        span:    SpanId,
        node_id: NodeId,
        path:    AstNodeRef<ExprPath>,
        fields:  Vec<StructPatternField>,
    }
}

impl AstNode for StructPattern {
    fn span(&self) -> SpanId {
        match self {
            StructPattern::Inferred { span, .. } => *span,
            StructPattern::Path { span, .. } => *span,
        }
    }

    fn node_id(&self) -> NodeId {
        match self {
            StructPattern::Inferred { node_id, .. } => *node_id,
            StructPattern::Path { node_id, .. } => *node_id,
        }
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Struct Pattern", |logger| match self {
            StructPattern::Inferred { span, node_id, fields } => {
                logger.prefixed_logln("Inferred Path");
                logger.set_last_at_indent();
                logger.log_indented_slice("Fields", fields, |logger, field| field.log(logger));
            },
            StructPattern::Path { span, node_id, path, fields } => {
                logger.log_indented_node_ref("Path", path);
                logger.set_last_at_indent();
                logger.log_indented_slice("Fields", fields, |logger, field| field.log(logger));
            },
        });
    }
}

impl AstNodeParseHelper for StructPattern {
    fn set_node_id(&mut self, ast_node_id: NodeId) {
        match self {
            StructPattern::Inferred { node_id, .. } => *node_id = ast_node_id,
            StructPattern::Path { node_id, .. }     => *node_id = ast_node_id,
        }
    }
}

pub enum StructPatternField {
    Named {
        span:    SpanId,
        name:    NameId,
        pattern: Pattern,
    },
    TupleIndex {
        span:    SpanId,
        idx:     LiteralId,
        pattern: Pattern
    },
    Iden {
        span:   SpanId,
        is_ref: bool,
        is_mut: bool,
        iden:   NameId,
        bound:  Option<Pattern>
    },
    Rest,
}

impl StructPatternField {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            StructPatternField::Named { span, name, pattern } => logger.log_indented("Named Struct Field", |logger| {
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                logger.set_last_at_indent();
                logger.log_node(pattern);
            }),
            StructPatternField::TupleIndex { span, idx, pattern } => logger.log_indented("Tuple Index Struct Field", |logger| {
                logger.prefixed_log_fmt(format_args!("Index: {}\n", logger.resolve_literal(*idx)));
                logger.set_last_at_indent();
                logger.log_node(pattern);
            }),
            StructPatternField::Iden { span, is_ref, is_mut, iden, bound } => logger.log_indented("Iden Struct Field", |logger| {
                logger.prefixed_log_fmt(format_args!("Is Ref: {}\n", is_ref));
                logger.prefixed_log_fmt(format_args!("Is Mut: {}\n", is_mut));
                logger.set_last_at_indent();
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*iden)));
                logger.log_indented_opt_node("Bound", bound);
            }),
            StructPatternField::Rest => {
                logger.prefixed_logln("Rest Struct Field");
            },
        }
    }
}

// TODO: doesn't seem to get parsed, check this out
pub enum TupleStructPattern {
    Inferred {
        span:     SpanId,
        node_id:  NodeId,
        patterns: Vec<Pattern>,
    },
    Named {
        span:     SpanId,
        node_id:  NodeId,
        path:     AstNodeRef<ExprPath>,
        patterns: Vec<Pattern>,
    },
}

impl AstNode for TupleStructPattern {
    fn span(&self) -> SpanId {
        match self {
            TupleStructPattern::Inferred { span, .. } => *span,
            TupleStructPattern::Named { span, .. }    => *span,
        }    
    }

    fn node_id(&self) -> NodeId {
        match self {
            TupleStructPattern::Inferred { node_id, .. } => *node_id,
            TupleStructPattern::Named { node_id, .. }    => *node_id,
        }    
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Tuple Struct Pattern", |logger| {
            match self {
                TupleStructPattern::Inferred { span, node_id, patterns } => {
                    logger.set_last_at_indent();
                    logger.log_indented_node_slice("Patterns", patterns);
                },
                TupleStructPattern::Named { span, node_id, path, patterns } => {
                    logger.prefixed_logln("Inferred Path");
                    logger.set_last_at_indent();
                    logger.log_indented_node_slice("Patterns", patterns);
                },
            }
        });
    }
}

impl AstNodeParseHelper for TupleStructPattern {
    fn set_node_id(&mut self, ast_node_id: NodeId) {
        match self {
            TupleStructPattern::Inferred { node_id, .. } => *node_id = ast_node_id,
            TupleStructPattern::Named { node_id, .. }    => *node_id = ast_node_id,
        }
    }
}

pub struct TuplePattern {
    pub span:     SpanId,
    pub node_id:  NodeId,
    pub patterns: Vec<Pattern>
}

impl AstNode for TuplePattern {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Tuple Pattern", |logger| {
            logger.set_last_at_indent();
            logger.log_indented_node_slice("Patterns", &self.patterns);
        });
    }
}

impl AstNodeParseHelper for TuplePattern {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct GroupedPattern {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub pattern: Pattern
}

impl AstNode for GroupedPattern {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Grouped Pattern", |logger| {
            logger.set_last_at_indent();
            logger.log_node(&self.pattern);
        })
    }
}

impl AstNodeParseHelper for GroupedPattern {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct SlicePattern {
    pub span:     SpanId,
    pub node_id:  NodeId,
    pub patterns: Vec<Pattern>
}

impl AstNode for SlicePattern {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Slice Pattern", |logger| {
            logger.set_last_at_indent();
            logger.log_indented_node_slice("Patterns", &self.patterns);
        })
    }
}

impl AstNodeParseHelper for SlicePattern {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct EnumMemberPattern {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub name:    NameId,
}

impl AstNode for EnumMemberPattern {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Enum Member Pattern", |logger| {
            logger.prefixed_log_fmt(format_args!("Enum Member: {}\n", logger.resolve_name(self.name)));
        });
    }
}

impl AstNodeParseHelper for EnumMemberPattern {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct AlternativePattern {
    pub span:     SpanId,
    pub node_id:  NodeId,
    pub patterns: Vec<Pattern>,
}

impl AstNode for AlternativePattern {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Alternative Pattern", |logger| {
            logger.set_last_at_indent();
            logger.log_indented_node_slice("Patterns", &self.patterns);
        });
    }
}

impl AstNodeParseHelper for AlternativePattern {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct TypeCheckPattern {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub ty:      Type
}

impl AstNode for TypeCheckPattern {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Type Check Pattern", |logger| {
            logger.set_last_at_indent();
            logger.log_node(&self.ty);
        })
    }
}

impl AstNodeParseHelper for TypeCheckPattern {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

// =============================================================================================================================

pub enum Type {
    Paren(AstNodeRef<ParenthesizedType>),
    Primitive(AstNodeRef<PrimitiveType>),
    Unit(AstNodeRef<UnitType>),
    Never(AstNodeRef<NeverType>),
    Path(AstNodeRef<PathType>),
    Tuple(AstNodeRef<TupleType>),
    Array(AstNodeRef<ArrayType>),
    Slice(AstNodeRef<SliceType>),
    StringSlice(AstNodeRef<StringSliceType>),
    Pointer(AstNodeRef<PointerType>),
    Ref(AstNodeRef<ReferenceType>),
    Optional(AstNodeRef<OptionalType>),
    Fn(AstNodeRef<FnType>),
    Record(AstNodeRef<RecordType>),
    EnumRecord(AstNodeRef<EnumRecordType>),
}

impl AstNode for Type {
    fn span(&self) -> SpanId {
        match self {
            Type::Paren(ty)       => ty.span(),
            Type::Primitive(ty)   => ty.span(),
            Type::Never(ty)       => ty.span(),
            Type::Unit(ty)        => ty.span(),
            Type::Path(ty)        => ty.span(),
            Type::Tuple(ty)       => ty.span(),
            Type::Array(ty)       => ty.span(),
            Type::Slice(ty)       => ty.span(),
            Type::StringSlice(ty) => ty.span(),
            Type::Pointer(ty)     => ty.span(),
            Type::Ref(ty)         => ty.span(),
            Type::Optional(ty)    => ty.span(),
            Type::Fn(ty)          => ty.span(),
            Type::Record(ty)      => ty.span(),
            Type::EnumRecord(ty)  => ty.span(),
        }
    }

    fn node_id(&self) -> NodeId {
        match self {
            Type::Paren(ty)       => ty.node_id(),
            Type::Primitive(ty)   => ty.node_id(),
            Type::Never(ty)       => ty.node_id(),
            Type::Unit(ty)        => ty.node_id(),
            Type::Path(ty)        => ty.node_id(),
            Type::Tuple(ty)       => ty.node_id(),
            Type::Array(ty)       => ty.node_id(),
            Type::Slice(ty)       => ty.node_id(),
            Type::StringSlice(ty) => ty.node_id(),
            Type::Pointer(ty)     => ty.node_id(),
            Type::Ref(ty)         => ty.node_id(),
            Type::Optional(ty)    => ty.node_id(),
            Type::Fn(ty)          => ty.node_id(),
            Type::Record(ty)      => ty.node_id(),
            Type::EnumRecord(ty)  => ty.node_id(),
        }
    }

    fn log(&self, logger: &mut AstLogger) {
        match self {
            Type::Paren(ty)       => logger.log_node_ref(ty),
            Type::Primitive(ty)   => logger.log_node_ref(ty),
            Type::Never(ty)       => logger.log_node_ref(ty),
            Type::Unit(ty)        => logger.log_node_ref(ty),
            Type::Path(ty)        => logger.log_node_ref(ty),
            Type::Tuple(ty)       => logger.log_node_ref(ty),
            Type::Array(ty)       => logger.log_node_ref(ty),
            Type::Slice(ty)       => logger.log_node_ref(ty),
            Type::StringSlice(ty) => logger.log_node_ref(ty),
            Type::Pointer(ty)     => logger.log_node_ref(ty),
            Type::Ref(ty)         => logger.log_node_ref(ty),
            Type::Optional(ty)    => logger.log_node_ref(ty),
            Type::Fn(ty)          => logger.log_node_ref(ty),
            Type::Record(ty)      => logger.log_node_ref(ty),
            Type::EnumRecord(ty)  => logger.log_node_ref(ty),
        }
    }
}

pub struct ParenthesizedType {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub ty:      Type,
}

impl AstNode for ParenthesizedType {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Parenthesized Type", |logger| {
            logger.set_last_at_indent();
            logger.log_node(&self.ty)
        })
    }
}

impl AstNodeParseHelper for ParenthesizedType {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct PrimitiveType {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub ty:      type_system::PrimitiveType,
}

impl AstNode for PrimitiveType {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Primitive Type", |logger| {
            logger.set_last_at_indent();
            logger.prefixed_log("Primitive: ");
            match self.ty {
                type_system::PrimitiveType::U8     => logger.logln("u8"),
                type_system::PrimitiveType::U16    => logger.logln("u16"),
                type_system::PrimitiveType::U32    => logger.logln("u32"),
                type_system::PrimitiveType::U64    => logger.logln("u64"),
                type_system::PrimitiveType::U128   => logger.logln("u128"),
                type_system::PrimitiveType::Usize  => logger.logln("usize"),
                type_system::PrimitiveType::I8     => logger.logln("i8"),
                type_system::PrimitiveType::I16    => logger.logln("i16"),
                type_system::PrimitiveType::I32    => logger.logln("i32"),
                type_system::PrimitiveType::I64    => logger.logln("i64"),
                type_system::PrimitiveType::I128   => logger.logln("i128"),
                type_system::PrimitiveType::Isize  => logger.logln("isize"),
                type_system::PrimitiveType::F16    => logger.logln("f16"),
                type_system::PrimitiveType::F32    => logger.logln("f32"),
                type_system::PrimitiveType::F64    => logger.logln("f64"),
                type_system::PrimitiveType::F128   => logger.logln("f128"),
                type_system::PrimitiveType::Bool   => logger.logln("bool"),
                type_system::PrimitiveType::B8     => logger.logln("b8"),
                type_system::PrimitiveType::B16    => logger.logln("b16"),
                type_system::PrimitiveType::B32    => logger.logln("b32"),
                type_system::PrimitiveType::B64    => logger.logln("b64"),
                type_system::PrimitiveType::Char   => logger.logln("char"),
                type_system::PrimitiveType::Char7  => logger.logln("char7"),
                type_system::PrimitiveType::Char8  => logger.logln("char8"),
                type_system::PrimitiveType::Char16 => logger.logln("char16"),
                type_system::PrimitiveType::Char32 => logger.logln("char32"),
            }
        });
    }
}

impl AstNodeParseHelper for PrimitiveType {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct UnitType {
    pub span:    SpanId,
    pub node_id: NodeId,
}

impl AstNode for UnitType {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.prefixed_logln("Unit Type")
    }
}

impl AstNodeParseHelper for UnitType {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct NeverType {
    pub span:    SpanId,
    pub node_id: NodeId,
}

impl AstNode for NeverType {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.prefixed_logln("Never Type")
    }
}

impl AstNodeParseHelper for NeverType {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct PathType {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub path:    AstNodeRef<TypePath>,
}

impl AstNode for PathType {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Path Type", |logger| {
            logger.log_node_ref(&self.path);
        });
    }
}

impl AstNodeParseHelper for PathType {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct TupleType {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub types:   Vec<Type>,
}

impl AstNode for TupleType {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Tuple Type", |logger| {
            logger.log_indented_node_slice("Types", &self.types);
        });
    }
}

impl AstNodeParseHelper for TupleType {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct ArrayType {
    pub span:     SpanId,
    pub node_id:  NodeId,
    pub size:     Expr,
    pub sentinel: Option<Expr>,
    pub ty:       Type,
}

impl AstNode for ArrayType {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Array Type", |logger| {
            logger.log_indented_node("Size", &self.size);
            logger.log_indented_opt_node("Sentinel", &self.sentinel);
            logger.set_last_at_indent();
            logger.log_node(&self.ty);
        });
    }
}

impl AstNodeParseHelper for ArrayType {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct SliceType {
    pub span:     SpanId,
    pub node_id:  NodeId,
    pub sentinel: Option<Expr>,
    pub ty:       Type,
}

impl AstNode for SliceType {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Slice Type", |logger| {
            logger.log_indented_opt_node("Sentinel", &self.sentinel);
            logger.set_last_at_indent();
            logger.log_node(&self.ty);
        });
    }
}

impl AstNodeParseHelper for SliceType {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct StringSliceType {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub ty:      type_system::StringSliceType,
}

impl AstNode for StringSliceType {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("String Slice Type", |logger| {
            logger.prefixed_log("StringSlice: ");
            match self.ty {
                type_system::StringSliceType::Str   => logger.logln("str"),
                type_system::StringSliceType::Str7  => logger.logln("str7"),
                type_system::StringSliceType::Str8  => logger.logln("str8"),
                type_system::StringSliceType::Str16 => logger.logln("str16"),
                type_system::StringSliceType::Str32 => logger.logln("str32"),
                type_system::StringSliceType::CStr  => logger.logln("cstr"),
            }
        });
    }
}

impl AstNodeParseHelper for StringSliceType {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct PointerType {
    pub span:     SpanId,
    pub node_id:  NodeId,
    pub is_multi: bool,
    pub is_mut:   bool,
    pub ty:       Type,
    pub sentinel: Option<Expr>,
}

impl AstNode for PointerType {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Pointer Type", |logger| {
            logger.prefixed_log_fmt(format_args!("Is Multi-elem: {}\n", self.is_multi));
            logger.prefixed_log_fmt(format_args!("Is Mut: {}\n", self.is_mut));
            
            logger.set_last_at_indent_if(self.sentinel.is_none());
            logger.log_indented_node("Type", &self.ty);
            
            logger.set_last_at_indent();
            logger.log_indented_opt_node("Sentinel", &self.sentinel);
        });
    }
}

impl AstNodeParseHelper for PointerType {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct ReferenceType {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub is_mut:  bool,
    pub ty:      Type,
}

impl AstNode for ReferenceType {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Reference Type", |logger| {
            logger.prefixed_log_fmt(format_args!("Is Mut: {}\n", self.is_mut));
            
            logger.set_last_at_indent();
            logger.log_indented_node("Type", &self.ty);
        });
    }
}

impl AstNodeParseHelper for ReferenceType {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct OptionalType {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub ty:     Type,
}

impl AstNode for OptionalType {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Optional Type", |logger| {
            logger.set_last_at_indent();
            logger.log_node(&self.ty);
        });
    }
}

impl AstNodeParseHelper for OptionalType {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct FnType {
    pub span:      SpanId,
    pub node_id:   NodeId,
    pub is_unsafe: bool,
    pub abi:       Option<LiteralId>,
    pub params:    Vec<(Vec<NameId>, Type)>,
    pub return_ty: Option<Type>,
}

impl AstNode for FnType {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Function Type", |logger| {
            logger.prefixed_log_fmt(format_args!("Is unsafe: {}\n", self.is_unsafe));

            logger.set_last_at_indent_if(self.params.is_empty() && self.return_ty.is_none());
            if let Some(abi) = self.abi {
                logger.prefixed_log_fmt(format_args!("ABI: {}\n", &logger.resolve_literal(abi)))
            }

            logger.set_last_at_indent_if(self.return_ty.is_none());
            logger.log_indented_slice("Params", &self.params, |logger, (names, ty)| {
                logger.log_indented_slice("Names", &names, |logger, name| {
                    logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                });
                logger.set_last_at_indent();
                logger.log_indented_node("Type", ty);
            });

            logger.set_last_at_indent();
            logger.log_indented_opt_node("Return Type", &self.return_ty);
        });
    }
}

impl AstNodeParseHelper for FnType {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct RecordType {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub fields:  Vec<RegStructField>
}

impl AstNode for RecordType {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Record type", |logger| {
            logger.set_last_at_indent();
            logger.log_indented_slice("Fields", &self.fields, |logger, field| field.log(logger));
        });
    }
}

impl AstNodeParseHelper for RecordType {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub struct EnumRecordType {
    pub span:     SpanId,
    pub node_id:  NodeId,
    pub variants: Vec<EnumVariant>,
}

impl AstNode for EnumRecordType {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Enum Record Type", |logger| {
            logger.set_last_at_indent();
            logger.log_indented_slice("Variants", &self.variants, |logger, variant| variant.log(logger));
        });
    }
}

impl AstNodeParseHelper for EnumRecordType {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

// =============================================================================================================================

pub struct GenericParams {

}
impl AstNode for GenericParams {
    fn span(&self) -> SpanId {
        SpanId::INVALID
    }

    fn node_id(&self) -> NodeId {
        NodeId::default()
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_indented("Generic Params", |logger| {

        });
    }
}

impl AstNodeParseHelper for GenericParams {
    fn set_node_id(&mut self, node_id: NodeId) {
        
    }
}

pub struct GenericArgs {

}
impl AstNode for GenericArgs {
    fn span(&self) -> SpanId {
        SpanId::INVALID
    }

    fn node_id(&self) -> NodeId {
        NodeId::default()
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_indented("Generic Args", |logger| {

        });
    }
}

impl AstNodeParseHelper for GenericArgs {
    fn set_node_id(&mut self, node_id: NodeId) {
        
    }
}

pub struct WhereClause {

}
impl AstNode for WhereClause {
    fn span(&self) -> SpanId {
        SpanId::INVALID
    }

    fn node_id(&self) -> NodeId {
        NodeId::default()
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_indented("Where Clause", |logger| {

        });
    }
}

impl AstNodeParseHelper for WhereClause {
    fn set_node_id(&mut self, node_id: NodeId) {
        
    }
}

pub struct TraitBounds {
    
}
impl AstNode for TraitBounds {
    fn span(&self) -> SpanId {
        SpanId::INVALID
    }

    fn node_id(&self) -> NodeId {
        NodeId::default()
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_indented("Trait Bounds", |logger| {

        });  
    }
}

impl AstNodeParseHelper for TraitBounds {
    fn set_node_id(&mut self, node_id: NodeId) {
        
    }
}

// =============================================================================================================================

pub enum Visibility {
    Pub {
        span:    SpanId,
        node_id: NodeId,
    },
    Super {
        span:    SpanId,
        node_id: NodeId,
    },
    Lib {
        span:    SpanId,
        node_id: NodeId,
    },
    Package {
        span:    SpanId,
        node_id: NodeId,
    },
    Path{
        span:    SpanId,
        node_id: NodeId,
        path:    AstNodeRef<SimplePath>
    }
}

impl AstNode for Visibility {
    fn span(&self) -> SpanId {
        match self {
            Visibility::Pub { span, .. }     => *span,
            Visibility::Super { span, .. }   => *span,
            Visibility::Lib { span, .. }     => *span,
            Visibility::Package { span, .. } => *span,
            Visibility::Path { span, .. }    => *span,
        }    
    }

    fn node_id(&self) -> NodeId {
        match self {
            Visibility::Pub { node_id, .. }     => *node_id,
            Visibility::Super { node_id, .. }   => *node_id,
            Visibility::Lib { node_id, .. }     => *node_id,
            Visibility::Package { node_id, .. } => *node_id,
            Visibility::Path { node_id, .. }    => *node_id,
        }    
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.prefixed_log("Visibility: ");
        match self {
            Visibility::Pub{ .. }        => logger.logln("pub"),
            Visibility::Super{ .. }      => logger.logln("pub(super)"),
            Visibility::Lib{ .. }        => logger.logln("pub(lib)"),
            Visibility::Package{ .. }    => logger.logln("pub(package)"),
            Visibility::Path{ path, .. } => logger.log_indented_node_ref("pub(..)", path),
        }
    }
}

impl AstNodeParseHelper for Visibility {
    fn set_node_id(&mut self, ast_node_id: NodeId) {
        match self {
            Visibility::Pub { span, node_id } => *node_id = ast_node_id,
            Visibility::Super { span, node_id } => *node_id = ast_node_id  ,
            Visibility::Lib { span, node_id } => *node_id = ast_node_id,
            Visibility::Package { span, node_id } => *node_id = ast_node_id,
            Visibility::Path { span, node_id, path } => *node_id = ast_node_id ,
        }
    }
}

// =============================================================================================================================

pub struct Attribute {
    pub span:    SpanId,
    pub node_id: NodeId,
    pub is_mod:  bool,
    pub metas:   Vec<AttribMeta>,
}

impl AstNode for Attribute {
    fn span(&self) -> SpanId {
        self.span
    }

    fn node_id(&self) -> NodeId {
        self.node_id
    }

    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Attribute", |logger| {
            logger.prefixed_log_fmt(format_args!("Is Module Attribute: {}\n", self.is_mod));
            logger.set_last_at_indent();
            logger.log_indented_slice("Meta", &self.metas, |logger, meta| meta.log(logger));
        })
    }
}

impl AstNodeParseHelper for Attribute {
    fn set_node_id(&mut self, node_id: NodeId) {
        self.node_id = node_id;
    }
}

pub enum AttribMeta {
    Simple {
        span:    SpanId,
        node_id: NodeId,
        path:    AstNodeRef<SimplePath>,
    },
    Expr {
        span:    SpanId,
        node_id: NodeId,
        expr:    Expr,
    },
    Assign {
        span:    SpanId,
        node_id: NodeId,
        path:    AstNodeRef<SimplePath>,
        expr:    Expr
    },
    Meta {
        span:    SpanId,
        node_id: NodeId,
        path:    AstNodeRef<SimplePath>,
        metas:   Vec<AttribMeta>,
    }
}

impl AttribMeta {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            Self::Simple { path, .. }       => logger.log_indented_node_ref("Simple Attrib Meta", path),
            Self::Expr { expr, .. }         => logger.log_indented_node("Expression Attrib Meta", expr),
            Self::Assign { path, expr, .. } => logger.log_indented("Assign Attribute Meta", |logger| {
                logger.log_indented_node_ref("Path", path);
                logger.set_last_at_indent();
                logger.log_indented_node("Expr", expr)
            }),
            Self::Meta { path, metas, .. }  => logger.log_indented("Nested Attribute Meta", |logger| {
                logger.log_indented_node_ref("Path", path);
                logger.set_last_at_indent();
                logger.log_indented_slice("Metas", metas, |logger, meta| meta.log(logger));
            }),
        }
    }
}

// =============================================================================================================================

pub struct Contract {

}

impl AstNode for Contract {
    fn span(&self) -> SpanId {
        SpanId::INVALID
    }

    fn node_id(&self) -> NodeId {
        NodeId::default()
    }

    fn log(&self, logger: &mut AstLogger) {
        
    }
}

impl AstNodeParseHelper for Contract {
    fn set_node_id(&mut self, node_id: NodeId) {
    }
}

// =============================================================================================================================

// #[derive(PartialEq, Eq, Debug)]
// pub struct AstNodeRef<T> {
//     idx:      usize,
//     _phantom: PhantomData<T>,
// }

// impl<T> Clone for AstNodeRef<T> {
//     fn clone(&self) -> Self {
//         *self
//     }
// }
// impl<T> Copy for AstNodeRef<T> {}

// impl<T> AstNodeRef<T> {
//     pub fn index(&self) -> usize {
//         self.idx
//     }
// }

pub type AstNodeRef<T> = Arc<T>;



pub struct Ast {
    pub file:   PathBuf,
    pub nodes:  Vec<Arc<dyn AstNode>>,
    pub meta:   Vec<AstNodeMeta>,

    pub items:  Vec<Item>,

    pub tokens: TokenStore,
}

impl Ast {
    pub fn new() -> Self {
        Self {
            file:  PathBuf::new(),
            nodes: Vec::new(),
            meta:  Vec::new(),
            items: Vec::new(),
            tokens: TokenStore::new_dummy(),
        }
    }

    pub fn add_node<T: AstNode + AstNodeParseHelper + 'static>(&mut self, mut node: T, meta: AstNodeMeta) -> AstNodeRef<T> {
        let idx = self.nodes.len();
        node.set_node_id(NodeId(idx));
        let node_ref = Arc::new(node);
        self.nodes.push(node_ref.clone());
        self.meta.push(meta);
        node_ref
    }

    pub fn log(&self, names: &NameTable, literals: &LiteralTable, puncts: &PuncutationTable) {
        let mut logger = AstLogger::new(self, names, literals, puncts);

        let path = self.file.to_str().unwrap();
        logger.log_fmt(format_args!("AST for File: {path}\n"));
        for (idx, item) in self.items.iter().enumerate() {
            if idx == self.items.len() - 1 {
                logger.set_last_at_indent();
            }
            item.log(&mut logger);
        }

    }
}

pub struct AstLogger<'a> {
    ast:          &'a Ast,
    names:        &'a NameTable,
    literals:     &'a LiteralTable,
    puncts:       &'a PuncutationTable,

    logger:       IndentLogger,
    indent_kinds: Vec<bool>,

    node_id:      NodeId,
}

impl<'a> AstLogger<'a> {
    pub fn new(ast: &'a Ast, names: &'a NameTable, literals: &'a LiteralTable, puncts: &'a PuncutationTable) -> Self {
        Self {
            ast,
            names,
            literals,
            puncts,
            logger: IndentLogger::new("    ", "|   ", "+---"),
            indent_kinds: vec![true],
            node_id: NodeId::default(),
        }
    }
}

impl AstLogger<'_> {
    pub fn log(&self, s: &str) {
        self.logger.log(s);
    }
    
    pub fn prefixed_log(&self, s: &str) {
        self.logger.prefixed_log(s);
    }
    
    pub fn logln(&self, s: &str) {
        self.logger.logln(s);
    }

    pub fn prefixed_logln(&self, s: &str) {
        self.logger.prefixed_logln(s);
    }

    pub fn log_fmt(&self, args: fmt::Arguments) {
        self.logger.log_fmt(args);
    }

    pub fn prefixed_log_fmt(&self, args: fmt::Arguments) {
        self.logger.prefixed_log_fmt(args);
    }

    pub fn push_indent(&mut self) {
        self.logger.push_indent();
    }

    pub fn pop_indent(&mut self) {
        self.logger.pop_indent();
    }

    pub fn set_last_at_indent(&mut self) {
        self.logger.set_last_at_indent();
    }

    pub fn set_last_at_indent_if(&mut self, cond: bool) {
        self.logger.set_last_at_indent_if(cond);
    }

    pub fn resolve_name(&self, id: NameId) -> &str {
        &self.names[id]
    }

    pub fn resolve_literal(&self, id: LiteralId) -> String {
        self.literals[id].to_string()
    }

    pub fn resolve_punctuation(&self, punt: Punctuation) -> String {
        punt.as_str(&self.puncts).to_string()
    }

    fn log_node_ref<T: AstNode + 'static>(&mut self, node: &AstNodeRef<T>) {
        let prev_id = self.node_id;
        self.node_id = node.node_id();
        
        node.log(self);
        
        self.node_id = prev_id;
    }

    fn log_node<T: AstNode + 'static>(&mut self, node: &T) {
        node.log(self);
    }

    pub fn log_node_slice<T: AstNode + 'static>(&mut self, nodes: &[T]) {
        if nodes.is_empty() {
            return;
        }

        for (idx, node) in nodes.iter().enumerate() {
            if idx == nodes.len() - 1 {
                self.set_last_at_indent();
            }
            node.log(self);
        }
    }

    pub fn log_ast_node<F>(&mut self, name: &'static str, f: F) where
        F: Fn(&mut Self)
    {
        let (first_tok, last_tok) = if self.node_id.index() < self.ast.meta.len() {
            let meta = &self.ast.meta[self.node_id.index()];
            (meta.first_tok, meta.last_tok)
        } else {
            (0, 0)
        };


        self.prefixed_log_fmt(format_args!("[ {name} ] (node id: {}, tokens: [{first_tok}..{last_tok}])\n", self.node_id.index()));
        self.push_indent();
        f(self);
        self.pop_indent();
    }

    pub fn log_indented<F>(&mut self, name: &'static str, f: F) where
        F: Fn(&mut Self)
    {
        self.prefixed_logln(name);
        self.push_indent();
        f(self);
        self.pop_indent();
    }

    pub fn log_indented_slice<T, F>(&mut self, name: &'static str, slice: &[T], f: F) where 
        F: Fn(&mut Self, &T)
    {
        if slice.is_empty() {
            return;
        }

        self.log_indented(name, |logger| for (idx, val) in slice.iter().enumerate() {
            if idx == slice.len() - 1 {
                logger.set_last_at_indent();
            }
            f(logger, val);
        })
    }

    pub fn log_indented_node_ref<T: AstNode + 'static>(&mut self, name: &'static str, node: &AstNodeRef<T>) {
        self.node_id = node.node_id();
        self.log_indented_node(name, &**node);
    }

    pub fn log_indented_node<T: AstNode>(&mut self, name: &'static str, node: &T) {
        self.prefixed_logln(name);
        self.push_indent();
        self.set_last_at_indent();
        node.log(self);
        self.pop_indent();
    }

    pub fn log_indented_node_ref_slice<T: AstNode + 'static>(&mut self, name: &'static str, nodes: &[AstNodeRef<T>]) {
        if nodes.is_empty() {
            return;
        }

        self.log_indented(name, |logger| for (idx, node) in nodes.iter().enumerate() {
            if idx == nodes.len() - 1 {
                logger.set_last_at_indent();
            }
            logger.log_node_ref(node);
        });
    }

    pub fn log_indented_node_slice<T: AstNode + 'static>(&mut self, name: &'static str, nodes: &[T]) {
        if nodes.is_empty() {
            return;
        }

        self.log_indented(name, |logger| for (idx, node) in nodes.iter().enumerate() {
            if idx == nodes.len() - 1 {
                logger.set_last_at_indent();
            }
            node.log(logger);
        });
    }

    pub fn log_opt<T, F>(&mut self, val: &Option<T>, f: F) where
        F: Fn(&mut Self, &T)
    {
        if let Some(val) = val {
            f(self, val)
        }
    }

    pub fn log_indented_opt<T, F>(&mut self, name: &'static str, val: &Option<T>, f: F) where
        F: Fn(&mut Self, &T)
    {
        if let Some(val) = val {
            self.log_indented(name, |logger| f(logger, val))
        }
    }

    pub fn log_opt_node_ref<T: AstNode + 'static>(&mut self, node: &Option<AstNodeRef<T>>) {
        if let Some(id) = node {
            self.log_node_ref(id);
        }
    }

    pub fn log_indented_opt_node<T: AstNode + 'static>(&mut self, name: &'static str, val: &Option<T>) {
        if let Some(val) = val {
            self.log_indented_node(name, val)
        }
    }
    
    pub fn log_indented_opt_node_ref<T: AstNode + 'static>(&mut self, name: &'static str, val: &Option<AstNodeRef<T>>) {
        if let Some(val) = val {
            self.log_indented_node_ref(name, val)
        }
    }
    
    pub fn log_opt_node<T: AstNode + 'static>(&mut self, node: &Option<T>) {
        if let Some(node) = node {
            node.log(self);
        }
    }
}