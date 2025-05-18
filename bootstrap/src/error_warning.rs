use core::fmt::Display;

use crate::{common::{Scope, SymbolPath}, lexer::{OpenCloseSymbol, Token}};


// TODO: Split into distinct error subsets
// Error ranges
// E0000-E0999: Reserved
// E1000-E1999: Lexer error
// E2000-E2999: Parser error
// E3000-E3999: AST errors
// E4000-E4999: HIR errors
// E5000-E5999: MIR errors
// E6000-E6999: LIR errors
// E7000-E7999: ASM errors
// E8000-E8999: Reserved
// E9000-E9999: Reserved

//==============================================================================================================================


// Range: E2000 - E2999
#[derive(Debug)]
pub enum LexErrorCode {
    #[allow(unused)]
    InternalError(&'static str),

    // E1000-E1999: Lexer error

    /// Invalid BOM
    InvalidBOM(&'static str),

    // Invalid character in binary literal
    InvalidBinInLit,
    // Invalid character in octal literal
    InvalidOctInLit,
    // Invalid character in hexadecimal literal
    InvalidHexInLit,
    // Invalid leading hexadecimal in hex floating point literal
    InvalidLeadHexFp,
    // Missing hex floating point exponent indicator 'p'
    MissHexFpInd,
    //  Invalid character in decimal literal
    InvalidDecInLit,

    // Block comment is not closed
    UnclosedBlockComment,

    // Not enough characters left for valid character literal.
    NotEnoughCharInLit,
    // Invalid escape code in character literal.
    InvalidEscape,
    // Invalid character in hex character literal
    InvalidHexInChar,
    // Invalid character in unicode character literal
    InvalidUnicodeInLit,
    // Character is not a valid unicode character
    InvalidUnicode,

    // Not enough characters left for a valid string
    NotEnoughString,
    // String cannot be accross multiple lines without a string continuation sequence
    StringNoContinue,
    // Not enough characters left for a valid raw string
    NotEnoughRawString,
    // Missing '"' after 'r' or '#' at the start of a raw string
    InvalidStartRawString,

    // Trying to close ... block without its respective opening symbol
    NoOpeningSym{ sym: OpenCloseSymbol },
    // Mismatch when closing block, found ... expected ...
    MismatchCloseSym{ found: OpenCloseSymbol, expected: OpenCloseSymbol },

    InvalidCharInOp{ ch: char },
    InvalidOpSequence { name: String },
}

impl Display for LexErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code: u16 = 1000 + unsafe { *((self as *const Self).cast::<u16>()) };
        write!(f, "E{code:04}: ")?;
        match self {
            Self::InternalError(err)                     => write!(f, "Internal compiler error: {err}"),
            // Lexer
            Self::InvalidBOM(bom)                        => write!(f, "Found unsupported Byte Order Marker (BOM): {bom}, expected either no BOM or a utf-8 BOM."),
            Self::InvalidBinInLit                        => write!(f, "Found invalid character in binary literal"),
            Self::InvalidOctInLit                        => write!(f, "Found invalid character in octal literal"),
            Self::InvalidHexInLit                        => write!(f, "Found invalid character in hexadecimal integer literal"),
            Self::InvalidLeadHexFp                       => write!(f, "Found invalid leading digit in a hexadecimal floating point literal"),
            Self::MissHexFpInd                           => write!(f, "Missing hexadecimal floating point exponent indicator 'p'"),
            Self::InvalidDecInLit                        => write!(f, "Found invalid character in decimal literal"),
            Self::UnclosedBlockComment                   => write!(f, "Block comment was not closed"),
            Self::NotEnoughCharInLit                     => write!(f, "Not enough characters to form a valid character literal"),
            Self::InvalidEscape                          => write!(f, "Invalid escape code in integer literal"),
            Self::InvalidHexInChar                       => write!(f, "Invalid character in hex character literal"),
            Self::InvalidUnicodeInLit                    => write!(f, "Invalid character in unicode character literal"),
            Self::InvalidUnicode                         => write!(f, "Invalid unicode codepoint"),
            Self::NotEnoughString                        => write!(f, "Not enough characters left for a valid string"),
            Self::StringNoContinue                       => write!(f, "String cannot cross multiple lines without a string continuation sequence"),
            Self::NotEnoughRawString                     => write!(f, "Not enough characters left for a valid raw string"),
            Self::InvalidStartRawString                  => write!(f, "Missing '\"' after 'r' or '#' at start of raw string"),
            Self::NoOpeningSym { sym }                   => write!(f, "Trying to close '{}{}' block without matching opening '{}' symbol", sym.as_open_display_str(), sym.as_close_display_str(), sym.as_open_display_str()),
            Self::MismatchCloseSym { found, expected }   => write!(f, "Mismatch when closing block, found '{}', expected '{}'", found.as_close_display_str(), expected.as_close_display_str()),
            Self::InvalidCharInOp { ch }                 => write!(f, "Unsupported character in operator: '{ch}'"),
            Self::InvalidOpSequence { name }             => write!(f, "Unsupported character sequence in operator: {name}"),
        }
    }
}

