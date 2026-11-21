# AL Language Specification — 0.1 Draft

**Status:** Initial design draft  
**Implementation:** Rust  
**Specification version:** 0.1.0

## 1. Purpose

AL is designed around the principle: humans write, AI understands, computers execute.

AL is a distinct language. Rust is used to implement its compiler and runtime; Rust syntax is not AL syntax.

## 2. Execution pipeline

The intended pipeline is:

1. AL source text (`.al`).
2. Lexer: source text to tokens.
3. Parser: tokens to abstract syntax tree (AST).
4. Semantic analyzer: name resolution, type checking, and rule validation.
5. AL intermediate representation (AL-IR).
6. Validator and capability/permission checks.
7. Runtime execution.

An AI interface may help produce or explain AL source, but it does not bypass any compiler or security stage.

## 3. Initial source structure

The initial design reserves these top-level declarations:

- `PROGRAM`
- `VERSION`
- `IMPORT`
- `DATA`
- `RULE`
- `FUNCTION`
- `TASK`
- `OUTPUT`
- `END`

The exact grammar and declaration semantics remain subject to refinement before the 0.1 specification is frozen.

## 4. Lexical conventions

- Keywords are uppercase ASCII words.
- Identifiers begin with an ASCII letter or underscore and may contain ASCII letters, digits, and underscores.
- String literals use double quotes.
- Supported string escapes in this implementation: `\\n`, `\\r`, `\\t`, `\\"`, and `\\\\`.
- A line comment begins with `//` and continues to the end of the line.
- A colon (`:`) introduces a declaration or block header.
- An equals sign (`=`) introduces an initial value.
- Newlines are retained as tokens.
- Whitespace indentation is currently informational; block structure is represented by keywords such as `END`.

## 5. Initial type vocabulary

The design reserves the following type names for later parser and semantic work:

- `TEXT`
- `INTEGER`
- `DECIMAL`
- `BOOLEAN`
- `LIST`
- `OBJECT`
- `NULL`
- `RESULT`
- `EVIDENCE`
- `CONFIDENCE`

These names are not yet enforced by a type checker. In particular, `CONFIDENCE` must not be treated as a calibrated probability unless its scale, interpretation, and calibration method are explicitly specified.

## 6. AI and semantic integrity

- The language specification, not an AI model's guess, defines program meaning.
- AI-produced source is untrusted input.
- AI output must pass the same lexer, parser, semantic analyzer, and policy checks as other source.
- An AI component cannot certify its own claims as verified merely by assigning a boolean flag.
- Facts, hypotheses, and inferences should remain distinguishable in future evidence-aware constructs.

## 7. Security requirements

The planned runtime must follow deny-by-default principles:

- No filesystem, network, process, or device access without explicit capabilities.
- Read and write permissions are separate.
- Sensitive operations require policy checks.
- AI components cannot grant themselves capabilities.
- Execution events and permission decisions are auditable.

These are design requirements; the runtime is not implemented in version 0.1.0.

## 8. Current implementation boundary

Version 0.1.0 implements lexical analysis only. It does not parse, type-check, validate program semantics, or execute AL programs. The sample program is therefore a lexer demonstration, not an executable AL application.

## 9. Compatibility

Changes to token meanings or lexical rules must be documented in the changelog and accompanied by tests. Breaking changes require a specification-version decision.
