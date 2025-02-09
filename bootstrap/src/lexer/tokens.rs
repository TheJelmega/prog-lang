use std::{
    fmt::{self, Write as _},
    io
};
use super::{NameId, NameTable, PunctuationId, PuncutationTable, SpanId, SpanRegistry};
use crate::{lexer::FormatSpan, literals::{Literal, LiteralId, LiteralTable}};


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
    Break,
    Char,
    Char7,
    Char8,
    Char16,
    Char32,
    Const,
    Constraint,
    Continue,
    CStr,
    Defer,
    Do,
    Dyn,
    Else,
    Enum,
    ErrDefer,
    ExclaimIn,
    ExclaimIs,
    Extern,
    F16,
    F32,
    F64,
    F128,
    False,
    Fallthrough,
    Fn,
    For,
    I8,
    I16,
    I32,
    I64,
    I128,
    If,
    In,
    Impl,
    Is,
    Isize,
    Let,
    Loop,
    Match,
    Mod,
    Move,
    Mut,
    Pub,
    SelfTy,
    SelfName,
    Static,
    Str,
    Str7,
    Str8,
    Str16,
    Str32,
    Struct,
    Throw,
    Trait,
    True,
    Try,
    TryExclaim,
    Type,
    Ref,
    Return,
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
    While,

    // Reserved
    Async,
    Await,
    Yield,
}

impl StrongKeyword {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::As          => "as",
            Self::AsQuestion  => "as?",
            Self::AsExclaim   => "as!",
            Self::Assert      => "assert",
            Self::B8          => "b8",
            Self::B16         => "b16",
            Self::B32         => "b32",
            Self::B64         => "b64",
            Self::Bitfield    => "bitfield",
            Self::Bool        => "bool",
            Self::Break       => "break",
            Self::Char        => "char",
            Self::Char7       => "char7",
            Self::Char8       => "char8",
            Self::Char16      => "char16",
            Self::Char32      => "char32",
            Self::Const       => "const",
            Self::Constraint  => "constraint",
            Self::Continue    => "continue",
            Self::CStr        => "cstr",
            Self::Defer       => "defer",
            Self::Do          => "do",
            Self::Dyn         => "dyn",
            Self::Else        => "else",
            Self::Enum        => "enum",
            Self::ErrDefer    => "errdefer",
            Self::ExclaimIn   => "!in",
            Self::ExclaimIs   => "!is",
            Self::Extern      => "extern",
            Self::F16         => "f16",
            Self::F32         => "f32",
            Self::F64         => "f64",
            Self::F128        => "f128",
            Self::False       => "false",
            Self::Fallthrough => "fallthrough",
            Self::Fn          => "fn",
            Self::For         => "for",
            Self::I8          => "i8",
            Self::I16         => "i16",
            Self::I32         => "i32",
            Self::I64         => "i64",
            Self::I128        => "i128",
            Self::If          => "if",
            Self::Impl        => "impl",
            Self::Is          => "is",
            Self::In          => "in",
            Self::Isize       => "isize",
            Self::Let         => "let",
            Self::Loop        => "loop",
            Self::Match       => "match",
            Self::Mod         => "mod",
            Self::Move        => "move",
            Self::Mut         => "mut",
            Self::Pub         => "pub",
            Self::Ref         => "ref",
            Self::Return      => "return",
            Self::SelfName    => "self",
            Self::SelfTy      => "Self",
            Self::Static      => "static",
            Self::Str         => "str",
            Self::Str7        => "str7",
            Self::Str8        => "str8",
            Self::Str16       => "str16",
            Self::Str32       => "str32",
            Self::Struct      => "struct",
            Self::Throw       => "throw",
            Self::Trait       => "trait",
            Self::True        => "true",
            Self::Try         => "try",
            Self::TryExclaim  => "try!",
            Self::Type        => "type",
            Self::U8          => "u8",
            Self::U16         => "u16",
            Self::U32         => "u32",
            Self::U64         => "u64",
            Self::U128        => "u128",
            Self::Union       => "union",
            Self::Unsafe      => "unsafe",
            Self::Use         => "use",
            Self::Usize       => "usize",
            Self::When        => "when",
            Self::Where       => "where",
            Self::While       => "while",
            Self::Async       => "asycn",
            Self::Await       => "await",
            Self::Yield       => "yield",
        }
    }
}

