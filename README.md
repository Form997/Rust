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


## H0.2.3 — Rust Rulebook Validator (draft)

The Rust CLI now has a separate Rulebook validation path while preserving the existing AL source lexer invocation:

```sh
cargo run -- rulebook validate metric-card governance/fixtures/metric-card.proposed.json
cargo run -- rulebook validate source-record governance/fixtures/source-record.unverified.json
cargo run -- rulebook validate decision-record governance/fixtures/decision-record.proposed.json
cargo run -- rulebook validate training-run-manifest governance/fixtures/training-run-manifest.evaluation-only.json
```

Supported record kinds: `bundle`, `metric-card`, `source-record`, `decision-record`, and `training-run-manifest`. The validator returns a non-zero process exit code for errors, prints warnings separately, and includes unit tests for valid fixtures and rejected governance states.

The Rust validator enforces selected governance invariants: an approved metric threshold needs a value and evidence IDs; verified sources need HTTPS URLs and a review date; approved decisions need an approved approval state, approver and timestamp; training/fine-tuning requires an approved Rulebook, verified dataset-rights status and approved governance review. Unverified sources remain warnings rather than being silently treated as authoritative.

JSON Schema CI remains the structural contract check. Rust validation is an additional semantic gate, not a substitute for official-source verification, cryptographic checksum verification, legal review, scientific validation or human approval. It does not execute AL programs or grant permissions.


## Rulebook Authority

- H0.2.4 adds Ed25519 verification for human-approved Rulebook envelopes. See [`governance/H0.2.4-CORE-AUTHORITY.md`](governance/H0.2.4-CORE-AUTHORITY.md).
- Verify a signed envelope with `cargo run -- rulebook verify-signed envelope.json trusted-keys.json`.
- The example trusted-key registry is intentionally empty. Production trust roots must be anchored outside the mutable repository; this phase does not yet implement production key custody or bundle activation.


## H0.2.5 — Root of Trust and Rulebook Update Center

- [`governance/H0.2.5-ROOT-OF-TRUST-UPDATE-CENTER.md`](governance/H0.2.5-ROOT-OF-TRUST-UPDATE-CENTER.md) defines the offline root trust chain and update lifecycle.
- `al rulebook verify-registry` verifies the root-signed signer registry against a SHA-256 fingerprint supplied out-of-band.
- `al rulebook compare` produces an informational top-level JSON change review.
- `al rulebook verify-update` verifies the pinned root, signer authorization, human approval, current-parent hash and strictly increasing semantic version.
- No production private key or root fingerprint is stored in this repository. This phase does not yet activate updates or provide a graphical UI.


## H0.2.6 — Rulebook Enforcement Core

- [`governance/H0.2.6-ENFORCEMENT-CORE.md`](governance/H0.2.6-ENFORCEMENT-CORE.md) documents the fail-closed subsystem policy gates and integration requirements.
- Seven gates are defined: `compiler`, `runtime`, `ai`, `training`, `evaluation`, `scientific`, and `release`.
- `cargo run -- rulebook gate <gate> <signed-envelope.json> <signed-registry.json> <pinned-root-fingerprint-hex>` verifies the external root pin, human-signed Rulebook and at least one approved in-scope rule with explicit control IDs.
- Missing or unapproved policy denies authorization. The central gate engine is implemented; actual compiler/runtime/AI/training/evaluation/scientific/release module wiring remains pending until those modules exist.


## H0.2.7 — Control Evidence Binding

- `governance/schemas/control-evidence.schema.json` defines a verifiable control record: gate, implementation and verification states, owner, scope, timestamp, test IDs and evidence IDs.
- `src/control_evidence.rs` resolves every control ID referenced by an approved in-scope rule to a matching evidence record.
- The `rulebook gate` CLI now denies authorization if the signed Rulebook lacks the catalogue or any referenced control is not implemented, verified, complete or scoped to the requested gate.
- CI validates positive and negative control-evidence fixtures; unit tests exercise missing catalogue, unresolved IDs, unverified state, wrong gate and missing tests/evidence.

Evidence records are signed assertions and references, not proof that a production module actually invokes the gate or that referenced CI artifacts exist. Actual subsystem integration and independent artifact/deployment attestation remain separate required work.
