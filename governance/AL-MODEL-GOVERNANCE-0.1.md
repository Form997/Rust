# AL Model Governance Specification

- Document ID: AL-GOV-001
- Version: 0.1.0
- Status: Draft for legal and engineering review
- Created: 2026-11-27
- Applies to: AL Local AI datasets, training runs, model artifacts, and distribution

## 1. Purpose and governing philosophy

AL is developed from accumulated human knowledge, independent engineering, and community participation. It aims to expand access to technology, respect cultural and linguistic diversity, encourage innovation, and protect the lawful interests of creators and other rights holders.

Project philosophy:

**Open Knowledge – Controlled Weights – Community Driven**

- **Open Knowledge:** publish the language specification, public documentation, protocols, examples, and source components under clearly selected licenses.
- **Controlled Weights:** distribute model weights, checkpoints, and selected operational assets under an explicit access and license policy.
- **Community Driven:** accept proposals, code, tests, and datasets with review, provenance, and appropriate rights information.

Controlled distribution is a project policy. It does not create a legal exemption or make an otherwise unlawful dataset or model lawful.

## 2. Scope

This specification governs:

- data collection, acquisition, storage, annotation, and preprocessing;
- dataset licensing, provenance, and personal-data review;
- model architecture experiments, training, fine-tuning if separately approved, and evaluation;
- checkpoints, weights, tokenizers, adapters, and exported model formats;
- internal access, external release, and third-party distribution;
- incident response, audit records, and removal or restriction requests.

It does not replace jurisdiction-specific legal advice or the terms of third-party licenses.

## 3. Core requirements

### GOV-01 — No untraceable training data

Every dataset component used in a release training run MUST have a recorded source, acquisition method, date, version or snapshot identifier, and rights assessment.

Data with unknown origin MUST NOT enter a production training run.

### GOV-02 — No unauthorized model artifacts

The project MUST NOT incorporate third-party weights, adapters, tokenizers, or model components unless their license and applicable terms have been reviewed and approved for the intended use.

The selected project direction is to train AL-owned weights from scratch. This does not remove the need to review the rights in training data, code, tokenizer resources, and other dependencies.

### GOV-03 — Dataset provenance

Each dataset MUST have a manifest recording, where applicable:

- dataset identifier and version;
- source organization, author, or URL;
- collection or acquisition date;
- license or other legal basis;
- permitted purposes and restrictions;
- geographic/jurisdictional considerations;
- personal-data assessment;
- transformations, filtering, and annotation history;
- responsible reviewer and approval status;
- cryptographic checksum for the released snapshot.

### GOV-04 — Reproducible training records

Each training run MUST record:

- source-code commit;
- dataset manifest versions and checksums;
- model configuration and tokenizer version;
- training configuration and hyperparameters;
- random seed where applicable;
- software and hardware environment;
- start/end time and run identifier;
- evaluation report and resulting artifact checksums.

### GOV-05 — Rights review before distribution

Before releasing weights or other model artifacts, the project MUST review the relevant training-data rights, dependency licenses, contractual restrictions, personal-data risks, and intended distribution terms.

The review MUST record unresolved risks and the decision owner. A model trained from scratch is not automatically free of third-party rights issues.

### GOV-06 — Personal data protection

Personal data MUST be collected and processed only where an appropriate lawful basis and purpose exist. The project SHOULD minimize collection, retention, and exposure; protect access; and maintain procedures for data-subject requests where applicable.

Sensitive personal data and confidential information MUST NOT be intentionally added to training corpora without a documented necessity and legal/security approval.

### GOV-07 — Transparency

The project SHOULD maintain an understandable summary of training-data categories, data selection methods, known limitations, model capabilities, and evaluation results. Where law requires a more specific transparency disclosure, the project MUST assess and meet the applicable requirement.

### GOV-08 — Controlled access and export

Access to private datasets, checkpoints, and controlled weights MUST be role-limited and logged. Export and redistribution MUST follow the artifact's assigned tier and license.

### GOV-09 — Security and integrity

Datasets and model artifacts MUST be checksum-verified. Training and release pipelines SHOULD use access controls, dependency review, malware scanning, and reproducible build records appropriate to the risk.

### GOV-10 — Community contributions

Community-provided code, data, translations, and evaluations MUST include contributor identity or attribution as appropriate, license/permission information, provenance, and a clear contribution review outcome. A public upload does not itself grant permission to use the material for training.

## 4. Distribution tiers

### AL PUBLIC

May include, subject to an explicit license:

- language specification and public documentation;
- public protocol schemas;
- examples and tutorials;
- selected source code and developer tooling;
- public evaluation methodology and reports.

### AL CONTROLLED

May include:

- model weights and checkpoints;
- internal or restricted datasets;
- private evaluation sets;
- sensitive operational configurations;
- security-sensitive implementation details.

Access conditions, permitted uses, redistribution rights, and revocation conditions MUST be stated in the applicable license or access agreement.

### AL COMMUNITY

May include:

- proposals and issue discussions;
- reviewed source contributions;
- properly licensed datasets;
- translations and test cases;
- community evaluation results.

Community status is not a substitute for license review or quality control.

## 5. Legal review framework

