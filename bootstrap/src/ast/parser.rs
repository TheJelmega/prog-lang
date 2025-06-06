#![allow(unused)]

use std::{
    fmt,
    marker::PhantomData,
    path::{self, Path},
    result
};

use crate::{
    ast::*, common::{NameId, NameTable, Span, SpanRegistry}, error_warning::ParseErrorCode, lexer::{OpenCloseSymbol, Punctuation, PunctuationId, StrongKeyword, Token, TokenMetadata, TokenStore, WeakKeyword}, literals::LiteralId
};

use super::*;

pub struct ParserErr {
    pub err:     ParseErrorCode,
    pub tok_idx: usize,
}


impl fmt::Display for ParserErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.err)
    }
}


#[derive(Clone, Copy)]
pub struct ParserFrame {
    span:  SpanId,
    token_id: u32,
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

    spans:          &'a mut SpanRegistry,

    frames:         Vec<ParserFrame>,
    last_frame:     ParserFrame,
    scope_stack:    Vec<OpenCloseSymbol>,

    names:          &'a NameTable,
    pub ast:        Ast,
}

impl<'a> Parser<'a> {
    pub fn new(token_store: &'a TokenStore, names: &'a NameTable, spans: &'a mut SpanRegistry) -> Self {
        Self {
            token_store,
            token_idx: 0,

            spans,

            frames: Vec::new(),
            last_frame: ParserFrame{ token_id: 0, span: SpanId::INVALID },
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
            Err(self.gen_error(ParseErrorCode::NotEnoughTokens))
        }
    }

    fn peek_at(&self, offset: usize) -> Result<Token, ParserErr> {
        if self.token_idx + offset < self.token_store.tokens.len() {
            Ok(self.token_store.tokens[self.token_idx + offset])
        } else {
            Err(self.gen_error(ParseErrorCode::NotEnoughTokens))
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

    fn consume_single(&mut self) -> (Token, SpanId) {
        let token_idx = self.token_idx;
        self.token_idx += 1;
        let span = self.token_store.metadata[token_idx].span_id;
        let tok = self.token_store.tokens[token_idx];
        (tok, span)
    }

    fn consume(&mut self, expected: Token) -> Result<(), ParserErr> {
        let peek = self.peek()?;
        if peek == expected {
            self.consume_single();
            Ok(())
        } else {
            Err(self.gen_error(ParseErrorCode::FoundButExpected{ found: peek, expected }))
        }
    }

    fn name_from_tok(&self, token: Token) -> Option<NameId> {
        match token {
            Token::Name(name_id) => Some(name_id),
            Token::WeakKw(kw)    => Some(self.token_store.get_name_from_weak_keyword(kw)),
            Token::StrongKw(kw)  => self.token_store.get_name_from_prim_ty(kw),
            _ => None
        }
    }

    fn try_consume_name(&mut self) -> Option<NameId> {
        let peek = match self.try_peek() {
            Some(peek) => peek,
            None       => return None,
        };

        let name = self.name_from_tok(peek);
        if name.is_some() {
            self.consume_single();
        }
        name
    }

    fn consume_name(&mut self) -> Result<NameId, ParserErr> {
        let peek = self.peek()?;
        match self.name_from_tok(peek) {
            Some(name) => {
                self.consume_single();
                Ok(name)
            },
            None => Err(self.gen_error(ParseErrorCode::FoundButExpected{ found: peek, expected: Token::Name(NameId::INVALID) })),
        }
    }

    fn consume_name_and_span(&mut self) -> Result<(NameId, SpanId), ParserErr> {
        let token = self.consume_name()?;
        let span = self.token_store.metadata[self.token_idx - 1].span_id;
        Ok((token, span))
    }

    fn consume_lit(&mut self) -> Result<LiteralId, ParserErr> {
        let peek = self.peek()?;
        if let Token::Literal(lit_id) = peek {
            self.consume_single();
            Ok(lit_id)
        } else {
            Err(self.gen_error(ParseErrorCode::FoundButExpected{ found: peek, expected: Token::Literal(LiteralId::INVALID) }))
        }
    }

    fn consume_any_punct(&mut self) -> Result<Punctuation, ParserErr> {
        let peek = self.peek()?;
        if let Token::Punctuation(punct) = peek {
            self.consume_single();
            Ok(punct)
        } else {
            Err(self.gen_error(ParseErrorCode::FoundButExpected{ found: peek, expected: Token::Punctuation(Punctuation::Custom(PunctuationId::INVALID)) }))
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

    fn gen_error(&self, err: ParseErrorCode) -> ParserErr {
        ParserErr {
            err,
            tok_idx: self.token_idx,
        }
    }

    fn push_meta_frame(&mut self) {
        let token_meta = &self.token_store.metadata[self.token_idx];

        self.frames.push(ParserFrame {
            span:  token_meta.span_id,
            token_id: self.token_idx as u32,
        })
    }

    fn push_last_frame(&mut self) {
        self.frames.push(self.last_frame);
    }

    fn pop_meta_frame(&mut self) -> Option<ParserFrame> {
        self.frames.pop()
    }

    fn add_node<T: AstNode + AstNodeParseHelper + 'static>(&mut self, node: T) -> AstNodeRef<T> {
        let meta = if let Some(frame) = self.pop_meta_frame() {
            self.last_frame = frame;
            AstNodeMeta {
                span: frame.span,
                first_tok: frame.token_id,
                last_tok: self.token_idx as u32,
            }
        } else {
            self.last_frame = ParserFrame{ token_id: 0, span: SpanId::INVALID };
            AstNodeMeta {
                span: SpanId::INVALID,
                first_tok: 0,
                last_tok: 0, 
            }  
        };
        self.ast.add_node(node, meta)
    }

    pub fn get_cur_span(&self) -> SpanId {
        self.token_store.metadata[self.token_idx].span_id
    }

    pub fn get_span_to_current(&mut self, begin: SpanId) -> SpanId {
        let end = self.token_store.metadata[self.token_idx - 1].span_id;
        self.spans.combine_spans(begin, end)
    }

// =============================================================================================================================

    fn parse_simple_path(&mut self, only_allow_none_start: bool) -> Result<AstNodeRef<SimplePath>, ParserErr> {
        self.push_meta_frame();

        let start = self.parse_simple_path_start(only_allow_none_start)?;
        let names = self.parse_punct_separated(Punctuation::Dot, Self::consume_name_and_span)?;

        let begin = start.as_ref().map_or_else(|| names.first().map_or(SpanId::INVALID, |(_, span)| *span), |start| start.span);
        let span = self.get_span_to_current(begin);

        Ok(self.add_node(SimplePath {
            span,
            node_id: NodeId::default(),
            start,
            names
        }))
    }

    fn parse_simple_path_start(&mut self, only_allow_none_start: bool) -> Result<Option<SimplePathStart>, ParserErr> {
        let tok = self.peek()?;
        match tok {
            Token::Name(name_id)                     => {
                Ok(None)
            },
            Token::Punctuation(Punctuation::Dot)     => if only_allow_none_start {
                Err(self.gen_error(ParseErrorCode::InvalidPathStart { found: tok, reason: "inferred simple paths are not allowed" }))
            } else {
                let (_, span) = self.consume_single();
                Ok(Some(SimplePathStart {
                    span,
                    kind: SimplePathStartKind::Inferred,
                }))
            },
            Token::WeakKw(WeakKeyword::Super)        => if only_allow_none_start {
                Err(self.gen_error(ParseErrorCode::InvalidPathStart { found: tok, reason: "'super' relative paths are not allowed" }))
            } else {
                let (_, span) = self.consume_single();
                Ok(Some(SimplePathStart {
                    span,
                    kind: SimplePathStartKind::Super,
                }))
            },
            Token::StrongKw(StrongKeyword::SelfName) => if only_allow_none_start {
                Err(self.gen_error(ParseErrorCode::InvalidPathStart { found: tok, reason: "'self' relative paths are not allowed" }))
            } else {
                let (_, span) = self.consume_single();
                Ok(Some(SimplePathStart {
                    span,
                    kind: SimplePathStartKind::SelfPath,
                }))
            },
            _                                        => Err(self.gen_error(ParseErrorCode::InvalidPathStart{ found: tok, reason: "" }))
        }
    }

    fn parse_identifier(&mut self, dot_generics: bool) -> Result<Identifier, ParserErr> {
        let begin = self.get_cur_span();
        let name = if self.try_begin_scope(OpenCloseSymbol::Paren) {
            let begin = self.get_cur_span();
            let (path, Some((name, name_span))) = self.parse_raw_trait_path(true)? else { unreachable!() };
            let path = self.add_node(path);

            let span = self.get_span_to_current(begin);
            IdenName::Disambig{
                span,
                trait_path: path,
                name,
                name_span,
            }
        } else {
            let (name, span) = self.consume_name_and_span()?;
            IdenName::Name(name, span)
        };

        let gen_args = self.parse_generic_args(dot_generics)?;
        let span = self.get_span_to_current(begin);
        Ok(Identifier { span, name, gen_args })
    }

    fn parse_path_start(&mut self) -> Result<PathStart, ParserErr> {
        match self.peek()? {
            Token::StrongKw(StrongKeyword::SelfTy) => {
                let span = self.get_cur_span();
                self.consume_single();
                Ok(PathStart::SelfTy(span))
            },
            Token::Punctuation(Punctuation::Dot) => {
                let span = self.get_cur_span();
                self.consume_single();
                Ok(PathStart::Inferred(span))
            },
            Token::OpenSymbol(OpenCloseSymbol::Paren) => {
                self.begin_scope(OpenCloseSymbol::Paren)?;
                self.consume_punct(Punctuation::Colon)?;
                let ty = self.parse_type()?;
                self.consume_punct(Punctuation::Colon)?;
                self.end_scope();
                Ok(PathStart::Typed(ty))
            },
            Token::StrongKw(kw) if kw.is_primitive_type() => {
                let ty = self.parse_type()?;
                Ok(PathStart::Typed(ty))
            },
            _ => Ok(PathStart::None),
        }
    }


    fn parse_type_path(&mut self) -> Result<AstNodeRef<TypePath>, ParserErr> {
        self.push_meta_frame();
        let begin = self.last_frame.span;

        let start = self.parse_path_start()?;
        let mut idens = if !matches!(start, PathStart::SelfTy(_)) || self.try_consume(Token::Punctuation(Punctuation::Dot)) {
            self.parse_punct_separated(Punctuation::Dot, |parser| Self::parse_identifier(parser, false))?
        } else {
            Vec::new()
        };

        let span = self.get_span_to_current(begin);
        Ok(self.add_node(TypePath{
            span,
            node_id: NodeId::INVALID,
            start,
            idens,
        }))
    }

    fn parse_expr_path(&mut self) -> Result<AstNodeRef<ExprPath>, ParserErr> {
        self.push_meta_frame();
        let begin = self.last_frame.span;

        let start = self.parse_path_start()?;
        let mut idens = self.parse_punct_separated(Punctuation::Dot, |parser| Self::parse_identifier(parser, false))?;

        let span = self.get_span_to_current(begin);
        Ok(self.add_node(ExprPath{
            span,
            node_id: NodeId::default(),
            start,
            idens
        }))
    }

    fn parse_trait_path(&mut self) -> Result<AstNodeRef<TraitPath>, ParserErr> {
        let (path, _) = self.parse_raw_trait_path(false)?;
        Ok(self.add_node(path))
    }

    fn parse_raw_trait_path(&mut self, seperate_last_iden: bool) -> Result<(TraitPath, Option<(NameId, SpanId)>), ParserErr> {
        self.push_meta_frame();
        let begin = self.last_frame.span;

        let start = self.parse_path_start()?;
        let mut prev_span_end = begin;
        let mut idens = self.parse_punct_separated(Punctuation::Dot, |parser| {
            prev_span_end = parser.get_cur_span();
            Self::parse_identifier(parser, false)
        })?;
 
        let fn_end = if self.peek()? == Token::OpenSymbol(OpenCloseSymbol::Paren) {
            let begin = self.get_cur_span();
            let iden = idens.pop().unwrap();
            if iden.gen_args.is_some() {
                return Err(self.gen_error(ParseErrorCode::InvalidTraitPathFnEnd { reason: "Cannot contain any generic arguments" }));
            }
            let name = match iden.name {
                IdenName::Name(name, _) => name,
                IdenName::Disambig{ .. } => return Err(self.gen_error(ParseErrorCode::InvalidTraitPathFnEnd { reason: "Cannot have trait disambiguation" })),
            };

            let params = self.parse_comma_separated_closed(OpenCloseSymbol::Paren, Self::parse_fn_type_param)?;
            let return_ty = if self.try_consume(Token::Punctuation(Punctuation::SingleArrowR)) {
                Some(self.parse_type()?)
            } else {
                None
            };
            let span = self.get_span_to_current(begin);

            Some(TraitPathFnEnd {
                span,
                name,
                params,
                return_ty,
            })
        } else {
            None
        };

        let last_name = if seperate_last_iden {
            if fn_end.is_some() {
                return Err(self.gen_error(ParseErrorCode::InvalidPathDisabmiguation { reason: "Cannot have a function-style end" }));
            }

            let iden = idens.pop().unwrap();
            if iden.gen_args.is_some() {
                return Err(self.gen_error(ParseErrorCode::InvalidPathDisabmiguation { reason: "Cannot contain any generic arguments" }));
            }
            match iden.name {
                IdenName::Name(name, span) => Some((name, span)),
                IdenName::Disambig{ .. } => return Err(self.gen_error(ParseErrorCode::InvalidPathDisabmiguation { reason: "Cannot have trait disambiguation" })),
            }
        } else {
            None
        };

        let span = self.get_span_to_current(begin);
        Ok((TraitPath{
            span,
            node_id: NodeId::default(),
            start,
            idens,
            fn_end,
        }, last_name))
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
            Token::StrongKw(StrongKeyword::Fn)       => self.parse_function(attrs, vis, false, false).map(|item| Item::Function(item)),
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
            Token::WeakKw(WeakKeyword::Distinct)     => self.parse_type_item(attrs, vis),
            Token::WeakKw(WeakKeyword::Op)           => if self.try_peek() == Some(Token::StrongKw(StrongKeyword::Use)) {
                self.parse_op_use(attrs, vis)
            } else {
                self.parse_op_set(attrs, vis)
            },
            Token::StrongKw(StrongKeyword::Const) => if self.check_peek(&[1, 2, 4, 5], Token::StrongKw(StrongKeyword::Fn)) {
                    self.parse_function(attrs, vis, false, false).map(|item| Item::Function(item))
                } else {
                    self.parse_const_item(attrs, vis).map(|item| Item::Const(item))
                },
            Token::StrongKw(StrongKeyword::Unsafe) => if self.check_peek(&[1, 2], Token::StrongKw(StrongKeyword::Trait))
                {
                    self.parse_trait(attrs, vis)
                } else if self.check_peek(&[1], Token::StrongKw(StrongKeyword::Impl)) {
                    self.parse_impl(attrs, vis)
                } else {
                    self.parse_function(attrs, vis, false, false).map(|item| Item::Function(item))
                },
            Token::StrongKw(StrongKeyword::Extern) => if self.check_peek(&[2], Token::StrongKw(StrongKeyword::Fn)) {
                    self.parse_function(attrs, vis, false, false).map(|item| Item::Function(item))
                } else if self.check_peek(&[2, 3], Token::StrongKw(StrongKeyword::Static)) {
                    self.parse_static_item(attrs, vis).map(|item| Item::Static(item))
                } else if self.check_peek(&[2], Token::OpenSymbol(OpenCloseSymbol::Brace)) {
                    self.parse_extern_block(attrs, vis)
                } else {
                    Err(self.gen_error(ParseErrorCode::InvalidExternUse))
                },
            Token::WeakKw(WeakKeyword::Record) => {
                if self.check_peek(&[1], Token::StrongKw(StrongKeyword::Struct)) {
                    self.parse_struct(attrs, vis)
                } else if self.check_peek(&[1], Token::StrongKw(StrongKeyword::Enum)) {
                    self.parse_enum(attrs, vis)
                } else if self.check_peek(&[1], Token::StrongKw(StrongKeyword::Bitfield)) {
                    self.parse_bitfield(attrs, vis)
                } else {
                    Err(self.gen_error(ParseErrorCode::UnexpectedFor{ found: peek, for_reason: "item" }))
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
                    Err(self.gen_error(ParseErrorCode::UnexpectedFor{ found: peek, for_reason: "Item" }))
                }
            },
            _ => Err(self.gen_error(ParseErrorCode::UnexpectedFor{ found: peek, for_reason: "Item" }))
        }
    }

    fn parse_impl_item(&mut self) -> Result<ImplItem, ParserErr> {
        self.push_meta_frame();
        
        let attrs = self.parse_attributes()?;
        let vis = self.parse_visibility()?;

        let peek = self.peek()?;
        match peek {
            Token::StrongKw(StrongKeyword::Fn)  => self.parse_impl_function(attrs, vis, false, false),
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
                    self.parse_impl_function(attrs, vis, false, false)
                } else {
                    self.parse_const_item(attrs, vis).map(|item| ImplItem::Const(item))
                }
            }
            Token::StrongKw(StrongKeyword::Unsafe) => {
                let peek_1 = self.peek_at(1)?;
                if peek_1 == Token::WeakKw(WeakKeyword::Property) {
                    self.parse_property(attrs, vis, false).map(|item| ImplItem::Property(item))
                } else {
                    self.parse_impl_function(attrs, vis, false, false)
                }
            },
            Token::StrongKw(StrongKeyword::Type) => self.parse_type_alias(attrs, vis).map(|item| ImplItem::TypeAlias(item)),
            Token::StrongKw(StrongKeyword::Mut) => {
                let peek_1 = self.try_peek_at(1);
                let peek_2 = self.try_peek_at(2);
                let peek_3 = self.try_peek_at(3);
                if peek_1 == Some(Token::StrongKw(StrongKeyword::Static)) ||
                    peek_2 == Some(Token::StrongKw(StrongKeyword::Static)) ||
                    peek_3 == Some(Token::StrongKw(StrongKeyword::Static))
                {
                    self.parse_static_item(attrs, vis).map(|item| ImplItem::Static(item))
                } else {
                    Err(self.gen_error(ParseErrorCode::UnexpectedFor{ found: peek, for_reason: "Item" }))
                }
            },
            Token::StrongKw(StrongKeyword::Static) => self.parse_static_item(attrs, vis).map(|item| ImplItem::Static(item)),
            Token::WeakKw(WeakKeyword::Tls) => self.parse_static_item(attrs, vis).map(|item| ImplItem::Static(item)),
            Token::WeakKw(WeakKeyword::Property) => self.parse_property(attrs, vis, false).map(|item| ImplItem::Property(item)),

            _ => Err(self.gen_error(ParseErrorCode::UnexpectedFor{ found: peek, for_reason: "Item" }))
        }
    }

