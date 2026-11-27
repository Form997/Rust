# AL Local AI Architecture Specification

- Document ID: AL-AI-ARCH-001
- Version: 0.1.0
- Status: Draft for engineering review
- Created: 2026-11-27
- Scope: Local-first AI assistance integrated with the AL programming language
- Normative language: MUST, MUST NOT, SHOULD, MAY are used as requirement terms.

## 1. Purpose

AL follows the design goal:

> Humans write it. AI understands it. Computers execute it.

AL Local AI assists people in expressing intent, understanding AL programs, generating candidate AL source, explaining diagnostics, and proposing corrections. It does not replace the AL language specification or the authoritative language implementation.

This document records the agreed architecture direction. It is not evidence that the components described here have already been implemented.

## 2. Architectural principles

1. **One language authority:** the versioned AL language specification is the single source of truth for syntax and semantics.
2. **Deterministic validation:** Rust components decide whether AL source is lexically, syntactically, semantically, and operationally valid.
3. **AI is an assistant, not an authority:** AI output is an untrusted candidate until validated.
4. **Local-first:** core language tooling MUST work without a network connection.
5. **Weak-to-strong hardware:** the system MUST provide a useful rule-based/lightweight mode on constrained computers and allow larger models on stronger hardware.
6. **Human control:** execution and sensitive capabilities require explicit permissions.
7. **Traceability:** datasets, model artifacts, evaluations, and releases MUST carry provenance and version information.
8. **Cultural and linguistic inclusion:** Vietnamese everyday language, technical Vietnamese, regional expressions, English, and additional languages should be supported without treating one culture as the universal default.

## 3. System boundaries

### 3.1 Rust authority layer

Rust owns language correctness and execution:

- lexer and token stream;
- parser and abstract syntax tree (AST);
- semantic analysis and type checking;
- diagnostics and source spans;
- intermediate representation (IR), when specified;
- compiler and runtime/virtual machine;
- sandbox, capability checks, and execution policy;
- versioned request/response protocol.

An AI model MUST NOT bypass these components or directly authorize execution.

### 3.2 Python AI layer

Python owns model and data experimentation:

- intent understanding and task routing;
- AL/Vietnamese training-data preparation;
- tokenizer and model experiments;
- training and evaluation pipelines;
- local inference adapters;
- model packaging and quantization experiments.

Python is not the authority for AL parsing, type rules, or execution semantics.

### 3.3 Initial bridge

The first bridge SHOULD use JSON Lines over standard input/output because it is inspectable, language-neutral, and straightforward to test.

- One JSON object per line.
- Standard output is reserved for protocol messages.
- Diagnostic logs go to standard error.
- Every request and response includes a protocol version and request identifier.
- Invalid messages return structured errors; they must not crash the language process.

PyO3, ONNX Runtime, or another embedded interface MAY be evaluated later, but only after profiling demonstrates a concrete benefit.

## 4. Logical workflow

1. Receive a user request or AL source.
2. Classify the task: explain, generate, diagnose, transform, or ask for clarification.
3. Produce a candidate response or candidate AL source.
4. Send candidate source to the Rust validation pipeline.
5. Return diagnostics and evidence to the AI layer.
6. If invalid, explain the errors and optionally propose a bounded correction.
7. Present the validated result to the user.
8. Execute only after explicit user authorization and Rust capability checks.

The AI layer MUST NOT silently execute generated code.

## 5. Suggested repository boundaries

The following is a target architecture, not a claim that these crates or packages already exist.

### Rust workspace

- `al-core`: shared identifiers, versions, and source abstractions.
- `al-lexer`: lexical analysis.
- `al-ast`: AST data structures.
- `al-parser`: syntax analysis.
- `al-diagnostics`: structured diagnostics and source locations.
- `al-semantic`: name and semantic validation.
- `al-types`: type system and type checking.
- `al-ir`: intermediate representation, after its design is approved.
- `al-compiler`: compilation pipeline.
- `al-runtime`: execution engine.
- `al-sandbox`: capability-limited execution.
- `al-ai-protocol`: versioned bridge messages.
- `al-cli`: command-line interface.

The existing repository may retain its current layout during incremental migration. Do not perform a broad crate split until the current code and tests have been reviewed.

### Python package

- `al_ai/data`: dataset manifests and preprocessing.
- `al_ai/intent`: task classification and intent structures.
- `al_ai/generation`: candidate AL generation.
- `al_ai/training`: reproducible training entry points.
- `al_ai/evaluation`: held-out tests and metrics.
- `al_ai/inference`: local model loading and inference adapters.
- `al_ai/protocol`: JSONL bridge client and schemas.

## 6. Protocol baseline

Protocol name: `AL-AI`; initial protocol version: `1`.

Illustrative request:

