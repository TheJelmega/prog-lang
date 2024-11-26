#![allow(unused)]

use std::{
    fmt::{self, write, Write as _},
    io::{Stdout, Write},
    marker::PhantomData,
    ops::{Index, IndexMut},
    path::{self, PathBuf},
};

use crate::{
    common::{IndentLogger, NameId, NameTable},
    lexer::{Punctuation, PuncutationTable, StrongKeyword, WeakKeyword},
    literals::{LiteralId, LiteralTable},
};


mod parser;
pub use parser::{Parser, ParserErr};

mod visitor;
pub use visitor::{Visitor, helpers};

pub trait AstNode {


    fn log(&self, logger: &mut AstLogger);
}

pub struct AstNodeMeta {
    pub first_tok: u32,
    pub last_tok: u32,
}

pub struct Identifier {
    pub name:     NameId,
    pub gen_args: Option<AstNodeRef<GenericArgs>>,
}

impl Identifier {
    pub fn log(&self, logger: &mut AstLogger) {
        logger.log_indented("Identifier", |logger| {
            logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(self.name)));
            if let Some(gen_args) = self.gen_args {
                logger.log_node_ref(gen_args);
            }
        })
    }
}

pub enum SimplePathStart {
    None,
    Inferred,
    SelfPath,
    Super,
}

impl SimplePathStart {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            SimplePathStart::None     => {},
            SimplePathStart::Inferred => logger.prefixed_logln("Path Start: Inferred"),
            SimplePathStart::SelfPath => logger.prefixed_logln("Path Start: self"),
            SimplePathStart::Super    => logger.prefixed_logln("Path Start: super"),
        }
    }
}

pub struct SimplePath {
    pub start:             SimplePathStart,
    pub names:             Vec<NameId>
}

impl AstNode for SimplePath {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Simple Path", |logger| {
            self.start.log(logger);
            for (idx, name) in self.names.iter().enumerate() {
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

pub enum TypePathIdentifier {
    Plain {
        name: NameId
    },
    GenArg{
        name: NameId,
        gen_args: AstNodeRef<GenericArgs>,
    },
    Fn {
        name: NameId,
        params: Vec<Type>,
        ret:    Option<Type>
    },
}

pub struct ExprPath {
    pub inferred: bool,
    pub idens:    Vec<Identifier>,
}

impl AstNode for ExprPath {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Expr Path", |logger| {
            logger.prefixed_log_fmt(format_args!("Is Inferred: {}\n", self.inferred));
            logger.set_last_at_indent();
            logger.log_indented_slice("Identifiers", &self.idens, |logger, iden| iden.log(logger));
        });
    }
}

pub struct TypePath {
    pub idens: Vec<TypePathIdentifier>,
}

impl AstNode for TypePath {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Type Path", |logger| for (idx, iden) in self.idens.iter().enumerate() {
            if idx == self.idens.len() - 1 {
                logger.set_last_at_indent();
            }

            match iden {
                TypePathIdentifier::Plain { name } => {
                    logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                },
                TypePathIdentifier::GenArg { name, gen_args } => logger.log_indented("Identifier", |logger| {
                    logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                    logger.set_last_at_indent();
                    logger.log_node_ref(*gen_args);
                }),
                TypePathIdentifier::Fn { name, params, ret } => logger.log_indented("Function Identifier", |logger| {
                    logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                }),
            }
        })
    }
}

pub struct QualifiedPath {
    pub ty:       Type,
    pub bound:    Option<Identifier>,
    pub sub_path: Vec<Identifier>,
}

impl AstNode for QualifiedPath {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_indented("Qualified Path", |logger| {
            logger.log_indented_node("Type", &self.ty);
            if let Some(bound) = &self.bound {
                logger.log_indented("Bound", |logger| bound.log(logger));
            }
            
            logger.set_last_at_indent();
            logger.log_indented_slice("Sub Path", &self.sub_path, |logger, iden| iden.log(logger));
        })
    }
}

pub struct Block {
    pub stmts:      Vec<Stmt>,
    pub final_expr: Option<AstNodeRef<ExprStmt>>,
}
impl AstNode for Block {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Block", |logger| {
            logger.set_last_at_indent_if(self.final_expr.is_none());
            logger.log_indented_node_slice("Statements", &self.stmts);
            logger.set_last_at_indent();
            logger.log_indented_opt_node_ref("Final expression", &self.final_expr);
        });
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
    CustomOp(AstNodeRef<OpTrait>),
    Precedence(AstNodeRef<Precedence>),
}

impl Item {
    pub fn node_id(&self) -> usize {
        match self {
            Item::Module(node_id)     => node_id.idx,
            Item::Use(node_id)        => node_id.idx,
            Item::Function(node_id)   => node_id.idx,
            Item::TypeAlias(node_id)  => node_id.idx,
            Item::Struct(node_id)     => node_id.idx,
            Item::Union(node_id)      => node_id.idx,
            Item::Enum(node_id)       => node_id.idx,
            Item::Bitfield(node_id)   => node_id.idx,
            Item::Const(node_id)      => node_id.idx,
            Item::Static(node_id)     => node_id.idx,
            Item::Property(node_id)   => node_id.idx,
            Item::Trait(node_id)      => node_id.idx,
            Item::Impl(node_id)       => node_id.idx,
            Item::Extern(node_id)     => node_id.idx,
            Item::CustomOp(node_id)   => node_id.idx,
            Item::Precedence(node_id) => node_id.idx,
        }
    }
}

impl AstNode for Item {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            Self::Module(module)          => logger.log_node_ref(*module),
            Self::Use(use_item)           => logger.log_node_ref(*use_item),
            Self::Function(fn_item)       => logger.log_node_ref(*fn_item),
            Self::TypeAlias(type_alias)   => logger.log_node_ref(*type_alias),
            Self::Struct(struct_item)     => logger.log_node_ref(*struct_item),
            Self::Union(union_item)       => logger.log_node_ref(*union_item),
            Self::Enum(enum_item)         => logger.log_node_ref(*enum_item),
            Self::Bitfield(bitfield_item) => logger.log_node_ref(*bitfield_item),
            Self::Const(const_item)       => logger.log_node_ref(*const_item),
            Self::Static(static_item)     => logger.log_node_ref(*static_item),
            Self::Property(prop_item)     => logger.log_node_ref(*prop_item),
            Self::Trait(trait_item)       => logger.log_node_ref(*trait_item),
            Self::Impl(impl_item)         => logger.log_node_ref(*impl_item),
            Self::Extern(impl_block)      => logger.log_node_ref(*impl_block),
            Self::CustomOp(impl_block)    => logger.log_node_ref(*impl_block),
            Self::Precedence(impl_block)  => logger.log_node_ref(*impl_block),
        }
    }
}

pub enum TraitItem {
    Function(AstNodeRef<Function>),
    TypeAlias(AstNodeRef<TypeAlias>),
    Const(AstNodeRef<Const>),
    Property(AstNodeRef<Property>),
}

impl TraitItem {
    pub fn node_id(&self) -> usize {
        match self {
            TraitItem::Function(node_id)  => node_id.idx,
            TraitItem::TypeAlias(node_id) => node_id.idx,
            TraitItem::Const(node_id)     => node_id.idx,
            TraitItem::Property(node_id)  => node_id.idx,
        }
    }
}

impl AstNode for TraitItem {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            Self::Function(fn_item)       => logger.log_node_ref(*fn_item),
            Self::TypeAlias(type_alias)   => logger.log_node_ref(*type_alias),
            Self::Const(const_item)       => logger.log_node_ref(*const_item),
            Self::Property(prop_item)     => logger.log_node_ref(*prop_item),
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

impl AssocItem {
    pub fn node_id(&self) -> usize {
        match self {
            AssocItem::Function(node_id)  => node_id.idx,
            AssocItem::TypeAlias(node_id) => node_id.idx,
            AssocItem::Const(node_id)     => node_id.idx,
            AssocItem::Static(node_id)    => node_id.idx,
            AssocItem::Property(node_id)  => node_id.idx,
        }
    }
}

impl AstNode for AssocItem {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            Self::Function(fn_item)       => logger.log_node_ref(*fn_item),
            Self::TypeAlias(type_alias)   => logger.log_node_ref(*type_alias),
            Self::Const(const_item)       => logger.log_node_ref(*const_item),
            Self::Static(static_item)     => logger.log_node_ref(*static_item),
            Self::Property(prop_item)     => logger.log_node_ref(*prop_item),
        }
    }
}

pub enum ExternItem {
    Function(AstNodeRef<Function>),
    Static(AstNodeRef<Static>),
}

impl ExternItem {
    pub fn node_id(&self) -> usize {
        match self {
            ExternItem::Function(node_id) => node_id.idx,
            ExternItem::Static(node_id)   => node_id.idx,
        }
    }
}

impl AstNode for ExternItem {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            Self::Function(fn_item)       => logger.log_node_ref(*fn_item),
            Self::Static(static_item)     => logger.log_node_ref(*static_item),
        }
    }
}

