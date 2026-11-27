//! Evidence binding for approved Rulebook controls.
//! A control ID is not sufficient by itself: it must resolve to an implemented,
//! independently verifiable control record inside the signed Rulebook payload.
use serde_json::Value;
use crate::policy_gate::Gate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlEvidenceError(pub String);
impl std::fmt::Display for ControlEvidenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) }
}
impl std::error::Error for ControlEvidenceError {}

fn non_empty(value: Option<&Value>) -> bool {
    value.and_then(Value::as_str).is_some_and(|s| !s.trim().is_empty())
}
fn non_empty_array(value: Option<&Value>) -> bool {
    value.and_then(Value::as_array).is_some_and(|a| !a.is_empty() &&
        a.iter().all(|v| v.as_str().is_some_and(|s| !s.trim().is_empty())))
}

/// Resolve every control referenced by an approved in-scope rule.
/// Evidence records are assertions in a signed bundle, not proof that a live
/// deployment is secure; integration tests and deployment attestations remain
/// necessary at the real subsystem boundary.
pub fn verify_control_evidence(payload: &Value, gate: Gate) -> Result<Vec<String>, ControlEvidenceError> {
    let rules = payload.get("rules").and_then(Value::as_array)
        .ok_or_else(|| ControlEvidenceError("Rulebook rules array is missing".into()))?;
    let catalogue = payload.get("control_evidence").and_then(Value::as_array)
        .ok_or_else(|| ControlEvidenceError("Rulebook control_evidence catalogue is missing; fail-closed".into()))?;
    let mut required = Vec::new();
    for rule in rules {
        if rule.get("status").and_then(Value::as_str) != Some("approved") { continue; }
        let scoped = rule.get("scope").and_then(Value::as_array)
            .is_some_and(|s| s.iter().any(|v| v.as_str()==Some(gate.as_str())));
        if !scoped { continue; }
        let ids = rule.get("control_ids").and_then(Value::as_array)
            .ok_or_else(|| ControlEvidenceError("approved in-scope rule has no control_ids".into()))?;
        for id in ids {
            let id = id.as_str().filter(|s| !s.trim().is_empty())
                .ok_or_else(|| ControlEvidenceError("control_ids contains an invalid identifier".into()))?;
            if !required.iter().any(|x: &String| x == id) { required.push(id.to_owned()); }
        }
    }
    if required.is_empty() {
        return Err(ControlEvidenceError(format!("no approved controls resolve for '{}' gate",gate.as_str())));
    }
    for id in &required {
        let record = catalogue.iter().find(|r| r.get("control_id").and_then(Value::as_str)==Some(id))
            .ok_or_else(|| ControlEvidenceError(format!("control '{id}' has no evidence record")))?;
        if record.get("status").and_then(Value::as_str)!=Some("implemented") {
            return Err(ControlEvidenceError(format!("control '{id}' is not marked implemented")));
        }
        if record.get("verification_status").and_then(Value::as_str)!=Some("verified") {
            return Err(ControlEvidenceError(format!("control '{id}' evidence is not verified")));
        }
        if !non_empty(record.get("owner")) || !non_empty(record.get("scope")) ||
           !non_empty(record.get("last_verified_at")) {
            return Err(ControlEvidenceError(format!("control '{id}' is missing owner, scope or verification timestamp")));
        }
        if !non_empty_array(record.get("test_ids")) || !non_empty_array(record.get("evidence_ids")) {
            return Err(ControlEvidenceError(format!("control '{id}' requires test_ids and evidence_ids")));
        }
        let record_scope = record.get("gate").and_then(Value::as_str);
        if record_scope != Some(gate.as_str()) {
            return Err(ControlEvidenceError(format!("control '{id}' evidence is not scoped to '{}' gate",gate.as_str())));
        }
    }
    Ok(required)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn payload() -> Value { json!({
        "rules":[{"rule_id":"R-1","status":"approved","scope":["compiler"],"control_ids":["C-1"]}],
        "control_evidence":[{"control_id":"C-1","gate":"compiler","status":"implemented",
            "verification_status":"verified","owner":"compiler-team","scope":"compiler preflight",
            "last_verified_at":"2026-11-27T00:00:00Z","test_ids":["IT-COMPILER-01"],
            "evidence_ids":["sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"]}]
    }) }
    #[test] fn accepts_resolved_verified_control() { assert_eq!(verify_control_evidence(&payload(),Gate::Compiler).unwrap(),vec!["C-1"]); }
    #[test] fn rejects_missing_catalogue() { let mut p=payload(); p.as_object_mut().unwrap().remove("control_evidence"); assert!(verify_control_evidence(&p,Gate::Compiler).unwrap_err().0.contains("catalogue")); }
    #[test] fn rejects_unresolved_control_id() { let mut p=payload(); p["control_evidence"]=json!([]); assert!(verify_control_evidence(&p,Gate::Compiler).unwrap_err().0.contains("no evidence record")); }
    #[test] fn rejects_unverified_control() { let mut p=payload(); p["control_evidence"][0]["verification_status"]="pending".into(); assert!(verify_control_evidence(&p,Gate::Compiler).unwrap_err().0.contains("not verified")); }
    #[test] fn rejects_control_for_wrong_gate() { let mut p=payload(); p["control_evidence"][0]["gate"]="runtime".into(); assert!(verify_control_evidence(&p,Gate::Compiler).unwrap_err().0.contains("not scoped")); }
    #[test] fn rejects_missing_test_evidence() { let mut p=payload(); p["control_evidence"][0]["test_ids"]=json!([]); assert!(verify_control_evidence(&p,Gate::Compiler).unwrap_err().0.contains("test_ids")); }
}