```json
{
  "protocol": "AL-AI/1",
  "request_id": "example-001",
  "task": "generate",
  "language_version": "0.1",
  "locale": "vi-VN",
  "input": {
    "text": "Tạo chương trình in lời chào"
  },
  "constraints": {
    "allow_execution": false
  }
}
```

Illustrative response:

```json
{
  "protocol": "AL-AI/1",
  "request_id": "example-001",
  "status": "candidate",
  "candidate_source": "PROGRAM \"Hello\"",
  "validation": {
    "performed": false,
    "diagnostics": []
  }
}
```

These examples define shape only. The exact schema, required fields, error codes, size limits, and compatibility policy MUST be formalized before the bridge is treated as stable. A response marked `candidate` is not validated code.

## 7. Hardware profiles

### AL Local Lite

Target: constrained CPU-only devices, including systems with approximately 4 GB RAM.

- Rust lexer/parser/validator remain the reliable core.
- Rule-based intent routing and templates provide baseline assistance.
- Optional compact models must be measured for RAM, latency, and quality before being enabled.
- Model loading must be optional; language tooling must remain useful when no model is installed.
- No claim is made that a general-purpose LLM can train or run effectively on this hardware.

### AL Development / Training

Target: stronger workstations or rented compute when necessary.

- Dataset preparation, experiments, training, and evaluation run here.
- Training artifacts are versioned and accompanied by manifests.
- Models are evaluated before export to Lite.
- Quantized or distilled artifacts are tested on the actual target hardware.

## 8. Training strategy

The selected project direction is to develop AL-owned model weights from scratch. This is a governance and product direction, not a promise that a large foundation model can be trained with currently available hardware.

Recommended staged progression:

1. Deterministic rules and curated examples.
2. Small task-specific classifier or structured prediction baseline.
3. Small generation experiments only when a measurable dataset and evaluation set exist.
4. Scale model size only when compute, data rights, and evaluation justify it.

Training data MUST be versioned, licensed or otherwise lawfully usable, and traceable to its sources. Training, validation, and test splits MUST be separated to reduce leakage.

## 9. Safety and security requirements

- AI output is untrusted input.
- All AL source passes the Rust validation pipeline.
- Execution is denied by default.
- The sandbox enforces explicit capabilities and resource limits.
- File, network, process, and device access are unavailable unless separately authorized.
- Protocol inputs have size limits and strict schema validation.
- Model and dataset files are integrity-checked.
- Secrets and credentials MUST NOT be placed in prompts, datasets, logs, or model artifacts.
- Errors must fail closed: an unavailable validator is not permission to execute.

## 10. Evaluation

Every release candidate SHOULD report:

- lexer/parser pass rate on a versioned conformance corpus;
- semantic and type-check pass rate;
- candidate validity rate after Rust validation;
- task success on a held-out Vietnamese/AL benchmark;
- unsupported-request abstention rate;
- correction success rate;
- false-acceptance rate (target: zero for invalid programs reaching execution);
- peak RAM, model size, and latency on each declared hardware profile;
- regression results against the previous release.

Metrics must include test-set size, dataset version, software commit, model checksum, and evaluation configuration. Do not publish a percentage without its denominator and test conditions.

## 11. Roadmap and completion gates

| Stage | Scope | Exit evidence |
|---|---|---|
| H0 | Architecture and specification alignment | Reviewed language spec, boundaries, and decisions |
| H0.1 | Compare current repository with this architecture | Inventory of implemented, missing, and conflicting items |
| H1 | Lexer/parser/AST/diagnostics and semantic foundation | Automated tests and conformance examples |
| H2 | IR/compiler/runtime/sandbox | Deterministic execution tests and security tests |
| H3 | Python AI package and Rust bridge | Protocol contract tests and failure handling |
| H4 | Provenance-tracked datasets | Dataset manifests and rights review |
| H5 | Model baselines and training experiments | Reproducible runs and held-out evaluation |
| H6 | Lite/Development profiles | Measured resource and quality benchmarks |
| H7 | Release hardening | Regression, security, documentation, and release checklist |

A stage is complete only when its exit evidence exists. A design document alone does not complete an implementation stage.

## 12. Open decisions

The following remain subject to review before implementation is locked:

- exact AL grammar and AST schema, aligned with the existing AL 0.1 specification;
- canonical type system and diagnostic codes;
- exact JSONL schemas and protocol compatibility rules;
- IR and runtime design;
- target Lite RAM and latency budgets;
- model architecture, tokenizer, and training corpus;
- supported model formats and distribution controls.

## 13. Change control

Changes to syntax or semantics require a versioned language-specification change and conformance tests. Changes to AI protocol require a protocol versioning decision. Model and dataset changes require updated manifests and evaluation reports.

## 14. Current implementation status

At the time this document was created, the repository README identifies AL 0.1.0 as a lexer bootstrap. The parser, AST, semantic/type checking, IR, runtime, and AI interface are listed as not yet implemented there. This architecture document does not change that status.
