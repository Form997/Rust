use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordKind { Bundle, MetricCard, SourceRecord, DecisionRecord, TrainingRunManifest }

impl RecordKind {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "bundle" => Some(Self::Bundle),
            "metric-card" => Some(Self::MetricCard),
            "source-record" => Some(Self::SourceRecord),
            "decision-record" => Some(Self::DecisionRecord),
            "training-run-manifest" => Some(Self::TrainingRunManifest),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationReport { pub errors: Vec<String>, pub warnings: Vec<String> }

impl ValidationReport {
    fn new() -> Self { Self { errors: Vec::new(), warnings: Vec::new() } }
    pub fn is_valid(&self) -> bool { self.errors.is_empty() }
}

fn is_object(value: &Value, path: &str, report: &mut ValidationReport) -> bool {
    if value.is_object() { true } else {
        report.errors.push(format!("{path}: expected an object"));
        false
    }
}
fn required_string(value: &Value, key: &str, path: &str, report: &mut ValidationReport) -> Option<String> {
    match value.get(key).and_then(Value::as_str) {
        Some(s) if !s.trim().is_empty() => Some(s.to_owned()),
        _ => { report.errors.push(format!("{path}.{key}: required non-empty string is missing")); None }
    }
}
fn enum_field(value: &Value, key: &str, allowed: &[&str], path: &str, report: &mut ValidationReport) -> Option<String> {
    let text = required_string(value, key, path, report)?;
    if allowed.contains(&text.as_str()) { Some(text) }
    else { report.errors.push(format!("{path}.{key}: unsupported value {text}")); None }
}
fn required_array<'a>(value: &'a Value, key: &str, path: &str, report: &mut ValidationReport) -> Option<&'a Vec<Value>> {
    match value.get(key).and_then(Value::as_array) {
        Some(items) => Some(items),
        _ => { report.errors.push(format!("{path}.{key}: expected an array")); None }
    }
}

fn validate_metric(value: &Value, report: &mut ValidationReport) {
    let path = "metric";
    if !is_object(value, path, report) { return; }
    for key in ["schema_version","metric_id","version","name","purpose","definition","unit"] { required_string(value,key,path,report); }
    enum_field(value,"status",&["proposed","verified","approved","deprecated","withdrawn","blocked"],path,report);
    enum_field(value,"domain",&["language","compiler","ai","retrieval","scientific","numeric","reliability","security","resource","other"],path,report);
    let Some(threshold) = value.get("threshold") else { report.errors.push("metric.threshold: required object is missing".into()); return; };
    if !is_object(threshold,"metric.threshold",report) { return; }
    let state = enum_field(threshold,"state",&["NOT_SET","proposed","approved"],"metric.threshold",report);
    let evidence = required_array(threshold,"evidence_ids","metric.threshold",report);
    if state.as_deref()==Some("approved") {
        match threshold.get("value") {
            None | Some(Value::Null) => report.errors.push("metric.threshold.value: approved threshold requires a value".into()),
            Some(Value::String(s)) if s.trim().is_empty() => report.errors.push("metric.threshold.value: approved threshold cannot be blank".into()),
            _ => {}
        }
        if evidence.is_none_or(|items| items.is_empty()) {
            report.errors.push("metric.threshold.evidence_ids: approved threshold requires at least one evidence ID".into());
        }
    }
    if state.as_deref()==Some("NOT_SET") && threshold.get("value").is_some_and(|v| !v.is_null()) {
        report.warnings.push("metric.threshold: NOT_SET threshold carries a value; review state/value consistency".into());
    }
}

fn validate_source(value: &Value, report: &mut ValidationReport) {
    let path="source";
    if !is_object(value,path,report) { return; }
    for key in ["schema_version","source_id","issuing_body","title","jurisdiction"] { required_string(value,key,path,report); }
    enum_field(value,"instrument_type",&["law","regulation","treaty","standard","technical_specification","recommendation","guidance","research","other"],path,report);
    let verification=enum_field(value,"verification_status",&["unverified","verified_official_source","blocked"],path,report);
    enum_field(value,"adoption_status_vn",&["adopted","partially_adopted","not_adopted","not_applicable","unverified"],path,report);
    let url=required_string(value,"official_url",path,report);
    if url.as_deref().is_some_and(|s| !s.starts_with("https://")) {
        report.errors.push("source.official_url: official source URL must use HTTPS".into());
    }
    if verification.as_deref()==Some("verified_official_source") {
        if url.is_none_or(|s| !s.starts_with("https://")) { report.errors.push("source.verification_status: verified source requires HTTPS URL".into()); }
        if value.get("reviewed_at").is_none_or(Value::is_null) { report.errors.push("source.reviewed_at: verified source requires review date".into()); }
    } else if verification.as_deref()==Some("unverified") {
        report.warnings.push("source: authenticity and current legal status have not been verified".into());
    }
    if value.get("adoption_status_vn").and_then(Value::as_str)==Some("unverified") {
        report.warnings.push("source: Vietnam adoption status is unverified".into());
    }
}

