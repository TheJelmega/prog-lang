
mod tokens;
mod tables;

use std::{fmt::{Debug, Display}, mem};

pub use tokens::*;
pub use tables::*;

use crate::{
    error_warning::ErrorCode,
    literals::{self, Literal, LiteralTable}
};

#[derive(Debug)]
#[allow(unused)]
pub struct LexerErr {
    file:        String,
    err:         ErrorCode,
    byte_offset: u64,
    char_offset: u64,
    line:        u32,
    columnn:     u32,
    byte_len:    u32,
    char_len:    u32,
}

impl LexerErr {
    pub fn set_path(&mut self, path: String) {
        self.file = path;
    }
}

impl Display for LexerErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}({}:{}): {}", &self.file, self.line, self.columnn, self.err)
    }
}

#[derive(Clone, Copy, PartialEq)]
enum DigitLexMode {
    Hex,
    Dec,
    Oct,
}

pub struct Lexer<'a> {
    pub tokens:   TokenStore,
    literals:     &'a mut LiteralTable,
    names:        &'a mut NameTable,
    punctuation:  &'a mut PuncutationTable,

    meta_elems:   Vec<MetaElem>,

    // 'source' can be a stream
    source:       &'a str,
    cursor:       &'a str,

    byte_offset:  u64,
    char_offset:  u64,
    line:         u32,
    columnn:      u32,

    tab_width:    u32,
}



impl<'a> Lexer<'a> {
    pub fn new(source: &'a str, literals: &'a mut LiteralTable, names: &'a mut NameTable, punctuation: &'a mut PuncutationTable) -> Self {
        Self {
            tokens: TokenStore::new(),
            literals,
            names,
            punctuation,
            meta_elems: Vec::new(),
            source,
            cursor: source,
            byte_offset: 0,
            char_offset: 0,
            line: 1,
            columnn: 1,
            tab_width: 4, // TODO: This is not guaranteed to be 4
        }
    }
}

const HORIZONTAL_WHITESPACE: [char; 4] = [ 
    '\u{0009}',
    '\u{0020}',
    '\u{200E}',
    '\u{200F}', 
];

// const VERTICAL_WHITESPACE: [char; 7] = [
//     '\u{000A}',
//     '\u{000B}',
//     '\u{000C}',
//     '\u{000D}',
//     '\u{0084}',
//     '\u{2028}',
//     '\u{2029}',
// ];