//==============================================================================================================================

// Range: E2000 - E2999
#[derive(Debug)]
#[repr(u16)]
pub enum ParseErrorCode {
    #[allow(unused)]
    InternalError(&'static str),
    // Not enough tokens
    NotEnoughTokens = 200,
    // Expected, found
    FoundButExpected{ found: Token, expected: Token },
    // Unexpected token ... for ...
    UnexpectedFor{ found: Token, for_reason: &'static str },

    // Invalid token at start of path
    InvalidPathStart{ found: Token, reason: &'static str },

    InvalidPathDisabmiguation{ reason: &'static str },
    InvalidTraitPathFnEnd{ reason: &'static str },

    // Use: expected package name or nothing before ':'
    ExpectPackageName{ found: Token },
    // Use: expected module name or nothing between ':' and '.'
    ExpectModuleName{ found: Token },
    

    // Invalid use of "extern"
    InvalidExternUse,

    MissingExternFuncNoBlock,

    // Duplicate property getter/setter
    DuplicateProp{ get_set: &'static str },

    // Label unsupported in location
    InvalidLabel,
    // Expr is not allowed
    ExprNotSupported{ expr: &'static str, loc: &'static str },
    // Invalid precedence associativity
    InvalidPrecedenceAssoc{ name: String },
    // Ambigouous operators
    AmbiguousOperators,

    EmptyStmtWithAttrs,

    ReceiverInFreeFunction,

    ParamPackNameDescMismatch{ name_count: u32, desc_count: u32 },
    ParamPackDefMisMatch{ elem_count: u32, def_count: u32 },
    GenericTypeBoundsNotAllowed,
}

impl Display for ParseErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code: u16 = 2000 + unsafe { *((self as *const Self).cast::<u16>()) };
        write!(f, "E{code:04}: ")?;
        match self {
            Self::InternalError(err)                                  => write!(f, "Internal compiler error: {err}"),

            Self::NotEnoughTokens                                     => write!(f, "not enough tokens to parse"),
            Self::FoundButExpected { found, expected }                => write!(f, "Expected `{}`, found `{}`", expected.as_display_str(), found.as_display_str()),
            Self::UnexpectedFor { found, for_reason }                 => write!(f, "Unexpected token {} for {for_reason}", found.as_display_str()),
            Self::InvalidPathStart { found, reason }                  => write!(f, "Invalid token at start of path: '{}'{}{reason}", found.as_display_str(), if reason.is_empty() { "" } else { ", reason: " }),
            Self::InvalidPathDisabmiguation { reason }                => write!(f, "Path disambiguation is invalid: {reason}"),
            Self::InvalidTraitPathFnEnd { reason }                    => write!(f, "Trait path ends on an invalid function-style end: {reason}"),
            Self::ExpectPackageName { found }                         => write!(f, "Unexpected token when parsing use declaration, expected a package name or nothing before ':', found '{}'", found.as_display_str()),
            Self::ExpectModuleName { found }                          => write!(f, "Unexpected token when parsing use declaration, expected a module name or nothing between ':' and '.', found '{}'", found.as_display_str()),
            Self::InvalidExternUse                                    => write!(f, "Invalid usage of 'extern', can only be applied to functions and statics"),
            Self::MissingExternFuncNoBlock                            => write!(f, "An empty block is only allowed on functions that are explicitly defined as extern (when not in a trait)"),
            Self::DuplicateProp { get_set }                           => write!(f, "Duplicate {get_set} in property item"),
            Self::InvalidLabel                                        => write!(f, "A label is not supported in this location"),
            Self::ExprNotSupported { expr, loc }                      => write!(f, "{expr} is not allowed in {loc}"),
            Self::InvalidPrecedenceAssoc { name }                     => write!(f, "Invalid precedence associativity: {name}"),
            Self::AmbiguousOperators                                  => write!(f, "Ambiguous operators, cannot figure out which operators is infix"),
            Self::EmptyStmtWithAttrs                                  => write!(f, "An empty statement cannot have attributes applied to it"),
            Self::ReceiverInFreeFunction                              => write!(f, "Free functions are not allowed to have a receiver"),
            Self::ParamPackNameDescMismatch{ name_count, desc_count } => write!(f, "Mismatch in number of paramter pack names ({name_count}) and descriptions ({desc_count})"),
            Self::ParamPackDefMisMatch { elem_count, def_count }      => write!(f, "Number of parameter pack defaults ({def_count}) need to ve an integer multiple of the element count ({elem_count})"),
            Self::GenericTypeBoundsNotAllowed                         => write!(f, "Generics type bounds are not allowed on an item that doesn't support a where clause"),

            #[allow(unreachable_patterns)]
            _                                                         => write!(f, "Unknown Parse error"),
        }
    }
}

//==============================================================================================================================

// Range: E3000 - E3999
// TODO: Better description of "param name"
#[derive(Debug)]
pub enum AstErrorCode {
    #[allow(unused)]
    InternalError(&'static str),

    InvalidAttribute{ info: String },
    InvalidAttributeData{ info: String },
    InvalidModulePath { paths: Vec<String> },
    NotTopLevel { path: String, info: String },
    InvalidAbiLiteral { lit: String, info: String },
    InvalidLiteral { lit: String, info: String },

    MultipleStructComplete,
    InvalidUninitVarDecl { info: String },
    
    VariadicMultiple,
    VariadicMultipleNames,
    VariadicInvalidPattern { info: String },
    
    ParamMultipleNamesWithDefVal,
    ParamReqAfterOpt,

    ParamPackExpectedTypeDef{ pos: u32 },
    ParamPackExpectedExprDef{ pos: u32 },

    TraitPropNotAllDefOrNone,
}

impl Display for AstErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code: u16 = 3000 + unsafe { *((self as *const Self).cast::<u16>()) };
        write!(f, "E{code:04}: ")?;
        match self {
            Self::InternalError(err)               => write!(f, "Internal compiler error: {err}"),
            Self::InvalidAttribute { info }        => write!(f, "Invalid attribute: {info}"),
            Self::InvalidAttributeData { info }    => write!(f, "Invalid attribute data: {info}"),
            Self::InvalidModulePath { paths }      => {
                write!(f, "Found invalid module, expected to find corresponding file at: ")?;
                if !paths.is_empty() {
                    write!(f, "'{}'", paths[0])?;
                    for path in &paths[1..] {
                        write!(f, ", or '{}'", path)?;
                    }
                }
                Ok(())
            },
            Self::NotTopLevel { path, info }       => write!(f, "Found top-level element in a nested module in path '{path}': {info}"),
            Self::InvalidAbiLiteral { lit, info }  => write!(f, "Invalid ABI literal '{lit}': {info}"),
            Self::InvalidLiteral { lit, info }     => write!(f, "Invalid literal '{lit}': {info}"),
            Self::MultipleStructComplete           => write!(f, "Structure expression may only contain 1 completion expression"),
            Self::InvalidUninitVarDecl { info }    => write!(f, "Invalid unitialized variable declaration: {info}"),
            
            Self::VariadicMultiple                => write!(f, "A function cannot have multiple variadic paramters"),
            Self::VariadicMultipleNames           => write!(f, "Variadic parameters may only have a single 'name'"),
            Self::VariadicInvalidPattern { info } => write!(f, "Invalid pattern in variadic parameter: {info}"),
            
            Self::ParamMultipleNamesWithDefVal    => write!(f, "When assigning a default value, a paramter may only have 1 'name'"),
            Self::ParamReqAfterOpt                => write!(f, "Required paramters need to be defined before all optional paramters"),

            Self::ParamPackExpectedTypeDef{ pos } => write!(f, "Expected a type as a paramter pack default in position {pos}"),
            Self::ParamPackExpectedExprDef{ pos } => write!(f, "Expected an expression as a paramter pack default in position {pos}"),

            Self::TraitPropNotAllDefOrNone        => write!(f, "Trait properties must either have no defaults, or all defined getters and/or set require to have defaults"),

            #[allow(unreachable_patterns)]
            _                                     => write!(f, "Unknown AST error"),
        }
    }
}

//==============================================================================================================================

// Range: E4000 - E4999
#[derive(Debug, Clone)]
pub enum HirErrorCode {
    #[allow(unused)]
    InternalError(&'static str),

    PrecedenceUnsupportedAttrib { info: String },
    PrecedenceInvalidOrder { info: String },

    OperatorDoesNotExist { op: String },
    OperatorNoPrecedence { op: String },
    OperatorNoOrder { op0: String, op1: String },

    CycleInTraitDag { cycle: String },
    CycleInPrecedenceDag { cycle: String },

    ExpectedTraitSymbol { kind: String, path: Scope },
    UnknownSymbol { path: Scope },

    ImplTraitNoMatchingItem {
        item: String,
        trait_name: SymbolPath,
        info: &'static str
    },
    ImplNoDefault {
        item: String,
    },

    NoHirItemForSymbol { kind: &'static str },


    NotSupportedYet { info: &'static str },
}

impl Display for HirErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code: u16 = 4000 + unsafe { *((self as *const Self).cast::<u16>()) };
        write!(f, "E{code:04}: ")?;

        match self {
            Self::InternalError(err)                   => write!(f, "Internal compiler error: {err}"),

            Self::PrecedenceUnsupportedAttrib { info } => write!(f, "Unsupported precedence attribute: {info}"),
            Self::PrecedenceInvalidOrder { info }      => write!(f, "Invalid precedence order: {info}"),

            Self::OperatorDoesNotExist { op }          => write!(f, "Operator does not exist: {op}"),
            Self::OperatorNoPrecedence { op }          => write!(f, "Operator does not have any precedence: {op}, this expression should be wrapped by parentheses to ensure a correct order"),
            Self::OperatorNoOrder { op0, op1 }         => write!(f, "Operators {op0} and {op1} do not have ordered precedences"),

            Self::CycleInTraitDag { cycle }            => write!(f, "Cycle in trait DAG: {cycle}"),
            Self::CycleInPrecedenceDag { cycle }       => write!(f, "Cycle in precedence DAG: {cycle}"),

            Self::ExpectedTraitSymbol { kind, path }   => write!(f, "Expected a trait symbol, found a {kind} symbol: {}", &path.to_string()),
            Self::UnknownSymbol { path }               => write!(f, "Cannot find symbol: {}", &path.to_string()),

            Self::ImplTraitNoMatchingItem { item, trait_name, info } =>
                write!(f, "Implementation trying to implement item ({item}) that does not exist within the trait ({trait_name}) being implemented: {info}"),
            Self::ImplNoDefault { item }               => write!(f, "Missing implementation for '{item}', as no default exists"),

            Self::NoHirItemForSymbol { kind }          => write!(f, "A {kind} symbol in the current library should always have a corresponding hir {kind} in the current library"),

            Self::NotSupportedYet { info }             => write!(f, "{info} is currently not supported yet"),

            #[allow(unreachable_patterns)]
            _                                          => write!(f, "Unknown HIR error"),
        }
    }
}