pub struct ModuleItem {
    pub attrs: Vec<AstNodeRef<Attribute>>,
    pub vis:   Option<AstNodeRef<Visibility>>,
    pub name:  NameId,
    pub block: Option<AstNodeRef<Block>>,
}

impl AstNode for ModuleItem {
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


pub struct UseItem {
    pub attrs:   Vec<AstNodeRef<Attribute>>,
    pub vis:     Option<AstNodeRef<Visibility>>,
    pub group:   Option<NameId>,
    pub package: Option<NameId>,
    pub module:  Option<NameId>,
    pub path:    AstNodeRef<UsePath>,
}

impl AstNode for UseItem {
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
            logger.log_node_ref(self.path);
        });                                         
    }
}

pub enum UsePath {
    SelfPath{
        alias: Option<NameId>
    },
    SubPaths{
        segments: Vec<NameId>,
        sub_paths: Vec<AstNodeRef<UsePath>>,
    },
    Alias{
        segments: Vec<NameId>,
        alias:     Option<NameId>,
    }
}
impl AstNode for UsePath {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Use Path", |logger| {
            match self {
                UsePath::SelfPath { alias } => {
                    logger.set_last_at_indent();
                    logger.prefixed_log("SelfPath");
                    
                    if let Some(alias) = alias {
                        logger.log_fmt(format_args!(", alias: {}\n", logger.resolve_name(*alias)));
                    }
                    logger.logln("");
                },
                UsePath::SubPaths { segments, sub_paths } => {
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
                UsePath::Alias { segments, alias } => {
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

pub struct Function {
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
    pub body:         AstNodeRef<Block>,
}

impl AstNode for Function {
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
            logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(self.name)));
            
           
            logger.log_opt_node_ref(&self.generics);
            logger.log_opt(&self.receiver, |logger, rec| rec.log(logger));
            logger.log_indented_slice("Params", &self.params, |logger, param| param.log(logger));
            logger.log_opt(&self.returns, |logger, ret| ret.log(logger));
            logger.set_last_at_indent();
            logger.log_node_ref(self.body);
        })
    }
}

pub enum FnReceiver {
    SelfReceiver{
        is_ref: bool,
        is_mut: bool,
    },
    SelfTyped{
        is_mut: bool,
        ty:     Type,  
    },
}

impl FnReceiver {
    pub fn log(&self, logger: &mut AstLogger) {
        match self {
            FnReceiver::SelfReceiver { is_ref, is_mut } => logger.log_indented("Self Receiver", |logger| {
                logger.prefixed_log_fmt(format_args!("Is Ref: {is_ref}\n"));
                logger.prefixed_log_fmt(format_args!("Is Mut: {is_mut}\n"));
            }),
            FnReceiver::SelfTyped{ is_mut, ty } => logger.log_indented("Typed Receiver", |logger| {
                logger.prefixed_log_fmt(format_args!("Is Mut: {is_mut}\n"));
                logger.set_last_at_indent();
                logger.log_indented_node("Type", ty);
            }),
        }
    }
}

pub struct FnParam {
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
    Type(Type),
    Named(Vec<(Vec<NameId>, Type)>)
}

impl FnReturn {
    pub fn log(&self, logger: &mut AstLogger) {
        match self {
            FnReturn::Type(ty) => logger.log_indented_node("Typed Function Return", ty),
            FnReturn::Named(rets) => logger.log_indented_slice("Named Function Return", rets, |logger, (names, ty)| {
                logger.log_indented("Return", |logger| {
                    logger.log_indented_slice("Names", &names, |logger, name| {
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
        attrs:       Vec<AstNodeRef<Attribute>>,
        vis:         Option<AstNodeRef<Visibility>>,
        name:        NameId,
        generics:    Option<AstNodeRef<GenericParams>>,
        ty:          Type,
    },
    Distinct {
        attrs:       Vec<AstNodeRef<Attribute>>,
        vis:         Option<AstNodeRef<Visibility>>,
        name:        NameId,
        generics:    Option<AstNodeRef<GenericParams>>,
        ty:          Type,
    },
    Trait {
        attrs:       Vec<AstNodeRef<Attribute>>,
        name:        NameId,
        generics:    Option<AstNodeRef<GenericParams>>,
    },
    Opaque {
        attrs:       Vec<AstNodeRef<Attribute>>,
        vis:         Option<AstNodeRef<Visibility>>,
        name:        NameId,
        size:        Option<Expr>,
    }
}

impl AstNode for TypeAlias {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            TypeAlias::Normal { attrs, vis, name, generics, ty } => logger.log_ast_node("Typealias", |logger| {
                logger.log_indented_node_ref_slice("Attributes", attrs);
                logger.log_opt_node_ref(vis);
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                logger.log_opt_node_ref(generics);
                logger.set_last_at_indent();
                logger.log_indented_node("Type", ty);
            }),
            TypeAlias::Distinct { attrs, vis, name, generics, ty } => logger.log_ast_node("Distinct Typealias", |logger| {
                logger.log_indented_node_ref_slice("Attributes", attrs);
                logger.log_opt_node_ref(vis);
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                logger.log_opt_node_ref(generics);
                logger.set_last_at_indent();
                logger.log_indented_node("Type", ty);
            }),
            TypeAlias::Trait { attrs, name, generics } => logger.log_ast_node("Trait Typealias", |logger| {
                logger.log_indented_node_ref_slice("Attributes", attrs);
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                logger.set_last_at_indent();
                logger.log_opt_node_ref(generics);
            }),
            TypeAlias::Opaque { attrs, vis, name, size } => logger.log_ast_node("Opaque Typealias", |logger| {
                logger.log_indented_node_ref_slice("Attributes", attrs);
                logger.log_opt_node_ref(vis);
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                logger.log_indented_opt_node("Size", size);
            }),
        }
    }
}

pub enum Struct {
    Regular {
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
        attrs:        Vec<AstNodeRef<Attribute>>,
        vis:          Option<AstNodeRef<Visibility>>,
        name:         NameId,
    }
}

impl AstNode for Struct {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            Struct::Regular { attrs, vis, is_mut, is_record, name, generics, where_clause, fields } => logger.log_ast_node("Struct", |logger| {
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
            Struct::Tuple { attrs, vis, is_mut, is_record, name, generics, where_clause, fields } => logger.log_ast_node("Tuple Struct", |logger| {
                logger.log_indented_node_ref_slice("Attributes", attrs);
                logger.log_opt_node_ref(vis);
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                logger.log_opt_node_ref(generics);
                logger.log_opt_node_ref(where_clause);
                logger.set_last_at_indent();
                logger.log_indented_slice("Fields", fields, |logger, field| field.log(logger));
            }),
            Struct::Unit { attrs, vis, name } => logger.log_ast_node("Unit Struct", |logger| {
                logger.log_indented_node_ref_slice("Attributes", attrs);
                logger.log_opt_node_ref(vis);
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
            }),
        }
    }
}

pub enum RegStructField {
    Field {
        attrs:  Vec<AstNodeRef<Attribute>>,
        vis:    Option<AstNodeRef<Visibility>>,
        is_mut: bool,
        names:  Vec<NameId>,
        ty:     Type,
        def:    Option<Expr>,
    },
    Use {
        attrs:  Vec<AstNodeRef<Attribute>>,
        vis:    Option<AstNodeRef<Visibility>>,
        is_mut: bool,
        path:   AstNodeRef<TypePath>,
    }
}

impl RegStructField {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            RegStructField::Field { attrs, vis, is_mut, names, ty, def } => logger.log_indented("Named Field", |logger| {
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
            RegStructField::Use { attrs, vis, is_mut, path } => logger.log_indented("Use Field", |logger| {
                logger.log_indented_node_ref_slice("Attributes", attrs);
                logger.log_opt_node_ref(vis);
                logger.prefixed_log_fmt(format_args!("Is Mut: {is_mut}\n"));
                logger.set_last_at_indent();
                logger.log_node_ref(*path);
            }),
        }
    }
}

pub struct TupleStructField {
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
    pub attrs:        Vec<AstNodeRef<Attribute>>,
    pub vis:          Option<AstNodeRef<Visibility>>,
    pub is_mut:       bool,
    pub name:         NameId,
    pub generics:     Option<AstNodeRef<GenericParams>>,
    pub where_clause: Option<AstNodeRef<WhereClause>>,
    pub fields:       Vec<UnionField>,
}

impl AstNode for Union {
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
        attrs:        Vec<AstNodeRef<Attribute>>,
        vis:          Option<AstNodeRef<Visibility>>,
        name:         NameId,
        variants:     Vec<FlagEnumVariant>,
    }
}

impl AstNode for Enum {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            Enum::Adt { attrs, vis, is_mut, is_record, name, generics, where_clause, variants } => logger.log_ast_node("Adt Enum", |logger| {
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
            Enum::Flag { attrs, vis, name, variants } => logger.log_ast_node("Flag Enum", |logger| {
                logger.log_indented_node_ref_slice("Attributes", attrs);
                logger.log_opt_node_ref(vis);
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                logger.set_last_at_indent();
                logger.log_indented_slice("Variants", variants, |logger, variant| variant.log(logger));
            }),
        }
    }
}

pub enum EnumVariant {
    Struct {
        attrs:        Vec<AstNodeRef<Attribute>>,
        is_mut:       bool,
        name:         NameId,
        fields:       Vec<RegStructField>,
        discriminant: Option<Expr>,
    },
    Tuple {
        attrs:        Vec<AstNodeRef<Attribute>>,
        is_mut:       bool,
        name:         NameId,
        fields:       Vec<TupleStructField>,
        discriminant: Option<Expr>,
    },
    Fieldless {
        attrs:        Vec<AstNodeRef<Attribute>>,
        name:         NameId,
        discriminant: Option<Expr>,
    }
}

impl EnumVariant {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            EnumVariant::Struct { attrs, is_mut, name, fields, discriminant } => logger.log_indented("Struct Variant", |logger| {
                logger.log_indented_node_ref_slice("Attributes", &attrs);
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));

                logger.set_last_at_indent_if(discriminant.is_none());
                logger.log_indented_slice("Fields", fields, |logger, field| field.log(logger));
                logger.set_last_at_indent();
                logger.log_indented_opt_node("Discriminant", discriminant);

            }),
            EnumVariant::Tuple { attrs, is_mut, name, fields, discriminant } => logger.log_indented("Tuple Variant", |logger| {
                logger.log_indented_node_ref_slice("Attributes", &attrs);
                
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));

                logger.set_last_at_indent_if(discriminant.is_none());
                logger.log_indented_slice("Fields", fields, |logger, field| field.log(logger));
                logger.set_last_at_indent();
                logger.log_indented_opt_node("Discriminant", discriminant);
            }),
            EnumVariant::Fieldless { attrs, name, discriminant } => logger.log_indented("Fieldless Variant", |logger| {
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

pub enum BitfieldField {
    Field {
        attrs:  Vec<AstNodeRef<Attribute>>,
        vis:    Option<AstNodeRef<Visibility>>,
        is_mut: bool,
        names:  Vec<NameId>,
        ty:     Type,
        bits:   Option<Expr>,
        def:    Option<Expr>,
    },
    Use {
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
            BitfieldField::Field { attrs, vis, is_mut, names, ty, bits, def } => logger.log_indented("Field", |logger| {
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
            BitfieldField::Use { attrs, vis, is_mut, path, bits } => logger.log_indented("Use Field", |logger| {
                logger.log_indented_node_ref_slice("Attributes", &attrs);
                logger.log_opt_node_ref(vis);
                logger.prefixed_log_fmt(format_args!("Is Mut: {is_mut}\n"));
                
                if bits.is_none() {
                    logger.set_last_at_indent();
                }
                logger.set_last_at_indent_if(bits.is_none());
                logger.log_indented_node_ref("Path", *path);

                logger.set_last_at_indent();
                logger.log_indented_opt_node("Bits", bits);
            }),
        }
    }
}


pub struct Const {
    pub attrs: Vec<AstNodeRef<Attribute>>,
    pub vis:   Option<AstNodeRef<Visibility>>,
    pub name:  NameId,
    pub ty:    Option<Type>,
    pub val:   Expr,
}

impl AstNode for Const {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Const Item", |logger| {
            logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(self.name)));
            logger.log_indented_opt_node("Type", &self.ty);
            logger.set_last_at_indent();
            logger.log_indented_node("Value", &self.val);
        });
    }
}