impl Lexer<'_> {

    fn consume_bytes(&mut self, num_bytes: u32) {
        self.byte_offset += num_bytes as u64;
        self.cursor = &self.cursor[num_bytes as usize..];
    }

    fn consume(&mut self, num_chars: u32, num_bytes: u32) {
        self.char_offset += num_chars as u64;
        self.byte_offset += num_bytes as u64;
        self.cursor = &self.cursor[num_bytes as usize..];

        self.columnn += num_chars;
    }

    fn consume_str(&mut self, mut s: &str) {
        while let Some(next) = Self::find_next_newline(s) {
            let num_chars = s[..next].chars().count() as u32 + 1;
            self.consume(num_chars, next as u32 + 1);
            self.new_line();
            s = &s[next + 1..];
        }
        self.consume(s.chars().count() as u32, s.len() as u32);
    }

    fn new_line(&mut self) {
        self.line += 1;
        self.columnn = 1;
    }

    fn next_char_len(s: &str) -> usize {
        if let Some(ch) = s.chars().next() {
            ch.len_utf8()
        } else {
            0
        }
    }

    fn find_next_newline(s: &str) -> Option<usize> {
        s.find(['\n', '\u{0085}'])
    }

    fn add_token(&mut self, token: Token, char_len: u32, byte_len: u32) {
        let meta_elems = mem::take(&mut self.meta_elems);

        let metadata = TokenMetadata {
            char_offset: self.char_offset,
            byte_offset: self.byte_offset,
            line: self.line,
            column: self.columnn,
            char_len,
            byte_len,
            meta_elems,
        };

        self.tokens.push(token, metadata);
        
        if char_len > 0 {
            self.consume(char_len, byte_len);
        } else {
            self.consume_bytes(byte_len)
        }
    }

    fn add_strong_keyword(&mut self, keyword: StrongKeyword) {
        let len = keyword.as_str().len() as u32;
        self.add_token(Token::StrongKw(keyword), len, len);
    }

    fn add_weak_keyword(&mut self, keyword: WeakKeyword) {
        let len = keyword.as_str().len() as u32;
        self.add_token(Token::WeakKw(keyword), len, len);
    }

    fn add_name(&mut self, name: &str) {
        let name_id = self.names.add(name);
        self.add_token(Token::Name(name_id), name.chars().count() as u32, name.len() as u32);
    }

    fn add_whitespace(&mut self, whitespace: &str) {
        self.consume(whitespace.chars().count() as u32, whitespace.len() as u32);

        if let Some(MetaElem::Whitespace(buf)) = self.meta_elems.last_mut() {
            buf.push_str(whitespace);
            return;
        }

        self.meta_elems.push(MetaElem::Whitespace(whitespace.to_string()));
    }

    fn add_comment(&mut self, comment: &str, is_block: bool, is_doc: bool, is_top_level: bool) {
        let comment = comment.to_string();
        let elem = match (is_block, is_doc) {
            (false, false) => MetaElem::LineComment(comment),
            (false, true ) => if is_top_level {
                MetaElem::LineTopDocComment(comment)
            } else {
                MetaElem::LineDocComment(comment)
            }
            (true , false) => MetaElem::BlockComment(comment),
            (true , true ) => if is_top_level {
                MetaElem::BlockTopDocComment(comment)
            } else {
                MetaElem::BlockDocComment(comment)
            },
        };
        self.meta_elems.push(elem);
    }

    fn add_punctuation(&mut self, s: &str) {
        let id = self.punctuation.add(s);
        self.add_token(Token::Punctuation(id), s.chars().count() as u32, s.len() as u32);
    }

    fn add_literal(&mut self, lit: Literal, char_len: u32, byte_len: u32) {
        let id = self.literals.add(lit);
        self.add_token(Token::Literal(id), char_len, byte_len);
    }

    fn gen_err(&self, err: ErrorCode, byte_len: u32, char_len: u32) -> LexerErr {
        LexerErr {
            file: String::new(),
            err,
            byte_offset: self.byte_offset,
            char_offset: self.char_offset,
            line: self.line,
            columnn: self.columnn,
            byte_len,
            char_len,
        }
    }
    
    pub fn lex(&mut self) -> Result<(), LexerErr> {
        if self.cursor.is_empty() {
            return Ok(());
        }

        self.lex_bom().map_err(|(err, len)| LexerErr {
            file: String::new(),
            err,
            byte_offset: self.byte_offset,
            char_offset: self.char_offset,
            line: self.line,
            columnn: self.columnn,
            byte_len: len,
            char_len: len,
        })?;
        self.lex_shebang();

        let mut open_close_stack = Vec::new(); 
        while !self.cursor.is_empty() {
            let ch = self.cursor.chars().next().unwrap();
            let non_alphanum_idx = self.cursor.find(|ch: char| !ch.is_alphanumeric() && ch != '_').unwrap_or(self.cursor.len());
            let sub_str = &self.cursor[..non_alphanum_idx];
 
            match (ch, non_alphanum_idx) {
                ('a', 2) => if sub_str == "as" {
                    let after_as = &self.cursor[2..];
                    if after_as.starts_with('!') {
                        self.add_strong_keyword(StrongKeyword::AsExclaim);
                    } else if after_as.starts_with('?') { 
                        self.add_strong_keyword(StrongKeyword::AsQuestion);
                    } else { 
                        self.add_strong_keyword(StrongKeyword::As);
                    }
                } else {
                    self.add_name(sub_str)
                },
                ('a', 5) => if sub_str == "async" {
                    self.add_strong_keyword(StrongKeyword::Async);
                } else if sub_str == "await" {
                    self.add_strong_keyword(StrongKeyword::Await);
                } else {
                    self.add_name(sub_str)
                },
                ('a', 6) => if sub_str == "assert" {
                    self.add_strong_keyword(StrongKeyword::Assert);
                } else {
                    self.add_name(sub_str)
                },
                ('b', 2) => if sub_str == "b8" {
                    self.add_strong_keyword(StrongKeyword::B8);
                } else {
                    self.add_name(sub_str)
                },
                ('b', 3) => if sub_str == "b16" {
                    self.add_strong_keyword(StrongKeyword::B16);
                } else if sub_str == "b32" {
                    self.add_strong_keyword(StrongKeyword::B32);
                } else if sub_str == "b64" {
                    self.add_strong_keyword(StrongKeyword::B64);
                } else {
                    self.add_name(sub_str)
                },
                ('b', 4) => if sub_str == "bool" {
                    self.add_strong_keyword(StrongKeyword::Bool);
                } else {
                    self.add_name(sub_str)
                },
                ('b', 8) => if sub_str == "bitfield" {
                    self.add_strong_keyword(StrongKeyword::Bitfield);
                } else {
                    self.add_name(sub_str)
                },
                ('c', 4) => if sub_str == "char" {
                    self.add_strong_keyword(StrongKeyword::Char);
                } else if sub_str == "cstr" {
                    self.add_strong_keyword(StrongKeyword::CStr);
                } else {
                    self.add_name(sub_str)
                },
                ('c', 5) => if sub_str == "const" {
                    self.add_strong_keyword(StrongKeyword::Const);
                } else if sub_str == "char7" {
                    self.add_strong_keyword(StrongKeyword::Char7);
                } else if sub_str == "char8" {
                    self.add_strong_keyword(StrongKeyword::Char8);
                } else {
                    self.add_name(sub_str)
                },
                ('c', 6) => if sub_str == "char16" {
                    self.add_strong_keyword(StrongKeyword::Char16);
                } else if sub_str == "char32" {
                    self.add_strong_keyword(StrongKeyword::Char32);
                } else {
                    self.add_name(sub_str)
                },
                ('c', 10) => if sub_str == "constraint" {
                    self.add_strong_keyword(StrongKeyword::Constraint);
                } else {
                    self.add_name(sub_str)
                },
                ('d', 3) => if sub_str == "dyn" {
                    self.add_strong_keyword(StrongKeyword::Dyn);
                } else {
                    self.add_name(sub_str)
                },
                ('d', 5) => if sub_str == "defer" {
                    self.add_strong_keyword(StrongKeyword::Defer);
                } else {
                    self.add_name(sub_str)
                },
                ('d', 8) => if sub_str == "distinct" {
                    self.add_weak_keyword(WeakKeyword::Distinct);
                } else {
                    self.add_name(sub_str);
                },
                ('e', 4) => if sub_str == "enum" {
                    self.add_strong_keyword(StrongKeyword::Enum);
                } else {
                    self.add_name(sub_str)
                },
                ('f', 2) => if sub_str == "fn" {
                    self.add_strong_keyword(StrongKeyword::Fn);
                } else {
                    self.add_name(sub_str)
                },
                ('f', 3) => if sub_str == "f16" {
                    self.add_strong_keyword(StrongKeyword::F16);
                } else if sub_str == "f32" {
                    self.add_strong_keyword(StrongKeyword::F32);
                } else if sub_str == "f64" {
                    self.add_strong_keyword(StrongKeyword::F64);
                } else {
                    self.add_name(sub_str)
                },
                ('f', 4) => if sub_str == "f128" {
                    self.add_strong_keyword(StrongKeyword::F128);
                } else if sub_str == "flag" {
                    self.add_weak_keyword(WeakKeyword::Flag);
                } else {
                    self.add_name(sub_str)
                },
                ('f', 5) => if sub_str == "false" {
                    self.add_strong_keyword(StrongKeyword::False);
                },
                ('i', 2) => if sub_str == "i8" {
                    self.add_strong_keyword(StrongKeyword::I8);
                } else if sub_str == "is" {
                    self.add_strong_keyword(StrongKeyword::Is);
                } else if sub_str == "in" {
                    self.add_strong_keyword(StrongKeyword::In);
                } else {
                    self.add_name(sub_str)
                },
                ('i', 3) => if sub_str == "i16" {
                    self.add_strong_keyword(StrongKeyword::I16);
                } else if sub_str == "i32" {
                    self.add_strong_keyword(StrongKeyword::I32);
                } else if sub_str == "i64" {
                    self.add_strong_keyword(StrongKeyword::I64);
                } else {
                    self.add_name(sub_str)
                },
                ('i', 4) => if sub_str == "i128" {
                    self.add_strong_keyword(StrongKeyword::I128);
                } else if sub_str == "impl" {
                    self.add_strong_keyword(StrongKeyword::Impl);
                } else {
                    self.add_name(sub_str)
                },
                ('i', 5) => if sub_str == "isize" {
                    self.add_strong_keyword(StrongKeyword::Isize);
                } else if sub_str == "infix" {
                    self.add_weak_keyword(WeakKeyword::Infix);
                } else if sub_str == "invar" {
                    self.add_weak_keyword(WeakKeyword::Invar);
                } else {
                    self.add_name(sub_str)
                },
                ('m', 3) => if sub_str == "mut" {
                    self.add_strong_keyword(StrongKeyword::Mut);
                } else {
                    self.add_name(sub_str)
                },
                ('o', 6) => if sub_str == "opaque" {
                    self.add_weak_keyword(WeakKeyword::Opaque);
                } else {
                    self.add_name(sub_str)
                },
                ('o', 8) => if sub_str == "override" {
                    self.add_weak_keyword(WeakKeyword::Override);
                } else {
                    self.add_name(sub_str)
                },
                ('p', 3) => if sub_str == "pre" {
                    self.add_weak_keyword(WeakKeyword::Pre);
                } else {
                    self.add_name(sub_str)
                },
                ('p', 4) => if sub_str == "post" {
                    self.add_weak_keyword(WeakKeyword::Post);
                } else {
                    self.add_name(sub_str)
                },
                ('p', 6) => if sub_str == "prefix" {
                    self.add_weak_keyword(WeakKeyword::Prefix);
                } else {
                    self.add_name(sub_str)
                },
                ('p', 7) => if sub_str == "postfix" {
                    self.add_weak_keyword(WeakKeyword::Postfix);
                } else {
                    self.add_name(sub_str)
                },
                ('p', 8) => if sub_str == "property" {
                    self.add_weak_keyword(WeakKeyword::Property);
                } else {
                    self.add_name(sub_str)
                },
                ('r', 3) => if sub_str == "ref" {
                    self.add_strong_keyword(StrongKeyword::Ref);
                } else {
                    self.add_name(sub_str)
                },
                ('r', 6) => if sub_str == "record" {
                    self.add_weak_keyword(WeakKeyword::Record);
                } else {
                    self.add_name(sub_str)
                },
                ('s', 3) => if sub_str == "str" {
                    self.add_strong_keyword(StrongKeyword::Str);
                } else {
                    self.add_name(sub_str)
                },
                ('s', 4) => if sub_str == "str7" {
                    self.add_strong_keyword(StrongKeyword::Str7);
                } else if sub_str == "str8" {
                    self.add_strong_keyword(StrongKeyword::Str8);
                } else if sub_str == "self" {
                    self.add_strong_keyword(StrongKeyword::KwSelf);
                } else {
                    self.add_name(sub_str)
                },
                ('s', 5) => if sub_str == "str16" {
                    self.add_strong_keyword(StrongKeyword::Str16);
                } else if sub_str == "str32" {
                    self.add_strong_keyword(StrongKeyword::Str32);
                } else if sub_str == "super" {
                    self.add_weak_keyword(WeakKeyword::Super);
                } else {
                    self.add_name(sub_str)
                },
                ('s', 6) => if sub_str == "static" {
                    self.add_strong_keyword(StrongKeyword::Static);
                } else if sub_str == "struct" {
                    self.add_strong_keyword(StrongKeyword::Struct);
                } else if sub_str == "sealed" {
                    self.add_weak_keyword(WeakKeyword::Sealed);
                } else {
                    self.add_name(sub_str)
                },
                ('t', 3) => if sub_str == "try" {
                    if self.source[3..].starts_with('!') {
                        self.add_strong_keyword(StrongKeyword::Try);
                    } else {
                        self.add_strong_keyword(StrongKeyword::TryExclaim);
                    }
                } else if sub_str == "tls" {
                    self.add_weak_keyword(WeakKeyword::Tls);
                } else {
                    self.add_name(sub_str)
                },
                ('t', 4) => if sub_str == "true" {
                    self.add_strong_keyword(StrongKeyword::True);
                } else {
                    self.add_name(sub_str)
                },
                ('t', 5) => if sub_str == "throw" {
                    self.add_strong_keyword(StrongKeyword::Throw);
                } else {
                    self.add_name(sub_str)
                },
                ('u', 2) => if sub_str == "u8" {
                    self.add_strong_keyword(StrongKeyword::U8);
                } else {
                    self.add_name(sub_str)
                },
                ('u', 3) => if sub_str == "u16" {
                    self.add_strong_keyword(StrongKeyword::U16);
                } else if sub_str == "u32" {
                    self.add_strong_keyword(StrongKeyword::U32);
                } else if sub_str == "u64" {
                    self.add_strong_keyword(StrongKeyword::U64);
                } else if sub_str == "use" {
                    self.add_strong_keyword(StrongKeyword::Use);
                } else {
                    self.add_name(sub_str)
                },
                ('u', 4) => if sub_str == "u128" {
                    self.add_strong_keyword(StrongKeyword::U128);
                } else {
                    self.add_name(sub_str)
                },
                ('u', 5) => if sub_str == "usize" { 
                    self.add_strong_keyword(StrongKeyword::Usize);
                } else if sub_str == "union" {
                    self.add_strong_keyword(StrongKeyword::Union);
                } else {
                    self.add_name(sub_str)
                },
                ('u', 6) => if sub_str == "unsafe" { 
                    self.add_strong_keyword(StrongKeyword::Unsafe);
                } else {
                    self.add_name(sub_str)
                },
                ('w', 4) => if sub_str == "when" {
                    self.add_strong_keyword(StrongKeyword::When);
                } else {
                    self.add_name(sub_str)
                },
                ('w', 5) => if sub_str == "where" {
                    self.add_strong_keyword(StrongKeyword::Where);
                } else {
                    self.add_name(sub_str)
                },
                ('y', 5) => if sub_str == "yield" {
                    self.add_strong_keyword(StrongKeyword::Yield);
                } else {
                    self.add_name(sub_str)
                },
                ('0', _) => if sub_str.starts_with("0b") {
                    self.lex_binary_lit(sub_str).map_err(|err| self.gen_err(err, sub_str.len() as u32, sub_str.chars().count() as u32))?;
                } else if sub_str.starts_with("0o") {
                    self.lex_octal_lit(sub_str).map_err(|err| self.gen_err(err, sub_str.len() as u32, sub_str.chars().count() as u32))?;
                } else if sub_str.starts_with("0x") {
                    self.lex_hex_lit(sub_str).map_err(|err| self.gen_err(err, sub_str.len() as u32, sub_str.chars().count() as u32))?;
                } else if sub_str.find(|ch: char| (ch < '0' || ch > '9') && ch != '_').is_some() {
                    self.add_name(sub_str);
                } else {
                    self.lex_decimal(sub_str).map_err(|err| self.gen_err(err, sub_str.len() as u32, sub_str.chars().count() as u32))?;
                },
                (ch @ (' ' | '\u{000B}' | '\u{000C}' | '\u{2028}' | '\u{2029}'), _) => {
                    let end = self.cursor.find(|it: char| it != ch).unwrap_or(self.cursor.len());
                    self.add_whitespace(&self.cursor[..end]);
                },
                ('\t', _) => {
                    let end = self.cursor.find(|ch: char| ch != '\t').unwrap_or(self.cursor.len());
                    self.add_whitespace(&self.cursor[..end]);
                    self.columnn += end as u32 * (self.tab_width - 1);
                },
                ('\u{200E}' | '\u{200F}', _) => {
                    let end = self.cursor.find(|ch: char| ch != '\u{200E}' && ch != '\u{200F}').unwrap_or(self.cursor.len());
                    self.add_whitespace(&self.cursor[..end]);
                },
                ('\n' | '\u{0085}', _) => {
                    self.add_whitespace(&self.cursor[..1]);
                    self.new_line();
                },
                ('\r', _) => {
                    if self.cursor.len() >= 2 && self.cursor.as_bytes()[1] == b'\n' {
                        self.add_whitespace("\r\n");
                        self.new_line();
                    } else {
                        self.add_whitespace("\r");
                    }
                },
                ('/', _) => {
                    let is_comment = self.lex_comment().map_err(|(err, byte_len, char_len)| self.gen_err(err, byte_len, char_len))?;
                    if !is_comment {
                        self.lex_punctuation(ch);
                    }
                },
                ('\'', _) => {
                    self.lex_character().map_err(|(err, byte_len, char_len)| self.gen_err(err, byte_len, char_len))?;
                },
                ('"', _) => {
                    self.lex_string().map_err(|(err, byte_len, char_len)| self.gen_err(err, byte_len, char_len))?;
                },
                ('r', 1) => {
                    let bytes = self.cursor.as_bytes();
                    if self.cursor.len() > 1 && (bytes[1] == b'#' || bytes[1] == b'"') {
                        self.lex_raw_string().map_err(|(err, byte_len, char_len)| self.gen_err(err, byte_len, char_len))?;
                    } else {
                        self.add_name(sub_str);
                    }
                },
                ('1'..='9', _) => {
                    if sub_str.find(|ch: char| (ch < '0' || ch > '9') && ch != '_').is_some() {
                        self.add_name(sub_str);
                    } else {
                        self.lex_decimal(sub_str).map_err(|err| self.gen_err(err, sub_str.len() as u32, sub_str.chars().count() as u32))?;
                    }
                },
                ('!', _) => {
                    let end = self.cursor[1..].find(|ch: char| !ch.is_alphabetic()).unwrap_or(self.cursor.len());
                    let sub_str = &self.cursor[..end];
                    if sub_str == "!in" {
                        self.add_strong_keyword(StrongKeyword::ExclaimIn);
                    } else if sub_str == "!is" {
                        self.add_strong_keyword(StrongKeyword::ExclaimIs);
                    } else {
                        self.lex_punctuation(ch);
                    }
                },
                ('(' | '{' | '[', _) => {
                    self.add_token(Token::OpenSymbol(ch), 1, 1);
                    open_close_stack.push(ch);
                },
                (')' | '}' | ']', _) => {
                    let open = match ch {
                        ')' => '(',
                        '}' => '{',
                        ']' => '[',
                        _ => unreachable!(),
                    };
                    
                    if let Some(prev) = open_close_stack.pop() {
                        if prev != open {
                            let expected = match ch {
                                '(' => ')',
                                '{' => '}',
                                '[' => ']',
                                _ => unreachable!(),    
                            };

                            return Err(self.gen_err(ErrorCode::lex_block_unexpected(ch, expected), 1, 1));
                        }
                    } else {
                        return Err(self.gen_err(ErrorCode::lex_block_no_open(open, ch), 1, 1));
                    }
                    
                    self.add_token(Token::CloseSymbol(ch), 1, 1);
                },
                // character
                (_, 0) => {
                    self.lex_punctuation(ch);
                },
                // Name or symbol
                _ => {
                    self.add_name(sub_str);
                },
            }
        }

        if !self.meta_elems.is_empty() {
            self.tokens.tail_meta_elems = mem::take(&mut self.meta_elems);
        }

        Ok(())
    }

    fn lex_bom(&mut self) -> Result<(), (ErrorCode, u32)> {
        let bytes = self.cursor.as_bytes();
        match bytes[0] {
            0xEF => if bytes[0..=2] == [0xEF, 0xBB, 0xBF] {
                self.tokens.has_bom = true;
                self.consume_bytes(3);

                // make sure that we don't ever log any BOMs
                self.source = self.cursor;
            },
            0xFE => if bytes[0..=1] == [0xFE, 0xFF] {
                return Err((ErrorCode::lex_invalid_bom("utf16 (be)"), 2));
            }
            0xFF => 
            if bytes[0..=1] == [0xFF, 0xFE] {
                if bytes[2..=3] == [0x00, 0x00] {
                    return Err((ErrorCode::lex_invalid_bom("utf32 (le)"), 4));
                } else {
                    return Err((ErrorCode::lex_invalid_bom("utf16 (le)"), 2));
                }
            },
            0x00 => if bytes[0..=3] == [0x00, 0x00, 0xFE, 0xFF] {
                return Err((ErrorCode::lex_invalid_bom("utf32 (be)"), 4));
            },
            0x2B => if bytes[0..=2] == [0x2B, 0x2F, 0x76] {
                return Err((ErrorCode::lex_invalid_bom("utf-7"), 3));
            },
            0xF7 => if bytes[0..=2] == [0xF7, 0x64, 0x4C] {
                return Err((ErrorCode::lex_invalid_bom("utf-1"), 3));
            },
            0xDD => if bytes[0..=3] == [0xDD, 0x73, 0x66, 0x73] {
                return Err((ErrorCode::lex_invalid_bom("utf-ecbdic"), 4));
            },
            0x0E => if bytes[0..=2] == [0x0E, 0xFE, 0xFF] {
                return Err((ErrorCode::lex_invalid_bom("scsu"), 3));
            },
            0xFB => if bytes[0..=2] ==[0xFB, 0xEE, 0x28] {
                return Err((ErrorCode::lex_invalid_bom("bocu-1"), 3));
            },
            0x84 => if bytes[0..=3] == [0x84, 0x31, 0x95, 0x33] {
                return Err((ErrorCode::lex_invalid_bom("gb18030"), 4));
            },
            _ => {},
        }

        Ok(())
    }

    fn lex_shebang(&mut self) {
        if !self.cursor.starts_with("#!") {
            return;
        }

        let end = Self::find_next_newline(&self.cursor).unwrap_or(self.cursor.len());
        let shebang_str = if end > 1 && self.cursor.as_bytes()[end - 1] == b'\r' {
            self.cursor[2..end - 1].to_string()
        } else {
            self.cursor[2..end].to_string()
        };

        let nw_len = end + Self::next_char_len(&self.cursor[end..]);

        self.consume_str(&self.cursor[..nw_len]);
        self.tokens.shebang = Some(shebang_str);
    }

    fn lex_binary_lit(&mut self, sub_str: &str) -> Result<(), ErrorCode> {
        let mut bytes = Vec::with_capacity((sub_str.len() - 2 + 7) / 8);

        let mut acc = 0;
        let mut idx = 0;
        for ch in sub_str[2..].bytes().rev() {
            if ch == b'_' {
                continue;
            } else if ch != b'0' && ch != b'1' {
                return Err(ErrorCode::lex_bin_lit_invalid_char());
            }

            let shift = idx & 7;
            idx += 1;
            let val = ch - b'0';

            acc |= val << shift;
            if shift == 7 {
                bytes.push(acc);
                acc = 0;
            }
        }
        if idx & 7 != 0 {
            bytes.push(acc);
        }

        if bytes.len() > 1 && bytes.last().map_or(false, |&val| val == 0) {
            bytes.pop();
        }

        bytes.reverse();

        let len = sub_str.len() as u32;
        self.add_literal(literals::Literal::Binary { bytes }, len, len);

        Ok(())
    }

    fn lex_punctuation(&mut self, ch: char) {
        const SINGLE_SYMBOLS: &[char] = &[
            '@', '#', '$', ':', ';'
        ];

        if SINGLE_SYMBOLS.contains(&ch) {
            self.add_punctuation(&self.cursor[..1]);
        } else {
            let end = self.cursor.find(|ch: char| ch.is_alphanumeric() || ch.is_whitespace() || SINGLE_SYMBOLS.contains(&ch)).unwrap_or(self.cursor.len());
            let punct_str = &self.cursor[..end];
            self.add_punctuation(punct_str);
        }

        
    }

    fn lex_octal_lit(&mut self, sub_str: &str) -> Result<(), ErrorCode> {
        let digits = Self::lex_lit_digits(&sub_str[2..], DigitLexMode::Oct, false).map_err(|_| ErrorCode::lex_oct_lit_invalid_char())?;

        let len = sub_str.len() as u32;
        self.add_literal(literals::Literal::Octal { digits }, len, len);

        Ok(())
    }

    fn lex_hex_lit(&mut self, sub_str: &str) -> Result<(), ErrorCode> {
        if self.cursor.len() >= 4 && self.cursor.as_bytes()[3] == b'.' {
            // Hex floating point

            let bytes = sub_str.as_bytes();
            let initial_digit = if bytes[2] == b'0' {
                false
            } else if bytes[2] == b'1' {
                true
            } else {
                return Err(ErrorCode::lex_hex_fp_lit_invalid_leading_digit());
            };
            
            let Some(exp_p_offset) = self.cursor.find('p') else {
                return Err(ErrorCode::lex_hex_fp_lit_missing_exp_indicator());
            };

            let mantissa = Self::lex_lit_digits(&self.cursor[4..exp_p_offset], DigitLexMode::Hex, true).map_err(|_| ErrorCode::lex_hex_lit_invalid_char())?;

            let sub_str = &self.cursor[exp_p_offset + 1..];
            let (exp_sign, has_sign, offset) = if sub_str.starts_with('+') {
                (true, true, 1)
            } else if sub_str.starts_with('-') {
                (false, true, 1)
            } else if sub_str.starts_with(|ch: char| (ch >= '0' && ch <= '9') || (ch >= 'a' && ch <= 'f') || (ch >= 'A' && ch <= 'F') || ch == '_' ) {
                (true, false, 0)
            } else {
                return Err(ErrorCode::lex_hex_lit_invalid_char());
            };
            let sub_str = &sub_str[offset..];

            let end = sub_str.find(|ch: char| !ch.is_alphanumeric() && ch != '_').unwrap_or(sub_str.len());
            let exp_str = &sub_str[..end];
            let exponent = Self::lex_lit_digits(exp_str, DigitLexMode::Hex, false).map_err(|_| ErrorCode::lex_hex_lit_invalid_char())?;         

            let len = exp_p_offset as u32 + end as u32 + has_sign as u32 + 1;

            self.add_literal(Literal::HexFp {                                                      
                initial_digit,
                mantissa,
                exp_sign,
                exponent,
            }, len, len);


        } else {
            // Hex integer
            let nibbles = Self::lex_lit_digits(&sub_str[2..], DigitLexMode::Hex, false).map_err(|_| ErrorCode::lex_hex_lit_invalid_char())?;
            let len = sub_str.len() as u32;
            self.add_literal(Literal::HexInt { nibbles }, len, len);
        }

        Ok(())
    }

    fn lex_decimal(&mut self, sub_str: &str) -> Result<(), ErrorCode> {
        let int_digits = Self::lex_lit_digits(sub_str, DigitLexMode::Dec, false).map_err(|_| ErrorCode::lex_dec_lit_invalid_char())?;

        let mut end = sub_str.len();
        let (frac_digits, exp_sign, exp_digits) = if self.cursor.len() > sub_str.len() && self.cursor.as_bytes()[sub_str.len()] == b'.' {
            let offset = sub_str.len();
            let bytes = self.cursor.as_bytes();
            let offset = offset + 1;
            let frac_end = self.cursor[offset..].find(|ch: char| (ch < '0' || ch > '9') && ch != '_').unwrap_or(self.cursor.len() - offset);

            if frac_end == 0 {
                (Vec::new(), false, Vec::new())
            } else {
                end = offset + frac_end;
                let frac_str = &self.cursor[offset..end];
                let frac_digits = Self::lex_lit_digits(frac_str, DigitLexMode::Dec, true).map_err(|_| ErrorCode::lex_dec_lit_invalid_char())?;

                let (exp_sign, expr_digits) = if bytes.len() > end + 1 && bytes[end] == b'e' {
                    let (exp_sign, offset) = if bytes[end + 1] == b'-' {
                        (false, end + 2)
                    } else if bytes[end + 1] == b'+' {
                        (true, end + 2)
                    } else {
                        (true, end + 1)
                    };

                    end = offset + self.cursor[offset..].find(|ch: char| (ch < '0' || ch > '9') && ch != '_').unwrap_or(self.cursor.len() - offset);
                    let exp_string = &self.cursor[offset..end];
                    let exp_digits = Self::lex_lit_digits(exp_string, DigitLexMode::Dec, false).map_err(|_| ErrorCode::lex_dec_lit_invalid_char())?;

                    (exp_sign, exp_digits)
                } else {
                    (true, Vec::new())
                };

                (frac_digits, exp_sign, expr_digits)
            }
        } else {
            (Vec::new(), true, Vec::new())
        };

        self.add_literal(Literal::Decimal {
            int_digits,
            frac_digits,
            exp_sign,
            exp_digits,
        }, end as u32, end as u32);

        Ok(())
    }

    fn lex_lit_digits(sub_str: &str, lex_mode: DigitLexMode, keep_preceding_zeroes: bool) -> Result<Vec<u8>, ()> {
        let mut nibbles = Vec::with_capacity((sub_str.len() + 7) / 8);

        let mut acc = 0;
        let mut idx = 0;
        for ch in sub_str.bytes().rev() {
            if ch == b'_' {
                continue;
            }

            let val = Self::lex_digit(ch, lex_mode)?;

            let nibble_idx = idx & 1;
            idx += 1;
            let shift = nibble_idx * 4;

            acc |= val << shift;
            if nibble_idx == 1 {
                nibbles.push(acc);
                acc = 0;
            }
        }
        if idx & 1 == 1 {
            nibbles.push(acc);
        }

        while !keep_preceding_zeroes && nibbles.len() > 1 && nibbles.last().map_or(false, |&val| val == 0) {
            nibbles.pop();
        }

        nibbles.reverse();
        
        while keep_preceding_zeroes && nibbles.len() > 1 && nibbles.last().map_or(false, |&val| val == 0) {
            nibbles.pop();
        }

        Ok(nibbles)
    }

    fn lex_digit(ch: u8, lex_mode: DigitLexMode) -> Result<u8, ()> {
        let val = match ch {
            b'0'..=b'7' => ch - b'0',
            b'8'..=b'9' => if lex_mode != DigitLexMode::Oct { ch - b'0' } else { return Err(()) },
            b'a'..=b'f' => if lex_mode == DigitLexMode::Hex { 10 + ch - b'a' } else { return Err(()) },
            b'A'..=b'F' => if lex_mode == DigitLexMode::Hex { 10 + ch - b'A' } else { return Err(()) },
            _ => return Err(()),
        };
        Ok(val)
    }

    // TODO: nested comment as sub-elems for tools  (probably post-process step)
    fn lex_comment(&mut self) -> Result<bool, (ErrorCode, u32, u32)> {
        let comment_kind_indicator = self.cursor.as_bytes()[1];
        if comment_kind_indicator == b'/' {
            
            let end = Self::find_next_newline(&self.cursor).unwrap_or(self.cursor.len());

            let (is_doc, is_top) = if end > 3 {
                let bytes = self.cursor.as_bytes();
                if bytes[2] == b'/' {
                    (true, false)
                } else if bytes[2] == b'!' {
                    (true, true)
                } else {
                    (false, false)
                }
            } else {
                (false, false)
            };

            let start = 2 + is_doc as usize;

            let comment = &self.cursor[start..end];

            self.add_comment(comment, false, is_doc, is_top);
            self.consume_str(&self.cursor[..end + 1]);
            return Ok(true);
        } else if comment_kind_indicator == b'*' {
            let (is_doc, is_top) = if self.cursor.len() > 3 {
                let bytes = self.cursor.as_bytes();
                if bytes[2] == b'*' {
                    (true, false)
                } else if bytes[2] == b'!' {
                    (true, true)
                } else {
                    (false, false)
                }
            } else {
                (false, false)
            };
            let start = 2 + is_doc as usize;
            
            let mut depth = 1;
            let mut cursor = &self.cursor[start..];
            let mut comment_len = start;
            loop {
                let Some(next) = cursor.find(|ch: char| ch == '*' || ch == '/') else {
                    return Err((ErrorCode::lex_block_comment_not_closed(), self.cursor.len() as u32, self.cursor.chars().count() as u32));
                };
                if next + 1 >= cursor.len() {
                    return Err((ErrorCode::lex_block_comment_not_closed(), self.cursor.len() as u32, self.cursor.chars().count() as u32));
                }

                if cursor.as_bytes()[next] == b'*' {
                    if cursor.as_bytes()[next + 1] == b'/' {
                        depth -= 1;
                    }
                } else if cursor.as_bytes()[next + 1] == b'*' {
                    depth += 1;
                }
                comment_len += next + 2;
                cursor = &cursor[next + 2..];
                
                if depth == 0 {
                    break;
                }
            }

            let comment = &self.cursor[..comment_len];
            self.add_comment(&comment[start..comment_len - 2], true, is_doc, is_top);
            self.consume_str(comment);
            return Ok(true);
        }

        Ok(false)
    }

    fn lex_character(&mut self) -> Result<(), (ErrorCode, u32, u32)> {
        let bytes = self.cursor.as_bytes();
        if bytes.len() <= 3 {
            return Err((ErrorCode::lex_char_lit_not_enough_chars(), self.cursor.len() as u32, self.cursor.chars().count() as u32));
        }

        if bytes[1] == b'\\' {
            let (ch, len) = match bytes[2] {
                b'0' => ('\0', 4),
                b't' => ('\t', 4),
                b'n' => ('\n', 4),
                b'r' => ('\r', 4),
                b'"' => ('"', 4),
                b'\'' => ('\'', 4),
                b'\\' => ('\\', 4),
                b'x' => {
                    if bytes.len() <= 6 {
                        return Err((ErrorCode::lex_char_lit_not_enough_chars(), self.cursor.len() as u32, self.cursor.chars().count() as u32));
                    }

                    let hi = Self::lex_digit(bytes[3], DigitLexMode::Hex).map_err(|_| (ErrorCode::lex_char_lit_invalid_hex_val(), 6, 6))?;
                    let low = Self::lex_digit(bytes[4], DigitLexMode::Hex).map_err(|_| (ErrorCode::lex_char_lit_invalid_hex_val(), 6, 6))?;

                    let val = (hi << 4) | low;
                    (val as char, 6)
                },
                b'u' => {
                    if bytes.len() <= 7 {
                        return Err((ErrorCode::lex_char_lit_not_enough_chars(), self.cursor.len() as u32, self.cursor.chars().count() as u32));
                    }
                    if bytes[3] != b'{' {
                        return Err((ErrorCode::lex_char_lit_invalid_unicode_val(), 4, 4));
                    }
                    let Some(end) = self.cursor[4..].find('\'').map(|val| val + 4) else {
                        return Err((ErrorCode::lex_char_lit_not_enough_chars(), self.cursor.len() as u32, self.cursor.chars().count() as u32));
                    };
                    if bytes[end - 1] != b'}' || end > 11 {
                        return Err((ErrorCode::lex_char_lit_invalid_unicode_val(), end as u32, end as u32));
                    }

                    let mut code: u32 = 0;
                    for ch in self.cursor[4..end - 1].chars() {
                        if ch.len_utf8() > 1 {
                            return Err((ErrorCode::lex_char_lit_invalid_unicode_val(), end as u32, end as u32));
                        }

                        code <<= 4;
                        code |= Self::lex_digit(ch as u8, DigitLexMode::Hex).map_err(|_| (ErrorCode::lex_char_lit_invalid_unicode_val(), end as u32, end as u32))? as u32;
                    }

                    if code > 0x10FFFF {
                        return Err((ErrorCode::lex_invalid_unicode_codepoint(), end as u32, end as u32));
                    }

                    (unsafe { char::from_u32_unchecked(code) }, end as u32 + 1)
                },
                _ => return Err((ErrorCode::lex_char_lit_invalid_escape_code(), 4, 4)),
            };

            self.add_literal(Literal::Char(ch), len, len);
        } else {
            let ch = self.cursor[1..].chars().next().unwrap();
            self.add_literal(Literal::Char(ch), 3, ch.len_utf8() as u32 + 2);
        }

        Ok(())
    }

    fn lex_string(&mut self) -> Result<(), (ErrorCode, u32, u32)> {
        if self.source.len() <= 2 {
            return Err((ErrorCode::lex_string_lit_not_enough_chars(), self.cursor.len() as u32, self.cursor.chars().count() as u32));
        }

        let mut cursor = &self.cursor[1..];
        let mut end = 1;
        let mut string_content = String::new();
        loop {
            let Some(mut next) = cursor.find(|ch: char| ch == '"' || ch == '\n') else {
                return Err((ErrorCode::lex_string_lit_not_enough_chars(), self.cursor.len() as u32, self.cursor.chars().count() as u32));
            };
            end += next + 1;

            if !string_content.is_empty() {
                let Some(start) = cursor.find(|ch: char| !HORIZONTAL_WHITESPACE.contains(&ch)) else {
                    return Err((ErrorCode::lex_string_lit_not_enough_chars(), self.cursor.len() as u32, self.cursor.chars().count() as u32));
                };
                cursor = &cursor[start..];
                next -= start;
            }
            
            let bytes = cursor.as_bytes();
            if bytes[next] == b'\n' {
                if next > 2 && bytes[next - 1] != b'\\' {
                    return Err((ErrorCode::lex_string_lit_invalid_multi_line(), self.cursor.len() as u32, self.cursor.chars().count() as u32));
                }
                string_content += &cursor[..next - 1];

            } else if next > 2 && bytes[next - 1] != b'\\' {
                string_content += &cursor[..next];
                break;
            } else {
                string_content += &cursor[..next];
            }
            cursor = &cursor[next + 1..];
        }

        let full_string = &self.cursor[..end];
        self.add_literal(Literal::String(string_content), full_string.chars().count() as u32, full_string.len() as u32);

        Ok(())
    }

    fn lex_raw_string(&mut self) -> Result<(), (ErrorCode, u32, u32)> {
        // Function only called when it starts with 'r#', so no panic can happen
        let num_hashes = self.cursor[1..].find(|ch: char| ch != '#').unwrap();

        if self.cursor.as_bytes().len() < 2 * num_hashes + 2 {
            return Err((ErrorCode::lex_raw_string_lit_not_enough_chars(), self.cursor.len() as u32, self.cursor.chars().count() as u32));
        }
        if self.cursor.as_bytes()[num_hashes + 1] != b'"' {
            return Err((ErrorCode::lex_raw_string_lit_invalid_start(), self.cursor.len() as u32, self.cursor.chars().count() as u32));
        }

        let start = num_hashes + 2;
        let cursor = &self.cursor[num_hashes + 2..];

        let mut ending = "\"".to_string();
        ending.reserve(num_hashes + 1);
        for _ in 0..num_hashes {
            ending.push('#');
        }
        let Some(end) = cursor.find(&ending) else {
            return Err((ErrorCode::lex_raw_string_lit_not_enough_chars(), self.cursor.len() as u32, self.cursor.chars().count() as u32));
        };

        let end = start + end;
        let full_string = &self.cursor[..end + num_hashes + 1];

        let raw_str = &self.cursor[start..end];
        self.add_literal(Literal::String(raw_str.to_string()), 0, 0);
        
        self.consume_str(full_string);

        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use crate::{lexer::{MetaElem, NameTable, PuncutationTable}, literals::{Literal, LiteralTable}};

    use super::{Lexer, Token};



    #[test]
    fn test_bin_lex() {
        let source = r#"
0b0
0b00000000
0b0000_0000
0b_00000000
0b00000000_
0b000000000
0b1
0b0001
0b0000_0000_1
0b0001_0010_0011_0100_0101_0110_0111_1000
0b0101_01010101
"#;

        let mut literals = LiteralTable::new();
        let mut names = NameTable::new();
        let mut punctuation = PuncutationTable::new();
        let mut lexer = Lexer::new(&source, &mut literals, &mut names, &mut punctuation);
        lexer.lex().unwrap();

        let token_store = &lexer.tokens;

        let expected: &[&[u8]] = &[
             &[0],
             &[0],
             &[0],
             &[0],
             &[0],
             &[0],
             &[1],
             &[1],
             &[1],
             &[0x12, 0x34, 0x56, 0x78],
             &[0b0101, 0b01010101],
        ];

        assert_eq!(token_store.tokens.len(), expected.len());

        for (token, expected) in token_store.tokens.iter().zip(expected.iter()) {
            let Token::Literal(lit_id) = token else { panic!("Expected a literal expression") };
            let Literal::Binary { bytes } = &literals[*lit_id] else { panic!("Expected byte literal") };
            assert_eq!(bytes, expected);
        }
    }

    #[test]
    fn test_oct_lex() {
        let source = r#"
0o0
0o00
0o0_0
0o_00
0o00_
0o000
0o1
0o01
0o0001
0o12345670
"#;

        let mut literals = LiteralTable::new();
        let mut names = NameTable::new();
        let mut punctuation = PuncutationTable::new();
        let mut lexer = Lexer::new(&source, &mut literals, &mut names, &mut punctuation);
        lexer.lex().unwrap();

        let token_store = &lexer.tokens;

        let expected: &[&[u8]] = &[
             &[0],
             &[0],
             &[0],
             &[0],
             &[0],
             &[0],
             &[1],
             &[1],
             &[1],
             &[0x12, 0x34, 0x56, 0x70],
        ];

        assert_eq!(token_store.tokens.len(), expected.len());

        for (token, expected) in token_store.tokens.iter().zip(expected.iter()) {
            let Token::Literal(lit_id) = token else { panic!("Expected a literal expression") };
            let Literal::Octal { digits } = &literals[*lit_id] else { panic!("Expected octal literal") };
            assert_eq!(digits, expected);
        }
    }

    #[test]
    fn test_hex_int_lex() {
        let source = r#"
0x0
0x00
0x0_0
0x_00
0x00_
0x000
0x1
0x01
0x0001
0x123456789ABCDEF0
"#;

        let mut literals = LiteralTable::new();
        let mut names = NameTable::new();
        let mut punctuation = PuncutationTable::new();
        let mut lexer = Lexer::new(&source, &mut literals, &mut names, &mut punctuation);
        lexer.lex().unwrap();

        let token_store = &lexer.tokens;

        let expected: &[&[u8]] = &[
             &[0],
             &[0],
             &[0],
             &[0],
             &[0],
             &[0],
             &[1],
             &[1],
             &[1],
             &[0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0],
        ];

        assert_eq!(token_store.tokens.len(), expected.len());

        for (token, expected) in token_store.tokens.iter().zip(expected.iter()) {
            let Token::Literal(lit_id) = token else { panic!("Expected a literal expression") };
            let Literal::HexInt{ nibbles } = &literals[*lit_id] else { panic!("Expected hexadecimal integer literal") };
            assert_eq!(nibbles, expected);
        }
    }

    #[test]
    fn test_hex_fp_lex() {
        let source = r#"
0x0.0000p000
0x1.ABCDp-EF
0x1.0001p+1
"#;

        let mut literals = LiteralTable::new();
        let mut names = NameTable::new();
        let mut punctuation = PuncutationTable::new();
        let mut lexer = Lexer::new(&source, &mut literals, &mut names, &mut punctuation);
        lexer.lex().unwrap();

        let token_store = &lexer.tokens;

        let expected: &[(bool, &[u8], bool, &[u8])] = &[
             (false, &[0x00], true , &[0]),
             (true , &[0xAB, 0xCD], false, &[0xEF]),
             (true , &[0x00, 0x01], true , &[1])
        ];

        assert_eq!(token_store.tokens.len(), expected.len());

        for (token, expected) in token_store.tokens.iter().zip(expected.iter()) {
            let Token::Literal(lit_id) = token else { panic!("Expected a literal expression") };
            let Literal::HexFp { initial_digit, mantissa, exp_sign, exponent } = &literals[*lit_id] else { panic!("Expected hexadecimal floating point literal") };
            assert_eq!(*initial_digit, expected.0);
            assert_eq!(mantissa, expected.1);
            assert_eq!(*exp_sign, expected.2);
            assert_eq!(exponent, expected.3);
        }
    }

    #[test]
    fn text_dec_lex() {
        let source = r#"
0
42
13.37
1.2e3
4.5e-6
7.8e+9
"#;

        let mut literals = LiteralTable::new();
        let mut names = NameTable::new();
        let mut punctuation = PuncutationTable::new();
        let mut lexer = Lexer::new(&source, &mut literals, &mut names, &mut punctuation);
        lexer.lex().unwrap();

        let token_store = &lexer.tokens;

        let expected: &[(&[u8], &[u8], bool, &[u8])] = &[
             (&[0x00], &[], true , &[]),
             (&[0x42], &[], true , &[]),
             (&[0x13], &[0x37], true , &[]),
             (&[0x1], &[0x2], true , &[0x3]),
             (&[0x4], &[0x5], false , &[0x6]),
             (&[0x7], &[0x8], true , &[0x9]),
        ];

        assert_eq!(token_store.tokens.len(), expected.len());

        for (token, expected) in token_store.tokens.iter().zip(expected.iter()) {
            let Token::Literal(lit_id) = token else { panic!("Expected a literal expression") };
            let Literal::Decimal { int_digits, frac_digits, exp_sign, exp_digits } = &literals[*lit_id] else { panic!("Expected hexadecimal floating point literal") };
            assert_eq!(int_digits, expected.0);
            assert_eq!(frac_digits, expected.1);
            assert_eq!(*exp_sign, expected.2);
            assert_eq!(exp_digits, expected.3);
        }
    }

    #[test]
    fn text_char_lex() {
        let source = r#"
' '
'A'
'本'
'\n'
'\\'
'\x7F'
'\u{0085}'
"#;

        let mut literals = LiteralTable::new();
        let mut names = NameTable::new();
        let mut punctuation = PuncutationTable::new();
        let mut lexer = Lexer::new(&source, &mut literals, &mut names, &mut punctuation);
        lexer.lex().unwrap();

        let token_store = &lexer.tokens;

        let expected: &[char] = &[
            ' ',
            'A',
            '本',
            '\n',
            '\\',
            '\x7F',
            '\u{0085}' ,
        ];

        assert_eq!(token_store.tokens.len(), expected.len());

        for (token, expected) in token_store.tokens.iter().zip(expected.iter()) {
            let Token::Literal(lit_id) = token else { panic!("Expected a literal expression") };
            let Literal::Char(ch) = &literals[*lit_id] else { panic!("Expected hexadecimal floating point literal") };
            assert_eq!(ch, expected);
        }
    }

    #[test]
    fn text_string_lex() {
        let source = r###"
"hello world"
"プログラミング"
"multi \
       line"
r"raw

string"
r##"raw
string
2
"##
"###;

        let mut literals = LiteralTable::new();
        let mut names = NameTable::new();
        let mut punctuation = PuncutationTable::new();
        let mut lexer = Lexer::new(&source, &mut literals, &mut names, &mut punctuation);
        lexer.lex().unwrap();

        let token_store = &lexer.tokens;

        let expected: &[&str] = &[
            "hello world",
            "プログラミング",
            "multi line",
            "raw\n\nstring",
            "raw\nstring\n2\n",
        ];

        assert_eq!(token_store.tokens.len(), expected.len());

        for (token, expected) in token_store.tokens.iter().zip(expected.iter()) {
            let Token::Literal(lit_id) = token else { panic!("Expected a literal expression") };
            let Literal::String(s) = &literals[*lit_id] else { panic!("Expected hexadecimal floating point literal") };
            assert_eq!(s, expected);
        }
    }
    
    #[test]
    fn test_line_comment_lex() {
        let source = r#"
// A line comment

/// Another line comment, but this time a doc comment

//! And another one, but this time a top-level doc comment
"#;

        let mut literals = LiteralTable::new();
        let mut names = NameTable::new();
        let mut punctuation = PuncutationTable::new();
        let mut lexer = Lexer::new(&source, &mut literals, &mut names, &mut punctuation);
        lexer.lex().unwrap();

        let token_store = &lexer.tokens;

        assert_eq!(token_store.tokens.len(), 0);

        assert_eq!(token_store.tail_meta_elems.len(), 6);

        assert_eq!(token_store.tail_meta_elems[1], MetaElem::LineComment(" A line comment".to_string()));
        assert_eq!(token_store.tail_meta_elems[3], MetaElem::LineDocComment(" Another line comment, but this time a doc comment".to_string()));
        assert_eq!(token_store.tail_meta_elems[5], MetaElem::LineTopDocComment(" And another one, but this time a top-level doc comment".to_string()));
    }
    
    #[test]
    fn test_block_comment_lex() {
        let source = r#"
/* A single line block comment */

/* A multi-line
Block
Comment */

/** A single line doc block comment */

/** A muli-line
Doc block
Comment
*/

/*! A single line top doc block comment */

/*! A multi-line
Top doc block
Comment
*/
"#;

        let mut literals = LiteralTable::new();
        let mut names = NameTable::new();
        let mut punctuation = PuncutationTable::new();
        let mut lexer = Lexer::new(&source, &mut literals, &mut names, &mut punctuation);
        lexer.lex().unwrap();

        let token_store = &lexer.tokens;

        assert_eq!(token_store.tokens.len(), 0);

        assert_eq!(token_store.tail_meta_elems.len(), 13);

        assert_eq!(token_store.tail_meta_elems[1], MetaElem::BlockComment(" A single line block comment ".to_string()));
        assert_eq!(token_store.tail_meta_elems[3], MetaElem::BlockComment(" A multi-line\nBlock\nComment ".to_string()));
        assert_eq!(token_store.tail_meta_elems[5], MetaElem::BlockDocComment(" A single line doc block comment ".to_string()));
        assert_eq!(token_store.tail_meta_elems[7], MetaElem::BlockDocComment(" A muli-line\nDoc block\nComment\n".to_string()));
        assert_eq!(token_store.tail_meta_elems[9], MetaElem::BlockTopDocComment(" A single line top doc block comment ".to_string()));
        assert_eq!(token_store.tail_meta_elems[11], MetaElem::BlockTopDocComment(" A multi-line\nTop doc block\nComment\n".to_string()));
    }
}