// Weak keywords
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WeakKeyword {
    Assign,
    Associativity,
    Distinct,
    Flag,
    Get,
    HigherThan,
    Infix,
    Invar,
    Lib,
    LowerThan,
    Op,
    Opaque,
    Override,
    Package,
    Post,
    Postfix,
    Pre,
    Precedence,
    Prefix,
    Property,
    Record,
    Sealed,
    Set,
    Super,
    Tls
}

impl WeakKeyword {
    pub fn as_str(self) -> &'static str {
        &Self::WEAK_KEYWORD_NAMES[self as usize]
    }

    pub const WEAK_KEYWORD_NAMES: [&'static str; 25] = [
        "assign",
        "associativity",
        "distinct",
        "flag",
        "get",
        "higher_than",
        "infix",
        "invar",
        "lib",
        "lower_than",
        "op",
        "opaque",
        "override",
        "package",
        "post",
        "postfix",
        "pre",
        "precedence",
        "prefex",
        "property",
        "record",
        "sealed",
        "set",
        "super",
        "tls",
    ];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Punctuation {
    Dot,
    DotDot,
    DotDotDot,
    DotDotEquals,
    Semicolon,
    At,
    AtExclaim,
    Colon,
    ColonEquals,
    Comma,
    Exclaim,
    Caret,
    Ampersand,
    Question,
    QuestionDot,
    Or,
    Equals,
    AndAnd,

    SingleArrowR,
    SingleArrowL,
    DoubleArrow,

    // Special cases
    Contains,
    NotContains,

    Custom(PunctuationId),
}

impl Punctuation {
    pub fn as_str<'a>(&'a self, punctuations: &'a PuncutationTable) -> &'a str {
        match self {
            Self::Custom(id) => &punctuations[*id],
            _ => self.as_display_str(),
        }
    }

