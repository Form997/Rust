# H1.2 — AL 0.1 Grammar Lock

**Status:** Proposed parser contract for review  
**Target language version:** AL 0.1.0  
**Depends on:** H1.1 source spans and diagnostics  
**Normative boundary:** This document freezes only the conservative core demonstrated by the AL 0.1 specification and `examples/hello.al`. It does not invent syntax for reserved declarations whose forms have not yet been specified.

## 1. Purpose and compatibility rule

H1.2 defines a deterministic, line-oriented grammar target before parser/AST implementation begins. The grammar is intentionally smaller than the complete future AL language.

The existing AL 0.1 design names `PROGRAM`, `VERSION`, `IMPORT`, `DATA`, `RULE`, `FUNCTION`, `TASK`, `OUTPUT`, and `END` as reserved top-level words. A reserved word is not, by itself, a defined grammatical construct. Only the core forms in this document are admitted by the H1.2 parser profile.

Any expansion of this grammar must be recorded as a specification change and accompanied by conformance fixtures and tests. The parser must not guess what an unsupported declaration means.

## 2. Source and lexical model

### 2.1 Character encoding and positions

- Source files are UTF-8.
- Token spans use zero-based UTF-8 byte offsets with an exclusive end.
- Line endings accepted by the source layer are LF, CRLF, and bare CR. CRLF is one logical newline.
- Newline is syntactically significant between statements.
- Horizontal indentation consists of ASCII space and tab. Indentation is informational; it does not create nested scopes.
- A line comment starts with `//` and continues up to, but does not consume, the line ending.

### 2.2 Identifiers and keywords

- An identifier starts with ASCII `A-Z`, `a-z`, or underscore.
- Remaining identifier characters are ASCII letters, ASCII digits, or underscore.
- Keywords are uppercase ASCII and case-sensitive.
- Identifiers are therefore case-sensitive: `count` and `Count` are distinct spellings.
- Vietnamese and other Unicode text is supported inside string literals and comments, but Unicode identifiers are not part of AL 0.1.

### 2.3 Literals

- Strings use double quotes and cannot contain a physical line ending.
- Supported escapes are `\\n`, `\\r`, `\\t`, `\\"`, and `\\\\`.
- Numbers contain one or more ASCII digits, optionally followed by a decimal point and zero or more ASCII digits. A leading decimal point is not allowed.
- A number is retained lexically as its source spelling. Numeric range, precision, exponent notation, and interpretation belong to later semantic/numeric specifications.

### 2.4 Punctuation

The core grammar uses only colon `:` and equals `=` as punctuation. Other punctuation is rejected in this profile unless a later specification explicitly admits it.

## 3. Core grammar (EBNF)

The following grammar describes the H1.2 core profile. `NEWLINE` is a logical line ending token; `EOF` is the end of source.

```ebnf
source          = { blank_line }, program, { blank_line }, EOF ;

program         = program_header, NEWLINE,
                  { blank_line },
                  version_header, NEWLINE,
                  { blank_line },
                  [ data_block, { blank_line } ],
                  output_statement,
                  { blank_line } ;

program_header  = "PROGRAM", horizontal_space, string ;
version_header  = "VERSION", horizontal_space, string ;

data_block      = "DATA", ":", NEWLINE,
                  { blank_line },
                  data_declaration,
                  { NEWLINE, { blank_line }, data_declaration },
                  NEWLINE, "END" ;

data_declaration
                = identifier, ":", type_name,
                  [ horizontal_space, "=", horizontal_space, literal ] ;

output_statement
                = "OUTPUT", horizontal_space, identifier ;

type_name       = identifier ;
literal         = string | number ;

identifier      = identifier_start, { identifier_continue } ;
identifier_start
                = "A"…"Z" | "a"…"z" | "_" ;
identifier_continue
                = identifier_start | "0"…"9" ;

number          = digit, { digit }, [ ".", { digit } ] ;
digit           = "0"…"9" ;

string          = '"', { string_character | escape }, '"' ;
escape          = "\\n" | "\\r" | "\\t" | '\\"' | "\\\\" ;

blank_line      = { horizontal_space }, NEWLINE ;
horizontal_space
                = (" " | "\\t"), { " " | "\\t" } ;
```