The project MUST assess applicable law based on the actual countries, users, data sources, processing activities, and distribution channels. International instruments and regional laws are not one universal rulebook.

The following are reference points for legal review, not blanket permissions:

1. **Berne Convention and TRIPS:** international copyright framework and national implementation.
2. **EU Directive 2019/790:** text-and-data-mining provisions, including Articles 3 and 4, subject to their conditions and applicable reservations.
3. **EU AI Act, Regulation (EU) 2024/1689:** obligations for relevant providers of general-purpose AI models, including Article 53 where applicable; role, model category, timing, and exceptions must be assessed.
4. **GDPR, Regulation (EU) 2016/679:** personal-data principles and lawful-basis requirements where processing falls within its territorial scope.
5. **United States copyright law and Copyright Office guidance:** training, outputs, and model-related questions remain fact- and jurisdiction-dependent.
6. **UNESCO Recommendation on the Ethics of AI:** non-binding ethical guidance on human rights, transparency, accountability, and diversity.

Official reference links:

- WIPO: https://www.wipo.int/
- EU DSM Directive 2019/790: https://eur-lex.europa.eu/eli/dir/2019/790/oj
- EU AI Act 2024/1689: https://eur-lex.europa.eu/eli/reg/2024/1689/oj
- GDPR 2016/679: https://eur-lex.europa.eu/eli/reg/2016/679/oj
- UNESCO AI ethics recommendation: https://www.unesco.org/en/artificial-intelligence/recommendation-ethics
- U.S. Copyright Office: https://www.copyright.gov/ai/

Legal conclusions MUST be reviewed for the relevant jurisdiction and facts. The list above is not exhaustive and does not establish that any particular training corpus or model release is lawful.

## 6. Operational workflow

1. Propose a dataset, dependency, model artifact, or release.
2. Record origin, license, purpose, restrictions, and intended use.
3. Perform rights, privacy, security, and quality review.
4. Approve, reject, or restrict the proposed material.
5. Add approved inputs to versioned manifests.
6. Run training and evaluation with recorded configurations.
7. Review model behavior, memorization/privacy risks, and release suitability.
8. Assign a distribution tier and license.
9. Publish the required documentation and retain audit evidence.
10. Monitor reports and respond to valid rights, privacy, and security requests.

## 7. Roles and accountability

At minimum, the project SHOULD assign these responsibilities, even if one person initially holds multiple roles:

- **Project maintainer:** approves policy and release decisions.
- **Dataset steward:** maintains provenance and data quality records.
- **Model maintainer:** manages training, evaluation, and artifact integrity.
- **Security reviewer:** assesses access, sandbox, and release risks.
- **Legal reviewer:** reviews jurisdiction-specific rights and compliance questions.

Approval records MUST identify the responsible person or role and the date.

## 8. Incident and request handling

The project SHOULD maintain a documented channel and process for:

- copyright or license complaints;
- privacy and personal-data requests;
- confidential-data exposure;
- model artifact compromise;
- dataset poisoning or provenance failure;
- unsafe or materially misleading model behavior.

Responses should preserve relevant audit evidence, restrict affected assets when necessary, assess impact, and document corrective actions. Removal from a dataset does not automatically remove learned influence from an existing model; retraining or other mitigation may be necessary and must be evaluated case by case.

## 9. Suggested repository records

```text
governance/
  README.md
  policies/
    AL-MODEL-GOVERNANCE-0.1.md
  datasets/
    manifest.schema.json
    provenance/
  training/
    training-manifest.schema.json
    runs/
  models/
    registry.schema.json
    registry.json
    checksums/
  security/
    access-policy.md
    export-policy.md
  audit/
    decisions/
    incidents/
```

The schema files and records above are proposed artifacts; they are not claimed to exist until implemented and reviewed.

## 10. Initial risk register

| Risk | Initial control | Evidence required |
|---|---|---|
| Unknown training-data origin | Reject untraceable data | Dataset manifest and review |
| Incompatible third-party license | License gate before use/release | Dependency and rights review |
| Personal or confidential data leakage | Minimization, access control, testing | Privacy assessment and test report |
| Model memorization or reproduction | Evaluation and mitigation | Documented test cases and outcomes |
| Unauthorized model export | Tiered access and export logging | Access and release logs |
| AI-generated invalid AL | Mandatory Rust validation | Validator test results |
| Unsafe execution | Deny-by-default sandbox | Security and capability tests |
| Misleading performance claims | Publish benchmark conditions | Reproducible evaluation report |

## 11. Review and change control

This document is a versioned project policy draft. Changes MUST be reviewed and recorded. Legal references and applicability MUST be rechecked before each material distribution decision and when relevant law changes.

## 12. Project declaration

> AL được xây dựng từ tri thức nhân loại, phát triển bằng sáng tạo độc lập và hướng đến phục vụ con người. AL tôn trọng sự đa dạng văn hóa, thúc đẩy khả năng tiếp cận công nghệ, khuyến khích đổi mới và bảo vệ quyền lợi hợp pháp của mọi chủ thể. Công nghệ không thuộc riêng một nhóm người; tri thức và cơ hội phát triển cần được mở rộng cho cộng đồng.
