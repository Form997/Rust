# AL Training and Evaluation Metrics Framework 0.1

- Document ID: AL-TRAIN-METRICS-001
- Version: 0.1.0
- Status: Framework proposal; metric thresholds are task-specific and not yet approved
- Date: 2026-11-27

## 1. Principle

No training or evaluation indicator may be used as a decision gate until its definition, population, protocol, unit/scale, aggregation, uncertainty, acceptance threshold, failure behavior and source rationale are recorded and approved.

There is no universal accuracy threshold suitable for every AL task. Do not invent a threshold to make a model appear ready.

## 2. Metric specification card

Every metric MUST define:
1. Stable metric ID and name.
2. Task and intended decision it supports.
3. Operational definition and mathematical formula.
4. Numerator, denominator, averaging and treatment of missing/abstained cases.
5. Unit or scale, direction of improvement and valid range.
6. Target population, sampling frame and dataset version.
7. Evaluation protocol, split strategy, leakage controls and reference implementation.
8. Confidence interval or uncertainty method where statistically appropriate.
9. Thresholds: pass, warning, fail, or explicitly NOT SET with rationale.
10. Subgroup/domain breakdowns and known blind spots.
11. Resource conditions: hardware, RAM, latency, throughput and energy/cost where relevant.
12. Provenance, citations, license and reviewer.
13. Metric version and compatibility notes.

## 3. Metric families

### 3.1 Language and compiler
- lexical correctness and span validity;
- parser acceptance/rejection accuracy against a frozen conformance corpus;
- diagnostic precision/recall and location correctness;
- semantic/type-check pass rates on valid and invalid programs;
- deterministic build and conformance results.

### 3.2 AI language assistance
- intent macro-F1 and per-class precision/recall;
- abstention coverage and selective risk;
- structured-output schema validity;
- AL parse and semantic validation pass rates;
- task success on held-out scenarios;
- unsupported-claim and fabricated-source rate under an explicit annotation protocol;
- harmful/unauthorized action rate and false-action rate.

### 3.3 Retrieval and evidence
- Recall@k, Precision@k, MRR and nDCG@k;
- citation/source-span correctness;
- answer support/faithfulness under a defined human or adjudicated protocol;
- stale-source and access-control violation rate.

### 3.4 Scientific and numerical systems
- MAE/RMSE or task-appropriate loss with units and target range;
- bias and calibration;
- residual, convergence and conservation checks;
- dimensional/unit consistency;
- uncertainty coverage and interval calibration;
- external validation across sites, seasons, instruments or populations where applicable.

### 3.5 Reliability, security and resources
- crash/error rate and recovery behavior;
- reproducibility across repeated runs;
- peak resident memory, startup time, latency and throughput;
- capability-policy violation rate;
- data/model provenance completeness;
- vulnerability and dependency findings with severity context.

## 4. Threshold governance

Thresholds MUST be justified by task risk, baseline performance, domain practice, user impact and validation data. A threshold may be:
- sourced from a verified standard or regulation;
- justified by a domain-specific validation study;
- selected as a project engineering target with explicit rationale and approval.

Label these origins distinctly. A project target MUST NOT be represented as an international standard. If no defensible threshold exists, mark NOT SET and block only the release/training decision that requires it.

## 5. Training-run gate

A run manifest MUST pin:
- rulebook bundle/version/checksum;
- metric cards and protocol versions;
- dataset manifest and split hashes;
- source-code commit and dependencies;
- model architecture/configuration and tokenizer;
- random seeds where applicable;
- hardware/software environment;
- run id, timestamps, logs and artifact checksums;
- evaluation report, subgroup results and reviewer decision.

## 6. Comparability and change

Metric definition, dataset, label policy, split, reference implementation or threshold changes may break historical comparability. New results MUST identify the changed component. Never silently recompute old results under a new definition or compare scores without a compatibility statement.

## 7. Initial status

This framework defines what must be specified. It does not approve any model, dataset, metric threshold, or training run. Initial AL task cards and metric cards must be created before the first model training experiment.
