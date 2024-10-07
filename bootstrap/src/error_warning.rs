use core::fmt::Display;
use std::mem::discriminant;

use crate::lexer::{OpenCloseSymbol, Token};


// Error ranges
// E0000-E0999: Internal errors
// E1000-E1999: Lexer error
// E2000-E2999: Parser error
// E3000-E3999:
// E4000-E4999:
// E5000-E5999:
// E6000-E6999:
// E7000-E7999:
// E8000-E8999:
// E9000-E9999:
#[derive(Debug)]
#[repr(u16)]
pub enum ErrorCode {
    InternalError(&'static str) = 0,

    // E1000-E1999: Lexer error

    /// Invalid BOM
    LexInvalidBOM(&'static str) = 100,

    // Invalid character in binary literal
    LexInvalidBinInLit = 1001,
    // Invalid character in octal literal
    LexInvalidOctInLit = 1002,
    // Invalid character in hexadecimal literal
    LexInvalidHexInLit = 1003,
    // Invalid leading hexadecimal in hex floating point literal
    LexInvalidLeadHexFp = 1004,
    // Missing hex floating point exponent indicator 'p'
    LexMissHexFpInd = 1005,
    //  Invalid character in decimal literal
    LexInvalidDecInLit = 1006,

    // Block comment is not closed
    LexUnclosedBlockComment = 1010,

    // Not enough characters left for valid character literal.
    LexNotEnoughCharInLit = 1020,
    // Invalid escape code in character literal.
    LexInvalidEscape = 1021,
    // Invalid character in hex character literal
    LexInvalidHexInChar = 1022,
    // Invalid character in unicode character literal
    LexInvalidUnicodeInLit = 1023,
    // Character is not a valid unicode character
    LexInvalidUnicode = 1024,

    // Not enough characters left for a valid string
    LexNotEnoughString = 1030,
    // String cannot be accross multiple lines without a string continuation sequence
    LexStringNoContinue = 1031,
    // Not enough characters left for a valid raw string
    LexNotEnoughRawString = 1032,
    // Missing '"' after 'r' or '#' at the start of a raw string
    LexInvalidStartRawString = 1033,

    // Trying to close ... block without its respective opening symbol
    LexNoOpeningSym{ sym: OpenCloseSymbol } = 1040,
    // Mismatch when closing block, found ... expected ...
    LexMismatchCloseSym{ found: OpenCloseSymbol, expected: OpenCloseSymbol } = 1041,

    // Not enough tokens
    ParseNotEnoughTokens = 200,
    // Expected, found
    ParseFoundButExpected{ found: Token, expected: Token } = 2001,
    // Unexpected token ... for ...
    ParseUnexpectedFor{ found: Token, for_reason: &'static str } = 2002,

    // Invalid token at start of path
    ParseInvalidPathStart{ found:Token } = 2010,

    // Use: expected package name or nothing before ':'
    ParseExpectPackageName{ found: Token } = 2011,
    // Use: expected module name or nothing between ':' and '.'
    ParseExpectModuleName{ found: Token } = 2012,
    

    // Invalid use of "extern"
    ParseInvalidExternUse = 2020,

    // Duplicate property getter/setter
    ParseDuplicateProp{ get_set: &'static str } = 2021,

    // Label unsupported in location
    ParseInvalidLabel = 2030,

    ParseExprNotSupported{ expr: &'static str, loc: &'static str } = 2031,

    ParseInvalidPrecedenceAssoc{ name: String } = 2032,
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code: u16 = unsafe { *((self as *const Self).cast()) };
        write!(f, "E{code:04}: ")?;
        match self {
            Self::InternalError(err)                        => write!(f, "Internal compiler error: {err}"),
            // Lexer
            Self::LexInvalidBOM(bom)                        => write!(f, "Found unsupported Byte Order Marker (BOM): {bom}, expected either no BOM or a utf-8 BOM."),
            Self::LexInvalidBinInLit                        => write!(f, "Found invalid character in binary literal"),
            Self::LexInvalidOctInLit                        => write!(f, "Found invalid character in octal literal"),
            Self::LexInvalidHexInLit                        => write!(f, "Found invalid character in hexadecimal integer literal"),
            Self::LexInvalidLeadHexFp                       => write!(f, "Found invalid leading digit in a hexadecimal floating point literal"),
            Self::LexMissHexFpInd                           => write!(f, "Missing hexadecimal floating point exponent indicator 'p'"),
            Self::LexInvalidDecInLit                        => write!(f, "Found invalid character in decimal literal"),
            Self::LexUnclosedBlockComment                   => write!(f, "Block comment was not closed"),
            Self::LexNotEnoughCharInLit                     => write!(f, "Not enough characters to form a valid character literal"),
            Self::LexInvalidEscape                          => write!(f, "Invalid escape code in integer literal"),
            Self::LexInvalidHexInChar                       => write!(f, "Invalid character in hex character literal"),
            Self::LexInvalidUnicodeInLit                    => write!(f, "Invalid character in unicode character literal"),
            Self::LexInvalidUnicode                         => write!(f, "Invalid unicode codepoint"),
            Self::LexNotEnoughString                        => write!(f, "Not enough characters left for a valid string"),
            Self::LexStringNoContinue                       => write!(f, "String cannot cross multiple lines without a string continuation sequence"),
            Self::LexNotEnoughRawString                     => write!(f, "Not enough characters left for a valid raw string"),
            Self::LexInvalidStartRawString                  => write!(f, "Missing '\"' after 'r' or '#' at start of raw string"),
            Self::LexNoOpeningSym { sym }                   => write!(f, "Trying to close '{}{}' block without matching opening '{}' symbol", sym.as_open_str(), sym.as_close_str(), sym.as_open_str()),
            Self::LexMismatchCloseSym { found, expected }   => write!(f, "Mismatch when closing block, found '{}', expected '{}'", found.as_close_str(), expected.as_close_str()),

            // Parser & ASTs
            Self::ParseNotEnoughTokens                      => write!(f, "not enough tokens to parse"),
            Self::ParseFoundButExpected { found, expected } => write!(f, "Expected `{}`, found `{}`", expected.as_display_str(), found.as_display_str()),
            Self::ParseUnexpectedFor { found, for_reason }  => write!(f, "Unexpected token {} for {for_reason}", found.as_display_str()),
            Self::ParseInvalidPathStart { found }           => write!(f, "Invalid token at start of path: {}", found.as_display_str()),
            Self::ParseExpectPackageName { found }          => write!(f, "Unexpected token when parsing use declaration, expected a package name or nothing before ':', found '{}'", found.as_display_str()),
            Self::ParseExpectModuleName { found }           => write!(f, "Unexpected token when parsing use declaration, expected a module name or nothing between ':' and '.', found '{}'", found.as_display_str()),
            Self::ParseInvalidExternUse                     => write!(f, "Invalid usage of 'extern', can only be applied to functions and statics"),
            Self::ParseDuplicateProp { get_set }            => write!(f, "Duplicate {get_set} in property item"),
            Self::ParseInvalidLabel                         => write!(f, "A label is not supported in this location"),
            Self::ParseExprNotSupported { expr, loc }       => write!(f, "{expr} is not allowed in {loc}"),
            Self::ParseInvalidPrecedenceAssoc { name }      => write!(f, "Invalid precedence associativity: {name}"),
        }
    }
}