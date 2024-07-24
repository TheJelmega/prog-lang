# Xenon language design

Version: 0.0

## Tables of contents

1. [Introduction](#1-introduction-)
    - [This document is provisional](#this-document-is-provisional)
2. [Lexical structure](#2-source-code-representation-)
    1. [Input format](#21-input-format-)
    2. [Byte order markers](#22-byte-order-markers-)
    3. [Newline sequences](#23-newline-sequences-)
    4. [Shebang](#24-shebang-)
3. [Lexical structure](#3-lexical-structure-)
    1. [Whitespace](#31-whitespace-)
    2. [Keywords](#32-keywords-)
        - [Strong keywords](#strong-keywords)
        - [Reserved keywords](#reserved-keywords)
        - [Weak keywords](#weak-keywords)
    3. [Comments](#33-comments-)
4. [Package Stucture](#4-package-structure-)
    1. [Packages](#41-packages-)
    2. [Artifacts](#42-artifacts-)
    3. [Modules](#43-modules-)
5. [Names and path](#5-names-and-paths-)
6. [Literals](#6-literals-)
    1. [Numeric literals](#61-numeric-literals-)
        1. [Decimal literals](#611-decimal-literal-)
        2. [Binary literals](#612-binary-literals-)
        3. [Octal literals](#613-octal-literals-)
        4. [Hexadecimal integer literals](#614-hexadecimal-integer-literals-)
        5. [Hexadecimal floating-point literals](#615-hexadecimal-floating-point-literals-)
    2. [Boolean literals](#62-boolean-literals-)
    3. [Character literals](#63-character-literals-)
        1. [Escape codes](#631-escape-codes-)
    4. [String literals](#64-string-literals-)
        1. [String coninuation sequence](#641-string-continuation-sequence-)
    5. [Literal operators](#65-literal-operators-)
7. [Items](#7-items-)
    1. [Module item](#71-module-item-)
        1. [Inline modules](#711-inline-modules-)
        2. [File modules](#712-file-modules-)
        3. [Path attibute](#713-path-attribute-)
    2. [Use declaration](#72-use-declarations-)
        1. [Use visibility](#721-use-visibility-)
        2. [Underscore imports](#722-underscore-imports-)
    3. [Functions](#73-function-)
        1. [Parameters](#731-parameters-)
        2. [Returns](#732-returns-)
        3. [Pseudo-function overloading](#733-pseudo-function-overloading-)
        4. [Const function](#734-const-function-)
        5. [Trait function](#735-trait-function-)
            - [Trait function override resolution](#trait-function-override-resolution)
        6. [External function qualifier](#736-external-function-qualifier-)
    4. [Type aliases](#74-type-aliases-)
        1. [Distinct types](#741-distinct-types-)
        3. [Opaque types](#742-opaque-types-)
    5. [Structs](#75-structs-)
        1. [Regular structs](#751-regular-structure-)
            - [Use fields](#use-fields)
            - [Record struct](#record-struct)
        2. [Tuple structs](#752-tuple-structure-)
            - [Record tuple struct](#record-tuple-struct)
        3. [Unit structs](#753-unit-structure-)
    6. [Union](#76-union-)
        1. [Union field offsets](#761-union-field-offsets-)
        2. [Pattern matching on unions](#762-pattern-matching-on-unions-)
        3. [References to union fields](#763-references-to-union-fields-)
    7. [Enum](#77-enum-)
        1. [ADT enum](#771-adt-enum-)
            - [Discriminant](#discriminant)
            - [Field-less enum](#field-less-enum)
        2. [Record enum](#772-record-enums-)
        3. [Flag enum](#773-flag-enum-)
    8. [Bitfield](#78-bitfield-)
    9. [Const item](#79-const-item-)
        1. [Associated trait constant](#791-associated-trait-constant-)
    10. [Static item](#710-static-item-)
        1. [Thread local storage](#7101-thread-local-storage-)
        2. [Statics and generics](#7102-statics-and-generics-)
        3. [External statics](#7103-external-statics-)
    11. [Properties](#711-properties-)
        1. [Getters & setter](#7111-getters--setters-)
        2. [Internal representation](#7121-object-safety-)
    12. [Traits](#712-trait-)
        1. [Object safety](#7121-object-safety-)
        2. [Supertraits](#7122-supertraits-)
        3. [Usafe traits](#7123-unsafe-traits-)
        4. [Visibility](#7124-visibility-)
    13. [Implementation](#713-implementation-)
        1. [Inherent implementation](#7131-inherent-implementation-)
        2. [Trait implementation](#7132-trait-implementation-)
            - [Coherence](#coherence)
    14. [Associated items](#714-associated-items-)
        1. [Associated types](#7141-associated-types-)
            - [Associated trait type](#associated-trait-type)
        2. [Associated constants](#7142-associated-constants-)
        3. [Associated properties](#7143-associated-properties-)
        4. [Associated functions](#7144-associated-functions-)
    15. [External block](#715-external-block-)
8. [Statements](#8-statements-)
    1. [Item statement](#81-item-statement-)
    2. [Variable declaration](#82-variable-declaration-)
    3. [Expression statement](#83-expression-statement-)
    4. [Defer statement](#84-defer-statement-)
9. [Expressions](#9-expressions-)
    1. [Expression details](#91-expression-details-)
        1. [Place, value & assign expressions](#911-place-value--assign-expressions-)
            - [Place expressions](#place-expressions)
            - [Assign expressions](#assign-expressions)
            - [Value expressions](#value-expressions)
        2. [Move & copy types](#912-moved--copied-types-)
        3. [Mutability](#913-mutability-)
        4. [Temporaries](#914-temporaries-)
        4. [Implicit borrows](#915-implicit-borrows-)
    2. [Literal expressin](#92-literal-expression-)
    3. [Path exprssion](#93-path-expression-)
    4. [Unit expression](#94-unit-expression-)
    5. [Block expression](#95-block-expression-)
        1. [Unsafe block](#951-unsafe-block-)
        2. [Const block](#952-const-block-)
    6. [Operator expression](#96-operator-expression-)
    7. [Perenthesized expression](#97-parenthesized-expression-)
    8. [In-place expression](#98-in-place-expression-)
    9. [Type cast expression](#99-type-cast-expression-)
        1. [Builtin casts](#991-builtin-casts-)
            - [Numeric cast semantics](#numeric-cast-semantics)
            - [Enum cast semantics](#enum-cast-semantics)
            - [Primitive to integer cast semantics](#primtive-to-integer-cast-semantics)
            - [Integer to character cast semantics](#integer-to-character-cast-semantics)
            - [Pointer to address cast semantics](#pointer-to-address-casts-semantics)
            - [Pointer to pointer cast semantics](#pointer-to-pointer-semantics)
        2. [Type and unwrap casts](#99-2-try-and-unwrap-casts-)
    10. [Type check expression](#910-type-check-expression-)
    11. [Constructing expression](#911-constructing-expression-)
        1. [Tuple expression](#9111-tuple-expression-)
        2. [Array expression](#9112-array-expression-)
        3. [Struct expression](#9113-struct-expressions-)
            - [Struct (& union) expression](#struct--union-expression)
                - [Functional update syntax](#functional-update-syntax)
                - [Struct field shorthand](#struct-field-shorhand)
                - [Default fields](#default-fields)
            - [Tuple struct expression](#tuple-struct-expression)
            - [Unit struct](#unit-struct)
    12. [Intex expression](#912-index-expression-)
    13. [Tuple index expression](#913-tuple-index-expression-)
    14. [Call expression](#914-call-expression-)
        1. [Universal function call syntax (UFCS) & disambiguation function calls](#0141-universal-function-call-syntax-ufcs--disambiguating-function-calls-)
    15. [Method call expression](#915-method-call-expression-)
    16. [Field access](#916-field-access-)
        1. [Automatic dereferncing](#9161-automatic-dereferencing-)
        2. [Borrowing](#9162-borrowing-)
    17. [Closure experssion](#917-closure-expressions-)
        1. [Closure trait implementations](#9171-closure-trait-implementations-)
    18. [Range expression](#918-range-expression-)
    19. [If expression](#919-if-expression-)
        1. [If let](#9191-if-let-)
    20. [Loops](#920-loops-)
        1. [Loop expression](#9201-loop-expression-)
        2. [While expression](#9202-while-expression-)
            - [While let](#while-let)
        3. [Do-while expression](#9203-do-while-expression-)
        4. [For expression](#9204-for-expression-)
        5. [labelled block expression](#9205-labelled-block-expressions-)
        6. [Loop labels](#9206-loop-labels-)
    21. [Match expression](#921-match-expression-)
        1. [Match guards](#9211-match-guards-)
        2. [Fallthrough labels](#9212-fallthrough-labels-)
    22. [Break expression](#922-break-expression-)
    23. [Continue expression](#923-continue-expression-)
    24. [Fallthrough expression](#924-fallthrough-expression-)
    25. [Return expression](#925-return-expression-)
    26. [Underscore expression](#926-underscore-expression-)
10. [Patterns](#10-patterns-)
    1. [Literal pattern](#101-literal-pattern-)
    2. [Identifier pattern](#102-identifier-pattern-)
    3. [Wildcard patter](#103-wildcard-pattern-)
    4. [Rest pattern](#104-rest-pattern-)
    5. [Range pattern](#105-range-pattern-)
    6. [Reference pattern](#106-reference-pattern-)
    7. [Struct pattern](#107-struct-pattern-)
    8. [Tuple struct pattern](#108-tuple-struct-pattern-)
    9. [Tuple pattern](#109-tuple-pattern-)
    10. [Grouped pattern](#1010-grouped-pattern-)
    11. [Slice pattern](#1011-slice-pattern-)
    12. [Path pattern](#1012-path-pattern-)
    13. [Enum member pattern](#1013-enum-member-pattern-)
    14. [Alternative pattern](#1014-alternative-pattern-)
    15. [Type check pattern](#1015-type-check-pattern-)
11. [Types System](#11-type-system-)
    1. [Types](#111-types-)
        1. [Recursive types](#1111-rescursive-types-)
        2. [Parenthesized types](#1112-parenthesized-types-)
        3. [Primitive types](#1113-primitive-types-)
            - [Unsinged types](#unsigned-types)
            - [Signed types](#signed-types)
            - [Floating-point types](#floating-point-types)
            - [Boolean types](#boolean-types)
            - [Character types](#character-types)
        4. [Unit type](#1114-unit-type-)
        5. [Never type](#1115-never-type-)
        6. [Path types](#1116-path-types-)
        7. [Tuple types](#1117-tuple-types-)
        8. [Array types](#1118-array-types-)
        9. [Slice types](#1119-slice-types-)
        10. [String slice types](#11110-string-slice-types-)
        11. [Pointer types](#11111-pointer-types-)
        12. [Reference types](#11112-reference-types-)
            - [Shared reference](#shared-reference)
            - [Mutable reference](#mutable-reference)
        13. [Optional types](#11113-optional-types-)
        14. [Function types](#11114-function-types-)
        15. [Function poiner types](#11115-function-pointer-type-)
        16. [Closure types](#11116-closure-types-)
        17. [Trait Object types](#11117-intereface-object-types-)
        18. [Impl trait types](#11118-impl-trait-types-)
            - [Anonymous type parameter](#anonymous-type-parameter)
            - [Abstract return types](#abstract-return-types)
            - [Abstract return types in trait declarations](#abstract-return-types-in-trait-declarations)
            - [Limitations](#impl-trait-limitations)
        19. [Record types](#11119-record-types-)
        20. [Enum record types](#11120-enum-record-types-)
        21. [Inferred types](#11121-inferred-types-)
    2. [Dynamically sized types](#112-dynamically-sized-types-)
    3. [Nominal vs stuctural types](#113-nominal-vs-structural-types-)
    4. [Type layout](#114-type-layout-)
        1. [Size and alignment](#1141-size-and-alignment-)
        2. [Primitive layout](#1142-primitive-layout-)
        3. [Unit and never type layout](#1143-unit-and-never-type-layout-)
        4. [Pointer and reference layout](#1144-pointer-and-reference-layout-)
        5. [Array layout](#1145-array-layout-)
        6. [Slice layout](#1146-slice-layout-)
        7. [String slice layout](#1147-string-slice-layout-)
        8. [Tuple layout](#1148-tuple-layout-)
        9. [Trait object layout](#1149-trait-object-layout-)
        10. [Closure layout](#11410-closure-layout-)
        11. [Bitfield layout](#11411-bitfield-layout-)
        12. [Layout representation](#11412-layout-representation-)
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
    5. [Interior mutability](#115-interior-mutability-)
    6. [Type coercions](#116-type-coercions-)
        1. [Coercion sites](#1161-coercion-sites-)
        2. [Coercion types](#1162-coecion-types-)
        3. [Unsized coercions](#1163-unsized-coercions-)
        4. [Least upper bound coercions](#1164-least-upper-bound-coercions-)
    7. [Destructors](#117-destructors-)
        1. [Drop scopes](#1171-drop-scopes-)
        2. [Scopes of function paramters](#1172--scopes-of-function-parameters-)
        3. [Scopes of local variables](#1173-scopes-of-local-variables-)
        4. [Temporary scopes](#1174-temporary-scopes-)
        5. [Operands](#1175-operands-)
        6. [Constant promotion](#1176-constant-promotion-)
        7. [Temporary lifetime extension](#1177-temporary-lifetime-extension-)
        8. [Extending based on patterns](#1178-extending-based-on-patterns-)
        9. [Extending based on expressions](#1179-extending-based-on-expressions-)
        10. [Not running destructors](#11710-not-running-destructors-)
12. [Generics](#12-generics-)
13. [Macros](#13-macros-)
14. [Operators](#14-operators-)
15. [Precedence](#15-precedence-)
16. [Attributes](#16-attributes-)
17. [Implicit context](#17-implicit-context-)
18. [Effect system](#18-effect-system-)
19. [Contracts](#19-contracts-)
20. [ABI](#20-abi-)

# 1. Introduction [↵](#tables-of-contents)

This file contains the current langauge design for the Xenon language, including rationals for design decisions.
This is not a full specification, as the specification will be derived from this design once the langauge reaches v1.0 of the langague.

This documentation is an overview of the Xenon language in it's current state, and is written for the development of the langauge and those who are interested in the langauge.

## This document is provisional

The contents of this document is still provisional and is subject to change at any time.
This means that the syntax, languages uresl, core and standard libary, compiler infrastructure, package manager/build tool, and other aspect of the designes that have not bee decided on yet.
This therefore will contain gaps for parts that have not been decided on yet.

## Notation

The notation used in the design documents can be found within the [Notation section of the combined grammar](grammar.md#notation)

# 2. Source code representation [↵](#tables-of-contents)

This section contains info about the source code representation in the file, and by extension on disk

## 2.1. Input format [↵](#tables-of-contents)

Each source file is interpreted as a sequence of Unicode codepoints encoded within the utf-8 format.
If a file does not contain a valid utf-8 sequence, this will result in an error.

Xenon source files use the extension `.xn`

## 2.2. Byte order markers [↵](#tables-of-contents)

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

## 2.3. Newline sequences [↵](#tables-of-contents)

```
<new-line> := [ "\r" ] "\n"
```

A newline within the file is represented using a newline sequence `\n` (U+000A).
This may also be preceded by a carriage return '\r' (U+000D), any other occurance is ignored by the compiler.
Any other occurance of a carriage returned in the file will be ignored.

Carriage returns will be preserved in any reconstructed file.

## 2.4. Shebang [↵](#tables-of-contents)

```
<shebang> := '#!' ? any valid character ? <newline>
```

A file may contain a shebang in the first line in a file, but will be ignored by the compiler.
When a shebang is encountered on the first line, it will be skipped until the first newline sequence is encountered.

_todo: depending on the attribute syntax, we might have to change this definition slightly to include the used of `#!`_

# 3. Lexical structure [↵](#tables-of-contents)

This section contains info about the lexical struture of a code file, which will be interpreted as tokens.

## 3.1. Whitespace [↵](#tables-of-contents)

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

## 3.2. Keywords [↵](#tables-of-contents)

Keywords represent names within the code that have a special meaning in the language, such as declaring a function.

There are 3 types of keywords:
- strong
- reserved
- weak

### Strong keywords

A strong keyword is a keyword that always has a meaning, regardless of where in the code it is located, and can therefore not be used for anything else
A list of strong keywords can be found below (in a close to alphabetic order):
```
as
b8
b16
b32
b64
bitfield
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
is
isize
mut
self
static
str
str7
str8
str16
str32
struct
true
ref
u8
u16
u32
u64
u128
union
unsafe
use
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
A list of weak keywords can be found below (in a close to alphabetic order):
```
distinct
flag
opaque
override
property
record
sealed
super
tls
```

## 3.3. Comments [↵](#tables-of-contents)

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

# 4. Package structure [↵](#tables-of-contents)

Additional info can be found in [the package design](packages.md).

## 4.1. Packages [↵](#tables-of-contents)

A package represents the upper level of the hierarchy of artifacts and the main unit of distribution.

Packages themselves are not the direct result of compilation, but play an integral part if code organization, including how packages are 'imported'.
A package can contain any number of artifacts, allowing allow related code to be shared as a single unit,
meaning that if a project is split up in modularized components, they can still be easilty distributed, without having to result to sub-naming.

## 4.2. Artifacts [↵](#tables-of-contents)

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

## 4.3. Modules [↵](#tables-of-contents)

A module generally represents a single file or inlined module definition (if a file is not direclty included within another file).
Each module is allowed to have multiple sub-modules.

Each artifact has it's own main module, which by default uses the following files:
- binaries: `main.xn`
- static and dynamic libraries: `lib.xn`

# 5. Names and paths [↵](#tables-of-contents)

Names, identifiers, and paths are used to refer to things like:
- types
- items
- generic paramters
- variable bindings
- loop labels
- fields
- attributes
- etc.

## 5.1 Names [↵](#tables-of-contents)

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
_TODO: Need to look into this a bit more_

Normal names can be used everywhere a name can be uses, including in locations extended names are avaiable.
Extended names on the other hand have much more limited scope of where they can be used, mainly in locations they cannot cause confusion.

## 5.2. Identifiers [↵](#tables-of-contents)

```
<identifier> := <name> [ <generic-args> ]
```

An identifier is a sub-segment of a path, which consists out of a name and optional generic arguments.

Identifiers refer to a single element in a path which can be uniquely identified by it's name and generics.

## 5.3. Paths [↵](#tables-of-contents)

A path is a sequence of one or more identifiers, logically separated by a `.`.
If a path consists out of only one segment, refers to either an item or variable in the local scope.
If a path has multiple paramters, it refers to an item.

Two examples are:
```
x;
x.y.z;
```

There are multiple types of paths:

### 5.3.1. Simple paths [↵](#tables-of-contents)

```
<simple-path> := [ '.' ] <simple-path-segment> { '.' <simple-path-segment> }*
<simple-path-segment> := <name> | 'super' | 'self'
```

Simple path are used for visitility, attributes, macros and use items.

### 5.3.2. Paths in expression [↵](#tables-of-contents)

```
<path-in-expr> := [ '.' ] <path-expr-segment> { '.' <path-expr-segment> }*
<path-expr-segment> := <path-ident> [ <generic-path-args> ]
```

Paths in experessions allow for paths with generic arguments specified.
They are used in various places in expressions and patterns.

### 5.3.3. Qualified paths [↵](#tables-of-contents)
_TODO_

# 6. Literals [↵](#tables-of-contents)

```
<literal> := <numeric-literals>
           | <boolean-literals>
           | <character-literal>
           | <string-literals>
```

A literal is a compile time constant representing a given value as either an integer or floating-point type.

> _Note_: Literals are tokens and not symbols, and will therefore be processed in the lexer stage_

## 6.1. Numeric literals [↵](#tables-of-contents)

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

### 6.1.1. Decimal literal [↵](#tables-of-contents)

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

### 6.1.2. Binary literals [↵](#tables-of-contents)

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

### 6.1.3. Octal literals [↵](#tables-of-contents)

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

### 6.1.4. Hexadecimal integer literals [↵](#tables-of-contents)

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

### 6.1.5. Hexadecimal floating point literals [↵](#tables-of-contents)

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

## 6.2. Boolean literals [↵](#tables-of-contents)
```
<bool-literal> := 'true' | 'false'
```

A boolean literal represents either a `true` of a `false` value

## 6.3. Character literals [↵](#tables-of-contents)

A character literal defines a character, represented by it's unicode codepoints.

```
<character-literal> := "'" ( ? any unicode codepoint, except for \ and ' ? | <escape-code> ) "'"
```

### 6.3.1. Escape codes [↵](#tables-of-contents)

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

## 6.4. String literals [↵](#tables-of-contents)

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

### 6.4.1. String continuation sequence [↵](#tables-of-contents)

```
<string-continuation-sequence> := '\' <newline> ? any whitespace character ? ? any non-whitespace character
```

A string continuation sequence allows a regular line to be split up between lines.

Whenever a `\` is encoutered, directly followed by a new line sequence, the string will pause parsing any character until it finds the next non-whitespace character,
it will then continue to parse the string.

## 6.5. Literal operators [↵](#tables-of-contents)

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


# 7. Items [↵](#tables-of-contents)

```
<item> := <module-item>
        | <use-item>
        | <fn-item>
        | <type-alias-item>
        | <struct-item>
        | <union-item>
        | <enum-item>
        | <bitfield-item>
        | <const-item>
        | <static-item>
        | <trait-item>
        | <impl-item>
        | <external-block>
```

An item is a component of a package.
Items are organized within a package, inside of modules.
Every artifact within the package has a single "outermost" anonymous module; all further items within the package have paths within the package hierarchy.

Items are entirely determined at compile time, generally remain fixed during execution, and may reside in read-only memory.

Some items form an implicit scope for the declarations of sub-items.
In other words, within a function of module, declarations of items can (in many cases) be mixed with the statements, control blocks, and similar comonents that otherwhise compose the item body.
If the item was declared outside of the outer scope - i still a static item - except that the item's path name within the module namespace is qualified by the name of the enclosing item, or is private to the enclosing item (in the case of functions).

## 7.1. Module item [↵](#tables-of-contents)

```
<module-item> := { <attrib>* } [<vis>] "mod" <ext-name> ';'
               | { <attrib>* } [<vis>] "mod" <ext-name> '{' { <module-attribute> }* { <item> }* '}'
```

A module is a container of zero or more items.

A module item introduses a a new named module into the tree of modules making up the current artifact.
Modules can be nested arbitrarily.

_TODO: exmaple_

Modules adn types share the same namespace.
Declaring a named type with the same name as a module in a scope is forbidden; that is, any item cannot shadow the name of a module in the scope and vice versa.
Items brought into scope with a `use` also have this restriction.

Modules are generally split up in 2 kinds.

### 7.1.1. Inline modules [↵](#tables-of-contents)

Inline modules are declared directly within another module and allows manual nesting within the file.

Inline modules are allowed to declare any file modules within them, but the path is interpreted differently, see below for more info.
An inline module can also have a single segment path defined to name the sub-folder they would map to if they would have been file modules.

When using a nested module in a file:
```
mod bar {
    mod baz;
}
```
The following set of nested modules will be produces and there  corresponding filesystem structure when using the default module structure:

Module path     | Filesystem path          | File content
----------------|--------------------------|----------------
`:`             | `lib.xn` or `main.xn`    | `mod foo;`
`:.foo`         | `foo/mod.xn` or `foo.xn` | see code above
`:.foo.bar`     | `foo/mod.xn` or `foo.xn` | see code above
`:.foo.bar.baz` | `foo/bar/baz.xn`         | `

### 7.1.2. File modules [↵](#tables-of-contents)

A file modulre refers to code located within an extrenal file.
If no explicit path is defined for the module, the path to the file will mirror the logical module path.
All ancestor module path elements are represented by a path of nested directories within the artifact's source module.

The default naming of sub-modules is done in 2 ways:
- As a file located in the same directory as this folder, with the name of the module (only applicable for declarations in root-files).
- As a `mod.xn` file within a sub-directory with the name of the module.

The following is an example of a set of nested modules and there corresponding filesystem structure when using the default module structure:

Module path | Filesystem path                  | File content
------------|----------------------------------|--------------
`:`         | `lib.xn` or `main.xn`            | `mod foo;`
`:.foo`     | `foo/mod.xn` or `foo.xn`         | `mod bar;`
`:.foo.bar` | `foo/bar/mod.xn` or `boo/bar.xn` |

### 7.1.3. Path attribute [↵](#tables-of-contents)

The directory and files used for loading a file module can be influenced using the `path` attribute.

If a `path` attribute is applied on a module that is not inside an inline module, the path is relative to the directory the source file is located in.

For example, with the following code in a file:
_TODO: Is this attribute notation correct?_
```
#[path = "foo.xn"]
mod c;
```
will produce the following paths:

Module path    | `c`'s file location | `c`'s module path
---------------|---------------------|-------------------
`src/a/b.xn`   | `src/a/b/foo.xn`    | `:.a.b.c`
`src/a/mod.xn` | `src/a/foo.xn`      | `:.a.c`

For a `path` attribute inside an inline module, the relative location of the file path depends on the kind of source file the `path` attributre is located in.
If in a root module (such as `main.xn` or `lib.xn`) or in a `mod.xn` file,  the path is relative to the directory it would have been in, if it was only using file modules, meaning that it will interpret all inline module modules as a directories.
Otherwise, it is almost the same, with the exception that the path starts with the name of the current module.

For example, for the following code:
_TODO: Is this attribute notation correct?_
```
mod inline {
    #[path = "other.xn"]
    mod inner;
}
```
The path will be the following depending what file it is in:

Module path    | `inner`'s file location   | `inner`'s module path
---------------|---------------------------|-------------------
`src/a/b.xn`   | `src/a/b/inline/other.xn` | `:.a.b.inline.inner`
`src/a/mod.xn` | `src/a/inline/other.xn`   | `:.a.inline.inner`

## 7.2. Use declarations [↵](#tables-of-contents)

```
<use-item> := `use` <use-root> [ '.' <use-tree> ] ';'
            | `use` <use-tree> ';'
<use-root> := [ <name> ] ':' [ <name> ]
<use-tree> := <simple-path> '.' '*'
            | <simple-path> '.` '{' <use-tree> { ',' <use-tree> }* [ ',' ] '}'
            | <simple-path> [ "as" ( <ext-name> ) ]
```

A `use` declaration creates a local binding associated to a module path.
These are used for 2 reasons:
- Introduce a libary's root module into the scope
- Shorten the path required to refer to a module.

These declations may appear in modules and blocks.

To access any path from outside the current scope, each `use` declaration must start by indicating the package and library modules come from.
This is called the root name as is shown as `package:library`, these do not explicitly be written down in the following usecases:
- the package can be left out if the path refers to the current package
- the library can be left out in 2 cases:
    - If there is no explicit package (i.e. the current package), it will refer to the current package
    - If tehre is an explicit package, it will refer to the library within that package with the same name

An example of this can be seen in the below table for the following project structure

For example, with the package and library structure:
```
A (package)
- Cur (lib)
- A (lib)
- C (lib)
B (package)
- B (lib)
- D (lib)
```

And with the current library being `Cur`, the path will point to the following packages and libraries

Use root | Package | Library
---------|---------|---------
`:`      | `A`     | `Cur`
`:C`     | `A`     | `C`
`A:`     | `A`     | `A`
`B:`     | `B`     | `B`
`B:D`    | `B`     | `D`

The `use` root can be omitted for any value relative to the current module, including at most 1 level up using the `super` keyword.

Use declarations support a number of convenient shortcuts:
- Simultaneously bind a list of paths with a common prefix, using a braced represetnation, i.e. `:.a.b.{c, d.e, f.g.h}`
- Simultaneously bind a list of paths with a common prefix, and their parent module, e.g. `:.a.b.{self, c, d.e}`
- Rebind a module or item to a local name, e.g. `:.a.b.c as d`
- Bind all paths with a common prefix, e.g. `:.a.b.*`

### 7.2.1. Use visibility [↵](#tables-of-contents)

Like other items, `use` declarations are private to the containing module by default.
But it can also have its visibility declared, while for most items, this is explained in the [attribute]() section, visibility attributes work slightly differently on `use` declarations.
`use` declaration can be used to re-export symbols to a different target definition with a diffferent visibility and/or name.
For example, a symbol with a more restricted visibility like 'private' in one module to a `pub` symbol in another module.
If the resulting sequence of re-exports form a cycle or cannot be resolved, this will be a compile error.

An example of redirection:
```
mod quux {
    use :.foo.{bar, baz};
    pub mod foo {
        pub fn bar() {}
        pub fn baz() {}
    }
}

fn main {
    quux.bar();
    quux.baz();
}
```
### 7.2.2. Underscore imports [↵](#tables-of-contents)

Items can be imported without binding to a name by using an underscore with the form `use path as _`.
This is particularly useful to import an trait so that its methods may be used without impoiting the trait symbol, for example if the trait's symbol may conflict with another symbol.

## 7.3 Function [↵](#tables-of-contents)

```
<fn-item> := { <attribute> }* [ <vis> ] [ 'const ] [ 'unsafe' ] [ 'extern' <abi> ] 'fn' <name> [ <generic-params> ] '(' [ <fn-params> ] ')' [ <effects> ] [ '->' <fn-return> ] [ <where-clause> ] { <contract> }* <fn-body>
<fn-body> := <block-with-return>
           | <block-no-return>
           | ';'
```

A function associates a block of code with a name with a set of generics, parameters, effects, a return type, only the name is required.

When refered to, a function yields a first-class value of a the corresponding zero-sized [function item type](#11114-function-types), which when called evaluates to a drectir call to a function.

A function can be declared `unsafe`, requiring it to be called from an unsafe context, but allowing any `unsafe` code to be called from within the function.

### 7.3.1. Parameters [↵](#tables-of-contents)

```
<fn-params> := [ <receiver-param> ] [  <fn-param> { ',' <opt-fn-param> }* ] [  <fn-param> { ',' <opt-fn-param> }* ]  [ ',' [ <variadic-param> ] ]


<receiver-param> := <simple-receiver> | <typed-receiver>
<simple-receiver> := [ '&' [ 'mut' ] ] 'self'
<typed-receiver> := 'self' ':' <type>

<fn-param> := { <attribute> }* <fn-param-name> { ',' <fn-param-name> }* ':' <type>
<opt-fn-param> := { <attribute> }* <name> <pattern-top-no-alt> ':' <type> '=' <expr>
<fn-param-name> := [ <name> ] <name>
                 | <name> <pattern-top-no-alt>

<variadic-param> := <label> '...' ':' <type>
```

Function parameters consists out of an label, a pattern, and a type.
A label can be optional if the pattern is an identifier pattern.

The first parameter can be a special receiver parameter, which indicates that this function is a method, and can therefore only be declared within an implementation block.
The receiver has an implicit '_' label, and can be any of the following types:
- `Self`
- `&Self`
- `&mut Self`
- any `T` that implements `Deref<Target = Self>`. _TODO: might need an additional `Dispatch` bound_

Any other parameter is a normal parameters.
If an explicit label is provided, it can be either
- a name, this is the label any argument needs to be 'bound' to, and needs to be provided for this parameter when calling the function
- an '_', this implies an unnamed parameter and has no label needs to be provided for this parameter when calling the function.

If no explicit label is provided, this will default to a label with the same name as a paramters, so `foo: i32` will become `foo foo: i32`.

Parameters can be provided default parameters after `=`, and are also known as default parameters.
The default value needs to be an expression that can be evaluated at compile time.
All default parameters are required to have a label, as these may appear in any order in a function call.

If a function has no default parameters, or only has labeled default parameters, they may be followed by a variadic parameter.
This is a special parameter that allows any number of element of that type to be provided.
This will generate a generic paramter pack with a type constraint to the type given.

Any 2 parameters may not have the same label.

### 7.3.2. Returns [↵](#tables-of-contents)

```
<fn-return> := <fn-return-single> | <fn-return-multi>
<fn-return-single> := [ 'name' ':' ] <type>
<fn-multi-return> := '(' fn-return-single { ',' <fn-return-single> }* ')'
```

Functions can return either a normal type, or a set of labeled returns.
If no label is specified, the function is just returned with `return` statement, or with the result of the last expression (if it is not ended by a semicolon).

When named returns are used, the function can only be returned using an empty `return` or an expression returning a unit type (if not ended by a semicolon).

Named return are required to be assigned a value, using them inside of a function is the same as any other mutable variable.

### 7.3.3. Pseudo-function overloading [↵](#tables-of-contents)

Xenon supports a way of overloading functions, but instead of it being based on any type, it is based on the labels of the receiver and non-default paramters.

Function overlap gets checked for each set of functions in the following steps:
1. Generate a label signature for each function, in the styled of `fn_name(arg0:_:arg1?def1?def2?...)`
2. If the name of the function does not match that of the other function, we have no collision, otherwise proceed to the next step.
3. Check the first set of matching required parameters, meaning if function `a` has `N` parameters, and `b` has `M` paramters, compare the first `min(N, M)` parameters:
   If a pair of corresponding parameters do not match, we have no collisions, otherwise proceed to the next step.
4. If both functions have the same number of non-default paramters, proceed to the next step, otherwise take the `N` parameters that are left from the additional paramters one of the functions has, and for each paramters, do the following:
   1. Walk through the other functions default arguments
      - If a label matches that of the optional argument and we are at the last paramter of the function (i.e. no other params left), we got a collision
      - If we only match the labels, and go the next iteration.
      - Otherwise go to the next sub-step
   2. If we hit the end of the other functions optional parameters, we have no collision, otherwise break otherwise go to the next iteration
5. If any default parameters are left over, there is a collision, otherwise go the the next step
6. If both functions have variadic arguments, we have a collision, otherwise we don't have one and the functions 

#### Resolve examples

Below are example of where something can happen in a collision check.

2. Different names: **_no collision_**
```
fn foo() { ... }
fn bar() { ... }
```

3. Same number of paramters with same labels: **_collision_**
```
fn foo() { ... }
fn foo() { ... }
```
or
```
fn foo(a: i32) { ... }
fn foo(a: f64) { ... }
```

4. Overlap between func with required and func with default values: **_collision_**
```
fn foo(a: i32, b: i32)
fn foo(a: i32, b: i32 = 0)
```
or
```
fn foo(a: i32,             c: i32)
fn foo(a: f64, b: i32 = 0, c: i32 = 1)
```

Overlap between func with required and func with default values, but with a non-default left over: **_no collision_**
```
fn foo(a: i32, b: i32, d: i32)
fn foo(a: i32, b: i32 = 0)
```
or
```
fn foo(a: i32,             c: i32, d: i32)
fn foo(a: f64, b: i32 = 0, c: i32 = 1)
```

5. Any left over defaults: **_collision_**

```
fn foo()
fn foo(a: i32 = 0)
```
or
```
fn foo(a: i32, c: i32 = 0)
fn foo(a: i32, c: i32 = 1)
```

6. Both have variadics: **_collision_**
```
fn foo(a: i32, b: i32...)
fn foo(a: i32, c: f64...)
```
or 
```
fn foo(a: i32,             c: i32...)
fn foo(a: i32, b: i32 = 1, c: i32...)
```

### 7.3.4. Const function [↵](#tables-of-contents)

A const function allows the function to be called at compile-time.

### 7.3.5. Trait function [↵](#tables-of-contents)

```
<assoc-fn> := <fn-item> | <assoc-override-fn>
<assoc-override-fn> := [ 'override' ] 'fn' <name> [ <generic-params> ] '(' <fn-params> ')' [ <effects> ] [ '->' <fn-return> ] <fn-body>
<assoc-fn-body> := ';' | <fn-body>
```
An associated function is allowed to leave out a body, if this is done, the function must be implemented (either down the trait bounded by the current trait), or by the type that implements the trait.
If an associated function has it's body defined, this definition will act as the default definition of the function.

Any function that in the base interface may have its default implementation overwritten by the current interface, for this the weak keyword `override` can be used.
As this might cause some issues between between common 'child' interfaces, for more info about this conflict, please check [here](#7122-function-override-resolution).

> _Note_: Overridden functions do not define a function with the same name for the current trait, but instead exclusively overwrites a default implementation.


#### Trait function override resolution

As traits can override the default implementation of a supertrait without inserting a new function into the current trait, there is a possiblity for these overrided to incur the so-called "diamond problem".
Imagine a trait `A`, defining a function foo (with or without a default implementation).
2 traits, `B` and `C` both have `A` as a supertrait and overide the default implementation.
Finally a trait `D` would now be declared, having both `B` and `C` as supertrait.

The trait hierachy is now:
```
 A
/ \
B C
\ /
 D
```

Since both `B` and `C` override the functions default implementation, the compiler cannot determine which one to use for `D`, therefore `D` needs to explicitly define the default implementation for hte given function, or a compile error will occur.

This example is illustrated in the following code:
```
trait A {
    fn foo() -> i32;
}

trait B: A { 
    // Create/override the default implementation for A.foo
    override fn foo() -> i32 { 1 }
}

trait C: A {
    // Create/override the default implementation for A.foo
    override fn foo() -> i32 { 2 }
}

trait D: B, C {
    // Override 'foo' to resolve the conflicting default implementations from B and C
    // Removing this override will result in a compile error
    override fn foo() -> i32 { C.foo() }
}
```

This is somewhat similar to the resolution for conflicting generic specializations.


### 7.3.6. External function qualifier [↵](#tables-of-contents)

The extern qualifier on functions allows the programmer to specify the API without requiring them to put the function inside of an external block.

If an extern function does not define a body, then this is a binding to a function declared in an external library.
If it has a body, then this is a function that gets exported so it can be used from external code.

## 7.4 Type aliases [↵](#tables-of-contents)

```
<type-alias-item> := { <attribute*> } [ <vis> ] ( <alias-type> | <new-type> | <opaque-type> )
<alias-type> := 'type' <name> [ <generic-params> ] '=' <type> ';'
<new-type> := 'distinct' 'type' <name> [ <generic-params> ] '=' <type> ';'
<opaque-type> := 'type' <name> '=' 'opaque' [ '[' <expr> ']' ]
```

A type alias defines a new name for an existing type, an allows for partial specialization of the generic parameters.
The 'alias type' is the new type being created, the 'aliasee' is the type that is being aliased, i.e. `type alias_type = aliasee;`.

If a generic type is passed to the aliasee, the generic in the alias type itself will gain the same bounds as those for the aliasee.

Type aliases are declared using the `type` keyword.

There are also 2 'variants' of the type alias.

### 7.4.1. Distinct types [↵](#tables-of-contents)

A distinct type is a special type alias, that does not only gives a different name, etc to a type, but splits it of into a separate type, these are also known as 'newtypes.'

Distinct types take over all fields and functionality of the aliasee, but can also implement additional functionality independently of the type.

> _Note_: a limitation of this is that a disctinct type cannot acces fields that are private to the aliasee.

### 7.4.2. Opaque types [↵](#tables-of-contents)

An opaque type represents a type with an unknown layout, which can either be a DST, or it can have a given size.
If a size is set, the size expression must be able to be evaluated at compile time.

Internally, an opaque type is represented as:
- When sized, it is represented by `[N]T`, where `N` is the size of the opaque type
- When unsized, it is represented by `dyn ?Sized`.

## 7.5. Structs [↵](#tables-of-contents)

```
<struct-item> := { <attribute> }* [ <vis> ] ( <struct-stuct> | <tuple-struct> )
```

A struct is a composite type that consists out of a number of types, called 'fields'.

Fields within a structure can be defined as either being mutable, or not (which is the default).
Non-mutable field can only be set when the struct is fully assigned, i.e. the individual field cannot be modified.
If the entire struct is marked as mutable, all fields within it will be mutable, no matter if they are individual declared as.

Fields can also be assigned default value, which need to be able to be evaluated at compile time, but allow them to be left out when constructing a value of this type.

> _Note_: Default values for fields should not be confused with the value of fields if the `Default` trait is implemented.
> Field default values are used to allow omitted when constructing a new struct, not to retrieve a default value for the entire struct,
> this means that `Default::default()` may return a different value that a field's individual default value, as it is allowed to decide these values at runtime.

There are 3 kinds of structs:

### 7.5.1. Regular structure [↵](#tables-of-contents)

```
<nominal-struct> := [ 'mut' ] [ 'record' ] 'struct' <name> [ <generic-params> ] [ <where-clause> ] '{' [ <struct-fields> ] '}'
<struct-fields> := <struct-field> { ',' <struct-field> } ','
<struct-field> := <struct-member> | <struct-use>
<struct-member> := [ <vis> ] [ 'mut' ] <name> { ',' <name> }* ':' <type>
                 | [ <vis> ] [ 'mut' ] <name> ':' <type> [ '=' <expr> ]
<struct-usze> := [ <vis> ] 'use' <type>
```

A regular structure exists out of a collection of named fields.
A field can be left out, but have its space reserved for future use, by giving it the name of `_` which will the field for all other purposes.

Each field defintion may contain multiple names, this will result in a field to be created for each, with the type defined before each field.
If only a single name is defined, a field may also have a default value assigned to it (see note above).

#### Use fields

Sometimes it may be usefull to add the contents from another structure directly within the body of the current struct.
This can be done using the special case of the `use` keywords, as inside of a structure, instead of importing another module, it means that the body of the strucuture after the will be placed within the current stuct.
There are some limitiations when it comes caused by the visibility of fields, for a struct to be included within another struct:
- if the `use` comes from another library, all it's member need to be public to include it within the body.
- if the `use` comes from the same library, all fields need to be visible from the current namespace and and the visibility of the `use` may not be greater than that of any of those fields.

These fields will then all also be given the visibility as define before the `use`.

To take over mutable fields, the `use`, it needs to be explicitly marked as `mut`.
Use will also take over default field values

Below is an example of `use`:
```
// Library file

struct Quux {
    pub     qq: i32 = 1,
    pub mut qr: f64,
}

// File: main.xn

use :lib; // Include lib

struct Foo {
    pub(mod) a: i32,
    pub mut  b: i32
}

struct Bar {
    c: i32
}

struct Baz {
    pub(mod) use Foo, //< 'b' will not not be `pub`, but `pub(mod)` instead
    // use Bar,       //< uncommenting this like will result in a compile error, as 'c' is not visible from 'Baz'
    mut use lib.Quux, //< Mutably include `lib.Quux`
}

// When compiler, Baz will result in
struct ...Baz {
    pub(mod) a:  i32,
    pub(mod) b:  i32,
             qq: i32 = 1,
    mut      qr: f64,
}
```

#### Record struct

A variation of a struct is a record struct, unlike a normal struct, this is not a nominal type, but instead a structural type.
The distinction can be made by the weak keyword `record` coming before the `struct` keyword.

They are generally similar to inline records, but allow visibility and default value for fields to be specified.

### 7.5.2. Tuple structure [↵](#tables-of-contents)

```
<tuple-struct> := [ 'mut' ] [ 'record' ] 'struct' <name> [ <generic-params> ] [ <where-clause> ] '(' [ <tuple-struct-fields> ] [ <tuple-struct-tail-fields> ] [ ',' ] ')'
<tuple-struct-fields> := [ <tuple-struct-field> { ',' <tuple-struct-field> }* ]  [ ',' ]

<tuple-struct-tail-field> := [ <tuple-struct-tail-field> { ',' <tuple-struct-tail-field> }* ]

<tuple-struct-field> := [ <vis> ] [ 'mut' ] <type>
<tuple-struct-tail-field> := [ <vis> ] [ 'mut' ] <type> [ '=' <expr> ]
```

A tuple struct, also called a named tuple, represents a list of types that form their own nominal type.

Tuple struct fields may contain default values, but these need to come at the end of the tuple.
If a field without a default value follows one with a default value, it is a compile error

#### Record tuple struct

A variation of a tuple struct is a record tuple struct, unlike a tuple struct, this is not a nominal type, but instead a structural type.
The distinction can be made by the weak keyword `record` coming before the `struct` keyword.

### 7.5.3. Unit structure [↵](#tables-of-contents)

```
<unit-struct> := { <attribute> }* [ <vis> ] 'struct' <name> ';'
```

A unit structure is a special structure containing no fields, and which can be be initialized by using the structure as the initialization expression.
Unit stuctures can be seen as distinct type aliases of the unit type, but with the ergonomics of being an extual individual structure.

## 7.6. Union [↵](#tables-of-contents)

```
<union-item> := { <attribute> }* [ 'vis' ] 'union' <name> [ <generic-params> ] [ <where-clause> ] '{' <union-fields> '}'
<union-fields> := <union-field> { ',' <union-field> }* [ ',' ]
<union-field> := [ <vis> ] <name> ':' <type>
```

A union is a struct-like type, but instead of all fields being available at all times, a union's main characteristic is that all field share a common storage.
As a result, a write to 1 field can overwrite other fields.

Union fields are restricted to the following subset of types:
- `Copy` types (including records)
- References ( `&T` and `&mut T` for an arbitrary `T` )
- `ManuallyDrop<T>` (for an arbitrary `T`)
- Tuples and arrays containing values allowed by unions

When initializing a union, only 1 field can be set, in this design document, this is known as the 'active field'.

> _Note_: unions have no notion of an 'active field', i.e. it has no special meaning, but is only used to in the design document to indicate the currently assigned field

Any member can be accessed at any time, it directly reads the underlying memory as the type of the field being accessed.
Which means that any field which has an incompattible layout with the active field **may** therefore contain invalid data.
The programmer should be certain that the field contains valid before using it, failing to do so in undefined behavior.

Because of what's mentioned above, this means that all reads field in the union are `unsafe`.
Unlike reads, writes are always safe, as the user is just overwriting arbitrary data, so cannot be undefined behavior.
As union field will never be dropped.

### 7.6.1. Union field offsets [↵](#tables-of-contents)

By default, all fields are guaranteed to be at an offset of `0`.
Sometimes it might be useful to have certain fields overlap at a non-zero offset, this can be done using the `union_offset` attribute.

The `union_offset` macro defines an offset in bytes, which will then be the resulting offset of the field.

### 7.6.2. Pattern matching on unions [↵](#tables-of-contents)

Another way to access union fields is to use pattern matching.
Pattern mathinc on union field uses the same syntax as those for structs, except that the pattern must specificy exactly 1 field.
Since reading from a union is unsafe, the entire match expression must be in an unsafe block.

### 7.6.3. References to union fields [↵](#tables-of-contents)

Since unions fields share a common storage, gaining writing access to one field of the union can give write access to all its remaining fields.
For this reason, if any field is borrowed immutably, no other field can be borrowed mutably in the same lifetime.

## 7.7. Enum [↵](#tables-of-contents)

```
<enum-item> { <attribute> }* [ <vis> ] ( <adt-enum> | <flag-enum> )
```

An enum, or enumeration, is a type that can be used for one of the following use-cases:
- A field-less enum used to correlate names to a given set of possible values (these are a variant of ADT enums)
- An ADT (Algebreic Data Type) enum, allowing fields to be associated with a specific variant (also known as tagged unions)
- A set of flags

The visibility of the enum is shared by all variants and their fields

### 7.7.1. ADT enum [↵](#tables-of-contents)

```
<adt-enum> { <attribute> }* [ <vis> ] [ 'flag' ] 'enum' [ <generic-params> ] [ <where-clause> ] '{' <enum-variants> '}'
<enum-variants> := <enum-variant> { ',' <enum-variant> } [ ',' ]
<enum-variant> := <name> [ <variant-body> ] [ '=' <expr> ]
<variant-body> := <struct-variant-body> | <tuple-variant-body>
<struct-variant-body> := '{' <struct-fields> '}'
<tuple-variant-body> := '(' <type> { ',' <type> }* [ ',' ] ')'
```

Each ADT enum constists out of at minimum a discriminant, but may inn addition also contain a set of fields that are associated with each variant.

Variants with fields can be presented as either a regular structure or a tuple structure, and is called the variant's 'payload'.
This payload is effectively a struct or enum struct, where it's body is defined after the name of the variant.
For more info about the payloads, see the [Struct item](#75-structs-).

> _Note_: field may not have their own visibility defined

#### Discriminant

Each enum variant is represented using its discriminant, this is an integer value that encodes the current variant that is stored inside of the enum.

When using the default representation, while the discriminant will be interpreted as an `isize`, the compiler is allowed to change this to a smaller type that can still fit all discriminants.
An explicit type can be chosen using a primitive representation.

Discriminant values can be explicitly be set by following the variant with a `=` followed by an expression returning an integer.
The discriminant needs to be a value that can be evaluated at compile time.
The expression may also not reference any other varaint in the enum.

When no explicit discriminant is given, this will automatically be set to one higher than the discriminant of the pervious variant.
If the discriminant for the first value is not set, this will become 0, and if at any point, the discriminant would overflow, this will result in a compile error.

No 2 variants may have the same descriminant, meaning that if an implicit discriminant value appears after the maximum value of the current descriminant type, that this will result in an error.

The discriminant value of any enum can be extracted using `discriminant`
_TODO: add full path to function_

If an enum has a known discriminant type, it is allowed to cast a pointer to the enum to an pointer with the descrimant type.

#### Field-less enum

A field-less enum is a variant of an ADT enum that contains no payload and it therefore just its discriminant internally.
This allows field-less enums to be cast to their underlying integer type.

### 7.7.2. Record enums [↵](#tables-of-contents)

A record enum is a variant of a normal enum, but instead of each variant with fields holding a nominal type, each variants holds a structural type.

### 7.7.3. Flag enum [↵](#tables-of-contents)

```
<flag-enum> := 'flag' 'enum' <name> '{' [ <flag-enum-variants> ] '}'
<flag-enum-variants> := <flag-enum-variant> { ',' <flag-enum-variant> } [ ',' ]
<flag-enum-variant> := <name> [ '=' <expr> ]
```

A flag enum can be thought of as a special variant of a field-less enum, but instead of representing discrete fields, it represents a collection of bitflags.
Each flag enum can contain as many unique flags are as allowed by the primitive type, by default this will be chosen based on the number of variants within the enum.

When no explicit discriminant is given, this will automomatically be set as the next power of 2 that greater than the previous flags.
If the discriminant for hte first value is not , this will become 1, and if at any point the next flag value would overflow, this will result in a compile error.

If no explicit flag is provided with a value of 0, the enum will implicitly add a `.None` flag.

When writing an expression for the enum value, the expression may directly mention another flag, unlike normal enums.

By default, a flag enum will have a set of functions implicitly generated to allow the use of flags in code, these are:
_TODO: list of functions_

## 7.8. Bitfield [↵](#tables-of-contents)

```
<bitfield-item> := { <attribute> }* [ <vis> ] 'bitfield' <name> [ <generic-params> ] [ ':' <expr> ] [ <where-clause> ] '{' <bitfield-fields> '}'
<bitfield-fields> := <bitfield-field> { ',' <bitfield-field> }* [ ',' ]
<bitfield-field> := <name> ':' <type> [ '|' <expr> ]
```

A bitfield is a type similar to a record struct, but which is allowed to contain values that can be represented with non-byte aligned type.

A field in a bitfield may is defined as a signle named field would be in struct, and optionally followed by an additonal `|` and an expression giving the number of bits the value should take in.

If the number of bits is given explicitly, the expressions must represent a value that can be evaluated at compile time.

If no explicit number of bits have been given, the type will automatically take in only as many bits as required, this is done in the following ways:
- If the type is a primitive type, it used the number of bits needed as defined in their layout.
- If the type is a field-less enum or flags enum, it will only take in as many bits that are needed to be able to fit the full range of possible discriminants.
- If the type has a bit-size defined using the `bit_size` attribute, it will take this value as the bit-size.

The size of a bitfield may be explicitly defined in an expression after the bitfield's name.
If no explicit size is defined, it will take in the minimum number of bits needed to store all field in the bitset.

Access to bitfield elements are an example of propties being used.

## 7.9. Const item [↵](#tables-of-contents)

```
<const-item> := { <attribute> }* [ <vis> ] 'const' <name> [ ':' <type> ] '=' <expr> ';'
```

A constant item is a named constant value which is not associated with a specific memory location, i.e. the value is known at compile time.
Contants are essentially inlined when they are used, meaning that they are copied directly into the relevant context when used.
This includes constants from other libraries and non-`Copy` items.

When a reference is taken to a constant value from different locations, they are not neccesarily guarenteed to point to the same memory location.

Constants are generally explicitly types, unless a certain sub-set expressions are used, these are:
- literal expression with a literal operator
- _TODO: others_

Constants live throught the entirety of the program and any reference to them is always valid.

Constants may be of types that have a destructor, and will be dropped when the copy of the value that they are assigned too go out of scope.

When defined inside of an implementation, the const item will be an associated with that type.

### 7.9.1. Associated trait constant [↵](#tables-of-contents)

```
<assoc-trait-const> := 'const' <name> ':' <type> [ '=' <expr> ] ';'
```

An associated trait type declares a signature for an associated constant implementation.
It declares both the name and the type the associated constant should have.

## 7.10. Static item [↵](#tables-of-contents)

```
<static-item> := { <attribute> }* [ <vis> ] [ [ 'mut' ] 'tls' ] 'static' <name> [ ':' <type> ] '=' <expr> ';'
<extern-static-item> := { <attribute> }* [ <vis> ] [ 'mut' ] [ 'tls' ] 'static' <name> [ ':' <type> ] ';'
```

A static item is a named location within the programs static memory.
All references to a static refer to the same memory location.
Static item live for the entirety of the programs life and are never dropped at the end of the program.
Therefore it is not allowed to assign a type which implements `Drop` as the type of a static.

Static items must be initialized using a an expression that can be evaluated at compile time.

Non-mutable static items do not support interior mutability and will be allocated in read-only static memory.

All access to statics is safe, but there are a number of restrictions:
- The type must have a `Sync` trait bound to allow thread-safe access.
- Statics may not be refered to from a constant.

### 7.10.1. Thread local storage [↵](#tables-of-contents)

Static values may also be allocated as thread local storage, using the weak `tls` keyword before the `static` keyword.
Tls statics are unique to the thread they are running on and are not shared with other threads.

Unlike static items, a thread local static can be mutable without requiring [interior mutability](#115-interior-mutability-), as it can only be accessed from the current thread.

### 7.10.2. Statics and generics [↵](#tables-of-contents)

When a static variable is declared within a generic scope, it will result in exactly 1 static item being defined, shared accross all monomorphization of that scope.

### 7.10.3 External statics [↵](#tables-of-contents)

```
<extern-static> := { <attribute> }* [ <vis> ] [ 'extern' <abi> ] ['mut']  <name> ':' <type> ';'
```

Statics can be defined external, or within an external block.
These are declared without an initial value, as this will be retrieved from an external location.

It is always `unsafe` to access an external static, whether or not it is mutable or not, as there is no guarantees that the bit pattern in static memory contains is valid for the type declared, since argitraty (e.g. C) code is in charge of initializing this value.

Unlike normal statics, an external static is allowed to be declared mutable, without needing to rely on interior mutability.
An immutable static must be initialized before any Xenon code is executed.

When declaring a static within a external block, `extern` has to be left out.

## 7.11. Properties [↵](#tables-of-contents)

```
<prop-item> := { <attribute> }* [ <vis> ] 'property' <name> '{' { <prop-get-set> }[1,4] '}'
<prop-get-set> := <prop-get> | <prop-ref-get> <prop-mut-get> | <prop-set>
<prop-get> := 'get' <expr-no-block> ';'
            | 'get' <expr-with-block>
<prop-ref-get> := 'ref' 'get' <expr-no-block> ';'
                | 'ref' 'get' <expr-with-block>
<prop-mut-get> := 'mut' 'get' <expr-no-block> ';'
                | 'mut' 'get' <expr-with-block>
<prop-get> := 'set' <expr-no-block> ';'
            | 'set' <expr-with-block>
```

A property allows a field-like value to be associated with a set of expressions that handle the underlying value changes.

Properties are implemented as having either a _getter_, a _setter_ or both.


The program needs to be aware that using properties may result in slower code, depending on the underlying implementation

Properties can only be declared as associated items.

### 7.11.1. Getters & setters [↵](#tables-of-contents)

The value of a property can be access and/or modified in 4 ways:
- A _value getter_, this return the value stored within the property.
  This requires the property to have a type implementing `Copy`.
  This gives access to `&self` within the expression.
- A _reference getter_, this returns a reference to the value stored within the property.
  This gives access to `&self` within the expression.
- A _mutable getter_, this returns a mutable reference to the value stored within the property.
  This gives access to `&mut self` within the expression.
- A _setter_, this set the value inside of the 
  This gives access to `&mut self` and the implicit argument `value` within the expression.

A property needs to have at minimum one of them.

#### Internal representation

Internally, getters and setters get converted to internal function that get called when a property get's accesed.

```
property value : Type { get { ... } };
// => 
fn get_value(&self) -> Type { ... }

property value : Type { ref get { ... } };
// => 
fn get_ref_value(&self) -> &Type { ... }

property value : Type { mut get { ... } };
// => 
fn get_mut_value(&mut self) -> &mut Type { ... }

property value : Type { set { ... } };
// => 
fn get_value(&self, value: Type) { ... }
```

### 7.11.3. Associated trait properties [↵](#tables-of-contents)

```
<assoc-trait-property> := 'property' <name> ':' <type> '{' { <trait-prop-get-set> }[1,4] '}'
<trait-prop-get-set> := [ 'ref' | 'mut' ] 'get' ';'
                      | 'set' ';'
```

An associated trait type declares a signature for an associated propery implementation.
It declares the name, type and which getter/setter combo needs to exist of the property.

Trait implementation cannot implement additional getters/setters.

## 7.12. Trait [↵](#tables-of-contents)

```
<trait-item> := { <attribute> }* [ <vis> ] [ 'unsafe' ] [ 'sealed' ] 'triat' <name> [ <generic-params> ] [ ':' <trait-bound> ] [ <where-clause> ] '{' { <trait-elem> }* '}'
<trait-elem> := <assoc-func>
              | <assoc-type>
              | <assoc-const>
              | <assoc-property>
<assoc-const> := 'const' <name> ':' <type> ';'
```

A trait represents an abstract interface that type can implement.
This consists out of a set of associated items, there are the following:
- functions
- types
- constants
- properties

All traits define an implicit `Self` type, and refers to "the type that is implementing this trait".
Any generic paramter applied to the trait, are also passed along to the `Self` type

Traits can be implemented via individual implementations.

A trait can be defined as sealed, this means that the trait can only be implemented from the current library and any implementation outside of the current library will result in a compile error.

### 7.12.1. Object safety [↵](#tables-of-contents)

Object safety specifies a set of requireents that the interface needs to adhere to to be allowed to be used in places where an interface object type is allowed.
These are:
- All supertraits must be object safe.
- The trait cannot be sized, i.e. it may not requires `Self is Sized`.
- It must not have associated constants.
- It murst not have associated types using generics.
- All associated functions must either be dispatchable from a trait object or be explicilty non-dispatchable.
    - Dispatchable functions must adhere to:
        - Not have any generic parameters.
        - Method is only allowed to use the `Self` within the receiver.
        - The receiver needs to allow for dynamic disapatch, e.g. `&self` or `&mut Self`, and types implementing `DispatchFromDyn`.
        - Parameters and return type must not be an inferable type, meaning they may not be an impl trait type.
        - May not have a sized bound on the receiver (`Self is Sized` implies this).
    - Explicit non-dispatchable functions require:
        - Have a sized bound on the receiver (`Self is Sized` implies this).

### 7.12.2. Supertraits [↵](#tables-of-contents)

A 'super trait' is a trait that is required to be implemented by a type to implement a specific trait.
Aneywhere a generic or interface object is bounded by a trait, it is also bound by that trait's supertraits.

Supertraits are declared as a trait bound on the `Self` type, and transitively the supertraits of traits declared in those trait bounds.
The can either be defined as a bound directly on the trait, or to `Self` in a where clause.
A trait cannot be its own supertrait, and they cannot form any cyclical supertrait dependence.

### 7.12.3. Unsafe traits [↵](#tables-of-contents)

Traits can be declared as `unsafe`.
Unsafe traits come with additional requirements that the programmer needs to guarantee to follow.

### 7.12.4. Visibility [↵](#tables-of-contents)

Traits define their visiblity directly on the trait itself, and all items within the trait take on that visibility.
Individual associated items cannot declare their own visibility.

## 7.13. Implementation [↵](#tables-of-contents)

```
<impl-item> := <inherent-impl> | <trait-impl>
```

An implementation is an items that associates items with an implementing type.
There are 2 types of implementations:

### 7.13.1. Inherent implementation [↵](#tables-of-contents)

```
<inherent-impl> := { <attribute> }* [ <vis> ] [ 'unsafe' ] 'impl' [ <generic-params> ] <type> [ <where-clause> ] '{' { <assoc-item> }* '}'
```

An inherent implementation is defined without specifying a trait to implements.
The type implementing is called the _implementing type_ and the associated itms are the _associated items_ of the implementing type.

Inherent implementations assoicated hte contained items ot the implementing type.
Inherent implementaions can support associated functions (including methods), properties, and constants.

The path to an assoicated item is the path to the implementing type, followed by the associated item's identifier as the final component of the path.

A type can also have multiple inherent implementations.
An implementation for a type must be defined in the same library as the original type definition.

If a visibility attribute is defined for the block, all items with in the block will default to that visibility and may not be lowered.
If `unsafe` is added to the block, then all functions within the block will be marked as unsafe.

### 7.13.2. Trait implementation [↵](#tables-of-contents)

```
<trait-impl> := { <attribute> }* [ 'unsafe' ] 'impl' [ <generic-params> ] <type> 'as' <path> [ <where-clause> ] '{' { <assoc-item> }* '}'
```

A `trait` implementation is defined like an inherent implementation, but also include the interface to be implemented.

The trait is known as the _implemented trait_, and the implementing type implements the trait.

A trait implementation must define all non-default associated types declared by the implemented trait and it can redefine (i.e. override) an item that has a default implementation.
It is not allowed to define any implementation that is not defined in the implemented trait.

If an implemented trait contains an override for an associated function, but the implementing type has already implemented it by itself, the overriden default will be ignored.

Unsafe traits require the `unsafe` keyword to be added to the implementation.
`trait` implemetnations are not allowed to specify any visibility for items. 

#### Coherence

A trait implemention is coherent when it can be be defined within the current library.

A trait implemention is considered coherent if either the below rules aren't followed, or there are overlapping implementations.

Two trait implementations overlap when there is 2 implementations ca be instantiated for the same type.

The coherence rules require that the implementation `impl<P0..=Pn> T0 as Trait<T1..=Tn>` to adhere to one of the following:
- Trait is a local trait
- At least one type `T0..=Tn` must be a local type

> _Note_: Coherence rules might be changed in the future

## 7.14. Associated items [↵](#tables-of-contents)

```
<assoc-item> := <fn-item>
              | <type-alias>
              | <const-item>
              | <property-item>
```

Associated items are items that can be defined in traits or implemetentations to be associated with the given trait or type.
They are a subset of items that can be clared inside of a module.

Associated items are useful when wanting to make items logically related to a given item.

Every associated can come in 2 variations:
- Ones that define an implementation
- Ones that only define a signature (only allowed inside of traits)

### 7.14.1. Associated types [↵](#tables-of-contents)

```
<assoc-type> := <assoc-trait-type> | <assoc-impl-type>
```

An associated types are only allowed to be defined in trait implementations and in traits.

Associated types can generally be split into 2 types:

#### Associated trait type

```
<assoc-trait-type> := 'type' <name> [ <generic-params> ] [ ':' <trait-bounds> ] [ <where-clause> ] ';'
```

An associated trait type definition declared a signature for associated type implementation.
They can include generic paramters, trait bounds and a where clause.
When a trait bound is defined, it requires any type which can be used as the associated type to implement those traits
An implicit `Sized` trait is also bound on associated types, but can be relaxed using the `?Sized` bound.

#### Associated type implementation

```
<assoc-impl-type> := { <attribute> }* [ <vis> ] 'type' <name> '=' <type> ';'
```

An associated type definition has a very similar syntax than that of a type alias, except is cannot define any generic paramters.
If a type `Item` has an associated type `Assoc` from a trait `Trait`, then `path_to_trait_assoc_type` is a type tht is an alias to the type specificed in the associated type definition.
Otherwise a the type can be accessed as `Item::Assoc`, this can also be used if there in only 1 trait implementation with an associated type with a given name.

### 7.14.2. Associated constants [↵](#tables-of-contents)

```
<assoc-const> := <assoc-trait-const> | <assoc-impl-const>
```

Associated constants are constants associated with a type.

For more info, see the section on [const items](#79-const-item-)

### 7.14.3. Associated properties [↵](#tables-of-contents)

```
<assoc-property> := <assoc-trait-property> | <assoc-property-item>
```

Associated properties are properties associated with a type.

For more info, see the section on [property items](#711-properties-)

### 7.14.4. Associated functions [↵](#tables-of-contents)

Associated functions are that are associated with a type.
They come in 2 kinds:
- Regular functions associated with a type
- Methods taking in a the type as a receiver

For more info, ee the section about function at [7.3. Function](#73-function-)

## 7.15. External block [↵](#tables-of-contents)

```
<external-block> := 'extern' [ <abi> ] '{' { <extern-static> | <extern-fn> }* '}'
```

An external block provides declarations of items that are not defined in the current library and are used for the foreign function interface.

There are 2 kind of items that are allowed within an external block: functions and static items.
Calling functions or static items that are declared in external blocks are only allowed within an unsafe context.

More info about [external functions](#736-external-function-qualifier-) and [statics](#7103-external-statics-) can be found in their respecitive sections.

# 8. Statements [↵](#tables-of-contents)

```
<stmt> := <var-decl>
        | <expr-stmt>
        | <defer-stmt>
```

A statement is a component of a block, which is in turn part of an outer expression or a functions.
Statements differ from expressions, as they do not return a value and can only directly exist within a scope

## 8.1. Item statement [↵](#tables-of-contents)

```
<item-stmt> := <item>
```

An item statement is a itemswhich can be defined within a block.
Declaring them at a location a statement can be defined limits their definition to the block they belong to.
As such, they cannot be referenced outside of the specific scope they are declared in.

They may implicitly capture generics from an outer scope, unless they are shadowed by the generic with the same name by the item.

## 8.2. Variable declaration [↵](#tables-of-contents)

```
<var-decl> := 'let' <pattern-top-no-alt> [ ':' <type> ] [ '=' <expr> [ 'else' <block-expr> ] ] ';'
```

A variable declartion introduces a new variable withing a scope.
By default, variables are immuatable and need to explicitly be defined as `mut` to be able to be mutated.

If no type is given, the compiler will infer the type from the surrounding context, or will return an error if insuffient information can be retreived from code.

Any variable introduced will be visible until the end of the scope, unless they are shadowed by another declaration.

A variable may also be declared as being unitialized, this requires:
- An explicit type to be given
- Only identifier or tuple patterns
- The variable needs to be assigned a value in all possible paths that can reach the first use of that variable.

A variable declaration may also contain an `else` block, allowing a refutable pattern.
If this patten fails to match, the else block will get executed, this is generally used to either panic or return from the function.
If an `else` block is not present, the pattern needs to be irrefutable.

## 8.3. Expression statement [↵](#tables-of-contents)

```
<expr-stmt> := <expr-no-block> ';'
             | <expr-with-block>
```

An expressions statement evaluated a given expression and ignored the result.
As a rule, an expression statement's purpose is to trigger the effects of evaluating its expression.

If an expression ends with a block, if used in a context where a statement is permitted, the trailing semicolon can be omitted.
This could lead to ambiguity when this can be parsed as both part of a larger expression or as a standalone expression, it will be parsed as a statement.

The return type of an exprssion used in a statement must be a unit type.

## 8.4. Defer statement [↵](#tables-of-contents)

```
<defer-stmt> := 'defer' <expr-with-block>
              | `defer` <expr-no-block> ';'
```

A defer expressions delays the execution of an expression until the end of the scope, but before any destructors are being run.
Defers ere evaluated in the reverse order they are called, in a so-called LIFO (Last-In First-Out) order.

# 9. Expressions [↵](#tables-of-contents)

```
<expr> := <expr-with-block> | <expr-no-block>
<expr-with-block> := <block-expr>
                   | <if-expr>
                   | <loop-expr>
                   | <match-expr>
<expr-no-block> := <literal-expr>
                 | <path-expr>
                 | <unit-expr>
                 | <operator-expr>
                 | <in-place-expr>
                 | <type-cast-expr>
                 | <type-check-expr>
                 | <parenthesized-expr>
                 | <contructing-expr>
                 | <index-expr>
                 | <tuple-index-expr>
                 | <call-expr>
                 | <method-call-expr>
                 | <field-access-expr>
                 | <closure-expr>
                 | <range-expr>
                 | <break-expr>
                 | <continue-expr>
                 | <fallthrough-expr>
                 | <return-expr>
                 | <underscore-expr>
```

Expressions are to do 2 things:
- create a value
- produce a side-effect

Each expression will return the value produced by it, while also applying any effect during evaluation.
An expression ca contain multiple sub-expressions, which are called the operands of an expression.

Each expression dictates the following:
- Whether or not to evaluate teh operands when evaluating the expression.
- The order in which to evaluate the operands
- How to combine the operands' values to obtain the value of the expression

In this way, the structure of the expression dictates the structure of execution.

For information about the precedence of expression, see the [precedence section](#15-precedence-).

In general, the operands of an expression will be evaluated before any side-effects will be applied, and the operands are evaluated from left to right.
Each expression will define if and in which order there expressions are evaluated, if they deviate from this order.

## 9.1. Expression details [↵](#tables-of-contents)

### 9.1.1 Place, value & assign expressions [↵](#tables-of-contents)

Expressions can be divided in 3 categories:
- Place expressions.
- value expressions.
- Assign expressions.

With each expression, operands may likewise occur in either place or value context.
The evaluation of an expression depends both on its own category and the context it occurs in.

#### Place expressions

A place expressions represents an expression that point to a location in memory.

They refer to the following expressions:
- Local variable, like a path
- Static variables, like a path
- Dereferenced addresses or references
- Indexing resulting in a place expression
- Field references
- Parenthesized place expressions
- Any call (function and operator) that results in a place expression
- Any propery resulting in a place expression

#### Assign expressions

An assign expression is any expression which can appear on the left hand size of an assignment operator.

They refer to the following expressions:
- Place expressions
- Underscores
- Tuples of assign expressions
- slices of assign expressions
- Tuple structs of assign expressions
- Aggregate structs of assign expressions (with possible named fields).
- Unit structs

#### Value expressions

A value expression is any other expressions.

### 9.1.2. Moved & copied types [↵](#tables-of-contents)

When a place expression is evaluated in a value expression, or is bound to a value expression in a pattern, it denotes the value held in that memory location.
If the type is copyable, then the value will be copied, otherwise if the value is sized, it is moved.
Only the following place expressions can be moved out from:
- variables that are not currently borrowed
- temporary fields
- fields of place expressions that can be moved out of, if the field is does not need to be dropped or used in a drop implementation, i.e. if the field can be partially moved
- Result of a expressions that supports moving out of. _TODO: This needs a good API_

After moving out of a place expression that evaluates a local expression, the location is deinitialized and cannot be read from again until it is reinitialized.

In all other places, a place expression in a value expression will result in an error.

### 9.1.3. Mutability [↵](#tables-of-contents)

For a place expression to be able to be assigned to, it needs to be mutable, either by being mutably referenced (either explicitly or implicitly), or must be explicitly refered to as mutable in a pattern.
These are called _mutable place expression_, all other place expressions are _immutable place expressions_

The following expressions can be used as mutable expressions:
- Mutable variable that is currently not borrowed.
- Temporary values
- Fields
- Dereferences of mutable pointers, i.e. `*mut T`
- Dereferences of a variable or a field of one, with a type of `&mut T`
- Dereferences of types supporting mutable dereferences, when the `DerefMut` trait is 
- Any expressions that results in a place expression that is mutable

### 9.1.4. Temporaries [↵](#tables-of-contents)

When using a value expression in a location a place expression is expected, a temporary unnamed memory location is created (usually on the stack) and is set to the value of the expression creating the temporary.
The temporary value will then be used as the place expressions and will be dropped at the end of the expression's scope.

### 9.1.5 Implicit borrows [↵](#tables-of-contents)

Certain expressions will treat an expression as a place expression by implicitly borrow it.

Implicit borrowing takes place in the following expressions:
- Left operand in a method call
- Left operand in a field expression
- Left operand in a call expression
- Left operand in an index expression
- Operand of a derefence operator
- Operands of a comparison
- Left operand of a compound assignment

## 9.2. Literal expression [↵](#tables-of-contents)

```
<lit-expr> := <literal> [ ':' <name> ]
```

A literal expression is used to get a value of a given literals, and is evaluated at compile time.

Literal expressions also allow a special literal operator to be applied to them, htis is a constant function that can a value from a literal.

## 9.3. Path expression [↵](#tables-of-contents)

```
<path-expr> := <path-in-expr>
             | '.' <name>
```

A path expression uses a path to refer to a local variable or item.
Path expressions referencing local or static variables are place expression, all other path expressions are value expressons.

A path may also refer to an inferred path, which is represented by a `.`, followed by a name.
This is currently limited to plain enum members of enum types that can be inferred.


## 9.4. Unit expression [↵](#tables-of-contents)

```
<unit-expr> := '(' ')'
```

A unit expressions is an empty expressions that does nothing and return a unit type.

## 9.5. Block expression [↵](#tables-of-contents)

```
<block-expr> := [ 'unsafe' | 'const' ] <block>
```

A block expression creates a new anonymous scope witin an expression, allowing more than just expressions to be defined in a location normally only an expressions would be allowed.
A block executes its non-item components and then its last optional expression.
Any items or local variable in the scope only live for the lenght of the scope and are not accessible outside of the scope.

The block can contain a final expression that is not ended by a semicolon, this will implicitly return its value from the block.

There are 2 special types of block expressions:
- `unsafe`: they allow expression that are normally only allowed in unsafe contexts
- `const`: their contents is resolved at compile time.

Blocks allow for the arbitrary nesting of code, meaning that it allows statements, expressions, and items.

Blocks are always value expressions.

### 9.5.1. Unsafe block [↵](#tables-of-contents)

An unsafe block will run the entirety of its code within an unsafe constext, allowing unsafe operation to be called within it.

### 9.5.2. Const block [↵](#tables-of-contents)

A constant block will execute its code at compile time and will become an inline constant value after compilation.

## 9.6. Operator expression [↵](#tables-of-contents)

```
<op-expr> := <prefix-op> <expr>
           | <expr> <postfix-op>
           | <expr> <binary-op> <expr>
```


An operator expression applies operators on 1 or 2 sub-expressions.
The resulting value of these expression will depend on the implementation of the operators.

For additional info on operators, check the [Operator section](#14-operators-).

## 9.7. Parenthesized expression [↵](#tables-of-contents)

```
<paren-expr> := '(' <expr> ')'
```

A parenthesized epxression wraps a single expression, allowing the expression to be evaluated before any other expressions that are outside of the parentheses will be executed.

Parenthesized expressions can be both place and value expressions, depending on the expression within parentheses.

Parentheses explicitly increase the precedence of this expression above that of other expressions, allowing expressions that would have a lower precedence to be executed before outer expressions use this expression.

## 9.8. In-place expression [↵](#tables-of-contents)

```
<in-place-expr> := <expr> '<-' <expr>
```

In some occasions, it might be preferable to directly write to the assignee, without creating an temporary value on the stack first, particularly for large types.
An in-place assignment expession allows a value to be directly writtin inside of an assignee expressions.

Currently the expressions allows to be used for in-place assignments are limited to so called 'constructing expressions`.

> _Note_: might need some syntax to pass arguments through to functions

## 9.9. Type cast expression [↵](#tables-of-contents)

```
<type-cast-expr> := <expr> <as-op> <type>
<as-op> := 'as' | 'as?' | 'as!'
```

A type cast expression is a special binary operator, which has a type on the right hand side.

Executing the expression will cast the value on the left hand side to the type on the right hand side.


### 9.9.1. Builtin casts [↵](#tables-of-contents)

The builtin cast `as` can be used to explicilty perform coercions, as well as the follwoing casts.
Any cast that does not fis eitehr a coercion rule or an entry in the table is a compiler error.
Here `*T` means either `*const T` or `*mut T`. `m_` stands for an optional `mut` in referecne types and `mut` or `const` in pointer types.

Type of `e`               | `U`                                 | Cast performed by `e as U`
--------------------------|-------------------------------------|----------------------------
Integer or Float type     | Integer or float type               | Numeric cast
Enumeration               | Integer type                        | Enum cast
Boolean or character type | Integer type                        | Primitive to integer cast
Integer type              | Character type                      | Integer to character cast
`*T`                      | `*U` where `U` is sized *           | Pointer to pointer cast †
`*T` where `T` is sized   | Integer type                        | Pointer to address cast †
`&m1 T`                   | `*m2 T` **                          | Reference to pointer cast †
`&m1 [T; N]`              | `*m2 T` **                          | Array to pointer cast †
Function item             | Function pointer                    | Function item to function pointer cast †
Function item             | `*U` where `U` is sized             | Function item to pointer cast †
Function item             | Integer                             | Function item to address cast †
Function pointer          | `*U` where `U` is sized             | Function pointer to address cast †
Function pointer          | Integer                             | Function pointer to address cast †
Closure ***               | Function pointer                    | Closure to function pointer cast †
`T`                       | Opaque type                         | Type to opaque cast
`*T`                      | `*U` where 'U' is an opaque type    | Type to opaque cast
`&m1T`                    | `&m2 U` where 'U' is an opaque type | Type to opaque cast

\* or `T` and `U` are compatible unsized types, e.g. both slices, both are the same interface 

\** only when `m1` is `mut` or `m2` is `const`. Casting `mut` reference to `const` pointer is allowed.

\*** only for closure that do not capture (close over) any local variables

† Casts are unsafe

_NOTE: casting an integer type to a pointer is only allowed via the appropriate library functions_

#### Numeric cast semantics
- Casting between two integer types of he same size (e.g. i32 -> u32) is a no-op (2's complement is used for negative numbers)
- Casting from a larger integer to a smaller integer (e.g. u32 -> u8) will truncate
- Casting from a smaller integer to a larger integer (e.g. u8 -> u32) will
  - Zero extend when the source is unsigned
  - Sign extend when the source is signed
- Casting from a float to an integer will round the float towards zero
  - `NaN` will return 0
  - Values larger than the maximum value, including `INFINITY`, will saturate to the maximum value of the integer type
  - Values samller than the minimum integer value, including `-INFINITY`, will saturate to the minimum value of the integer type
- Casting from an integer to a floating point will produce the closest possible float *
  - if necessary, rounding is accoring to `roundTiesToEven` mode ***
  - on overflow, infinity (of the same sign as the input) is produces
  - note: with teh current set of numeric types, overflow can only happen on `u128 as f32` for values greater or equal to `f32::MAX + 0.5`
- Casting from an f32 to an f64 is perfect and lossless
- Casting from an f64 to an f32 will produce the closest possible f32 value **
  - if necessary, rounding according to `roundTiesToEven` mode ***
  - on overflow, infinity (of the same sign as the input is produced)

\* if integer-to-float casts with this rounding mode and overflow behavior are not supported natively by the hardware, these casts will likely be slower than expected.

\** If f64-to-f32 casts with this rounding mode and overflow behavior are not supported natively by the hardware, these casts will likely be slower than expected.

\*** as defined in IEEE-754-2008 §4.3.1: pick the nearest floating point number, preferring the one with an even least significant digit if exactly half way between two floating point numbers.

#### Enum cast semantics

Casts from an enum to its distriminant, then uses a numeric cast is needed. Casting is limited to the following kinds of enumerations:
- Unit-only enums
- Field-less enums without explicit discriminants, or werhe only unit variants have explicit discriminants
- Flag enums

#### Primtive to integer cast semantics

- `false` casts to 0, `true` casts to 1.
- character types cast to the value of the code point, then uses a numeric cast is needed.

#### Integer to character cast semantics

Casts an integer type corresponding to the size of the character type, then cast to a character type with the corresponding code point.

#### Pointer to address casts semantics

Casting from a raw pointer to an integer produces the machine address of the referenced memory.
If the integer type is smaller than the pointer type, the address may be truncated; using `usize` avoids this.

#### Pointer-to-pointer semantics

`*const T`/`*mut T` can be cast to `*const U`/`*mut U` with the following behavior:
- If `T` and `U` are both sized, the pointer returned is unchanged.
- If `T` and `U` are both unsized, the pointer is also returned unchanged. In particular, the metadata is preserved exactly.
  If `T` and `U` are objects, this does require that they are compatible types, e.g. same non-marker interfaces.

  For instance, a cast from `*const [T]` to `*const [U]` preserves the number of elements.
  Note that, as a consequence, such casts do not neccesarily preserve the size of the pointer's referent (e.g. casting `*const [u16]` to `*const [u8]`) will result in a raw pointer which refers to an object of half the size of the original).
  The same holds for `str` and any compound type whose unsized tail is a slice type, such as `struct Foo(i32, [u8])` or `(u64, Foo)`
- If `T` is unsized and `U` is sized, the cast discards all metadata that completes the wide pointe `T` and produces a thin ponter `U` consisting of hte data part of the unsized pointer.

### 9.9. 2. Try and unwrap casts [↵](#tables-of-contents)

A try cast `as?` can be used to cast a type from an interface object, impl interface object, or a generic to a given type, returning an optional type with valid value when the cast is possible and a `None` when it's not.
This can therefore be used to dynamically handle code based on a type when RTTI info is avaiable.

An unwrap cast `as!` is similar to a try cast, but meant for in usecases the user is certain that the cast is possible, as it will unwrap the resulting nullable type.
This could also be written as `(a as? T)!`.
By default, it will panic when the cast is not available, but in certain configuration, this can be changed into a cast that always passes, so may return in UB if not used correctly.

Any cast that happens on a generic or impl interface object will be resolved at compile time.

## 9.10. Type check expression [↵](#tables-of-contents)

```
<type-check-expr> := <expr> <is-op> <type>
<is-op> := 'is' | '!is'
```

A type check expression is a special binary operator, which has a type on the right hand side.
A type check expression can be used to check if an interface object, impl interface object, or a generic is of a given type.
This check can only occur on place expressions.

There is both a positive and negative version of this expression.

When the positive version is used in the condition of a conditional expression, and it is the only type check experssion on this value, the value will be implicitly promoted within the block that gets executed when the condition is true.

Any cast that happens on a generic or impl interface object will be resolved at compile time.

## 9.11. Constructing expression [↵](#tables-of-contents)

```
<constructing-expressions> := <tuple-expr>
                            | <array-expr>
                            | <aggregate-expr>
```

A constructing expression constructs a new instance of a type.
This consists of a group of multiple expressions and can be used 

### 9.11.1. Tuple expression [↵](#tables-of-contents)

```
<tuple-expr> := '(' <expr> { ',' <expr> }+ ')'
```

A tuple expression constructs a tuple value.

The construction exists out of a comma separated list of values that need to be placed within the tuple.
Since 1-ary tuples are not supported, if the expression only contains 1 operand, it will be interpreted as a parenthesized expression.
Similarly if the expressions contains 0 operands, a unit type will be created.

The number of operands within the tuple initializer defines the arity of the tuple.
When initializing a tuple, the operand will be evaluated in the order they are written, i.e. left-to-right.
Each operand will be assigned to the field they represent within the expression, i.e. the first operand will be assigned to field '0', and so on.

Tuple expressions are value expressions.

### 9.11.2. Array expression [↵](#tables-of-contents)

```
<array-expr> := <array-list-expr> | <array-count-expr>
<array-list-expr> := '[' ( <expr> { ',' <expr> }* [ ',' ] ) ']'
<array-count-expr> ;= '[' <expr> ';' <expr> ']'
```

An array expression constructs arrays, an come in 2 forms.

The first form lists out all values in the array, this is represented as a comma separated list of expressions.
Each expression is evaluated in the order that they are written, i.e. left-to-right.

The second form consists out of 2 expression separated by a `;`.
The expression on the left is called the 'repeat' operand, the expression on the right the 'count' operand.
The count operand must be able to be evaluated at compile time and have a `usize` type.
This form creates an array with a length of the value of the cound operand, with each value being a copy of the value evaluated from the repeat operand.
This means that `[a;b]` create an array of `b` elements with the value `a`.
If the value of the count operand is larger than 1, the repeat operand must be copyable or must point to a constant item.

Creating a multi-dimensional array can be done by nesting array expressions within other array expression, i.e. `[[..], [..], [..]]` will result in a 2D array.

### 9.11.3. Struct expressions [↵](#tables-of-contents)

```
<struct-expr> := <struct-expr-path> '{' [ <struct-expr-member> { ',' <struct-expr-member> }* [ ',' [<struct-complete>] ] ] '}'
<struct-expr-path> := <path> | '.'
<struct-expr-member> := [ <ext-name> ':' ] <expr>
<struct-complete> := '..' <expr>
```

A struct expression creates a structure, enum, or union value.
There are 3 forms of this expression, corresponding in the 3 types it can create

#### Struct (& union) expression

A struct expressions with fields enclosed in curly braces allows the specifying of values for each individual field in the structure.

A unions is created as a struct expression with only a single field.

##### Functional update syntax

An struct expression tha constructs a value of a struct type can terminate with a `..` followed by an expression.
This entire expression uses the given values for fields that were specified and moved or copies the remaining fields from the base expression.
As with all struct expressions, all of hte views of tghe struct must be visble, even those not explicitly named.

Using this expression will also overwrite all default fields.

##### Struct field shorhand

When initializing an struct value with named fields, it is allowed to write `fieldname` instead of `fieldname: fieldname`.
This allows for a more compact syntax with less duplication.

##### Default fields

When a struct has default field values, they are not required to assign a value to those fields.

#### Tuple struct expression

An struct expression with fields enclosed in parentheses constucts a tuple struct.
Though listed here as specific experssion, this is equivalent to the a call expression to the tuple struct's pseudo-constructor.

#### Unit struct

A unit struct expression creates is either just a path or an implied path.
This refers to the unit struct's implicit constant of it's value.
The unit struct value can also be constructed in 2 ways:
- as a path
- as a fieldless struct expression
- as an implied fieldless struct expression

## 9.12. Index expression [↵](#tables-of-contents)

```
<index-expr> := <expr> '[' <expr> { ',' <expr> }* ']'
```

An index expression can be used to get a value out of a type using a given index.

When the expression being indexed is either an array or a slice, it will get the relevant element at a given index or a subslice at the given range.
If the array of slice is mutable, the resulting value will be memory location that can be assigned to.

When the array or slice being index is a multi-dimensional array, it can be indexed with a range of comma separated of indices.
If a range shows up, the elements after it must only be boundless range exprsions, i.e. `..`, as we can't get a slice across multiple sub-dimensions.

Indices are 0-based for arrays and slices.
If array access is a constant expression, bounds can be checked at compile-time, otherwise the check will be performed at runtime and will panic when being indexed out of range

For any other type, the resulting indexing will depend on whether the index implementation returns a reference or not.

For all other types, the following operations will happen:
- In an immutable place context, the resulting value will be `Index::index(&a, b)`.

  If the index implementation were to return a refernce, it would be implicitly dereferenced.

- In a mutable place context, the resulting value will be `*IndexMut::index_mut(&a, b)`.


The interfaces associated with the index expressions are:
- `Index`
- `IndexMut`

## 9.13. Tuple index expression [↵](#tables-of-contents)

```
<tuple-index-expr> := <expr> . <unsigned-decimal-literal>
```

A tuple index expressions is used to access fields within a tuple type (a tuple or tuple structure).

A tuple is indexed using an unsigned decimal literal, wit h no leading zeros or underscores.

Evaluation of a tuple has no side-effects, other than the evaluation of the tuple operand.
This expressions is a place expression, so it evaluateds to the location of tuple field with the same name as the tuple index.

## 9.14. Call expression [↵](#tables-of-contents)

```
<func-call> := <expr> '(' [ <function-args> ] ')'
<func-args> := <func-arg> { ',' <func-arg> }* [ ',' ]
<func-args> := [ <name> ':' ] <expr>
```

A call expessions call a function.

The expression will complete when the function returns.
If the function return a value, this value will be returned, this function is therefore a place or value expression, depending on the returned value.

The function expression can be called if it follows either of the following cases
- The expression is function or function pointer expr type.
- The expression is of a value that implement one of the relavent function interfaces.

If needed, an automatic borrow of the function expression is takes.

An argument can have an additional function argument label in case the function requires one.
Any default arguments do not need to be provided and will be evaluated after evaluating the supplied operands, in the order they were defined in the signature.

Arguments are evaluated in the order they are written. i.e. left-to-right.

### 0.14.1. Universal function call syntax (UFCS) & disambiguating function calls [↵](#tables-of-contents)

All function calls support UFCS, meaning that for method calls, if they are called as normal functions, the receiver is passed as the first argument to the function and has an optional 'self' label.

Several situation can occur with result in ambiguities of which function is being called.
This situation only will happen when the first argument is unlabeled, as a receiver is unlabeled, and may occur when:
- Multiple in-scope interfaces define methods with the same name, and parameters for the same types.
- Auto-`deref` is undesireable; for example, distinguishing between methods on a smart pointer itself and their pointer's referent,  
- Methods which take no arguments and return properies of types.

To resolve the ambiguity, the programmer may refer to their desired method or function using more specific paths, types, or interfaces.

## 9.15. Method call expression [↵](#tables-of-contents)

```
<method-call-expr> := <expr> '.' <name> '(' ( <func-args> )? ')'
```

A method call constists of an expression (the 'receiver') followed dot, and identifier, and a set of function arguments.
Methods calls are resolved to associated methods on specific interfaces, either statically dispatching to a method if the exact self-type of the left hand-size is known,
or dynamically dispatching if the left-hand-side expression is an indidirect interface object.

When looking up a method call, the receiver may be automatically dereferenced or borrowed in order to call a method.
This requires a more complex lookup process than for other functions, since there may be a number of possible methods to call. The following procedure is used:

1. Build a list of candidate receiver types.
   1. Obtained by repeatedly dereferencing the receiver's type, adding each type encountered to the list.
   2. Finally attempt an unsized coercion at the end, and adding the result type to the candidate list if that is successful.
   3. Then for each candidate `T`, add `&T` and `&mut T` to the list immediately after `T`.
2. Then for each candidate type `T`, search for a visible method with a receiver of that type in the following places.
   1. `T`'s inherent methods (methods implemented directly by T).
   2. Any of the methods provided by a visible interface implemented by `T`.
      If `T` is a type parameter, methods provided by interface bounds on `T` are looked up first.
   3. All remaining methods scopes are looked up.
3. Pick the methods matching the arguments.

> _Note_: more detailed info about argument resolution, check the function definition item

If this results in multiple possible candidates, then it is an error, and the receiver must be converted to an appropriate receiver type to make the method call.

This process does not take into account the mutability of the receiver, or whether a method is `unsafe`.
Once a method is looked up, if it can't be called for one (or more) of those reasons, it will result in a compiler error.

If a step is reacehhd where there is more than one possible methods, such as where generic methods or interfaces are considered the same, the it is a compiler error.
These cases requre a disambiguating function call syntax for metods and functon invocations.

An argument can have an additional function argument label in case the function requires one.
Any default arguments do not need to be provided and will be evaluated after evaluating the supplied operands, in the order they were defined in the signature.

## 9.16. Field access [↵](#tables-of-contents)

```
<field-access-expr> := <expr> ( '.' | '?.' ) <name>
```

A field expression is a place expression that evaluates to the location of a field of a struct or union.
When the operand is mutable, the field expression is also mutable.

Field expression cannot be followed by an opening parenthesis, as this would be a method call expression.

A field can be accessed in 2 ways:
- A direct field access using `.`: this will just access the field
- A 'null'-checked field access using `?.`: this will eitehr access the field if the left-hand-side expression is valid, in case of an erronous value, it will just return the value of the expression.

The pseudo-access `!.` is actually an unwrap operator, followed by a field access.

### 9.16.1 Automatic dereferencing [↵](#tables-of-contents)

If the type of the left-hand-side operand implements `Deref` or `DerefMut` depending on whether the operand is mutable, it is automatically dereferenced as many times as necessary to make the field access possible.
This process is also called 'autoderef' for short.

### 9.16.2. Borrowing [↵](#tables-of-contents)

The field of a struct or a reference to a struct are treated as separate entities when borrowing.
If the struct does not implement `Drop` and iis stored in a local variable, this also applies to moving out of each of its fields.
This also does not apply if automatic dereferencing is done through user defined types that don't support this.

## 9.17. Closure expressions [↵](#tables-of-contents)

```
<closure-expr> := ( 'move' )? '|' ( <closure-parameters> ) '|' ( <expr> | ( '->' <func-ret> <block> ) )
<closure-parameters> := <closure-parameter> ( ',' <closure-parameter> )* ( ',' )?
<closure-parameter> := ( <attribute> )* <pattern-no-top-alt> ( ':' <type> )
```

A closure expression, also known as a lambda expression, a lambda, or a functor in some languages, defines a closure type and evaluates to a value of that type.
Each parameter can have an optional type, but this can be infered depending on the location the closure is defined.
If there is a return type, the closure must have a block.

A closure expression denotes a function that maps a list of parameters onto the expression that follows the paramters.
Just like a `let` binding, the closure paramters are irrefutable patterns, whose type annotation is optional and will be inferred from context if not given.
Each closure expression has a unique, anonymous type.

Significantly, closure expressions capture their environment, which regular function definitions do not.
Without the `move` keyword, the closure expression infers how it captures each variable from its environment, preferring capture by shared reference, effectively borrowing all outer variables mentioned inside the closure's body.
If needed, the compiler will infer that insted of mutable references should be taken, or that the values should be moved or copied (depending on their type) from the environment.
A closure can be forced to capture its environment by copying or moving valures by prefixing it with the `move` keyword.
This is often used to ensure that the closure's lifetime is static.

### 9.17.1. Closure trait implementations [↵](#tables-of-contents)

Which trait a closure type implemetns depends on how variables are captured and the typesss of the captured expression.
See the call trait section for how and when a closure implements the respective trait.

## 9.18. Range expression [↵](#tables-of-contents)

```
<range-expr> := <range-exclusive-expr>
              | <range-from-expr>
              | <range-to-expr>
              | <range-full-expr>
              | <range-inclusive-expr>
              | <range-to-inclusive-expr>

<range-exclusive-expr> := <expr> '..' <expr>
<range-from-expr> := <expr> '..'
<range-to-expr> := '..' <expr>
<range-full-expr> := '..'
<range-inclusive-expr> := <expr> '..=' <expr>
<range-to-inclusive-expr> := '..=' <expr>
```

The '..' and '..=' operators will construct an object of one of the following range variants:

Production                  | Syntax        | Type               | Range
----------------------------|---------------|--------------------|-------------------
<`range-exclusive-expr`>    | `start..end`  | `Range`            | start <= x < end
<`range-from-expr`>         | `start..`     | `RangeFrom`        | start <= x
<`range-to-expr`>           | `..end`       | `RangeTo`          | x <= end
<`range-full-expr`>         | `..`          | `RangeFull`        | -
<`range-inclusive-expr`>    | `start..=end` | `RangeInclusive`   | start <= x <= end
<`range-to-inclusive-expr`> | `..=end`      | `RangeToInclusive` | x <= end

## 9.19. If expression [↵](#tables-of-contents)

```
<if-expr> := <label-decl> 'if' <branch-condition> <block> ( 'else' <block> )?
<branch-condition> := ( <expr> | <cond-let-binding> ) ( '&&' ( <expr> | <cond-let-binding> ) )*
<cond-let-binding> := 'let' <pattern> '=' { <scrutinee> excluding lazy boolean operator expressions }
```

An `if` expression is a conditional branch in program control.
The condition must resolve to a boolean expression.
If the condition operand evaluates to `true`, the consequent block is executed and any subsequent `else if` and `else` block is skipped.
If the condition operand evaluates to `false`, the consequent block is skipped and the subsequenct `else if`  condition is evaluate.
If all `if` and `else if` conditons evaluated to `false`, then any `else` block is executed.
An `if` expression evaluates to the same value as the executed block, or `()` if no block is evaluated.
An `if` expression must have the sae type in all situations.

When a constant experession used for the condition operand, the `if` will be essentially eliminated, depending on the result of the value.

When any branch returns a value, all possible branches should return the same value.

### 9.19.1 If let [↵](#tables-of-contents)

In addition to general expression, the `if` expressions also supported let bindings.
A let binding will be true if the scrutinee matches the pattern matches the pattern.
When a pattern matches, the bound variable will be accessible within the consequent block.

Multiple pattens may be specified using the `|` operator.
This is the same semantics as with `|` in `match` expressions.

When a `let` binding is introduces, the use on the lazy OR boolean operator is not allowed when not in a parenthesized expression.

## 9.20. Loops [↵](#tables-of-contents)

Xenon supports five loop expressions:
- a `loop` expression denoting an infinite loop
- a `while` expression looping until a predicate is false
- a `do while` expression looping until a predicate is false, guaranteeing to run the loop at least once
- a `for` expression extracting a value from an interator, looping until the iterator is empty
- a labelled block expression running a loop exactly once, but allowing the loop to exit early with `break`

All six types of loop expression support `break` expressions and labels.
All except labelled block expressions support `continue` expressions.
Only `loop` and labelled block expressions support evaluating to non-trivial values.

### 9.20.1. Loop expression [↵](#tables-of-contents)

```
<loop-expr> := <label-decl> 'loop' <basic-block>
```

A `loop` expression repeats execution of a body continuously.

A `loop` expression without an associated `break` expression is diverging and has type `!`.
A loop expression containing associated `break` expressions will terminate, and must be type compatible with the value of the `break` expressions.

### 9.20.2. While expression [↵](#tables-of-contents)

```
<while-expr> := <label-decl> 'while' <branch-condition> <basic-block>
```

A `while` loop begins by evaluating the loop condition operand.
If the loop conditional operand evaluates to true, the loop block executes, the control return to the loop conditional operand.
If the loop conditional expression evaluates to `false`, the `while` expression completes.

_TODO: `while else` might have some good usecases_

#### While let

In addition to a general expression, the `while` expression also supports let bindings.
A let binding will be true if the scrutinee matches the pattern matches the pattern.
When a pattern matches, the bound variable will be accessible within the consequent block.

Multiple pattens may be specified using the `|` operator.
This is the same semantics as with `|` in `match` expressions.

When a `let` binding is introduces, the use on the lazy OR boolean operator is not allowed when not in a parenthesized expression.

## 9.20.3. Do-while expression [↵](#tables-of-contents)

```
<do-while-expr> := <label-decl> 'do' <basic-block> 'while' <expr>
```

A `do while` loops begins by running the body of the loop at least once, after which the boolean loop condition operand is evaluated.
If the loop conditional operand evaluates to true, the loop block executes, the control return to the loop conditional operand.
If the loop conditional expression evaluates to `false`, the `do while` expression completes.

### 9.20.4. For expression [↵](#tables-of-contents)

```
<for-expr> := ( <label-decl>? ) 'for' <patern> 'in' <expr> <block>
```

A `for` expression is a syntactic construct for looping over elements provided by an implementation of `IntoIterator`.
If the iterator yields a value, that value is matched against the irrefutable pattern, the body of hte loop is executed, and then control returns to the head of the `for` loop.
If the iterator is empty, the `for` expression completes.

### 9.20.5. Labelled block expressions [↵](#tables-of-contents)

```
<labelled-block-expr> := <label> <block-expr>
```

Labelled block expressions are exactly like block expressions, except they allow using `break` expressions within the block.
Unlike loops, `break` expressions within a labelled block experssion must have a label (i.e. the label is not optional).
Similarly, labelled block expressions must begin with a label.

### 9.20.6. Loop labels [↵](#tables-of-contents)

A loop expression may optionally have a label.
If the label is present, the labeled `break` and `continue` expressions nested within the loop may exit out of this loop or return control to its head.

Labels follow the hygeine and shadowing rules of local variables.

## 9.21. Match expression [↵](#tables-of-contents)

```
<match-expr> := ( <label-decl> )? 'match' <expr> '{' ( ( <match-case> )* <final-case> ) '}'
<match-case> := ( <label-decl> )? <pattern> ( <match-guard> )? '=>' ( ( <expr> ',' ) | ( <block> ( ',' )? ) )
<final-case> := ( <label-decl> )? <pattern> ( <match-guard> )? '=>' ( ( <expr> ( ',' )? ) | ( <block> ( ',' )? ) )
<scrutinee> := { <expr> except structure expressions }
```

A `match` expression branches on a pattern.
The exact form of matching that occurs depends on the pattern.
A `match` expressions has a scrutinee expression, which is the value to compare to the patterns.
The scrutinee expression and the patterns must have the same type.

A `match` behaves differently depending on whether or not the scrutinee expression is a place or value expression.
If the scrutinee expression is a value expression, if is first evaluated into a temporary location, and the resulting value is subsequently compared to the patterns in the arms until a match is found.
The first arm with a matching pattern is chosen as the branch target of the `match`, any variables bound  by the patten are assigned to local variables in the arm's block, and control enters the block.

When the scrutinee is a place expression, the match does not allocate a temporary location; however, a by-value binding may copy or move from the memory location.
When possible, it is preferable t omatch on place expressions, as the lifetie of these matches inherits the lifetime of the place expression rather than being restricted to the inside of the match.

Variables bound within the pattern are scoped to the match guard and the arm's expression.
The binding mode (move, copy, or reference) depends on the pattern.

Multiple match patterns may be joinded with the '|' operator.
Each pattern will be tested in a left-to-right sequence until a successful match is found

Every binding in each `|` separated pattern must appear in all of the patterns in the arm.
Every binding of the same name must have the same type, and have the same binding mode.

### 9.21.1. Match guards [↵](#tables-of-contents)

```
<match-guard> := 'if' <expr>
```

Match arms can accept match guards to further refine the criteria for matching a case.
Patten guards appear after the pattern and consts of a boolean expression.

When the pattern matches successfully, the pattern guard expression is executed.
If the expression evaluates to `true`, the pattern is successfully matched against.
Otherwise, the next pattern including other matching with the `|` operator in the same arm is tested.

A pattern guard may refer to the variable bound within the pattern they follow.
Before evaluating the guard, this shared reference is then used when accessing the variable.
Only when the guard evaluates to `true` is the value moved, or copied without moving out of the scrutinee in case the guard fails to match.
Moreover, by holding a shared reference while evaluating the guard, mutation inside the guard is also prevented.

### 9.21.2. Fallthrough labels [↵](#tables-of-contents)

A pattern is allowed to have a label.
A label may only be referenced by a `fallthrough` expression within an arm of the `match` expression.
This will then proceed to evaluate another arm in the `match`.

Labels are only allowed if the arm does not capture any bindings.


## 9.22. Break expression [↵](#tables-of-contents)

```
<break-expr> := 'break' ( <label> )? ( <expr> )?
```

When `break` is encountered:
- in a loop, execution of the associated loop body is immediatelly terminated.
- in a `match`, execution of the associated arm is immediatelly terminated.

A `break` expression is normaly associated with the innermost loop or `match` exclosing the `break` expression, but a label can be used to specify which enclosing loop or `match` is affected.

A `break` expression is only permited in the body of a loop, or an arm of a `match`.

### 9.22.1. Break and loop/match values [↵](#tables-of-contents)

When associated with a loop, a break expression may be used to return a value from that loop, via one of the forms `break EXPR` or `break 'label EXPR`,
where `EXPR` is an expression whose result is returned from the loop.

In the case a loop has an associated `break`, it is not consifered diverging, and the `loop` must have a type compatible with each `break` expression.
`break` without an explicit expression is considered identical to a `break` with the expression `()`.

## 9.23. Continue expression [↵](#tables-of-contents)

```
<continue-expr> := 'continue' ( <label> )?
```

When `continue` is encountered, the current iteration of the associated loop body is immediatally terminated, returning control to the loop head.
These correspond to the following for given loops:
- `while` and `do while` loop: the head is the conditional expression controlling the loop
- iterator `for` loop: the head is the call expression controlling the loop
- manual `for` loop: the head is the increment expression of the loop.

Like a `break`, `continue` is normally associated with the innermost enclosing loop, but `continue 'label` may be used to specify the loop affected.
A `continue` expression is only permitted in the body of a loop.

## 9.24. Fallthrough expression [↵](#tables-of-contents)

```
<fallthrough-expr> := 'fallthrough' ( <label> )?
```

When a `fallthrough` is encountered, the current arm of a `match` will immediatelly terminate and the arm next arm will be evaluated next.
If a label is given, the associated with the label will be evaluated instead.

## 9.25. Return expression [↵](#tables-of-contents)

```
<return-expr> := 'return' ( <expr> )?
```

Return expressions moves its argument into the designated output location for the current function call, destroys the current function activation frame, and transfers control to the caller frame.
When the function being called has named returns, the `return` expression is allowed to overwrite the named return values.

## 9.26. Underscore expression [↵](#tables-of-contents)

```
<underscore-expr> := '_'
```

Underscore experssions are used to signify a placeholder in a destructuring assignment.
The may only appear in the left-hand side of an assignment.

> _Note_: that this is distinct from a wildcard pattern.


# 10. Patterns [↵](#tables-of-contents)

```
<pattern> := <pattern-no-top-alt> ( | <pattern-no-top-alt> )*
<pattern-no-top-alt> := <pattern-no-range>
                      | <range-pattern>
<pattern-no-range> := <lit-pattern>
                    | <identifier-pattern>
                    | <wildcard-pattern>
                    | <reference-pattern>
                    | <struct-pattern>
                    | <tuple-struct-pattern>


```

Patterns are both used to match values, but also to optionally bind them (in case of uses like 'let ...', binding is the intended usecase).

Patterns can be used to destructure types like struct, enums, and tuples.
Destructuring breaks up a value in its constituent elements.

Patterns can be said to be refutable if there is a possibility for it to not be matched, if they will always be matched, they are said to be irrifutable.

## 10.1. Literal pattern [↵](#tables-of-contents)

```
<lit-pattern> := <literal>
```

Literal patterns match the exact value of the literal.

## 10.2. Identifier pattern [↵](#tables-of-contents)

```
<identifier-pattern> := [ 'ref' ] [ 'mut' ] <name> [ '@' <pattern> ]
```

Identifier patterns bind the value they are matched to, to a variable of a given name.
This names needs to be unique within the pattern.
This binding (newly created variable) is allowed to shadow any variable that is defined before the pattern.
The scope of the binding depends on the location of where the pattern is used.

'mut' can be added to make the resulting binding mutable in code.
'ref' can be added to take reference to the element being matched, instead of moving or copying it on match.
'ref' must be used instead of '&' as it actually does the oposite of this.

In addition, a binding may also have a restriction placed on it by appending a pattern behind the name.

By default, the binding mode of this is determined based on the variable being compared.

## 10.3. Wildcard pattern [↵](#tables-of-contents)

```
<wildcard-patter> := '_'
```

A wildcard pattern matches any single element in a pattern, and is used to ignore its value.

## 10.4. Rest pattern [↵](#tables-of-contents)

```
<rest-pattern> := '..'
```

A special case of the wildcard that matches 0 or more elements, and can be used to discard any remaining elements that are not cared about in the match.

## 10.5. Range pattern [↵](#tables-of-contents)

```
<range-pattern> := <exclusive-range-pattern>
                 | <inclusive-range-pattern>
                 | <from-range-pattern>
                 | <to-range-pattern>
                 | <inclusive-to-range-pattern>
<exclusive-range-pattern> := <range-pattern-bound> '..' <range-pattern-bound>
<inclusive-range-pattern> := <range-pattern-bound> '..=' <range-pattern-bound>
<from-range-pattern> := <range-pattern-bound> '..'
<to-range-pattern> := '..' <range-pattern-bound>
<inclusive-to-range-pattern> := '..=' <range-pattern-bound>
<range-pattern-bound> := <number-literal>
                       | <char-literal>
                       | <path-expr>
```

A range pattern can match a value within the given range.
The start of the range needs to preceed the value of the end.

When using path as a bound, it has to be able to be resolved at compile time.

## 10.6. Reference pattern [↵](#tables-of-contents)

```
<reference-pattern> := '&' [ 'mut' ] <pattern-no-range>
```

Reference patterns is used to derefence pointers and references.

Similar to identifier patterns, 'mut' can be added to make the resulting variable mutable.

## 10.7. Struct pattern [↵](#tables-of-contents)

```
<struct-pattern> := <path> '{' [ ( <struct-pattern-elem> { ',' <struct-pattern-elem> }* [ ',' ] ) | <rest-pattern> ] '}'
<struct-pattern-elem> := ( <attribute> )* ( <struct-pattern-elem-tuple> | <struct-pattern-elem-member> | <struct-pattern-elem-iden> )
<struct-pattern-elem-tuple> := <tuple-index> ':' <pattern>
<struct-pattern-elem-member> := <ext-name> ':' pattern
<struct-pattern-elem-iden> := [ 'ref' ] [ 'mut' ] <ext-name>
```

A struct pattern can match struct, enum, and union values that match the defined criteria in the subpatterns.
The also allow for the value to be deconstructed to its members.

There are 3 ways of matching elements:
- Using a tuple element in case of tuple-like types
- Using a values name, followed by a pattern
- Using a value directly with a matching name (this requires a normal name and not an extended name).

## 10.8. Tuple struct pattern [↵](#tables-of-contents)

```
<tuple-struct-pattern> := <path> '(' ( ( <pattern> ( ',' <pattern> ) [ ',' [ <rest-patter> ] ] ) ) | <rest-patter> ')'
```

A tuple struct pattern can match tuple structs that match the defined criteria in the subpatterns.

## 10.9. Tuple pattern [↵](#tables-of-contents)

```
<tuple-pattern> := '(' ( <pattern> ( ',' <patter> )* [ ',' [ <rest-pattern> ] ] ) | <rest-pattern> ')'
```

A tuple pattern can match a tuple values that match the defined criteria in the subpatterns.

## 10.10. Grouped pattern [↵](#tables-of-contents)

```
<grouped-pattern> := '(' <pattern> ')'
```

Grouped patterns are used to explicitly control the precedence of compound patterns.

## 10.11. Slice pattern [↵](#tables-of-contents)

```
<slice-patter> := '[' ( <pattern> ( ',' <pattern> ) [ ',' [ <rest-pattern> ] ] ) | <rest-patter> ']'
```

A slice pattern can match array and slice values that match the defined criteria in the subpatterns.

## 10.12. Path pattern [↵](#tables-of-contents)

```
<path-pattern> := <path>
```

A path pattern can match any constant, or struct or enum member that have no fields.

## 10.13. Enum member pattern [↵](#tables-of-contents)

```
<enum-member-pattern> := ':' <ext-name>
```

A enum member pattern can match any enum member that has no field.

## 10.14. Alternative pattern [↵](#tables-of-contents)

```
<alt-pattern> := <pattern-no-top-alt> { | <pattern-no-top-alt> }*
```

An alternative pattern is a set of subpattern where only a single one needs to match.
Use of this pattern does disallow any identifier patterns, as they cannot be guaranteed to have a value, therefore if you need to capture, you should use individual matches.

## 10.15. Type check pattern [↵](#tables-of-contents)

```
<type-check-patter> := 'is' <type>
```

A type check pattern can be used to explicitly check for a certain type, this includes builtin-types.
Type check patterns can also be used to check if a DST is a given type.

# 11. Type System [↵](#tables-of-contents)

## 11.1. Types [↵](#tables-of-contents)

```
<type> := <type-no-bound>
        | <trait-object-type>
        | <impl-trait-type>

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

### 11.1.1. Rescursive types [↵](#tables-of-contents)

Nominal types may be recursive, meaning that a tpe may havae member that refers, directly or indirectly, to the current type.
These are some limiations on how types can be nested:
- Type aliases must include a nominal type in the recursion, meaning type aliases, or other structural types like arrays and tuples are not allowed.
  i.e. `type Foo = &[Foo]` is not allowed.
- The size of a recursive type must be finite, meanign that the recursive field must be 'broken up' by a type like a pointer or reference type.

### 11.1.2. Parenthesized types [↵](#tables-of-contents)

```
<parenthesized-type> := '(' <type> ')'
```

In some locations it may be possible that a type would be ambiguous, this can be solved using a parenthesized type.
For example, a reference to an trait object type with multiple bounds can be unclear, as we cannot cleanly determine if the one of the bounds is a reference, or the whole set of bounds constitute a single type without requiring to rely heavily on context.

### 11.1.3. Primitive types [↵](#tables-of-contents)

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
<floating-point-type> := 'f16' | 'f32' | 'f64' | 'f128'
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

### 11.1.4. Unit type [↵](#tables-of-contents)

```
<unit-type> := '(' ')'
```

The unit type is a special type representing a zero-sided type.
This is also known as `void` in some other languages.

### 11.1.5. Never type [↵](#tables-of-contents)

The never type is a special type that represents an operation that can never complete.
This type can be implicitly coerced into any type.
It can only ever appear as the return value of a function and can therefore not be part of any type, meaning you can only ever return a never type.

```
<never-type> := '!'
```

### 11.1.6. Path types [↵](#tables-of-contents)

```
<path-type> := <type-path>
```

A path type refers to a user-defined path by its path, there are 3 types it can represent.

### 11.1.7. Tuple types [↵](#tables-of-contents)

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

### 11.1.8. Array types [↵](#tables-of-contents)

```
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

### 11.1.9. Slice types [↵](#tables-of-contents)

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

### 11.1.10. String slice types [↵](#tables-of-contents)

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

### 11.1.11. Pointer types [↵](#tables-of-contents)

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

### 11.1.12. Reference types [↵](#tables-of-contents)

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

#### 11.1.13. Optional types [↵](#tables-of-contents)

```
<optional-type> := '?' <type>
```

An optional type allows a value to be represented using a 'null' or `None` state, which can be used to represent a type with no value set.
When an optional type (or the `Option<T>` type) is used, then depending on the value within, the compiler is allowed to do certain optimizations to encode the 'null' state within the value.
An example is a nullable pointer, where the 'null' state is represented with an address of `0x00000000`.

This is synctactic suger of `Option<T>`.

### 11.1.14 Function types [↵](#tables-of-contents)

A function type is an anonymous compiler-generated type, which cannot be manually defined.
The type references a specific function, including its name and its signature (including parameter labels).

Since this is specific to each function, a value of this type does not need to use any indirection to be called, as it does not contain an actual function pointer.
This also makes this type a 0-sized type.

Separating each function in its own type allows for additional optimization.

When an error message is generated using this type, it will generally show up as something like `fn(_:i32) -> i32 { name }`

### 11.1.15. Function pointer type [↵](#tables-of-contents)

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

### 11.1.16. Closure types [↵](#tables-of-contents)

A closure type is a compiler generated type which cannot be declared manually, and refers to a closure using a unique anymous type.

For more info about closure, see the [closure expression].

### 11.1.17. Intereface Object types [↵](#tables-of-contents)

```
<trait-object-type> := 'dyn' <trait-bound> { '+' <trait-bound> }*
```

An trait object type is an opaque type that implements a set of traits, any set of traits is allowed, except of an opt-in trait like `?Sized`.
The objects are guaranteed to not only implement the given traits, but also their parent traits.

Different trait objects may alias each other if the traits match, but are in different orders, meaning that `dyn A + B + C` is the same as `dyn A + B + C`

An intereface can be assigned to a less specific trait objects, meaning that it can be assgined to a type that has less trait bounds.
This *may* incur some additional overhead, as a new vtable needs to be retrieved and assigned, if this cannot be determined at compile time.

Due to the opaqueness of trait objects, this type is dynamically sized, meaning that it must be stored behind a reference, a pointer, or a type accepting DTSs.

Trait objects are stored in so-called "fat pointers' which consists out of 2 components:
- A pointer to the an object of a type `T` that implements the trait bounds
- A virtual table, also known as a vtable, which contains both RTTI info and a list of function pointers to the methods of the traits and their parent types, of `T`'s implementation.

Trait object types allowe for "late binding" in cases where the types being used cannot be known at compile time, but the programmer knowns the functionality they posses.
Calling a method will use a virtual dispatch of the method: that is, teh function pointer is loaded from the vtable, and is then invoked indirectly, incurring a pointer indirection.
The actual implemention of each vtable may vary on an object-by-object basis.

### 11.1.18. Impl trait types [↵](#tables-of-contents)

```
<impl-trait-type> := 'impl' <trait-bound> { '+' <trait-bound> }
```

An impl trait type introduces an unnamed generic parameter that implements the given intrefaces to the item it is used in.
It can appear in only 2 locations: function paramters (where it acts as an anonymous type of the parameter to the function) and function return types (where it acts as an abstract return type).

#### Anonymous type parameter

A function can use an impl trait type as the type of its parameter, where it declares the parameter to be of an anonymous type.
The caller must provide a type that statisfies the bounds declared in the anonymous type paramter, and the function can only use the functionality available through the trait bounds of the anonymous type paramter.

An example of this would be:
```
trait Trait {}

// Generic type parameter
fn with_generic_type<T is Trait>(param: T) {}

// impl trait typed paramter
fn with_impl_type(param: impl Trait) {}
```

This can be seens as synctactic sugar for a generic type paramter like `<T is Trait>`, except that the type is anonymous and does not appear within the generic argument list.

> _Note_: For function arguments, generic type parameters and `impl Trait` are not completely equivalent
> With a generic type paramter `<T is Trait>`, the caller is able to explicitly specify the type of the generic type parameter `T` when calling the function.
> If an `impl Trait` is used, the caller cannot ever specify the type of the parameter when calling the function.
>
> Therefore, changing between these types within a function signature should be considered a breaking change.

#### Abstract return types

A function can use an impl trait type as the type in its return type.
These types stand in for another concrete type wher the caller may only used the functinality declared by the specified traits.
Each possible return type of the function must resolve to the same concrete type.

An `impl Trait` in the return allows to return a abstract type that does not have to be stored within dynamic memory.
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

#### Abstract return types in trait declarations

Functions in traits may also return an abstract return types, this will create an anonymous associated type within the trait.

Evety `impl Trait` in the return type of an associated function in an trait is desugared to an anonymous associated type.
The return type that appears in teh implementation's funciton signature is used to determine the value of hte associated type.

##### Differences between generics and `impl Trait` in a return

When used as a type argument, `impl trait` work similar to the semantics of generic type parameters.
But when used in the return, there are significant changes, as unlike with a generic parameter where the caller can choose the return type, the implementation chooses the function's return type.

For example, the function
```
fn foo<T is Trait>() -> T { ... }
```
Allows the caller to determine the return type.

In contrast, the function
```
fn foo() -> impl Trait { ... }
```
doesn't allow the caller to explicitly determine the return type.
Instead the function chooses the return type, with the only guarantee that it implements the required traits.

#### Impl trait limitations

An impl trait type may only occur for non-`extern` functions.
It can also not be the type of a variable declaration, a field, or appear inside a type alias.

### 11.1.19. Record types [↵](#tables-of-contents)

```
<record-type> := '{' <record-members> '}'
<record-members> := <record-member> { ',' <record-member> }* [ ',' ]
<record-member> := { <attribute> }* <ext-name> { ',' <ext-name> }* ':' <type>
```

A record is a _structural_ type is that, similarly to a tuple, consisting out of a list of fields of other types.

But unlike a tuple, fields can be given explicit names, which can then be used to index the fields of the record.

### 11.1.20. Enum record types [↵](#tables-of-contents)

```
<enum-record> := 'enum' '{' <enum-fields> '}'
```

An enum record is a _structural_ type and is a variant of a record.

Unlike a record, it does not represent a collection of fields, but a type that is similar to that of an `enum`.
Access to enum members work essentially identical than those of an enum.

### 11.1.21. Inferred types [↵](#tables-of-contents)

```
<inferred-type> := '_'
```

An inferred type tell the compiler to infer the type (if possible) based on the surrounding information available.
Inferred types cannot be used in generic arguments.

Inferred types are often used to let the compiler infer the type of generic parameters:
```
TODO
```

## 11.2. Dynamically sized types [↵](#tables-of-contents)

Most types have a fixed size that is known at compile time and implements the `Sized` trait.
A type wit ha size tha is only known at compile-time is called a dynamically sized type (DST), or informally, unsized types.
Slices and trait objects are two such examples.

DSTs can only be used in certain cases:
- Pointers and references to DSTs are sized, but have twice the size of a pointer of a sized type.
    - Pointers to slices store the number of elements in the slice.
    - Pointers to trait objects store a pointer to their vtable.
- DSTs can be provided as type arguments to generic type parameters that have a special `?Sized` bound.
  They can also be used for associated type definitions when the corresponding associated type is declared using the `?Sized` bound.
  By default, any type parameter has a `Sized` bound, unless explicitly relaxed using `?Sized`
- Trait may be implemented for DSTs.
  Unlike with generic type paramters, `Self is ?Sized` is the default in trait definitions.
- Struct may contains a DST as the last field, this makes the struct itself a DST.

## 11.3. Nominal vs structural types [↵](#tables-of-contents)

Xenon has types that can either be nominal or structural, between these 2 kinds of types.

Both have the same type layout and mutability rules, but there are some important differences:

Nominal types:
- Nominal types do **not** implicitly implement any traits.
- Nominal types can have additional functionality and traits implemented.
- All field have configurable visibility.
- The types can be accessed directly from other scopes when 'imported'.

Structural types:
- Structural types implicitly implement a set of traits, depending on the values of the members, these are:
    - `Clone`
    - `Copy`
    - `PartialEq`
    - `Eq`
    - `Hash`
    - `Debug` _TODO: this will likely the be the trait, but depends on the standard format implementation._
- Structural types do not allow any additional functionality to be implemented, as they are strictly plain data types.
- Fields cannot have explicit visibility.
- The types only exist within the scope they are defined, unless publically aliased.


## 11.4. Type layout [↵](#tables-of-contents)

The layout of a type defines its size, alignment, and its internal representation of data/fields.
For enums, how their distriminant is laid out is also part of the layout.

Type layouts can change inbetween compilations.

### 11.4.1. Size and Alignment [↵](#tables-of-contents)

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
Meanwhile types that are not known at compile time, as known as [dynamically sized types](#112-dynamically-sized-types).

Since all values of a sized types share their size and alignment, we say that they have the type's size and alignment.

### 11.4.2. Primitive layout [↵](#tables-of-contents)

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

### 11.4.3. Unit and never type layout [↵](#tables-of-contents)

Unit and never types are both 0-sized types with an alignment of 1.

### 11.4.4. Pointer and reference layout [↵](#tables-of-contents)

Pointers and references have the same layout.
The mutabilty of a pointer or reference has not impact on the layout.

Pointers and references to sized tyes are the same as those of a `usize`.

Pointers and references to usized types are typed. Their size and alignement is guaranteed to be at least eqal to the size of a `usize` and have the same alignment.

> _Note_: Currently all pointers and references to DST are twice the size of a `usize` and have the same alignment.
> Although this should not be relied on.

### 11.4.5. Array layout [↵](#tables-of-contents)

An array of the form `[N]T` has a size that is `N` times that of the size of type `T` and has the same alignment as type `T`.
Arrays are laid out so that the zero-based `n`th element of the array is offset from the start of the array by `n` times the size of type `T`.

When an array is sentinal terminated, the array contains an additional element of type `T` at the end, so the size of the array will be `N + 1` times the size of type `T`.

### 11.4.6. Slice layout [↵](#tables-of-contents)

Slices have the same alyout as a section of an array

> _Note_: This is about the ray `[]T` type, not pointers to arrays to slices, e.g. (`&[N]T`)

### 11.4.7. String slice layout [↵](#tables-of-contents)

A string slice's layout depends on the type of string slice, but they have the same representation as their internal slice layout.

Below is a table of string slices that have a corresponding type layout to the following slice types

String slice | Slice
-------------|-------
`str`        | `[u8]`
`str7`       | `[char7]`
`str8`       | `[char8]`
`str16`      | `[char16]`
`str32`      | `[char32]`
`cstr`       | `[char8]`

### 11.4.8. Tuple layout [↵](#tables-of-contents)

Tuples are laid out as defined in the [Xenon representation]().

### 11.4.9. Trait object layout [↵](#tables-of-contents)

Trait objects have the same layout as the value the trait that implements it.

> _Note_: THis is for the trait object itself, not a type containing the object, such as a reference.

### 11.4.10. Closure layout [↵](#tables-of-contents)

A closure has no layout guarantees.

### 11.4.11. Bitfield layout [↵](#tables-of-contents)

A bitfield will have the size and alignment of the smallest primitive types that fits the contents of the bitfield.

### 11.4.12. Layout representation [↵](#tables-of-contents)

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

This representation can be applied to `struct`s, `enum`s, and `union`s.

The C representation also affects the alignment of primitive types for the current target architecture.

##### `repr(C)` structs and records

The alignment of a struct will be that of the most-aligned field.

The size of the type, and the size and offset of the fields will be determined uisng the method described below.

The current offset start at 0, then for each field within a type:
1. determine the size and alignment of the field
2. if the current offset is not a multiple of the field's alignment, set the current offset to the next multiple of the field's alignment. This space is padding.
3. the current offset will now become the offset for the field
4. increment the current offset by the size of the field.

> _Note_: This algorithm can produce zero-sized structs.
> While this is generally considered to be illegal in C, some compiler support option to enable zero-sized structs.
> Meanwhile C++ gives empty structures a size of 1, unless the are inherited or have fields using the `[[no_unique_address]]` attribute,
> in which case they do not contribute to the size of the overall struct.

##### `repr(C)` unions

A union with a C representaton has the same layout as the union would have if it were defined in C for the target platform.

The union will have the size of the largest fields in the union, and the alignment of the most-aligned field in the union.
These values may be taken from different fields.

##### `repr(C)` field-less enums and enum records, and flags enums

When an enum is field-less, the C representation has the size and alignment of the default `enum` size for the target platform's C ABI.

> _Note_: The enum representation in C is implementation defined, so this is really a "best guess".
> In particular, this may be incorrect when the C code of interst is compile with certain flags
> If a known enum size is required, use a primitive represention.

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

## 11.5. Interior mutability [↵](#tables-of-contents)

Sometimes a type needs to be mutated while having multiple aliases.
This can be achieved using a concept called _interior mutability_.
A type has interior mutability if its internal state can be modified from a shared reference to it.
This goes against the usual requirement that the value pointed to by a shared reference is not mutated.

`UnsafeCell<T>` is the only way of disabling this requirement.
When `UnsafeCell<T>` is immutably aliased, it is still safe to mutate or obtain a mutable reference to the `T` it contains.
As with all other types, it is undefined behavior to have multiple `&mut UnsafeCell<T>` aliases.

Other types with interior mutabiliity can be created using `UnsafeCell<T>` as a field.

> **Warning**: The programmer must ensure that this does not cause any unininted consequences or may cause other undefined behavior.

## 11.6. Type coercions [↵](#tables-of-contents)

Type coercions are implicit operations that change the type of a value.
They happen automatically at specific locations and are highly restricted in what types are allowed to coerce.

Any conversions allowed by coercion can als obe explicitly performed using the type cast operator `as`.

> _Note_: This description is informal and not yet fully defined, and should be more specific

### 11.6.1. Coercion sites [↵](#tables-of-contents)

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


### 11.6.2. Coecion types [↵](#tables-of-contents)

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

### 11.6.3. Unsized coercions  [↵](#tables-of-contents)

The following coercions arr called `unsized coercions`, since they relate to conversting sized types, and are permitted in a few cases where other coercions are not, as described above.
They can still happen anywhere a coercion can be done.

Two traits `Unsize` and `CoerceUnsized`, are used to assigst in this process and expose it for library use.
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

### 11.6.4. Least upper bound coercions [↵](#tables-of-contents)

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


## 11.7. Destructors [↵](#tables-of-contents)

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
    - Trait objects run the destructor of the underlying type
    - Other types don't result in any further drops

If a destructor must be run manually, such as when implementing a smart pointer, `drop_in_place` can be used.

### 11.7.1. Drop scopes [↵](#tables-of-contents)

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

### 11.7.2.  Scopes of function parameters [↵](#tables-of-contents)

All function paramters are in the scope of the entire function, so are dropped last when evaluating the function.
Each actual function parameter is dropped after any bindings introduced in that parameter's pattern.

_TODO: Example_

### 11.7.3. Scopes of local variables [↵](#tables-of-contents)

Local variables declared in a variable declaration are associated to the scope that contains the declaration.
Local variables declared in a `match` expression are associated to the arm scope of the `match` that they are declared in.

_TODO: Example_

If multiple patterns are used in the same arm of a `match` expressions, then an unspecified pattern will be used to determin the drop order.

### 11.7.4. Temporary scopes [↵](#tables-of-contents)

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
> Their drop scope is the entire function, as tehre is no smaller enclosing temporary scope.
>
> The scrutinee of a `match` expression is not a temporary scope, so temporaries in the scrutinee can be dropped after the `match` expression.
> For example, the temporary for `1` in `match 1 { ref mut z => z };` lives until the end of the statement.

_TODO: Example_


### 11.7.5. Operands [↵](#tables-of-contents)

Temporaries are also created to hold the result of operands to an expressions while the other operands are evaluated.
The temporaries are associated to the scope of the expressions with that operand.
Since the temporaries are moved from once the expreesssion is evaluated, dropping them has no effect unless one of the operands to an expression break out of he expression, returns, or panics.

_TODO: Example_

### 11.7.6. Constant promotion [↵](#tables-of-contents)

Promotion of a value expression to a `static` slot occurs when the expression could be written in a constant and borowed, and that borrow could be dereferenced where the exprssion was originally written, without changing the runtime behavior.
That is, the promoted expression can be evaluated at compile-time and the resulting value does not contain [interior mutability](#115-interior-mutability-) or [destructors](#117-destructors-) (these properties are determined based on the value when possible).

### 11.7.7. Temporary lifetime extension [↵](#tables-of-contents)

> _Note_: This is subject to change

The temporary scopes for expressions in variable declarations are sometimes extended to the scope of the block containing the declaration.
This is done wherer the usual temporary scope would be too small, based on syntactic rules.

If a borrow, dereference, field, or tuple expression has an extended temporary scope, the nteh indexed experssions also has an extended scope.

### 11.7.8. Extending based on patterns [↵](#tables-of-contents)

An extending pattern is either:
- An identifier pattern that binds by refernce or mutable reference.
- A struct, tuple, tuple struct, or slice pattern where at least one of the direct subpatterns in an extending pattern.

So `ref x`, `V(ref x)` and `[ref x, y]` are all extending patterns, but `x`, `&x` and `&(ref x, _)` are not.

If the pattern in a variable declaration is an extending pattern, then the temporary scope of the initializer expression is extended.

### 11.7.9. Extending based on expressions [↵](#tables-of-contents)

For a variable declaration with an initializer, an extending expression is an experssion whici is one of the following:
- The initializer expression.
- The operand of an extending borrow experssion.
- The operand of an extending array, cast, braced struct, or tuple expression.
- The final expression of any extending block expression.

So the borrow expression is `&mut 0`, `(&1, &mut 2)`, and `Some{ 0: &mut 3 }` are all extending expressions.
The borrows in `&0 + &1` and `Some(&mut 0)` are not: the latter is syntactically a function call expression.

The operand of any extending expression has its temporary scope extended.

### 11.7.10. Not running destructors [↵](#tables-of-contents)

`forget` can be used to prevent the destructor of a variable from being run, `ManuallyDrop` provides a wrapper to prevent a variable or field from being dropped automatically.

> _Note_: Preventing a destructor from being run via `forget` or other means is safe even if the type isn't static.
> Besides the place where destructors are guaranteed to run as defined by this document, types may not safely rely on a destructor being run for soundness.

# 12. Generics [↵](#tables-of-contents)
_TODO_

## 12.1. Type generics [↵](#tables-of-contents)

## 12.2. Value generics [↵](#tables-of-contents)

## 12.3. Paramter packs [↵](#tables-of-contents)

## 12.4. Contraints [↵](#tables-of-contents)

# 13. Macros [↵](#tables-of-contents)
_TODO_

# 14. Operators [↵](#tables-of-contents)
_TODO_

# 15. Precedence [↵](#tables-of-contents)
_TODO_

# 16. Attributes [↵](#tables-of-contents)
_TODO_

# 17. Implicit context [↵](#tables-of-contents)
_TODO_

# 18. Effect system [↵](#tables-of-contents)
_TODO_

# 19. Contracts [↵](#tables-of-contents)
_TODO_

# 20. ABI [↵](#tables-of-contents)
_TODO_