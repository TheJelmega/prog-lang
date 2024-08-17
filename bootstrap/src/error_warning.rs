use core::fmt::Display;


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
pub enum ErrorCode {
    // E1000-E1999: Lexer error



    /// Invalid BOM
    E1000(&'static str),

    // Invalid character in binary literal
    E1001,
    // Invalid character in octal literal
    E1002,
    // Invalid character in hexadecimal literal
    E1003,
    // Invalid leading hexadecimal in hex floating point literal
    E1004,
    // Missing hex floating point exponent indicator 'p'
    E1005,
    //  Invalid character in decimal literal
    E1006,

    // Block comment is not closed
    E1010,

    // Not enough characters left for valid character literal.
    E1020,
    // Invalid escape code in character literal.
    E1021,
    // Invalid character in hex character literal
    E1022,
    // Invalid character in unicode character literal
    E1023,
    // Character is not a valid unicode character
    E1024,

    // Not enough characters left for a valid string
    E1030,
    // String cannot be accross multiple lines without a string continuation sequence
    E1031,
    // Not enough characters left for a valid raw string
    E1032,
    // Missing '"' after 'r' or '#' at the start of a raw string
    E1033,

    // Trying to close ... block without its respective opening symbol
    E1040{ open: char, close: char },
    // Mismatch when closing block, found ... expected ...
    E1041{ found: char, expected: char },
}

impl ErrorCode {
    pub fn lex_invalid_bom(bom: &'static str) -> Self {
        Self::E1000(bom)
    }

    pub fn lex_bin_lit_invalid_char() -> Self {
        Self::E1001
    }

    pub fn lex_oct_lit_invalid_char() -> Self {
        Self::E1002
    }

    pub fn lex_hex_lit_invalid_char() -> Self {
        Self::E1003
    }

    pub fn lex_hex_fp_lit_invalid_leading_digit() -> Self {
        Self::E1004
    }

    pub fn lex_hex_fp_lit_missing_exp_indicator() -> Self {
        Self::E1005
    }

    pub fn lex_dec_lit_invalid_char() -> Self {
        Self::E1006
    }

    pub fn lex_block_comment_not_closed() -> Self {
        Self::E1010
    }

    pub fn lex_char_lit_not_enough_chars() -> Self {
        Self::E1020
    }

    pub fn lex_char_lit_invalid_escape_code() -> Self {
        Self::E1021
    }

    pub fn lex_char_lit_invalid_hex_val() -> Self {
        Self::E1022
    }

    pub fn lex_char_lit_invalid_unicode_val() -> Self {
        Self::E1023
    }

    pub fn lex_invalid_unicode_codepoint() -> Self {
        Self::E1024
    }

    pub fn lex_string_lit_not_enough_chars() -> Self {
        Self::E1030
    }

    pub fn lex_string_lit_invalid_multi_line() -> Self {
        Self::E1031
    }

    pub fn lex_raw_string_lit_not_enough_chars() -> Self {
        Self::E1032
    }

    pub fn lex_raw_string_lit_invalid_start() -> Self {
        Self::E1033
    }

    pub fn lex_block_no_open(open: char, close: char) -> Self {
        Self::E1040 { open, close }
    }

    pub fn lex_block_unexpected(found: char, expected: char) -> Self {
        Self::E1041 { found, expected }
    }
}

impl Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::E1000(bom) => writeln!(f, "E1000: Found unsupported Byte Order Marker (BOM): {bom}, expected either no BOM or a utf-8 BOM."),
            Self::E1001 => write!(f, "E1001: Found invalid character in binary literal"),
            Self::E1002 => write!(f, "E1002: Found invalid character in octal literal"),
            Self::E1003 => write!(f, "E1003: Found invalid character in hexadecimal integer literal"),
            Self::E1004 => write!(f, "E1004: Found invalid leading digit in a hexadecimal floating point literal"),
            Self::E1005 => write!(f, "E1005: Missing hexadecimal floating point exponent indicator 'p'"),
            Self::E1006 => write!(f, "E1006: Found invalid character in decimal literal"),
            Self::E1010 => write!(f, "E1010: Block comment was not closed"),
            Self::E1020 => write!(f, "E1020: Not enough characters to form a valid character literal"),
            Self::E1021 => write!(f, "E1021: Invalid escape code in integer literal"),
            Self::E1022 => write!(f, "E1022: Invalid character in hex character literal"),
            Self::E1023 => write!(f, "E1023: Invalid character in unicode character literal"),
            Self::E1024 => write!(f, "E1024: Invalid unicode codepoint"),
            Self::E1030 => write!(f, "E1030: Not enough characters left for a valid string"),
            Self::E1031 => write!(f, "E1031: String cannot cross multiple lines without a string continuation sequence"),
            Self::E1032 => write!(f, "E1032: Not enough characters left for a valid raw string"),
            Self::E1033 => write!(f, "E1033: Missing '\"' after 'r' or '#' at start of raw string"),
            Self::E1040 { open, close } => write!(f, "E1040: Trying to close '{open}{close}' block without matching opening '{open}' symbol"),
            Self::E1041 { found, expected } => write!(f, "E1041: Missmatch when closing block, found '{found}', expected '{expected}'"),
        }
    }
}