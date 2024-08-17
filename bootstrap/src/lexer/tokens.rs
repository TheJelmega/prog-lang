use std::{
    fmt::{Display, Write as _},
    io
};
use super::{NameTable, NameTableId, PunctuationId, PuncutationTable};
use crate::literals::{LiteralId, LiteralTable};


/// Strong keywords
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StrongKeyword {
    As,
    AsQuestion,
    AsExclaim,
    Assert,
    B8,
    B16,
    B32,
    B64,
    Bitfield,
    Bool,
    Char,
    Char7,
    Char8,
    Char16,
    Char32,
    Const,
    Constraint,
    CStr,
    Defer,
    Dyn,
    Enum,
    ExclaimIn,
    ExclaimIs,
    F16,
    F32,
    F64,
    F128,
    False,
    Fn,
    I8,
    I16,
    I32,
    I64,
    I128,
    In,
    Impl,
    Is,
    Isize,
    Mut,
    KwSelf,
    Static,
    Str,
    Str7,
    Str8,
    Str16,
    Str32,
    Struct,
    Throw,
    True,
    Try,
    TryExclaim,
    Ref,
    U8,
    U16,
    U32,
    U64,
    U128,
    Union,
    Unsafe,
    Use,
    Usize,
    When,
    Where,

    // Reserved
    Async,
    Await,
    Yield,
}

impl StrongKeyword {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::As         => "as",
            Self::AsQuestion => "as?",
            Self::AsExclaim  => "as!",
            Self::Assert     => "assert",
            Self::B8         => "b8",
            Self::B16        => "b16",
            Self::B32        => "b32",
            Self::B64        => "b64",
            Self::Bitfield   => "bitfield",
            Self::Bool       => "bool",
            Self::Char       => "char",
            Self::Char7      => "char7",
            Self::Char8      => "char8",
            Self::Char16     => "char16",
            Self::Char32     => "char32",
            Self::Const      => "const",
            Self::Constraint => "constraint",
            Self::CStr       => "cstr",
            Self::Defer      => "defer",
            Self::Dyn        => "dyn",
            Self::Enum       => "enum",
            Self::ExclaimIn  => "!in",
            Self::ExclaimIs  => "!is",
            Self::F16        => "f16",
            Self::F32        => "f32",
            Self::F64        => "f64",
            Self::F128       => "f128",
            Self::False      => "false",
            Self::Fn         => "fn",
            Self::I8         => "i8",
            Self::I16        => "i16",
            Self::I32        => "i32",
            Self::I64        => "i64",
            Self::I128       => "i128",
            Self::Impl       => "impl",
            Self::Is         => "is",
            Self::In         => "in",
            Self::Isize      => "isize",
            Self::Mut        => "mut",
            Self::KwSelf     => "self",
            Self::Static     => "static",
            Self::Str        => "str",
            Self::Str7       => "str7",
            Self::Str8       => "str8",
            Self::Str16      => "str16",
            Self::Str32      => "str32",
            Self::Struct     => "struct",
            Self::Throw      => "throw",
            Self::True       => "true",
            Self::Try        => "try",
            Self::TryExclaim => "try!",
            Self::Ref        => "ref",
            Self::U8         => "u8",
            Self::U16        => "u16",
            Self::U32        => "u32",
            Self::U64        => "u64",
            Self::U128       => "u128",
            Self::Union      => "union",
            Self::Unsafe     => "unsafe",
            Self::Use        => "use",
            Self::Usize      => "usize",
            Self::When       => "when",
            Self::Where      => "where",
            Self::Async      => "asycn",
            Self::Await      => "await",
            Self::Yield      => "yield",
        }
    }
}

// Weak keywords
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WeakKeyword {
    Distinct,
    Flag,
    Infix,
    Invar,
    Opaque,
    Override,
    Post,
    Postfix,
    Pre,
    Prefix,
    Property,
    Record,
    Sealed,
    Super,
    Tls
}

impl WeakKeyword {
    pub fn as_str(self) -> &'static str {
        match self {
            WeakKeyword::Distinct => "distinct",
            WeakKeyword::Flag     => "flag",
            WeakKeyword::Infix    => "infix",
            WeakKeyword::Invar    => "invar",
            WeakKeyword::Opaque   => "opaque",
            WeakKeyword::Override => "override",
            WeakKeyword::Post     => "post",
            WeakKeyword::Postfix  => "postfix",
            WeakKeyword::Pre      => "pre",
            WeakKeyword::Prefix   => "prefex",
            WeakKeyword::Property => "property",
            WeakKeyword::Record   => "record",
            WeakKeyword::Sealed   => "sealed",
            WeakKeyword::Super    => "super",
            WeakKeyword::Tls      => "tls",
        }
    }
}

pub enum Token {
    StrongKw(StrongKeyword),
    WeakKw(WeakKeyword),
    Name(NameTableId),
    Punctuation(PunctuationId), 
    OpenSymbol(char),
    CloseSymbol(char),
    Literal(LiteralId),
}

