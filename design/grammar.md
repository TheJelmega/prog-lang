# Introduction

This file contains both the notation which is used to define the langauge gramar and a collection of all grammar definitions within the project.

# Notation

The notation is a custom notation defined below.
The notation is based on the base BNF (Backus-Naur Form).

Notation           | Meaning               | Example
-------------------|-----------------------|---------
`'...'` or `"..."` | Literal or keyword    | `"foo"`
`<...>`            | Symbol                | `<symbol-name>`
`... := ...`       | Assignment            | `<symbol> := "val"`
`... ...`          | Concatination         | `"foo" "bar"`
`... \| ...`       | Alternation           | `"foo" \| "bar"`
`(...)`            | Grouping              | `("foo")`
`[...]`            | Optional              | `["foo"]`
`{...}*`           | 0 or more repetitions | `{"foo"}*`
`{...}+`           | 1 or more repetitions | `{"foo"}+`
`{...}[N,M]`       | N to M repetitions    | `{"foo"}[1,6]`
`... - ...`        | Range (inclusive)     | `'a'-'z'`
`? ... ?`          | Custom definition     | `? any unicode codepoint ?`

Whenever a sequence can be both captured by the current of next element within the notation, it is assumed the next element will take up consume the sequence.
Meaning that the sequence `{ 'a' | 'b' }* 'a'` and `{ 'b' }* 'a'` represent the same sequence, this is done to prevent abiguity during parsing.

A set of concatinated tokens can imply the presence of a whitespace in between the tokens in the following cases:
- tokens starting or ending on a letter
- tokens starting or ending on a number, when they contain at least 1 letter
- tokens starting or ending on a `_`, when they contain at least 1 letter or number

# Full language grammar

```
<byte-order-marker> := "\xEF\xBB\xBF"
<shebang> := '#!' ? any character other than '\n' ? '\n'

<file> := [ <byte-order-marker> ] [ <shebang> ] ...

<letter> := ? any unicode character with the letter category (L) ?
<number> := ? any unicode character with the number category (L) ?
<name-char> := <letter> | '_'

<bin-digit> := '0' | '1'
<oct-digit> := '0'-'7'
<dec-digit> := '0'-'9'
<hex-digit> := <dec-digit> | 'a'-'z' | 'A'-'Z'
<digit-sep> := "_" | "'"

<bin-literal> := "0b" <bin-digit> [ { <bin-digit> | <digit-sep> }* <bin-digit> ]
<oct-literal> := "0o" <oct-digit> [ { <oct-digit> | <digit-sep> }* <oct-digit> ]
<hex-literal> := "0x" <oct-digit> [ { <oct-digit> | <digit-sep> }* <oct-digit> ]
<int-dec-literal> :=  <dec-digit> [ { <dec-digit> | <digit-sep> }* <dec-digit> ]
<fp-dec-literal> := <int-dec-literal> '.' <int-dec-literal> [ 'e' [ '-' ] <int-dec-literal> ]

<name> := <name-char> { <letter> | <number> | '_' }*
<ext-name> := ( <name-char> | <number> ) { <letter> | <number> | '_' }*


<escape-code> := '\0'
               | '\t'
               | '\n'
               | '\r'
               | '\"'
               | '\''
               | '\\'
```