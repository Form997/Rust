# AL Rulebook Center — Architecture 0.1

- Document ID: AL-RULEBOOK-ARCH-001
- Version: 0.1.0
- Status: Proposed architecture; implementation pending
- Date: 2026-11-27

## 1. Purpose

The Rulebook Center is the authoritative project service for registering, validating, approving, publishing, applying and auditing AL rules. It governs AL Core, AI, training data, metrics, scientific packages, security controls and releases.

The Center is not an AI agent and must not silently rewrite policy. AI may discover candidate changes, summarize differences and prepare impact reports; authorized human maintainers approve normative changes.

## 2. Logical components

1. **Rule Registry:** stable rule IDs, text, scope, status, authority, version and relationships.
2. **Standards Register:** source organization, instrument type, edition, official publication, jurisdiction, adoption status and review dates.
3. **Requirement Mapper:** maps external clauses to AL rules, controls, tests and evidence.
4. **Metrics Registry:** canonical definitions, units/scales, protocols, thresholds, datasets and limitations.
5. **Decision Ledger:** immutable decision history, approvals, exceptions, conflicts and rationale.
6. **Update Monitor:** checks approved official source feeds/catalogues and maintainer-defined market/technology signals.
7. **Impact Analyzer:** identifies affected AL components, schemas, datasets, models, tests and releases.
8. **Validation Engine:** validates manifests, schema compatibility, required evidence and policy gates.
9. **Publisher:** creates signed/versioned rulebook bundles and release notes.
10. **User Interface:** dedicated “Update Rulebook” action with preview, diff, source evidence, impact and approval state.

## 3. Update lifecycle

`DISCOVER → FETCH → VERIFY SOURCE → CLASSIFY → MAP IMPACT → REVIEW → TEST → APPROVE → PUBLISH → APPLY → AUDIT`

- Discovery never equals approval.
- A failed source-integrity or applicability check blocks adoption.
- Draft updates run in a staging workspace and cannot alter active policy.
- Approval records identify the human approver and exact bundle checksum.
- Deployment is atomic: either the complete validated bundle becomes active or the previous version remains active.
- Rollback restores the prior approved bundle and records why it was reverted.

## 4. Dedicated update control

The product MUST expose a distinct action named **Update Rulebook** (Vietnamese UI: **Cập nhật Bộ quy tắc**). It MUST provide:
- last successful check and next scheduled check;
- current active rulebook version and source snapshot date;
- discovered candidate updates and official source links;
- source authenticity and applicability status;
- human-readable and machine-readable diffs;
- impacted rules, metrics, datasets, models, packages and releases;
- breaking-change and migration warnings;
- validation results and unresolved issues;
- explicit approve/apply control for authorized maintainers;
- rollback and audit history.

The button must never imply that a source was checked when no check occurred. Offline mode must show the last verified snapshot and clearly label it stale or current only relative to its recorded review policy.

## 5. Rulebook bundle

A published bundle SHOULD contain:
- manifest with bundle id, semantic version, creation time and checksum;
- rule registry and standards register;
- metrics definitions and evaluation protocols;
- requirement-to-control/test mappings;
- decision and approval records;
- deprecation and migration notices;
- source references and archived metadata;
- compatibility declarations and validation report.

The bundle MUST be usable offline after verified download. Signature and key-management design is a separate security task and must be reviewed before claiming authenticity guarantees.

## 6. Integration gates

- **AL compiler/core:** grammar, type, diagnostics, source and runtime rules are pinned to an approved compatible bundle.
- **AI interface:** prompts and output schemas cannot override normative rules; Rust validation is authoritative.
- **Training:** every run pins rulebook bundle, metric definitions, dataset manifests and code revision.
- **Evaluation:** metric version changes trigger comparability review; historical scores are not silently rewritten.
- **Scientific packages:** equations, parameters, units, validity ranges and reference tests map to relevant rules.
- **Release:** release pipeline verifies required approvals, source manifests, test evidence and known deviations.

## 7. Security and resilience

Treat remote documents, feeds, model outputs and community submissions as untrusted data. Require TLS, content checksums, source allowlists or trust policy, size limits, parser hardening, access control, audit logging and offline fallback. Never execute downloaded scripts as part of rulebook update. A malicious or compromised source must not be able to activate a rule.

## 8. Performance and availability

The Rulebook Center must not be a runtime dependency for ordinary AL execution. AL uses a locally cached, approved bundle. Update checks are asynchronous and bounded. Failure to reach an update source must not silently change active rules or disable deterministic core functionality.

## 9. Implementation sequence

- R0: governance documents and vocabulary.
- R1: JSON schemas, sample bundle and validator.
- R2: standards/source register and human review workflow.
- R3: CLI status/check/diff/validate commands.
- R4: dedicated UI action and signed bundle distribution.
- R5: CI/release gates and impact analysis.
- R6: scheduled monitoring, notifications and measured update SLAs.

No component above is implemented merely by this architecture document.
