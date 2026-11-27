# AL Standards and Regulatory Sources Register 0.1

- Document ID: AL-STANDARDS-REGISTER-001
- Version: 0.1.0
- Status: Register template; entries require source-by-source verification
- Date: 2026-11-27

## 1. Purpose

Maintain a traceable inventory of external standards, laws, regulations, treaties, recommendations and technical specifications that may affect AL.

This register is not a claim that every listed instrument applies to AL, is adopted in Vietnam, or is legally mandatory. Applicability depends on scope, jurisdiction, project role, release date and facts.

## 2. Required fields for every entry

| Field | Requirement |
|---|---|
| source_id | Stable identifier |
| issuing_body | Official issuing organization/authority |
| title | Exact official title |
| instrument_type | Law, regulation, treaty, standard, technical specification, recommendation, guidance |
| identifier_edition | Official number and edition/version |
| jurisdiction | International, regional, national or sectoral |
| publication_date | Official date, if available |
| effective_date | Effective date, if applicable |
| status | Current, amended, withdrawn, superseded, draft, unknown |
| official_url | Canonical official source |
| source_snapshot | Retrieval date and archived checksum/reference |
| adoption_status_vn | Adopted, partially adopted, not adopted, not applicable, unverified |
| applicability | Reasoned AL scope and role assessment |
| mapped_rules | AL rule IDs and control/test mappings |
| reviewer | Named technical/legal reviewer or role |
| reviewed_at | Last verification date |
| next_review | Due date or trigger |
| confidence_and_gaps | Uncertainty and unresolved matters |

## 3. Candidate source families to investigate

These are discovery categories, not pre-approved requirements:
- ISO and IEC standards relevant to software engineering, quality, security, AI management, risk, data governance and testing.
- IEEE standards and recommended practices relevant to software, AI and systems engineering.
- W3C and IETF specifications relevant to web, Unicode-related interoperability, identity, protocols and accessibility.
- NIST publications and frameworks relevant to cybersecurity, AI risk and measurement.
- UNESCO recommendations and other intergovernmental non-binding guidance on AI ethics, education, language and cultural diversity.
- WIPO instruments and official copyright guidance relevant to content rights and model artifacts.
- Vietnamese laws, decrees, circulars, TCVN and QCVN relevant to software, AI, cybersecurity, data, privacy, intellectual property and sector-specific deployments.
- Domain-specific scientific standards and metrology guidance for units, measurements, uncertainty and test methods.

## 4. Source verification rules

- Prefer the issuing body's official publication or legal database.
- Search-engine snippets, blogs, vendor marketing and third-party summaries are discovery aids only.
- Verify title, identifier, edition, status, dates, scope and amendments against the official source.
- Record paywall or access limitations; do not fabricate clause text.
- Do not reproduce copyrighted standards beyond permitted quotation. Record clause references and licensed access status.
- Distinguish a standard's existence from AL's conformity to it.
- A standard can be adopted as a voluntary AL engineering baseline without being legally mandatory; label that distinction.
- Vietnam adoption must be verified through competent Vietnamese official sources, including the relevant standards/technical-regulation catalogue or legal instrument.

## 5. Initial register state

At creation, no external standard in this file is marked APPROVED as an AL requirement. The candidate families above require research, applicability mapping, technical review and, where relevant, legal review. This avoids converting a name-drop list into a false compliance claim.

## 6. Review triggers

Review entries when an issuing body publishes a new edition/amendment, a law changes, Vietnam adopts or withdraws a national equivalent, AL changes product role or deployment geography, a major incident exposes a gap, or market requirements materially change.