    pub fn as_display_str(self) -> &'static str {
        match self {
            Self::Dot          => ".",
            Self::DotDot       => "..",
            Self::DotDotDot    => "...",
            Self::DotDotEquals => "..=",
            Self::Semicolon    => ";",
            Self::At           => "@",
            Self::AtExclaim    => "@!",
            Self::Colon        => ":",
            Self::ColonEquals  => ":=",
            Self::Comma        => ",",
            Self::Exclaim      => "!",
            Self::Caret        => "^",
            Self::Ampersand    => "&",
            Self::Question     => "?",
            Self::QuestionDot  => "?.",
            Self::Or           => "|",
            Self::Equals       => "=",
            Self::AndAnd       => "&&",

            Self::SingleArrowR => "->",
            Self::SingleArrowL => "<-",
            Self::DoubleArrow  => "=>",

            Self::Contains     => "in",
            Self::NotContains  => "!in",
         
            Self::Custom(_)    => "custom_punct",
        }
    }

    pub fn from_str(s: &str, punctuations: &mut PuncutationTable) -> Self {
        match s {
            "."   => Punctuation::Dot,
            ".."  => Punctuation::DotDot,
            "..." => Punctuation::DotDotDot,
            "..=" => Punctuation::DotDotEquals,
            ";"   => Punctuation::Semicolon,
            "@"   => Punctuation::At,
            "@!"  => Punctuation::AtExclaim,
            ":"   => Punctuation::Colon,
            ":="  => Punctuation::ColonEquals,
            ","   => Punctuation::Comma,
            "!"   => Punctuation::Exclaim,
            "^"   => Punctuation::Caret,
            "&"   => Punctuation::Ampersand,
            "?"   => Punctuation::Question,
            "?."  => Punctuation::QuestionDot,
            "|"   => Punctuation::Or,
            "="   => Punctuation::Equals,
            "&&"  => Punctuation::AndAnd,

            "->"  => Punctuation::SingleArrowR,
            "<-"  => Punctuation::SingleArrowL,
            "=>"  => Punctuation::DoubleArrow,

            _ => {
                let id = punctuations.add(s);
                Punctuation::Custom(id)
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OpenCloseSymbol {
    Paren,
    Brace,
    Bracket,
    Custom(PunctuationId),
}

impl OpenCloseSymbol {
    pub fn get_closing_char(ch: char) -> char {
        match ch {
            _ => '\0'
        }
    }

    pub fn as_open_str<'a>(&'a self, punctuations: &'a PuncutationTable) -> &'a str {
        match self {
            Self::Custom(id) => &punctuations[*id],
            _                => self.as_open_display_str(),
        }
    }

    pub fn as_open_display_str(self) -> &'static str {
        match self {
            Self::Paren      => "(",
            Self::Brace      => "{",
            Self::Bracket    => "[",
            Self::Custom(_)  => "custom_open",
        }
    }

    pub fn as_close_str<'a>(&'a self, punctuations: &'a PuncutationTable) -> &'a str {
        match self {
            Self::Custom(id) => &punctuations[*id],
            _                => self.as_close_display_str(),
        }
    }

    pub fn as_close_display_str(self) -> &'static str {
        match self {
            Self::Paren      => ")",
            Self::Brace      => "}",
            Self::Bracket    => "]",
            Self::Custom(_)  => "custom_close"
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Token {
    StrongKw(StrongKeyword),
    WeakKw(WeakKeyword),
    Name(NameId),
    Punctuation(Punctuation), 
    OpenSymbol(OpenCloseSymbol),
    CloseSymbol(OpenCloseSymbol),
    Literal(LiteralId),
    Underscore
}

impl Token {
    pub fn fmt_full(&self, f: &mut dyn fmt::Write, names: &NameTable, literals: &LiteralTable, punctuations: &PuncutationTable) -> fmt::Result {
        match self {
            Self::StrongKw(kw) => write!(f, "{}", kw.as_str()),
            Self::WeakKw(kw) => write!(f, "{}", kw.as_str()),
            Self::Name(name_id) => write!(f, "{}", &names[*name_id]),
            Self::Punctuation(punct) => write!(f, "{}", punct.as_str(&punctuations)),
            Self::OpenSymbol(sym) => write!(f, "{}", sym.as_open_display_str()),
            Self::CloseSymbol(sym) => write!(f, "{}", sym.as_close_display_str()),
            Self::Literal(lit_id) => write!(f, "{}", &literals[*lit_id]),
            Self::Underscore => write!(f, "_"),
        }
    }

    pub fn as_display_str(&self) -> &str {
        match self {
            Self::StrongKw(kw) => kw.as_str(),
            Self::WeakKw(kw) => kw.as_str(),
            Self::Name(_) => "name",
            Self::Punctuation(punct) => punct.as_display_str(),
            Self::OpenSymbol(sym) => sym.as_open_display_str(),
            Self::CloseSymbol(sym) => sym.as_close_display_str(),
            Self::Literal(_) => "literal",
            Self::Underscore => "_",
        }
    }
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
    pub line:        u32, // TODO: remove
    pub column:      u32, // TODO: remove
    pub span_id:     SpanId,
    pub meta_elems:  Vec<MetaElem>,
}

pub struct TokenStore {
    pub has_bom:          bool,
    pub shebang:          Option<String>,
    pub tokens:           Vec<Token>,
    pub metadata:         Vec<TokenMetadata>,
    pub tail_meta_elems:  Vec<MetaElem>,
    pub weak_kw_name_map: Vec<NameId>,
}

impl TokenStore {
    pub fn new(names: &mut NameTable) -> Self {
        let mut weak_kw_name_map = Vec::with_capacity(WeakKeyword::WEAK_KEYWORD_NAMES.len());
        for kw_name in &WeakKeyword::WEAK_KEYWORD_NAMES {
            weak_kw_name_map.push(names.add(kw_name));
        }

        Self {
            has_bom: false,
            shebang: None,
            tokens: Vec::new(),
            metadata: Vec::new(),
            tail_meta_elems: Vec::new(),
            weak_kw_name_map,
        }
    }

    pub fn new_dummy() -> Self {
        Self {
            has_bom: false,
            shebang: None,
            tokens: Vec::new(),
            metadata: Vec::new(),
            tail_meta_elems: Vec::new(),
            weak_kw_name_map: Vec::new(),
        }
    }

    pub fn push(&mut self, token: Token, meta: TokenMetadata) {
        self.tokens.push(token);
        self.metadata.push(meta);
    }

    pub fn get_name_from_weak_keyword(&self, kw: WeakKeyword) -> NameId {
        self.weak_kw_name_map[kw as usize]
    }

    pub fn log(&self, literals: &LiteralTable, names: &NameTable, punctuations: &PuncutationTable, spans: &SpanRegistry) {
        println!("Lexer output");
        println!("has BOM: {}", self.has_bom);
        if let Some(shebang) = &self.shebang {
            println!("shebang: Some(\"{shebang}\")");
        } else {
            println!("shebang: None");
        }

        let mut print_buf = String::with_capacity(64);
        let mut depth = 0;
        for (idx, (token, meta)) in self.tokens.iter().zip(self.metadata.iter()).enumerate() {
            if let Token::CloseSymbol(_) = token {
                depth -= 1;
            }

            print_buf.clear();
            _ = write!(&mut print_buf, "{idx:05} ");

            for _ in 0..depth {
                _ = write!(&mut print_buf, "| ");
            }
            _ = write!(&mut print_buf, "+");

            _ = match token {
                Token::StrongKw(kw) => write!(&mut print_buf, "StrongKw: {}", kw.as_str()),
                Token::WeakKw(kw) => write!(&mut print_buf, "WeakKw: {}", kw.as_str()),
                Token::Name(name_id) => write!(&mut print_buf, "Name({name_id}): {}", &names[*name_id]),
                Token::Punctuation(punct) => write!(&mut print_buf, "Symbol: {}", punct.as_str(&punctuations)),
                Token::OpenSymbol(sym) => {
                    depth += 1;
                    write!(&mut print_buf, "OpenSymbol: {}", sym.as_open_display_str())
                },
                Token::CloseSymbol(sym) => {
                    write!(&mut print_buf, "CloseSymbol: {}", sym.as_close_display_str())
                },
                Token::Literal(lit_id) => write!(&mut print_buf, "Literal({lit_id}): {}", &literals[*lit_id]),
                Token::Underscore => write!(&mut print_buf, "Underscore"),
            };

            println!("{print_buf:64} | {}", FormatSpan{ registry: spans, span: meta.span_id });
        }
    }

    pub fn log_csv(&self, writer: &mut dyn io::Write, literals: &LiteralTable, names: &NameTable, punctuations: &PuncutationTable, spans: &SpanRegistry) -> io::Result<()> {
        writeln!(writer, "token,value,line,column,char_offset,byte_offset,char_len,byte_len")?;

        for (token, meta) in self.tokens.iter().zip(self.metadata.iter()) {
            match token {
                Token::StrongKw(kw) => write!(writer, "strong keyword,{},", kw.as_str())?,
                Token::WeakKw(kw) => write!(writer, "weak keyword,{},", kw.as_str())?,
                Token::Name(name_id) => write!(writer, "name,{},", &names[*name_id])?,
                Token::Punctuation(punct) => write!(writer, "punctuation,\"{}\",", punct.as_str(&punctuations))?,
                Token::OpenSymbol(sym) => write!(writer, "punctuation,\"{}\",", sym.as_open_display_str())?,
                Token::CloseSymbol(sym) => write!(writer, "punctuation,\"{}\",", sym.as_close_display_str())?,
                Token::Literal(lit_id) => {
                    let lit = &literals[*lit_id];
                    if let Literal::String(s) = lit {
                        let s = s.replace('\"', "\"\"");
                        write!(writer, "literal,{s},")?;
                    } else {
                        write!(writer, "literal,{lit},", )?;
                    }

                },
                Token::Underscore => write!(writer, "underscore,_,")?,
            }

            let span = &spans[meta.span_id];
            writeln!(writer, "{},{},{},{},{},{}", span.row, span.column, span.char_offset, span.byte_offset, span.char_len, span.byte_len)?;
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