pub enum Static {
    Static {
        attrs:  Vec<AstNodeRef<Attribute>>,
        vis:    Option<AstNodeRef<Visibility>>,
        name:   NameId,
        ty:     Type,
        val:    Expr,
    },
    Tls {
        attrs:  Vec<AstNodeRef<Attribute>>,
        vis:    Option<AstNodeRef<Visibility>>,
        is_mut: bool,
        name:   NameId,
        ty:     Type,
        val:    Expr,
    },
    Extern {
        attrs:  Vec<AstNodeRef<Attribute>>,
        vis:    Option<AstNodeRef<Visibility>>,
        abi:    LiteralId,
        is_mut: bool,
        name:   NameId,
        ty:     Type,
    }
}

impl AstNode for Static {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            Static::Static { attrs, vis, name, ty, val } => logger.log_ast_node("Static", |logger| {
                logger.log_indented_node_ref_slice("Attributes", &attrs);
                logger.log_opt_node_ref(vis);
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                logger.log_indented_node("Type", ty);
                logger.set_last_at_indent();
                logger.log_indented_node("Val", val);
            }),
            Static::Tls { attrs, vis, is_mut, name, ty, val } => logger.log_ast_node("Tls Static", |logger| {
                logger.log_indented_node_ref_slice("Attributes", &attrs);
                logger.log_opt_node_ref(vis);
                logger.prefixed_log_fmt(format_args!("Is Mut: {is_mut}\n"));
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                logger.log_indented_node("Type", ty);
                logger.set_last_at_indent();
                logger.log_indented_node("Val", val);
            }),
            Static::Extern { attrs, vis, abi, is_mut, name, ty } => logger.log_ast_node("Extern Static", |logger| {
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

pub struct Property {
    attrs:     Vec<AstNodeRef<Attribute>>,
    vis:       Option<AstNodeRef<Visibility>>,
    is_unsafe: bool,
    name:      NameId,
    body:      PropertyBody,
}


pub enum PropertyBody {
    Assoc {
        get:       Option<Expr>,
        ref_get:   Option<Expr>,
        mut_get:   Option<Expr>,
        set:       Option<Expr>,
    },
    Trait {
        has_get:     bool,
        has_ref_get: bool,
        has_mut_get: bool,
        has_set:     bool,
    }
}

impl AstNode for Property {
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

impl Property {
    pub fn is_trait_property(&self) -> bool {
        matches!(self.body, PropertyBody::Trait{ .. })
    }

    pub fn has_get(&self) -> bool {
        match &self.body {
            PropertyBody::Assoc { get, .. } => get.is_some(),
            PropertyBody::Trait { has_get, .. } => *has_get,
        }
    }
    pub fn has_ref_get(&self) -> bool {
        match &self.body {
            PropertyBody::Assoc { ref_get, .. } => ref_get.is_some(),
            PropertyBody::Trait { has_ref_get, .. } => *has_ref_get,
        }
    }
    pub fn has_mut_get(&self) -> bool {
        match &self.body {
            PropertyBody::Assoc { mut_get, .. } => mut_get.is_some(),
            PropertyBody::Trait { has_mut_get, .. } => *has_mut_get,
        }
    }
    pub fn has_set(&self) -> bool {
        match &self.body {
            PropertyBody::Assoc { set, .. } => set.is_some(),
            PropertyBody::Trait { has_set, .. } => *has_set,
        }
    }
}

impl PropertyBody {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            PropertyBody::Assoc { get, ref_get, mut_get, set } => {
                logger.set_last_at_indent_if(ref_get.is_none() && mut_get.is_none() && set.is_none());
                logger.log_indented_opt_node("Get", get);
                logger.set_last_at_indent_if(mut_get.is_none() && set.is_none());
                logger.log_indented_opt_node("Ref Get", ref_get);
                logger.set_last_at_indent_if(set.is_none());
                logger.log_indented_opt_node("Mut Get", mut_get);
                logger.set_last_at_indent();
                logger.log_indented_opt_node("Set", set);
            },
            PropertyBody::Trait { has_get, has_ref_get, has_mut_get, has_set } => {
                logger.prefixed_log_fmt(format_args!("Has Get: {has_get}\n"));
                logger.prefixed_log_fmt(format_args!("Has Ref Get: {has_ref_get}\n"));
                logger.prefixed_log_fmt(format_args!("Has Mut Get: {has_mut_get}\n"));
                logger.prefixed_log_fmt(format_args!("Has Set: {has_set}\n"));
            },
        }
    }
}


pub struct Trait {
    pub attrs:       Vec<AstNodeRef<Attribute>>,
    pub vis:         Option<AstNodeRef<Visibility>>,
    pub is_unsafe:   bool,
    pub is_sealed:   bool,
    pub name:        NameId,
    pub bounds:      Option<AstNodeRef<TraitBounds>>,
    pub assoc_items: Vec<TraitItem>,
}

impl AstNode for Trait {
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

pub struct Impl {
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

pub struct ExternBlock {
    pub attrs: Vec<AstNodeRef<Attribute>>,
    pub vis:   Option<AstNodeRef<Visibility>>,
    pub abi:   LiteralId,
    pub items: Vec<ExternItem>,
}

impl AstNode for ExternBlock {
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

// TODO: might be moved to different file
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OpType {
    Prefix,
    Infix,
    Postfix,
    Assign,
}

impl fmt::Display for OpType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpType::Prefix    => write!(f, "prefix"),
            OpType::Infix     => write!(f, "infix"),
            OpType::Postfix   => write!(f, "postfix"),
            OpType::Assign    => write!(f, "assign"),
        }
    }
}

pub enum OpTrait {
    Base {
        attrs:      Vec<AstNodeRef<Attribute>>,
        vis:        Option<AstNodeRef<Visibility>>,
        name:       NameId,
        precedence: Option<AstNodeRef<SimplePath>>,
        elems:      Vec<OpElem>,
    },
    Extended {
        attrs:      Vec<AstNodeRef<Attribute>>,
        vis:        Option<AstNodeRef<Visibility>>,
        name:       NameId,
        bases:      Vec<AstNodeRef<SimplePath>>,
        elems:      Vec<OpElem>,
    }
}

impl AstNode for OpTrait {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            OpTrait::Base { attrs, vis, name, precedence, elems } => logger.log_ast_node("Operator Trait", |logger| {   
                logger.log_indented_node_ref_slice("Attributes", attrs);
                logger.log_opt_node_ref(vis);

                logger.set_last_at_indent_if(precedence.is_none() && elems.is_empty());
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                logger.set_last_at_indent_if(elems.is_empty());
                logger.log_indented_opt_node_ref("Precedence", precedence);
                logger.set_last_at_indent();
                logger.log_indented_slice("Elements", elems, |logger, elem| elem.log(logger));
            }),
            OpTrait::Extended { attrs, vis, name, bases, elems } => logger.log_ast_node("Operator Extension", |logger| {
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

pub enum OpElem {
    Def {
        op_type: OpType,
        op:      Punctuation,
        name:    NameId,
        ret:     Option<Type>,
        def:     Option<Expr>,
    },
    Extend {
        op_type: OpType,
        op:      Punctuation,
        def:     Expr,
    },
    Contract {
        expr:    AstNodeRef<BlockExpr>
    }
}

impl OpElem {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            OpElem::Def { op_type, op, name, ret, def } => logger.log_indented("Operator Definition", |logger| {
                logger.prefixed_log_fmt(format_args!("Operator Type: {op_type}\n"));
                logger.prefixed_log_fmt(format_args!("Operator: {}\n", logger.resolve_punctuation(*op)));
                logger.set_last_at_indent_if(def.is_none());
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                logger.set_last_at_indent_if(def.is_none());
                logger.log_indented_opt_node("Return Type", ret);
                logger.set_last_at_indent();
                logger.log_indented_opt_node("Default Implementation", def);
            }),
            OpElem::Extend { op_type, op, def } => logger.log_indented("Operator Specialization", |logger| {
                logger.prefixed_log_fmt(format_args!("Operator Type: {op_type}\n"));
                logger.prefixed_log_fmt(format_args!("Operator: {}\n", logger.resolve_punctuation(*op)));
                logger.set_last_at_indent();
                logger.log_indented_node("Default Implementation", def);
            }),
            OpElem::Contract { expr } => logger.log_indented_node_ref("Contract", *expr),
        }
    }
}

// TODO: May be moved into separate fill
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PrecedenceAssociativity {
    None,
    Left,
    Right,
}

impl fmt::Display for PrecedenceAssociativity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrecedenceAssociativity::None  => write!(f, "none"),
            PrecedenceAssociativity::Left  => write!(f, "left"),
            PrecedenceAssociativity::Right => write!(f, "right"),
        }
    }
}