fn validate_decision(value: &Value, report: &mut ValidationReport) {
    let path="decision";
    if !is_object(value,path,report) { return; }
    for key in ["schema_version","decision_id","title","context","decision","rulebook_version"] { required_string(value,key,path,report); }
    let status=enum_field(value,"status",&["proposed","under_review","approved","rejected","superseded","revoked"],path,report);
    let Some(approval)=value.get("approval") else { report.errors.push("decision.approval: required object is missing".into()); return; };
    if !is_object(approval,"decision.approval",report) { return; }
    let state=enum_field(approval,"state",&["not_requested","pending","approved","rejected","revoked"],"decision.approval",report);
    if status.as_deref()==Some("approved") {
        if state.as_deref()!=Some("approved") { report.errors.push("decision.approval.state: approved decision requires approved approval state".into()); }
        if approval.get("approver").and_then(Value::as_str).is_none_or(|s| s.trim().is_empty()) { report.errors.push("decision.approval.approver: approved decision requires named approver".into()); }
        if approval.get("approved_at").is_none_or(Value::is_null) { report.errors.push("decision.approval.approved_at: approved decision requires timestamp".into()); }
    }
    if status.as_deref()==Some("proposed") && state.as_deref()==Some("approved") {
        report.errors.push("decision: proposed decision cannot carry approved approval state".into());
    }
}

fn validate_training(value: &Value, report: &mut ValidationReport) {
    let path="training_run";
    if !is_object(value,path,report) { return; }
    required_string(value,"schema_version",path,report);
    required_string(value,"run_id",path,report);
    let run_type=enum_field(value,"run_type",&["training","fine_tuning","evaluation","data_preparation","benchmark"],path,report);
    let Some(rulebook)=value.get("rulebook") else { report.errors.push("training_run.rulebook: required object is missing".into()); return; };
    if !is_object(rulebook,"training_run.rulebook",report) { return; }
    required_string(rulebook,"version","training_run.rulebook",report);
    required_string(rulebook,"bundle_checksum","training_run.rulebook",report);
    let rulebook_status=enum_field(rulebook,"status",&["approved","proposed","evaluation_only"],"training_run.rulebook",report);
    let datasets=required_array(value,"dataset",path,report);
    let governance=value.get("governance_review");
    let governance_state=governance.and_then(|g| enum_field(g,"state",&["pending","approved","rejected","blocked"],"training_run.governance_review",report));
    if matches!(run_type.as_deref(),Some("training")|Some("fine_tuning")) {
        if rulebook_status.as_deref()!=Some("approved") { report.errors.push("training_run.rulebook.status: training/fine-tuning requires approved Rulebook".into()); }
        if governance_state.as_deref()!=Some("approved") { report.errors.push("training_run.governance_review.state: training/fine-tuning requires approved governance review".into()); }
        if let Some(items)=datasets {
            for (index,dataset) in items.iter().enumerate() {
                if dataset.get("license_status").and_then(Value::as_str)!=Some("verified") {
                    report.errors.push(format!("training_run.dataset[{index}].license_status: training requires verified dataset rights"));
                }
                if dataset.get("provenance_record").and_then(Value::as_str).is_none_or(|s| s.trim().is_empty()) {
                    report.errors.push(format!("training_run.dataset[{index}].provenance_record: provenance record is required"));
                }
            }
        }
    } else if rulebook_status.as_deref()!=Some("approved") {
        report.warnings.push("training_run: non-training run uses unapproved Rulebook; not release evidence".into());
    }
    if datasets.is_some_and(Vec::is_empty) { report.errors.push("training_run.dataset: at least one dataset record is required".into()); }
}

