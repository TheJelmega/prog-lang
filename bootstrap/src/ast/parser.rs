#![allow(unused)]

use std::{
    fmt,
    marker::PhantomData,
    path::{self, Path},
    result
};

use crate::{
    ast::*,
    error_warning::ErrorCode,
    lexer::{OpenCloseSymbol, Punctuation, PunctuationId, StrongKeyword, Token, TokenMetadata, TokenStore, WeakKeyword},
    literals::LiteralId,
    common::{NameTable, NameId}
};

use super::*;

pub struct ParserErr {
    pub err:     ErrorCode,
    pub tok_idx: usize,
}


impl fmt::Display for ParserErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.err)
    }
}


#[derive(Clone, Copy)]
pub struct ParserFrame {
    token_start: u32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ExprParseMode {
    AllowComma,
    Prefix,
    AllowLet,
    Scrutinee,
    NoStructLit,
    General,
}

pub struct Parser<'a> {
    token_store:    &'a TokenStore,
    token_idx:      usize,

    frames:         Vec<ParserFrame>,
    last_frame:     ParserFrame,
    scope_stack:    Vec<OpenCloseSymbol>,

    names:          &'a NameTable,
    pub ast:        Ast,
}

impl<'a> Parser<'a> {
    pub fn new(token_store: &'a TokenStore, names: &'a NameTable) -> Self {
        Self {
            token_store,
            token_idx: 0,

            frames: Vec::new(),
            last_frame: ParserFrame{ token_start: 0 },
            scope_stack: Vec::new(),

            names,
            ast: Ast::new(),
        }
    }

}

impl Parser<'_> {
    pub fn parse(&mut self) -> Result<(), ParserErr> {
        while self.token_idx != self.token_store.tokens.len() {
            let item = self.parse_item(None)?;
            self.ast.items.push(item);
        }

        Ok(())
    }

    fn try_peek(&self) -> Option<Token> {
        if self.token_idx < self.token_store.tokens.len() {
            Some(self.token_store.tokens[self.token_idx])
        } else {
            None
        }
    }

    fn try_peek_at(&self, offset: usize) -> Option<Token> {
        if self.token_idx + offset < self.token_store.tokens.len() {
            Some(self.token_store.tokens[self.token_idx + offset])
        } else {
            None
        }
    }

    fn peek(&self) -> Result<Token, ParserErr> {
        if self.token_idx < self.token_store.tokens.len() {
            Ok(self.token_store.tokens[self.token_idx])
        } else {
            Err(self.gen_error(ErrorCode::ParseNotEnoughTokens))
        }
    }

    fn peek_at(&self, offset: usize) -> Result<Token, ParserErr> {
        if self.token_idx + offset < self.token_store.tokens.len() {
            Ok(self.token_store.tokens[self.token_idx + offset])
        } else {
            Err(self.gen_error(ErrorCode::ParseNotEnoughTokens))
        }
    }

    fn check_peek(&self, indices: &[usize], token: Token) -> bool {
        for idx in indices {
            if self.try_peek_at(*idx) == Some(token) {
                return true;
            }
        }
        false
    }

    fn consume_single(&mut self) -> Token {
        let token_idx = self.token_idx;
        self.token_idx += 1;
        self.token_store.tokens[token_idx]
    }

    fn consume(&mut self, expected: Token) -> Result<(), ParserErr> {
        let peek = self.peek()?;
        if peek == expected {
            self.consume_single();
            Ok(())
        } else {
            Err(self.gen_error(ErrorCode::ParseFoundButExpected{ found: peek, expected }))
        }
    }

    fn try_consume_name(&mut self) -> Option<NameId> {
        let peek = match self.try_peek() {
            Some(peek) => peek,
            None       => return None,
        };
        match peek {
            Token::Name(name_id) => {
                self.consume_single();
                Some(name_id)
            },
            Token::WeakKw(kw) => {
                self.consume_single();
                let id = self.token_store.get_name_from_weak_keyword(kw);
                Some(id)
            }
            _ => None
        }
    }

    fn consume_name(&mut self) -> Result<NameId, ParserErr> {
        let peek = self.peek()?;
        match peek {
            Token::Name(name_id) => {
                self.consume_single();
                Ok(name_id)
            },
            Token::WeakKw(kw) => {
                self.consume_single();
                let id = self.token_store.get_name_from_weak_keyword(kw);;
                Ok(id)
            }
            _ => Err(self.gen_error(ErrorCode::ParseFoundButExpected{ found: peek, expected: Token::Name(NameId::INVALID) }))
        }
    }

    fn consume_lit(&mut self) -> Result<LiteralId, ParserErr> {
        let peek = self.peek()?;
        if let Token::Literal(lit_id) = peek {
            self.consume_single();
            Ok(lit_id)
        } else {
            Err(self.gen_error(ErrorCode::ParseFoundButExpected{ found: peek, expected: Token::Literal(LiteralId::INVALID) }))
        }
    }

    fn consume_any_punct(&mut self) -> Result<Punctuation, ParserErr> {
        let peek = self.peek()?;
        if let Token::Punctuation(punct) = peek {
            self.consume_single();
            Ok(punct)
        } else {
            Err(self.gen_error(ErrorCode::ParseFoundButExpected{ found: peek, expected: Token::Punctuation(Punctuation::Custom(PunctuationId::INVALID)) }))
        }
    }

    fn consume_punct(&mut self, punct: Punctuation) -> Result<(), ParserErr> {
        self.consume(Token::Punctuation(punct))
    }

    fn consume_strong_kw(&mut self, kw: StrongKeyword) -> Result<(), ParserErr> {
        self.consume(Token::StrongKw(kw))
    }

    fn consume_weak_kw(&mut self, kw: WeakKeyword) -> Result<(), ParserErr> {
        self.consume(Token::WeakKw(kw))
    }

    fn try_consume(&mut self, token: Token) -> bool {
        if let Some(peek) = self.try_peek() {
            if peek == token {
                self.consume_single();
                return true;
            }
        }
        false
    }

    fn begin_scope(&mut self, sym: OpenCloseSymbol) -> Result<(), ParserErr> {
        self.consume(Token::OpenSymbol(sym))?;
        self.scope_stack.push(sym);
        Ok(())
    }

    fn try_begin_scope(&mut self, sym: OpenCloseSymbol) -> bool {
        if self.try_consume(Token::OpenSymbol(sym)) {
            self.scope_stack.push(sym);
            true
        } else {
            false
        }
    }

    fn end_scope(&mut self) -> Result<(), ParserErr> {
        let sym = self.scope_stack.pop().unwrap();
        self.consume(Token::CloseSymbol(sym))
    }

    fn try_end_scope(&mut self) -> bool {
        let sym = *self.scope_stack.last().unwrap();
        if self.try_consume(Token::CloseSymbol(sym)) {
            self.scope_stack.pop();
            true
        } else {
            false
        }
    }

    fn gen_error(&self, err: ErrorCode) -> ParserErr {
        ParserErr {
            err,
            tok_idx: self.token_idx,
        }
    }

    fn push_meta_frame(&mut self) {
        self.frames.push(ParserFrame {
            token_start: self.token_idx as u32,
        })
    }

    fn push_last_frame(&mut self) {
        self.frames.push(self.last_frame);
    }

    fn pop_meta_frame(&mut self) -> Option<ParserFrame> {
        self.frames.pop()
    }

    fn add_node<T: AstNode + 'static>(&mut self, node: T) -> AstNodeRef<T> {
        let meta = if let Some(frame) = self.pop_meta_frame() {
            self.last_frame = frame;
            AstNodeMeta {
                first_tok: frame.token_start,
                last_tok: self.token_idx as u32,
            }
        } else {
            self.last_frame = ParserFrame{ token_start: 0 };
            AstNodeMeta {
                first_tok: 0,
                last_tok: 0, 
            }  
        };
        self.ast.add_node(node, meta)
    }

// =============================================================================================================================

    fn parse_simple_path(&mut self, only_allow_none_start: bool) -> Result<AstNodeRef<SimplePath>, ParserErr> {
        self.push_meta_frame();

        let start = self.parse_simple_path_start(only_allow_none_start)?;
        let names = self.parse_punct_separated(Punctuation::Dot, Self::consume_name)?;
        Ok(self.add_node(SimplePath {
            start,
            names
        }))
    }

    fn parse_simple_path_start(&mut self, only_allow_none_start: bool) -> Result<SimplePathStart, ParserErr> {
        let tok = self.peek()?;
        match tok {
            Token::Name(name_id)                     => {
                Ok(SimplePathStart::None)
            },
            Token::Punctuation(Punctuation::Dot)     => if only_allow_none_start {
                Err(self.gen_error(ErrorCode::ParseInvalidPathStart { found: tok, reason: "inferred simple paths are not allowed" }))
            } else {
                self.consume_single();
                Ok(SimplePathStart::Inferred)
            },
            Token::WeakKw(WeakKeyword::Super)        => if only_allow_none_start {
                Err(self.gen_error(ErrorCode::ParseInvalidPathStart { found: tok, reason: "'super' relative paths are not allowed" }))
            } else {
                self.consume_single();
                Ok(SimplePathStart::Super)
            },
            Token::StrongKw(StrongKeyword::SelfName) => if only_allow_none_start {
                Err(self.gen_error(ErrorCode::ParseInvalidPathStart { found: tok, reason: "'self' relative paths are not allowed" }))
            } else {
                self.consume_single();
                Ok(SimplePathStart::SelfPath)
            },
            _                                        => Err(self.gen_error(ErrorCode::ParseInvalidPathStart{ found: tok, reason: "" }))
        }
    }

    fn parse_type_path(&mut self) -> Result<AstNodeRef<TypePath>, ParserErr> {
        self.push_meta_frame();
        let idens = self.parse_punct_separated(Punctuation::Dot, |parser| {
            let name = parser.consume_name()?;

            if let Some(gen_args) = parser.parse_generic_args(false)? {
                return Ok(TypePathIdentifier::GenArg { name, gen_args });
            }
            if let Some(gen_args) = parser.parse_generic_args(true)? {
                return Ok(TypePathIdentifier::GenArg { name, gen_args });
            }

            if parser.peek()? == Token::OpenSymbol(OpenCloseSymbol::Paren) {
                let params = parser.parse_comma_separated_closed(OpenCloseSymbol::Paren, Self::parse_type)?;

                let ret = if parser.try_consume(Token::Punctuation(Punctuation::SingleArrowR)) {
                    parser.consume_single();
                    Some(parser.parse_type()?)
                } else {
                    None
                };

                Ok(TypePathIdentifier::Fn { name, params, ret })
            } else {
                Ok(TypePathIdentifier::Plain { name })
            }
        })?;

        Ok(self.add_node(TypePath{ idens }))
    }

    fn parse_identifier(&mut self, dot_generics: bool) -> Result<Identifier, ParserErr> {
        let name = self.consume_name()?;
        let gen_args = self.parse_generic_args(dot_generics)?;
        Ok(Identifier { name, gen_args })
    }

    fn parse_expr_path(&mut self) -> Result<AstNodeRef<ExprPath>, ParserErr> {
        self.push_meta_frame();
        let inferred = self.try_consume(Token::Punctuation(Punctuation::Dot));

        let mut idens = Vec::new();
        loop {
            idens.push(self.parse_identifier(true)?);

            if self.peek()? != Token::Punctuation(Punctuation::Dot) ||!matches!(self.peek_at(1)?, Token::Name(_)) {
                break;
            }
            self.consume_punct(Punctuation::Dot)?;
        }
        Ok(self.add_node(ExprPath{
            inferred,
            idens
        }))
    }

    fn parse_qualified_path(&mut self) -> Result<AstNodeRef<QualifiedPath>, ParserErr> {
        self.begin_scope(OpenCloseSymbol::Paren);
        self.consume_punct(Punctuation::Colon)?;

        let ty = self.parse_type()?;
        let bound = if self.try_consume(Token::StrongKw(StrongKeyword::As)) {
            let name = self.consume_name()?;
            let gen_args = self.parse_generic_args(true)?;
            Some(Identifier{ name, gen_args })
        } else {
            None
        };

        self.consume_punct(Punctuation::Colon)?;
        self.end_scope();

        let mut sub_path = Vec::new();
        loop {
            let name = self.consume_name()?;
            let gen_args = self.parse_generic_args(true)?;
            sub_path.push(Identifier{ name, gen_args });

            if self.peek()? != Token::Punctuation(Punctuation::Dot) ||!matches!(self.peek_at(1)?, Token::Name(_)) {
                break;
            }
            self.consume_punct(Punctuation::Dot)?;
        }

        Ok(self.add_node(QualifiedPath {
            ty,
            bound,
            sub_path,
        }))
    }