pub struct Precedence {
    pub attrs:         Vec<AstNodeRef<Attribute>>,
    pub vis:           Option<AstNodeRef<Visibility>>,
    pub name:          NameId,
    pub higher_than:   Option<NameId>,
    pub lower_than:    Option<NameId>,
    pub associativity: Option<PrecedenceAssociativity>
}

impl AstNode for Precedence {
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

// =============================================================================================================================

pub enum Stmt {
    Empty,
    Item(Item),
    VarDecl(AstNodeRef<VarDecl>),
    Defer(AstNodeRef<Defer>),
    ErrDefer(AstNodeRef<ErrDefer>),
    Expr(AstNodeRef<ExprStmt>),
}

impl AstNode for Stmt {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            Self::Empty             => {
                logger.prefixed_logln("Emtpy");
            },
            Self::Item(item)         => item.log(logger),
            Self::VarDecl(var_decl)  => logger.log_node_ref(*var_decl),
            Self::Defer(defer)       => logger.log_node_ref(*defer),
            Self::ErrDefer(errdefer) => logger.log_node_ref(*errdefer),
            Self::Expr(expr)         => logger.log_node_ref(*expr),
        }
    }
}

pub enum VarDecl {
    Named {
        attrs: Vec<AstNodeRef<Attribute>>,
        names: Vec<(bool, NameId)>,
        expr:  Expr,
    },
    Let {
        attrs:      Vec<AstNodeRef<Attribute>>,
        pattern:    Pattern,
        ty:         Option<Type>,
        expr:       Option<Expr>,
        else_block: Option<AstNodeRef<BlockExpr>>,
    }
}

impl AstNode for VarDecl {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            VarDecl::Named { attrs, names, expr } => logger.log_ast_node("Named Variable Declaration", |logger| {
                logger.log_indented_node_ref_slice("Attributes", attrs);
                logger.log_indented_slice("Names", names, |logger, (is_mut, name)| {
                    logger.log_indented("Name", |logger| {
                        logger.prefixed_log_fmt(format_args!("Is Mut: {is_mut}\n"));
                        logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                    });
                });

                logger.set_last_at_indent();
                logger.log_indented_node("Value", expr);
            }),
            VarDecl::Let { attrs, pattern, ty, expr, else_block } => logger.log_ast_node("Let Variable Declaration", |logger| {
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

pub struct Defer {
    pub attrs: Vec<AstNodeRef<Attribute>>,
    pub expr:  Expr,
}

impl AstNode for Defer {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Defer Statement", |logger| {
            logger.log_indented_node_ref_slice("Attributes", &self.attrs);
            logger.set_last_at_indent();
            logger.log_indented_node("Expr", &self.expr); 
        });
    }
}

pub struct ErrDefer {
    pub attrs:    Vec<AstNodeRef<Attribute>>,
    pub receiver: Option<ErrDeferReceiver>,
    pub expr:     Expr,

}

impl AstNode for ErrDefer {
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

pub struct ErrDeferReceiver {
    pub is_mut: bool,
    pub name:   NameId,
}

pub struct ExprStmt {
    attrs:    Vec<AstNodeRef<Attribute>>,
    expr:     Expr,
    has_semi: bool,
}
impl AstNode for ExprStmt {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Expression Statement", |logger| {
            logger.log_indented_node_ref_slice("Attributes", &self.attrs);
            logger.set_last_at_indent();
            logger.log_node(&self.expr);
        })
    }
}

// =============================================================================================================================


pub enum Expr {
    Literal(AstNodeRef<LiteralExpr>),
    // Can include a sequence of field accesses
    Path(AstNodeRef<PathExpr>),
    Unit,
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
    FullRange,
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
    Underscore,
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
    fn log(&self, logger: &mut AstLogger) {
        match self {
            Self::Literal(lit)      => logger.log_node_ref(*lit),
            Self::Path(path)        => logger.log_node_ref(*path),
            Self::Unit              => logger.prefixed_logln("Unit Expression"),
            Self::Block(block)      => logger.log_node_ref(*block),
            Self::Prefix(expr)      => logger.log_node_ref(*expr),
            Self::Postfix(expr)     => logger.log_node_ref(*expr),
            Self::Infix(expr)       => logger.log_node_ref(*expr),
            Self::Paren(expr)       => logger.log_node_ref(*expr),
            Self::Inplace(expr)     => logger.log_node_ref(*expr),
            Self::TypeCast(expr)    => logger.log_node_ref(*expr),
            Self::TypeCheck(expr)   => logger.log_node_ref(*expr),
            Self::Tuple(expr)       => logger.log_node_ref(*expr),
            Self::Array(expr)       => logger.log_node_ref(*expr),
            Self::Struct(expr)      => logger.log_node_ref(*expr),
            Self::Index(expr)       => logger.log_node_ref(*expr),
            Self::TupleIndex(expr)  => logger.log_node_ref(*expr),
            Self::FnCall(expr)      => logger.log_node_ref(*expr),
            Self::Method(expr)      => logger.log_node_ref(*expr),
            Self::FieldAccess(expr) => logger.log_node_ref(*expr),
            Self::Closure(expr)     => logger.log_node_ref(*expr),
            Self::FullRange         => logger.prefixed_logln("Full Range Expression"),
            Self::If(expr)          => logger.log_node_ref(*expr),
            Self::Let(expr)         => logger.log_node_ref(*expr),
            Self::Loop(expr)        => logger.log_node_ref(*expr),
            Self::While(expr)       => logger.log_node_ref(*expr),
            Self::DoWhile(expr)     => logger.log_node_ref(*expr),
            Self::For(expr)         => logger.log_node_ref(*expr),
            Self::Match(expr)       => logger.log_node_ref(*expr),
            Self::Break(expr)       => logger.log_node_ref(*expr),
            Self::Continue(expr)    => logger.log_node_ref(*expr),
            Self::Fallthrough(expr) => logger.log_node_ref(*expr),
            Self::Return(expr)      => logger.log_node_ref(*expr),
            Self::Underscore        => logger.prefixed_logln("Underscore Expression"),
            Self::Throw(expr)       => logger.log_node_ref(*expr),
            Self::Comma(expr)       => logger.log_node_ref(*expr),
            Self::When(expr)        => logger.log_node_ref(*expr),
        }
    }
}