fn validate_bundle(value: &Value, report: &mut ValidationReport) {
    let path="bundle";
    if !is_object(value,path,report) { return; }
    for key in ["schema_version","bundle_id","rulebook_version","created_at"] { required_string(value,key,path,report); }
    let status=enum_field(value,"status",&["proposed","verified","approved","deprecated","withdrawn","blocked"],path,report);
    let approval=value.get("approval");
    let approval_state=approval.and_then(|a| enum_field(a,"state",&["not_requested","pending","approved","rejected","revoked"],"bundle.approval",report));
    if status.as_deref()==Some("approved") {
        if approval_state.as_deref()!=Some("approved") { report.errors.push("bundle.approval.state: approved bundle requires approved approval state".into()); }
        for (i,source) in value.get("sources").and_then(Value::as_array).into_iter().flatten().enumerate() {
            if source.get("verification_status").and_then(Value::as_str)!=Some("verified_official_source") {
                report.errors.push(format!("bundle.sources[{i}]: approved bundle cannot include unverified source"));
            }
        }
        for (i,rule) in value.get("rules").and_then(Value::as_array).into_iter().flatten().enumerate() {
            if rule.get("status").and_then(Value::as_str)!=Some("approved") {
                report.errors.push(format!("bundle.rules[{i}]: approved bundle cannot include non-approved rule"));
            }
        }
    }
}

pub fn validate(kind: RecordKind, value: &Value) -> ValidationReport {
    let mut report=ValidationReport::new();
    match kind {
        RecordKind::Bundle=>validate_bundle(value,&mut report),
        RecordKind::MetricCard=>validate_metric(value,&mut report),
        RecordKind::SourceRecord=>validate_source(value,&mut report),
        RecordKind::DecisionRecord=>validate_decision(value,&mut report),
        RecordKind::TrainingRunManifest=>validate_training(value,&mut report),
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(text:&str)->Value { serde_json::from_str(text).expect("fixture JSON") }

    #[test] fn accepts_unset_threshold_metric() {
        let v=fixture(include_str!("../governance/fixtures/metric-card.proposed.json"));
        assert!(validate(RecordKind::MetricCard,&v).is_valid());
    }
    #[test] fn rejects_approved_metric_without_value_and_evidence() {
        let v=fixture(include_str!("../governance/fixtures/invalid-metric-card-approved-without-threshold.json"));
        let r=validate(RecordKind::MetricCard,&v);
        assert!(!r.is_valid());
        assert!(r.errors.iter().any(|e|e.contains("requires a value")));
        assert!(r.errors.iter().any(|e|e.contains("evidence ID")));
    }
    #[test] fn warns_on_unverified_source() {
        let v=fixture(include_str!("../governance/fixtures/source-record.unverified.json"));
        let r=validate(RecordKind::SourceRecord,&v);
        assert!(r.is_valid(),"{:?}",r.errors);
        assert!(!r.warnings.is_empty());
    }
    #[test] fn accepts_proposed_decision() {
        let v=fixture(include_str!("../governance/fixtures/decision-record.proposed.json"));
        assert!(validate(RecordKind::DecisionRecord,&v).is_valid());
    }
    #[test] fn accepts_evaluation_only_manifest() {
        let v=fixture(include_str!("../governance/fixtures/training-run-manifest.evaluation-only.json"));
        assert!(validate(RecordKind::TrainingRunManifest,&v).is_valid());
    }
    #[test] fn rejects_training_without_gates() {
        let mut v=fixture(include_str!("../governance/fixtures/training-run-manifest.evaluation-only.json"));
        v["run_type"]=Value::String("training".into());
        v["rulebook"]["status"]=Value::String("proposed".into());
        v["governance_review"]["state"]=Value::String("pending".into());
        let r=validate(RecordKind::TrainingRunManifest,&v);
        assert!(!r.is_valid());
        assert!(r.errors.iter().any(|e|e.contains("approved Rulebook")));
        assert!(r.errors.iter().any(|e|e.contains("verified dataset rights")));
        assert!(r.errors.iter().any(|e|e.contains("approved governance review")));
    }
    #[test] fn rejects_approved_decision_without_approver() {
        let mut v=fixture(include_str!("../governance/fixtures/decision-record.proposed.json"));
        v["status"]=Value::String("approved".into());
        v["approval"]["state"]=Value::String("approved".into());
        let r=validate(RecordKind::DecisionRecord,&v);
        assert!(!r.is_valid());
        assert!(r.errors.iter().any(|e|e.contains("named approver")));
    }
}
