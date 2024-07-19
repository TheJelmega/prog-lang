# Introduction

This file contains both the notation which is used to define the langauge gramar and a collection of all grammar definitions within the project.

# Notation

The notation is a custom notation defined below.
The notation is based on the base BNF (Backus-Naur Form).

Notation           | Meaning                | Example
-------------------|------------------------|---------
`'...'` or `"..."` | Literal or keyword     | `"foo"`
`<...>`            | Symbol                 | `<symbol-name>`
`... := ...`       | Assignment             | `<symbol> := "val"`
`... ...`          | Concatination          | `"foo" "bar"`
`... \| ...`       | Alternation            | `"foo" \| "bar"`
`(...)`            | Grouping               | `("foo")`
`[...]`            | Optional               | `["foo"]`
`{...}*`           | 0 or more repetitions  | `{"foo"}*`
`{...}+`           | 1 or more repetitions  | `{"foo"}+`
`{...}[N,M]`       | N to M repetitions     | `{"foo"}[1,6]`
`{...}[,M]`        | at most M repetitions  | `{"foo"}[,6]`
`{...}[N,] `       | at least N repetitions | `{"foo"}[1,]`
`... - ...`        | Range (inclusive)      | `'a'-'z'`
`? ... ?`          | Custom definition      | `? any unicode codepoint ?`

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

<name> := <name-char> { <letter> | <number> | '_' }*
<ext-name> := ( <name-char> | <number> ) { <letter> | <number> | '_' }*

<escape-code> := '\0'
               | '\t'
               | '\n'
               | '\r'
               | '\"'
               | "\'"
               | '\\'
               | '\x' <hex-digit> <hex-digit>
               | '\u{' { <hex-digit> }[1,6] '}'
               
<literal> := <numeric-literals>
           | <boolean-literals>
           | <character-literal>
           | <string-literals>

<digit_sep> := "_"
<numeric-literals> := <int-decimal-literal>
                    | <float-decimal-literal>
                    | <binary-literal>
                    | <octal-literal>
                    | <int-hexadecimal-literal>
                    | <float-hexadecimal-literal>

<int-dec-literal> := [ '-' ] { <dec-digit> }+
<float-dec-literal> := [ '-' ] { <dec-digit> }+ [ '.' { <dec-digit> }+ ] [ ( 'e' | 'E' ) [ '-' ] { dec-digit }+ ]
<bin-literal> := "0x" <bin-digit> [ { <bin-digit> | <digit-sep> }[,126] <bin-digit> ]
<oct-literal> := "0o" <oct-digit> [ { <oct-digit> | <digit-sep> }[,41] <oct-digit> ]
<int-hex-literal> := "0o" <hex-digit> [ { <hex-digit> | <digit-sep> }[,30] <hex-digit> ]
<float-hex-literal> := [ '-' | '+' ] "0x" ( "1." | "0." ) <hex-digit> { <hex-digit> | <digit-sep> } 'p' [ '-' | '+' ] { <dec-digit> }[,4]

<bool-literal> := 'true' | 'false'

<character-literal> := "'" ( ? any unicode codepoint, except for \ and ' ? | <escape-code> ) "'"

<string-literal> := <regular-string-literal> | <raw-string-literal>
<regular-string-literal> := '"' { ? any valid unicode codepoint, except for \ and '"' ? | ? string continuation sequence ? | <escape-code> }* '"'
<raw-string-literal> := 'r' { '#' }[N] { ? any valid unicode codepoint ? }* '"' { '#' }[N]









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

<parenthesized-type> := '(' <type> ')'
<primitive-type> := <unsigned-type>
                  | <signed-type>
                  | <floating-point-type>
                  | <boolean-type>
                  | <character-type>
                  
<unsigned-type> := 'u8' | 'u16' | 'u32' | 'u64' | 'u128'
<signed-type> := 'i8' | 'i16' | 'i32' | 'i64' | 'i128'
<floating-point-type> := 'f16' | 'f32' | 'f64' | 'f128'
<boolean-type> := 'bool' | 'b8' | 'b16' | `b32' | 'b64'
<character-type> := 'char' | 'char7' | 'char8' | 'char16' | 'char32'

<unit-type> := '(' ')'
<never-type> := '!'
<path-type> := <type-path>
<tuple-type> := '(' <type> { ',' <type> }+ [ ',' ] ')'
<array-type> := '[' <expr> [ ';' <expr> ] ']' <type>
<slice-type> := `[` ';' <expr> `]` <type>
<string-slice-type> := 'str' | 'str7' | 'str8' | 'str16' | 'str32' | 'cstr'
<pointer-type> := ( '*' | '[' '*' [ ';' <expr> ] ']' ) ( 'const' | 'mut ) <type>
<reference-type> := `&` [ 'mut' ] <type>
<optional-type> := '?' <type>

<fn-type> := [ 'unsafe' [ 'extern' <abi> ] ] 'fn' '(' <fn-type-params> ')' [ '->' <type-no-bounds> ]
<fn-type-params> := <fn-type-param> { ',' <fn-type-param> }* [ ',' ]
<fn-type-param> := { <attribute> }* [ ( <ext-name> | '_' ) { ',' ( <ext-name> | '_' ) }* ':' ] <type>

<interface-object-type> := 'dyn' <interface-bound> { '+' <interface-bound> }*
<impl-interface-type> := 'impl' <interface-bound> { '+' <interface-bound> }

<record-type> := '{' <record-members> '}'
<record-members> := <record-member> { ',' <record-member> }* [ ',' ]
<record-member> := { <attribute> }* <ext-name> { ',' <ext-name> }* ':' <type>
<enum-record> := 'enum' '{' <enum-fields> '}'

<inferred-type> := '_'
```