pub enum LiteralValue {
    Lit(LiteralId),
    Bool(bool),
}

pub struct LiteralExpr {
    pub literal: LiteralValue,
    pub lit_op:  Option<LiteralOp>
}

impl AstNode for LiteralExpr {
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

pub enum PathExpr {
    Named {
        iden: Identifier,
    },
    Inferred {
        iden: Identifier,
    },
    SelfPath,
}

impl AstNode for PathExpr {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            PathExpr::Named { iden } => logger.log_ast_node("Path Expr", |logger| {
                logger.set_last_at_indent();
                iden.log(logger);
            }),
            PathExpr::Inferred { iden } => logger.log_ast_node("Inferred Path Expr", |logger| {
                logger.set_last_at_indent();
                iden.log(logger);
            }),
            PathExpr::SelfPath => {
                logger.set_last_at_indent();
                logger.prefixed_logln("Self Path Expr");
            },
        }
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
    pub kind:  BlockExprKind,
    pub block: AstNodeRef<Block>
}

impl AstNode for BlockExpr {
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
            logger.log_node_ref(self.block);
        });
    }
}

pub struct PrefixExpr {
    pub op:   Punctuation,
    pub expr: Expr,
}

impl AstNode for PrefixExpr {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Prefix expression", |logger| {
            logger.prefixed_log_fmt(format_args!("Op: {}\n", logger.resolve_punctuation(self.op)));
            logger.set_last_at_indent();
            logger.log_node(&self.expr);
        });
    }
}

pub struct PostfixExpr {
    pub op:   Punctuation,
    pub expr: Expr,
}

impl AstNode for PostfixExpr {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Postfix expression", |logger| {
            logger.prefixed_log_fmt(format_args!("Op: {}\n", logger.resolve_punctuation(self.op)));
            logger.set_last_at_indent();
            logger.log_node(&self.expr);
        });
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InfixOp {
    Punct(Punctuation),
    Contains,
    NotContains,
}

impl InfixOp {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            InfixOp::Punct(punct) => logger.prefixed_log_fmt(format_args!("Op: {}\n", logger.resolve_punctuation(*punct))),
            InfixOp::Contains     => logger.prefixed_logln("Op: in"),
            InfixOp::NotContains  => logger.prefixed_logln("Op: !in"),
        }
    }
}

pub struct InfixExpr {
    pub op:    InfixOp,
    pub left:  Expr,
    pub right: Expr,
}

impl AstNode for InfixExpr {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Infix expression", |logger| {
            self.op.log(logger);
            logger.log_node(&self.left);
            logger.set_last_at_indent();
            logger.log_node(&self.right);
        });
    }
}

pub struct ParenExpr {
    pub expr: Expr,
}

impl AstNode for ParenExpr {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Parenthesized Expression", |logger| {
            logger.set_last_at_indent();
            logger.log_node(&self.expr);
        });
    }
}

pub struct InplaceExpr {
    pub left:  Expr,
    pub right: Expr,
}

impl AstNode for InplaceExpr {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Inplace Expession", |logger| {
            logger.log_node(&self.left);
            logger.set_last_at_indent();
            logger.log_node(&self.right);
        });
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TypeCastKind {
    Normal,
    Try,
    Unwrap,
}

pub struct TypeCastExpr {
    pub kind: TypeCastKind,
    pub expr: Expr,
    pub ty:   Type,
}

impl AstNode for TypeCastExpr {
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

pub struct TypeCheckExpr {
    pub negate: bool,
    pub expr:   Expr,
    pub ty:     Type,
}

impl AstNode for TypeCheckExpr {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Type Check Expression", |logger| {
            logger.prefixed_log_fmt(format_args!("Negate: {}\n", self.negate));
            logger.log_node(&self.expr);
            logger.set_last_at_indent();
            logger.log_node(&self.ty);
        })
    }
}

pub struct TupleExpr {
    pub exprs: Vec<Expr>
}

impl AstNode for TupleExpr {
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

pub struct ArrayExpr {
    pub exprs: Vec<Expr>
}

impl AstNode for ArrayExpr {
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

pub enum StructArg {
    Expr(NameId, Expr),
    Name(NameId),
    Complete(Expr),
}

impl StructArg {
    pub fn log(&self, logger: &mut AstLogger) {
        match self {
            StructArg::Expr(name, expr) => logger.log_indented("Named Argument", |logger| {
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                logger.set_last_at_indent();
                logger.log_node(expr);
            }),
            StructArg::Name(name)     => logger.prefixed_log_fmt(format_args!("Name-only: {}\n", logger.resolve_name(*name))),
            StructArg::Complete(path) => logger.log_indented_node("Struct Completion", path),
        }
    }
}

pub struct StructExpr {
    pub path: Expr,
    pub args: Vec<StructArg>,
}

impl AstNode for StructExpr {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Struct Expression", |logger| {
            logger.set_last_at_indent_if(self.args.is_empty());
            logger.log_indented_node("Path", &self.path);
            logger.set_last_at_indent();
            logger.log_indented_slice("Arguments", &self.args, |logger, arg| arg.log(logger));
        });
    }
}

pub struct IndexExpr {
    pub is_opt: bool,
    pub expr:   Expr,
    pub index:  Expr,
}

impl AstNode for IndexExpr {
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

pub struct TupleIndexExpr {
    pub expr:  Expr,
    pub index: LiteralId
}

impl AstNode for TupleIndexExpr {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Tuple Index Expression", |logger| {
            logger.prefixed_log_fmt(format_args!("Index: {}\n", logger.resolve_literal(self.index)));
            logger.set_last_at_indent();
            logger.log_node(&self.expr);
        });
    }
}

pub enum FnArg {
    Expr(Expr),
    Labeled{
        label: NameId,
        expr:  Expr,
    }
}

impl FnArg {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            FnArg::Expr(expr) => logger.log_indented_node("Argument", expr),
            FnArg::Labeled { label, expr } => logger.log_indented("Labeled Argument", |logger| {
                logger.prefixed_log_fmt(format_args!("Label: {}\n", logger.resolve_name(*label)));
                logger.set_last_at_indent();
                logger.log_indented_node("Expression", expr)
            }),
        }
    }
}

pub enum FnCallExpr {
    Expr {
        expr: Expr,
        args: Vec<FnArg>,
    },
    Qual {
        path: AstNodeRef<QualifiedPath>,
        args: Vec<FnArg>,
    },
}

impl AstNode for FnCallExpr {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            Self::Expr { expr, args } => logger.log_ast_node("Expression Function Call", |logger| {
                logger.set_last_at_indent_if(args.is_empty());
                logger.log_indented_node("Function", expr);
                logger.set_last_at_indent();
                logger.log_indented_slice("Arguments", args, |logger, arg| arg.log(logger));
            }),
            Self::Qual { path, args } => logger.log_ast_node("Qualified Function Call", |logger| {
                logger.set_last_at_indent_if(args.is_empty());
                logger.log_indented_node_ref("Qualified path", *path);
                logger.set_last_at_indent();
                logger.log_indented_slice("Arguments", args, |logger, arg| arg.log(logger));
            }),
        }
    }
}

pub struct MethodCallExpr {
    pub receiver:       Expr,
    pub method:         NameId,
    pub gen_args:       Option<AstNodeRef<GenericArgs>>,
    pub args:           Vec<FnArg>,
    pub is_propagating: bool,
}

impl AstNode for MethodCallExpr {
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

pub struct FieldAccessExpr {
    pub expr:           Expr,
    pub field:          NameId,
    pub gen_args:       Option<AstNodeRef<GenericArgs>>,
    pub is_propagating: bool
}

impl AstNode for FieldAccessExpr {
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

pub struct ClosureExpr {
    pub is_moved: bool,
    pub params:   Vec<FnParam>,
    pub ret:      Option<FnReturn>,
    pub body:     Expr,
}

impl AstNode for ClosureExpr {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Closure Expression", |logger| {

        });
    }
}

pub struct LetBindingExpr {
    pub pattern:   Pattern,
    pub scrutinee: Expr,
}

impl AstNode for LetBindingExpr {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Let Binding Expression", |logger| {
            logger.log_node(&self.pattern);
            logger.set_last_at_indent();
            logger.log_node(&self.scrutinee);
        });
    }
}

pub struct IfExpr {
    pub cond:      Expr,
    pub body:      AstNodeRef<BlockExpr>,
    pub else_body: Option<Expr>,
}