    fn parse_extern_item(&mut self) -> Result<ExternItem, ParserErr> {
        let attrs = self.parse_attributes()?;
        let vis = self.parse_visibility()?;

        let peek = self.peek()?;
        match peek {
            Token::StrongKw(StrongKeyword::Fn)  => self.parse_function(attrs, vis, true, false).map(|item| ExternItem::Function(item)),
            Token::StrongKw(StrongKeyword::Unsafe) => self.parse_function(attrs, vis, true, false).map(|item| ExternItem::Function(item)),
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
                    Err(self.gen_error(ParseErrorCode::UnexpectedFor{ found: peek, for_reason: "Item" }))
                }
            },
            Token::StrongKw(StrongKeyword::Static) => self.parse_static_item(attrs, vis).map(|item| ExternItem::Static(item)),

            _ => Err(self.gen_error(ParseErrorCode::UnexpectedFor{ found: peek, for_reason: "Item" }))
        }
    }

    fn parse_module(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<Item, ParserErr> {
        let begin = self.get_cur_span();
        self.consume(Token::StrongKw(StrongKeyword::Mod))?;
        let name = self.consume_name()?;
        
         let block = if self.try_consume(Token::Punctuation(Punctuation::Semicolon)) {
            None
        } else {
            Some(self.parse_block()?)
        };

        let span = self.get_span_to_current(begin);
        Ok(Item::Module(self.add_node(ModuleItem {
            span,
            node_id: NodeId::default(),
            attrs,
            vis,
            name,
            block,
        })))
    }

    fn parse_use(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<Item, ParserErr> {
        let begin = self.get_cur_span();
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
            _ => return Err(self.gen_error(ParseErrorCode::ExpectPackageName{ found: peek })),
        };
        self.consume_punct(Punctuation::Colon)?;

        let peek = self.peek()?;
        let library = match peek {
            Token::Punctuation(Punctuation::Dot) => None,
            Token::Name(name_id) => {
                self.consume_single();
                Some(name_id)
            },
            _ => return Err(self.gen_error(ParseErrorCode::ExpectModuleName{ found: peek })),
        };
        self.consume_punct(Punctuation::Dot)?;

        let path = self.parse_use_path()?;

        self.consume_punct(Punctuation::Semicolon);

        let span = self.get_span_to_current(begin);
        Ok(Item::Use(self.add_node(UseItem {
            span,
            node_id: NodeId::default(),
            attrs,
            vis,
            group,
            package,
            library,
            path,
        })))
    }

    fn parse_use_path(&mut self) -> Result<AstNodeRef<UsePath>, ParserErr> {
        let begin = self.get_cur_span();
        if self.try_consume(Token::StrongKw(StrongKeyword::SelfName)) {

            let alias = if self.try_consume(Token::StrongKw(StrongKeyword::As)) {
                Some(self.consume_name()?)
            } else {
                None
            };
            let span = self.get_span_to_current(begin);
            Ok(self.add_node(UsePath::SelfPath { span, node_id: NodeId::default(), alias }))
        } else {
            let mut segments = Vec::new();
            let mut sub_paths = Vec::new();
            let mut is_wildcard = false;

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
                    Token::Punctuation(Punctuation::Asterisk) => {
                        is_wildcard = true;
                        self.consume_single();
                        break;
                    },
                    _ => return Err(self.gen_error(ParseErrorCode::UnexpectedFor { found: peek, for_reason: "Use path" })),
                }
            }

            if is_wildcard {
                let span = self.get_span_to_current(begin);
                Ok(self.add_node(UsePath::Wildcard { span, node_id: NodeId::default(), segments }))
            } else if !sub_paths.is_empty() {
                let span = self.get_span_to_current(begin);
                Ok(self.add_node(UsePath::SubPaths { span, node_id: NodeId::default(), segments, sub_paths }))
            } else {
                let alias = if self.try_consume(Token::StrongKw(StrongKeyword::As)) {
                    Some(self.consume_name()?)  
                } else {
                    None
                };
        
                let span = self.get_span_to_current(begin);
                Ok(self.add_node(UsePath::Alias { span, node_id: NodeId::default(), segments, alias }))
            }
        }
    }

    fn parse_function(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>, in_extern: bool, in_trait: bool) -> Result<AstNodeRef<Function>, ParserErr> {
        let begin = self.get_cur_span();
        let is_const = self.try_consume(Token::StrongKw(StrongKeyword::Const));
        let is_unsafe = self.try_consume(Token::StrongKw(StrongKeyword::Unsafe));

        let abi = if self.try_consume(Token::StrongKw(StrongKeyword::Extern)) {
            Some(self.consume_lit()?)
        } else {
            None
        };

        self.consume_strong_kw(StrongKeyword::Fn)?;
        let name = self.consume_name()?;
        let generics = self.parse_generic_params(true)?;
        let (receiver, params) = self.parse_fn_receiver_and_params()?;

        if receiver.is_some() {
            return Err(self.gen_error(ParseErrorCode::ReceiverInFreeFunction));
        }

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

        let body = if self.try_consume(Token::Punctuation(Punctuation::Semicolon)) {
            if abi.is_none() && !in_extern && !in_trait {
                return Err(ParserErr {
                    err: ParseErrorCode::MissingExternFuncNoBlock,
                    tok_idx: self.token_idx,
                })
            }

            None
        } else {
            Some(self.parse_block()?)
        };

        let span = self.get_span_to_current(begin);

        Ok(self.add_node(Function {
            span,
            node_id: NodeId::default(),
            attrs,
            vis,
            is_const,
            is_unsafe,
            abi,
            name,
            generics,
            params,
            returns,
            where_clause,
            contracts,
            body,
        }))
    }

    fn parse_fn_receiver_and_params(&mut self) -> Result<(Option<FnReceiver>, Vec<FnParam>), ParserErr> {
        self.begin_scope(OpenCloseSymbol::Paren)?;
        let (receiver, has_possible_params) = self.parse_fn_receiver()?;

        let mut params = if has_possible_params {
            self.parse_punct_separated_end(Punctuation::Comma, Token::CloseSymbol(OpenCloseSymbol::Paren), Self::parse_function_param)?
        } else {
            Vec::new()
        };
        self.end_scope()?;

        Ok((receiver, params))
    }

    fn parse_fn_receiver(&mut self) -> Result<(Option<FnReceiver>, bool), ParserErr> {
        if self.peek()? == Token::StrongKw(StrongKeyword::SelfName) ||
            self.peek_at(1)? == Token::StrongKw(StrongKeyword::SelfName) ||
            self.peek_at(2)? == Token::StrongKw(StrongKeyword::SelfName)
        {
            let res = if self.peek_at(1)? == Token::Punctuation(Punctuation::Colon) ||
                self.peek_at(2)? == Token::Punctuation(Punctuation::Colon)
            {
                let begin = self.get_cur_span();

                let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
                self.consume(Token::StrongKw(StrongKeyword::SelfName))?;
                self.consume_punct(Punctuation::Colon)?;
                let ty = self.parse_type()?;

                let span = self.get_span_to_current(begin);
                FnReceiver::SelfTyped{ span, is_mut, ty }
            } else {
                let begin = self.get_cur_span();

                let is_ref = self.try_consume(Token::Punctuation(Punctuation::Ampersand));
                let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
                self.consume(Token::StrongKw(StrongKeyword::SelfName))?;

                let span = self.get_span_to_current(begin);
                FnReceiver::SelfReceiver { span, is_ref, is_mut }
            };

            let has_possible_params = self.try_consume(Token::Punctuation(Punctuation::Comma));
            Ok((Some(res), has_possible_params))
        } else {
            Ok((None, true))
        }
    }

    fn parse_function_param(&mut self) -> Result<FnParam, ParserErr> {
        let begin = self.get_cur_span();

        let names = self.parse_punct_separated_end(Punctuation::Comma, Token::CloseSymbol(OpenCloseSymbol::Paren), Self::parse_param_name)?;

        self.consume_punct(Punctuation::Colon)?;
        let ty = self.parse_type()?;
        let is_variadic = self.try_consume(Token::Punctuation(Punctuation::DotDotDot));

        let def_val = if self.try_consume(Token::Punctuation(Punctuation::Equals)) {
            Some(self.parse_expr(ExprParseMode::General)?)
        } else {
            None
        };

        let span = self.get_span_to_current(begin);
        Ok (FnParam {
            span,
            names,
            ty,
            is_variadic,
            def_val,
        })
    }

    fn parse_param_name(&mut self) -> Result<FnParamName, ParserErr> {
        let begin = self.get_cur_span();
        let label = if self.try_consume(Token::Punctuation(Punctuation::Colon)) {
            Some(self.consume_name()?)
        } else {
            None
        };
        
        let attrs = self.parse_attributes()?;
        let pattern = self.parse_pattern()?;
        
        let span = self.get_span_to_current(begin);
        Ok(FnParamName {
            span,
            attrs,
            label,
            pattern,
        })
    }

    fn parse_func_return(&mut self) -> Result<FnReturn, ParserErr> {
        let begin = self.get_cur_span();
        if self.try_begin_scope(OpenCloseSymbol::Brace) {
            let mut vars = Vec::new();
            while !self.try_end_scope() {
                let mut names = Vec::new();
                loop {
                    names.push(self.consume_name_and_span()?);
                    if !self.try_consume(Token::Punctuation(Punctuation::Comma)) {
                        break;
                    }
                }
                self.consume_punct(Punctuation::Colon);
                let ty = self.parse_type()?;
                vars.push((names, ty));
                
                if !self.try_consume(Token::Punctuation(Punctuation::Comma)) {
                    self.end_scope()?;
                    break;
                }
            }

            let span = self.get_span_to_current(begin);
            Ok(FnReturn::Named{ span, vars })
        } else {
            let ty = self.parse_type()?;
            let span = self.get_span_to_current(begin);
            Ok(FnReturn::Type{ span, ty })
        }
    }

    fn parse_type_item(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<Item, ParserErr> {
        let begin = self.get_cur_span();

        if self.try_consume(Token::WeakKw(WeakKeyword::Distinct)) {
            self.consume_strong_kw(StrongKeyword::Type)?;
            let name = self.consume_name()?;
            let generics = self.parse_generic_params(false)?;
            self.consume_punct(Punctuation::Equals)?;

            let ty = self.parse_type()?;
            self.consume_punct(Punctuation::Semicolon)?;
            let span = self.get_span_to_current(begin);
            return Ok(Item::DistinctType(self.add_node(DistinctType {
                span,
                node_id: NodeId::default(),
                attrs,
                vis,
                name,
                generics,
                ty,
            })));
        }


        let is_distinct = self.try_consume(Token::WeakKw(WeakKeyword::Distinct));
        self.consume_strong_kw(StrongKeyword::Type)?;
        let name = self.consume_name()?;
        let generics = self.parse_generic_params(false)?;

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

            let span = self.get_span_to_current(begin);
            Ok(Item::OpaqueType(self.add_node(OpaqueType {
                span,
                node_id: NodeId::default(),
                attrs,
                vis,
                name,
                size,
            })))
        } else {   
            let ty = if self.try_consume(Token::Punctuation(Punctuation::Equals)) {
                Some(self.parse_type()?)
            } else {
                None
            };

            let ty = self.parse_type()?;
            self.consume_punct(Punctuation::Semicolon)?;
            let span = self.get_span_to_current(begin);
            Ok(Item::TypeAlias(self.add_node(TypeAlias {
                span,
                node_id: NodeId::default(),
                attrs,
                vis,
                name,
                generics,
                ty,
            })))
        }
    }

    fn parse_type_alias(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<AstNodeRef<TypeAlias>, ParserErr> {
        let begin = self.get_cur_span();

        let is_distinct = self.try_consume(Token::WeakKw(WeakKeyword::Distinct));
        self.consume_strong_kw(StrongKeyword::Type)?;
        let name = self.consume_name()?;
        let generics = self.parse_generic_params(false)?;

        self.consume_punct(Punctuation::Equals)?;
  
        let ty = if self.try_consume(Token::Punctuation(Punctuation::Equals)) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let ty = self.parse_type()?;
        self.consume_punct(Punctuation::Semicolon)?;
        let span = self.get_span_to_current(begin);
        Ok(self.add_node(TypeAlias {
            span,
            node_id: NodeId::default(),
            attrs,
            vis,
            name,
            generics,
            ty,
        }))
    }

    fn parse_struct(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<Item, ParserErr> {
        let begin = self.get_cur_span();
        let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
        let is_record = self.try_consume(Token::WeakKw(WeakKeyword::Record));

        self.consume_strong_kw(StrongKeyword::Struct)?;
        let name = self.consume_name()?;

        let generics = self.parse_generic_params(true)?;
        let where_clause = self.parse_where_clause()?;

        let peek = self.peek()?;
        match peek {
            Token::OpenSymbol(OpenCloseSymbol::Brace) => {
                let fields = self.parse_comma_separated_closed(OpenCloseSymbol::Brace, Self::parse_struct_field)?;
                let span = self.get_span_to_current(begin);
                Ok(Item::Struct(self.add_node(Struct::Regular {
                    span,
                    node_id: NodeId::default(),
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
                let span = self.get_span_to_current(begin);
                Ok(Item::Struct(self.add_node(Struct::Tuple {
                    span,
                    node_id: NodeId::default(),
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
                let span = self.get_span_to_current(begin);
                Ok(Item::Struct(self.add_node(Struct::Unit { span, node_id: NodeId::default(), attrs, vis, name })))
            }
            _ => Err(self.gen_error(ParseErrorCode::UnexpectedFor{ found: peek, for_reason: "struct" }))
        }
    }

    fn parse_struct_field(&mut self) -> Result<RegStructField, ParserErr> {
        let begin = self.get_cur_span();
        let attrs = self.parse_attributes()?;
        let vis = self.parse_visibility()?;
        let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));

        if self.try_consume(Token::StrongKw(StrongKeyword::Use)) {
            let span = self.get_span_to_current(begin);
            let path = self.parse_type_path()?;
            Ok(RegStructField::Use {
                span,
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

            let span = self.get_span_to_current(begin);
            Ok(RegStructField::Field {
                span,
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
        let begin = self.get_cur_span();
        let attrs = self.parse_attributes()?;
        let vis = self.parse_visibility()?;
        let ty = self.parse_type()?;
        let def = if self.try_consume(Token::Punctuation(Punctuation::Equals)) {
            Some(self.parse_expr(ExprParseMode::General)?)
        } else {
            None
        };

        let span = self.get_span_to_current(begin);
        Ok(TupleStructField {
            span,
            attrs,
            vis,
            ty,
            def,
        })
    }

    fn parse_union(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<Item, ParserErr> {
        let begin = self.get_cur_span();
        let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
        self.consume_strong_kw(StrongKeyword::Union)?;
        let name = self.consume_name()?;
        let generics = self.parse_generic_params(true)?;
        let where_clause = self.parse_where_clause()?;

        let fields = self.parse_comma_separated_closed(OpenCloseSymbol::Brace, Self::parse_union_field)?;
        let span = self.get_span_to_current(begin);
        Ok(Item::Union(self.add_node(Union {
            span,
            node_id: NodeId::default(),
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
        let begin = self.get_cur_span();
        let attrs = self.parse_attributes()?;
        let vis = self.parse_visibility()?;
        let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
        let name = self.consume_name()?;
        self.consume_punct(Punctuation::Colon)?;
        let ty = self.parse_type()?;

        let span = self.get_span_to_current(begin);
        Ok(UnionField {
            span,
            attrs,
            vis,
            is_mut,
            name,
            ty,
        })
    }

    fn parse_enum(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<Item, ParserErr> {
        let begin = self.get_cur_span();
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

                let span = parser.get_span_to_current(begin);
                Ok(FlagEnumVariant{ span, attrs, name, discriminant })
            })?;

            let span = self.get_span_to_current(begin);
            Ok(Item::Enum(self.add_node(Enum::Flag {
                span,
                node_id: NodeId::default(),
                attrs,
                vis,
                name,
                variants,
            })))
        } else {
            let generics = self.parse_generic_params(true)?;
            let where_clause = self.parse_where_clause()?;
            let variants = self.parse_comma_separated_closed(OpenCloseSymbol::Brace, Self::parse_enum_variant)?;

            let span = self.get_span_to_current(begin);
            Ok(Item::Enum(self.add_node(Enum::Adt {
                span,
                node_id: NodeId::default(),
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
        let begin = self.get_cur_span();
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

                let span = self.get_span_to_current(begin);
                Ok(EnumVariant::Struct {
                    span,
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

                let span = self.get_span_to_current(begin);
                Ok(EnumVariant::Tuple {
                    span,
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

                let span = self.get_span_to_current(begin);
                Ok(EnumVariant::Fieldless {
                    span,
                    attrs,
                    name,
                    discriminant,
                })
            }
        }
    }

    fn parse_bitfield(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<Item, ParserErr> {
        let begin = self.get_cur_span();
        let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
        let is_record = self.try_consume(Token::WeakKw(WeakKeyword::Record));
        self.consume_strong_kw(StrongKeyword::Bitfield)?;
        let name = self.consume_name()?;
        let generics = self.parse_generic_params(true)?;

        let bit_count = if self.try_consume(Token::Punctuation(Punctuation::Colon)) {
            Some(self.parse_expr(ExprParseMode::General)?)
        } else {
            None
        };

        let where_clause = self.parse_where_clause()?;
        let fields = self.parse_comma_separated_closed(OpenCloseSymbol::Brace, Self::parse_bitfield_field)?;

        let span = self.get_span_to_current(begin);
        Ok(Item::Bitfield(self.add_node(Bitfield {
            span,
            node_id: NodeId::default(),
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
        let begin = self.get_cur_span();
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

            let span = self.get_span_to_current(begin);
            Ok(BitfieldField::Use {
                span,
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

            let span = self.get_span_to_current(begin);
            Ok(BitfieldField::Field {
                span,
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
        let begin = self.get_cur_span();
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

        let span = self.get_span_to_current(begin);
        Ok(self.add_node(Const {
            span,
            node_id: NodeId::default(),
            attrs,
            vis,
            name,
            ty,
            val,
        }))
    }

    fn parse_static_item(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<AstNodeRef<Static>, ParserErr> {
        let begin = self.get_cur_span();
        if self.try_consume(Token::StrongKw(StrongKeyword::Extern)) {
            let abi = self.consume_lit()?;
            let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
            self.consume_strong_kw(StrongKeyword::Static)?;
            let name = self.consume_name()?;
            self.consume_punct(Punctuation::Colon)?;
            let ty = self.parse_type()?;
            self.consume_punct(Punctuation::Semicolon)?;

            let span = self.get_span_to_current(begin);
            Ok(self.add_node(Static::Extern {
                span,
                node_id: NodeId::default(),
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
            let ty = if self.try_consume(Token::Punctuation(Punctuation::Colon)) {
                Some(self.parse_type()?)
            } else {
                None
            };
            self.consume_punct(Punctuation::Equals);
            let val = self.parse_expr(ExprParseMode::General)?;
            self.consume_punct(Punctuation::Semicolon)?;

            let span = self.get_span_to_current(begin);
            if is_tls {
                Ok(self.add_node(Static::Tls {
                    span,
                    node_id: NodeId::default(),
                    attrs,
                    vis,
                    is_mut,
                    name,
                    ty,
                    val,
                }))
            } else {
                Ok(self.add_node(Static::Static {
                    span,
                    node_id: NodeId::default(),
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
        let begin = self.get_cur_span();
        let is_unsafe = self.try_consume(Token::StrongKw(StrongKeyword::Unsafe));
        self.consume_weak_kw(WeakKeyword::Property)?;
        let name = self.consume_name()?;

        let ty = if self.try_consume(Token::Punctuation(Punctuation::Colon)) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let mut get = None;
        let mut ref_get = None;
        let mut mut_get = None;
        let mut set = None;
        
        self.begin_scope(OpenCloseSymbol::Brace)?;
        while !self.try_end_scope() {
            let peek = self.peek()?;
            match peek {
                Token::WeakKw(WeakKeyword::Get) => {
                    let begin = self.get_cur_span();
                    self.consume_single();
                    let expr = self.parse_expr(ExprParseMode::General)?;
                    if !expr.has_block() {
                        self.consume_punct(Punctuation::Semicolon)?;
                    }
                    if get.is_some() {
                        return Err(self.gen_error(ParseErrorCode::DuplicateProp{ get_set: "get" }));
                    }
                    
                    let span = self.get_span_to_current(begin);
                    get = Some((span, expr))
                },
                Token::StrongKw(StrongKeyword::Ref) => {
                    let begin = self.get_cur_span();
                    self.consume_single();
                    self.consume_weak_kw(WeakKeyword::Get)?;
                    let expr = self.parse_expr(ExprParseMode::General)?;
                    if !expr.has_block() {
                        self.consume_punct(Punctuation::Semicolon)?;
                    }
                    if ref_get.is_some() {
                        return Err(self.gen_error(ParseErrorCode::DuplicateProp{ get_set: "ref get" }));
                    }
                
                    let span = self.get_span_to_current(begin);
                    ref_get = Some((span, expr))
                },
                Token::StrongKw(StrongKeyword::Mut) => {
                    let begin = self.get_cur_span();
                    self.consume_single();
                    self.consume_weak_kw(WeakKeyword::Get)?;
                    let expr = self.parse_expr(ExprParseMode::General)?;
                    if !expr.has_block() {
                        self.consume_punct(Punctuation::Semicolon)?;
                    }
                    if mut_get.is_some() {
                        return Err(self.gen_error(ParseErrorCode::DuplicateProp{ get_set: "mut get" }));
                    }
                    
                     let span = self.get_span_to_current(begin);
                    mut_get = Some((span, expr))
                },
                Token::WeakKw(WeakKeyword::Set) => {
                    let begin = self.get_cur_span();
                    self.consume_single();
                    let expr = self.parse_expr(ExprParseMode::General)?;
                    if !expr.has_block() {
                        self.consume_punct(Punctuation::Semicolon)?;
                    }
                    if set.is_some() {
                        return Err(self.gen_error(ParseErrorCode::DuplicateProp{ get_set: "set" }));
                    }
                
                    let span = self.get_span_to_current(begin);
                    set = Some((span, expr))
            },
            _ => return Err(self.gen_error(ParseErrorCode::UnexpectedFor{ found: peek, for_reason: "property getter/setter" }))
            }
        }

        let span = self.get_span_to_current(begin);
        Ok(self.add_node(Property {
            span,
            node_id: NodeId::default(),
            attrs,
            vis,
            is_unsafe,
            name,
            ty,
            get,
            ref_get,
            mut_get,
            set,
        }))
    }

    pub fn parse_property_expr(&mut self) -> Result<Option<Expr>, ParserErr> {
        if self.try_consume(Token::Punctuation(Punctuation::Equals)) {
            let expr = self.parse_expr(ExprParseMode::General)?;
            self.consume_punct(Punctuation::Semicolon)?;
            Ok(Some(expr))
        } else if self.peek()? == Token::OpenSymbol(OpenCloseSymbol::Brace) {
            Ok(Some(self.parse_expr(ExprParseMode::General)?))
        } else {
            Ok(None)
        }
    }

    //--------------------------------------------------------------

    fn parse_trait(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<Item, ParserErr> {
        let begin = self.get_cur_span();
        let is_unsafe = self.try_consume(Token::StrongKw(StrongKeyword::Unsafe));
        let is_sealed = self.try_consume(Token::WeakKw(WeakKeyword::Sealed));
        self.consume_strong_kw(StrongKeyword::Trait)?;
        let name = self.consume_name()?;

        let generics = self.parse_generic_params(true)?;

        let bounds = if self.try_consume(Token::Punctuation(Punctuation::Colon)) {
            Some(self.parse_trait_bounds()?)
        } else {
            None
        };

        let where_clause = self.parse_where_clause()?;

        let mut assoc_items = Vec::new();
        self.begin_scope(OpenCloseSymbol::Brace);
        while !self.try_end_scope() {
            assoc_items.push(self.parse_trait_item()?);
        }

        let span = self.get_span_to_current(begin);
        Ok(Item::Trait(self.add_node(Trait {
            span,
            node_id: NodeId::default(),
            attrs,
            vis,
            is_unsafe,
            is_sealed,
            name,
            generics,
            bounds,
            where_clause,
            assoc_items,
        })))
    }

    fn parse_trait_item(&mut self) -> Result<TraitItem, ParserErr> {
        self.push_meta_frame();

        let attrs = self.parse_attributes()?;

        let peek = self.peek()?;
        match peek {
            Token::StrongKw(StrongKeyword::Fn) => self.parse_trait_function(attrs),
            Token::StrongKw(StrongKeyword::Const) => {
                let peek_1 = self.try_peek_at(1);
                let peek_2 = self.try_peek_at(2);
                if  peek_1 == Some(Token::StrongKw(StrongKeyword::Fn)) || // const fn..
                    peek_2 == Some(Token::StrongKw(StrongKeyword::Fn))    // const unsafe fn..
                {
                    self.parse_trait_function(attrs)
                } else {
                    self.parse_trait_const(attrs).map(|item| TraitItem::Const(item))
                }
            }
            Token::StrongKw(StrongKeyword::Unsafe) => {
                let peek = self.peek_at(1)?;
                if peek == Token::WeakKw(WeakKeyword::Property) {
                    self.parse_trait_property(attrs).map(|item| TraitItem::Property(item))
                } else {
                    self.parse_trait_function(attrs)
                }
            },
            Token::StrongKw(StrongKeyword::Type) => self.parse_trait_type_alias(attrs).map(|item| TraitItem::TypeAlias(item)),
            Token::WeakKw(WeakKeyword::Property) => self.parse_trait_property(attrs).map(|item| TraitItem::Property(item)),
            _ => Err(self.gen_error(ParseErrorCode::UnexpectedFor{ found: peek, for_reason: "Item" })),
        }
    }

    fn parse_trait_function(&mut self, attrs: Vec<AstNodeRef<Attribute>>) -> Result<TraitItem, ParserErr> {
        let begin = self.get_cur_span();
        let is_const = self.try_consume(Token::StrongKw(StrongKeyword::Const));
        let is_unsafe = self.try_consume(Token::StrongKw(StrongKeyword::Unsafe));

        self.consume_strong_kw(StrongKeyword::Fn)?;
        let name = self.consume_name()?;
        let generics = self.parse_generic_params(true)?;
        let (receiver, params) = self.parse_fn_receiver_and_params()?;
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

        let body = if self.try_consume(Token::Punctuation(Punctuation::Semicolon)) {
            None
        } else {
            Some(self.parse_block()?)
        };

        let span = self.get_span_to_current(begin);
        if let Some(receiver) = receiver {
            Ok(TraitItem::Method(self.add_node(TraitMethod {
                span,
                node_id: NodeId::default(),
                attrs,
                is_const,
                is_unsafe,
                name,
                generics,
                receiver,
                params,
                returns,
                where_clause,
                contracts,
                body,
            })))
        } else {
            Ok(TraitItem::Function(self.add_node(TraitFunction {
                span,
                node_id: NodeId::default(),
                attrs,
                is_const,
                is_unsafe,
                name,
                generics,
                params,
                returns,
                where_clause,
                contracts,
                body,
            })))
        }
    }

    fn parse_trait_type_alias(&mut self, attrs: Vec<AstNodeRef<Attribute>>) -> Result<AstNodeRef<TraitTypeAlias>, ParserErr> {
        let begin = self.get_cur_span();

        self.consume_strong_kw(StrongKeyword::Type)?;
        let name = self.consume_name()?;
        let generics = self.parse_generic_params(true)?;

        let bounds = if self.try_consume(Token::Punctuation(Punctuation::Colon)) {
            self.parse_punct_separated(Punctuation::Ampersand, Self::parse_generic_type_bound)?
        } else {
            Vec::new()
        };
        let where_clause = self.parse_where_clause()?;

        self.consume_punct(Punctuation::Equals)?;

        let ty = if self.try_consume(Token::Punctuation(Punctuation::Equals)) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let def = if self.try_consume(Token::Punctuation(Punctuation::Equals)) {
            Some(self.parse_type()?)
        } else {
            None
        };
        self.consume_punct(Punctuation::Semicolon)?;

        let span = self.get_span_to_current(begin);
        Ok(self.add_node(TraitTypeAlias {
            span,
            node_id: NodeId::default(),
            attrs,
            name,
            generics,
            bounds,
            where_clause,
            def,
        }))
    }

    fn parse_trait_const(&mut self, attrs: Vec<AstNodeRef<Attribute>>) -> Result<AstNodeRef<TraitConst>, ParserErr> {
        let begin = self.get_cur_span();
        self.consume_strong_kw(StrongKeyword::Const)?;
        let name = self.consume_name()?;
        let ty = self.parse_type()?;
        let def = if self.try_consume(Token::Punctuation(Punctuation::Colon)) {
            Some(self.parse_expr(ExprParseMode::General)?)
        } else {
            None
        };
        self.consume_punct(Punctuation::Semicolon)?;

        let span = self.get_span_to_current(begin);
        Ok(self.add_node(TraitConst {
            span,
            node_id: NodeId::default(),
            attrs,
            name,
            ty,
            def,
        }))
    }

    fn parse_trait_property(&mut self, attrs: Vec<AstNodeRef<Attribute>>) -> Result<AstNodeRef<TraitProperty>, ParserErr> {
        let begin = self.get_cur_span();
        let is_unsafe = self.try_consume(Token::StrongKw(StrongKeyword::Unsafe));
        self.consume_weak_kw(WeakKeyword::Property)?;
        let name = self.consume_name()?;
        self.consume_punct(Punctuation::Colon)?;
        let ty = self.parse_type()?;

        let mut get = None;
        let mut ref_get = None;
        let mut mut_get = None;
        let mut set = None;

        self.begin_scope(OpenCloseSymbol::Brace)?;
        while !self.try_end_scope() {
            let peek = self.peek()?;
            match peek {
                Token::WeakKw(WeakKeyword::Get) => {
                    let begin = self.get_cur_span();
                    self.consume_single();
                    let expr = self.parse_property_expr()?;
                    if get.is_some() {
                        return Err(self.gen_error(ParseErrorCode::DuplicateProp{ get_set: "get" }));
                    }
                    
                    let span = self.get_span_to_current(begin);
                    get = Some((span, expr));
                },
                Token::StrongKw(StrongKeyword::Ref) => {
                    let begin = self.get_cur_span();
                    self.consume_single();
                    self.consume_weak_kw(WeakKeyword::Get)?;
                    let expr = self.parse_property_expr()?;
                    if ref_get.is_some() {
                        return Err(self.gen_error(ParseErrorCode::DuplicateProp{ get_set: "ref get" }));
                    }

                    let span = self.get_span_to_current(begin);
                    ref_get = Some((span, expr));
                },
                Token::StrongKw(StrongKeyword::Mut) => {
                    let begin = self.get_cur_span();
                    self.consume_single();
                    self.consume_weak_kw(WeakKeyword::Get)?;
                    let expr = self.parse_property_expr()?;
                    if mut_get.is_some() {
                        return Err(self.gen_error(ParseErrorCode::DuplicateProp{ get_set: "mut get" }));
                    }
                    
                    let span = self.get_span_to_current(begin);
                    mut_get = Some((span, expr));
                },
                Token::WeakKw(WeakKeyword::Set) => {
                    let begin = self.get_cur_span();
                    self.consume_single();
                    let expr = self.parse_property_expr()?;
                    if set.is_some() {
                       return Err(self.gen_error(ParseErrorCode::DuplicateProp{ get_set: "set" }));
                    }
                
                    let span = self.get_span_to_current(begin);
                    set = Some((span, expr));
                },
                _ => return Err(self.gen_error(ParseErrorCode::UnexpectedFor{ found: peek, for_reason: "property getter/setter" }))
            }
        }

        let span = self.get_span_to_current(begin);
        Ok(self.add_node(TraitProperty {
            span,
            node_id: NodeId::default(),
            attrs,
            is_unsafe,
            name,
            ty,
            get,
            ref_get,
            mut_get,
            set,
        }))
    }

    //--------------------------------------------------------------

    fn parse_impl(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<Item, ParserErr> {
        let begin = self.get_cur_span();
        let is_unsafe = self.try_consume(Token::StrongKw(StrongKeyword::Unsafe));
        self.consume_strong_kw(StrongKeyword::Impl)?;
        let generics = self.parse_generic_params(true)?;
        let ty = self.parse_type()?;
        let impl_trait = if self.try_consume(Token::StrongKw(StrongKeyword::As)) {
            Some(self.parse_trait_path()?)
        } else {
            None
        };
        let where_clause = self.parse_where_clause()?;

        let mut assoc_items = Vec::new();
        self.begin_scope(OpenCloseSymbol::Brace);
        while !self.try_end_scope() {
            assoc_items.push(self.parse_impl_item()?);
        }

        let span = self.get_span_to_current(begin);
        Ok(Item::Impl(self.add_node(Impl {
            span,
            node_id: NodeId::default(),
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

    fn parse_impl_function(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>, in_extern: bool, in_trait: bool) -> Result<ImplItem, ParserErr> {
        let begin = self.get_cur_span();
        let is_const = self.try_consume(Token::StrongKw(StrongKeyword::Const));
        let is_unsafe = self.try_consume(Token::StrongKw(StrongKeyword::Unsafe));

        self.consume_strong_kw(StrongKeyword::Fn)?;
        let name = self.consume_name()?;
        let generics = self.parse_generic_params(true)?;
        let (receiver, params) = self.parse_fn_receiver_and_params()?;
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

        let span = self.get_span_to_current(begin);
        if let Some(receiver) = receiver {
            Ok(ImplItem::Method(self.add_node(Method {
                span,
                node_id: NodeId::default(),
                attrs,
                vis,
                is_const,
                is_unsafe,
                name,
                generics,
                receiver,
                params,
                returns,
                where_clause,
                contracts,
                body,
            })))
        } else {
            Ok(ImplItem::Function(self.add_node(Function {
                span,
                node_id: NodeId::default(),
                attrs,
                vis,
                is_const,
                is_unsafe,
                abi: None,
                name,
                generics,
                params,
                returns,
                where_clause,
                contracts,
                body: Some(body),
            })))
        }
    }

    //--------------------------------------------------------------

    fn parse_extern_block(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<Item, ParserErr> {
        let begin = self.get_cur_span();
        self.consume_strong_kw(StrongKeyword::Extern)?;
        let abi = self.consume_lit()?;
        
        let mut items = Vec::new();
        self.begin_scope(OpenCloseSymbol::Brace);
        while !self.try_end_scope() {
            items.push(self.parse_extern_item()?);
        }

        let span = self.get_span_to_current(begin);
        Ok(Item::Extern(self.add_node(ExternBlock {
            span,
            node_id: NodeId::default(),
            attrs,
            vis,
            abi,
            items,
        })))
    }

    //--------------------------------------------------------------

    fn parse_op_set(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<Item, ParserErr> {
        let begin = self.get_cur_span();
        self.consume_weak_kw(WeakKeyword::Op)?;
        self.consume_strong_kw(StrongKeyword::Trait)?;
        let name = self.consume_name()?;

        let (bases, precedence) = if self.try_consume(Token::Punctuation(Punctuation::Colon)) {
            let mut bases = Vec::new();
            loop {
                bases.push(self.consume_name_and_span()?);
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

        let span = self.get_span_to_current(begin);
        if !bases.is_empty() {
            Ok(Item::OpSet(self.add_node(OpSet::Extended {
                span,
                node_id: NodeId::default(),
                attrs,
                vis,
                name,
                bases,
                elems,
            })))
        } else {
            Ok(Item::OpSet(self.add_node(OpSet::Base {
                span,
                node_id: NodeId::default(),
                attrs,
                vis,
                name,
                precedence,
                elems,
            })))
        }

    }

    fn parse_op_elem(&mut self) -> Result<OpElem, ParserErr> {
        let begin = self.get_cur_span();
        if self.try_consume(Token::WeakKw(WeakKeyword::Invar)) {
            self.push_meta_frame();
            let expr = self.parse_block_expr(self.get_cur_span(), None)?;
            
            let span = self.get_span_to_current(begin);
            return Ok(OpElem::Contract{ span, expr });
        }

        let (peek, _span) = self.consume_single();
        let op_type = match peek {
            Token::WeakKw(WeakKeyword::Prefix)    => OpType::Prefix,
            Token::WeakKw(WeakKeyword::Postfix)   => OpType::Postfix,
            Token::WeakKw(WeakKeyword::Infix)     => OpType::Infix,
            Token::WeakKw(WeakKeyword::Assign)    => OpType::Assign,
            _ => return Err(self.gen_error(ParseErrorCode::UnexpectedFor { found: peek, for_reason: "operator type" }))
        };

        self.consume_weak_kw(WeakKeyword::Op)?;
        let op = self.consume_any_punct()?;

        self.consume_punct(Punctuation::Colon)?;

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

        let span = self.get_span_to_current(begin);
        Ok(OpElem::Def {
            span,
            op_type,
            op,
            name,
            ret,
            def,
        })
    }

    fn parse_op_use(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<Item, ParserErr> {
        let begin = self.get_cur_span();
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
            _ => return Err(self.gen_error(ParseErrorCode::ExpectPackageName{ found: peek })),
        };
        self.consume_punct(Punctuation::Colon)?;

        let peek = self.peek()?;
        let library = match peek {
            Token::Punctuation(Punctuation::Dot) => None,
            Token::Name(name_id) => {
                self.consume_single();
                Some(name_id)
            },
            _ => return Err(self.gen_error(ParseErrorCode::ExpectModuleName{ found: peek })),
        };


        let op_sets = if self.try_consume(Token::Punctuation(Punctuation::Dot)) {
            let ops = self.parse_comma_separated_closed(OpenCloseSymbol::Brace, Self::consume_name)?;
            ops
        } else {
            Vec::new()
        };

        let span = self.get_span_to_current(begin);
        Ok(Item::OpUse(self.add_node(OpUse {
            span,
            node_id: NodeId::default(),
            group,
            package,
            library,
            op_sets,
        })))
    }

    //--------------------------------------------------------------

    fn parse_precedence(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<Item, ParserErr> {
        let begin = self.get_cur_span();
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
                    higher_than = Some(parser.consume_name_and_span()?);
                    Ok(())
                },
                Token::WeakKw(WeakKeyword::LowerThan) => {
                    parser.consume_single();
                    parser.consume_punct(Punctuation::Colon)?;
                    lower_than = Some(parser.consume_name_and_span()?);
                    Ok(())
                },
                Token::WeakKw(WeakKeyword::Associativity) => {
                    let begin = parser.get_cur_span();
                    parser.consume_single();
                    parser.consume_punct(Punctuation::Colon)?;
                    let name_id = parser.consume_name()?;
                    let kind = match &parser.names[name_id] {
                        "none" => PrecedenceAssocKind::None,
                        "left" => PrecedenceAssocKind::Left,
                        "right" => PrecedenceAssocKind::Right,
                        _ => return Err(parser.gen_error(ParseErrorCode::InvalidPrecedenceAssoc{ name: parser.names[name_id].to_string() }))
                    };
                    
                    let span = parser.get_span_to_current(begin);
                    associativity = Some(PrecedenceAssociativity {
                        span,
                        kind
                    });
                    Ok(())
                },
                Token::Punctuation(Punctuation::Comma) => {
                    parser.consume_single();
                    Ok(())
                },
                _ => Err(parser.gen_error(ParseErrorCode::UnexpectedFor { found: peek, for_reason: "precedence" })),
            }
        })?;

        let span = self.get_span_to_current(begin);
        Ok(Item::Precedence(self.add_node(Precedence {
            span,
            node_id: NodeId::default(),
            attrs,
            vis,
            name,
            higher_than,
            lower_than,
            associativity,
        })))
    }

    fn parse_precedence_use(&mut self, attrs: Vec<AstNodeRef<Attribute>>, vis: Option<AstNodeRef<Visibility>>) -> Result<Item, ParserErr> {
        let begin = self.get_cur_span();
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
            _ => return Err(self.gen_error(ParseErrorCode::ExpectPackageName{ found: peek })),
        };
        self.consume_punct(Punctuation::Colon)?;

        let peek = self.peek()?;
        let library = match peek {
            Token::Punctuation(Punctuation::Dot) => None,
            Token::Name(name_id) => {
                self.consume_single();
                Some(name_id)
            },
            _ => return Err(self.gen_error(ParseErrorCode::ExpectModuleName{ found: peek })),
        };


        let precedences =  if self.try_consume(Token::Punctuation(Punctuation::Dot)) {
            let precedences = self.parse_comma_separated_closed(OpenCloseSymbol::Brace, Self::consume_name)?;
            precedences
        } else {
            Vec::new()
        };

        let span = self.get_span_to_current(begin);
        Ok(Item::PrecedenceUse(self.add_node(PrecedenceUse {
            span,
            node_id: NodeId::default(),
            group,
            package,
            library,
            precedences,
        })))
    }

// =============================================================================================================================

    fn parse_block(&mut self) -> Result<AstNodeRef<Block>, ParserErr> {
        self.push_meta_frame();
        let begin = self.last_frame.span;
        self.begin_scope(OpenCloseSymbol::Brace)?;

        let mut stmts = Vec::new();
        while !self.try_end_scope() {
            stmts.push(self.parse_stmt(true)?);
        }

        let final_expr = if let Some(Stmt::Expr(stmt)) = stmts.last() {
            if stmt.has_semi {
                let Some(Stmt::Expr(stmt)) = stmts.pop() else { return Err(self.gen_error(ParseErrorCode::InternalError("Final expr in block stopped existing when removing it"))) };
                Some(stmt)
            } else {
                None
            }
        } else {
            None
        };

        let span = self.get_span_to_current(begin);
        Ok(self.add_node(Block {
            span,
            node_id: NodeId::default(),
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
                if !attrs.is_empty() {
                    return Err(self.gen_error(ParseErrorCode::EmptyStmtWithAttrs));
                }

                let span = self.get_cur_span();
                self.consume_single();
                Ok(Stmt::Empty(self.add_node(EmptyStmt {
                    span,
                    node_id: NodeId::default(),
                })))
            },
            _ => self.parse_expr_stmt(attrs, allow_expr_without_semicolon).map(|stmt| Stmt::Expr(stmt))
        }
    }

    fn parse_name_var_decl(&mut self, attrs: Vec<AstNodeRef<Attribute>>) -> Result<AstNodeRef<VarDecl>, ParserErr> {
        let begin = self.get_cur_span();
        let names = self.parse_punct_separated(Punctuation::Comma, |parser| {
            let begin = parser.get_cur_span();
            let is_mut = parser.try_consume(Token::StrongKw(StrongKeyword::Mut));
            let name = parser.consume_name()?;
            let span = parser.get_span_to_current(begin);
            Ok((is_mut, name, span))
        })?;

        self.consume_punct(Punctuation::ColonEquals)?;
        let expr = self.parse_expr(ExprParseMode::AllowComma)?;
        self.consume_punct(Punctuation::Semicolon)?;

        let span = self.get_span_to_current(begin);
        Ok(self.add_node(VarDecl::Named {
            span,
            node_id: NodeId::default(),
            attrs,
            names,
            expr,
        }))
    }

    fn parse_let_var_decl(&mut self, attrs: Vec<AstNodeRef<Attribute>>) -> Result<AstNodeRef<VarDecl>, ParserErr> {
        let begin = self.get_cur_span();
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
            Some(self.parse_block_expr(self.get_cur_span(), None)?)
        } else {
            None
        };
        self.consume_punct(Punctuation::Semicolon)?;

        let span = self.get_span_to_current(begin);
        Ok(self.add_node(VarDecl::Let {
            span,
            node_id: NodeId::default(),
            attrs,
            pattern,
            ty,
            expr,
            else_block,
        }))
    }

    fn parse_defer_stmt(&mut self, attrs: Vec<AstNodeRef<Attribute>>) -> Result<AstNodeRef<Defer>, ParserErr> {
        let begin = self.get_cur_span();
        self.consume_strong_kw(StrongKeyword::Defer)?;
        let expr = self.parse_expr(ExprParseMode::General)?;
        if !expr.has_block() {
            self.consume_punct(Punctuation::Semicolon)?;
        }
        
        let span = self.get_span_to_current(begin);
        Ok(self.add_node(Defer {
            span,
            node_id: NodeId::default(),
            attrs,
            expr,
        }))
    }

    fn parse_err_defer_stmt(&mut self, attrs: Vec<AstNodeRef<Attribute>>) -> Result<AstNodeRef<ErrDefer>, ParserErr> {
        let begin = self.get_cur_span();
        self.consume_strong_kw(StrongKeyword::ErrDefer)?;
        let receiver = if self.try_consume(Token::Punctuation(Punctuation::Or)) {
            let begin = self.get_cur_span();
            let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
            let name = self.consume_name()?;
            self.consume_punct(Punctuation::Or)?;

            let span = self.get_span_to_current(begin);
            Some(ErrDeferReceiver { span, is_mut, name })
        } else {
            None
        };

        let expr = self.parse_expr(ExprParseMode::General)?;
        if !expr.has_block() {
            self.consume_punct(Punctuation::Semicolon)?;
        }

        let span = self.get_span_to_current(begin);
        Ok(self.add_node(ErrDefer {
            span,
            node_id: NodeId::default(),
            attrs,
            receiver,
            expr,
        }))
    }

    fn parse_expr_stmt(&mut self, attrs: Vec<AstNodeRef<Attribute>>, allow_expr_without_semicolon: bool) -> Result<AstNodeRef<ExprStmt>, ParserErr> {
        let begin = self.get_cur_span();
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

        let span = self.get_span_to_current(begin);
        Ok(self.add_node(ExprStmt {
            span,
            node_id: NodeId::default(),
            attrs,
            expr,
            has_semi,
        }))
    }

// =============================================================================================================================

    fn parse_expr(&mut self, mode: ExprParseMode) -> Result<Expr, ParserErr> {
        let begin = self.get_cur_span();
        self.push_meta_frame();
        
        let peek = self.peek()?;
        let mut expr = match peek {
            Token::StrongKw(StrongKeyword::True)          |
            Token::StrongKw(StrongKeyword::False)         |
            Token::Literal(_)                             => self.parse_literal_expr()?,
            Token::Name(_)                                |
            Token::Punctuation(Punctuation::Dot)          |
            Token::StrongKw(StrongKeyword::SelfName)      => self.parse_path_expr()?,
            Token::StrongKw(kw) if kw.is_primitive_type() => self.parse_path_expr()?,
            Token::StrongKw(StrongKeyword::Unsafe)        => self.parse_unsafe_block_expr()?,
            Token::StrongKw(StrongKeyword::Const)         => self.parse_const_block_expr()?,
            Token::StrongKw(StrongKeyword::TryExclaim)    |
            Token::StrongKw(StrongKeyword::Try)           => self.parse_try_block_expr()?,
            Token::StrongKw(StrongKeyword::If)            => self.parse_if_expr()?,
            Token::StrongKw(StrongKeyword::Loop)          => self.parse_loop_expr(self.get_cur_span(), None)?,
            Token::StrongKw(StrongKeyword::While)         => self.parse_while_expr(self.get_cur_span(), None)?,
            Token::StrongKw(StrongKeyword::Do)            => self.parse_do_while_expr(self.get_cur_span(), None)?,
            Token::StrongKw(StrongKeyword::For)           => self.parse_for_expr(self.get_cur_span(),None)?,
            Token::StrongKw(StrongKeyword::Match)         => self.parse_match_expr(self.get_cur_span(), None)?,
            Token::StrongKw(StrongKeyword::Break)         => self.parse_break_expr()?,
            Token::StrongKw(StrongKeyword::Continue)      => self.parse_continue_expr()?,
            Token::StrongKw(StrongKeyword::Fallthrough)   => self.parse_fallthrough_expr()?,
            Token::StrongKw(StrongKeyword::Return)        => self.parse_return_expr()?,
            Token::StrongKw(StrongKeyword::When)          => self.parse_when_expr()?,
            Token::StrongKw(StrongKeyword::Let) if mode == ExprParseMode::AllowLet => self.parse_let_binding_expr()?,

            Token::StrongKw(StrongKeyword::Move)          |
            Token::Punctuation(Punctuation::Or)           => self.parse_closure_expr()?,

            Token::Punctuation(Punctuation::Colon)        => {
                let begin = self.get_cur_span();
                let label = Some(self.parse_label()?);
                let peek = self.peek()?;
                match peek {
                    Token::StrongKw(StrongKeyword::Loop)      => self.parse_loop_expr(begin, label)?,
                    Token::StrongKw(StrongKeyword::While)     => self.parse_while_expr(begin, label)?,
                    Token::StrongKw(StrongKeyword::Do)        => self.parse_do_while_expr(begin, label)?,
                    Token::StrongKw(StrongKeyword::For)       => self.parse_for_expr(begin, label)?,
                    Token::StrongKw(StrongKeyword::Match)     => self.parse_match_expr(begin, label)?,
                    Token::OpenSymbol(OpenCloseSymbol::Brace) => Expr::Block(self.parse_block_expr(begin, label)?),
                    _ => return Err(self.gen_error(ParseErrorCode::InvalidLabel)),
                }
            },
            Token::Punctuation(Punctuation::Comma)            |
            Token::Punctuation(Punctuation::Semicolon)        => return Err(self.gen_error(ParseErrorCode::UnexpectedFor { found: peek, for_reason: "expression" })),

            Token::Punctuation(_)                             => self.parse_prefix_expr()?,

            Token::OpenSymbol(OpenCloseSymbol::Brace)         => Expr::Block(self.parse_block_expr(self.get_cur_span(), None)?),
            Token::OpenSymbol(OpenCloseSymbol::Bracket)       => self.parse_array_expr()?,
            Token::OpenSymbol(OpenCloseSymbol::Paren)         => {
                if self.check_peek(&[1], Token::CloseSymbol(OpenCloseSymbol::Paren)) {
                    self.consume_single();
                    self.consume_single();
                    let span = self.get_span_to_current(begin);
                    Expr::Unit(self.add_node(UnitExpr {
                        span,
                        node_id: NodeId::default(),
                    }))
                } else if self.check_peek(&[1], Token::Punctuation(Punctuation::Colon)) {
                    self.parse_path_expr()?
                } else {
                    self.parse_paren_expr()?
                }
            },

            Token::Underscore => {
                self.consume_single();
                Expr::Underscore(self.add_node(UnderscoreExpr {
                    span: begin,
                    node_id: NodeId::default(),
                }))
            },

            _ => return Err(self.gen_error(ParseErrorCode::UnexpectedFor { found: peek, for_reason: "expression" })),
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
                        
                        _ => return Err(self.gen_error(ParseErrorCode::UnexpectedFor{ found: peek_1, for_reason: "expression" })),
                    }
                },
                
                Token::Punctuation(_) => {
                    let op = self.consume_any_punct()?;
                    if self.is_end_of_expr() {
                        let span = self.get_span_to_current(begin);
                        Expr::Postfix(self.add_node(PostfixExpr {
                            span,
                            node_id: NodeId::default(),
                            op,
                            expr,
                        }))
                    } else {
                        if let Token::Punctuation(_) = self.peek()? {
                            // If we have 2 operators following each other, try to figure out which on in infix

                            let has_prev_whitespace = !self.token_store.metadata[self.token_idx - 1].meta_elems.is_empty();
                            let has_next_whitespace = !self.token_store.metadata[self.token_idx + 1].meta_elems.is_empty();

                            if has_prev_whitespace == has_next_whitespace {
                                return Err(self.gen_error(ParseErrorCode::AmbiguousOperators));
                            } else if has_prev_whitespace {
                               let right = self.parse_expr(mode)?;
                               let span = self.get_span_to_current(begin);
                                Expr::Infix(self.add_node(InfixExpr {
                                    span,
                                    node_id: NodeId::default(),
                                    op,
                                    left: expr,
                                    right,
                                }))
                            } else { // if has_next_whitepace
                                let span = self.get_span_to_current(begin);
                                Expr::Postfix(self.add_node(PostfixExpr {
                                    span,
                                    node_id: NodeId::default(),
                                    op,
                                    expr,
                                }))
                            }
                        } else {    
                            let right = self.parse_expr(mode)?;
                            let span = self.get_span_to_current(begin);
                            Expr::Infix(self.add_node(InfixExpr {
                                span,
                                node_id: NodeId::default(),
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
                    let span = self.get_span_to_current(begin);
                    Expr::Infix(self.add_node(InfixExpr {
                        span,
                        node_id: NodeId::default(),
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
                Token::Literal(_) => return Err(self.gen_error(ParseErrorCode::UnexpectedFor{ found: peek, for_reason: "expression" })),
                
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
        let begin = self.get_cur_span();
        let peek = self.peek()?;
        match peek {
            Token::Literal(lit_id) => {
                let literal = self.consume_lit()?;
                let lit_op = self.parse_literal_op()?;

                let span = self.get_span_to_current(begin);
                Ok(LiteralExpr {
                    span,
                    node_id: NodeId::default(),
                    literal: LiteralValue::Lit(literal),
                    lit_op
                })
            },
            Token::StrongKw(StrongKeyword::True) |
            Token::StrongKw(StrongKeyword::False) => {
                let (tok, _span) = self.consume_single();
                let value = tok == Token::StrongKw(StrongKeyword::True);
                let lit_op = self.parse_literal_op()?;
                
                let span = self.get_span_to_current(begin);
                Ok(LiteralExpr {
                    span,
                    node_id: NodeId::default(),
                    literal: LiteralValue::Bool(value),
                    lit_op,
                })
            }

            _ => Err(self.gen_error(ParseErrorCode::UnexpectedFor { found: peek, for_reason: "literal" })),
        }
    }

    fn parse_literal_op(&mut self) -> Result<Option<LiteralOp>, ParserErr> {
        Ok(if self.try_consume(Token::Punctuation(Punctuation::Colon)) {
            let (peek, span) = self.consume_single();
            Some(match peek {
                Token::Name(name_id) => LiteralOp::Name(name_id),
                Token::StrongKw(kw) => match kw {
                    StrongKeyword::U8     => LiteralOp::Primitive(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::U8 }),
                    StrongKeyword::U16    => LiteralOp::Primitive(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::U16 }),
                    StrongKeyword::U32    => LiteralOp::Primitive(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::U32 }),
                    StrongKeyword::U64    => LiteralOp::Primitive(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::U64 }),
                    StrongKeyword::U128   => LiteralOp::Primitive(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::U128 }),
                    StrongKeyword::I8     => LiteralOp::Primitive(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::I8 }),
                    StrongKeyword::I16    => LiteralOp::Primitive(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::I16 }),
                    StrongKeyword::I32    => LiteralOp::Primitive(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::I32 }),
                    StrongKeyword::I64    => LiteralOp::Primitive(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::I64 }),
                    StrongKeyword::I128   => LiteralOp::Primitive(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::I128 }),
                    StrongKeyword::F16    => LiteralOp::Primitive(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::F16 }),
                    StrongKeyword::F32    => LiteralOp::Primitive(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::F32 }),
                    StrongKeyword::F64    => LiteralOp::Primitive(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::F64 }),
                    StrongKeyword::F128   => LiteralOp::Primitive(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::F128 }),
                    StrongKeyword::Bool   => LiteralOp::Primitive(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::Bool }),
                    StrongKeyword::B8     => LiteralOp::Primitive(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::B8 }),
                    StrongKeyword::B16    => LiteralOp::Primitive(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::B16 }),
                    StrongKeyword::B32    => LiteralOp::Primitive(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::B32 }),
                    StrongKeyword::B64    => LiteralOp::Primitive(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::B64 }),
                    StrongKeyword::Char   => LiteralOp::Primitive(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::Char }),
                    StrongKeyword::Char7  => LiteralOp::Primitive(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::Char7 }),
                    StrongKeyword::Char8  => LiteralOp::Primitive(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::Char8 }),
                    StrongKeyword::Char16 => LiteralOp::Primitive(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::Char16 }),
                    StrongKeyword::Char32 => LiteralOp::Primitive(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::Char32 }),
                    StrongKeyword::Str    => LiteralOp::StringSlice(StringSliceType { span, node_id: NodeId::default(), ty: type_system::StringSliceType::Str }),
                    StrongKeyword::Str7   => LiteralOp::StringSlice(StringSliceType { span, node_id: NodeId::default(), ty: type_system::StringSliceType::Str7 }),
                    StrongKeyword::Str8   => LiteralOp::StringSlice(StringSliceType { span, node_id: NodeId::default(), ty: type_system::StringSliceType::Str8 }),
                    StrongKeyword::Str16  => LiteralOp::StringSlice(StringSliceType { span, node_id: NodeId::default(), ty: type_system::StringSliceType::Str16 }),
                    StrongKeyword::Str32  => LiteralOp::StringSlice(StringSliceType { span, node_id: NodeId::default(), ty: type_system::StringSliceType::Str32 }),
                    StrongKeyword::CStr   => LiteralOp::StringSlice(StringSliceType { span, node_id: NodeId::default(), ty: type_system::StringSliceType::CStr }),
                    _ => return Err(self.gen_error(ParseErrorCode::UnexpectedFor{ found: peek, for_reason:  "literal operator" })),
                }
                _ => return Err(self.gen_error(ParseErrorCode::UnexpectedFor{ found: peek, for_reason: "literal operator" })),
            })
        } else {
            None
        })
    }

    fn parse_path_expr(&mut self) -> Result<Expr, ParserErr> {
        let begin = self.get_cur_span();
        if self.try_consume(Token::StrongKw(StrongKeyword::SelfName)) {
            Ok(Expr::Path(self.add_node(PathExpr::SelfVar {
                span: begin,
                node_id: NodeId::INVALID,
            })))
        } else {
            
            let start = self.parse_path_start()?;
            let iden = self.parse_identifier(true)?;
            let span = self.get_span_to_current(begin);
            
            Ok(Expr::Path(self.add_node(PathExpr::Path {
                span: span,
                node_id: NodeId::INVALID,
                start,
                iden,
            })))
        }
    }

    fn parse_block_expr(&mut self, begin: SpanId, label: Option<NameId>) -> Result<AstNodeRef<BlockExpr>, ParserErr> {
        let block = self.parse_block()?;
        let span = self.get_span_to_current(begin);
        Ok(self.add_node(BlockExpr {
            span,
            node_id: NodeId::default(),
            kind: if let Some(label) = label { BlockExprKind::Labeled { label } } else { BlockExprKind::Normal },
            block
        }))
    }

    fn parse_unsafe_block_expr(&mut self) -> Result<Expr, ParserErr> {
        let begin = self.get_cur_span();
        self.consume_strong_kw(StrongKeyword::Unsafe)?;

        let block = self.parse_block()?;

        let span = self.get_span_to_current(begin);
        Ok(Expr::Block(self.add_node(BlockExpr{
            span,
            node_id: NodeId::default(),
            kind: BlockExprKind::Unsafe,
            block,
        })))
    }

    fn parse_const_block_expr(&mut self) -> Result<Expr, ParserErr> {
        let begin = self.get_cur_span();
        self.consume_strong_kw(StrongKeyword::Const)?;
        let block = self.parse_block()?;

        let span = self.get_span_to_current(begin);
        Ok(Expr::Block(self.add_node(BlockExpr{
            span,
            node_id: NodeId::default(),
            kind: BlockExprKind::Const,
            block,
        })))
    }

    fn parse_try_block_expr(&mut self) -> Result<Expr, ParserErr> {
        let begin = self.get_cur_span();
        let kind = if self.try_consume(Token::StrongKw(StrongKeyword::TryExclaim)) {
            BlockExprKind::TryUnwrap
        } else {
            self.consume_strong_kw(StrongKeyword::Try)?;
            BlockExprKind::Try
        };
        let block = self.parse_block()?;

        let span = self.get_span_to_current(begin);
        Ok(Expr::Block(self.add_node(BlockExpr {
            span,
            node_id: NodeId::default(),
            kind,
            block,
        })))
    }

    fn parse_prefix_expr(&mut self) -> Result<Expr, ParserErr> {
        let begin = self.get_cur_span();
        let op = self.consume_any_punct()?;
        let expr = self.parse_expr(ExprParseMode::Prefix)?;
        let span = self.get_span_to_current(begin);
        Ok(Expr::Prefix(self.add_node(PrefixExpr {
            span,
            node_id: NodeId::default(),
            op,
            expr,
        })))
    }

    fn parse_paren_expr(&mut self) -> Result<Expr, ParserErr> {
        let begin = self.get_cur_span();
        let mut exprs = self.parse_comma_separated_closed(OpenCloseSymbol::Paren, |parser| parser.parse_expr(ExprParseMode::AllowComma))?;
        let span = self.get_span_to_current(begin);
        if exprs.len() == 1 {
            Ok(Expr::Paren(self.add_node(ParenExpr {
                span,
                node_id: NodeId::default(),
                expr: exprs.pop().unwrap(),
            })))
        } else {
            Ok(Expr::Tuple(self.add_node(TupleExpr {
                span,
                node_id: NodeId::default(),
                exprs,
            })))
        }
    }

    fn parse_inplace_expr(&mut self, left: Expr) -> Result<Expr, ParserErr> {
        let begin = left.span();
        self.consume_punct(Punctuation::SingleArrowL)?;
        let right = self.parse_expr(ExprParseMode::AllowComma)?;

        let span = self.get_span_to_current(begin);
        Ok(Expr::Inplace(self.add_node(InplaceExpr {
            span,
            node_id: NodeId::default(),
            left,
            right,
        })))
    }

    fn parse_type_cast(&mut self, expr: Expr) -> Result<Expr, ParserErr> {
        let begin = expr.span();
        if self.try_consume(Token::StrongKw(StrongKeyword::AsQuestion)) {
            let ty = self.parse_type()?;
            let span = self.get_span_to_current(begin);
            Ok(Expr::TypeCast(self.add_node(TypeCastExpr {
                span,
                node_id: NodeId::default(),
                kind: TypeCastKind::Try,
                expr,
                ty,
            })))
        } else if self.try_consume(Token::StrongKw(StrongKeyword::AsExclaim)) {
            let ty = self.parse_type()?;
            let span = self.get_span_to_current(begin);
            Ok(Expr::TypeCast(self.add_node(TypeCastExpr {
                span,
                node_id: NodeId::default(),
                kind: TypeCastKind::Unwrap,
                expr,
                ty,
            })))
        } else {
            self.consume_strong_kw(StrongKeyword::As)?;
            let ty = self.parse_type()?;
            let span = self.get_span_to_current(begin);
            Ok(Expr::TypeCast(self.add_node(TypeCastExpr {
                span,
                node_id: NodeId::default(),
                kind: TypeCastKind::Normal,
                expr,
                ty,
            })))
        }
    }

    fn parse_type_check(&mut self, expr: Expr) -> Result<Expr, ParserErr> {
        let begin = expr.span();
        let negate = if self.try_consume(Token::StrongKw(StrongKeyword::ExclaimIs)) {
            true
        } else {
            self.consume_strong_kw(StrongKeyword::Is)?;
            false
        };
        let ty = self.parse_type()?;
        let span = self.get_span_to_current(begin);
        Ok(Expr::TypeCheck(self.add_node(TypeCheckExpr {
            span,
            node_id: NodeId::default(),
            negate,
            expr,
            ty,
        })))
    }

    fn parse_array_expr(&mut self) -> Result<Expr, ParserErr> {
        let begin = self.get_cur_span();
        self.begin_scope(OpenCloseSymbol::Brace)?;
        let val = self.parse_expr(ExprParseMode::General)?;
        if self.try_consume(Token::Punctuation(Punctuation::Semicolon)) {
            let count = self.parse_expr(ExprParseMode::General)?;
            self.end_scope()?;
            let span = self.get_span_to_current(begin);
            Ok(Expr::Array(self.add_node(ArrayExpr::Count {
                span,
                node_id: NodeId::default(),
                val,
                count,
            })))
        } else {
            let exprs = if self.try_consume(Token::Punctuation(Punctuation::Comma)) {
                let mut exprs = vec![val];
                while !self.try_end_scope() {
                    exprs.push(self.parse_expr(ExprParseMode::General)?);
                    if !self.try_consume(Token::Punctuation(Punctuation::Comma)) {
                        break;
                    }
                }
                exprs
            } else {
                vec![val]
            };
            let span = self.get_span_to_current(begin);
            Ok(Expr::Array(self.add_node(ArrayExpr::Slice {
                span,
                node_id: NodeId::default(),
                exprs,
            })))
        }
    }

    fn parse_struct_expr(&mut self, path: Expr, allow: bool) -> Result<Expr, ParserErr> {
        let begin = self.get_cur_span();
        if !allow {
            let peek_1 = self.peek_at(1)?;
            let peek_2 = self.peek_at(2)?;
            if matches!(peek_1, Token::Name(_)) && peek_2 == Token::Punctuation(Punctuation::Colon) {
                return Err(self.gen_error(ParseErrorCode::ExprNotSupported { expr: "Struct Expression", loc: "for loop's source value" }));
            }

            return Ok(path);
        }

        let args = self.parse_comma_separated_closed(OpenCloseSymbol::Brace, Self::parse_struct_arg)?;

        if !allow {
            return Err(self.gen_error(ParseErrorCode::ExprNotSupported { expr: "Struct Expression", loc: "for loop's source value" }));
        }

        let span = self.get_span_to_current(begin);
        Ok(Expr::Struct(self.add_node(StructExpr {
            span,
            node_id: NodeId::default(),
            path,
            args,
        })))
    }

    fn parse_struct_arg(&mut self) -> Result<StructArg, ParserErr> {
        let begin = self.get_cur_span();
        let peek = self.peek()?;
        match peek {
            Token::Name(_) => if self.peek_at(1)? == Token::Punctuation(Punctuation::Colon) {
                let name = self.consume_name()?;
                self.consume_punct(Punctuation::Colon);
                let expr = self.parse_expr(ExprParseMode::General)?;
                let span = self.get_span_to_current(begin);
                Ok(StructArg::Expr{ span ,name, expr })
            } else {
                let name = self.consume_name()?;
                let span = self.get_span_to_current(begin);
                Ok(StructArg::Name{ span, name })
            },
            Token::Punctuation(Punctuation::DotDot) => {
                self.consume_single();
                let expr = self.parse_expr(ExprParseMode::General)?;
                let span = self.get_span_to_current(begin);
                Ok(StructArg::Complete{ span, expr })
            },
            _ => Err(self.gen_error(ParseErrorCode::UnexpectedFor{ found: peek, for_reason: "struct argument" }))
        }
    }

    fn parse_index_expr(&mut self, expr: Expr) -> Result<Expr, ParserErr> {
        let begin = expr.span();
        self.begin_scope(OpenCloseSymbol::Bracket)?;
        let is_opt = self.try_consume(Token::Punctuation(Punctuation::Question));
        let index = self.parse_expr(ExprParseMode::AllowComma)?;

        let span = self.get_span_to_current(begin);
        Ok(Expr::Index(self.add_node(IndexExpr {
            span,
            node_id: NodeId::default(),
            is_opt,
            expr,
            index,
        })))
    }

    fn parse_tuple_index(&mut self, expr: Expr) -> Result<Expr, ParserErr> {
        let begin = expr.span();
        self.consume_punct(Punctuation::Dot);
        let index = self.consume_lit()?;
        
        let span = self.get_span_to_current(begin);
        Ok(Expr::TupleIndex(self.add_node(TupleIndexExpr {
            span,
            node_id: NodeId::default(),
            expr,
            index,
        })))
    }

    fn parse_call_expression(&mut self, expr: Expr) -> Result<Expr, ParserErr> {
        let begin = expr.span();
        let args = self.parse_comma_separated_closed(OpenCloseSymbol::Paren, Self::parse_func_arg)?;
        
        let span = self.get_span_to_current(begin);
        Ok(Expr::FnCall(self.add_node(FnCallExpr{
            span,
            node_id: NodeId::default(),
            expr,
            args
        })))
    }

    fn parse_func_arg(&mut self) -> Result<FnArg, ParserErr> {
        let begin = self.get_cur_span();

        if matches!(self.peek()?, Token::Name(_)) && self.peek_at(1)? == Token::Punctuation(Punctuation::Colon) {
            let label = self.consume_name()?;
            self.consume_punct(Punctuation::Colon);
            let expr = self.parse_expr(ExprParseMode::General)?;
            let span = self.get_span_to_current(begin);
            Ok(FnArg::Labeled { span, label, expr })
        } else {
            let expr = self.parse_expr(ExprParseMode::General)?;
            let span = self.get_span_to_current(begin);
            Ok(FnArg::Expr{ span, expr })
        }
    }

    fn parse_field_access_or_method_expr(&mut self, expr: Expr) -> Result<Expr, ParserErr> {
        let begin = self.get_cur_span();
        let is_propagating = if self.try_consume(Token::Punctuation(Punctuation::QuestionDot)) {
            true
        } else {
            self.consume_punct(Punctuation::Dot)?;
            false
        };

        let field = self.parse_identifier(true)?;

        if self.peek()? == Token::OpenSymbol(OpenCloseSymbol::Paren) {
            let args = self.parse_comma_separated_closed(OpenCloseSymbol::Paren, Self::parse_func_arg)?;
            let span = self.get_span_to_current(begin);
            Ok(Expr::Method(self.add_node(MethodCallExpr {
                span,
                node_id: NodeId::default(),
                receiver: expr,
                method: field,
                args,
                is_propagating,
            })))
        } else {
            let span = self.get_span_to_current(begin);
            Ok(Expr::FieldAccess(self.add_node(FieldAccessExpr {
                span,
                node_id: NodeId::default(),
                expr,
                field,
                is_propagating,
            })))
        }
    }

    fn parse_closure_expr(&mut self) -> Result<Expr, ParserErr> {
        let begin = self.get_cur_span();
        let is_moved = self.try_consume(Token::StrongKw(StrongKeyword::Move));
        self.consume_punct(Punctuation::Or)?;
        let params = self.parse_punct_separated_end(Punctuation::Comma, Token::Punctuation(Punctuation::Or), Self::parse_function_param)?;
        self.consume_punct(Punctuation::Or)?;

        let ret = if self.try_consume(Token::Punctuation(Punctuation::SingleArrowR)) {
            Some(self.parse_func_return()?)
        } else {
            None
        };

        let body = self.parse_expr(ExprParseMode::General)?;

        let span = self.get_span_to_current(begin);
        Ok(Expr::Closure(self.add_node(ClosureExpr {
            span,
            node_id: NodeId::default(),
            is_moved,
            params,
            ret,
            body,
        })))
    }

    fn parse_if_expr(&mut self) -> Result<Expr, ParserErr> {
        let begin = self.get_cur_span();
        self.consume_strong_kw(StrongKeyword::If)?;
        let cond = self.parse_expr(ExprParseMode::AllowLet)?;
        
        self.push_meta_frame();
        let body = self.parse_block_expr(self.get_cur_span(), None)?;

        let else_body = if self.try_consume(Token::StrongKw(StrongKeyword::Else)) {
            if self.peek()? == Token::StrongKw(StrongKeyword::If) {
                Some(self.parse_if_expr()?)
            } else {
                self.push_meta_frame();
                Some(Expr::Block(self.parse_block_expr(self.get_cur_span(), None)?))
            }
        } else {
            None
        };

        let span = self.get_span_to_current(begin);
        Ok(Expr::If(self.add_node(IfExpr {
            span,
            node_id: NodeId::default(),
            cond,
            body,
            else_body,
        })))
    }

    fn parse_let_binding_expr(&mut self) -> Result<Expr, ParserErr> {
        let begin = self.get_cur_span();
        self.consume_strong_kw(StrongKeyword::Let)?;
        self.push_meta_frame();
        let pattern = self.parse_pattern_no_top_alternative()?;
        self.consume_punct(Punctuation::Equals)?;
        let scrutinee = self.parse_expr(ExprParseMode::Scrutinee)?;

        let span = self.get_span_to_current(begin);
        Ok(Expr::Let(self.add_node(LetBindingExpr {
            span,
            node_id: NodeId::default(),
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

    fn parse_loop_expr(&mut self, begin: SpanId, label: Option<NameId>) -> Result<Expr, ParserErr> {
        self.consume_strong_kw(StrongKeyword::Loop)?;
        let body = self.parse_block_expr(self.get_cur_span(), None)?;
        let span = self.get_span_to_current(begin);
        Ok(Expr::Loop(self.add_node(LoopExpr {
            span,
            node_id: NodeId::default(),
            label,
            body,
        })))
    }

    fn parse_while_expr(&mut self, begin: SpanId, label: Option<NameId>) -> Result<Expr, ParserErr> {
        self.consume_strong_kw(StrongKeyword::While)?;
        let cond = self.parse_expr(ExprParseMode::AllowLet)?;
        let inc = if self.try_consume(Token::Punctuation(Punctuation::Semicolon)) {
            Some(self.parse_expr(ExprParseMode::General)?)
        } else {
            None
        };
        
        self.push_meta_frame();
        let body = self.parse_block_expr(self.get_cur_span(), None)?;
        let else_body = if self.try_consume(Token::StrongKw(StrongKeyword::Else)) {
            self.push_meta_frame();
            let else_body = self.parse_block_expr(self.get_cur_span(), None)?;
            Some(else_body)
        } else {
            None
        };

        let span = self.get_span_to_current(begin);
        Ok(Expr::While(self.add_node(WhileExpr {
            span,
            node_id: NodeId::default(),
            label,
            cond,
            inc,
            body,
            else_body,
        })))
    }

    fn parse_do_while_expr(&mut self, begin: SpanId, label: Option<NameId>) -> Result<Expr, ParserErr> {
        self.consume_strong_kw(StrongKeyword::Do)?;
        let body = self.parse_block_expr(self.get_cur_span(), None)?;
        self.consume_strong_kw(StrongKeyword::While)?;
        let cond = self.parse_expr(ExprParseMode::General)?;
        
        let span = self.get_span_to_current(begin);
        Ok(Expr::DoWhile(self.add_node(DoWhileExpr {
            span,
            node_id: NodeId::default(),
            label,
            body,
            cond,
        })))
    }

    fn parse_for_expr(&mut self, begin: SpanId, label: Option<NameId>) -> Result<Expr, ParserErr> {
        self.consume_strong_kw(StrongKeyword::For)?;
        let pattern = self.parse_pattern()?;
        self.consume_strong_kw(StrongKeyword::In)?;
        let src = self.parse_expr(ExprParseMode::NoStructLit)?;
        
        self.push_meta_frame();
        let body = self.parse_block_expr(self.get_cur_span(), None)?;
        let else_body = if self.try_consume(Token::StrongKw(StrongKeyword::Else)) {
            self.push_meta_frame();
            Some(self.parse_block_expr(self.get_cur_span(), None)?)
        } else {
            None
        };

        let span = self.get_span_to_current(begin);
        Ok(Expr::For(self.add_node(ForExpr {
            span,
            node_id: NodeId::default(),
            label,
            pattern,
            src,
            body,
            else_body,
        })))
    }

    fn parse_match_expr(&mut self, begin: SpanId, label: Option<NameId>) -> Result<Expr, ParserErr> {
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
        
        let span = self.get_span_to_current(begin);
        Ok(Expr::Match(self.add_node(MatchExpr {
            span,
            node_id: NodeId::default(),
            label,
            scrutinee,
            branches,
        })))
    }

    fn parse_match_branch(&mut self) -> Result<MatchBranch, ParserErr> {
        let begin = self.get_cur_span();
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

        let span = self.get_span_to_current(begin);
        Ok(MatchBranch {
            span,
            label,
            pattern,
            guard,
            body,
        })
    }
    
    fn parse_break_expr(&mut self) -> Result<Expr, ParserErr> {
        let begin = self.get_cur_span();
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

        let span = self.get_span_to_current(begin);
        Ok(Expr::Break(self.add_node(BreakExpr {
            span,
            node_id: NodeId::default(),
            label,
            value,
        })))
    }
    
    fn parse_continue_expr(&mut self) -> Result<Expr, ParserErr> {
        let begin = self.get_cur_span();
        self.consume_strong_kw(StrongKeyword::Continue);
        let label = if self.peek()? == Token::Punctuation(Punctuation::Colon) {
            Some(self.parse_label()?)
        } else {
            None
        };

        let span = self.get_span_to_current(begin);
        Ok(Expr::Continue(self.add_node(ContinueExpr {
            span,
            node_id: NodeId::default(),
            label,
        })))
    }
    
    fn parse_fallthrough_expr(&mut self) -> Result<Expr, ParserErr> {
        let begin = self.get_cur_span();
        self.consume_strong_kw(StrongKeyword::Fallthrough);
        let label = if self.peek()? == Token::Punctuation(Punctuation::Colon) {
            Some(self.parse_label()?)
        } else {
            None
        };

        let span = self.get_span_to_current(begin);
        Ok(Expr::Fallthrough(self.add_node(FallthroughExpr {
            span,
            node_id: NodeId::default(),
            label,
        })))
    }
    
    fn parse_return_expr(&mut self) -> Result<Expr, ParserErr> {
        let begin = self.get_cur_span();
        self.consume_strong_kw(StrongKeyword::Return);
        let value = if self.peek()? != Token::Punctuation(Punctuation::Semicolon) {
            Some(self.parse_expr(ExprParseMode::AllowComma)?)
        } else {
            None
        };

        let span = self.get_span_to_current(begin);
        Ok(Expr::Return(self.add_node(ReturnExpr {
            span,
            node_id: NodeId::default(),
            value,
        })))
    }

    fn parse_throw_expr(&mut self) -> Result<Expr, ParserErr> {
        let begin = self.get_cur_span();
        self.consume_strong_kw(StrongKeyword::Throw)?;
        let expr = self.parse_expr(ExprParseMode::General)?;
        
        let span = self.get_span_to_current(begin);
        Ok(Expr::Throw(self.add_node(ThrowExpr {
            span,
            node_id: NodeId::default(),
            expr,
        })))
    }

    fn parse_comma_expr(&mut self, first: Expr) -> Result<Expr, ParserErr> {
        let begin = self.get_cur_span();
        self.consume_punct(Punctuation::Comma);

        let mut exprs = vec![first];
        loop {
            exprs.push(self.parse_expr(ExprParseMode::General)?);
            if !self.try_consume(Token::Punctuation(Punctuation::Comma)) {
                break;
            }
        }

        let span = self.get_span_to_current(begin);
        Ok(Expr::Comma(self.add_node(CommaExpr {
            span,
            node_id: NodeId::default(),
            exprs,
        })))
    }

    fn parse_when_expr(&mut self) -> Result<Expr, ParserErr> {
        let begin = self.get_cur_span();
        self.consume_strong_kw(StrongKeyword::When)?;
        let cond = self.parse_expr(ExprParseMode::NoStructLit)?;

        self.push_meta_frame();
        let body = self.parse_block_expr(self.get_cur_span(), None)?;

        let else_body = if self.try_consume(Token::StrongKw(StrongKeyword::Else)) {
            if self.peek()? == Token::StrongKw(StrongKeyword::If) {
                Some(self.parse_if_expr()?)
            } else {  
                self.push_meta_frame();
                Some(Expr::Block(self.parse_block_expr(self.get_cur_span(), None)?))
            }
        } else {
            None
        };

        let span = self.get_span_to_current(begin);
        Ok(Expr::When(self.add_node(WhenExpr {
            span,
            node_id: NodeId::default(),
            cond,
            body,
            else_body,
        })))
    }

// =============================================================================================================================

    fn parse_pattern(&mut self) -> Result<Pattern, ParserErr> {
        let begin = self.get_cur_span();
        self.push_meta_frame();
        
        let mut patterns = self.parse_punct_separated(Punctuation::Or, Self::parse_pattern_no_top_alternative)?;
        if patterns.len() == 1 {
            Ok(patterns.pop().unwrap())
        } else {
            let span = self.get_span_to_current(begin);
            Ok(Pattern::Alternative(self.add_node(AlternativePattern {
                span,
                node_id: NodeId::default(),
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
                let span = self.get_cur_span();
                self.consume_single();
                Pattern::Wildcard(self.add_node(WildcardPattern {
                    span,
                    node_id: NodeId::default(),
                }))
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
        let begin = self.get_cur_span();
        let lit = self.parse_literal_expr_node()?;
        
        let span = self.get_span_to_current(begin);
        Ok(Pattern::Literal(self.add_node(LiteralPattern {
            span,
            node_id: NodeId::default(),
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
        let begin = self.get_cur_span();
        let is_ref = self.try_consume(Token::StrongKw(StrongKeyword::Ref));
        let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));

        let name = self.consume_name()?;

        let bound = if self.try_consume(Token::Punctuation(Punctuation::At)) {
            Some(self.parse_pattern()?)
        } else {
            None
        };

        let span = self.get_span_to_current(begin);
        Ok(Pattern::Identifier(self.add_node(IdentifierPattern {
            span,
            node_id: NodeId::default(),
            is_ref,
            is_mut,
            name,
            bound,
        })))
    }

    fn parse_dotdot_pattern(&mut self) -> Result<Pattern, ParserErr> {
        let begin = self.get_cur_span();
        self.consume_punct(Punctuation::DotDot)?;
        if self.pattern_available() {
            self.push_meta_frame();
            let end = self.parse_pattern_no_top_alternative()?;
            let span = self.get_span_to_current(begin);
            Ok(Pattern::Range(self.add_node(RangePattern::To { span, node_id: NodeId::default(), end })))
        } else {
            Ok(Pattern::Rest(self.add_node(RestPattern {
                span: begin,
                node_id: NodeId::default(),
            })))
        }
    }

    fn parse_inclusive_to_pattern(&mut self) -> Result<Pattern, ParserErr> {
        let begin = self.get_cur_span();
        self.consume_punct(Punctuation::DotDotEquals)?;
        self.push_meta_frame();
        let end = self.parse_pattern_no_top_alternative()?;
        
        let span = self.get_span_to_current(begin);
        Ok(Pattern::Range(self.add_node(RangePattern::InclusiveTo { span, node_id: NodeId::default(), end })))
    }

    fn parse_path_like_pattern(&mut self) -> Result<Pattern, ParserErr> {
        let begin = self.get_cur_span();
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

            let span = self.get_span_to_current(begin);
            return Ok(Pattern::Identifier(self.add_node(IdentifierPattern {
                span,
                node_id: NodeId::default(),
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
                    let span = self.get_span_to_current(begin);
                    Ok(Pattern::Tuple(self.add_node(TuplePattern { span, node_id: NodeId::default(), patterns })))
                },
                OpenCloseSymbol::Brace => {
                    let fields = self.parse_comma_separated_closed(sym, Self::parse_struct_pattern_field)?;
                    let span = self.get_span_to_current(begin);
                    Ok(Pattern::Struct(self.add_node(StructPattern::Path{ span, node_id: NodeId::default(), path, fields })))
                },
                _ => Err(self.gen_error(ParseErrorCode::UnexpectedFor{ found: Token::OpenSymbol(sym), for_reason: "pattern" })),
            }

        } else {
            let span = self.get_span_to_current(begin);
            Ok(Pattern::Path(self.add_node(PathPattern{ span, node_id: NodeId::default(), path })))
        }
    }

    fn parse_struct_pattern_field(&mut self) -> Result<StructPatternField, ParserErr> {
        let begin = self.get_cur_span();
        let peek = self.peek()?;
        match peek {
            Token::Name(name) => {
                self.consume_single();
                if self.try_consume(Token::Punctuation(Punctuation::Colon)) {
                    let pattern = self.parse_pattern()?;
                    let span = self.get_span_to_current(begin);
                    Ok(StructPatternField::Named { span, name, pattern })
                } else {
                    let bound = if self.try_consume(Token::Punctuation(Punctuation::At)) {
                        Some(self.parse_pattern()?)
                    } else {
                        None
                    };
                    let span = self.get_span_to_current(begin);
                    Ok(StructPatternField::Iden { span, is_ref: false, is_mut: false, iden: name, bound })
                }
            },
            Token::Literal(lit_id) => {
                self.consume_single();
                self.consume_punct(Punctuation::Colon)?;
                let pattern = self.parse_pattern()?;
                let span = self.get_span_to_current(begin);
                Ok(StructPatternField::TupleIndex { span, idx: lit_id, pattern })
            },
            Token::StrongKw(StrongKeyword::Mut) => {
                self.consume_single();
                let iden = self.consume_name()?;
                let bound = if self.try_consume(Token::Punctuation(Punctuation::At)) {
                    Some(self.parse_pattern()?)
                } else {
                    None
                };
                let span = self.get_span_to_current(begin);
                Ok(StructPatternField::Iden { span, is_ref: false, is_mut: true, iden, bound })
            },
            Token::Punctuation(Punctuation::Ampersand) => {
                self.consume_single();
                let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
                let iden = self.consume_name()?;
                let bound = if self.try_consume(Token::Punctuation(Punctuation::At)) {
                    Some(self.parse_pattern()?)
                } else {
                    None
                };
                let span = self.get_span_to_current(begin);
                Ok(StructPatternField::Iden { span, is_ref: true, is_mut, iden, bound })
            },
            Token::Punctuation(Punctuation::DotDot) => {
                self.consume_single();
                Ok(StructPatternField::Rest)
            }
            _ => Err(self.gen_error(ParseErrorCode::UnexpectedFor { found: peek, for_reason: "struct pattern field" }))
        }
    }

    fn parse_inferred_struct_pattern(&mut self) -> Result<Pattern, ParserErr> {
        let begin = self.get_cur_span();
        self.consume_punct(Punctuation::Dot)?;
        let fields = self.parse_comma_separated_closed(OpenCloseSymbol::Brace, Self::parse_struct_pattern_field)?;
        let span = self.get_span_to_current(begin);
        Ok(Pattern::Struct(self.add_node(StructPattern::Inferred { span, node_id: NodeId::default(), fields })))
    }

    fn parse_enum_member_pattern(&mut self) -> Result<Pattern, ParserErr> {
        let begin = self.get_cur_span();
        self.consume_punct(Punctuation::Dot)?;
        let name = self.consume_name()?;
        let span = self.get_span_to_current(begin);
        Ok(Pattern::EnumMember(self.add_node(EnumMemberPattern {
            span,
            node_id: NodeId::default(),
            name,
        })))
    }

    fn parse_struct_pattern_elem(&mut self) -> Result<StructPatternField, ParserErr> {
        let begin = self.get_cur_span();
        if self.try_consume(Token::Punctuation(Punctuation::DotDot)) {
            return Ok(StructPatternField::Rest);
        }

        match self.peek()? {
            Token::StrongKw(StrongKeyword::Ref | StrongKeyword::Mut) => {
                let is_ref = self.try_consume(Token::StrongKw(StrongKeyword::Ref));
                let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
                let iden = self.consume_name()?;
                let bound = if self.try_consume(Token::Punctuation(Punctuation::At)) {
                    Some(self.parse_pattern()?)
                } else {
                    None
                };
                let span = self.get_span_to_current(begin);
                Ok(StructPatternField::Iden { span, is_ref, is_mut, iden, bound })
            }
            Token::Literal(lit_id) => {
                self.consume_single();
                self.consume_punct(Punctuation::Colon)?;
                let pattern = self.parse_pattern()?;

                let span = self.get_span_to_current(begin);
                Ok(StructPatternField::TupleIndex { span, idx: lit_id, pattern })
            },
            Token::Name(iden) => {
                self.consume_single();
                if !self.try_consume(Token::Punctuation(Punctuation::Colon)) {
                    let bound = if self.try_consume(Token::Punctuation(Punctuation::At)) {
                        Some(self.parse_pattern()?)
                    } else {
                        None
                    };
                    let span = self.get_span_to_current(begin);
                    Ok(StructPatternField::Iden { span, is_ref: false, is_mut: false, iden, bound })
                } else {
                    let pattern = self.parse_pattern()?;
                    let span = self.get_span_to_current(begin);
                    Ok(StructPatternField::Named { span, name: iden, pattern })
                }
            }
            _ => Err(self.gen_error(ParseErrorCode::UnexpectedFor{ found: self.peek()?, for_reason: "struct pattern element" }))
        }
    }

    fn parse_reference_pattern(&mut self) -> Result<Pattern, ParserErr> {
        let begin = self.get_cur_span();
        self.consume_punct(Punctuation::Ampersand)?;
        let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
        let pattern = self.parse_pattern()?;

        let span = self.get_span_to_current(begin);
        Ok(Pattern::Reference(self.add_node(ReferencePattern {
            span,
            node_id: NodeId::default(),
            is_mut,
            pattern,
        } )))
    }

    fn parse_tuple_like_pattern(&mut self) -> Result<Pattern, ParserErr> {
        let begin = self.get_cur_span();
        let mut patterns = self.parse_comma_separated_closed(OpenCloseSymbol::Paren, Self::parse_pattern)?;
        if patterns.len() == 1 {
            let span = self.get_span_to_current(begin);
            Ok(Pattern::Grouped(self.add_node(GroupedPattern {
                span,
                node_id: NodeId::default(),
                pattern: patterns.pop().unwrap()
            })))
        } else {
            let span = self.get_span_to_current(begin);
            Ok(Pattern::Tuple(self.add_node(TuplePattern{
                span,
                node_id: NodeId::default(),
                patterns
            })))
        }
    }

    fn parse_slice_pattern(&mut self) -> Result<Pattern, ParserErr> {
        let begin = self.get_cur_span();
        let patterns = self.parse_comma_separated_closed(OpenCloseSymbol::Bracket, Self::parse_pattern)?;
        let span = self.get_span_to_current(begin);
        Ok(Pattern::Slice(self.add_node(SlicePattern { span, node_id: NodeId::default(), patterns })))
    }

    fn parse_type_check_pattern(&mut self) -> Result<Pattern, ParserErr> {
        let begin = self.get_cur_span();
        self.consume_strong_kw(StrongKeyword::Is)?;
        let ty = self.parse_type()?;
        let span = self.get_span_to_current(begin);
        Ok(Pattern::TypeCheck(self.add_node(TypeCheckPattern { span, node_id: NodeId::default(), ty })))
    }

    fn parse_range_pattern(&mut self, begin: Pattern) -> Result<Pattern, ParserErr> {
        let begin_span = begin.span();

        self.consume_punct(Punctuation::DotDot)?;
        if self.pattern_available() {
            self.push_meta_frame();
            let end = self.parse_pattern_no_top_alternative()?;
            let span = self.get_span_to_current(begin_span);
            Ok(Pattern::Range(self.add_node(RangePattern::Exclusive { span, node_id: NodeId::default(), begin, end })))
        } else {
            let span = self.get_span_to_current(begin_span);
            Ok(Pattern::Range(self.add_node(RangePattern::From { span, node_id: NodeId::default(), begin })))
        }
    }
    
    fn parse_inclusive_range_pattern(&mut self, begin: Pattern) -> Result<Pattern, ParserErr> {
        let begin_span = begin.span();
        self.consume_punct(Punctuation::DotDotEquals)?;
        let end = self.parse_pattern()?;
        let span = self.get_span_to_current(begin_span);
        Ok(Pattern::Range(self.add_node(RangePattern::Inclusive { span, node_id: NodeId::default(), begin, end })))
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
        let span = self.get_cur_span();
        match peek {
            Token::OpenSymbol(OpenCloseSymbol::Paren) => self.parse_tuple_like_type(),
            Token::Punctuation(Punctuation::Exclaim) => {
                self.consume_single();
                Ok(Type::Never(self.add_node(NeverType {
                    span,
                    node_id: NodeId::default(),
                })))
            },
            Token::OpenSymbol(OpenCloseSymbol::Bracket) => self.parse_slice_like_type(),
            Token::Punctuation(Punctuation::Caret)      => self.parse_pointer_type(),
            Token::Punctuation(Punctuation::Ampersand)  => self.parse_reference_type(),
            Token::Punctuation(Punctuation::Question)   => self.parse_optional_type(),
            Token::StrongKw(StrongKeyword::Unsafe       |
                StrongKeyword::Extern                   |
                StrongKeyword::Fn)                      => self.parse_fn_type(),
            Token::StrongKw(StrongKeyword::Enum)        => self.parse_enum_record_type(),
            Token::StrongKw(StrongKeyword::Struct)      => self.parse_record_type(),
            Token::StrongKw(StrongKeyword::SelfTy)      => self.parse_path_type(),
            Token::StrongKw(kw)                         => self.parse_type_from_strong_kw(span, kw),
            _                                           => self.parse_path_type(),
        }
    }

    fn parse_tuple_like_type(&mut self) -> Result<Type, ParserErr> {
        let begin = self.get_cur_span();
        let mut types = self.parse_comma_separated_closed(OpenCloseSymbol::Paren, Self::parse_type)?;
        let span = self.get_span_to_current(begin);
        if types.is_empty() {
            Ok(Type::Unit(self.add_node(UnitType {
                span,
                node_id: NodeId::default(),
            })))
        } else if types.len() == 1 {
            Ok(Type::Paren(self.add_node(ParenthesizedType {
                span,
                node_id: NodeId::default(),
                ty: types.pop().unwrap(),
            })))
        } else {
            Ok(Type::Tuple(self.add_node(TupleType {
                span,
                node_id: NodeId::default(),
                types,
            })))
        }
    }

    fn parse_type_from_strong_kw(&mut self, span: SpanId, kw: StrongKeyword) -> Result<Type, ParserErr> {
        let ty = match kw {
            StrongKeyword::U8     => Type::Primitive(self.add_node(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::U8 })),
            StrongKeyword::U16    => Type::Primitive(self.add_node(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::U16 })),
            StrongKeyword::U32    => Type::Primitive(self.add_node(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::U32 })),
            StrongKeyword::U64    => Type::Primitive(self.add_node(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::U64 })),
            StrongKeyword::U128   => Type::Primitive(self.add_node(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::U128 })),
            StrongKeyword::Usize  => Type::Primitive(self.add_node(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::Usize })),
            StrongKeyword::I8     => Type::Primitive(self.add_node(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::I8 })),
            StrongKeyword::I16    => Type::Primitive(self.add_node(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::I16 })),
            StrongKeyword::I32    => Type::Primitive(self.add_node(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::I32 })),
            StrongKeyword::I64    => Type::Primitive(self.add_node(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::I64 })),
            StrongKeyword::I128   => Type::Primitive(self.add_node(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::I128 })),
            StrongKeyword::Isize  => Type::Primitive(self.add_node(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::Isize })),
            StrongKeyword::F16    => Type::Primitive(self.add_node(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::F16 })),
            StrongKeyword::F32    => Type::Primitive(self.add_node(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::F32 })),
            StrongKeyword::F64    => Type::Primitive(self.add_node(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::F64 })),
            StrongKeyword::F128   => Type::Primitive(self.add_node(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::F128 })),
            StrongKeyword::Bool   => Type::Primitive(self.add_node(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::Bool })),
            StrongKeyword::B8     => Type::Primitive(self.add_node(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::B8 })),
            StrongKeyword::B16    => Type::Primitive(self.add_node(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::B16 })),
            StrongKeyword::B32    => Type::Primitive(self.add_node(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::B32 })),
            StrongKeyword::B64    => Type::Primitive(self.add_node(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::B64 })),
            StrongKeyword::Char   => Type::Primitive(self.add_node(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::Char })),
            StrongKeyword::Char7  => Type::Primitive(self.add_node(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::Char7 })),
            StrongKeyword::Char8  => Type::Primitive(self.add_node(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::Char8 })),
            StrongKeyword::Char16 => Type::Primitive(self.add_node(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::Char16 })),
            StrongKeyword::Char32 => Type::Primitive(self.add_node(PrimitiveType { span, node_id: NodeId::default(), ty: type_system::PrimitiveType::Char32 })),
            StrongKeyword::Str    => Type::StringSlice(self.add_node(StringSliceType { span, node_id: NodeId::default(), ty: type_system::StringSliceType::Str })),
            StrongKeyword::Str7   => Type::StringSlice(self.add_node(StringSliceType { span, node_id: NodeId::default(), ty: type_system::StringSliceType::Str7 })),
            StrongKeyword::Str8   => Type::StringSlice(self.add_node(StringSliceType { span, node_id: NodeId::default(), ty: type_system::StringSliceType::Str8 })),
            StrongKeyword::Str16  => Type::StringSlice(self.add_node(StringSliceType { span, node_id: NodeId::default(), ty: type_system::StringSliceType::Str16 })),
            StrongKeyword::Str32  => Type::StringSlice(self.add_node(StringSliceType { span, node_id: NodeId::default(), ty: type_system::StringSliceType::Str32 })),
            StrongKeyword::CStr   => Type::StringSlice(self.add_node(StringSliceType { span, node_id: NodeId::default(), ty: type_system::StringSliceType::CStr })),
            _ => {
                let peek = self.peek()?;
                return Err(self.gen_error(ParseErrorCode::UnexpectedFor{ found: peek, for_reason: "type" }))
            },
        };

        self.consume_single();
        Ok(ty)
    }

    fn parse_path_type(&mut self) -> Result<Type, ParserErr> {
        let path = self.parse_type_path()?;
        let span = path.span;
        Ok(Type::Path(self.add_node(PathType{ span, node_id: NodeId::default(), path })))
    }

    fn parse_slice_like_type(&mut self) -> Result<Type, ParserErr> {
        let begin = self.get_cur_span();
        self.begin_scope(OpenCloseSymbol::Bracket)?;
        let peek = self.peek()?;
        match peek {
            Token::CloseSymbol(OpenCloseSymbol::Bracket) => {
                self.end_scope();
                let ty = self.parse_type_no_bounds()?;
                let span = self.get_span_to_current(begin);
                Ok(Type::Slice(self.add_node(SliceType { span, node_id: NodeId::default(), sentinel: None, ty })))
            },
            Token::Punctuation(Punctuation::Semicolon) => {
                self.consume_single();
                let sentinel = Some(self.parse_expr(ExprParseMode::General)?);
                self.end_scope()?;
                let ty = self.parse_type_no_bounds()?;
                let span = self.get_span_to_current(begin);
                Ok(Type::Slice(self.add_node(SliceType { span, node_id: NodeId::default(), sentinel, ty })))
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
                let span = self.get_span_to_current(begin);
                Ok(Type::Pointer(self.add_node(PointerType { span, node_id: NodeId::default(), is_multi: true, is_mut, sentinel, ty })))
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
                let span = self.get_span_to_current(begin);
                Ok(Type::Array(self.add_node(ArrayType { span, node_id: NodeId::default(), size, sentinel, ty })))
            }
        }
    }

    fn parse_pointer_type(&mut self) -> Result<Type, ParserErr> {
        let begin = self.get_cur_span();
        self.consume_punct(Punctuation::Caret)?;
        let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
        let ty = self.parse_type_no_bounds()?;
        let span = self.get_span_to_current(begin);
        Ok(Type::Pointer(self.add_node(PointerType { span, node_id: NodeId::default(), is_multi: false, is_mut, sentinel: None, ty })))
    }

    fn parse_reference_type(&mut self) -> Result<Type, ParserErr> {
        let begin = self.get_cur_span();
        self.consume_punct(Punctuation::Ampersand)?;
        let is_mut = self.try_consume(Token::StrongKw(StrongKeyword::Mut));
        let ty = self.parse_type_no_bounds()?;
        let span = self.get_span_to_current(begin);
        Ok(Type::Ref(self.add_node(ReferenceType { span, node_id: NodeId::default(), is_mut, ty })))
    }

    fn parse_optional_type(&mut self) -> Result<Type, ParserErr> {
        let begin = self.get_cur_span();
        self.consume_punct(Punctuation::Question)?;
        let ty = self.parse_type_no_bounds()?;
        let span = self.get_span_to_current(begin);
        Ok(Type::Optional(self.add_node(OptionalType { span, node_id: NodeId::default(), ty })))
    }

    fn parse_fn_type(&mut self) -> Result<Type, ParserErr> {
        let begin = self.get_cur_span();
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


        let span = self.get_span_to_current(begin);
        Ok(Type::Fn(self.add_node(FnType {
            span,
            node_id: NodeId::default(),
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
        let begin = self.get_cur_span();
        self.consume_strong_kw(StrongKeyword::Struct);
        let fields = self.parse_comma_separated_closed(OpenCloseSymbol::Brace, Self::parse_struct_field)?;
        let span = self.get_span_to_current(begin);
        Ok(Type::Record(self.add_node(RecordType {
            span,
            node_id: NodeId::default(),
            fields
        })))
    }

    fn parse_enum_record_type(&mut self) -> Result<Type, ParserErr> {
        let begin = self.get_cur_span();
        self.consume_strong_kw(StrongKeyword::Enum);
        let variants = self.parse_comma_separated_closed(OpenCloseSymbol::Brace, Self::parse_enum_variant)?;
        let span = self.get_span_to_current(begin);
        Ok(Type::EnumRecord(self.add_node(EnumRecordType {
            span,
            node_id: NodeId::default(),
            variants
        })))
    }

// =============================================================================================================================

    fn parse_generic_params(&mut self, allow_bounds: bool) -> Result<Option<AstNodeRef<GenericParams>>, ParserErr> {
        let begin = self.get_cur_span();
        if !self.try_begin_scope(OpenCloseSymbol::Bracket) {
            return Ok(None);
        }

        let mut params = Vec::new();
        while !self.try_end_scope() {
            if self.peek()? == Token::OpenSymbol(OpenCloseSymbol::Paren) {
                params.push(self.parse_mutli_parameter_pack()?);
                self.end_scope()?;
                break;
            }
            if self.peek_at(1)? == Token::Punctuation(Punctuation::DotDotDot) {
                params.push(self.parse_single_parameter_pack()?);
                self.end_scope()?;
                break;
            }

            params.push(match self.peek()? {
                Token::StrongKw(StrongKeyword::Is) => self.parse_generic_type_spec()?,
                Token::OpenSymbol(OpenCloseSymbol::Brace) => self.parse_generic_const_spec()?,
                _ => if self.peek_at(1)? == Token::Punctuation(Punctuation::Colon) {
                    self.parse_generic_const_param()?
                } else {
                    self.parse_generic_type_param(allow_bounds)?
                }
            });
        }

        let span = self.get_span_to_current(begin);
        Ok(Some(self.add_node(GenericParams {
            span,
            node_id: NodeId::INVALID,
            params,
        })))
    }

    fn parse_generic_type_param(&mut self, allow_bounds: bool) -> Result<GenericParam, ParserErr> {
        let begin = self.get_cur_span();
        let name = self.consume_name()?;
        let bounds = if allow_bounds && self.try_consume(Token::StrongKw(StrongKeyword::Is))  {
            if !allow_bounds {
                return Err(self.gen_error(ParseErrorCode::GenericTypeBoundsNotAllowed));
            }

            let mut bounds = Vec::new();
            loop {
                let bound = self.parse_generic_type_bound()?;
                bounds.push(bound);
                if !self.try_consume(Token::Punctuation(Punctuation::Or)) {
                    break;
                }
            }
            bounds
        } else {
            Vec::new()
        };

        let def = if self.try_consume(Token::Punctuation(Punctuation::Equals)) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let span = self.get_span_to_current(begin);
        Ok(GenericParam::Type(self.add_node(GenericTypeParam {
            span,
            node_id: NodeId::INVALID,
            name,
            bounds,
            def,
        })))
    }

    fn parse_generic_type_spec(&mut self) -> Result<GenericParam, ParserErr> {
        let begin = self.get_cur_span();
        self.consume_strong_kw(StrongKeyword::Is)?;
        let ty = self.parse_type()?;

        let span = self.get_span_to_current(begin);
        Ok(GenericParam::TypeSpec(self.add_node(GenericTypeSpec {
            span,
            node_id: NodeId::INVALID,
            ty,
        })))
    }

    fn parse_generic_const_param(&mut self) -> Result<GenericParam, ParserErr> {
        let begin = self.get_cur_span();
        let name = self.consume_name()?;
        self.consume_punct(Punctuation::Colon);
        let ty = self.parse_type()?;
        let def = if self.try_consume(Token::Punctuation(Punctuation::Equals)) {
            Some(self.parse_expr(ExprParseMode::General)?)
        } else {
            None
        };

        let span = self.get_span_to_current(begin);
        Ok(GenericParam::Const(self.add_node(GenericConstParam {
            span,
            node_id: NodeId::INVALID,
            name,
            ty,
            def,
        })))
    }

    fn parse_generic_const_spec(&mut self) -> Result<GenericParam, ParserErr> {
        let begin = self.get_cur_span();
        let expr = self.parse_block_expr(begin, None)?;
        Ok(GenericParam::ConstSpec(self.add_node(GenericConstSpec {
            span: expr.span,
            node_id: NodeId::INVALID,
            expr,
        })))
    }

    fn parse_single_parameter_pack(&mut self) -> Result<GenericParam, ParserErr> {
        let begin = self.get_cur_span();
        let name = self.consume_name_and_span()?;
        self.consume_punct(Punctuation::DotDotDot)?;
        let desc = if self.try_consume(Token::Punctuation(Punctuation::Colon)) {
            self.parse_param_pack_desc()?
        } else {
            GenericParamPackDesc::Type(SpanId::INVALID)
        };

        let defs = if self.try_consume(Token::Punctuation(Punctuation::Equals)) {
            vec![self.parse_param_pack_def()?]
        } else {
            Vec::new()
        };

        let span = self.get_span_to_current(begin);
        Ok(GenericParam::Pack(self.add_node(GenericParamPack {
            span,
            node_id: NodeId::INVALID,
            names: vec![name],
            descs: vec![desc],
            defs,
        })))
    }

    fn parse_mutli_parameter_pack(&mut self) -> Result<GenericParam, ParserErr> {
        let begin = self.get_cur_span();
        let names = self.parse_comma_separated_closed(OpenCloseSymbol::Paren, Parser::consume_name_and_span)?;
        self.consume_punct(Punctuation::Colon);
        let descs = self.parse_comma_separated_closed(OpenCloseSymbol::Paren, Parser::parse_param_pack_desc)?;
        let defs = if self.try_consume(Token::Punctuation(Punctuation::Equals)) {
            self.parse_punct_separated(Punctuation::Comma, |parser| parser.parse_param_pack_def())?
        } else {
            Vec::new()
        };

        if names.len() != descs.len() {
            return Err(self.gen_error(ParseErrorCode::ParamPackNameDescMismatch { name_count: names.len() as u32, desc_count: descs.len() as u32 }));
        }
        if defs.len() % names.len() != 0 {
            return Err(self.gen_error(ParseErrorCode::ParamPackDefMisMatch { elem_count: names.len() as u32, def_count: defs.len() as u32 }));
        }

        let span = self.get_span_to_current(begin);
        Ok(GenericParam::Pack(self.add_node(GenericParamPack {
            span,
            node_id: NodeId::INVALID,
            names,
            descs,
            defs,
        })))
    }

    fn parse_param_pack_desc(&mut self) -> Result<GenericParamPackDesc, ParserErr> {
        match self.peek()? {
            Token::StrongKw(StrongKeyword::Type) => {
                let (_, span) = self.consume_single();
                Ok(GenericParamPackDesc::Type(span))
            },
            Token::StrongKw(StrongKeyword::Is) => {
                let begin = self.get_cur_span();
                self.consume_strong_kw(StrongKeyword::Is)?;
                let bounds = self.parse_punct_separated(Punctuation::Ampersand, Parser::parse_generic_type_bound)?;
                let span = self.get_span_to_current(begin);
                Ok(GenericParamPackDesc::TypeBounds(span, bounds))
            },
            _ => {
                let ty = self.parse_type()?;
                Ok(GenericParamPackDesc::Expr(ty))
            }
        }
    }

    fn parse_param_pack_def(&mut self) -> Result<GenericParamPackDef, ParserErr> {
        if self.peek()? == Token::OpenSymbol(OpenCloseSymbol::Brace) {
            let begin = self.get_cur_span();
            let block_expr = self.parse_block_expr(begin, None)?;
            Ok(GenericParamPackDef::Expr(block_expr))
        } else {
            let ty = self.parse_type()?;
            Ok(GenericParamPackDef::Type(ty))
        }
    }

    pub fn parse_generic_type_bound(&mut self) -> Result<GenericTypeBound, ParserErr> {
        let path = self.parse_type_path()?;
        Ok(GenericTypeBound::Type(path))
    }

    fn parse_generic_args(&mut self, start_with_dot: bool) -> Result<Option<AstNodeRef<GenericArgs>>, ParserErr> {
        let begin = self.get_cur_span();
        if start_with_dot {
            if self.try_peek_at(1) != Some(Token::OpenSymbol(OpenCloseSymbol::Bracket)) || !self.try_consume(Token::Punctuation(Punctuation::Dot)) {
                return Ok(None);
            }
        } else {
            if self.peek()? != Token::OpenSymbol(OpenCloseSymbol::Bracket) {
                return Ok(None);
            }
        }
        let args = self.parse_comma_separated_closed(OpenCloseSymbol::Bracket, Self::parse_generic_arg)?;

        let span = self.get_span_to_current(begin);
        Ok(Some(self.add_node(GenericArgs {
            span,
            node_id: NodeId::INVALID,
            args,
        })))
    }

    fn parse_generic_arg(&mut self) -> Result<GenericArg, ParserErr> {
        let peek = self.peek()?;
        match peek {
            Token::OpenSymbol(OpenCloseSymbol::Brace) => {
                let begin = self.get_cur_span();
                let expr = self.parse_block_expr(begin, None)?;
                Ok(GenericArg::Value(expr))  
            },
            Token::Name(name_id) => {
                let peek_1 = self.peek_at(1)?;
                if matches!(peek_1, Token::Punctuation(Punctuation::Comma) | Token::CloseSymbol(_)) {
                    self.consume_single();
                    Ok(GenericArg::TypeOrValue(name_id))
                } else {
                    let ty = self.parse_type()?;
                    Ok(GenericArg::Type(ty))
                }
            },
            _ => {
                let ty = self.parse_type()?;
                Ok(GenericArg::Type(ty))
            },
        }
    }

    fn parse_where_bound(&mut self) -> Result<WhereBound, ParserErr> {
        let begin = self.get_cur_span();
        if self.peek()? == Token::OpenSymbol(OpenCloseSymbol::Brace) {
            let bound = self.parse_block_expr(begin, None)?;
            return Ok(WhereBound::Value {
                bound,
            });
        }

        let ty = self.parse_type()?;
        if self.try_consume(Token::Punctuation(Punctuation::Colon)) {
            let bounds = self.parse_punct_separated(Punctuation::Ampersand, Self::parse_generic_type_bound)?;
            let span = self.get_span_to_current(begin);
            Ok(WhereBound::Type {
                span,
                ty,
                bounds,
            })
        } else {
            self.consume_strong_kw(StrongKeyword::In)?;
            let bounds = self.parse_punct_separated(Punctuation::Or, Self::parse_type)?;
            let span = self.get_span_to_current(begin);
            Ok(WhereBound::ExplicitType {
                span,
                ty,
                bounds,
            })
        }
    }

    fn parse_where_clause(&mut self) -> Result<Option<AstNodeRef<WhereClause>>, ParserErr> {
        let begin = self.get_cur_span();
        if !self.try_consume(Token::StrongKw(StrongKeyword::Where)) {
            return Ok(None);
        }

        let bounds = self.parse_punct_separated(Punctuation::Comma, Self::parse_where_bound)?;
        let span = self.get_span_to_current(begin);
        Ok(Some(self.add_node(WhereClause {
            span,
            node_id: NodeId::INVALID,
            bounds,
        })))
    }

    fn parse_trait_bounds(&mut self) -> Result<AstNodeRef<TraitBounds>, ParserErr> {
        let begin = self.get_cur_span();
        let bounds = self.parse_punct_separated(Punctuation::Ampersand, Self::parse_trait_path)?;

        let span = self.get_span_to_current(begin);
        Ok(self.add_node(TraitBounds {
            span,
            node_id: NodeId::INVALID,
            bounds,
        }))
    }

// =============================================================================================================================

    fn parse_visibility(&mut self) -> Result<Option<AstNodeRef<Visibility>>, ParserErr> {
        if !self.try_consume(Token::StrongKw(StrongKeyword::Pub)) {
            return Ok(None);
        }
        
        self.push_meta_frame();
        let begin = self.get_cur_span();
        if self.try_begin_scope(OpenCloseSymbol::Paren) {
            let vis = match self.try_peek().unwrap() {
                Token::WeakKw(WeakKeyword::Package) => {
                    self.consume_single();
                    self.end_scope()?;
                    let span = self.get_span_to_current(begin);
                    Visibility::Package{ span, node_id: NodeId::default() }
                },
                Token::WeakKw(WeakKeyword::Lib) => {
                    self.consume_single();
                    self.end_scope()?;
                    let span = self.get_span_to_current(begin);
                    Visibility::Lib{ span, node_id: NodeId::default() }
                },
                Token::WeakKw(WeakKeyword::Super) => {
                    self.consume_single();
                    self.end_scope()?;
                    let span = self.get_span_to_current(begin);
                    Visibility::Super{ span, node_id: NodeId::default() }
                },
                _ => {
                    let path = self.parse_simple_path(true)?;
                    let span = self.get_span_to_current(begin);
                    Visibility::Path{ span, node_id: NodeId::default(), path }
                }
            };

            Ok(Some(self.add_node(vis)))
        } else {
            Ok(Some(self.add_node(Visibility::Pub{ span: begin, node_id: NodeId::default() })))
        }
    }

// =============================================================================================================================

    fn parse_attributes(&mut self) -> Result<Vec<AstNodeRef<Attribute>>, ParserErr> {
        let mut attrs = Vec::new();

        loop {
            self.push_meta_frame();
            let begin = self.get_cur_span();
            
            let is_mod = if self.try_consume(Token::Punctuation(Punctuation::At)) {
                false
            } else if self.try_consume(Token::Punctuation(Punctuation::AtExclaim)) {
                true
            } else {
                self.pop_meta_frame();
                break;
            };

            let path = self.parse_simple_path(true)?;
            let metas = if self.try_peek() == Some(Token::OpenSymbol(OpenCloseSymbol::Paren)) {
                self.parse_comma_separated_closed(OpenCloseSymbol::Paren, Self::parse_attrib_meta)?
            } else {
                Vec::new()
            };

            let span = self.get_span_to_current(begin);
            let attr = self.add_node(Attribute {
                span,
                node_id: NodeId::default(),
                is_mod,
                path,
                metas,
            });
            attrs.push(attr);
        }
        Ok(attrs)
    }

    fn parse_attrib_meta(&mut self) -> Result<AttribMeta, ParserErr> {
        let begin = self.get_cur_span();
        if matches!(self.peek()?, Token::Name(_)) {
            let path = self.parse_simple_path(false)?;
            if self.peek()? == Token::Punctuation(Punctuation::Equals) {
                self.consume_punct(Punctuation::Equals)?;
                let expr = self.parse_expr(ExprParseMode::General)?;
                let span = self.get_span_to_current(begin);
                Ok(AttribMeta::Assign { span, path, expr })
            } else if self.peek()? == Token::OpenSymbol(OpenCloseSymbol::Paren) {
                let metas = self.parse_comma_separated_closed(OpenCloseSymbol::Paren, Self::parse_attrib_meta)?;
                let span = self.get_span_to_current(begin);
                Ok(AttribMeta::Meta { span, path, metas })
            } else {
                let span = self.get_span_to_current(begin);
                Ok(AttribMeta::Simple { path })
            }
        } else {
            let expr = self.parse_expr(ExprParseMode::General)?;
            let span = self.get_span_to_current(begin);
            Ok(AttribMeta::Expr { expr })
        }
    }

// =============================================================================================================================

    fn parse_contract(&mut self) -> Result<AstNodeRef<Contract>, ParserErr> {
        todo!()
    }

// =============================================================================================================================

    /// Parse comma separated values ending with with a CloseSymbol, returning the nodes between them and the span id of the closing symbol
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

    fn parse_punct_separated_end<T, F>(&mut self, separator: Punctuation, end: Token, mut parse_single: F) -> Result<Vec<T>, ParserErr> where
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