# Xenon language design

## Tables of contents

1. [Introduction](#1-introduction)
    - [This document is provisional](#this-document-is-provisional)
2. [Lexical structure](#2-source-code-representation)
    1. [Input format](#21-input-format)
3. [Lexical structure](#3-lexical-structure)
    1. [Whitespace](#31-whitespace)
    2. [Keywords](#32-keywords)
    3. [Comments](#33-comments)
4. [Package Stucture](#4-package-structure)
    1. [Packages](#41-packages)
    2. [Artifacts](#42-artifacts)
    3. [Modules](#43-modules)

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
If a file does not contain a valid utf-8 sequence, this will result in an error

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
<shebang> := '#!' ? any character other than '\n' ? '\n'
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

_Note: This is **not** a direct mapping to the unicode separator category `Z`_

_Note: While newline sequences count as whitespace, they are handled separately, see [Newline sequences](#23-newline-sequences)._

## 3.2. Keywords

Keywords represent names within the code that have a special meaning in the language, such as declaring a function.

There are 3 types of keywords:
- strong
- reserved
- weak

### Strong keywords

A strong keyword is a keyword that always has a meaning, regardless of where in the code it is located, and can therefore not be used for anything else
A list of strong keywords can be found below:
```
```

### Reserved keywords

A reserved keyword is keyword that is not currently used, but has been set aside as not being possible to be used by the programmer for future use.
A list of reserved keywords can be found below:
```
async
await
yield
```

### Weak keywords

A weak keyword is a keyword that is dependent on the surrounding context and can be used anywhere outside
A list of strong keywords can be found below:
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
_TODO: is module the correct name?_

### Examples
_TODO_

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

# 6. Types

# 7. Items

# 8. Statements

# 9. Expressions

# 10. Generics

# 11. Macros