impl AstNode for IfExpr {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("If Expression", |logger| {
            logger.log_indented_node("Condition", &self.cond);
            logger.set_last_at_indent_if(self.else_body.is_none());
            logger.log_indented_node_ref("Body", self.body);
            logger.set_last_at_indent();
            logger.log_indented_opt_node("Else Body", &self.else_body);
        });
    }
}

pub struct LoopExpr {
    pub label: Option<NameId>,
    pub body:  AstNodeRef<BlockExpr>,
}

impl AstNode for LoopExpr {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Loop expression", |logger| {
            if let Some(label) = &self.label {
                logger.prefixed_log_fmt(format_args!("Label: {}\n", logger.resolve_name(*label)));
            }
            logger.set_last_at_indent();
            logger.log_node_ref(self.body);
        });
    }
}

pub struct WhileExpr {
    pub label:     Option<NameId>,
    pub cond:      Expr,
    pub inc:       Option<Expr>,
    pub body:      AstNodeRef<BlockExpr>,
    pub else_body: Option<AstNodeRef<BlockExpr>>,
}

impl AstNode for WhileExpr {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("While Expression", |logger| {
            if let Some(label) = &self.label {
                logger.prefixed_log_fmt(format_args!("Label: {}\n", logger.resolve_name(*label)));
            }
            logger.log_indented_node("Condition", &self.cond);
            logger.log_indented_opt_node("Increment", &self.inc);
            logger.set_last_at_indent_if(self.else_body.is_none());
            logger.log_indented_node_ref("Body", self.body);
            logger.set_last_at_indent();
            logger.log_indented_opt_node_ref("Else Body", &self.else_body);
        });
    }
}

pub struct DoWhileExpr {
    pub label: Option<NameId>,
    pub body:  AstNodeRef<BlockExpr>,
    pub cond:  Expr,
}

impl AstNode for DoWhileExpr {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Do While Expression", |logger| {
            if let Some(label) = &self.label {
                logger.prefixed_log_fmt(format_args!("Label: {}\n", logger.resolve_name(*label)));
            }
            logger.log_indented_node_ref("Body", self.body);
            logger.set_last_at_indent();
            logger.log_indented_node("Condition", &self.cond);
        });
    }
}

pub struct ForExpr {
    pub label:     Option<NameId>,
    pub pattern:   Pattern,
    pub src:       Expr,
    pub body:      AstNodeRef<BlockExpr>,
    pub else_body: Option<AstNodeRef<BlockExpr>>,
}

impl AstNode for ForExpr {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("For Expression", |logger| {
            if let Some(label) = &self.label {
                logger.prefixed_log_fmt(format_args!("Label: {}\n", logger.resolve_name(*label)));
            }
            logger.log_indented_node("Pattern", &self.pattern);
            logger.log_indented_node("Src", &self.src);
            logger.set_last_at_indent_if(self.else_body.is_some());
            logger.log_indented_node_ref("Body", self.body);
            logger.set_last_at_indent();
            logger.log_indented_opt_node_ref("Else Body", &self.else_body);
        });
    }
}

pub struct MatchExpr {
    pub label:     Option<NameId>,
    pub scrutinee: Expr,
    pub branches:  Vec<MatchBranch>,
}

impl AstNode for MatchExpr {
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

pub struct MatchBranch {
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
    pub label: Option<NameId>,
    pub value: Option<Expr>,
}

impl AstNode for BreakExpr {
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

pub struct ContinueExpr {
    pub label: Option<NameId>
}

impl AstNode for ContinueExpr {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Continue Expr", |logger| {
            if let Some(label) = &self.label {
                logger.prefixed_log_fmt(format_args!("Label: {}\n", logger.resolve_name(*label)));
            }
        });
    }
}

pub struct FallthroughExpr {
    pub label: Option<NameId>
}

impl AstNode for FallthroughExpr {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Fallthrough Expr", |logger| {
            if let Some(label) = &self.label {
                logger.prefixed_log_fmt(format_args!("Label: {}\n", logger.resolve_name(*label)));
            }
        });
    }
}

pub struct ReturnExpr {
    pub value: Option<Expr>,
}

impl AstNode for ReturnExpr {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Continue Expr", |logger| {
            logger.set_last_at_indent();
            logger.log_opt_node(&self.value);
        });
    }
}

pub struct ThrowExpr {
    pub expr: Expr,
}

impl AstNode for ThrowExpr {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Throw Expression", |logger| {
            logger.log_node(&self.expr);
        });
    }
}

pub struct CommaExpr {
    pub exprs: Vec<Expr>,
}

impl AstNode for CommaExpr {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Comma expression", |logger| {
            logger.log_node_slice(&self.exprs);
        })
    }
}

pub struct WhenExpr {
    pub cond:      Expr,
    pub body:      AstNodeRef<BlockExpr>,
    pub else_body: Option<Expr>,
}

impl AstNode for WhenExpr {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("When Expression", |logger| {
            logger.log_indented_node("Condition", &self.cond);
            logger.set_last_at_indent_if(self.else_body.is_none());
            logger.log_indented_node_ref("Body", self.body);
            logger.set_last_at_indent();
            logger.log_indented_opt_node("Else Body", &self.else_body);
        });
    }
}

// =============================================================================================================================

pub enum Pattern {
    Literal(AstNodeRef<LiteralPattern>),
    Identifier(AstNodeRef<IdentifierPattern>),
    Path(AstNodeRef<PathPattern>),
    Wildcard,
    Rest,
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
    fn log(&self, logger: &mut AstLogger) {
        match self {
            Pattern::Literal(pattern)     => logger.log_node_ref(*pattern),
            Pattern::Identifier(pattern)  => logger.log_node_ref(*pattern),
            Pattern::Path(pattern)        => logger.log_node_ref(*pattern),
            Pattern::Wildcard             => logger.prefixed_logln("Wildcard Pattern"),
            Pattern::Rest                 => logger.prefixed_logln("Rest Pattern"),
            Pattern::Range(pattern)        => logger.log_node_ref(*pattern),
            Pattern::Reference(pattern)    => logger.log_node_ref(*pattern),
            Pattern::Tuple(pattern)        => logger.log_node_ref(*pattern),
            Pattern::Struct(pattern)       => logger.log_node_ref(*pattern),
            Pattern::TupleStruct(pattern)  => logger.log_node_ref(*pattern),
            Pattern::Grouped(pattern)      => logger.log_node_ref(*pattern),
            Pattern::Slice(pattern)        => logger.log_node_ref(*pattern),
            Pattern::EnumMember(pattern)   => logger.log_node_ref(*pattern),
            Pattern::Alternative(pattern)  => logger.log_node_ref(*pattern),
            Pattern::TypeCheck(pattern)    => logger.log_node_ref(*pattern),
        }
    }
}

pub struct LiteralPattern {
    pub literal: LiteralValue,
    pub lit_op:  Option<LiteralOp>,
}

impl AstNode for LiteralPattern {
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

pub struct IdentifierPattern {
    pub is_ref: bool,
    pub is_mut: bool,
    pub name:   NameId,
    pub bound:  Option<Pattern>,
}

impl AstNode for IdentifierPattern {
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

pub struct PathPattern {
    pub path: AstNodeRef<ExprPath>,
}

impl AstNode for PathPattern {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Path Pattern", |logger| {
            logger.set_last_at_indent();
            logger.log_node_ref(self.path);
        });
    }
}

pub enum RangePattern {
    Exclusive{ begin: Pattern, end: Pattern },
    Inclusive{ begin: Pattern, end: Pattern },
    From { begin: Pattern },
    To { end: Pattern },
    InclusiveTo { end: Pattern }
}

impl AstNode for RangePattern {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            RangePattern::Exclusive { begin, end } => logger.log_indented("Exclusive Range Pattern", |logger| {
                logger.log_indented_node("Begin", begin);
                logger.set_last_at_indent();
                logger.log_indented_node("End", end);
            }),
            RangePattern::Inclusive { begin, end } => logger.log_indented("Inclusive Range Pattern", |logger| {
                logger.log_indented_node("Begin", begin);
                logger.set_last_at_indent();
                logger.log_indented_node("End", end);
            }),
            RangePattern::From { begin } => logger.log_indented("From Range Pattern", |logger| {
                logger.set_last_at_indent();
                logger.log_indented_node("Begin", begin);
            }),
            RangePattern::To { end } => logger.log_indented("To Range Pattern", |logger| {
                logger.set_last_at_indent();
                logger.log_indented_node("End", end);
            }),
            RangePattern::InclusiveTo { end } => logger.log_indented("Inclusive To Range Pattern", |logger| {
                logger.set_last_at_indent();
                logger.log_indented_node("End", end);
            }),
        }
    }
}

pub struct ReferencePattern {
    pub is_mut:  bool,
    pub pattern: Pattern,
}