### 3.1 Grammar interpretation notes

1. A source contains exactly one `PROGRAM` header and exactly one `VERSION` header, in that order.
2. A `DATA` block is optional in this conservative profile. If present, it contains at least one declaration and is closed by `END`.
3. `OUTPUT` is required and currently accepts one identifier only.
4. A data declaration has an identifier, a type-name identifier, and an optional initializer. An initializer is a string or numeric literal in this profile.
5. The type vocabulary in `AL-0.1.md` is reserved for semantic work. The grammar accepts a type-name identifier; semantic analysis will decide whether that name is a supported type.
6. Blank lines are permitted between top-level forms and data declarations. A statement cannot be continued across physical lines.
7. A final line ending after `OUTPUT` is optional. The source may end immediately after the statement.
8. Comments are removed by lexical analysis; a comment-only line behaves as a blank line.
9. Whitespace around `:` is not admitted by the conservative grammar. Whitespace around `=` is admitted; the lexer already discards horizontal whitespace, so the parser sees the same token sequence either way. The implementation should not make indentation semantically meaningful.

## 4. Reserved but not yet grammatical

The following words remain reserved tokens but are not accepted as declarations/statements by the H1.2 core grammar:

- `IMPORT`
- `RULE`
- `FUNCTION`
- `TASK`

Their syntax, nesting rules, ordering, scope, parameters, return values, and semantics require separate design decisions. The parser must emit an explicit unsupported/unexpected-construct diagnostic rather than silently skipping them.

## 5. Explicit non-goals for AL 0.1

The H1.2 core grammar does not define:

- expressions or operator precedence;
- arithmetic, comparison, boolean, or assignment statements;
- arrays, lists, objects, records, or nested blocks;
- function parameters, return statements, or function calls;
- imports, modules, package resolution, or namespaces;
- rules, tasks, loops, branches, or concurrency;
- Unicode identifiers;
- multiline strings, interpolation, or additional escape forms;
- scientific quantities, units, uncertainty, evidence, or confidence syntax;
- execution semantics or permissions.

These are not rejected as ideas; they are simply not part of this frozen parser target.

## 6. Canonical valid example

```al
PROGRAM "Hello AL"
VERSION "0.1.0"

DATA:
    message: TEXT = "Xin chao Hai Dat"
END

OUTPUT message
```

This example is a grammar fixture, not proof that AL currently parses or executes it. The current implementation boundary remains lexical analysis until the parser and later stages are implemented and tested.

## 7. Required conformance cases for H1.2/H1.3

The parser test suite must include at least:

### Accept
- the canonical example above;
- LF, CRLF, and bare-CR line endings;
- blank lines between headers, data declarations, and output;
- comments between statements;
- UTF-8 Vietnamese content inside strings;
- declarations with and without an initializer;
- integer-looking and decimal-looking number spellings.

### Reject
- missing or duplicate `PROGRAM` header;
- missing or duplicate `VERSION` header;
- headers in the wrong order;
- missing `OUTPUT` or more than one output statement;
- missing `END` for a data block;
- empty data block;
- malformed declaration missing identifier, colon, or type name;
- unsupported top-level reserved constructs;
- unknown punctuation and malformed literals;
- unexpected trailing tokens after the output statement.

A test case must assert both acceptance/rejection and, where applicable, a stable diagnostic code and source span. Tests must not assert terminal caret display columns until display-width rendering is specified.

## 8. Decisions still requiring explicit review

The following are conservative choices made to keep the first parser deterministic, not claims that the full AL language must always remain this small:

- whether `DATA` should be mandatory;
- whether a program may contain multiple `OUTPUT` statements;
- whether output should accept expressions rather than a single identifier;
- whether initializers should accept booleans and null in 0.1;
- whether type names should be restricted to the reserved type vocabulary;
- whether optional whitespace around colons should be accepted;
- whether a final newline should be required.

Until these decisions are reviewed, parser implementation should follow this document as the H1.2 baseline and isolate any change behind tests rather than broadening the grammar ad hoc.
