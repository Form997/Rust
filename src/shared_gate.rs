//! Shared Rulebook Gate API.
//!
//! This module provides one decision contract for subsystem adapters. It
//! verifies the owner-signed Rulebook envelope and binds a decision to one
//! operation, subject digest, policy digest and evidence references.
//! A decision is scoped to one request, not a blanket artifact approval.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use crate::authority::verify_signed_bundle;
use crate::control_evidence::verify_control_evidence;
use crate::policy_gate::{authorize_gate, Gate};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Decision { Allow, Deny, Review }
impl Decision {
    pub fn as_str(&self) -> &'static str {
        match self { Self::Allow => "ALLOW", Self::Deny => "DENY", Self::Review => "REVIEW" }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateRequest {
    pub decision_id: String,
    pub subsystem: String,
    pub operation: String,
    pub subject_type: String,
    pub subject_sha256: String,
    pub evaluated_at: String,
    pub verifier_version: String,
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateDecision {
    pub decision_id: String,
    pub decision: Decision,
    pub subsystem: String,
    pub operation: String,
    pub subject_type: String,
    pub subject_sha256: String,
    pub rulebook_version: Option<String>,
    pub rulebook_sha256: Option<String>,
    pub evaluated_at: String,
    pub verifier_version: String,
    pub control_ids: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub reason_codes: Vec<String>,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateApiError(pub String);
impl std::fmt::Display for GateApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) }
}
impl std::error::Error for GateApiError {}

fn valid_identifier(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 256 &&
        value.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ':' | '/'))
}
fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit()) && value == value.to_ascii_lowercase()
}
fn valid_rfc3339_utc(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 20 && bytes.get(4) == Some(&b'-') && bytes.get(7) == Some(&b'-') &&
        bytes.get(10) == Some(&b'T') && bytes.get(13) == Some(&b':') &&
        bytes.get(16) == Some(&b':') && value.ends_with('Z')
}
fn sha256_json(value: &Value) -> Result<String, GateApiError> {
    let bytes = serde_json::to_vec(value).map_err(|e| GateApiError(format!("cannot serialize Rulebook payload: {e}")))?;
    let digest = Sha256::digest(bytes);
    let mut out = String::with_capacity(64);
    for byte in digest { out.push_str(&format!("{byte:02x}")); }
    Ok(out)
}
fn allowed_operation(gate: Gate, operation: &str) -> bool {
    match gate {
        Gate::Compiler => matches!(operation, "parse" | "type_check" | "compile"),
        Gate::Runtime => matches!(operation, "load" | "execute"),
        Gate::Ai => matches!(operation, "infer" | "generate" | "explain" | "transform"),
        Gate::Training => matches!(operation, "prepare_training" | "train" | "fine_tune"),
        Gate::Evaluation => matches!(operation, "evaluate" | "benchmark"),
        Gate::Scientific => matches!(operation, "calculate" | "simulate" | "publish_result"),
        Gate::Release => matches!(operation, "build" | "package" | "publish_release"),
    }
}
fn deny(request: &GateRequest, code: &str, explanation: String) -> GateDecision {
    GateDecision {
        decision_id: request.decision_id.clone(), decision: Decision::Deny,
        subsystem: request.subsystem.clone(), operation: request.operation.clone(),
        subject_type: request.subject_type.clone(), subject_sha256: request.subject_sha256.clone(),
        rulebook_version: None, rulebook_sha256: None, evaluated_at: request.evaluated_at.clone(),
        verifier_version: request.verifier_version.clone(), control_ids: vec![],
        evidence_refs: request.evidence_refs.clone(), reason_codes: vec![code.into()], explanation,
    }
}
fn review(request: &GateRequest, code: &str, explanation: String) -> GateDecision {
    let mut result = deny(request, code, explanation);
    result.decision = Decision::Review;
    result
}