impl AstNode for ReferencePattern {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Reference Pattern", |logger| {
            logger.prefixed_log_fmt(format_args!("Is Mut: {}\n", self.is_mut));
            logger.set_last_at_indent();
            logger.log_node(&self.pattern);
        })
    }
}

pub struct StructPattern {
    pub fields: Vec<StructPatternField>,
}

impl AstNode for StructPattern {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Struct Pattern", |logger| {
            logger.set_last_at_indent();
            logger.log_indented_slice("Fields", &self.fields, |logger, field| field.log(logger));
        });
    }
}

pub enum StructPatternField {
    Named {
        name:    NameId,
        pattern: Pattern,
    },
    TupleIndex {
        idx:     LiteralId,
        pattern: Pattern
    },
    Iden {
        is_ref: bool,
        is_mut: bool,
        iden:   NameId,
    },
    Rest,
}

impl StructPatternField {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            StructPatternField::Named { name, pattern } => logger.log_indented("Named Struct Field", |logger| {
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*name)));
                logger.set_last_at_indent();
                logger.log_node(pattern);
            }),
            StructPatternField::TupleIndex { idx, pattern } => logger.log_indented("Tuple Index Struct Field", |logger| {
                logger.prefixed_log_fmt(format_args!("Index: {}\n", logger.resolve_literal(*idx)));
                logger.set_last_at_indent();
                logger.log_node(pattern);
            }),
            StructPatternField::Iden { is_ref, is_mut, iden } => logger.log_indented("Iden Struct Field", |logger| {
                logger.prefixed_log_fmt(format_args!("Is Ref: {}\n", is_ref));
                logger.prefixed_log_fmt(format_args!("Is Mut: {}\n", is_mut));
                logger.set_last_at_indent();
                logger.prefixed_log_fmt(format_args!("Name: {}\n", logger.resolve_name(*iden)));
            }),
            StructPatternField::Rest => {
                logger.prefixed_logln("Rest Struct Field");
            },
        }
    }
}

pub struct TupleStructPattern {
    pub patterns: Vec<Pattern>,
}

impl AstNode for TupleStructPattern {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Tuple Struct Pattern", |logger| {
            logger.set_last_at_indent();
            logger.log_indented_node_slice("Patterns", &self.patterns);
        });
    }
}

pub struct TuplePattern {
    pub patterns: Vec<Pattern>
}

impl AstNode for TuplePattern {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Tuple Pattern", |logger| {
            logger.set_last_at_indent();
            logger.log_indented_node_slice("Patterns", &self.patterns);
        });
    }
}

pub struct GroupedPattern {
    pub pattern: Pattern
}

impl AstNode for GroupedPattern {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Grouped Pattern", |logger| {
            logger.set_last_at_indent();
            logger.log_node(&self.pattern);
        })
    }
}

pub struct SlicePattern {
    pub patterns: Vec<Pattern>
}

impl AstNode for SlicePattern {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Slice Pattern", |logger| {
            logger.set_last_at_indent();
            logger.log_indented_node_slice("Patterns", &self.patterns);
        })
    }
}

pub struct EnumMemberPattern {
    pub name: NameId,
}

impl AstNode for EnumMemberPattern {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Enum Member Pattern", |logger| {
            logger.prefixed_log_fmt(format_args!("Enum Member: {}\n", logger.resolve_name(self.name)));
        });
    }
}

pub struct AlternativePattern {
    pub patterns: Vec<Pattern>,
}

impl AstNode for AlternativePattern {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Alternative Pattern", |logger| {
            logger.set_last_at_indent();
            logger.log_indented_node_slice("Patterns", &self.patterns);
        });
    }
}

pub struct TypeCheckPattern {
    pub ty: Type
}

impl AstNode for TypeCheckPattern {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Type Check Pattern", |logger| {
            logger.set_last_at_indent();
            logger.log_node(&self.ty);
        })
    }
}

// =============================================================================================================================

pub enum Type {
    Paren(AstNodeRef<ParenthesizedType>),
    Primitive(AstNodeRef<PrimitiveType>),
    Unit,
    Never,
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
    fn log(&self, logger: &mut AstLogger) {
        match self {
            Type::Paren(ty)       => logger.log_node_ref(*ty),
            Type::Primitive(ty)   => logger.log_node_ref(*ty),
            Type::Never           => logger.prefixed_logln("Never Type"),
            Type::Unit            => logger.prefixed_logln("Unit Type"),
            Type::Path(ty)        => logger.log_node_ref(*ty),
            Type::Tuple(ty)       => logger.log_node_ref(*ty),
            Type::Array(ty)       => logger.log_node_ref(*ty),
            Type::Slice(ty)       => logger.log_node_ref(*ty),
            Type::StringSlice(ty) => logger.log_node_ref(*ty),
            Type::Pointer(ty)     => logger.log_node_ref(*ty),
            Type::Ref(ty)         => logger.log_node_ref(*ty),
            Type::Optional(ty)    => logger.log_node_ref(*ty),
            Type::Fn(ty)          => logger.log_node_ref(*ty),
            Type::Record(ty)      => logger.log_node_ref(*ty),
            Type::EnumRecord(ty)  => logger.log_node_ref(*ty),
        }
    }
}

pub struct ParenthesizedType {
    pub ty: Type,
}

impl AstNode for ParenthesizedType {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Parenthesized Type", |logger| {
            logger.set_last_at_indent();
            logger.log_node(&self.ty)
        })
    }
}

pub enum PrimitiveType {
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    F16,
    F32,
    F64,
    F128,
    Bool,
    B8,
    B16,
    B32,
    B64,
    Char,
    Char7,
    Char8,
    Char16,
    Char32,
}

impl AstNode for PrimitiveType {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Primitive Type", |logger| {
            logger.set_last_at_indent();
            logger.prefixed_log("Primitive: ");
            match self {
                PrimitiveType::U8     => logger.logln("u8"),
                PrimitiveType::U16    => logger.logln("u16"),
                PrimitiveType::U32    => logger.logln("u32"),
                PrimitiveType::U64    => logger.logln("u64"),
                PrimitiveType::U128   => logger.logln("u128"),
                PrimitiveType::Usize  => logger.logln("usize"),
                PrimitiveType::I8     => logger.logln("i8"),
                PrimitiveType::I16    => logger.logln("i16"),
                PrimitiveType::I32    => logger.logln("i32"),
                PrimitiveType::I64    => logger.logln("i64"),
                PrimitiveType::I128   => logger.logln("i128"),
                PrimitiveType::Isize  => logger.logln("isize"),
                PrimitiveType::F16    => logger.logln("f16"),
                PrimitiveType::F32    => logger.logln("f32"),
                PrimitiveType::F64    => logger.logln("f64"),
                PrimitiveType::F128   => logger.logln("f128"),
                PrimitiveType::Bool   => logger.logln("bool"),
                PrimitiveType::B8     => logger.logln("b8"),
                PrimitiveType::B16    => logger.logln("b16"),
                PrimitiveType::B32    => logger.logln("b32"),
                PrimitiveType::B64    => logger.logln("b64"),
                PrimitiveType::Char   => logger.logln("char"),
                PrimitiveType::Char7  => logger.logln("char7"),
                PrimitiveType::Char8  => logger.logln("char8"),
                PrimitiveType::Char16 => logger.logln("char16"),
                PrimitiveType::Char32 => logger.logln("char32"),
            }
        });
    }
}

pub struct PathType {
    pub path: AstNodeRef<TypePath>,
}

impl AstNode for PathType {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Path Type", |logger| {
            logger.log_node_ref(self.path);
        });
    }
}

pub struct TupleType {
    pub types: Vec<Type>,
}

impl AstNode for TupleType {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Tuple Type", |logger| {
            logger.log_indented_node_slice("Types", &self.types);
        });
    }
}

pub struct ArrayType {
    pub size:     Expr,
    pub sentinel: Option<Expr>,
    pub ty:       Type,
}

impl AstNode for ArrayType {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Array Type", |logger| {
            logger.log_indented_node("Size", &self.size);
            logger.log_indented_opt_node("Sentinel", &self.sentinel);
            logger.set_last_at_indent();
            logger.log_node(&self.ty);
        });
    }
}

pub struct SliceType {
    pub sentinel: Option<Expr>,
    pub ty:       Type,
}

impl AstNode for SliceType {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Slice Type", |logger| {
            logger.log_indented_opt_node("Sentinel", &self.sentinel);
            logger.set_last_at_indent();
            logger.log_node(&self.ty);
        });
    }
}

pub enum StringSliceType {
    Str,
    Str7,
    Str8,
    Str16,
    Str32,
    CStr,
}

impl AstNode for StringSliceType {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("String Slice Type", |logger| {
            logger.prefixed_log("StringSlice: ");
            match self {
                StringSliceType::Str   => logger.logln("str"),
                StringSliceType::Str7  => logger.logln("str7"),
                StringSliceType::Str8  => logger.logln("str8"),
                StringSliceType::Str16 => logger.logln("str16"),
                StringSliceType::Str32 => logger.logln("str32"),
                StringSliceType::CStr  => logger.logln("cstr"),
            }
        });
    }
}

