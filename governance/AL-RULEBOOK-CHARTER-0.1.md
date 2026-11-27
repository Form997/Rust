# AL Rulebook Charter

- Document ID: AL-GOV-RULEBOOK-CHARTER-001
- Version: 0.1.0
- Status: Foundational proposal; requires maintainer approval and external review
- Created: 2026-11-27
- Applies to: AL language, Rust core, local AI, datasets, training, evaluation, scientific packages, security, releases and market-facing claims

## 1. Foundational mandate

Every material AL decision MUST be checked against the current approved AL Rulebook, applicable law, verified standards and recorded evidence. A decision that conflicts with an approved mandatory rule, or lacks evidence required by that rule, MUST be stopped, documented and escalated before implementation or release.

This charter is a project governance policy. It does not itself create legal authority, national adoption, certification, or worldwide recognition.

## 2. Authority and precedence

Apply requirements in this order, resolving conflicts rather than silently choosing one:

1. Applicable law and binding regulatory obligations in each relevant jurisdiction.
2. Approved AL Rulebook requirements that do not conflict with applicable law.
3. Applicable, verified national and international standards and formally adopted technical specifications.
4. Verified scientific evidence and reproducible engineering results.
5. Approved project architecture and product requirements.
6. Schedule, cost and market preference.

Where sources conflict, record the conflict, jurisdiction, scope, version, legal/technical reviewer and resolution. A lower-level source MUST NOT override a higher-level binding requirement.

## 3. Mandatory decision gate

Every material decision MUST have a Decision Record containing:
- unique decision id, owner, date and status;
- affected subsystem, release and jurisdictions;
- applicable rule identifiers and exact rulebook versions;
- source references with issuing body, title, edition/version, publication/effective/review dates and stable official URL;
- evidence, assumptions, uncertainty and unresolved questions;
- impact on safety, security, privacy, accessibility, language/culture, data rights, performance, cost and market fit as applicable;
- verification method, acceptance criteria and test/evaluation evidence;
- approval, rejection, exception or stop rationale;
- migration, rollback and review date where applicable.

No implementation is considered compliant merely because a document mentions a standard. The relevant requirement must be mapped to a verifiable control and tested where testable.

## 4. Rule lifecycle and status vocabulary

Each rule MUST use one of these statuses:
- PROPOSED: under discussion; not binding.
- VERIFIED: source and applicability checked; not yet approved as AL policy.
- APPROVED: adopted as an AL project requirement within a stated scope.
- DEPRECATED: retained for history; replacement identified.
- WITHDRAWN: no longer applicable; rationale retained.
- BLOCKED: unresolved conflict, legal concern or evidence gap prevents use.

Only APPROVED rules are normative AL project requirements. VERIFIED does not mean globally recognized or legally mandatory.

## 5. Non-negotiable controls

- Unknown or untraceable training data MUST NOT enter an approved training run.
- AI-generated output MUST be treated as untrusted and validated by deterministic AL controls.
- Execution MUST be denied by default and require explicit, scoped capabilities.
- Every training/evaluation run MUST identify rulebook, dataset, code, configuration and artifact versions/checksums.
- Metrics MUST define their population, protocol, units/scales, aggregation, thresholds, uncertainty and failure conditions.
- Unsupported claims, invented thresholds and fabricated citations are prohibited.
- Updates MUST be traceable, reviewed, testable and reversible.
- If a mandatory gate fails, the affected activity MUST stop; unrelated safe work may continue only if it does not depend on the blocked decision.

## 6. Rulebook versioning

Use semantic versioning for the rulebook package:
- MAJOR: incompatible normative change or authority/precedence change.
- MINOR: new or materially expanded requirements without breaking existing contracts.
- PATCH: clarification or correction that does not change normative meaning.

Every release MUST include a change log, compatibility impact, migration guidance, source snapshots/references, approval record and integrity checksum. Rule identifiers remain stable; withdrawn identifiers are never silently reused.

## 7. International and Vietnam alignment

AL SHOULD map relevant requirements to authoritative international standards bodies and applicable Vietnamese laws, national standards (TCVN), technical regulations (QCVN) and competent authority guidance. The register MUST distinguish:
- international standard vs regulation vs treaty vs non-binding recommendation;
- published vs adopted vs legally applicable vs contractually required;
- original edition vs national adoption or translated edition;
- verified source vs candidate source.

AL MUST NOT claim that Vietnam has adopted a standard unless an official adoption record supports that claim. AL MUST NOT claim worldwide acceptance or certification without independent, verifiable evidence.

## 8. Accountability

The project maintainer approves AL policy. Named reviewers MUST be assigned for technical, data/model, security/privacy and legal/jurisdictional questions as relevant. A single person may temporarily cover multiple roles, but the decision record must disclose this and identify independent review still needed.

## 9. Enforcement

The rulebook is intended to become machine-readable and enforceable through:
- repository policy checks and required review templates;
- versioned schemas and validation tools;
- training/evaluation manifest gates;
- release checks and compliance reports;
- a dedicated Rulebook Center update workflow.

Until those controls are implemented and tested, this charter is a governance specification, not an automated enforcement claim.
