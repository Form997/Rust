# AL Language

**AL** is an experimental programming-language project with the design goal:

> Humans write it. AI understands it. Computers execute it.

This repository contains the Rust implementation of the AL toolchain. Rust is the implementation language; AL has its own syntax, grammar, type system, and execution semantics.

## Current status

**Version:** 0.1.0 — lexer bootstrap

Implemented:
- Rust Cargo project structure.
- Initial AL token model with source spans.
- Lexer for the AL 0.1 keyword set, identifiers, strings, numbers, punctuation, newlines, and line comments.
- Lexical errors for unsupported characters, malformed escapes, and unterminated strings.
- Command-line entry point that reads an `.al` file and prints tokens.
- Unit tests for core lexer behavior.

Not implemented yet:
- Parser and AST.
- Semantic analysis and type checking.
- AL intermediate representation (AL-IR).
- Runtime and execution engine.
- AI intent/interface layer.

No program is executed by this version. The current CLI performs lexical analysis only.

## Requirements

- Rust toolchain with Cargo.
- Rust edition 2024 support.

## Run

```bash
cargo test
cargo run -- examples/hello.al
```

## Initial AL example

```al
PROGRAM "Hello AL"
VERSION "0.1.0"

DATA:
    message: TEXT = "Xin chao Hai Dat"
END

OUTPUT message
```

The example is currently a source-language design sample. Version 0.1.0 tokenizes it; it does not yet parse or execute it.

## Design principles

1. AL semantics are defined by the language specification, not improvised by an AI model.
2. AI-generated or AI-interpreted AL must pass the same parser, semantic checks, and permission checks as human-authored AL.
3. Execution is denied by default until a capability is explicitly granted.
4. Facts, hypotheses, inferences, and evidence must remain distinguishable.
5. The language specification and implementation are versioned independently and tested together.

## Project layout

- `specification/` — language specification.
- `src/token.rs` — token and source-span model.
- `src/lexer.rs` — lexical analyzer.
- `src/main.rs` — command-line entry point.
- `examples/` — AL source examples.
- `tests/` — integration tests.
- `docs/AL-LOCAL-AI-ARCHITECTURE-0.1.md` — AL Local AI architecture draft.
- `docs/H0.1-SCIENTIFIC-COMPUTING-AND-LOCAL-AI-REVIEW.md` — H0.1 review scope: data structures, algorithms, mathematics, scientific computing, learning methods, local LLM/LLMs, and RAG.
- `governance/AL-MODEL-GOVERNANCE-0.1.md` — model/data governance draft.

## Roadmap

| Version | Target |
|---|---|
| 0.1.x | Lexer and lexical test suite |
| 0.2.x | Parser, AST, syntax diagnostics |
| 0.3.x | Semantic analysis and type checking |
| 0.4.x | AL intermediate representation |
| 0.5.x | Sandboxed runtime and permission model |
| 0.6.x | AI interface with validated intent mapping |
| 1.0.0 | Stable specification, compiler, runtime, and conformance tests |

## License

MIT. See [LICENSE](LICENSE).


## H0.2.1 — Rulebook validation implementation

- `governance/bundles/al-rulebook-0.1.0.proposed.json` — proposed example bundle (not approved).
- `governance/fixtures/invalid-rulebook-missing-required.json` — negative validation fixture.
- `.github/workflows/rulebook-schema.yml` — CI checks schema meta-structure, accepts the valid fixture and rejects the invalid fixture.
- `governance/H0.2.1-VALIDATION-IMPLEMENTATION.md` — scope and limitations.

The workflow is committed but must run in GitHub Actions before its result can be called passing. Structural schema validation is not standards verification or legal compliance.

## H0.2 — AL Rulebook governance foundation (draft)

The H0.2 branch establishes the proposed governance layer before further compiler and AI implementation:

- `governance/AL-RULEBOOK-CHARTER-0.1.md` — decision authority, stop conditions, rule lifecycle and versioning.
- `governance/AL-RULEBOOK-ARCHITECTURE-0.1.md` — Rulebook Center and dedicated update workflow.
- `governance/AL-STANDARDS-REGISTER-0.1.md` — verified-source register; candidate families are not yet approved requirements.
- `governance/AL-TRAINING-METRICS-FRAMEWORK-0.1.md` — mandatory metric specification and training gates.
- `governance/AL-VIETNAM-ADOPTION-PATH-0.1.md` — evidence-based path for Vietnam alignment and international review.
- `governance/schemas/al-rulebook-bundle.schema.json` — machine-readable bundle manifest.

These are proposed governance artifacts on the H0.2 branch. They are not yet approved as binding project policy, externally certified, or automatically enforced. No standard is claimed as adopted by Vietnam or accepted worldwide without official evidence.

## H0.1 technical completion pack

- docs/H0.1-COMPLETION-REPORT.md — completion report, inspected source boundary and deferred work.
- docs/H0.1-TECHNICAL-BASELINE.md — data structures, algorithms, numerical/scientific computing, ML, local AI, RAG, hardware plan and H1 backlog.
- docs/H0.1-ARCHITECTURE-DECISIONS.md — architecture decision records and evidence references.
- schemas/h0-1-data-contracts.schema.json — versioned interchange data contracts.

H0.1 design/review is complete. This does not mean parser, runtime, scientific solvers, local models or RAG have been implemented.


## H0.2.2 — Governance record contracts (draft)

H0.2.2 adds machine-readable JSON Schema contracts and example fixtures for:

- `governance/schemas/metric-card.schema.json` — metric definition, formula, unit, scale, population, measurement protocol, aggregation, uncertainty, threshold state and evidence.
- `governance/schemas/source-record.schema.json` — source provenance, instrument type, jurisdiction, applicability, official URL and Vietnam adoption status.
- `governance/schemas/decision-record.schema.json` — decision context, alternatives, rule references, evidence, impact, rollback and approval.
- `governance/schemas/training-run-manifest.schema.json` — pinned Rulebook/metric versions, dataset provenance and checksums, code commit, configuration, seeds, environment and artifacts.

The CI workflow validates every schema meta-structure, all positive fixtures and deliberate negative fixtures. A schema pass proves structural conformity only; it does not prove a source is authentic, a metric scientifically valid, dataset use lawful, or a decision approved. Example records remain proposed/unverified/evaluation-only. No threshold is invented.