/// trusted_registry must have been verified against the separately pinned root
/// using authority::verify_signed_registry before it is passed here.
pub fn evaluate_gate(
    request: &GateRequest,
    envelope: &Value,
    trusted_registry: &Value,
) -> Result<GateDecision, GateApiError> {
    if !valid_identifier(&request.decision_id) || !valid_identifier(&request.subsystem) ||
        !valid_identifier(&request.operation) || !valid_identifier(&request.subject_type) ||
        !valid_identifier(&request.verifier_version) {
        return Err(GateApiError("request contains a missing or malformed identifier".into()));
    }
    if !valid_sha256(&request.subject_sha256) {
        return Err(GateApiError("subject_sha256 must be 64 lowercase hexadecimal characters".into()));
    }
    if !valid_rfc3339_utc(&request.evaluated_at) {
        return Err(GateApiError("evaluated_at must be an RFC3339 UTC timestamp ending in Z".into()));
    }
    if request.evidence_refs.iter().any(|id| !valid_identifier(id)) {
        return Err(GateApiError("evidence_refs contains an invalid identifier".into()));
    }
    let gate = Gate::parse(&request.subsystem)
        .ok_or_else(|| GateApiError(format!("unknown subsystem gate: {}", request.subsystem)))?;
    if !allowed_operation(gate, &request.operation) {
        return Ok(deny(request, "OPERATION_NOT_ALLOWED_FOR_SUBSYSTEM",
            format!("operation '{}' is not defined for subsystem '{}'", request.operation, request.subsystem)));
    }
    if request.evidence_refs.is_empty() {
        return Ok(review(request, "REQUIRED_EVIDENCE_NOT_SUPPLIED",
            "caller supplied no evidence references; no operation may be implicitly allowed".into()));
    }
    if let Err(error) = verify_signed_bundle(envelope, trusted_registry) {
        return Ok(deny(request, "RULEBOOK_AUTHORITY_VERIFICATION_FAILED", error.to_string()));
    }
    let payload = envelope.get("payload")
        .ok_or_else(|| GateApiError("verified envelope unexpectedly lacks payload".into()))?;
    let version = payload.get("rulebook_version").and_then(Value::as_str)
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| GateApiError("verified Rulebook has no version".into()))?.to_owned();
    let policy_digest = sha256_json(payload)?;
    let permit = match authorize_gate(envelope, trusted_registry, gate) {
        Ok(permit) => permit,
        Err(error) => return Ok(deny(request, "NO_APPROVED_RULE_FOR_SUBSYSTEM", error.to_string())),
    };
    let rules = payload.get("rules").and_then(Value::as_array)
        .ok_or_else(|| GateApiError("Rulebook rules array is missing".into()))?;
    let operation_covered = rules.iter().any(|rule| {
        rule.get("status").and_then(Value::as_str) == Some("approved") &&
        rule.get("scope").and_then(Value::as_array)
            .is_some_and(|scopes| scopes.iter().any(|s| s.as_str() == Some(gate.as_str()))) &&
        rule.get("operations").and_then(Value::as_array)
            .is_some_and(|ops| ops.iter().any(|op| op.as_str() == Some(request.operation.as_str())))
    });
    if !operation_covered {
        return Ok(deny(request, "NO_APPROVED_RULE_FOR_OPERATION",
            format!("no approved Rulebook rule explicitly covers operation '{}'", request.operation)));
    }
    let control_ids = match verify_control_evidence(payload, gate) {
        Ok(ids) => ids,
        Err(error) => return Ok(review(request, "CONTROL_EVIDENCE_INCOMPLETE", error.to_string())),
    };
    let catalogue = payload.get("control_evidence").and_then(Value::as_array)
        .ok_or_else(|| GateApiError("control evidence catalogue disappeared after validation".into()))?;
    let known_evidence: std::collections::HashSet<&str> = catalogue.iter()
        .filter_map(|record| record.get("evidence_ids").and_then(Value::as_array))
        .flat_map(|items| items.iter().filter_map(Value::as_str)).collect();
    for evidence_id in &request.evidence_refs {
        if !known_evidence.contains(evidence_id.as_str()) {
            return Ok(review(request, "EVIDENCE_REFERENCE_UNRESOLVED",
                format!("evidence reference '{}' does not resolve in the signed control evidence catalogue", evidence_id)));
        }
    }
    let _ = permit;
    Ok(GateDecision {
        decision_id: request.decision_id.clone(), decision: Decision::Allow,
        subsystem: gate.as_str().into(), operation: request.operation.clone(),
        subject_type: request.subject_type.clone(), subject_sha256: request.subject_sha256.clone(),
        rulebook_version: Some(version), rulebook_sha256: Some(policy_digest),
        evaluated_at: request.evaluated_at.clone(), verifier_version: request.verifier_version.clone(),
        control_ids, evidence_refs: request.evidence_refs.clone(), reason_codes: vec![],
        explanation: format!("All declared mandatory checks passed for this {} operation under the signed Rulebook snapshot. This ALLOW is limited to this request and subject digest.", request.operation),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request() -> GateRequest {
        GateRequest { decision_id:"dec-001".into(), subsystem:"compiler".into(), operation:"compile".into(),
            subject_type:"al_source".into(), subject_sha256:"a".repeat(64),
            evaluated_at:"2026-11-27T10:00:00Z".into(), verifier_version:"0.2.10.2".into(),
            evidence_refs:vec!["E-COMPILER-001".into()] }
    }
    #[test] fn accepts_well_formed_request_shape() {
        let r=request();
        assert!(valid_identifier(&r.decision_id));
        assert!(valid_sha256(&r.subject_sha256));
        assert!(valid_rfc3339_utc(&r.evaluated_at));
        assert!(allowed_operation(Gate::Compiler,&r.operation));
    }
    #[test] fn rejects_operation_outside_subsystem() {
        assert!(!allowed_operation(Gate::Compiler,"train"));
        assert!(!allowed_operation(Gate::Runtime,"publish_release"));
    }
    #[test] fn rejects_uppercase_digest() { assert!(!valid_sha256(&"A".repeat(64))); }
    #[test] fn rejects_wrong_digest_length() { assert!(!valid_sha256("abc")); }
    #[test] fn requires_utc_timestamp() {
        assert!(valid_rfc3339_utc("2026-11-27T10:00:00Z"));
        assert!(!valid_rfc3339_utc("2026-11-27T10:00:00+07:00"));
    }
    #[test] fn unknown_gate_is_not_supported() { assert!(Gate::parse("dataset").is_none()); }
}
