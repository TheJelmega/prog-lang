# Xenon language design

Version: 0.0

## Tables of contents

1. [Introduction](#1-introduction)
    - [This document is provisional](#this-document-is-provisional)
2. [Lexical structure](#2-source-code-representation)
    1. [Input format](#21-input-format)
    2. [Byte order markers](#22-byte-order-markers)
    3. [Newline sequences](#23-newline-sequences)
    4. [Shebang](#24-shebang)
3. [Lexical structure](#3-lexical-structure)
    1. [Whitespace](#31-whitespace)
    2. [Keywords](#32-keywords)
        - [Strong keywords](#strong-keywords)
        - [Reserved keywords](#reserved-keywords)
        - [Weak keywords](#weak-keywords)
    3. [Comments](#33-comments)
4. [Package Stucture](#4-package-structure)
    1. [Packages](#41-packages)
    2. [Artifacts](#42-artifacts)
    3. [Modules](#43-modules)
5. [Literals](#5-literals)
    1. [Numeric literals](#51-numeric-literals)
        - [Decimal literals](#decimal-literal)
        - [Binary literals](#binary-literals)
        - [Octal literals](#octal-literals)
        - [Hexadecimal integer literals](#hexadecimal-integer-literals)
        - [Hexadecimal floating-point literals](#hexadecimal-floating-point-literals)
    2. [Boolean literals](#52-boolean-literals)
    3. [Character literals](#53-character-literals)
        - [Escape codes](#escape-codes)
    4. [String literals](#54-string-literals)
        - [String coninuation sequence](#string-continuation-sequence)
    5. [Literal operators](#55-literal-operators)
6. [Items](#6-items)
7. [Statements](#7-statements)
8. [Expressions](#8-expressions)
9. [Patterns](#9-patterns)
10. [Types System](#10-type-system)
    1. [Types](#101-types)
        1. [Recursive types](#1011-rescursive-types)
        2. [Parenthesized types](#1012-parenthesized-types)
        3. [Primitive types](#1013-primitive-types)
            - [Unsinged types](#unsigned-types)
            - [Signed types](#signed-types)
            - [Floating-point types](#floating-point-types)
            - [Boolean types](#boolean-types)
            - [Character types](#character-types)
        4. [Unit type](#1014-unit-type)
        5. [Never type](#1015-never-type)
        6. [Path types](#1016-path-types)
        7. [Tuple types](#1017-tuple-types)
        8. [Array types](#1018-array-types)
        9. [Slice types](#1019-slice-types)
        10. [String slice types](#10110-string-slice-types)
        11. [Pointer types](#10111-pointer-types)
        12. [Reference types](#10112-reference-types)
            - [Shared reference](#shared-reference)
            - [Mutable reference](#mutable-reference)
        13. [Optional types](#10113-optional-types)
        14. [Function types](#10114-function-types)
        15. [Function poiner types](#10115-function-pointer-type)
        16. [Closure types](#10116-closure-types)
        17. [Interface Object types](#10117-intereface-object-types)
        18. [Impl interface types](#10118-impl-interface-types)
            - [Anonymous type parameter](#anonymous-type-parameter)
            - [Abstract return types](#abstract-return-types)
            - [Abstract return types in interface declarations](#abstract-return-types-in-interface-declarations)
            - [Limitations](#impl-interface-limitations)
        19. [Record types](#10119-record-types)
        20. [Enum record types](#10120-enum-record-types)
        21. [Inferred types](#10121-inferred-types)
    2. [Dynamically sized types](#102-dynamically-sized-types)
    3. [Nominal vs stuctural types](#103-nominal-vs-structural-types)
    4. [Type layout](#104-type-layout)
        1. [Size and alignment](#1041-size-and-alignment)
        2. [Primitive layout](#1042-primitive-layout)
        3. [Unit and never type layout](#1043-unit-and-never-type-layout)
        4. [Pointer and reference layout](#1044-pointer-and-reference-layout)
        5. [Array layout](#1045-array-layout)
        6. [Slice layout](#1046-slice-layout)
        7. [String slice layout](#1047-string-slice-layout)
        8. [Tuple layout](#1048-tuple-layout)
        9. [Interface object layout](#1049-interface-object-layout)
        10. [Closure layout](#10410-closure-layout)
        11. [Bitfield layout](#10411-bitfield-layout)
        12. [Layout representation](#10412-layout-representation)
            - [Xenon representation](#xenon-representation)
                - [Field priority](#field-priority)
            - [C representation](#c-representation)
                - [`repr(C)` structs and records](#reprc-structs-and-records)
                - [`repr(C)` unions](#reprc-unions)
                - [`repr(C)` field-less enums and enum records, and flags enums](#reprc-field-less-enums-and-enum-records-and-flags-enums)
                - [`repr(C)` enums and enum records with fields](#reprc-enums-and-enum-records-with-fields)
            - [Primitive representation](#primitive-representation)
            - [Transparent representation](#transparent-representation)
            - [SOA (structure of array) representation](#sao-structure-of-array-representation)
            - [Additional layout modifiers](#additional-layout-modifiers)
    5. [Interior mutability](#105-interior-mutability)
    6. [Type coercions](#106-type-coercions)
        1. [Coercion sites](#1061-coercion-sites)
        2. [Coercion types](#1062-coecion-types)
        3. [Unsized coercions](#1063-unsized-coercions)
        4. [Least upper bound coercions](#1064-least-upper-bound-coercions)
    7. [Destructors](#107-destructors)
        1. [Drop scopes](#1071-drop-scopes)
        2. [Scopes of function paramters](#1072--scopes-of-function-parameters)
        3. [Scopes of local variables](#1073-scopes-of-local-variables)
        4. [Temporary scopes](#1074-temporary-scopes)
        5. [Operands](#1075-operands)
        6. [Constant promotion](#1076-constant-promotion)
        7. [Temporary lifetime extension](#1077-temporary-lifetime-extension)
        8. [Extending based on patterns](#1078-extending-based-on-patterns)
        9. [Extending based on expressions](#1079-extending-based-on-expressions)
        10. [Not running destructors](#10710-not-running-destructors)
11. [Generics](#11-generics)
12. [Macros](#12-macros)
13. [Operators and Precedence](#13-operators-and-precedence)
14. [Attributes](#14-attributes)
15. [Implicit context](#15-implicit-context)
16. [Effect system](#16-effect-system)
17. [Constracts](#17-contracts)

# 1. Introduction

This file contains the current langauge design for the Xenon language, including rationals for design decisions.
This is not a full specification, as the specification will be derived from this design once the langauge reaches v1.0 of the langague.

This documentation is an overview of the Xenon language in it's current state, and is written for the development of the langauge and those who are interested in the langauge.

## This document is provisional

The contents of this document is still provisional and is subject to change at any time.
This means that the syntax, languages uresl, core and standard libary, compiler infrastructure, package manager/build tool, and other aspect of the designes that have not bee decided on yet.
This therefore will contain gaps for parts that have not been decided on yet.

## Notation

The notation used in the design documents can be found within the [Notation section of the combined grammar](grammar.md#notation)

# 2. Source code representation

This section contains info about the source code representation in the file, and by extension on disk

## 2.1. Input format

Each source file is interpreted as a sequence of Unicode codepoints encoded within the utf-8 format.
If a file does not contain a valid utf-8 sequence, this will result in an error.

Xenon source files use the extension `.xn`

## 2.2. Byte order markers

```
<byte-order-marker> := "\xEF\xBB\xBF"
```

The file may begin using a byte order marker, this marker is kept track of, but generally ignored by the compiler.
This is as the utf-8 byte order marker does not encode the order, as utf-8 work in single byte units and can therefore not be in a different marker.
It is mainly there to indicate the that content of this file encodes a utf-8 sequence, preventing it to be interpreted as another text encoding.

If the file would be reconstructed from its lexical representation, the file will be rebuilt to include the byte order marker if it was present before.

The utf-8 byte order marker is the following: `EF BB BF`.

Any other byte order marker is invalid and will produce an error, as the text file would represent another encoding.
The disallowed byte order markers are the following:

Encoding    | Representation
------------|----------------
utf-16 (be) | FE FF
utf-16 (le) | FF FE
utf-32 (be) | 00 00 FE FF
utf-32 (le) | FF FE 00 00
utf-7       | 2B 2F 76
utf-1       | F7 64 4C
utf-ebcdic  | DD 73 66 73
scsu        | 0E FE FF
bocu-1      | FB EE 28
gb18030     | 84 31 95 33

## 2.3. Newline sequences

```
<new-line> := [ "\r" ] "\n"
```

A newline within the file is represented using a newline sequence `\n` (U+000A).
This may also be preceded by a carriage return '\r' (U+000D), any other occurance is ignored by the compiler.
Any other occurance of a carriage returned in the file will be ignored.

Carriage returns will be preserved in any reconstructed file.

## 2.4. Shebang

```
<shebang> := '#!' ? any valid character ? <newline>
```

A file may contain a shebang in the first line in a file, but will be ignored by the compiler.
When a shebang is encountered on the first line, it will be skipped until the first newline sequence is encountered.

_todo: depending on the attribute syntax, we might have to change this definition slightly to include the used of `#!`_

# 3. Lexical structure

This section contains info about the lexical struture of a code file, which will be interpreted as tokens.

## 3.1. Whitespace

Whitespace is used to separate lexical elements within a file, other than being used to separate elements, whitespace is essentially ignored.
All whitespace is preserved in any reconstructed file.

Below are lists of both all unicode characters recognized as horizontal and vertical whitespace:
- Horizontal whitespace:
  - U+0009 CHARACTER TABULATION (horizontal tab / HT)
  - U+0020 SPACE
  - U+200E LEFT-TO-RIGHT MARK
  - U+200F RIGHT-TO-LEFT MARK
- Vertical whitespace:
  - U+000A: LINE FEED (newline / LF)
  - U+000B: LINE TABULATION (vertical tab / VT)
  - U+000C: FORM FEED (page break / FF)
  - U+000D: CARRIAGE RETURN (CR)
  - U+0085: NEXT LINE (unicode newline)
  - U+2028: LINE SEPARATOR
  - U+2029: PARAGRAPH SEPARATOR

> _Note_: This is **not** a direct mapping to the unicode separator category `Z`

> _Note_: While newline sequences count as whitespace, they are handled separately, see [Newline sequences](#23-newline-sequences).

## 3.2. Keywords

Keywords represent names within the code that have a special meaning in the language, such as declaring a function.

There are 3 types of keywords:
- strong
- reserved
- weak

### Strong keywords

A strong keyword is a keyword that always has a meaning, regardless of where in the code it is located, and can therefore not be used for anything else
A list of strong keywords can be found below (in a close to alphabetic order):
```
b8
b16
b32
b64
bool
char
char7
char8
char16
char32
const
cstr
defer
dyn
enum
f16
f32
f64
f128
false
fn
i8
i16
i32
i64
i128
impl
isize
mut
static
str
str7
str8
str16
str32
true
ref
u8
u16
u32
u64
u128
union
usize
```

### Reserved keywords

A reserved keyword is keyword that is not currently used, but has been set aside as not being possible to be used by the programmer for future use.
A list of reserved keywords can be found below (in a close to alphabetic order):
```
async
await
yield
```

### Weak keywords

A weak keyword is a keyword that is dependent on the surrounding context and can be used anywhere outside
A list of strong keywords can be found below (in a close to alphabetic order):
```
```

## 3.3. Comments

Comments are used to add additional info to code.

There are 3 types of comments, both having 2 forms:
- Line comments: these are comments that begin at a given token and will complete at the end of the current line
- Block comments: these are comments with an explicit begin and end using given tokens.
  Block comments are also allowed to be nested within each other

### Regular comment


```
<regular-comment> := <line-comment> | <block-comment>
<line-comment> := "//" {? any unicode character ?}* <new-line>
<block-comment> := "/*" { ? any unicode character ? | <block-comment> } "*/"
```

Regular comments are add additional info to code only, and can also be used to comment out code, meaning the code is still in the file, but interpreted as a comment.

Comments are stored as metadata associated with tokens and are not tokens by themselves.

### Doc comments

```
<doc-comment> := <doc-line-comment> | <doc-block-comment>
<doc-line-comment> := "///" {? any unicode character ?}* <new-line>
<doc-block-comment> := "/**" { ? any unicode character ? | <block-comment> } "*/"

<top-lvl-doc-comment> := <top-lvl-doc-line-comment> | <top-lvl-doc-line-comment>
```

Doc(umentation) comments are used to provide documentation of the item that is blow them.
The comments are written like normal comment, but the character signalling them is slightly different:
- Line comments start with exactly 3 forward slashes, i.e. `///`
- Block comments start with a forward slash, followed by exactly 2 astrisks, i.e. `/**`

Doc comment act both like metadata to the tokens, but also as special documentation attributes for an item, mainly the `doc` attribute.
- `/// Foo` maps to _TODO when having figured out attribute syntax_
- `/** Bar */` maps to _TODO when having figured out attribute syntax_

During parsing, this metadata will be converted to the relavent attributes.

A carriage return (CR) is not allowed within a doc comment, except when followed immediatelly by a newline.

### Top level doc comments

```
<top-lvl-doc-comment> := <top-lvl-doc-line-comment> | <top-lvl-doc-line-comment>
<top-lvl-doc-line-comment> := "//!" {? any unicode character ?}* <new-line>
<top-lvl-doc-block-comment> := "/*!" { ? any unicode character ? | <block-comment> } "*/"
```

Top level doc(umentation) comments are similar to normal documentation comment, but instead of applying to the item below them, the apply to the module that contains them.

### Examples
_TODO_

# 3.4 Names

```
<letter> := ? any unicode letter ?
<ext-letter> := ? any <letter> except 'e' ?
<number> := ? any unicode number ?
<non-dec-number> := ? any <number> except <dec-digit> ?

<alphanum> := <number> | <letter>

<name> := ( <letter> | <non-dec-number> | '_' ) { <alphanum> |  }*
<ext-name> := { <alphanum> | '_' }* ( <ext-letter> | <non-dec-number> )  { <alphanum> | '_' }*
```

A name is part of an identifier and 

There are 2 types of names:
- Normal names that cannot start with a decimal digit
- Extended names that can start with a decimal digit, but must contain at least 1 non-decimal digit or letter (excluding 'e')

Normal names can be used everywhere a name can be uses, including in locations extended names are avaiable.
Extended names on the other hand have much more limited scope of where they can be used, mainly in locations they cannot cause confusion.

# 4. Package structure

Additional info can be found in [the package design](packages.md).

## 4.1. Packages

A package represents the upper level of the hierarchy of artifacts and the main unit of distribution.

Packages themselves are not the direct result of compilation, but play an integral part if code organization, including how packages are 'imported'.
A package can contain any number of artifacts, allowing allow related code to be shared as a single unit,
meaning that if a project is split up in modularized components, they can still be easilty distributed, without having to result to sub-naming.

## 4.2. Artifacts

Artifacts, unlike packages, are the direct result of a compilation process or stage.

An artifact consts out of 3 distinct types:
- binaries
- static libraries
- dynamic libraries

Artifact themselves are made up from modules.

### Binaries

Binaries are the resulting runnable executables, these are not meant to be 'imported', as they miss all the data required for it.
These can be delivered together with binaries not only be used as the final application, but also tools used for any operation

### Static ibraries

A static library is a library that is meant to be linked into any code using it.
It contains all info needed to 'import' and use it in other code, including the bytecode for all the relavent issues.

If possible, the compiler can inline any code within the static library.

### Dynamic ibraries

A dynamic library is a library that is meant to be referenced by code linking to it, unlike a static binary, this is not linked directly into the code, but lives as it's own file right next to io.
Dynamic libraries actually generates 2 resulting file: a xenon library and a OS-specific dynamic library.
The xenon library is similar to that produces for static libraries, but does not contain all data that the static library has, but only includes what is needed to successfully build and reference the dynamic library in code using it.

## 4.3. Modules

A module generally represents a single file or inlined module definition (if a file is not direclty included within another file).
Each module is allowed to have multiple sub-modules.

Each artifact has it's own main module, which by default uses the following files:
- binaries: main.xn
- static and dynamic libraries: lib.xn

# 5. Literals

```
<literal> := <numeric-literals>
           | <boolean-literals>
           | <character-literal>
           | <string-literals>
```

A literal is a compile time constant representing a given value as either an integer or floating-point type.

> _Note_: Literals are tokens and not symbols, and will therefore be processed in the lexer stage_

## 5.1. Numeric literals

```
<digit_sep> := "_"
<numeric-literals> := <int-decimal-literal>
                    | <float-decimal-literal>
                    | <binary-literal>
                    | <octal-literal>
                    | <int-hexadecimal-literal>
                    | <float-hexadecimal-literal>
```

Numeric literals are literals representing a value of either an integer or floating-point type.

A common feature for integer literals are digit separators.
These don't effect the value represented, but can make the literals more readable to the programmer.

There are generally 4 categories of numerics literals, and these are defined below.

### Decimal literal

```
<dec-digit> := '0'-'9'
<int-dec-literal> := [ '-' ] { <dec-digit> }+
<float-dec-literal> := [ '-' ] { <dec-digit> }+ [ '.' { <dec-digit> }+ ] [ ( 'e' | 'E' ) [ '-' ] { dec-digit }+ ]
```

A decimal literal can represent either an integer or floating point value.
Decimal literals may be prefixed with `0`s without affecting the value, unlike some other languages, this does **not** get interpreted as an octal value and they are ignored.
Decimal literals also support being preceded by a `-`, this is not counted as a separate operator, but is part of the component.

Integer values are a sequence of up to 39 digits and should represent a value that fits in at most a 128-bit limit.

Floating points have a more complex representation.
The start with at least a single digit, and are then optionally followed by a decimal separator (`.`) and its fractional component, but this is not required.
After this, it is also possible to use scientific notation by writing an 'e' or 'E', followed by the exponent, this will modify the value before it by multiplying it by `10 ^^ exponent`.
The exponent is limited to the range -308 to 308.

#### Examples
```
// Integers
10
-195
0042 // value of 42

// Floating point
0.5
-128.64
3e10
005.2 // value of 5.2
```

### Binary literals

```
<bin-digit> := '0' | '1'
<bin-literal> := "0x" <bin-digit> [ { <bin-digit> | <digit-sep> }[,126] <bin-digit> ]
```

A binary literal represents an integer value written as sequence of 0s or 1s, directly representing each bit in the resulting value.
Currently a binary literal is limited to containing 128 digits, representing a 128-bit type.

#### Examples
```
0x1010 // decimal value 10
0x1100_0011 // decimal value 195
0x1________1 // decimal value 1
0x1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111 // u128::MAX
```

### Octal literals

```
<oct-digit> := '0'-'7'
<oct-literal> := "0o" <oct-digit> [ { <oct-digit> | <digit-sep> }[,41] <oct-digit> ]
```

An octal literal represents an integer value written as a sequence of octal values ranging from 0 to 7.
Curently an octal literal is limited to 43 digits, with the upper digit of these being limited in the range 0 to 3, so not to overflow the maximum value of a 128-bit type.

#### Examples
```
0o12 // decimal value 10
0x303 // decimal value 195
0x3__0___3 // decimal value 195
0x377_7777_7777_7777_7777_7777_7777_7777_7777_7777_7777 // u128::MAX

```

### Hexadecimal integer literals

```
<hex-digit> := <dec-digit> | 'a'-'z' | 'A'-'Z'
<int-hex-literal> := "0o" <hex-digit> [ { <hex-digit> | <digit-sep> }[,30] <hex-digit> ]
```

A hexadecimal literal represents an integer value written as a sequence of nibbles, values ranging from 0 to 9, and then from A/a to F/f.
Mixing lower case and upper case letters is allowed, but is discouraged.
Currently a hexadecimal literal is limited to 32 digits, so not to over flow the maximum value of a 128-bit type.

#### Examples
```
0xA // decimal value 10
0xC3 // decimal value 195
0xC_____3 // decimal value 195
0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF // u128::MAX

```

### Hexadecimal floating point literals

```
<float-hex-literal> := [ '-' | '+' ] "0x" ( "1." | "0." ) <hex-digit> { <hex-digit> | <digit-sep> } 'p' [ '-' | '+' ] { <dec-digit> }[,4]
```

In addition to integer hexadecimal literals, there is also support to represent floating points as decimal literals.
These are composed out of a sign, a mantissa and an exponent.

The literal is written with an optional `-`, followed by a the hexadecimal indicator '0x'. 
This can then be followed by either a '0.' followed by at most 13 0s, or a '1.' follows by at most 13 hexadecimal digits.
After which the exponent indicator 'p' appears, followed by an either `-` or '+', and at the exponent in decimal digits.

When the literal starts with `0x0.`, both the mantissa and exponent are limited to 0.
The special values of 'SNAN', 'QNAN', '-INFINITY' or '+INFINITY' cannot be encoded this way, for these values, the associated constant of the type should be used.


#### Examples
```
0x0.0000000000000p0000 // value of 0
+0x0.0000000000000p+0000 // value of 0, but with included signs
-0x0.0000000000000p+0000 // value of -0
0x1.5555555555555p-2 // value of 1/3
```

## 5.2. Boolean literals
```
<bool-literal> := 'true' | 'false'
```

A boolean literal represents either a `true` of a `false` value

## 5.3. Character literals

A character literal defines a character, represented by it's unicode codepoints.

```
<character-literal> := "'" ( ? any unicode codepoint, except for \ and ' ? | <escape-code> ) "'"
```

### Escape codes

```
<escape-code> := '\0'
               | '\t'
               | '\n'
               | '\r'
               | '\"'
               | "\'"
               | '\\'
               | '\x' <hex-digit> <hex-digit>
               | '\u{' { <hex-digit> }[1,6] '}'
```

An escape code, also known as an escaped character is used to represent certain character value that normally cannot be represented in a character or string.

These can be generally split into 3 categories:
- Simple escape codes
- Hex codes
- Unicode codepoints

A simple escape code exists out of a forward slash `/`, followed by single character.
The following escape codes are available

code | Escaped codes
-----|-------------------------
`\0` | U+0000 (NUL)
`\t` | U+0009 (HT)
`\n` | U+000A (LF)
`\r` | U+000D (CR)
`\"` | U+0022 (QUOTATION MARK)
`\'` | U+0027 (APOSTROPHE)
`\\` | U+005C (BACKSLASH)

Hex codes can represent any 8-bit character value using a 2 digit hex code.
It is written as a `\x`, followed by 2 hex digits.

Unicode codepoints represent any valid unicode codepoint, including surrogate pairs, this means all characters in the range 0x000000-0x10FFFF.
A unicode escape code is written as `\u{`, followed by between 1 and 6 hex digits, and closed of with a `}`.

## 5.4. String literals

```
<string-literal> := <regular-string-literal> | <raw-string-literal>
<regular-string-literal> := '"' { ? any valid unicode codepoint, except for \ and '"' ? | ? string continuation sequence ? | <escape-code> }* '"'
<raw-string-literal> := 'r' { '#' }[N] { ? any valid unicode codepoint ? }* '"' { '#' }[N]
```

A string literal defines a static string withing a binray which can be used immutably, and are stored within read-only memory in the binary.

Regular string are usually limited to being on a single line, except for when a string continuation sequence is encountered (see below).
Regular strings are written as a sequence of characters between 2 `"`.

Raw string can appear accross multiple lines within code, the first like starts from the `"`, but any other line that start at he beginning will contain all whitespace since the start of the line.
Raw string also don't allow any escape codes, as they will just be interpreted as raw text.

to define a raw string, the prefix `r` is used, followed by any number of `#`, and then a `"`.
The text in the string will not run until the next encounter of a `"`, followed with as many `#`s as proceeed the string's starting `"`.

### String continuation sequence

```
<string-continuation-sequence> := '\' <newline> ? any whitespace character ? ? any non-whitespace character
```

A string continuation sequence allows a regular line to be split up between lines.

Whenever a `\` is encoutered, directly followed by a new line sequence, the string will pause parsing any character until it finds the next non-whitespace character,
it will then continue to parse the string.

## 5.5. Literal operators

While literals can coerce into a certain set of types, sometimes it can be useful to define a custom literal operator.
A literal operator can apply compile time checks on the value in the operator + changes the type generated by the literal

Below is a list of the builtin literal operators

literal operator | literal kind | resulting type | Info
-----------------|--------------|----------------|-------------
i8               | Number       | i8             | 8-bit signed integer literal
i16              | Number       | i16            | 16-bit signed integer literal
i32              | Number       | i32            | 16-bit signed integer literal
i64              | Number       | i64            | 16-bit signed integer literal
i128             | Number       | i128           | 128-bit signed integer literal
isize            | Number       | isize          | machine-sized signed integer literal
u8               | Number       | u8             | 8-bit unsigned integer literal
u16              | Number       | u16            | 16-bit unsigned integer literal
u32              | Number       | u32            | 16-bit unsigned integer literal
u64              | Number       | u64            | 16-bit unsigned integer literal
u128             | Number       | u128           | 128-bit unsigned integer literal
usize            | Number       | usize          | machine-sized unsigned integer literal
b                | Character    | u8             | Byte character literal
b                | String       | &[u8]          | Byte string literal, requires all codepoint to be <= 0x7F
c                | String       | cstr           | C-string literal (null-terminated), requires all codepoint to be <= 0x7F
ansi             | String       | str8           | ANSI string literal
utf7             | String       | str16          | UTF-7 string literal, requires all codepoint to be <= 0x7F
utf16            | String       | str16          | UTF-16 string literal
utf32            | String       | str32          | UTF-32 string literal

For more info, see the [Operator](#12-operators--precedence) section.


# 6. Items
_TODO_

## 6.N. Interfaces
_TODO_

_TODO: would 'trait' be a better name, as it could be better terminology?_

# 7. Statements
_TODO_

# 8. Expressions
_TODO_

# 9. Patterns
_TODO_

# 10. Type System

## 10.1. Types

```
<type> := <type-no-bound>
        | <interface-object-type>
        | <impl-interface-type>

<type-no-bound> := <parenthesized-type>
                 | <primitive-type>
                 | <unit-type>
                 | <never-type>
                 | <path-type>
                 | <tuple-type>
                 | <array-type>
                 | <slice-type>
                 | <string-slice-type>
                 | <pointer-type>
                 | <reference-type>
                 | <optional-type>
                 | <function-type>
                 | <function-pointer-type>
                 | <closure-type>
                 | <record-type>
                 | <enum-record-type>
                 | <inferred-type>
```

Types are an essential part of any program, each variable, value, and item have a type.
The type defines how a value is interpreted in memory and what operations can be performed using them.

Some types support unique functionality that cannot be replicated using user defined types.

### 10.1.1. Rescursive types

Nominal types may be recursive, meaning that a tpe may havae member that refers, directly or indirectly, to the current type.
These are some limiations on how types can be nested:
- Type aliases must include a nominal type in the recursion, meaning type aliases, or other structural types like arrays and tuples are not allowed.
  i.e. `type Foo = &[Foo]` is not allowed.
- The size of a recursive type must be finite, meanign that the recursive field must be 'broken up' by a type like a pointer or reference type.

### 10.1.2. Parenthesized types

```
<parenthesized-type> := '(' <type> ')'
```

In some locations it may be possible that a type would be ambiguous, this can be solved using a parenthesized type.
For example, a reference to an interface object type with multiple bounds can be unclear, as we cannot cleanly determine if the one of the bounds is a reference, or the whole set of bounds constitute a single type without requiring to rely heavily on context.

### 10.1.3. Primitive types

```
<primitive-type> := <unsigned-type>
                  | <signed-type>
                  | <floating-point-type>
                  | <boolean-type>
                  | <character-type>
```

A primitive type is a type that exists directly within the langauge and is handled specially by the compiler.
These are commonly types that fit in machine register and have specialized instruction for these types.

#### Unsigned types

```
<unsigned-type> := 'u8' | 'u16' | 'u32' | 'u64' | 'u128'
```

An unsigned type represents a natural number (including 0).

Unsigned numbers can generally represent a number between 0 and `(2^n)-1`

Below is a table of supported unsigned integer types:

Type   | Bit width | Min value | Max value 
-------|-----------|-----------|-----------------------------------------------------
`u8`   | 8-bit     | 0         | 255
`u16`  | 16-bit    | 0         | 65_535
`u32`  | 32-bit    | 0         | 4_294_967_295
`u64`  | 64-bit    | 0         | 18_446_744_073_709_511_615
`u128` | 128-bit   | 0         | 340_282_366_920_938_463_463_374_607_431_768_211_455

Both the size and alignment of the unsigned integers are defined by their bit-width.

All but `u128` are generally representable in a CPU register and have native instructions, if there are no native instruction, the program falls back to 'emulating' these types.

In addition to the above types, there is also another unsigned type: `usize`.
`usize` is an unsigned type with the size of a machine-register.

#### Signed types

```
<signed-type> := 'i8' | 'i16' | 'i32' | 'i64' | 'i128'
```

An unsigned type represents a integral number.

Unsigned numbers can generally represent a number between `-2^(n-1)` and `(2^(n-1))-1`

Below is a table of supported unsigned integer types:

Type   | Bit width | Min value                                            | Max value 
-------|-----------|------------------------------------------------------|-----------------------------------------------------
`i8`   | 8-bit     | -128                                                 | 127
`i16`  | 16-bit    | -32_768                                              | 32_767
`i32`  | 32-bit    | -2_147_483_648                                       | 2_147_483_647
`i64`  | 64-bit    | -9_223_372_036_854_775_808                           | 9_223_372_036_854_775_807
`i128` | 128-bit   | -170_141_183_460_469_231_731_687_303_715_884_105_728 | 170_141_183_460_469_231_731_687_303_715_884_105_727

Both the size and alignment of the signed integers are defined by their bit-width.

All but `i128` are generally representable in a CPU register and have native instructions, if there are no native instruction, the program falls back to 'emulating' these types.

In addition to the above types, there is also another signed type: `isize`.
`isize` is an unsigned type with the size of a machine-register.

#### Floating-point types

```
<signed-type> := 'f16' | 'f32' | 'f64' | 'f128'
```

A floating point type represent the same sized type as defined in the IEEE-754-2008 specification.

Below is a table of supported floating-point types:

Type   | Bit width | Mantissa bits      | Exponent bits | Min value  | Max value   | Smallest value | Significant decimal digits
-------|-----------|--------------------|---------------|------------|-------------|----------------|----------------------------
`f16`  | 16-bits   | 10 (11 implicit)   | 5             | 6.55e+5    | -6.55e+5    | 6.10e-5        | 3
`f32`  | 32-bits   | 23 (24 implicit)   | 8             | 3.40e+38   | -3.40e+38   | 1.17e-38       | 6
`f64`  | 64-bits   | 52 (53 implicit)   | 11            | 1.80e+308  | -1.80e+308  | 2.23e-308      | 15
`f128` | 128-bits  | 112 (113 implicit) | 15            | 1.19e+4932 | -1.19e+4932 | 3.36e-4932     | 34

Both the size and alignment of the floating points are defined by their bit-width.

_TODO: could include other floating-point types if wanted_

#### Boolean types

```
<boolean-type> := 'bool' | 'b8' | 'b16' | `b32' | 'b64'
```

A boolean type is a primitive type that can be used to define 1 out of 2 possible states: `true` or `false`.
As a boolean only can represent these 2 values, there are also only 2 valid bit representations for a boolean.
These are `0x0` and `0x1`, meaning that the lower bit is set to the value, and all other bits are set to 0.
Any other bitpattern is undefined behavior.

Below is a table of supported boolean types:

Type   | Bit-width | Bit-width in bitfield
-------|-----------|----------------------
`bool` | 8-bits    | 1-bit
`b8`   | 8-bits    | 8-bits
`b16`  | 16-bits   | 16-bits
`b32`  | 32-bits   | 32-bits
`b64`  | 64-bits   | 64-bits

Both the size and alignment of the booleans are defined by their bit-width.
When used in a bitfield, specific bit-with mentioned above is used.

#### Character types

```
<character-type> := 'char' | 'char7' | 'char8' | 'char16' | 'char32'
```

A character type is primitive type that can represent unicode characters.

Below is a table of supported character types

Type     | Meaning          | Bit-width | Bit-width in bitfield | Valid range
---------|------------------|-----------|-----------------------|------------------------------------------
`char`   | utf-32 codepoint | 32-bits   | 32-bits               | 0x000000 - 0x00D7FF & 0x00E00 - 0x10FFFF
`char7`  | 7-bit ANSI       | 8-bits    | 7-bit                 | 0x00     - 0x7F
`char8`  | 8-bit ANSI       | 8-bits    | 8-bits                | 0x00     - 0xFF
`char16` | utf-16 codepoint | 16-bits   | 16-bits               | 0x0000   - 0xFFFF
`char32` | uft-32 codepoint | 32-bits   | 32-bits               | 0x000000 - 0x10FFFF & 0x00E00 - 0x10FFFF

Both the size and alignment of the booleans are defined by their bit-width.
When used in a bitfield, specific bit-with mentioned above is used.

If a character has a value outside of its valid range, it is undefined behavior.

### 10.1.4. Unit type

```
<unit-type> := '(' ')'
```

The unit type is a special type representing a zero-sided type.
This is also known as `void` in some other languages.

### 10.1.5. Never type

The never type is a special type that represents an operation that can never complete.
This type can be implicitly coerced into any type.
It can only ever appear as the return value of a function and can therefore not be part of any type, meaning you can only ever return a never type.

```
<never-type> := '!'
```

### 10.1.6. Path types

```
<path-type> := <type-path>
```

A path type refers to a user-defined path by its path, there are 3 types it can represent.

### 10.1.7. Tuple types

```
<tuple-type> := '(' <type> { ',' <type> }+ [ ',' ] ')'
```

A tuple type is a _structural_ type consisting out of a list of other types.

The resulting tuple has a number of elements, specified using by the number of types contained within the tuple.
Meaning that the first field will be `0`, the second will be `1`, etc.
The type of each field is the type specified as the `N`-th type of the tuple type.

A tuple with N fields is as called an N-ary tuple, for example a tuple with 2 fields is a 2-ary tuple.

Tuples are required to have at least 2 types, otherwhise they will be resolved to the following types:
- 0 types will be interpreted as a unit type
- 1 type will be interpreted as a parenthesized type

### 10.1.8. Array types

```
<array-type> := '[' <type> ';' <expr> [ ';' <expr> ] ']'
<array-type> := '[' <expr> [ ';' <expr> ] ']' <type>
```

An array type (`[N]T`) is a fixed-size sequence of `N` elements of type `T`
Array types are laid out as a contiguous chunk of memory.

An array's size expression, which occurs after the `;`, needs to be a value that can be evaluated at compile time.

#### Sentinel-terminated arrays

An array can also have a sentinel value, which is declared after the size.
So an array `[N;M]T` has `N` elements of type `T`, with a sentinel value of `M`.
Like the size, the sentinel value needs to be evaluated at compile time.

When a sentinel value is defined, the array will contain 1 additional element past its lenght, this is the sentinel value.

Sentinel value mainly exist for interoperability with C and OS libraries that commonly expect a range of values ending in a sentinal value,
but these are not that useful when writing Xenon code itself

### 10.1.9. Slice types

```
<slice-type> := `[` ';' <expr> `]` <type>
```

A slice type (`[T]`) is a dynamically sized type that represents a 'view' within a sequence of elements of type `T`.

Slices are generally used through reference or pointer-like types:
- `&[T]`: a shared slice, often just called a slice. It borrows the data it point to.
- `&mut [T]`: a mutable slice. It mutably borrows the data it point to.

#### Sentinel-terminated slices

Like an array, a slice may also include sentinels, the slice will then contain 1 additional elements past its dynamically stored length, this is the sentinel value.
This value is meant to prevent accidental out of bounds writes.

A sentinel value can also be defined as an array of values of type `T`, if this is done, the array will contain multi-element sentinel.
The multi sentinels' size is dependent on the number of values in that array, so the resulting array will be as many elements larger.

Sentinel value mainly exist for interoperability with C and OS libraries that commonly expect a range of values ending in a sentinal value,
but these are not that useful when writing Xenon code itself

See the [index expression] for more info about how to create a sentinal terminated array.

### 10.1.10. String slice types

```
<string-slice-type> := 'str' | 'str7' | 'str8' | 'str16' | 'str32' | 'cstr'
```

A string slice typre repesents a special slice, encoding a string.
This is a separate type, which allows string specific functionality.

Below is a table of all string slice types

Type    | character type | internal representation | Meaning
--------|----------------|-------------------------|---------
`str`   | `char`         | `[]u8`                  | utf-8 string
`str7`  | `char7`        | `[]char7`               | utf-7 string
`str8`  | `char8`        | `[]char8`               | ANSI string
`str16` | `char16`       | `[]char16`              | utf-16 string
`str32` | `char32`       | `[]char32`              | utf-32 string
`cstr`  | `char8`        | `[*;0]char8`            | C-style string

### 10.1.11. Pointer types

```
<pointer-type> := ( '*' | '[' '*' [ ';' <expr> ] ']' ) ( 'const' | 'mut ) <type>
```

A pointer type represents an address in memory containing hte underlying type.
Pointer do not have any safety guarantees.
Copying or dropping pointer has no effect on the lifecycle of any value.
Derefencing a pointer is an `unsafe` operation.

Raw pointer are generally discourages, and are mainly there to allow for interopterability and perfomance-critical and low-level functionality.
It is preferable to use smart pointer wrapping the inner value.

When comparing pointers, they are compared by their address, rather than what they point to.
When comparing pointers to dynamically sized types, they also have their additional metadata compared.

A pointer cannot contain a 'null' value when not in an optional type.

Xenon has 3 kinds of pointers:
 
#### Single element pointers

A single element pointer `*const T` or `*mut T`, refers to exactly 1 element in the memory pointed to.

This pointer can be converted to a reference by re-borrowing it using `&*` or `&mut *`.

Single element pointer do not support any pointer arithmetic.

As an example, the pointer `*const i32` would represent a pointer to a single immutable `i32` value.

#### Multi element pointers

A multi-element pointer `[*]const T` or `[*]mut T`, pointing to an unknown number of elements.

Multi-element pointers allow, in addition to standard pointer functionality, also to be index and have pointer arithmetic to be performed on them.
When a pointer contains dynamically sized types, it will consist out of an array of fat pointers.

As an example, the pointer `[*]const i32` would represent a pointer to an unknwon number of immutable `i32` values.

#### Sentinel terminated pointer

A sentinel terminated pointer `[*;N]const T` or `[*;N]mut T` is very similar to a multi-element pointer.
The main difference lies in the fact that a sentinel terminated pointer will only contain the number of elements until the first occurance of the sentinel value.

The main purpose of this type is to prevent buffer overflows when interacting with C-style and OS code.

### 10.1.12. Reference types

```
<reference-type> := `&` [ 'mut' ] <type>
```

A reference type, like a pointer, point to a fiven value in memory, but which is owned by another value.
Copying a reference is a shallow opertion and will only copy of just the pointer to the memory, and any metadata required for dynamically sized types.
Releasing a reference has no effect on the lifecycle of the value it points to, except when refernecing a temporary value, it will keep it alive during the scope of the reference itself.

References are split into 2 types:

#### Shared reference

A shared reference prevents direct mutation of the value, but interior mutability provides an exception for this in certain circumstances.
As the name suggets, any mubmer of shared references to a value may exist.

A shared reference is written as `&T`.

#### Mutable reference

Mutable references (which haven't been borrowed) allow the underlying value to be directly modified.

A mutable reference is written as `&mut T`.

#### 10.1.13. Optional types

```
<optional-type> := '?' <type>
```

An optional type allows a value to be represented using a 'null' or `None` state, which can be used to represent a type with no value set.
When an optional type (or the `Option<T>` type) is used, then depending on the value within, the compiler is allowed to do certain optimizations to encode the 'null' state within the value.
An example is a nullable pointer, where the 'null' state is represented with an address of `0x00000000`.

This is synctactic suger of `Option<T>`.

### 10.1.14 Function types

A function type is an anonymous compiler-generated type, which cannot be manually defined.
The type references a specific function, including its name and its signature (including parameter labels).

Since this is specific to each function, a value of this type does not need to use any indirection to be called, as it does not contain an actual function pointer.
This also makes this type a 0-sized type.

Separating each function in its own type allows for additional optimization.

When an error message is generated using this type, it will generally show up as something like `fn(_:i32) -> i32 { name }`

### 10.1.15. Function pointer type

```
<fn-type> := [ 'unsafe' [ 'extern' <abi> ] ] 'fn' '(' <fn-type-params> ')' [ '->' <type-no-bounds> ]
<fn-type-params> := <fn-type-param> { ',' <fn-type-param> }* [ ',' ]
<fn-type-param> := { <attribute> }* [ ( <ext-name> | '_' ) { ',' ( <ext-name> | '_' ) }* ':' ] <type>
```

A function pointer type can refer to a function whose identity is not known at compile time.
The can be created via coercion from functins and non-capturing closures with a matching signature.

If a function is marked `unsafe`, it is able to be assgined from both safe and unsafe functions, and must be called from an unsafe context.
To assign a pointer with a specific ABI, the function needs to be marked as an `extern` function with a matching ABI.
If not marked with a ABI, it will use the default Xenon ABI.

Parameters may contain one or more names, but for the purposes of a function pointer these are ignored, but are instead usefull for documentation.
If multiple names are are given for a single parameter, these will be separate parameters with the same type.

_TODO: Variadic paramters, if possible_

### 10.1.16. Closure types

A closure type is a compiler generated type which cannot be declared manually, and refers to a closure using a unique anymous type.

For more info about closure, see the [closure expression].

### 10.1.17. Intereface Object types

```
<interface-object-type> := 'dyn' <interface-bound> { '+' <interface-bound> }*
```

An interface object type is an opaque type that implements a set of interfaces, any set of interfaces is allowed, except of an opt-in interface like `?Sized`.
The objects are guaranteed to not only implement the given interfaces, but also their parent interfaces.

Different interface objects may alias each other if the interfaces match, but are in different orders, meaning that `dyn A + B + C` is the same as `dyn A + B + C`

An intereface can be assigned to a less specific interface objects, meaning that it can be assgined to a type that has less interface bounds.
This *may* incur some additional overhead, as a new vtable needs to be retrieved and assigned, if this cannot be determined at compile time.

Due to the opaqueness of interface objects, this type is dynamically sized, meaning that it must be stored behind a reference, a pointer, or a type accepting DTSs.

Interface objects are stored in so-called "fat pointers' which consists out of 2 components:
- A pointer to the an object of a type `T` that implements the interface bounds
- A virtual table, also known as a vtable, which contains both RTTI info and a list of function pointers to the methods of the interfaces and their parent types, of `T`'s implementation.

Interface object types allowe for "late binding" in cases where the types being used cannot be known at compile time, but the programmer knowns the functionality they posses.
Calling a method will use a virtual dispatch of the method: that is, teh function pointer is loaded from the vtable, and is then invoked indirectly, incurring a pointer indirection.
The actual implemention of each vtable may vary on an object-by-object basis.

### 10.1.18. Impl interface types

```
<impl-interface-type> := 'impl' <interface-bound> { '+' <interface-bound> }
```

An impl interface type introduces an unnamed generic parameter that implements the given intrefaces to the item it is used in.
It can appear in only 2 locations: function paramters (where it acts as an anonymous type of the parameter to the function) and function return types (where it acts as an abstract return type).

#### Anonymous type parameter

A function can use an impl interface type as the type of its parameter, where it declares the parameter to be of an anonymous type.
The caller must provide a type that statisfies the bounds declared in the anonymous type paramter, and the function can only use the functionality available through the interface bounds of the anonymous type paramter.

An example of this would be:
```
interface Interface {}

// Generic type parameter
fn with_generic_type<T is Interface>(param: T) {}

// impl interface typed paramter
fn with_impl_type(param: impl Interface) {}
```

This can be seens as synctactic sugar for a generic type paramter like `<T is Interface>`, except that the type is anonymous and does not appear within the generic argument list.

> _Note_: For function arguments, generic type parameters and `impl Interface` are not completely equivalent
> With a generic type paramter `<T is Interface>`, the caller is able to explicitly specify the type of the generic type parameter `T` when calling the function.
> If an `impl Interface` is used, the caller cannot ever specify the type of the parameter when calling the function.
>
> Therefore, changing between these types within a function signature should be considered a breaking change.

#### Abstract return types

A function can use an impl interface type as the type in its return type.
These types stand in for another concrete type wher the caller may only used the functinality declared by the specified interfaces.
Each possible return type of the function must resolve to the same concrete type.

An `impl Interface` in the return allows to return a abstract type that does not have to be stored within dynamic memory.
This can be particularly usefull when writing a function returning a closure or iterator, as for example, a closure has an un-writable type.

Without this functionality, it would only be possible to return a 'boxed' type:
```
fn returns_closure() -> Box<todo> {
    Box::new(|x| x + 1)
}
```

This could incur performance panalties from heap allocation and dynamic dispatching.
However, using this type, it is possible to write it as:

```
fn returns-closure -> impl todo {
    |x| x + 1
}
```

Which avoids the drawbacks of the 'boxed' type.

_TODO: add note on (memory) effect implications_

#### Abstract return types in interface declarations

Functions in interfaces may also return an abstract return types, this will create an anonymous associated type within the interface.

Evety `impl Interface` in the return type of an associated function in an interface is desugared to an anonymous associated type.
The return type that appears in teh implementation's funciton signature is used to determine the value of hte associated type.

##### Differences between generics and `impl Interface` in a return

When used as a type argument, `impl Interfaces` work similar to the semantics of generic type parameters.
But when used in the return, there are significant changes, as unlike with a generic parameter where the caller can choose the return type, the implementation chooses the function's return type.

For example, the function
```
fn foo<T is Interface>() -> T { ... }
```
Allows the caller to determine the return type.

In contrast, the function
```
fn foo() -> impl Interface { ... }
```
doesn't allow the caller to explicitly determine the return type.
Instead the function chooses the return type, with the only guarantee that it implements the required interfaces.

#### Impl interface limitations

An impl interface type may only occur for non-`extern` functions.
It can also not be the type of a variable declaration, a field, or appear inside a type alias.

### 10.1.19. Record types

```
<record-type> := '{' <record-members> '}'
<record-members> := <record-member> { ',' <record-member> }* [ ',' ]
<record-member> := { <attribute> }* <ext-name> { ',' <ext-name> }* ':' <type>
```

A record is a _structural_ type is that, similarly to a tuple, consisting out of a list of fields of other types.

But unlike a tuple, fields can be given explicit names, which can then be used to index the fields of the record.

### 10.1.20. Enum record types

```
<enum-record> := 'enum' '{' <enum-fields> '}'
```

An enum record is a _structural_ type and is a variant of a record.

Unlike a record, it does not represent a collection of fields, but a type that is similar to that of an `enum`.
Access to enum members work essentially identical than those of an enum.

### 10.1.21. Inferred types

```
<inferred-type> := '_'
```

An inferred type tell the compiler to infer the type (if possible) based on the surrounding information available.
Inferred types cannot be used in generic arguments.

Inferred types are often used to let the compiler infer the type of generic parameters:
```
TODO
```

## 10.2. Dynamically sized types

Most types have a fixed size that is known at compile time and implements the `Sized` interface.
A type wit ha size tha is only known at compile-time is called a dynamically sized type (DST), or informally, unsized types.
Slices and interface objects are two such examples.

DSTs can only be used in certain cases:
- Pointers and references to DSTs are sized, but have twice the size of a pointer of a sized type.
    - Pointers to slices store the number of elements in the slice.
    - Pointers to interface objects store a pointer to their vtable.
- DSTs can be provided as type arguments to generic type parameters that have a special `?Sized` bound.
  They can also be used for associated type definitions when the corresponding associated type is declared using the `?Sized` bound.
  By default, any type parameter has a `Sized` bound, unless explicitly relaxed using `?Sized`
- Interface may be implemented for DSTs.
  Unlike with generic type paramters, `Self is ?Sized` is the default in interface definitions.
- Struct may contains a DST as the last field, this makes the struct itself a DST.

## 10.3. Nominal vs structural types

Xenon has types that can either be nominal or structural, between these 2 kinds of types.

Both have the same type layout and mutability rules, but there are some important differences:

Nominal types:
- Nominal types do **not** implicitly implement any interfaces.
- Nominal types can have additional functionality and interfaces implemented.
- All field have configurable visibility.
- The types can be accessed directly from other scopes when 'imported'.

Structural types:
- Structural types implicitly implement a set of interfaces, depending on the values of the members, these are:
    - `Clone`
    - `Copy`
    - `PartialEq`
    - `Eq`
    - `Hash`
    - `Debug` _TODO: this will likely the be the trait, but depends on the standard format implementation._
- Structural types do not allow any additional functionality to be implemented, as they are strictly plain data types.
- Fields cannot have explicit visibility.
- The types only exist within the scope they are defined, unless publically aliased.


## 10.4. Type layout

The layout of a type defines its size, alignment, and its internal representation of data/fields.
For enums, how their distriminant is laid out is also part of the layout.

Type layouts can change inbetween compilations.

### 10.4.1. Size and Alignment

All values have a size and alignment.

The alignment of a value specifies at what boundaries in memory the value can be stored.
A type with alignment `N` must be stored at an address that is a multiple of `N`.
Alignment is measured in bytes, is at least 1, and is a power of 2.

The size of a value specifies the offset that is needed to be able to place the next value, e.g. the offset of the subsequent element in an array.
The size will always be a multiple of its alignment, guruaranteeing that any subsequent value of this type will be correctly aligned by default.

Is it possible for a given type to be a zero-sized type, as a size of 0 is a valid multiple of its alignment.
On some platforms, a zero-sized types might still be required to follow a specific alignment, e.g. in the case of `[0]i32`, the value needs to be aligned to `4`.

the majority of types will know their size and alignment at compile time, these are called 'sized types'.
Sized types can have their size and alignment checked at compile time.
Meanwhile types that are not known at compile time, as known as [dynamically sized types](#102-dynamically-sized-types).

Since all values of a sized types share their size and alignment, we say that they have the type's size and alignment.

### 10.4.2. Primitive layout

The size of most primitive types can be found in the table below:

Types                                                | Size/Alignment (bytes) | Size in bitfield (bits) | Alignment in bitfield (bits)
-----------------------------------------------------|------------------------|-------------------------|------------------------------
`i8`   / `u8`            / `b8`  / `char8`           | 1                      | 8                       | 8
`i16`  / `u16`  / `f16`  / `b16` / `char16`          | 2                      | 16                      | 16
`i32`  / `i32`  / `f32`  / `b32` / `char32` / `char` | 4                      | 32                      | 32
`i64`  / `i64`  / `f64`  / `b64`                     | 8                      | 64                      | 64
`i128` / `u128` / `f128`                             | 16                     | 128                     | 128
`usize` / `isize`                                    | see below              | see below               | see below
`bool`                                               | 8                      | 1                       | 1
`char7`                                              | 1                      | 7                       | 1

`usize` and `isize` are different to other types, as they contain types that fit the entire memory address space of the target platform.
For example, on a 32-bit system, this is 4, and on an 64-bit system, this is 8.
These sized also often match up with that of the target register size, but this cannot be guaranteed.

The alignment of types is generally platform-specific, but to keep this consistent across architectures, Xenon has diced to make these the same as their size.

When used in a bitfield, some primitive types may have different sizes and alignment to fit more tightly into memory.

### 10.4.3. Unit and never type layout

Unit and never types are both 0-sized types with an alignment of 1.

### 10.4.4. Pointer and reference layout

Pointers and references have the same layout.
The mutabilty of a pointer or reference has not impact on the layout.

Pointers and references to sized tyes are the same as those of a `usize`.

Pointers and references to usized types are typed. Their size and alignement is guaranteed to be at least eqal to the size of a `usize` and have the same alignment.

> Note: Currently all pointers and references to DST are twice the size of a `usize` and have the same alignment.
>       Although this should not be relied on.

### 10.4.5. Array layout

An array of the form `[N]T` has a size that is `N` times that of the size of type `T` and has the same alignment as type `T`.
Arrays are laid out so that the zero-based `n`th element of the array is offset from the start of the array by `n` times the size of type `T`.

When an array is sentinal terminated, the array contains an additional element of type `T` at the end, so the size of the array will be `N + 1` times the size of type `T`.

### 10.4.6. Slice layout

Slices have the same alyout as a section of an array

> Note: This is about the ray `[]T` type, not pointers to arrays to slices, e.g. (`&[N]T`)

### 10.4.7. String slice layout

A string slice's layout depends on the type of string slice, but they have the same representation as their internal slice layout.

Below is a table of string slices that have a corresponding type layout to the following slice types

String slice | Slice
-------------|-------
`str`        | [u8]
`str7`       | [char7]
`str8`       | [char8]
`str16`      | [char16]
`str32`      | [char32]
`cstr`       | [char8]

### 10.4.8. Tuple layout

Tuples are laid out as defined in the [Xenon representation]().

### 10.4.9. Interface object layout

Interface objects have the same layout as the value the interface that implements it.

> Note: THis is for the interface object itself, not a type containing the object, such as a reference.

### 10.4.10. Closure layout

A closure has no layout guarantees.

### 10.4.11. Bitfield layout

A bitfield will have the size and alignment of the smallest primitive types that fits the contents of the bitfield.

### 10.4.12. Layout representation

All user-defined composite types have a representation that specifies how the type is laid out.
The possible representations for these are:
- `xenon`
- `C`
- `soa`
- primitive type
- `transparent`

While the representation of a type can affect the padding between fields, it does not change the layout of the fields themselves.
If a composite type contains a field that had another layout already defined, that field will still use its own layout representation, and will not use the layout representation of the type containing it.

#### Xenon representation

The `xenon` represention is the default representation for nominal types without a `repr` attribute.
If this representation is explicitly specified by using the `repr` attribute, it will result in the same layout as if it was not explicitly defined.

This represetnation makes a mininal amount of guarantees about the layout of fields, but does guarantee the following:
- Each field is properly aligned
- Fields do not overlap
- The alignment of the type is at least that of teh field with the highest alignment.

The first guarante means that the offset of a field will always be a multiple of its alignment.
The second guarantee means that the fields can be ordered such that the offset plus the size of any field is less than or equal to the offset of the next field in the type.
This does not mean that zero-sized fields will have a unique offset and multiple zero-sized fields may be located at the same address.
The third guarantee ensures alignment of the all fields can always be guaranteed.

There is no guarantee that the ordering is the same as the one defined within code.

##### Field priority

Since by default there is no guarantee on the ordering of fields, the type may lay out fields in such a way that they may not be optimally laid out for some usecases.
To ensure the programmer can provide additional hints to the compiler which fields should be prioritized during layout to ensure better caching of the type,
a field priority propery may be defined per field.

The priority takes on a value in the range of 0..=15, with 0 being the default for all fields.
Fields with a higher priority will be prefered to be laid out first in the type.

```
struct Foo {
    big: [256]u8,
    // Ensure the compiler lays out the fields in such a way that important will be most likely to be on a cache line
    #[field_priority(15)]
    important: u32,
}
```

#### C representation

The C representation has 2 purposes:
- creating types that are interoperable with C libraries/code.
- allow types to be laid out in such a way that the layout of the type can be relied on.

This representation can be applied to `struct`s, `enum`s, and `union`s, with the exception of zero-varient enums.

The C representation also affects the alignment of primitive types for the current target architecture.

##### `repr(C)` structs and records

The alignment of a struct will be that of the most-aligned field.

The size of the type, and the size and offset of the fields will be determined uisng the method described below.

The current offset start at 0, then for each field within a type:
1. determine the size and alignment of the field
2. if the current offset is not a multiple of the field's alignment, set the current offset to the next multiple of the field's alignment. This space is padding.
3. the current offset will now become the offset for the field
4. increment the current offset by the size of the field.

> Note: This algorithm can produce zero-sized structs.
>       While this is generally considered to be illegal in C, some compiler support option to enable zero-sized structs.
>       Meanwhile C++ gives empty structures a size of 1, unless the are inherited or have fields using the `[[no_unique_address]]` attribute,
>       in which case they do not contribute to the size of the overall struct.

##### `repr(C)` unions

A union with a C representaton has the same layout as the union would have if it were defined in C for the target platform.

The union will have the size of the largest fields in the union, and the alignment of the most-aligned field in the union.
These values may be taken from different fields.

##### `repr(C)` field-less enums and enum records, and flags enums

When an enum is field-less, the C representation has the size and alignment of the default `enum` size for the target platform's C ABI.

> Note: The enum representation in C is implementation defined, so this is really a "best guess".
>       In particular, this may be incorrect when the C code of interst is compile with certain flags
>       If a known enum size is required, use a primitive represention.

##### `repr(C)` enums and enum records with fields

The representation of an enum with fields is defined a `repr(C)` structure with 2 fields, these being:
- a `repr(C)` version of the enum with all field removed, i.e. the "tag"
- a `repr(C)` union of `repr(C)` records for the field of each variant that had them, i.e. the `payload`

#### Primitive representation

A primtiive representation is only allowed for `enum` values that have at least 1 variant, and on bitfields.

The allowed primitive types are `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, `ui`, `i16`, `i32`, `i64`, `i128`, and `isize`.
When defining an enum with a primitive representation, an enum will use this type as its descriminant.

If an enum has no fields, the resulting enum will have the same size and aligment as the primitive type it is represented by.

When an enum has fields, it will be represented as a `repr(C)` enum, with its payload using the `repr(C)` representation.
In addition to the primitive representation, a second (non-primitive) representation may be provided, affecting the layout of the payload of the enum.

#### Transparent representation

The transparent representation is only supported for structures and enum with only 1 variant, which have the following:
- a single field with a non-zero size
- any number of field with a 0-sized type and alignment 1

Type using this representation have the same lyout and ABI as the single non-zero field.

Unlike other representations, a type with this represetnation takes on that of the underlying non-zero sized type.

#### SAO (structure of array) representation
_TODO_

#### Additional layout modifiers

The `repr` attribute may also optionally contain an `align` or `packed` value, these can be used to raise or lower the alignment respectively.
On their own, neither provide guarantees about the ordering of any fields in the layout of the type, although they may be combined with representations such as 'C', which do provide such guarantees.

Either modifier may be applied to structs, unions, and records.
In addtion, `align` may also be applied to enums and enum records.

The alignment specified by either the `align` of `packed` attributes must be a power of 2 from 1 up to 2^32.
For `packed`, if no explicit alignment is given, this will default to 1.

The `align` modifier changes the minimum alignment for the type, if the value given is lower than the actual alignment of the type, the alignemnt is unaffected, otherwise, it will increase the alignment to the given value.

The `packed` modifies affect the alignment of each fields within the type, but does not chang the alignment of the layout of the fields themselves.
If this alignment is larger than the alignment of the type, the offset of fields are unaffected.
Otherwise the offset of fields is affect, as this modifier affects the minimal required alignment of fields that is decided by the current representation, i.e. fields will be aligned to the alignment provided to the attribute.

Only one of the `align` or `packed` modifiers can be applied to a type at any type, and it may only be applied to types with either a `xenon` or `C` representation.

## 10.5. Interior mutability

Sometimes a type needs to be mutated while having multiple aliases.
This can be achieved using a concept called _interior mutability_.
A type has interior mutability if its internal state can be modified from a shared reference to it.
This goes against the usual requirement that the value pointed to by a shared reference is not mutated.

`UnsafeCell<T>` is the only way of disabling this requirement.
When `UnsafeCell<T>` is immutably aliased, it is still safe to mutate or obtain a mutable reference to the `T` it contains.
As with all other types, it is undefined behavior to have multiple `&mut UnsafeCell<T>` aliases.

Other types with interior mutabiliity can be created using `UnsafeCell<T>` as a field.

> **Warning**: The programmer must ensure that this does not cause any unininted consequences or may cause other undefined behavior.

## 10.6. Type coercions

Type coercions are implicit operations that change the type of a value.
They happen automatically at specific locations and are highly restricted in what types are allowed to coerce.

Any conversions allowed by coercion can als obe explicitly performed using the type cast operator `as`.

> _Note_: This description is informal and not yet fully defined, and should be more specific

### 10.6.1. Coercion sites

A coersion can only occur at certain sites in a program; these are typically places weherere the desired type is explicit or can be derived from explicit types (without type interference).
Possible coercion sites are:
- variable declarations where an explicit type is given.
- `static` and `const` item declarations (similar to variable declarations)
- Arguments to function calls
- Default paramter values for functions
- Instantiations of struct, unions, records, and enum and enum record variants fields
- Default field values
- Function results - either the final line of a block if it is not semi-color-terminated, or any expression in a `return` statement

If the expressions in one of these coercion sites is a ceorcion-propagating expression, then the relevant sub-expressions in that expression are also coercion sites.
Propagation recurses from these new coercion sites.
Propagating epxresson and their relevant sub-expressions are the following:
- Array literals, where the array has type `[n]T`.
  Each sub-expression in the array literal corecion sites for coercion to type `T`.
- Array literals with a repeating syntax, where the array has type `[n]T`.
  The repeating sub-expression is a coercion site for a coercion to type `T`.
- Tuples, where a tuple is a coercion site of type `(T0, T1, ..., Tn)`.
  Each sub-exprssion is a coercion site to the respective type, e.g. the 0th sub-expression is a coercion site to type `T0`.
- Parenthesized sub-expressions ( `(e)` ): if the sub-expression has type `T`, then the sub-expression is a coercion site to `T`.
- Blocks: if a block has type `T`, the the last expression (if it is not semicolon terminated) is a coercion site to `T`.
  This includes blocks which are part of control flow statements, such as `if`/`else`, if the block has a known type.


### 10.6.2. Coecion types

Coercions are allowed betweeen the following types:
- `T1` to `T3`, if `T1` coerces to `T2` and `T2` coerces to `T3`
- `&mut T` to `&T`
- `*mut T` to `*T`
- `[*]mut T` to `[*]T`
- `[*;x]mut T` to `[*;x]T`
- `&T` to `*const T`
- `&mut T` to `*mut T`
- `&T` or `&mut T` to `&U`, if `T` implements `Deref<Target = U>`
- `^mut T` to `&mut U`, if `T` implements `DerefMut<Target = U>`
- Function item types to `fn` pointers
- Non capturing closures to `fn` pointers
- `!` to any `T`

> _NOTE_: Since coercion are not anywhere close to being finalized, this list is incomplete

### 10.6.3. Unsized coercions 

The following coercions arr called `unsized coercions`, since they relate to conversting sized types, and are permitted in a few cases where other coercions are not, as described above.
They can still happen anywhere a coercion can be done.

Two interfaces `Unsize` and `CoerceUnsized`, are used to assigst in this process and expose it for library use.
The following coercions are built-in and if `T` can coerce to `U` with one of them, than an implementation for `Unsize<U>` will be provide:
- `[n]T` to `[]T`
- `T` to `dyn U`, when T implements `U +Sized` and `U` is object safe.
- `Foo<..., T, ...>` to `Foo<..., U, ...>` when:
    - `Foo` is a struct
    - `T` implements `Unsize<U>`
    - The last field of `Foo` has a type involving `T`
    - If that field has type `Bar<..., T, ...>`, then `Bar<..., U, ...>` implements `Unsize<Bar<..., U, ...>>`
    - `T` is not part of hte type of any other fields

Additionally, a type `Foo<T>` can implement `CoerceUnsized<Foo<U>>` when `T` implements `Unsize<U>` or `CoerceUnsized<U>`.
This allows it to provide an unsized coercion to `Foo<T>`

> _NOTE_: Since coercion are not anywhere close to being finalized, this is incomplete

### 10.6.4. Least upper bound coercions

In some contexts, the compiler must coerce together multiple types to try and find the most general type.
This is called a "Least Upper Bound" coercion, or LUB coercions in short.
A LUB coercion is used and only used in the following situations:
- To find the common type for a series of `if` branches
- To find the common type for a series of `match` arms
- To find the common type between array elements
- To find the type for the return type of a closure with multiple return statements
- To check the type for the return tpe of a function with multiple return statements.

In each such case, there are a set of types `T0..Tn` to be mutually coerced to target type `Tt`, which is unknonw at the start.
Computing the LUB coercion is done iteratively.
The target type `Tt` begins as `T0`.
For each new type `Ti`, we cosider:
- If `Ti` can be coerced to the current target type `Tt`, then no change is made.
- Otherwise, check whether `Tt` can be coerced to `Ti`; if so, then `Tt` is changed to `Ti`.
  (this check is also conditional on whether all of the source expressions cosidered ths far have implicit coercions).
- If not, try to compute a mutual supertype of `Tt` and `Ti`, which will become the new target type.

If this fails, it will result in a compiler error.


## 10.7. Destructors

When an initialized variable or temporary goes out of scope, its destructor is run, or it is _dropped_ (this terminology is taken from rust).
Assignment also runs the destructor of its left-hand operatnd, if it's initialized.
If a variable has been partially initialized, only its initialized fields are dropped.

The destructor of a type `T` consists out of:
1. If `T is Drop`, calling `<T as Drop>::drop`, or
2. Recursively running the destructor of all its fields
    - The fields of a struct or record are dropped in declaration order
    - The fields of the active enum or enum record variant are dropped in declaration order
    - The fields of a tuple are dropped in order
    - The elements of an array or owned slice are dropped from the first element to the last.
    - The variables that a closure captures by move are dropped in an unspecified order
    - Interface objects run the destructor of the underlying type
    - Other types don't result in any further drops

If a destructor must be run manually, such as when implementing a smart pointer, `drop_in_place` can be used.

### 10.7.1. Drop scopes

Each variable or temporary is associated with a drop scope.
When control flow leaves a drop scope, all variables associated to that scope are dropped in reverse order of declaration (for varialbes) or creation (for temporaries).
Values are only dropped after running all `defer` statements within the same scope.

Drop scopes are determined after replacing `for`, `if` and `while` expressions (with let bindings) with the equivalent using `match`.
Overloaded or user-defined operators are not distinguished from built-in operators and binding modes are not considered.

Given a function, or closure, there aer drop scopes for:
- The entire function
- Each statement
- Each expression
- Each block, including the function body
    - In the case of block expressions, the scope for the block and the expressions are the same scope.
- Each arm of the `match` expression

Drop scopes are nested within each other as follows.
When multiple scopes are left at once, such as when returning from a function, variables are dropped from the inside outwards.
- The entire function scope is the outer scope
- The functon body block is contained within the scope of the entire function.
- The parent of the expression is an expression statement is the scope of the statement.
- The parent of the expression of a variable declaration is the declaration's scope.
- The parent of the statement is the scope of the block that contains the statement.
- The parent of the expression for a `match` guard is the scope of the arm that the guard is for.
- The parent of the experssion after the `=>` in a `match` is the scope of the arm it's in.
- The parent of the arm scope is the scope of the `match` expression that it belongs to.
- The parent of all other scopes is the cope of hte immediately enclosing expression.

### 10.7.2.  Scopes of function parameters

All function paramters are in the scope of the entire function, so are dropped last when evaluating the function.
Each actual function parameter is dropped after any bindings introduced in that parameter's pattern.

_TODO: Example_

### 10.7.3. Scopes of local variables

Local variables declared in a variable declaration are associated to the scope that contains the declaration.
Local variables declared in a `match` expression are associated to the arm scope of the `match` that they are declared in.

_TODO: Example_

If multiple patterns are used in the same arm of a `match` expressions, then an unspecified pattern will be used to determin the drop order.

### 10.7.4. Temporary scopes

The temporary scope of an expressions is the scope that is used for the temporary variable that holds the result of he exprssion when used in a place context, unless it is promoted.

Apart from lifetime extensions, the temprory scope of an expression is the smallest scoped that contins the expression and is one of the following:
- The entire function.
- A statement.
- The body of an `if`, `while` or `loop` expression.
- The `else` block.
- The condition expressions of an `if` or `while` expression, or a `match` guard.
- The body expression for a `match` arm.
- The second operand of a lazy boolean operator.

> _Note_: Temporaries that are created in the final exprssion of a function body are dropped after any named variables bound in the function body.
>         Their drop scope is the entire function, as tehre is no smaller enclosing temporary scope.
>
>         The scrutinee of a `match` expression is not a temporary scope, so temporaries in the scrutinee can be dropped after the `match` expression.
>         For example, the temporary for `1` in `match 1 { ref mut z => z };` lives until the end of the statement.

_TODO: Example_


### 10.7.5. Operands

Temporaries are also created to hold the result of operands to an expressions while the other operands are evaluated.
The temporaries are associated to the scope of the expressions with that operand.
Since the temporaries are moved from once the expreesssion is evaluated, dropping them has no effect unless one of the operands to an expression break out of he expression, returns, or panics.

_TODO: Example_

### 10.7.6. Constant promotion

Promotion of a value expression to a `static` slot occurs when the expression could be written in a constant and borowed, and that borrow could be dereferenced where the exprssion was originally written, without changing the runtime behavior.
That is, the promoted expression can be evaluated at compile-time and the resulting value does not contain [interior mutability](#105-interior-mutability) or [destructors](#107-destructors) (these properties are determined based on the value when possible).

### 10.7.7. Temporary lifetime extension

> _Note_: This is subject to change

The temporary scopes for expressions in variable declarations are sometimes extended to the scope of the block containing the declaration.
This is done wherer the usual temporary scope would be too small, based on syntactic rules.

If a borrow, dereference, field, or tuple expression has an extended temporary scope, the nteh indexed experssions also has an extended scope.

### 10.7.8. Extending based on patterns

An extending pattern is either:
- An identifier pattern that binds by refernce or mutable reference.
- A struct, tuple, tuple struct, or slice pattern where at least one of the direct subpatterns in an extending pattern.

So `ref x`, `V(ref x)` and `[ref x, y]` are all extending patterns, but `x`, `&x` and `&(ref x, _)` are not.

If the pattern in a variable declaration is an extending pattern, then the temporary scope of the initializer expression is extended.

### 10.7.9. Extending based on expressions

For a variable declaration with an initializer, an extending expression is an experssion whici is one of the following:
- The initializer expression.
- The operand of an extending borrow experssion.
- The operand of an extending array, cast, braced struct, or tuple expression.
- The final expression of any extending block expression.

So the borrow expression is `&mut 0`, `(&1, &mut 2)`, and `Some{ 0: &mut 3 }` are all extending expressions.
The borrows in `&0 + &1` and `Some(&mut 0)` are not: the latter is syntactically a function call expression.

The operand of any extending expression has its temporary scope extended.

### 10.7.10. Not running destructors

`forget` can be used to prevent the destructor of a variable from being run, `ManuallyDrop` provides a wrapper to prevent a variable or field from being dropped automatically.

> _Note_: Preventing a destructor from being run via `forget` or other means is safe even if the type isn't static.
>         Besides the place where destructors are guaranteed to run as defined by this document, types may not safely rely on a destructor being run for soundness.

# 11. Generics
_TODO_

# 12. Macros
_TODO_

# 13. Operators and Precedence
_TODO_

# 14. Attributes
_TODO_

# 15. Implicit context
_TODO_

# 16. Effect system
_TODO_

# 17. Contracts
_TODO_