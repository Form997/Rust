# Changelog

All notable changes to this project are documented here.

## [0.1.0] - 2026-11-21

### Added
- Initialized the AL compiler project using Rust and Cargo.
- Added the initial token model and source spans.
- Added a lexer for AL keywords, identifiers, string literals, numeric literals, punctuation, newlines, and line comments.
- Added lexical diagnostics for malformed input.
- Added a command-line entry point for token inspection.
- Added the first AL example and the AL 0.1 draft specification.
- Added unit tests for lexer behavior.

### Scope
This is a lexer bootstrap. Parsing, semantic analysis, AL-IR, runtime execution, and AI integration are not included yet.