#[derive(PartialEq, Eq, Debug)]
pub enum MetaElem {
    Whitespace(String),
    LineComment(String),
    LineDocComment(String),
    LineTopDocComment(String),
    BlockComment(String),
    BlockDocComment(String),
    BlockTopDocComment(String),
}

pub struct TokenMetadata {
    pub char_offset: u64,
    pub byte_offset: u64,
    pub line:        u32,
    pub column:      u32,
    pub char_len:    u32,
    pub byte_len:    u32,
    pub meta_elems:  Vec<MetaElem>,
}

impl Display for TokenMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line: {:4}, column: {:3}, char_offset: {:5}, byte_offset: {:5}, char_len: {:3}, byte_len: {:3}",
            self.line,
            self.column,
            self.char_offset,
            self.byte_offset,
            self.char_len,
            self.byte_len,
        )
    }
}

pub struct TokenStore {
    pub has_bom:         bool,
    pub shebang:         Option<String>,
    pub tokens:          Vec<Token>,
    pub metadata:        Vec<TokenMetadata>,
    pub tail_meta_elems: Vec<MetaElem>,
}

impl TokenStore {
    pub fn new() -> Self {
        Self {
            has_bom: false,
            shebang: None,
            tokens: Vec::new(),
            metadata: Vec::new(),
            tail_meta_elems: Vec::new(),
        }
    }

    pub fn push(&mut self, token: Token, meta: TokenMetadata) {
        self.tokens.push(token);
        self.metadata.push(meta);
    }   

    pub fn log(&self, literals: &LiteralTable, names: &NameTable, punctuations: &PuncutationTable) {
        println!("Lexer output");
        println!("has BOM: {}", self.has_bom);
        if let Some(shebang) = &self.shebang {
            println!("shebang: Some(\"{shebang}\")");
        } else {
            println!("shebang: None");
        }

        let mut print_buf = String::with_capacity(64);
        let mut depth = 0;
        for (token, meta) in self.tokens.iter().zip(self.metadata.iter()) {
            if let Token::CloseSymbol(_) = token {
                depth -= 1;
            }

            print_buf.clear();
            for _ in 0..depth {
                _ = write!(&mut print_buf, "| ");
            }
            _ = write!(&mut print_buf, "+");

            _ = match token {
                Token::StrongKw(kw) => write!(&mut print_buf, "StrongKw: {}", kw.as_str()),
                Token::WeakKw(kw) => write!(&mut print_buf, "WeakKw: {}", kw.as_str()),
                Token::Name(name_id) => write!(&mut print_buf, "Name({name_id}): {}", &names[*name_id]),
                Token::Punctuation(punct_id) => write!(&mut print_buf, "Symbol({punct_id}): {}", &punctuations[*punct_id]),
                Token::OpenSymbol(ch) => {
                    depth += 1;
                    write!(&mut print_buf, "OpenSymbol: {ch}")
                },
                Token::CloseSymbol(ch) => {
                    write!(&mut print_buf, "CloseSymbol: {ch}")
                },
                Token::Literal(lit_id) => write!(&mut print_buf, "Literal({lit_id}): {}", &literals[*lit_id]),
            };

            println!("{print_buf:64} | {}", meta);
        }
    }

    pub fn log_csv(&self, writer: &mut dyn io::Write, literals: &LiteralTable, names: &NameTable, punctuations: &PuncutationTable) -> io::Result<()> {
        writeln!(writer, "token,value,line,column,char_offset,byte_offset,char_len,byte_len")?;

        for (token, meta) in self.tokens.iter().zip(self.metadata.iter()) {
            match token {
                Token::StrongKw(kw) => write!(writer, "{},,", kw.as_str())?,
                Token::WeakKw(kw) => write!(writer, "{},,", kw.as_str())?,
                Token::Name(name_id) => write!(writer, "name,{},", &names[*name_id])?,
                Token::Punctuation(punct_id) => write!(writer, "punctuation,{},", &punctuations[*punct_id])?,
                Token::OpenSymbol(ch) => write!(writer, "punctuation,{},", ch)?,
                Token::CloseSymbol(ch) => write!(writer, "punctuation,{},", ch)?,
                Token::Literal(lit_id) => write!(writer, "literal,{},", literals[*lit_id])?,
            }

            writeln!(writer, "{},{},{},{},{},{}", meta.line, meta.column, meta.char_offset, meta.byte_offset, meta.char_offset, meta.byte_len)?;
        }

        Ok(())
    }
}

// pub enum Comment {
//     LineComment(Vec<CommentElement>),
//     LineDocComment(Vec<CommentElement>),
//
//     BlockComment(Vec<CommentElement>),
//     BlockDocComment(Vec<CommentElement>),
//
// }

// pub enum CommentElement {
//     Text(String),
//     Nested(Comment),
//     CodeBlock(String),
// }