pub struct PointerType {
    pub is_multi: bool,
    pub is_mut:   bool,
    pub ty:       Type,
    pub sentinel: Option<Expr>,
}

impl AstNode for PointerType {
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

pub struct ReferenceType {
    pub is_mut: bool,
    pub ty:     Type,
}

impl AstNode for ReferenceType {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Reference Type", |logger| {
            logger.prefixed_log_fmt(format_args!("Is Mut: {}\n", self.is_mut));
            
            logger.set_last_at_indent();
            logger.log_indented_node("Type", &self.ty);
        });
    }
}

pub struct OptionalType {
    pub ty: Type,
}

impl AstNode for OptionalType {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Optional Type", |logger| {
            logger.set_last_at_indent();
            logger.log_node(&self.ty);
        });
    }
}

pub struct FnType {
    pub is_unsafe: bool,
    pub abi:       Option<LiteralId>,
    pub params:    Vec<(Vec<NameId>, Type)>,
    pub return_ty: Option<Type>,
}

impl AstNode for FnType {
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

pub struct RecordType {
    pub fields: Vec<RegStructField>
}

impl AstNode for RecordType {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Record type", |logger| {
            logger.set_last_at_indent();
            logger.log_indented_slice("Fields", &self.fields, |logger, field| field.log(logger));
        });
    }
}

pub struct EnumRecordType {
    pub variants: Vec<EnumVariant>,
}

impl AstNode for EnumRecordType {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Enum Record Type", |logger| {
            logger.set_last_at_indent();
            logger.log_indented_slice("Variants", &self.variants, |logger, variant| variant.log(logger));
        });
    }
}

// =============================================================================================================================

pub struct GenericParams {

}
impl AstNode for GenericParams {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_indented("Generic Params", |logger| {

        });
    }
}

pub struct GenericArgs {

}
impl AstNode for GenericArgs {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_indented("Generic Args", |logger| {

        });
    }
}

pub struct WhereClause {

}
impl AstNode for WhereClause {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_indented("Where Clause", |logger| {

        });
    }
}

pub struct TraitBounds {
    
}
impl AstNode for TraitBounds {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_indented("Trait Bounds", |logger| {

        });  
    }
}

// =============================================================================================================================

pub enum Visibility {
    Pub,
    Super,
    Lib,
    Package,
    Path(AstNodeRef<SimplePath>)
}

impl AstNode for Visibility {
    fn log(&self, logger: &mut AstLogger) {
        logger.prefixed_log("Visibility: ");
        match self {
            Visibility::Pub        => logger.logln("pub"),
            Visibility::Super      => logger.logln("pub(super)"),
            Visibility::Lib        => logger.logln("pub(lib)"),
            Visibility::Package    => logger.logln("pub(package)"),
            Visibility::Path(path) => logger.log_indented_node_ref("pub(..)", *path),
        }
    }
}

// =============================================================================================================================

pub struct Attribute {
    pub is_mod: bool,
    pub metas:  Vec<AttribMeta>,
}

impl AstNode for Attribute {
    fn log(&self, logger: &mut AstLogger) {
        logger.log_ast_node("Attribute", |logger| {
            logger.prefixed_log_fmt(format_args!("Is Module Attribute: {}\n", self.is_mod));
            logger.set_last_at_indent();
            logger.log_indented_slice("Meta", &self.metas, |logger, meta| meta.log(logger));
        })
    }
}

pub enum AttribMeta {
    Simple{
        path: AstNodeRef<SimplePath>,
    },
    Expr {
        expr: Expr,
    },
    Assign{ 
        path: AstNodeRef<SimplePath>,
        expr: Expr
    },
    Meta{
        path:  AstNodeRef<SimplePath>,
        metas: Vec<AttribMeta>,
    }
}

impl AttribMeta {
    fn log(&self, logger: &mut AstLogger) {
        match self {
            Self::Simple { path }       => logger.log_indented_node_ref("Simple Attrib Meta", *path),
            Self::Expr { expr }         => logger.log_indented_node("Expression Attrib Meta", expr),
            Self::Assign { path, expr } => logger.log_indented("Assign Attribute Meta", |logger| {
                logger.log_indented_node_ref("Path", *path);
                logger.set_last_at_indent();
                logger.log_indented_node("Expr", expr)
            }),
            Self::Meta { path, metas }  => logger.log_indented("Nested Attribute Meta", |logger| {
                logger.log_indented_node_ref("Path", *path);
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
    fn log(&self, logger: &mut AstLogger) {
        
    }
}

// =============================================================================================================================

#[derive(PartialEq, Eq, Debug)]
pub struct AstNodeRef<T> {
    idx:      usize,
    _phantom: PhantomData<T>,
}

impl<T> Clone for AstNodeRef<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for AstNodeRef<T> {}

impl<T> AstNodeRef<T> {
    pub fn index(&self) -> usize {
        self.idx
    }
}



pub struct Ast {
    pub file:  PathBuf,
    pub nodes: Vec<Box<dyn AstNode>>,
    pub meta:  Vec<AstNodeMeta>,

    pub items: Vec<Item>,
}

impl Ast {
    pub fn new() -> Self {
        Self {
            file:  PathBuf::new(),
            nodes: Vec::new(),
            meta:  Vec::new(),
            items: Vec::new(),
        }
    }

    pub fn add_node<T: AstNode + 'static>(&mut self, node: T, meta: AstNodeMeta) -> AstNodeRef<T> {
        let idx = self.nodes.len();
        self.nodes.push(Box::new(node));
        self.meta.push(meta);
        AstNodeRef { idx, _phantom: PhantomData }
    }

    pub fn get<T: AstNode + 'static>(&self, id: AstNodeRef<T>) -> (&T, &AstNodeMeta) {
        let node = &*self.nodes[id.idx];

        // SAFETY: We cannot manually create node references and the indices are stable, so we will always have a value of the type we want here
        let node_ptr = &*node as *const _ as *const T;
        (unsafe { &*node_ptr }, &self.meta[id.idx])
    }

    pub fn get_mut<T: AstNode + 'static>(&mut self, id: AstNodeRef<T>) -> (&mut T, &mut AstNodeMeta) {
        let node = &mut *self.nodes[id.idx];

        // SAFETY: We cannot manually create node references and the indices are stable, so we will always have a value of the type we want here
        let node_ptr = &mut *node as *mut _ as *mut T;
        (unsafe { &mut *node_ptr }, &mut self.meta[id.idx])
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

impl<T: AstNode + 'static> Index<AstNodeRef<T>> for Ast {
    type Output = T;

    fn index(&self, index: AstNodeRef<T>) -> &Self::Output {
        self.get(index).0
    }
}

impl<T: AstNode + 'static> IndexMut<AstNodeRef<T>> for Ast {
    fn index_mut(&mut self, index: AstNodeRef<T>) -> &mut Self::Output {
        self.get_mut(index).0
    }
}

pub struct AstLogger<'a> {
    ast:          &'a Ast,
    names:        &'a NameTable,
    literals:     &'a LiteralTable,
    puncts:       &'a PuncutationTable,

    logger:       IndentLogger,
    indent_kinds: Vec<bool>,

    node_id:      usize,
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
            node_id: 0,
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

    fn log_node_ref<T: AstNode + 'static>(&mut self, id: AstNodeRef<T>) {
        let prev_id = self.node_id;
        self.node_id = id.idx;
        
        let node = &self.ast[id];
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
        let (first_tok, last_tok) = if self.node_id < self.ast.meta.len() {
            let meta = &self.ast.meta[self.node_id];
            (meta.first_tok, meta.last_tok)
        } else {
            (0, 0)
        };


        self.prefixed_log_fmt(format_args!("[ {name} ] (node id: {}, tokens: [{first_tok}..{last_tok}])\n", self.node_id));
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

    pub fn log_indented_node_ref<T: AstNode + 'static>(&mut self, name: &'static str, id: AstNodeRef<T>) {
        let node = &self.ast[id];
        self.node_id = id.idx;
        self.log_indented_node(name, node);
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
            logger.log_node_ref(*node);
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

    pub fn log_opt_node_ref<T: AstNode + 'static>(&mut self, id: &Option<AstNodeRef<T>>) {
        if let Some(id) = id {
            self.log_node_ref(*id);
        }
    }

    pub fn log_indented_opt_node<T: AstNode + 'static>(&mut self, name: &'static str, val: &Option<T>) {
        if let Some(val) = val {
            self.log_indented_node(name, val)
        }
    }
    
    pub fn log_indented_opt_node_ref<T: AstNode + 'static>(&mut self, name: &'static str, val: &Option<AstNodeRef<T>>) {
        if let Some(val) = val {
            self.log_indented_node_ref(name, *val)
        }
    }
    
    pub fn log_opt_node<T: AstNode + 'static>(&mut self, node: &Option<T>) {
        if let Some(node) = node {
            node.log(self);
        }
    }
}