// =============================================================================================================================

    fn parse_item(&mut self, attrs: Option<Vec<AstNodeRef<Attribute>>>) -> Result<Item, ParserErr> {
        self.push_meta_frame();

        let attrs = match attrs {
            Some(attrs) => attrs,
            None => self.parse_attributes()?
        };
        let vis = self.parse_visibility()?;

        let peek = self.peek()?;
        match peek {
            Token::StrongKw(StrongKeyword::Bitfield) => self.parse_bitfield(attrs, vis),
            Token::StrongKw(StrongKeyword::Fn)       => self.parse_function(attrs, vis).map(|item| Item::Function(item)),
            Token::StrongKw(StrongKeyword::Enum)     => self.parse_enum(attrs, vis),
            Token::StrongKw(StrongKeyword::Impl)     => self.parse_impl(attrs, vis),
            Token::StrongKw(StrongKeyword::Mod)      => self.parse_module(attrs, vis),
            Token::StrongKw(StrongKeyword::Static)   => self.parse_static_item(attrs, vis).map(|item| Item::Static(item)),
            Token::StrongKw(StrongKeyword::Struct)   => self.parse_struct(attrs, vis),
            Token::StrongKw(StrongKeyword::Trait)    => self.parse_trait(attrs, vis),
            Token::StrongKw(StrongKeyword::Use)      => self.parse_use(attrs, vis),
            Token::StrongKw(StrongKeyword::Union)    => self.parse_union(attrs, vis),
            Token::WeakKw(WeakKeyword::Flag)         => self.parse_enum(attrs, vis),
            Token::WeakKw(WeakKeyword::Sealed)       => self.parse_trait(attrs, vis),
            Token::WeakKw(WeakKeyword::Tls)          => self.parse_static_item(attrs, vis).map(|item| Item::Static(item)),
            Token::WeakKw(WeakKeyword::Precedence)   => if self.try_peek() == Some(Token::StrongKw(StrongKeyword::Use)) {
                self.parse_precedence_use(attrs, vis)
            } else {
                self.parse_precedence(attrs, vis)
            },
            Token::StrongKw(StrongKeyword::Type)     |
            Token::WeakKw(WeakKeyword::Distinct)     => self.parse_type_alias(attrs, vis).map(|item| Item::TypeAlias(item)),
            Token::WeakKw(WeakKeyword::Op)           => if self.try_peek() == Some(Token::StrongKw(StrongKeyword::Use)) {
                self.parse_op_use(attrs, vis)
            } else {
                self.parse_op_trait(attrs, vis)
            },
            Token::StrongKw(StrongKeyword::Const) => if self.check_peek(&[1, 2, 4, 5], Token::StrongKw(StrongKeyword::Fn)) {
                    self.parse_function(attrs, vis).map(|item| Item::Function(item))
                } else {
                    self.parse_const_item(attrs, vis).map(|item| Item::Const(item))
                },
            Token::StrongKw(StrongKeyword::Unsafe) => if self.check_peek(&[1, 2], Token::StrongKw(StrongKeyword::Trait))
                {
                    self.parse_trait(attrs, vis)
                } else if self.check_peek(&[1], Token::StrongKw(StrongKeyword::Impl)) {
                    self.parse_impl(attrs, vis)
                } else {
                    self.parse_function(attrs, vis).map(|item| Item::Function(item))
                },
            Token::StrongKw(StrongKeyword::Extern) => if self.check_peek(&[2], Token::StrongKw(StrongKeyword::Fn)) {
                    self.parse_function(attrs, vis).map(|item| Item::Function(item))
                } else if self.check_peek(&[2, 3], Token::StrongKw(StrongKeyword::Static)) {
                    self.parse_static_item(attrs, vis).map(|item| Item::Static(item))
                } else if self.check_peek(&[2], Token::OpenSymbol(OpenCloseSymbol::Brace)) {
                    self.parse_extern_block(attrs, vis)
                } else {
                    Err(self.gen_error(ErrorCode::ParseInvalidExternUse))
                },
            Token::WeakKw(WeakKeyword::Record) => {
                if self.check_peek(&[1], Token::StrongKw(StrongKeyword::Struct)) {
                    self.parse_struct(attrs, vis)
                } else if self.check_peek(&[1], Token::StrongKw(StrongKeyword::Enum)) {
                    self.parse_enum(attrs, vis)
                } else if self.check_peek(&[1], Token::StrongKw(StrongKeyword::Bitfield)) {
                    self.parse_bitfield(attrs, vis)
                } else {
                    Err(self.gen_error(ErrorCode::ParseUnexpectedFor{ found: peek, for_reason: "item" }))
                }
            },
            Token::StrongKw(StrongKeyword::Mut) => {
                if self.check_peek(&[1, 2], Token::StrongKw(StrongKeyword::Struct)) {
                    self.parse_struct(attrs, vis)
                } else if self.check_peek(&[1], Token::StrongKw(StrongKeyword::Union)) {
                    self.parse_union(attrs, vis)
                } else if self.check_peek(&[1, 2], Token::StrongKw(StrongKeyword::Enum)) {
                    self.parse_enum(attrs, vis)
                } else if self.check_peek(&[1, 2], Token::StrongKw(StrongKeyword::Bitfield)) {
                    self.parse_bitfield(attrs, vis)
                } else if self.check_peek(&[1, 2, 3], Token::StrongKw(StrongKeyword::Static)) {
                    self.parse_static_item(attrs, vis).map(|item| Item::Static(item))
                } else {
                    Err(self.gen_error(ErrorCode::ParseUnexpectedFor{ found: peek, for_reason: "Item" }))
                }
            },
            _ => Err(self.gen_error(ErrorCode::ParseUnexpectedFor{ found: peek, for_reason: "Item" }))
        }
    }

    fn parse_trait_item(&mut self) -> Result<TraitItem, ParserErr> {
        self.push_meta_frame();

        let attrs = self.parse_attributes()?;
        let vis = self.parse_visibility()?;

        let peek = self.peek()?;
        match peek {
            Token::StrongKw(StrongKeyword::Fn)  => self.parse_function(attrs, vis).map(|item| TraitItem::Function(item)),
            Token::StrongKw(StrongKeyword::Const) => {
                let peek_1 = self.try_peek_at(1);
                let peek_2 = self.try_peek_at(2);
                let peek_4 = self.try_peek_at(4);
                let peek_5 = self.try_peek_at(5); 
                if  peek_1 == Some(Token::StrongKw(StrongKeyword::Fn)) || // const fn..
                    peek_2 == Some(Token::StrongKw(StrongKeyword::Fn)) || // const unsafe fn..
                    peek_4 == Some(Token::StrongKw(StrongKeyword::Fn)) || // const extern "abi" fn.. (invalid)
                    peek_5 == Some(Token::StrongKw(StrongKeyword::Fn))    // const unsafe extenr "abi" fn... (invalid)
                {
                    self.parse_function(attrs, vis).map(|item| TraitItem::Function(item))
                } else {
                    self.parse_const_item(attrs, vis).map(|item| TraitItem::Const(item))
                }
            }
            Token::StrongKw(StrongKeyword::Unsafe) => {
                let peek = self.peek_at(1)?;
                if peek == Token::WeakKw(WeakKeyword::Property) {
                    self.parse_property(attrs, vis, true).map(|item| TraitItem::Property(item))
                } else {
                    self.parse_function(attrs, vis).map(|item| TraitItem::Function(item))
                }
            },
            Token::StrongKw(StrongKeyword::Type) => self.parse_type_alias(attrs, vis).map(|item| TraitItem::TypeAlias(item)),
            Token::WeakKw(WeakKeyword::Property) => self.parse_property(attrs, vis, true).map(|item| TraitItem::Property(item)),
            _ => Err(self.gen_error(ErrorCode::ParseUnexpectedFor{ found: peek, for_reason: "Item" }))
        }
    }

    fn parse_assoc_item(&mut self) -> Result<AssocItem, ParserErr> {
        self.push_meta_frame();
        
        let attrs = self.parse_attributes()?;
        let vis = self.parse_visibility()?;

        let peek = self.peek()?;
        match peek {
            Token::StrongKw(StrongKeyword::Fn)  => self.parse_function(attrs, vis).map(|item| AssocItem::Function(item)),
            Token::StrongKw(StrongKeyword::Const) => {
                let peek_1 = self.try_peek_at(1);
                let peek_2 = self.try_peek_at(2);
                let peek_4 = self.try_peek_at(4);
                let peek_5 = self.try_peek_at(5); 
                if  peek_1 == Some(Token::StrongKw(StrongKeyword::Fn)) || // const fn..
                    peek_2 == Some(Token::StrongKw(StrongKeyword::Fn)) || // const unsafe fn..
                    peek_4 == Some(Token::StrongKw(StrongKeyword::Fn)) || // const extern "abi" fn.. (invalid)
                    peek_5 == Some(Token::StrongKw(StrongKeyword::Fn))    // const unsafe extenr "abi" fn... (invalid)
                {
                    self.parse_function(attrs, vis).map(|item| AssocItem::Function(item))
                } else {
                    self.parse_const_item(attrs, vis).map(|item| AssocItem::Const(item))
                }
            }
            Token::StrongKw(StrongKeyword::Unsafe) => {
                let peek_1 = self.peek_at(1)?;
                if peek_1 == Token::WeakKw(WeakKeyword::Property) {
                    self.parse_property(attrs, vis, false).map(|item| AssocItem::Property(item))
                } else {
                    self.parse_function(attrs, vis).map(|item| AssocItem::Function(item))
                }
            },
            Token::StrongKw(StrongKeyword::Type) => self.parse_type_alias(attrs, vis).map(|item| AssocItem::TypeAlias(item)),
            Token::StrongKw(StrongKeyword::Mut) => {
                let peek_1 = self.try_peek_at(1);
                let peek_2 = self.try_peek_at(2);
                let peek_3 = self.try_peek_at(3);
                if peek_1 == Some(Token::StrongKw(StrongKeyword::Static)) ||
                    peek_2 == Some(Token::StrongKw(StrongKeyword::Static)) ||
                    peek_3 == Some(Token::StrongKw(StrongKeyword::Static))
                {
                    self.parse_static_item(attrs, vis).map(|item| AssocItem::Static(item))
                } else {
                    Err(self.gen_error(ErrorCode::ParseUnexpectedFor{ found: peek, for_reason: "Item" }))
                }
            },
            Token::StrongKw(StrongKeyword::Static) => self.parse_static_item(attrs, vis).map(|item| AssocItem::Static(item)),
            Token::WeakKw(WeakKeyword::Tls) => self.parse_static_item(attrs, vis).map(|item| AssocItem::Static(item)),
            Token::WeakKw(WeakKeyword::Property) => self.parse_property(attrs, vis, false).map(|item| AssocItem::Property(item)),

            _ => Err(self.gen_error(ErrorCode::ParseUnexpectedFor{ found: peek, for_reason: "Item" }))
        }
    }

    fn parse_extern_item(&mut self) -> Result<ExternItem, ParserErr> {
        let attrs = self.parse_attributes()?;
        let vis = self.parse_visibility()?;

        let peek = self.peek()?;
        match peek {
            Token::StrongKw(StrongKeyword::Fn)  => self.parse_function(attrs, vis).map(|item| ExternItem::Function(item)),
            Token::StrongKw(StrongKeyword::Unsafe) => self.parse_function(attrs, vis).map(|item| ExternItem::Function(item)),
            Token::StrongKw(StrongKeyword::Mut) => {
                let peek_1 = self.try_peek_at(1);
                let peek_2 = self.try_peek_at(2);
                let peek_3 = self.try_peek_at(3);
                if peek_1 == Some(Token::StrongKw(StrongKeyword::Static)) ||
                    peek_2 == Some(Token::StrongKw(StrongKeyword::Static)) ||
                    peek_3 == Some(Token::StrongKw(StrongKeyword::Static))
                {
                    self.parse_static_item(attrs, vis).map(|item| ExternItem::Static(item))
                } else {
                    Err(self.gen_error(ErrorCode::ParseUnexpectedFor{ found: peek, for_reason: "Item" }))
                }
            },
            Token::StrongKw(StrongKeyword::Static) => self.parse_static_item(attrs, vis).map(|item| ExternItem::Static(item)),

            _ => Err(self.gen_error(ErrorCode::ParseUnexpectedFor{ found: peek, for_reason: "Item" }))
        }
    }

    fn parse_module(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<Item, ParserErr> {
        self.consume(Token::StrongKw(StrongKeyword::Mod))?;
        let name = self.consume_name()?;
        
         let block = if self.try_consume(Token::Punctuation(Punctuation::Semicolon)) {
            None
        } else {
            Some(self.parse_block()?)
        };

        Ok(Item::Module(self.add_node(ModuleItem {
            attrs,
            vis,
            name,
            block,
        })))
    }

    fn parse_use(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<Item, ParserErr> {
        self.consume(Token::StrongKw(StrongKeyword::Use))?;

        let peek = self.peek()?;
        let (group, package) = match peek {
            Token::Punctuation(Punctuation::Colon) => (None, None),
            Token::Name(name_id) => {
                self.consume_single();
                if self.try_consume(Token::Punctuation(Punctuation::Dot)) {
                    let package_name_id = self.consume_name()?;
                    (Some(name_id), Some(package_name_id))
                } else {
                    (None, Some(name_id))
                }
            },
            _ => return Err(self.gen_error(ErrorCode::ParseExpectPackageName{ found: peek })),
        };
        self.consume_punct(Punctuation::Colon)?;

        let peek = self.peek()?;
        let module = match peek {
            Token::Punctuation(Punctuation::Dot) => None,
            Token::Name(name_id) => {
                self.consume_single();
                Some(name_id)
            },
            _ => return Err(self.gen_error(ErrorCode::ParseExpectModuleName{ found: peek })),
        };
        self.consume_punct(Punctuation::Dot)?;

        let path = self.parse_use_path()?;

        self.consume_punct(Punctuation::Semicolon);

        Ok(Item::Use(self.add_node(UseItem {
            attrs,
            vis,
            group,
            package,
            module,
            path,
        })))
    }

    fn parse_use_path(&mut self) -> Result<AstNodeRef<UsePath>, ParserErr> {
        if self.try_consume(Token::StrongKw(StrongKeyword::SelfName)) {

            let alias = if self.try_consume(Token::StrongKw(StrongKeyword::As)) {
                Some(self.consume_name()?)
            } else {
                None
            };
            Ok(self.add_node(UsePath::SelfPath { alias }))
        } else {
            let mut segments = Vec::new();
            let mut sub_paths = Vec::new();

            segments.push(self.consume_name()?);
    
            while self.try_consume(Token::Punctuation(Punctuation::Dot)) {
                let peek = self.peek()?;
                match peek {
                    Token::Name(name_id) => {
                        segments.push(name_id);
                        self.consume_single();
                    },
                    Token::OpenSymbol(OpenCloseSymbol::Brace) => {
                        self.begin_scope(OpenCloseSymbol::Brace)?;

                        let mut comma = true;
                        while comma && self.peek()? != Token::CloseSymbol(OpenCloseSymbol::Brace) {
                            sub_paths.push(self.parse_use_path()?);
                            comma = self.try_consume(Token::Punctuation(Punctuation::Comma));
                        }
                        self.end_scope()?;
                    },
                    _ => todo!()
                }
            }

            if !sub_paths.is_empty() {
                Ok(self.add_node(UsePath::SubPaths { segments, sub_paths }))
            } else {
                let alias = if self.try_consume(Token::StrongKw(StrongKeyword::As)) {
                    Some(self.consume_name()?)  
                } else {
                    None
                };
        
                Ok(self.add_node(UsePath::Alias { segments, alias }))
            }
        }
    }

    fn parse_function(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<AstNodeRef<Function>, ParserErr> {
        let is_override = self.try_consume(Token::WeakKw(WeakKeyword::Override));
        let is_const = self.try_consume(Token::StrongKw(StrongKeyword::Const));
        let is_unsafe = self.try_consume(Token::StrongKw(StrongKeyword::Unsafe));

        let abi = if self.try_consume(Token::StrongKw(StrongKeyword::Extern)) {
            Some(self.consume_lit()?)
        } else {
            None
        };

        self.consume_strong_kw(StrongKeyword::Fn)?;
        let name = self.consume_name()?;
        let generics = self.parse_generic_params()?;

        self.begin_scope(OpenCloseSymbol::Paren)?;
        let (receiver, has_possible_params) = if self.peek()? == Token::StrongKw(StrongKeyword::SelfName) ||
            self.peek_at(1)? == Token::StrongKw(StrongKeyword::SelfName) ||
            self.peek_at(2)? == Token::StrongKw(StrongKeyword::SelfName)
        {
            let res = if self.peek_at(1)? == Token::Punctuation(Punctuation::Colon) ||
                self.peek_at(2)? == Token::Punctuation(Punctuation::Colon)
            {
                let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
                self.consume(Token::StrongKw(StrongKeyword::SelfName))?;
                self.consume_punct(Punctuation::Colon)?;
                let ty = self.parse_type()?;
                FnReceiver::SelfTyped{ is_mut, ty }
            } else {
                let is_ref = self.try_consume(Token::Punctuation(Punctuation::Ampersand));
                let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
                self.consume(Token::StrongKw(StrongKeyword::SelfName))?;
                FnReceiver::SelfReceiver { is_ref, is_mut }
            };

            let has_possible_params = self.try_consume(Token::Punctuation(Punctuation::Comma));
            (Some(res), has_possible_params)
        } else {
            (None, true)
        };

        let mut params = if has_possible_params {
            self.parse_comma_separated_end(Punctuation::Comma, Token::CloseSymbol(OpenCloseSymbol::Paren), Self::parse_function_param)?
        } else {
            Vec::new()
        };
        self.end_scope()?;

        let returns = if self.try_consume(Token::Punctuation(Punctuation::SingleArrowR)) {
            Some(self.parse_func_return()?)
        } else {
            None
        };

        let where_clause = self.parse_where_clause()?;

        let contracts = if self.peek()? != Token::OpenSymbol(OpenCloseSymbol::Brace) {
            let mut contracts = Vec::new();
            while self.peek()? != Token::OpenSymbol(OpenCloseSymbol::Brace) {
                contracts.push(self.parse_contract()?)
            }
            contracts
        } else {
            Vec::new()
        };

        let body = self.parse_block()?;

        Ok(self.add_node(Function {
            attrs,
            vis,
            is_override,
            is_const,
            is_unsafe,
            abi,
            name,
            generics,
            receiver,
            params,
            returns,
            where_clause,
            contracts,
            body,
        }))
    }

    fn parse_function_param(&mut self) -> Result<FnParam, ParserErr> {
        let mut names = self.parse_param_names()?;
        self.consume_punct(Punctuation::Colon)?;
        let ty = self.parse_type()?;
        let is_variadic = self.try_consume(Token::Punctuation(Punctuation::DotDotDot));

        let def_val = if self.try_consume(Token::Punctuation(Punctuation::Equals)) {
            Some(self.parse_expr(ExprParseMode::General)?)
        } else {
            None
        };

        Ok (FnParam {
            names,
            ty,
            is_variadic,
            def_val,
        })
    }

    fn parse_param_names(&mut self) -> Result<Vec<FnParamName>, ParserErr> {
        let mut names = Vec::new();
        while self.peek()? != Token::CloseSymbol(OpenCloseSymbol::Paren) {
            let label = if self.try_consume(Token::Punctuation(Punctuation::Colon)) {
                Some(self.consume_name()?)
            } else {
                None
            };

            let attrs = self.parse_attributes()?;

            let pattern = self.parse_pattern()?;

            names.push(FnParamName {
                attrs,
                label,
                pattern,
            });

            if !self.try_consume(Token::Punctuation(Punctuation::Comma)) {
                break;
            }
        }

        Ok(names)
    }

    fn parse_func_return(&mut self) -> Result<FnReturn, ParserErr> {
        if self.try_begin_scope(OpenCloseSymbol::Brace) {
            let mut elems = Vec::new();
            while !self.try_end_scope() {
                let mut names = Vec::new();
                loop {
                    names.push(self.consume_name()?);
                    if !self.try_consume(Token::Punctuation(Punctuation::Comma)) {
                        break;
                    }
                }
                self.consume_punct(Punctuation::Colon);
                let ty = self.parse_type()?;
                elems.push((names, ty));
                
                if !self.try_consume(Token::Punctuation(Punctuation::Comma)) {
                    self.end_scope()?;
                    break;
                }
            }
            Ok(FnReturn::Named(elems))
        } else {
            let ty = self.parse_type()?;
            Ok(FnReturn::Type(ty))
        }
    }

    fn parse_type_alias(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<AstNodeRef<TypeAlias>, ParserErr> {
        if self.try_consume(Token::WeakKw(WeakKeyword::Distinct)) {
            self.consume_strong_kw(StrongKeyword::Type)?;
            let name = self.consume_name()?;
            let generics = self.parse_generic_params()?;
            self.consume_punct(Punctuation::Equals)?;

            let ty = self.parse_type()?;
            self.consume_punct(Punctuation::Semicolon)?;
            return Ok(self.add_node(TypeAlias::Distinct {
                attrs,
                vis,
                name,
                generics,
                ty,
            }));
        }


        let is_distinct = self.try_consume(Token::WeakKw(WeakKeyword::Distinct));
        self.consume_strong_kw(StrongKeyword::Type)?;
        let name = self.consume_name()?;
        let generics = self.parse_generic_params()?;

        if self.try_consume(Token::Punctuation(Punctuation::Semicolon)) {
            return Ok(self.add_node(TypeAlias::Trait {
                attrs,
                name,
                generics,
            }));
        }

        self.consume_punct(Punctuation::Equals)?;

        if self.try_consume(Token::WeakKw(WeakKeyword::Opaque)) {
            let size = if self.try_begin_scope(OpenCloseSymbol::Bracket) {
                let expr = self.parse_expr(ExprParseMode::AllowComma)?;
                self.end_scope()?;
                Some(expr)
            } else {
                None
            };
            self.consume_punct(Punctuation::Semicolon)?;

            Ok(self.add_node(TypeAlias::Opaque {
                attrs,
                vis,
                name,
                size,
            }))
        } else {   
            let ty = if self.try_consume(Token::Punctuation(Punctuation::Equals)) {
                Some(self.parse_type()?)
            } else {
                None
            };

            let ty = self.parse_type()?;
            self.consume_punct(Punctuation::Semicolon)?;
            Ok(self.add_node(TypeAlias::Normal {
                attrs,
                vis,
                name,
                generics,
                ty,
            }))
        }
    }

    fn parse_struct(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<Item, ParserErr> {
        let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
        let is_record = self.try_consume(Token::WeakKw(WeakKeyword::Record));

        self.consume_strong_kw(StrongKeyword::Struct)?;
        let name = self.consume_name()?;

        let generics = self.parse_generic_params()?;
        let where_clause = self.parse_where_clause()?;

        let peek = self.peek()?;
        match peek {
            Token::OpenSymbol(OpenCloseSymbol::Brace) => {
                let fields = self.parse_comma_separated_closed(OpenCloseSymbol::Brace, Self::parse_struct_field)?;
                Ok(Item::Struct(self.add_node(Struct::Regular {
                    attrs,
                    vis,
                    is_mut,
                    is_record,
                    name,
                    generics,
                    where_clause,
                    fields,
                })))
            },
            Token::OpenSymbol(OpenCloseSymbol::Paren) => {
                let fields = self.parse_comma_separated_closed(OpenCloseSymbol::Paren, Self::parse_tuple_struct_field)?;
                self.consume_punct(Punctuation::Semicolon)?;
                Ok(Item::Struct(self.add_node(Struct::Tuple {
                    attrs,
                    vis,
                    is_mut,
                    is_record,
                    name,
                    generics,
                    where_clause,
                    fields,
                })))
            },
            Token::Punctuation(Punctuation::Semicolon) => {
                if generics.is_some() {
                    todo!()
                }

                self.consume_punct(Punctuation::Semicolon)?;
                Ok(Item::Struct(self.add_node(Struct::Unit { attrs, vis, name })))
            }
            _ => Err(self.gen_error(ErrorCode::ParseUnexpectedFor{ found: peek, for_reason: "struct" }))
        }
    }

    fn parse_struct_field(&mut self) -> Result<RegStructField, ParserErr> {
        let attrs = self.parse_attributes()?;
        let vis = self.parse_visibility()?;
        let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));

        if self.try_consume(Token::StrongKw(StrongKeyword::Use)) {
            let path = self.parse_type_path()?;
            Ok(RegStructField::Use {
                attrs,
                vis,
                is_mut,
                path
            })
        } else {
            let mut names = Vec::new();
            loop {
                names.push(self.consume_name()?);
                if !self.try_consume(Token::Punctuation(Punctuation::Comma)) {
                    break;
                }
            }

            self.consume_punct(Punctuation::Colon)?;
            let ty = self.parse_type()?;

            let def = if self.try_consume(Token::Punctuation(Punctuation::Equals)) {
                Some(self.parse_expr(ExprParseMode::General)?)
            } else {
                None
            };

            Ok(RegStructField::Field {
                attrs,
                vis,
                is_mut,
                names,
                ty,
                def,
            })
        }
    }

    fn parse_tuple_struct_field(&mut self) -> Result<TupleStructField, ParserErr> {
        let attrs = self.parse_attributes()?;
        let vis = self.parse_visibility()?;
        let ty = self.parse_type()?;
        let def = if self.try_consume(Token::Punctuation(Punctuation::Equals)) {
            Some(self.parse_expr(ExprParseMode::General)?)
        } else {
            None
        };

        Ok(TupleStructField {
            attrs,
            vis,
            ty,
            def,
        })
    }

    fn parse_union(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<Item, ParserErr> {
        let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
        self.consume_strong_kw(StrongKeyword::Union)?;
        let name = self.consume_name()?;
        let generics = self.parse_generic_params()?;
        let where_clause = self.parse_where_clause()?;

        let fields = self.parse_comma_separated_closed(OpenCloseSymbol::Brace, Self::parse_union_field)?;
        Ok(Item::Union(self.add_node(Union {
            attrs,
            vis,
            is_mut,
            name,
            generics,
            where_clause,
            fields,
        })))
    }
 
    fn parse_union_field(&mut self) -> Result<UnionField, ParserErr> {
        let attrs = self.parse_attributes()?;
        let vis = self.parse_visibility()?;
        let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
        let name = self.consume_name()?;
        self.consume_punct(Punctuation::Colon)?;
        let ty = self.parse_type()?;

        Ok(UnionField {
            attrs,
            vis,
            is_mut,
            name,
            ty,
        })
    }

    fn parse_enum(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<Item, ParserErr> {
        let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
        let is_flag = self.try_consume(Token::WeakKw(WeakKeyword::Flag));
        let is_record = self.try_consume(Token::WeakKw(WeakKeyword::Record));
        self.consume_strong_kw(StrongKeyword::Enum)?;
        let name = self.consume_name()?;

        if is_flag {
            let variants = self.parse_comma_separated_closed(OpenCloseSymbol::Brace, |parser| {
                let attrs = parser.parse_attributes()?;
                let name = parser.consume_name()?;
                let discriminant = if parser.try_consume(Token::Punctuation(Punctuation::Equals)) {
                    Some(parser.parse_expr(ExprParseMode::General)?)
                } else {
                    None
                };

                Ok(FlagEnumVariant{ attrs, name, discriminant })
            })?;

            Ok(Item::Enum(self.add_node(Enum::Flag {
                attrs,
                vis,
                name,
                variants,
            })))
        } else {
            let generics = self.parse_generic_params()?;
            let where_clause = self.parse_where_clause()?;
            let variants = self.parse_comma_separated_closed(OpenCloseSymbol::Brace, Self::parse_enum_variant)?;

            Ok(Item::Enum(self.add_node(Enum::Adt {
                attrs,
                vis,
                is_mut,
                is_record,
                name,
                generics,
                where_clause,
                variants,
            })))
        }
    }

    fn parse_enum_variant(&mut self) -> Result<EnumVariant, ParserErr> {
        let attrs = self.parse_attributes()?;
        let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
        let name = self.consume_name()?;

        match self.peek()? {
            Token::OpenSymbol(OpenCloseSymbol::Brace) => {
                let fields = self.parse_comma_separated_closed(OpenCloseSymbol::Brace, Self::parse_struct_field)?;
                
                let discriminant = if self.try_consume(Token::Punctuation(Punctuation::Equals)) {
                    Some(self.parse_expr(ExprParseMode::General)?)
                } else {
                    None
                };

                Ok(EnumVariant::Struct {
                    attrs,
                    name,
                    is_mut,
                    fields,
                    discriminant,
                })
            },
            Token::OpenSymbol(OpenCloseSymbol::Paren) => {
                let fields = self.parse_comma_separated_closed(OpenCloseSymbol::Paren, Self::parse_tuple_struct_field)?;
                
                let discriminant = if self.try_consume(Token::Punctuation(Punctuation::Equals)) {
                    Some(self.parse_expr(ExprParseMode::General)?)
                } else {
                    None
                };

                Ok(EnumVariant::Tuple {
                    attrs,
                    name,
                    is_mut,
                    fields,
                    discriminant,
                })
            },
            _ => {
                let discriminant = if self.try_consume(Token::Punctuation(Punctuation::Equals)) {
                    Some(self.parse_expr(ExprParseMode::General)?)
                } else {
                    None
                };

                Ok(EnumVariant::Fieldless {
                    attrs,
                    name,
                    discriminant,
                })
            }
        }
    }

    fn parse_bitfield(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<Item, ParserErr> {
        let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
        let is_record = self.try_consume(Token::WeakKw(WeakKeyword::Record));
        self.consume_strong_kw(StrongKeyword::Bitfield)?;
        let name = self.consume_name()?;
        let generics = self.parse_generic_params()?;

        let bit_count = if self.try_consume(Token::Punctuation(Punctuation::Colon)) {
            Some(self.parse_expr(ExprParseMode::General)?)
        } else {
            None
        };

        let where_clause = self.parse_where_clause()?;
        let fields = self.parse_comma_separated_closed(OpenCloseSymbol::Brace, Self::parse_bitfield_field)?;

        Ok(Item::Bitfield(self.add_node(Bitfield {
            attrs,
            vis,
            is_mut,
            is_record,
            name,
            generics,
            bit_count,
            where_clause,
            fields,
        })))
    }

    fn parse_bitfield_field(&mut self) -> Result<BitfieldField, ParserErr> {
        let attrs = self.parse_attributes()?;
        let vis = self.parse_visibility()?;
        let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
        let is_record = self.try_consume(Token::WeakKw(WeakKeyword::Record));

        if self.try_consume(Token::StrongKw(StrongKeyword::Use)) {
            let path = self.parse_type_path()?;
            let bits = if self.try_consume(Token::Punctuation(Punctuation::Or)) {
                Some(self.parse_expr(ExprParseMode::General)?)
            } else {
                None
            };

            Ok(BitfieldField::Use {
                attrs,
                vis,
                is_mut,
                path,
                bits,
            })
        } else {
            
            let mut names = Vec::new();
            loop {
                names.push(self.consume_name()?);
                if !self.try_consume(Token::Punctuation(Punctuation::Comma)) {
                    break;
                }
            }

            self.consume_punct(Punctuation::Colon)?;
            let ty = self.parse_type()?;
            let bits = if self.try_consume(Token::Punctuation(Punctuation::Or)) {
                Some(self.parse_expr(ExprParseMode::General)?)
            } else {
                None
            };
            let def = if self.try_consume(Token::Punctuation(Punctuation::Or)) {
                Some(self.parse_expr(ExprParseMode::General)?)
            } else {
                None
            };

            Ok(BitfieldField::Field {
                attrs,
                vis,
                is_mut,
                names,
                ty,
                bits,
                def,
            })
        }
    }

    fn parse_const_item(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<AstNodeRef<Const>, ParserErr> {
        self.consume_strong_kw(StrongKeyword::Const)?;
        let name = self.consume_name()?;
        let ty = if self.try_consume(Token::Punctuation(Punctuation::Colon)) {
            Some(self.parse_type()?)
        } else {
            None
        };
        self.consume_punct(Punctuation::Equals)?;
        let val = self.parse_expr(ExprParseMode::General)?;
        self.consume_punct(Punctuation::Semicolon)?;

        Ok(self.add_node(Const {
            attrs,
            vis,
            name,
            ty,
            val,
        }))
    }

    fn parse_static_item(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<AstNodeRef<Static>, ParserErr> {
        if self.try_consume(Token::StrongKw(StrongKeyword::Extern)) {
            let abi = self.consume_lit()?;
            let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
            self.consume_strong_kw(StrongKeyword::Static)?;
            let name = self.consume_name()?;
            self.consume_punct(Punctuation::Colon)?;
            let ty = self.parse_type()?;
            self.consume_punct(Punctuation::Semicolon)?;

            Ok(self.add_node(Static::Extern {
                attrs,
                vis,
                abi,
                is_mut,
                name,
                ty,
            }))
        } else {
            let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
            let is_tls = self.try_consume(Token::WeakKw(WeakKeyword::Tls));

            self.consume_strong_kw(StrongKeyword::Static)?;
            let name = self.consume_name()?;
            self.consume_punct(Punctuation::Colon)?;
            let ty = self.parse_type()?;
            self.consume_punct(Punctuation::Equals);
            let val = self.parse_expr(ExprParseMode::General)?;
            self.consume_punct(Punctuation::Semicolon)?;

            if is_tls {
                Ok(self.add_node(Static::Tls {
                    attrs,
                    vis,
                    is_mut,
                    name,
                    ty,
                    val,
                }))
            } else {
                Ok(self.add_node(Static::Static {
                    attrs,
                    vis,
                    name,
                    ty,
                    val,
                }))
            }
        }
    }

    fn parse_property(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>, is_trait: bool) -> Result<AstNodeRef<Property>, ParserErr> {
        let is_unsafe = self.try_consume(Token::StrongKw(StrongKeyword::Unsafe));
        self.consume_weak_kw(WeakKeyword::Property)?;
        let name = self.consume_name()?;

        let body = if is_trait {
            let mut has_get = false;
            let mut has_ref_get = false;
            let mut has_mut_get = false;
            let mut has_set = false;

            self.begin_scope(OpenCloseSymbol::Brace)?;
            while !self.try_end_scope() {
                let peek = self.peek()?;
                match peek {
                    Token::WeakKw(WeakKeyword::Get) => {
                        self.consume_single();
                        self.consume_punct(Punctuation::Semicolon)?;
                        if has_get {
                            return Err(self.gen_error(ErrorCode::ParseDuplicateProp{ get_set: "get" }));
                        }
                        
                        has_get = true;
                    },
                    Token::StrongKw(StrongKeyword::Ref) => {
                        self.consume_single();
                        self.consume_weak_kw(WeakKeyword::Get)?;
                        self.consume_punct(Punctuation::Semicolon)?;
                        if has_ref_get {
                            return Err(self.gen_error(ErrorCode::ParseDuplicateProp{ get_set: "ref get" }));
                        }
                        
                        has_ref_get = true;
                    },
                    Token::StrongKw(StrongKeyword::Mut) => {
                        self.consume_single();
                        self.consume_weak_kw(WeakKeyword::Get)?;
                        self.consume_punct(Punctuation::Semicolon)?;
                        if has_mut_get {
                            return Err(self.gen_error(ErrorCode::ParseDuplicateProp{ get_set: "mut get" }));
                        }
                        
                        has_mut_get = true;
                    },
                    Token::WeakKw(WeakKeyword::Set) => {
                        self.consume_single();
                        self.consume_punct(Punctuation::Semicolon)?;
                        if has_set {
                           return Err(self.gen_error(ErrorCode::ParseDuplicateProp{ get_set: "set" }));
                        }
                    
                        has_set = true;
                    },
                    _ => return Err(self.gen_error(ErrorCode::ParseUnexpectedFor{ found: peek, for_reason: "property getter/setter" }))
                }
            }

            PropertyBody::Trait {
                has_get,
                has_ref_get,
                has_mut_get,
                has_set,
            }
        } else {
            let mut get = None;
            let mut ref_get = None;
            let mut mut_get = None;
            let mut set = None;
            
            self.begin_scope(OpenCloseSymbol::Brace)?;
            while !self.try_end_scope() {
                let peek = self.peek()?;
                match peek {
                    Token::WeakKw(WeakKeyword::Get) => {
                        self.consume_single();
                        let expr = self.parse_expr(ExprParseMode::General)?;
                        if !expr.has_block() {
                            self.consume_punct(Punctuation::Semicolon)?;
                        }
                        if get.is_some() {
                            return Err(self.gen_error(ErrorCode::ParseDuplicateProp{ get_set: "get" }));
                        }
                        
                        get = Some(expr)
                    },
                    Token::StrongKw(StrongKeyword::Ref) => {
                        self.consume_single();
                        self.consume_weak_kw(WeakKeyword::Get)?;
                        let expr = self.parse_expr(ExprParseMode::General)?;
                        if !expr.has_block() {
                            self.consume_punct(Punctuation::Semicolon)?;
                        }
                        if ref_get.is_some() {
                            return Err(self.gen_error(ErrorCode::ParseDuplicateProp{ get_set: "ref get" }));
                        }

                        ref_get = Some(expr)
                    },
                    Token::StrongKw(StrongKeyword::Mut) => {
                        self.consume_single();
                        self.consume_weak_kw(WeakKeyword::Get)?;
                        let expr = self.parse_expr(ExprParseMode::General)?;
                        if !expr.has_block() {
                            self.consume_punct(Punctuation::Semicolon)?;
                        }
                        if mut_get.is_some() {
                            return Err(self.gen_error(ErrorCode::ParseDuplicateProp{ get_set: "mut get" }));
                        }
                        
                        mut_get = Some(expr)
                    },
                    Token::WeakKw(WeakKeyword::Set) => {
                        self.consume_single();
                        let expr = self.parse_expr(ExprParseMode::General)?;
                        if !expr.has_block() {
                        self.consume_punct(Punctuation::Semicolon)?;
                    }
                    if set.is_some() {
                        return Err(self.gen_error(ErrorCode::ParseDuplicateProp{ get_set: "set" }));
                    }
                    
                    set = Some(expr)
                },
                _ => return Err(self.gen_error(ErrorCode::ParseUnexpectedFor{ found: peek, for_reason: "property getter/setter" }))
                }
            }

            PropertyBody::Assoc {
                get,
                ref_get,
                mut_get,
                set,
            }
        };

        Ok(self.add_node(Property {
            attrs,
            vis,
            is_unsafe,
            name,
            body,
        }))
    }

    fn parse_trait(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<Item, ParserErr> {
        let is_unsafe = self.try_consume(Token::StrongKw(StrongKeyword::Unsafe));
        let is_sealed = self.try_consume(Token::WeakKw(WeakKeyword::Sealed));
        self.consume_strong_kw(StrongKeyword::Trait)?;
        let name = self.consume_name()?;

        let bounds = if self.try_consume(Token::Punctuation(Punctuation::Colon)) {
            Some(self.parse_trait_bounds()?)
        } else {
            None
        };

        let mut assoc_items = Vec::new();
        self.begin_scope(OpenCloseSymbol::Brace);
        while !self.try_end_scope() {
            assoc_items.push(self.parse_trait_item()?);
        }

        Ok(Item::Trait(self.add_node(Trait {
            attrs,
            vis,
            is_unsafe,
            is_sealed,
            name,
            bounds,
            assoc_items,
        })))
    }

    fn parse_impl(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<Item, ParserErr> {
        let is_unsafe = self.try_consume(Token::StrongKw(StrongKeyword::Unsafe));
        self.consume_strong_kw(StrongKeyword::Impl)?;
        let generics = self.parse_generic_params()?;
        let ty = self.parse_type()?;
        let impl_trait = if self.try_consume(Token::StrongKw(StrongKeyword::As)) {
            Some(self.parse_type_path()?)
        } else {
            None
        };
        let where_clause = self.parse_where_clause()?;

        let mut assoc_items = Vec::new();
        self.begin_scope(OpenCloseSymbol::Brace);
        while !self.try_end_scope() {
            assoc_items.push(self.parse_assoc_item()?);
        }

        Ok(Item::Impl(self.add_node(Impl {
            attrs,
            vis,
            is_unsafe,
            generics,
            ty,
            impl_trait,
            where_clause,
            assoc_items,
        })))
    }

    fn parse_extern_block(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<Item, ParserErr> {
        self.consume_strong_kw(StrongKeyword::Extern)?;
        let abi = self.consume_lit()?;
        
        let mut items = Vec::new();
        self.begin_scope(OpenCloseSymbol::Brace);
        while !self.try_end_scope() {
            items.push(self.parse_extern_item()?);
        }

        Ok(Item::Extern(self.add_node(ExternBlock {
            attrs,
            vis,
            abi,
            items,
        })))
    }

    fn parse_op_trait(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<Item, ParserErr> {
        self.consume_weak_kw(WeakKeyword::Op)?;
        self.consume_strong_kw(StrongKeyword::Trait)?;
        let name = self.consume_name()?;

        let (bases, precedence) = if self.try_consume(Token::Punctuation(Punctuation::Colon)) {
            let mut bases = Vec::new();
            loop {
                bases.push(self.parse_simple_path(true)?);
                if !self.try_consume(Token::Punctuation(Punctuation::Ampersand)) {
                    break;
                }
            }

            (bases, None)
        } else if self.try_consume(Token::Punctuation(Punctuation::Or)) {
            let precedence = self.consume_name()?;
            (Vec::new(), Some(precedence))
        } else {
            (Vec::new(), None)
        };

        let elems = self.parse_comma_separated_closed(OpenCloseSymbol::Brace, Self::parse_op_elem)?;

        if !bases.is_empty() {
            Ok(Item::OpTrait(self.add_node(OpTrait::Extended {
                attrs,
                vis,
                name,
                bases,
                elems,
            })))
        } else {
            Ok(Item::OpTrait(self.add_node(OpTrait::Base {
                attrs,
                vis,
                name,
                precedence,
                elems,
            })))
        }

    }

    fn parse_op_elem(&mut self) -> Result<OpElem, ParserErr> {
        if self.try_consume(Token::WeakKw(WeakKeyword::Invar)) {
            self.push_meta_frame();
            let expr = self.parse_block_expr(None)?;
            return Ok(OpElem::Contract{ expr });
        }

        let peek = self.consume_single();
        let op_type = match peek {
            Token::WeakKw(WeakKeyword::Prefix)    => OpType::Prefix,
            Token::WeakKw(WeakKeyword::Postfix)   => OpType::Postfix,
            Token::WeakKw(WeakKeyword::Infix)     => OpType::Infix,
            Token::WeakKw(WeakKeyword::Assign)    => OpType::Assign,
            _ => return Err(self.gen_error(ErrorCode::ParseUnexpectedFor { found: peek, for_reason: "operator type" }))
        };

        self.consume_weak_kw(WeakKeyword::Op)?;
        let op = self.consume_any_punct()?;

        if self.try_consume(Token::Punctuation(Punctuation::Colon)) {
            let name = self.consume_name()?;
            let ret = if self.try_consume(Token::Punctuation(Punctuation::SingleArrowR)) {
                Some(self.parse_type()?)
            } else {
                None
            };
            let def = if self.try_consume(Token::Punctuation(Punctuation::Equals)) {
                Some(self.parse_expr(ExprParseMode::General)?)
            } else {
                None
            };

            Ok(OpElem::Def {
                op_type,
                op,
                name,
                ret,
                def,
            })
        } else {
            self.consume_punct(Punctuation::ColonEquals)?;
            let def = self.parse_expr(ExprParseMode::General)?;
            Ok(OpElem::Extend { op_type, op, def })
        }
    }

    fn parse_op_use(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<Item, ParserErr> {
        self.consume_weak_kw(WeakKeyword::Op)?;
        self.consume_strong_kw(StrongKeyword::Use)?;

        let peek = self.peek()?;
        let (group, package) = match peek {
            Token::Punctuation(Punctuation::Colon) => (None, None),
            Token::Name(name_id) => {
                self.consume_single();
                if self.try_consume(Token::Punctuation(Punctuation::Dot)) {
                    let package_name_id = self.consume_name()?;
                    (Some(name_id), Some(package_name_id))
                } else {
                    (None, Some(name_id))
                }
            },
            _ => return Err(self.gen_error(ErrorCode::ParseExpectPackageName{ found: peek })),
        };
        self.consume_punct(Punctuation::Colon)?;

        let peek = self.peek()?;
        let library = match peek {
            Token::Punctuation(Punctuation::Dot) => None,
            Token::Name(name_id) => {
                self.consume_single();
                Some(name_id)
            },
            _ => return Err(self.gen_error(ErrorCode::ParseExpectModuleName{ found: peek })),
        };


        let operators = if self.try_consume(Token::Punctuation(Punctuation::Dot)) {
            self.parse_comma_separated_closed(OpenCloseSymbol::Brace, Self::consume_any_punct)?
        } else {
            Vec::new()
        };

        Ok(Item::OpUse(self.add_node(OpUse {
            group,
            package,
            library,
            operators,
        })))
    }

    fn parse_precedence(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<Item, ParserErr> {
        self.consume_weak_kw(WeakKeyword::Precedence)?;
        let name = self.consume_name()?;

        let mut higher_than = None;
        let mut lower_than = None;
        let mut associativity = None;

        self.parse_comma_separated_closed(OpenCloseSymbol::Brace, |parser| {
            let peek = parser.peek()?;
            match peek {
                Token::WeakKw(WeakKeyword::HigherThan) => {
                    parser.consume_single();
                    parser.consume_punct(Punctuation::Colon)?;
                    higher_than = Some(parser.consume_name()?);
                    Ok(())
                },
                Token::WeakKw(WeakKeyword::LowerThan) => {
                    parser.consume_single();
                    parser.consume_punct(Punctuation::Colon)?;
                    lower_than = Some(parser.consume_name()?);
                    Ok(())
                },
                Token::WeakKw(WeakKeyword::Associativity) => {
                    parser.consume_single();
                    parser.consume_punct(Punctuation::Colon)?;
                    let name_id = parser.consume_name()?;
                    let assoc = match &parser.names[name_id] {
                        "none" => PrecedenceAssociativity::None,
                        "left" => PrecedenceAssociativity::Left,
                        "right" => PrecedenceAssociativity::Right,
                        _ => return Err(parser.gen_error(ErrorCode::ParseInvalidPrecedenceAssoc{ name: parser.names[name_id].to_string() }))
                    };
                    associativity = Some(assoc);
                    Ok(())
                },
                Token::Punctuation(Punctuation::Comma) => {
                    parser.consume_single();
                    Ok(())
                },
                _ => Err(parser.gen_error(ErrorCode::ParseUnexpectedFor { found: peek, for_reason: "precedence" })),
            }
        })?;

        Ok(Item::Precedence(self.add_node(Precedence {
            attrs,
            vis,
            name,
            higher_than,
            lower_than,
            associativity,
        })))
    }

    fn parse_precedence_use(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<Item, ParserErr> {
        self.consume_weak_kw(WeakKeyword::Precedence)?;
        self.consume_strong_kw(StrongKeyword::Use)?;

        let peek = self.peek()?;
        let (group, package) = match peek {
            Token::Punctuation(Punctuation::Colon) => (None, None),
            Token::Name(name_id) => {
                self.consume_single();
                if self.try_consume(Token::Punctuation(Punctuation::Dot)) {
                    let package_name_id = self.consume_name()?;
                    (Some(name_id), Some(package_name_id))
                } else {
                    (None, Some(name_id))
                }
            },
            _ => return Err(self.gen_error(ErrorCode::ParseExpectPackageName{ found: peek })),
        };
        self.consume_punct(Punctuation::Colon)?;

        let peek = self.peek()?;
        let library = match peek {
            Token::Punctuation(Punctuation::Dot) => None,
            Token::Name(name_id) => {
                self.consume_single();
                Some(name_id)
            },
            _ => return Err(self.gen_error(ErrorCode::ParseExpectModuleName{ found: peek })),
        };


        let precedences =  if self.try_consume(Token::Punctuation(Punctuation::Dot)) {
            self.parse_comma_separated_closed(OpenCloseSymbol::Brace, Self::consume_name)?
        } else {
            Vec::new()
        };

        Ok(Item::PrecedenceUse(self.add_node(PrecedenceUse {
            group,
            package,
            library,
            precedences,
        })))
    }

// =============================================================================================================================

    fn parse_block(&mut self) -> Result<AstNodeRef<Block>, ParserErr> {
        self.push_meta_frame();
        self.begin_scope(OpenCloseSymbol::Brace)?;

        let mut stmts = Vec::new();
        while !self.try_end_scope() {
            stmts.push(self.parse_stmt(true)?);
        }

        let final_expr = if let Some(Stmt::Expr(stmt)) = stmts.last() {
            if self.ast[*stmt].has_semi {
                let Some(Stmt::Expr(stmt)) = stmts.pop() else { return Err(self.gen_error(ErrorCode::InternalError("Final expr in block stopped existing when removing it"))) };
                Some(stmt)
            } else {
                None
            }
        } else {
            None
        };

        Ok(self.add_node(Block {
            stmts,
            final_expr,
        }))
    }

// =============================================================================================================================

    fn parse_stmt(&mut self, allow_expr_without_semicolon: bool) -> Result<Stmt, ParserErr> {
        self.push_meta_frame();
        
        let attrs = self.parse_attributes()?;

        let peek = self.peek()?;
        match peek {
            Token::StrongKw(StrongKeyword::Bitfield) |
            Token::StrongKw(StrongKeyword::Fn) |
            Token::StrongKw(StrongKeyword::Enum) |
            Token::StrongKw(StrongKeyword::Impl) |
            Token::StrongKw(StrongKeyword::Static) |
            Token::StrongKw(StrongKeyword::Struct) |
            Token::StrongKw(StrongKeyword::Trait) |
            Token::StrongKw(StrongKeyword::Use) |
            Token::StrongKw(StrongKeyword::Union) => self.parse_item(Some(attrs)).map(|stmt| Stmt::Item(stmt)),

            Token::StrongKw(StrongKeyword::Defer) => self.parse_defer_stmt(attrs).map(|stmt| Stmt::Defer(stmt)),
            Token::StrongKw(StrongKeyword::Let) => self.parse_let_var_decl(attrs).map(|stmt| Stmt::VarDecl(stmt)),
            Token::StrongKw(StrongKeyword::Mut) => {
                let peek_1 = self.peek_at(1)?;
                if let Token::Name(_) = peek_1 {
                    self.parse_name_var_decl(attrs).map(|stmt| Stmt::VarDecl(stmt))
                } else {
                    self.parse_item(Some(attrs)).map(|stmt| Stmt::Item(stmt))
                }
            },
            Token::StrongKw(StrongKeyword::ErrDefer) => self.parse_err_defer_stmt(attrs).map(|stmt| Stmt::ErrDefer(stmt)),

            Token::WeakKw(WeakKeyword::Flag) => if self.check_peek(&[1], Token::StrongKw(StrongKeyword::Enum)) {
                self.parse_item(Some(attrs)).map(|stmt| Stmt::Item(stmt))
            } else {
                self.parse_expr_stmt(attrs, allow_expr_without_semicolon).map(|stmt| Stmt::Expr(stmt))
            },
            Token::WeakKw(WeakKeyword::Sealed) => if self.check_peek(&[1], Token::StrongKw(StrongKeyword::Trait)) {
                self.parse_item(Some(attrs)).map(|stmt| Stmt::Item(stmt))
            } else {
                self.parse_expr_stmt(attrs, allow_expr_without_semicolon).map(|stmt| Stmt::Expr(stmt))
            },
            Token::WeakKw(WeakKeyword::Tls) => if self.check_peek(&[1], Token::StrongKw(StrongKeyword::Static)) {
                self.parse_item(Some(attrs)).map(|stmt| Stmt::Item(stmt))
            } else {
                self.parse_expr_stmt(attrs, allow_expr_without_semicolon).map(|stmt| Stmt::Expr(stmt))
            },
            Token::WeakKw(WeakKeyword::Distinct) => if self.check_peek(&[1], Token::StrongKw(StrongKeyword::Type)) {
                self.parse_item(Some(attrs)).map(|stmt| Stmt::Item(stmt))
            } else {
                self.parse_expr_stmt(attrs, allow_expr_without_semicolon).map(|stmt| Stmt::Expr(stmt))
            },
            Token::WeakKw(WeakKeyword::Record) => if self.check_peek(&[1], Token::StrongKw(StrongKeyword::Struct)) ||
                self.check_peek(&[1], Token::StrongKw(StrongKeyword::Enum)) ||
                self.check_peek(&[1], Token::OpenSymbol(OpenCloseSymbol::Brace))
            {
                self.parse_item(Some(attrs)).map(|stmt| Stmt::Item(stmt))
            } else {
                self.parse_expr_stmt(attrs, allow_expr_without_semicolon).map(|stmt| Stmt::Expr(stmt))
            },
            Token::Name(_) => {
                let peek_1 = self.peek_at(1)?;
                if peek_1 == Token::Punctuation(Punctuation::Comma) || peek_1 == Token::Punctuation(Punctuation::ColonEquals) {
                    self.parse_name_var_decl(attrs).map(|stmt| Stmt::VarDecl(stmt))
                } else {
                    self.parse_expr_stmt(attrs, allow_expr_without_semicolon).map(|stmt| Stmt::Expr(stmt))
                }
            },
            Token::Punctuation(Punctuation::Semicolon) => {
                self.consume_single();
                Ok(Stmt::Empty)
            },
            _ => self.parse_expr_stmt(attrs, allow_expr_without_semicolon).map(|stmt| Stmt::Expr(stmt))
        }
    }

    fn parse_name_var_decl(&mut self, attrs: Vec<AstNodeRef<Attribute>>) -> Result<AstNodeRef<VarDecl>, ParserErr> {
        let names = self.parse_punct_separated(Punctuation::Comma, |parser| {
            let is_mut = parser.try_consume(Token::StrongKw(StrongKeyword::Mut));
            let name = parser.consume_name()?;
            Ok((is_mut, name))
        })?;

        self.consume_punct(Punctuation::ColonEquals)?;
        let expr = self.parse_expr(ExprParseMode::AllowComma)?;
        self.consume_punct(Punctuation::Semicolon)?;

        Ok(self.add_node(VarDecl::Named {
            attrs,
            names,
            expr,
        }))
    }

    fn parse_let_var_decl(&mut self, attrs: Vec<AstNodeRef<Attribute>>) -> Result<AstNodeRef<VarDecl>, ParserErr> {
        self.consume_strong_kw(StrongKeyword::Let)?;
        self.push_meta_frame();
        let pattern = self.parse_pattern_no_top_alternative()?;
        let ty = if self.try_consume(Token::Punctuation(Punctuation::Colon)) {
            Some(self.parse_type()?)
        } else {
            None
        };
        let expr = if self.try_consume(Token::Punctuation(Punctuation::Equals)) {
            Some(self.parse_expr(ExprParseMode::AllowComma)?)
        } else {
            None
        };
        let else_block = if self.try_consume(Token::StrongKw(StrongKeyword::Else)) { 
            self.push_meta_frame();
            Some(self.parse_block_expr(None)?)
        } else {
            None
        };
        self.consume_punct(Punctuation::Semicolon)?;

        Ok(self.add_node(VarDecl::Let {
            attrs,
            pattern,
            ty,
            expr,
            else_block,
        }))
    }

    fn parse_defer_stmt(&mut self, attrs: Vec<AstNodeRef<Attribute>>) -> Result<AstNodeRef<Defer>, ParserErr> {
        self.consume_strong_kw(StrongKeyword::Defer)?;
        let expr = self.parse_expr(ExprParseMode::General)?;
        if !expr.has_block() {
            self.consume_punct(Punctuation::Semicolon)?;
        }
        Ok(self.add_node(Defer {
            attrs,
            expr,
        }))
    }

    fn parse_err_defer_stmt(&mut self, attrs: Vec<AstNodeRef<Attribute>>) -> Result<AstNodeRef<ErrDefer>, ParserErr> {
        self.consume_strong_kw(StrongKeyword::ErrDefer)?;
        let receiver = if self.try_consume(Token::Punctuation(Punctuation::Or)) {
            let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
            let name = self.consume_name()?;
            self.consume_punct(Punctuation::Or)?;
            Some(ErrDeferReceiver { is_mut, name })
        } else {
            None
        };

        let expr = self.parse_expr(ExprParseMode::General)?;
        if !expr.has_block() {
            self.consume_punct(Punctuation::Semicolon)?;
        }

        Ok(self.add_node(ErrDefer {
            attrs,
            receiver,
            expr,
        }))
    }

    fn parse_expr_stmt(&mut self, attrs: Vec<AstNodeRef<Attribute>>, allow_expr_without_semicolon: bool) -> Result<AstNodeRef<ExprStmt>, ParserErr> {
        let expr = self.parse_expr(ExprParseMode::AllowComma)?;
        let has_semi = if !expr.has_block() {
            if allow_expr_without_semicolon {
                self.try_consume(Token::Punctuation(Punctuation::Semicolon))
            } else {
                self.consume_punct(Punctuation::Semicolon)?;
                true
            }
        } else {
            false
        };

        Ok(self.add_node(ExprStmt {
            attrs,
            expr,
            has_semi,
        }))
    }

// =============================================================================================================================

    fn parse_expr(&mut self, mode: ExprParseMode) -> Result<Expr, ParserErr> {
        self.push_meta_frame();
        
        let peek = self.peek()?;
        let mut expr = match peek {
            Token::StrongKw(StrongKeyword::True)          |
            Token::StrongKw(StrongKeyword::False)         |
            Token::Literal(_)                             => self.parse_literal_expr()?,
            Token::Name(_)                                |
            Token::Punctuation(Punctuation::Dot)          => self.parse_path_expr()?,
            Token::StrongKw(StrongKeyword::Unsafe)        => self.parse_unsafe_block_expr()?,
            Token::StrongKw(StrongKeyword::Const)         => self.parse_const_block_expr()?,
            Token::StrongKw(StrongKeyword::TryExclaim)    |
            Token::StrongKw(StrongKeyword::Try)           => self.parse_try_block_expr()?,
            Token::StrongKw(StrongKeyword::If)            => self.parse_if_expr()?,
            Token::StrongKw(StrongKeyword::Loop)          => self.parse_loop_expr(None)?,
            Token::StrongKw(StrongKeyword::While)         => self.parse_while_expr(None)?,
            Token::StrongKw(StrongKeyword::Do)            => self.parse_do_while_expr(None)?,
            Token::StrongKw(StrongKeyword::For)           => self.parse_for_expr(None)?,
            Token::StrongKw(StrongKeyword::Match)         => self.parse_match_expr(None)?,
            Token::StrongKw(StrongKeyword::Break)         => self.parse_break_expr()?,
            Token::StrongKw(StrongKeyword::Continue)      => self.parse_continue_expr()?,
            Token::StrongKw(StrongKeyword::Fallthrough)   => self.parse_fallthrough_expr()?,
            Token::StrongKw(StrongKeyword::Return)        => self.parse_return_expr()?,
            Token::StrongKw(StrongKeyword::When)          => self.parse_when_expr()?,
            Token::StrongKw(StrongKeyword::SelfName)      => {
                self.consume_single();
                Expr::Path(self.add_node(PathExpr::SelfPath))
            },
            Token::StrongKw(StrongKeyword::Let) if mode == ExprParseMode::AllowLet => self.parse_let_binding_expr()?,

            Token::StrongKw(StrongKeyword::Move)          |
            Token::Punctuation(Punctuation::Or)           => self.parse_closure_expr()?,

            Token::Punctuation(Punctuation::Colon)        => {
                let label = Some(self.parse_label()?);
                let peek = self.peek()?;
                match peek {
                    Token::StrongKw(StrongKeyword::Loop)      => self.parse_loop_expr(label)?,
                    Token::StrongKw(StrongKeyword::While)     => self.parse_while_expr(label)?,
                    Token::StrongKw(StrongKeyword::Do)        => self.parse_do_while_expr(label)?,
                    Token::StrongKw(StrongKeyword::For)       => self.parse_for_expr(label)?,
                    Token::StrongKw(StrongKeyword::Match)     => self.parse_match_expr(label)?,
                    Token::OpenSymbol(OpenCloseSymbol::Brace) => Expr::Block(self.parse_block_expr(label)?),
                    _ => return Err(self.gen_error(ErrorCode::ParseInvalidLabel)),
                }
            },
            Token::Punctuation(Punctuation::Comma)            |
            Token::Punctuation(Punctuation::Semicolon)        => return Err(self.gen_error(ErrorCode::ParseUnexpectedFor { found: peek, for_reason: "expression" })),

            Token::Punctuation(_)                             => self.parse_prefix_expr()?,

            Token::OpenSymbol(OpenCloseSymbol::Brace)         => Expr::Block(self.parse_block_expr(None)?),
            Token::OpenSymbol(OpenCloseSymbol::Bracket)       => self.parse_array_expr()?,
            Token::OpenSymbol(OpenCloseSymbol::Paren)         => {
                if self.check_peek(&[1], Token::CloseSymbol(OpenCloseSymbol::Paren)) {
                    self.consume_single();
                    self.consume_single();
                    Expr::Unit
                } else if self.check_peek(&[1], Token::Punctuation(Punctuation::Colon)) {
                    self.parse_qualified_path_expr()?
                } else {
                    self.parse_paren_expr()?
                }
            },

            Token::Underscore => {
                self.consume_single();
                Expr::Underscore
            },

            _ => return Err(self.gen_error(ErrorCode::ParseUnexpectedFor { found: peek, for_reason: "expression" })),
        };

        if mode == ExprParseMode::Prefix {
            return Ok(expr)
        }

        Ok(loop {
            self.push_last_frame();
            expr = match self.peek()? {
                Token::Punctuation(Punctuation::Semicolon)    |
                Token::Punctuation(Punctuation::Colon)        |
                Token::Punctuation(Punctuation::DoubleArrow)  => {
                    self.pop_meta_frame();
                    break expr
                },
                

                Token::Punctuation(Punctuation::SingleArrowL) => break self.parse_inplace_expr(expr)?,
                Token::Punctuation(Punctuation::AndAnd) if mode == ExprParseMode::Scrutinee => {
                    self.pop_meta_frame();
                    break expr
                },
                Token::Punctuation(Punctuation::Comma)        => if mode == ExprParseMode::AllowComma {
                    self.parse_comma_expr(expr)?
                } else {
                    self.pop_meta_frame();
                    break expr;
                },
                
                Token::Punctuation(Punctuation::QuestionDot) => self.parse_field_access_or_method_expr(expr)?,
                Token::Punctuation(Punctuation::Dot) => {
                    let peek_1 = self.peek_at(1)?;
                    match peek_1 {
                        Token::Literal(_) => self.parse_tuple_index(expr)?,
                        Token::Name(_) => self.parse_field_access_or_method_expr(expr)?,
                        
                        _ => return Err(self.gen_error(ErrorCode::ParseUnexpectedFor{ found: peek_1, for_reason: "expression" })),
                    }
                },
                
                Token::Punctuation(_) => {
                    let op = self.consume_any_punct()?;
                    if self.is_end_of_expr() {
                        Expr::Postfix(self.add_node(PostfixExpr {
                            op,
                            expr,
                        }))
                    } else {
                        if let Token::Punctuation(_) = self.peek()? {
                            // If we have 2 operators following each other, try to figure out which on in infix

                            let has_prev_whitespace = !self.token_store.metadata[self.token_idx - 1].meta_elems.is_empty();
                            let has_next_whitespace = !self.token_store.metadata[self.token_idx + 1].meta_elems.is_empty();

                            if has_prev_whitespace == has_next_whitespace {
                                return Err(self.gen_error(ErrorCode::ParseAmbiguousOperators));
                            } else if has_prev_whitespace {
                               let right = self.parse_expr(mode)?;
                                Expr::Infix(self.add_node(InfixExpr {
                                    op,
                                    left: expr,
                                    right,
                                }))
                            } else { // if has_next_whitepace
                                Expr::Postfix(self.add_node(PostfixExpr {
                                    op,
                                    expr,
                                }))
                            }
                        } else {    
                            let right = self.parse_expr(mode)?;
                            Expr::Infix(self.add_node(InfixExpr {
                                op,
                                left: expr,
                                right,
                            }))
                        }
                    }
                },
                Token::StrongKw(StrongKeyword::In) |
                Token::StrongKw(StrongKeyword::ExclaimIn) => {
                    let op = if self.try_consume(Token::StrongKw(StrongKeyword::ExclaimIn)) {
                        Punctuation::NotContains
                    } else {
                        self.consume_strong_kw(StrongKeyword::In)?;
                        Punctuation::Contains
                    };
                    let right = self.parse_expr(mode)?;
                    Expr::Infix(self.add_node(InfixExpr {
                        op,
                        left: expr,
                        right,
                    }))
                }
                
                Token::OpenSymbol(OpenCloseSymbol::Brace)   => self.parse_struct_expr(expr, mode != ExprParseMode::NoStructLit)?,
                Token::OpenSymbol(OpenCloseSymbol::Bracket) => self.parse_index_expr(expr)?,
                Token::OpenSymbol(OpenCloseSymbol::Paren)   => self.parse_call_expression(expr)?,

                
                Token::StrongKw(StrongKeyword::As)         |
                Token::StrongKw(StrongKeyword::AsQuestion) |
                Token::StrongKw(StrongKeyword::AsExclaim)  => break self.parse_type_cast(expr)?,
                Token::StrongKw(StrongKeyword::Is) |
                Token::StrongKw(StrongKeyword::ExclaimIs)  => break self.parse_type_check(expr)?,


                Token::Name(_) |
                Token::Literal(_) => return Err(self.gen_error(ErrorCode::ParseUnexpectedFor{ found: peek, for_reason: "expression" })),
                
                _ => {
                    self.pop_meta_frame();
                    break expr
                },
            }
        })
    }

    pub fn is_end_of_expr(&mut self) -> bool {
        let peek = match self.try_peek() {
            Some(peek) => peek,
            None => return true,
        };
        match peek {
            Token::CloseSymbol(_)                      |
            Token::Punctuation(Punctuation::Semicolon) |
            Token::Punctuation(Punctuation::Comma)     => true,
            _ => false,
        }
    }

    fn parse_literal_expr(&mut self) -> Result<Expr, ParserErr> {
        self.parse_literal_expr_node().map(|node| Expr::Literal(self.add_node(node)))
    }

    fn parse_literal_expr_node(&mut self) -> Result<LiteralExpr, ParserErr> {
        let peek = self.peek()?;
        match peek {
            Token::Literal(lit_id) => {
                let literal = self.consume_lit()?;
                let lit_op = self.parse_literal_op()?;

                Ok(LiteralExpr {
                    literal: LiteralValue::Lit(literal),
                    lit_op
                })
            },
            Token::StrongKw(StrongKeyword::True) |
            Token::StrongKw(StrongKeyword::False) => {
                let value = self.consume_single() == Token::StrongKw(StrongKeyword::True);
                let lit_op = self.parse_literal_op()?;
                Ok(LiteralExpr {
                    literal: LiteralValue::Bool(value),
                    lit_op,
                })
            }

            _ => Err(self.gen_error(ErrorCode::ParseUnexpectedFor { found: peek, for_reason: "literal" })),
        }
    }

    fn parse_literal_op(&mut self) -> Result<Option<LiteralOp>, ParserErr> {
        Ok(if self.try_consume(Token::Punctuation(Punctuation::Colon)) {
            let peek = self.consume_single();
            Some(match peek {
                Token::Name(name_id) => LiteralOp::Name(name_id),
                Token::StrongKw(kw) => match kw {
                    StrongKeyword::U8     => LiteralOp::Primitive(PrimitiveType::U8),
                    StrongKeyword::U16    => LiteralOp::Primitive(PrimitiveType::U16),
                    StrongKeyword::U32    => LiteralOp::Primitive(PrimitiveType::U32),
                    StrongKeyword::U64    => LiteralOp::Primitive(PrimitiveType::U64),
                    StrongKeyword::U128   => LiteralOp::Primitive(PrimitiveType::U128),
                    StrongKeyword::I8     => LiteralOp::Primitive(PrimitiveType::I8),
                    StrongKeyword::I16    => LiteralOp::Primitive(PrimitiveType::I16),
                    StrongKeyword::I32    => LiteralOp::Primitive(PrimitiveType::I32),
                    StrongKeyword::I64    => LiteralOp::Primitive(PrimitiveType::I64),
                    StrongKeyword::I128   => LiteralOp::Primitive(PrimitiveType::I128),
                    StrongKeyword::F16    => LiteralOp::Primitive(PrimitiveType::F16),
                    StrongKeyword::F32    => LiteralOp::Primitive(PrimitiveType::F32),
                    StrongKeyword::F64    => LiteralOp::Primitive(PrimitiveType::F64),
                    StrongKeyword::F128   => LiteralOp::Primitive(PrimitiveType::F128),
                    StrongKeyword::Bool   => LiteralOp::Primitive(PrimitiveType::Bool),
                    StrongKeyword::B8     => LiteralOp::Primitive(PrimitiveType::B8),
                    StrongKeyword::B16    => LiteralOp::Primitive(PrimitiveType::B16),
                    StrongKeyword::B32    => LiteralOp::Primitive(PrimitiveType::B32),
                    StrongKeyword::B64    => LiteralOp::Primitive(PrimitiveType::B64),
                    StrongKeyword::Char   => LiteralOp::Primitive(PrimitiveType::Char),
                    StrongKeyword::Char7  => LiteralOp::Primitive(PrimitiveType::Char7),
                    StrongKeyword::Char8  => LiteralOp::Primitive(PrimitiveType::Char8),
                    StrongKeyword::Char16 => LiteralOp::Primitive(PrimitiveType::Char16),
                    StrongKeyword::Char32 => LiteralOp::Primitive(PrimitiveType::Char32),
                    StrongKeyword::Str    => LiteralOp::StringSlice(StringSliceType::Str),
                    StrongKeyword::Str7   => LiteralOp::StringSlice(StringSliceType::Str7),
                    StrongKeyword::Str8   => LiteralOp::StringSlice(StringSliceType::Str8),
                    StrongKeyword::Str16  => LiteralOp::StringSlice(StringSliceType::Str16),
                    StrongKeyword::Str32  => LiteralOp::StringSlice(StringSliceType::Str32),
                    StrongKeyword::CStr   => LiteralOp::StringSlice(StringSliceType::CStr),
                    _ => return Err(self.gen_error(ErrorCode::ParseUnexpectedFor{ found: peek, for_reason:  "literal operator" })),
                }
                _ => return Err(self.gen_error(ErrorCode::ParseUnexpectedFor{ found: peek, for_reason: "literal operator" })),
            })
        } else {
            None
        })
    }

    fn parse_path_expr(&mut self) -> Result<Expr, ParserErr> {
        if self.try_consume(Token::Punctuation(Punctuation::Dot)) {
            let iden = self.parse_identifier(true)?;
            Ok(Expr::Path(self.add_node(PathExpr::Inferred { iden })))
        } else {
            let iden = self.parse_identifier(true)?;
            Ok(Expr::Path(self.add_node(PathExpr::Named { iden })))
        }
    }

    fn parse_qualified_path_expr(&mut self) -> Result<Expr, ParserErr> {
        let path = self.parse_qualified_path()?;
        Ok(Expr::Path(self.add_node(PathExpr::Qualified { path })))
    }

    fn parse_block_expr(&mut self, label: Option<NameId>) -> Result<AstNodeRef<BlockExpr>, ParserErr> {
        let block = self.parse_block()?;
        Ok(self.add_node(BlockExpr {
            kind: if let Some(label) = label { BlockExprKind::Labeled { label } } else { BlockExprKind::Normal },
            block
        }))
    }

    fn parse_unsafe_block_expr(&mut self) -> Result<Expr, ParserErr> {
        self.consume_strong_kw(StrongKeyword::Unsafe)?;

        let block = self.parse_block()?;
        Ok(Expr::Block(self.add_node(BlockExpr{
            kind: BlockExprKind::Unsafe,
            block,
        })))
    }

    fn parse_const_block_expr(&mut self) -> Result<Expr, ParserErr> {
        self.consume_strong_kw(StrongKeyword::Const)?;
        let block = self.parse_block()?;
        Ok(Expr::Block(self.add_node(BlockExpr{
            kind: BlockExprKind::Const,
            block,
        })))
    }

    fn parse_try_block_expr(&mut self) -> Result<Expr, ParserErr> {
        let kind = if self.try_consume(Token::StrongKw(StrongKeyword::TryExclaim)) {
            BlockExprKind::TryUnwrap
        } else {
            self.consume_strong_kw(StrongKeyword::Try)?;
            BlockExprKind::Try
        };
        let block = self.parse_block()?;
        Ok(Expr::Block(self.add_node(BlockExpr {
            kind,
            block,
        })))
    }

    fn parse_prefix_expr(&mut self) -> Result<Expr, ParserErr> {
        let op = self.consume_any_punct()?;
        let expr = self.parse_expr(ExprParseMode::Prefix)?;
        Ok(Expr::Prefix(self.add_node(PrefixExpr {
            op,
            expr,
        })))
    }

    fn parse_paren_expr(&mut self) -> Result<Expr, ParserErr> {
        let mut exprs = self.parse_comma_separated_closed(OpenCloseSymbol::Paren, |parser| parser.parse_expr(ExprParseMode::AllowComma))?;
        if exprs.len() == 1 {
            Ok(Expr::Paren(self.add_node(ParenExpr {
                expr: exprs.pop().unwrap(),
            })))
        } else {
            Ok(Expr::Tuple(self.add_node(TupleExpr {
                exprs,
            })))
        }
    }

    fn parse_inplace_expr(&mut self, left: Expr) -> Result<Expr, ParserErr> {
        self.consume_punct(Punctuation::SingleArrowL)?;
        let right = self.parse_expr(ExprParseMode::AllowComma)?;

        Ok(Expr::Inplace(self.add_node(InplaceExpr {
            left,
            right,
        })))
    }

    fn parse_type_cast(&mut self, expr: Expr) -> Result<Expr, ParserErr> {
        if self.try_consume(Token::StrongKw(StrongKeyword::AsQuestion)) {
            let ty = self.parse_type()?;
            Ok(Expr::TypeCast(self.add_node(TypeCastExpr {
                kind: TypeCastKind::Try,
                expr,
                ty,
            })))
        } else if self.try_consume(Token::StrongKw(StrongKeyword::AsExclaim)) {
            let ty = self.parse_type()?;
            Ok(Expr::TypeCast(self.add_node(TypeCastExpr {
                kind: TypeCastKind::Unwrap,
                expr,
                ty,
            })))
        } else {
            self.consume_strong_kw(StrongKeyword::As)?;
            let ty = self.parse_type()?;
            Ok(Expr::TypeCast(self.add_node(TypeCastExpr {
                kind: TypeCastKind::Normal,
                expr,
                ty,
            })))
        }
    }

    fn parse_type_check(&mut self, expr: Expr) -> Result<Expr, ParserErr> {
        let negate = if self.try_consume(Token::StrongKw(StrongKeyword::ExclaimIs)) {
            true
        } else {
            self.consume_strong_kw(StrongKeyword::Is)?;
            false
        };
        let ty = self.parse_type()?;
        Ok(Expr::TypeCheck(self.add_node(TypeCheckExpr {
            negate,
            expr,
            ty,
        })))
    }

    fn parse_array_expr(&mut self) -> Result<Expr, ParserErr> {
        let exprs = self.parse_comma_separated_closed(OpenCloseSymbol::Bracket, |parser| parser.parse_expr(ExprParseMode::General))?;
        Ok(Expr::Array(self.add_node(ArrayExpr {
            exprs,
        })))
    }

    fn parse_struct_expr(&mut self, path: Expr, allow: bool) -> Result<Expr, ParserErr> {
        if !allow {
            let peek_1 = self.peek_at(1)?;
            let peek_2 = self.peek_at(2)?;
            if matches!(peek_1, Token::Name(_)) && peek_2 == Token::Punctuation(Punctuation::Colon) {
                return Err(self.gen_error(ErrorCode::ParseExprNotSupported { expr: "Struct Expression", loc: "for loop's source value" }));
            }

            return Ok(path);
        }

        let args = self.parse_comma_separated_closed(OpenCloseSymbol::Brace, Self::parse_struct_arg)?;

        if !allow {
            return Err(self.gen_error(ErrorCode::ParseExprNotSupported { expr: "Struct Expression", loc: "for loop's source value" }));
        }

        Ok(Expr::Struct(self.add_node(StructExpr {
            path,
            args,
        })))
    }

    fn parse_struct_arg(&mut self) -> Result<StructArg, ParserErr> {
        let peek = self.peek()?;
        match peek {
            Token::Name(_) => if self.peek_at(1)? == Token::Punctuation(Punctuation::Colon) {
                let name = self.consume_name()?;
                self.consume_punct(Punctuation::Colon);
                let expr = self.parse_expr(ExprParseMode::General)?;
                Ok(StructArg::Expr(name, expr))
            } else {
                let name = self.consume_name()?;
                Ok(StructArg::Name(name))
            },
            Token::Punctuation(Punctuation::DotDot) => {
                self.consume_single();
                let expr = self.parse_expr(ExprParseMode::General)?;
                Ok(StructArg::Complete(expr))
            },
            _ => Err(self.gen_error(ErrorCode::ParseUnexpectedFor{ found: peek, for_reason: "struct argument" }))
        }
    }

    fn parse_index_expr(&mut self, expr: Expr) -> Result<Expr, ParserErr> {
        self.begin_scope(OpenCloseSymbol::Bracket)?;
        let is_opt = self.try_consume(Token::Punctuation(Punctuation::Question));
        let index = self.parse_expr(ExprParseMode::AllowComma)?;

        Ok(Expr::Index(self.add_node(IndexExpr {
            is_opt,
            expr,
            index,
        })))
    }

    fn parse_tuple_index(&mut self, expr: Expr) -> Result<Expr, ParserErr> {
        self.consume_punct(Punctuation::Dot);
        let index = self.consume_lit()?;
        Ok(Expr::TupleIndex(self.add_node(TupleIndexExpr {
            expr,
            index,
        })))
    }

    fn parse_call_expression(&mut self, expr: Expr) -> Result<Expr, ParserErr> {
        let args = self.parse_comma_separated_closed(OpenCloseSymbol::Paren, Self::parse_func_arg)?;
        Ok(Expr::FnCall(self.add_node(FnCallExpr{
            expr,
            args
        })))
    }

    fn parse_func_arg(&mut self) -> Result<FnArg, ParserErr> {
        if matches!(self.peek()?, Token::Name(_)) && self.peek_at(1)? == Token::Punctuation(Punctuation::Colon) {
            let label = self.consume_name()?;
            self.consume_punct(Punctuation::Colon);
            let expr = self.parse_expr(ExprParseMode::General)?;
            Ok(FnArg::Labeled { label, expr })
        } else {
            let expr = self.parse_expr(ExprParseMode::General)?;
            Ok(FnArg::Expr(expr))
        }
    }

    fn parse_field_access_or_method_expr(&mut self, expr: Expr) -> Result<Expr, ParserErr> {
        let is_propagating = if self.try_consume(Token::Punctuation(Punctuation::QuestionDot)) {
            true
        } else {
            self.consume_punct(Punctuation::Dot)?;
            false
        };

        let field = self.consume_name()?;

        let gen_args = self.parse_generic_args(true)?;
        if self.peek()? == Token::OpenSymbol(OpenCloseSymbol::Paren) {
            let args = self.parse_comma_separated_closed(OpenCloseSymbol::Paren, Self::parse_func_arg)?;
            Ok(Expr::Method(self.add_node(MethodCallExpr {
                receiver: expr,
                method: field,
                gen_args,
                args,
                is_propagating,
            })))
        } else {
            Ok(Expr::FieldAccess(self.add_node(FieldAccessExpr {
                expr,
                gen_args,
                field,
                is_propagating,
            })))
        }
    }

    fn parse_closure_expr(&mut self) -> Result<Expr, ParserErr> {
        let is_moved = self.try_consume(Token::StrongKw(StrongKeyword::Move));
        self.consume_punct(Punctuation::Or)?;
        let params = self.parse_comma_separated_end(Punctuation::Comma, Token::Punctuation(Punctuation::Or), Self::parse_function_param)?;
        self.consume_punct(Punctuation::Or)?;

        let ret = if self.try_consume(Token::Punctuation(Punctuation::SingleArrowR)) {
            Some(self.parse_func_return()?)
        } else {
            None
        };

        let body = self.parse_expr(ExprParseMode::General)?;

        Ok(Expr::Closure(self.add_node(ClosureExpr {
            is_moved,
            params,
            ret,
            body,
        })))
    }

    fn parse_if_expr(&mut self) -> Result<Expr, ParserErr> {
        self.consume_strong_kw(StrongKeyword::If)?;
        let cond = self.parse_expr(ExprParseMode::AllowLet)?;
        
        self.push_meta_frame();
        let body = self.parse_block_expr(None)?;

        let else_body = if self.try_consume(Token::StrongKw(StrongKeyword::Else)) {
            if self.peek()? == Token::StrongKw(StrongKeyword::If) {
                Some(self.parse_if_expr()?)
            } else {
                self.push_meta_frame();
                Some(Expr::Block(self.parse_block_expr(None)?))
            }
        } else {
            None
        };

        Ok(Expr::If(self.add_node(IfExpr {
            cond,
            body,
            else_body,
        })))
    }

    fn parse_let_binding_expr(&mut self) -> Result<Expr, ParserErr> {
        self.consume_strong_kw(StrongKeyword::Let)?;
        self.push_meta_frame();
        let pattern = self.parse_pattern_no_top_alternative()?;
        self.consume_punct(Punctuation::Equals)?;
        let scrutinee = self.parse_expr(ExprParseMode::Scrutinee)?;
        Ok(Expr::Let(self.add_node(LetBindingExpr {
            pattern,
            scrutinee,
        })))
    }

    fn parse_label(&mut self) -> Result<NameId, ParserErr> {
        self.consume_punct(Punctuation::Colon)?;
        let label = self.consume_name()?;
        self.consume_punct(Punctuation::Colon)?;
        Ok(label)
    }

    fn parse_loop_expr(&mut self, label: Option<NameId>) -> Result<Expr, ParserErr> {
        self.consume_strong_kw(StrongKeyword::Loop)?;
        let body = self.parse_block_expr(None)?;
        Ok(Expr::Loop(self.add_node(LoopExpr {
            label,
            body,
        })))
    }

    fn parse_while_expr(&mut self, label: Option<NameId>) -> Result<Expr, ParserErr> {
        self.consume_strong_kw(StrongKeyword::While)?;
        let cond = self.parse_expr(ExprParseMode::AllowLet)?;
        let inc = if self.try_consume(Token::Punctuation(Punctuation::Semicolon)) {
            Some(self.parse_expr(ExprParseMode::General)?)
        } else {
            None
        };
        
        self.push_meta_frame();
        let body = self.parse_block_expr(None)?;
        let else_body = if self.try_consume(Token::StrongKw(StrongKeyword::Else)) {
            self.push_meta_frame();
            let else_body = self.parse_block_expr(None)?;
            Some(else_body)
        } else {
            None
        };

        Ok(Expr::While(self.add_node(WhileExpr {
            label,
            cond,
            inc,
            body,
            else_body,
        })))
    }

    fn parse_do_while_expr(&mut self, label: Option<NameId>) -> Result<Expr, ParserErr> {
        self.consume_strong_kw(StrongKeyword::Do)?;
        let body = self.parse_block_expr(None)?;
        self.consume_strong_kw(StrongKeyword::While)?;
        let cond = self.parse_expr(ExprParseMode::General)?;
        Ok(Expr::DoWhile(self.add_node(DoWhileExpr {
            label,
            body,
            cond,
        })))
    }

    fn parse_for_expr(&mut self, label: Option<NameId>) -> Result<Expr, ParserErr> {
        self.consume_strong_kw(StrongKeyword::For)?;
        let pattern = self.parse_pattern()?;
        self.consume_strong_kw(StrongKeyword::In)?;
        let src = self.parse_expr(ExprParseMode::NoStructLit)?;
        
        self.push_meta_frame();
        let body = self.parse_block_expr(None)?;
        let else_body = if self.try_consume(Token::StrongKw(StrongKeyword::Else)) {
            self.push_meta_frame();
            Some(self.parse_block_expr(None)?)
        } else {
            None
        };

        Ok(Expr::For(self.add_node(ForExpr {
            label,
            pattern,
            src,
            body,
            else_body,
        })))
    }

    fn parse_match_expr(&mut self, label: Option<NameId>) -> Result<Expr, ParserErr> {
        self.consume_strong_kw(StrongKeyword::Match)?;
        let scrutinee = self.parse_expr(ExprParseMode::NoStructLit)?;

        self.begin_scope(OpenCloseSymbol::Brace)?;
        let mut branches = Vec::new();
        while !self.try_end_scope() {
            let branch = self.parse_match_branch()?;
            let needs_comma = !branch.body.has_block();
            branches.push(branch);

            if needs_comma {   
                if !self.try_consume(Token::Punctuation(Punctuation::Comma)) {
                    self.end_scope()?;
                    break;
                }
            } else {
                self.try_consume(Token::Punctuation(Punctuation::Comma));
            }
        }
        
        Ok(Expr::Match(self.add_node(MatchExpr {
            label,
            scrutinee,
            branches,
        })))
    }

    fn parse_match_branch(&mut self) -> Result<MatchBranch, ParserErr> {
        let label = if self.peek()? == Token::Punctuation(Punctuation::Colon) {
            Some(self.parse_label()?)
        } else {
            None
        };
        let pattern = self.parse_pattern()?;
        let guard = if self.try_consume(Token::StrongKw(StrongKeyword::If)) {
            Some(self.parse_expr(ExprParseMode::NoStructLit)?)
        } else {
            None
        };
        self.consume_punct(Punctuation::DoubleArrow)?;
        let body = self.parse_expr(ExprParseMode::General)?;

        Ok(MatchBranch {
            label,
            pattern,
            guard,
            body,
        })
    }
    
    fn parse_break_expr(&mut self) -> Result<Expr, ParserErr> {
        self.consume_strong_kw(StrongKeyword::Break);
        let label = if self.peek()? == Token::Punctuation(Punctuation::Colon) {
            Some(self.parse_label()?)
        } else {
            None
        };
        let value = if self.peek()? != Token::Punctuation(Punctuation::Semicolon) {
            Some(self.parse_expr(ExprParseMode::AllowComma)?)
        } else {
            None
        };

        Ok(Expr::Break(self.add_node(BreakExpr {
            label,
            value,
        })))
    }
    
    fn parse_continue_expr(&mut self) -> Result<Expr, ParserErr> {
        self.consume_strong_kw(StrongKeyword::Continue);
        let label = if self.peek()? == Token::Punctuation(Punctuation::Colon) {
            Some(self.parse_label()?)
        } else {
            None
        };

        Ok(Expr::Continue(self.add_node(ContinueExpr {
            label,
        })))
    }
    
    fn parse_fallthrough_expr(&mut self) -> Result<Expr, ParserErr> {
        self.consume_strong_kw(StrongKeyword::Fallthrough);
        let label = if self.peek()? == Token::Punctuation(Punctuation::Colon) {
            Some(self.parse_label()?)
        } else {
            None
        };

        Ok(Expr::Fallthrough(self.add_node(FallthroughExpr {
            label,
        })))
    }
    
    fn parse_return_expr(&mut self) -> Result<Expr, ParserErr> {
        self.consume_strong_kw(StrongKeyword::Return);
        let value = if self.peek()? != Token::Punctuation(Punctuation::Semicolon) {
            Some(self.parse_expr(ExprParseMode::AllowComma)?)
        } else {
            None
        };

        Ok(Expr::Return(self.add_node(ReturnExpr {
            value,
        })))
    }

    fn parse_throw_expr(&mut self) -> Result<Expr, ParserErr> {
        self.consume_strong_kw(StrongKeyword::Throw)?;
        let expr = self.parse_expr(ExprParseMode::General)?;
        Ok(Expr::Throw(self.add_node(ThrowExpr {
            expr,
        })))
    }

    fn parse_comma_expr(&mut self, first: Expr) -> Result<Expr, ParserErr> {
        self.consume_punct(Punctuation::Comma);

        let mut exprs = vec![first];
        loop {
            exprs.push(self.parse_expr(ExprParseMode::General)?);
            if !self.try_consume(Token::Punctuation(Punctuation::Comma)) {
                break;
            }
        }

        Ok(Expr::Comma(self.add_node(CommaExpr {
            exprs,
        })))
    }

    fn parse_when_expr(&mut self) -> Result<Expr, ParserErr> {
        self.consume_strong_kw(StrongKeyword::When)?;
        let cond = self.parse_expr(ExprParseMode::NoStructLit)?;

        self.push_meta_frame();
        let body = self.parse_block_expr(None)?;

        let else_body = if self.try_consume(Token::StrongKw(StrongKeyword::Else)) {
            if self.peek()? == Token::StrongKw(StrongKeyword::If) {
                Some(self.parse_if_expr()?)
            } else {  
                self.push_meta_frame();
                Some(Expr::Block(self.parse_block_expr(None)?))
            }
        } else {
            None
        };

        Ok(Expr::When(self.add_node(WhenExpr {
            cond,
            body,
            else_body,
        })))
    }

// =============================================================================================================================

    fn parse_pattern(&mut self) -> Result<Pattern, ParserErr> {
        self.push_meta_frame();
        
        let mut patterns = self.parse_punct_separated(Punctuation::Or, Self::parse_pattern_no_top_alternative)?;
        if patterns.len() == 1 {
            Ok(patterns.pop().unwrap())
        } else {
            Ok(Pattern::Alternative(self.add_node(AlternativePattern {
                patterns
            })))
        }
    }

    fn parse_pattern_no_top_alternative(&mut self) -> Result<Pattern, ParserErr> {
        let peek = self.peek()?;
        let pattern = match peek {
            Token::StrongKw(StrongKeyword::True)          |
            Token::StrongKw(StrongKeyword::False)         |
            Token::Literal(_)                             => self.parse_literal_pattern()?,
            Token::StrongKw(StrongKeyword::Ref |
                            StrongKeyword::Mut)           => self.parse_identifier_pattern()?,
            Token::Underscore                             => {
                self.consume_single();
                Pattern::Wildcard
            },
            Token::Punctuation(Punctuation::DotDot)       => self.parse_dotdot_pattern()?,
            Token::Punctuation(Punctuation::DotDotEquals) => self.parse_inclusive_to_pattern()?,
            Token::Punctuation(Punctuation::Ampersand)    => self.parse_reference_pattern()?,
            Token::OpenSymbol(OpenCloseSymbol::Paren)     => self.parse_tuple_like_pattern()?,
            Token::OpenSymbol(OpenCloseSymbol::Bracket)   => self.parse_slice_pattern()?,
            Token::Punctuation(Punctuation::Dot)          => if self.try_peek_at(1) == Some(Token::OpenSymbol(OpenCloseSymbol::Brace)) {
                self.parse_inferred_struct_pattern()?
            } else {
                self.parse_enum_member_pattern()?
            },
            Token::StrongKw(StrongKeyword::Is)            => self.parse_type_check_pattern()?,
            _                                             => self.parse_path_like_pattern()?,
        };

        let peek = self.peek()?;
        match peek {
            Token::Punctuation(Punctuation::DotDot)       => self.parse_range_pattern(pattern),
            Token::Punctuation(Punctuation::DotDotEquals) => self.parse_inclusive_range_pattern(pattern),
            _                                             => Ok(pattern)
        }
    }

    fn parse_literal_pattern(&mut self) -> Result<Pattern, ParserErr> {
        let lit = self.parse_literal_expr_node()?;
        Ok(Pattern::Literal(self.add_node(LiteralPattern {
            literal: lit.literal,
            lit_op: lit.lit_op,
        })))
    }

    fn pattern_available(&mut self) -> bool {
        let Some(peek) = self.try_peek() else { return false; };
        match peek {
            Token::Literal(_) |
            Token::Name(_) |
            Token::Punctuation(Punctuation::Dot) => true,
            _ => false,
        }
    }

    fn parse_identifier_pattern(&mut self) -> Result<Pattern, ParserErr> {
        let is_ref = self.try_consume(Token::StrongKw(StrongKeyword::Ref));
        let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));

        let name = self.consume_name()?;

        let bound = if self.try_consume(Token::Punctuation(Punctuation::At)) {
            Some(self.parse_pattern()?)
        } else {
            None
        };

        Ok(Pattern::Identifier(self.add_node(IdentifierPattern {
            is_ref,
            is_mut,
            name,
            bound,
        })))
    }

    fn parse_dotdot_pattern(&mut self) -> Result<Pattern, ParserErr> {
        self.consume_punct(Punctuation::DotDot)?;
        if self.pattern_available() {
            self.push_meta_frame();
            let end = self.parse_pattern_no_top_alternative()?;
            Ok(Pattern::Range(self.add_node(RangePattern::To { end })))
        } else {
            Ok(Pattern::Rest)
        }
    }

    fn parse_inclusive_to_pattern(&mut self) -> Result<Pattern, ParserErr> {
        self.consume_punct(Punctuation::DotDotEquals)?;
        self.push_meta_frame();
        let end = self.parse_pattern_no_top_alternative()?;
        Ok(Pattern::Range(self.add_node(RangePattern::InclusiveTo { end })))
    }

    fn parse_path_like_pattern(&mut self) -> Result<Pattern, ParserErr> {
        let peek = self.peek()?;
        let peek_1 = self.peek_at(1)?;
        if peek_1 != Token::Punctuation(Punctuation::Dot) &&
            peek_1 != Token::OpenSymbol(OpenCloseSymbol::Paren) &&
            peek_1 != Token::OpenSymbol(OpenCloseSymbol::Brace)
        {
            let name = self.consume_name()?;

            let bound = if self.try_consume(Token::Punctuation(Punctuation::At)) {
                self.push_meta_frame();
                Some(self.parse_pattern_no_top_alternative()?)
            } else {
                None
            };

            return Ok(Pattern::Identifier(self.add_node(IdentifierPattern {
                is_ref: false,
                is_mut: false,
                name,
                bound,
            })));
        }

        let path = self.parse_expr_path()?;
        if let Token::OpenSymbol(sym) = self.peek()? {
            match sym {
                OpenCloseSymbol::Paren => {
                    let patterns = self.parse_comma_separated_closed(sym, Self::parse_pattern)?;
                    Ok(Pattern::Tuple(self.add_node(TuplePattern { patterns })))
                },
                OpenCloseSymbol::Brace => {
                    let fields = self.parse_comma_separated_closed(sym, Self::parse_struct_pattern_field)?;
                    Ok(Pattern::Struct(self.add_node(StructPattern::Path{ path, fields })))
                },
                _ => Err(self.gen_error(ErrorCode::ParseUnexpectedFor{ found: Token::OpenSymbol(sym), for_reason: "pattern" })),
            }

        } else {
            Ok(Pattern::Path(self.add_node(PathPattern{ path })))
        }
    }

    fn parse_struct_pattern_field(&mut self) -> Result<StructPatternField, ParserErr> {
        let peek = self.peek()?;
        match peek {
            Token::Name(name) => {
                self.consume_single();
                if self.try_consume(Token::Punctuation(Punctuation::Colon)) {
                    let pattern = self.parse_pattern()?;
                    Ok(StructPatternField::Named { name, pattern })
                } else {
                    Ok(StructPatternField::Iden { is_ref: false, is_mut: false, iden: name })
                }
            },
            Token::Literal(lit_id) => {
                self.consume_single();
                self.consume_punct(Punctuation::Colon)?;
                let pattern = self.parse_pattern()?;
                Ok(StructPatternField::TupleIndex { idx: lit_id, pattern })
            },
            Token::StrongKw(StrongKeyword::Mut) => {
                self.consume_single();
                let iden = self.consume_name()?;
                Ok(StructPatternField::Iden { is_ref: false, is_mut: true, iden })
            },
            Token::Punctuation(Punctuation::Ampersand) => {
                self.consume_single();
                let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
                let iden = self.consume_name()?;
                Ok(StructPatternField::Iden { is_ref: true, is_mut, iden })
            },
            Token::Punctuation(Punctuation::DotDot) => {
                self.consume_single();
                Ok(StructPatternField::Rest)
            }
            _ => Err(self.gen_error(ErrorCode::ParseUnexpectedFor { found: peek, for_reason: "struct pattern field" }))
        }
    }

    fn parse_inferred_struct_pattern(&mut self) -> Result<Pattern, ParserErr> {
        self.consume_punct(Punctuation::Dot)?;
        let fields = self.parse_comma_separated_closed(OpenCloseSymbol::Brace, Self::parse_struct_pattern_field)?;
        Ok(Pattern::Struct(self.add_node(StructPattern::Inferred { fields })))
    }

    fn parse_enum_member_pattern(&mut self) -> Result<Pattern, ParserErr> {
        self.consume_punct(Punctuation::Dot)?;
        let name = self.consume_name()?;
        Ok(Pattern::EnumMember(self.add_node(EnumMemberPattern {
            name,
        })))
    }

    fn parse_struct_pattern_elem(&mut self) -> Result<StructPatternField, ParserErr> {
        if self.try_consume(Token::Punctuation(Punctuation::DotDot)) {
            return Ok(StructPatternField::Rest);
        }

        match self.peek()? {
            Token::StrongKw(StrongKeyword::Ref | StrongKeyword::Mut) => {
                let is_ref = self.try_consume(Token::StrongKw(StrongKeyword::Ref));
                let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
                let iden = self.consume_name()?;
                Ok(StructPatternField::Iden { is_ref, is_mut, iden })
            }
            Token::Literal(lit_id) => {
                self.consume_single();
                self.consume_punct(Punctuation::Colon)?;
                let pattern = self.parse_pattern()?;

                Ok(StructPatternField::TupleIndex { idx: lit_id, pattern })
            },
            Token::Name(iden) => {
                self.consume_single();
                if !self.try_consume(Token::Punctuation(Punctuation::Colon)) {
                    Ok(StructPatternField::Iden { is_ref: false, is_mut: false, iden })
                } else {
                    let pattern = self.parse_pattern()?;
                    Ok(StructPatternField::Named { name: iden, pattern })
                }
            }
            _ => Err(self.gen_error(ErrorCode::ParseUnexpectedFor{ found: self.peek()?, for_reason: "struct pattern element" }))
        }
    }

    fn parse_reference_pattern(&mut self) -> Result<Pattern, ParserErr> {
        self.consume_punct(Punctuation::Ampersand)?;
        let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
        let pattern = self.parse_pattern()?;

        Ok(Pattern::Reference(self.add_node(ReferencePattern {
            is_mut,
            pattern,
        } )))
    }

    fn parse_tuple_like_pattern(&mut self) -> Result<Pattern, ParserErr> {
        let mut patterns = self.parse_comma_separated_closed(OpenCloseSymbol::Paren, Self::parse_pattern)?;
        if patterns.len() == 1 {
            Ok(Pattern::Grouped(self.add_node(GroupedPattern {
                pattern: patterns.pop().unwrap()
            })))
        } else {
            Ok(Pattern::Tuple(self.add_node(TuplePattern{
                patterns
            })))
        }
    }

    fn parse_slice_pattern(&mut self) -> Result<Pattern, ParserErr> {
        let patterns = self.parse_comma_separated_closed(OpenCloseSymbol::Bracket, Self::parse_pattern)?;
        Ok(Pattern::Slice(self.add_node(SlicePattern {patterns })))
    }

    fn parse_type_check_pattern(&mut self) -> Result<Pattern, ParserErr> {
        self.consume_strong_kw(StrongKeyword::Is)?;
        let ty = self.parse_type()?;
        Ok(Pattern::TypeCheck(self.add_node(TypeCheckPattern { ty })))
    }

    fn parse_range_pattern(&mut self, begin: Pattern) -> Result<Pattern, ParserErr> {
        self.consume_punct(Punctuation::DotDot)?;
        if self.pattern_available() {
            self.push_meta_frame();
            let end = self.parse_pattern_no_top_alternative()?;
            Ok(Pattern::Range(self.add_node(RangePattern::Exclusive { begin, end })))
        } else {
            Ok(Pattern::Range(self.add_node(RangePattern::From { begin })))
        }
    }
    
    fn parse_inclusive_range_pattern(&mut self, begin: Pattern) -> Result<Pattern, ParserErr> {
        self.consume_punct(Punctuation::DotDotEquals)?;
        let end = self.parse_pattern()?;
        Ok(Pattern::Range(self.add_node(RangePattern::Inclusive { begin, end })))
    }

// =============================================================================================================================

    fn parse_type(&mut self) -> Result<Type, ParserErr> {
        let peek = self.peek()?;
        match peek {
            Token::StrongKw(StrongKeyword::Dyn) => todo!(),
            Token::StrongKw(StrongKeyword::Impl) => todo!(),
            _ => self.parse_type_no_bounds()
        }
    }

    fn parse_type_no_bounds(&mut self) -> Result<Type, ParserErr> {
        self.push_meta_frame();
        
        let peek = self.peek()?;
        match peek {
            Token::OpenSymbol(OpenCloseSymbol::Paren) => self.parse_tuple_like_type(),
            Token::Punctuation(Punctuation::Exclaim) => {
                self.consume_single();
                Ok(Type::Never)
            },
            Token::OpenSymbol(OpenCloseSymbol::Bracket) => self.parse_slice_like_type(),
            Token::Punctuation(Punctuation::Caret)      => self.parse_pointer_type(),
            Token::Punctuation(Punctuation::Ampersand)  => self.parse_reference_type(),
            Token::Punctuation(Punctuation::Question)   => self.parse_optional_type(),
            Token::StrongKw(StrongKeyword::Unsafe       |
                StrongKeyword::Extern                   |
                StrongKeyword::Fn)                      => self.parse_fn_type(),
            Token::StrongKw(StrongKeyword::Enum)        => self.parse_enum_record_type(),
            Token::OpenSymbol(OpenCloseSymbol::Brace)   => self.parse_record_type(),
            Token::StrongKw(kw)                         => self.parse_type_from_strong_kw(kw),
            _                                           => self.parse_path_type(),
        }
    }

    fn parse_tuple_like_type(&mut self) -> Result<Type, ParserErr> {
        let mut types = self.parse_comma_separated_closed(OpenCloseSymbol::Paren, Self::parse_type)?;
        if types.is_empty() {
            Ok(Type::Unit)
        } else if types.len() == 1 {
            Ok(Type::Paren(self.add_node(ParenthesizedType {
                ty: types.pop().unwrap(),
            })))
        } else {
            Ok(Type::Tuple(self.add_node(TupleType {
                types,
            })))
        }
    }

    fn parse_type_from_strong_kw(&mut self, kw: StrongKeyword) -> Result<Type, ParserErr> {
        let ty = match kw {
            StrongKeyword::U8     => Type::Primitive(self.add_node(PrimitiveType::U8)),
            StrongKeyword::U16    => Type::Primitive(self.add_node(PrimitiveType::U16)),
            StrongKeyword::U32    => Type::Primitive(self.add_node(PrimitiveType::U32)),
            StrongKeyword::U64    => Type::Primitive(self.add_node(PrimitiveType::U64)),
            StrongKeyword::U128   => Type::Primitive(self.add_node(PrimitiveType::U128)),
            StrongKeyword::Usize  => Type::Primitive(self.add_node(PrimitiveType::Usize)),
            StrongKeyword::I8     => Type::Primitive(self.add_node(PrimitiveType::I8)),
            StrongKeyword::I16    => Type::Primitive(self.add_node(PrimitiveType::I16)),
            StrongKeyword::I32    => Type::Primitive(self.add_node(PrimitiveType::I32)),
            StrongKeyword::I64    => Type::Primitive(self.add_node(PrimitiveType::I64)),
            StrongKeyword::I128   => Type::Primitive(self.add_node(PrimitiveType::I128)),
            StrongKeyword::Isize  => Type::Primitive(self.add_node(PrimitiveType::Isize)),
            StrongKeyword::F16    => Type::Primitive(self.add_node(PrimitiveType::F16)),
            StrongKeyword::F32    => Type::Primitive(self.add_node(PrimitiveType::F32)),
            StrongKeyword::F64    => Type::Primitive(self.add_node(PrimitiveType::F64)),
            StrongKeyword::F128   => Type::Primitive(self.add_node(PrimitiveType::F128)),
            StrongKeyword::Bool   => Type::Primitive(self.add_node(PrimitiveType::Bool)),
            StrongKeyword::B8     => Type::Primitive(self.add_node(PrimitiveType::B8)),
            StrongKeyword::B16    => Type::Primitive(self.add_node(PrimitiveType::B16)),
            StrongKeyword::B32    => Type::Primitive(self.add_node(PrimitiveType::B32)),
            StrongKeyword::B64    => Type::Primitive(self.add_node(PrimitiveType::B64)),
            StrongKeyword::Char   => Type::Primitive(self.add_node(PrimitiveType::Char)),
            StrongKeyword::Char7  => Type::Primitive(self.add_node(PrimitiveType::Char7)),
            StrongKeyword::Char8  => Type::Primitive(self.add_node(PrimitiveType::Char8)),
            StrongKeyword::Char16 => Type::Primitive(self.add_node(PrimitiveType::Char16)),
            StrongKeyword::Char32 => Type::Primitive(self.add_node(PrimitiveType::Char32)),
            StrongKeyword::Str    => Type::StringSlice(self.add_node(StringSliceType::Str)),
            StrongKeyword::Str7   => Type::StringSlice(self.add_node(StringSliceType::Str7)),
            StrongKeyword::Str8   => Type::StringSlice(self.add_node(StringSliceType::Str8)),
            StrongKeyword::Str16  => Type::StringSlice(self.add_node(StringSliceType::Str16)),
            StrongKeyword::Str32  => Type::StringSlice(self.add_node(StringSliceType::Str32)),
            StrongKeyword::CStr   => Type::StringSlice(self.add_node(StringSliceType::CStr)),
            _ => {
                let peek = self.peek()?;
                return Err(self.gen_error(ErrorCode::ParseUnexpectedFor{ found: peek, for_reason: "type" }))
            },
        };

        self.consume_single();
        Ok(ty)
    }

    fn parse_path_type(&mut self) -> Result<Type, ParserErr> {
        let path = self.parse_type_path()?;
        Ok(Type::Path(self.add_node(PathType{ path })))
    }

    fn parse_slice_like_type(&mut self) -> Result<Type, ParserErr> {
        self.begin_scope(OpenCloseSymbol::Bracket)?;
        let peek = self.peek()?;
        match peek {
            Token::CloseSymbol(OpenCloseSymbol::Bracket) => {
                self.end_scope();
                let ty = self.parse_type_no_bounds()?;
                Ok(Type::Slice(self.add_node(SliceType { sentinel: None, ty })))
            },
            Token::Punctuation(Punctuation::Semicolon) => {
                self.consume_single();
                let sentinel = Some(self.parse_expr(ExprParseMode::General)?);
                self.end_scope()?;
                let ty = self.parse_type_no_bounds()?;
                Ok(Type::Slice(self.add_node(SliceType { sentinel, ty })))
            }
            Token::Punctuation(Punctuation::Caret) => {
                self.consume_single();
                let sentinel = if self.try_consume(Token::Punctuation(Punctuation::Semicolon)) {
                    Some(self.parse_expr(ExprParseMode::General)?)
                } else {
                    None
                };
                self.end_scope()?;
                let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
                let ty = self.parse_type_no_bounds()?;
                Ok(Type::Pointer(self.add_node(PointerType { is_multi: true, is_mut, sentinel, ty })))
            },
            _ => {
                let size = self.parse_expr(ExprParseMode::General)?;
                let sentinel = if self.try_consume(Token::Punctuation(Punctuation::Semicolon)) {
                    Some(self.parse_expr(ExprParseMode::General)?)
                } else {
                    None
                };
                self.end_scope()?;
                let ty = self.parse_type_no_bounds()?;
                Ok(Type::Array(self.add_node(ArrayType { size, sentinel, ty })))
            }
        }
    }

    fn parse_pointer_type(&mut self) -> Result<Type, ParserErr> {
        self.consume_punct(Punctuation::Caret)?;
        let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
        let ty = self.parse_type_no_bounds()?;
        Ok(Type::Pointer(self.add_node(PointerType { is_multi: false, is_mut, sentinel: None, ty })))
    }

    fn parse_reference_type(&mut self) -> Result<Type, ParserErr> {
        self.consume_punct(Punctuation::Ampersand)?;
        let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
        let ty = self.parse_type_no_bounds()?;
        Ok(Type::Ref(self.add_node(ReferenceType { is_mut, ty })))
    }

    fn parse_optional_type(&mut self) -> Result<Type, ParserErr> {
        self.consume_punct(Punctuation::Question)?;
        let ty = self.parse_type_no_bounds()?;
        Ok(Type::Optional(self.add_node(OptionalType { ty })))
    }

    fn parse_fn_type(&mut self) -> Result<Type, ParserErr> {
        let is_unsafe = self.try_consume(Token::StrongKw(StrongKeyword::Unsafe));
        let abi = if self.try_consume(Token::StrongKw(StrongKeyword::Extern)) {
            Some(self.consume_lit()?)
        } else {
            None
        };

        self.consume_strong_kw(StrongKeyword::Fn)?;
        let params = self.parse_comma_separated_closed(OpenCloseSymbol::Paren, Self::parse_fn_type_param)?;

        let return_ty = if self.try_consume(Token::Punctuation(Punctuation::SingleArrowR)) {
            Some(self.parse_type_no_bounds()?)
        } else {
            None
        };


        Ok(Type::Fn(self.add_node(FnType {
            is_unsafe,
            abi,
            params,
            return_ty,
        })))
    }

    fn parse_fn_type_param(&mut self) -> Result<(Vec<NameId>, Type), ParserErr> {
        let start_idx = self.token_idx;
        let peek = self.peek_at(1)?;

        // Try to parse names, if that doesn't work, it's a series of types
        let mut names = Vec::new();
        if peek == Token::Punctuation(Punctuation::Comma) || peek == Token::Punctuation(Punctuation::Colon) {
            names = match self.parse_punct_separated(Punctuation::Comma, Self::consume_name) {
                Ok(mut names) =>{
                    if !self.try_consume(Token::Punctuation(Punctuation::Colon)) {
                        names.clear();
                        self.token_idx = self.token_idx;        
                    }
                    names
                },
                Err(_) => {
                    names.clear();
                    self.token_idx = start_idx;
                    Vec::new()
                },
            }
        }

        let ty = self.parse_type_no_bounds()?;
        Ok((names, ty))
    }

    fn parse_record_type(&mut self) -> Result<Type, ParserErr> {
        let fields = self.parse_comma_separated_closed(OpenCloseSymbol::Brace, Self::parse_struct_field)?;
        Ok(Type::Record(self.add_node(RecordType {
            fields
        })))
    }

    fn parse_enum_record_type(&mut self) -> Result<Type, ParserErr> {
        self.consume_strong_kw(StrongKeyword::Enum);
        let variants = self.parse_comma_separated_closed(OpenCloseSymbol::Brace, Self::parse_enum_variant)?;
        Ok(Type::EnumRecord(self.add_node(EnumRecordType {
            variants
        })))
    }

// =============================================================================================================================

    fn parse_generic_params(&mut self) -> Result<Option<AstNodeRef<GenericParams>>, ParserErr> {
        // TODO
        Ok(None)
    }

    fn parse_generic_args(&mut self, start_with_dot: bool) -> Result<Option<AstNodeRef<GenericArgs>>, ParserErr> {
        // TODO
        Ok(None)
    }

    fn parse_where_clause(&mut self) -> Result<Option<AstNodeRef<WhereClause>>, ParserErr> {
        // TODO
        Ok(None)
    }

    fn parse_trait_bounds(&mut self) -> Result<AstNodeRef<TraitBounds>, ParserErr> {
        todo!()
    }

// =============================================================================================================================

    fn parse_visibility(&mut self) -> Result<Option<AstNodeRef<Visibility>>, ParserErr> {
        if !self.try_consume(Token::StrongKw(StrongKeyword::Pub)) {
            return Ok(None);
        }

        self.push_meta_frame();
        if self.try_begin_scope(OpenCloseSymbol::Paren) {
            let vis = match self.try_peek().unwrap() {
                Token::WeakKw(WeakKeyword::Package) => {
                    self.consume_single();
                    Visibility::Package
                },
                Token::WeakKw(WeakKeyword::Lib) => {
                    self.consume_single();
                    Visibility::Lib
                },
                Token::WeakKw(WeakKeyword::Super) => {
                    self.consume_single();
                    Visibility::Super
                },
                _ => {
                    let path = self.parse_simple_path(true)?;
                    Visibility::Path(path)
                }
            };

            self.end_scope()?;
            Ok(Some(self.add_node(vis)))
        } else {
            Ok(Some(self.add_node(Visibility::Pub)))
        }
    }

// =============================================================================================================================

    fn parse_attributes(&mut self) -> Result<Vec<AstNodeRef<Attribute>>, ParserErr> {
        let mut attrs = Vec::new();

        loop {
            self.push_meta_frame();
            
            let is_mod = if self.try_consume(Token::Punctuation(Punctuation::At)) {
                false
            } else if self.try_consume(Token::Punctuation(Punctuation::AtExclaim)) {
                true
            } else {
                self.pop_meta_frame();
                break;
            };

            let metas = self.parse_comma_separated_closed(OpenCloseSymbol::Bracket, Self::parse_attrib_meta)?;
            let attr = self.add_node(Attribute {
                is_mod,
                metas,
            });
            attrs.push(attr);
        }
        Ok(attrs)
    }

    fn parse_attrib_meta(&mut self) -> Result<AttribMeta, ParserErr> {
        if matches!(self.peek()?, Token::Name(_)) {
            let path = self.parse_simple_path(false)?;
            if self.peek()? == Token::Punctuation(Punctuation::Equals) {
                self.consume_punct(Punctuation::Equals)?;
                let expr = self.parse_expr(ExprParseMode::General)?;
                Ok(AttribMeta::Assign { path, expr })
            } else if self.peek()? == Token::OpenSymbol(OpenCloseSymbol::Paren) {
                let metas = self.parse_comma_separated_closed(OpenCloseSymbol::Paren, Self::parse_attrib_meta)?;
                Ok(AttribMeta::Meta { path, metas })
            } else {
                Ok(AttribMeta::Simple { path })
            }
        } else {
            let expr = self.parse_expr(ExprParseMode::General)?;
            Ok(AttribMeta::Expr { expr })
        }
    }

// =============================================================================================================================

    fn parse_contract(&mut self) -> Result<AstNodeRef<Contract>, ParserErr> {
        todo!()
    }

// =============================================================================================================================

    /// Parse comma separated values ending with with a CloseSymbol
    fn parse_comma_separated_closed<T, F>(&mut self, open_close: OpenCloseSymbol, mut parse_single: F) -> Result<Vec<T>, ParserErr> where
        F: FnMut(&mut Self) -> Result<T, ParserErr>
    {
        self.begin_scope(open_close)?;
        let mut values = Vec::new();
        while !self.try_end_scope() {
            values.push(parse_single(self)?);
            if !self.try_consume(Token::Punctuation(Punctuation::Comma)) {
                self.end_scope()?;
                break;
            }
        }
        Ok(values)
    }

    fn parse_punct_separated<T, F>(&mut self, separator: Punctuation, mut parse_single: F) -> Result<Vec<T>, ParserErr> where
        F: FnMut(&mut Self) -> Result<T, ParserErr>
    {
        let mut values = Vec::new();
        loop {
            values.push(parse_single(self)?);
            if !self.try_consume(Token::Punctuation(separator)) {
                break;
            }
        }
        Ok(values)
    }

    fn parse_comma_separated_end<T, F>(&mut self, separator: Punctuation, end: Token, mut parse_single: F) -> Result<Vec<T>, ParserErr> where
        F: FnMut(&mut Self) -> Result<T, ParserErr>
    {
        let mut values = Vec::new();
        while self.peek()? != end {
            values.push(parse_single(self)?);
            if !self.try_consume(Token::Punctuation(separator)) {
                break;
            }
        }
        Ok(values)
    }
}