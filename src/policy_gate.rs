//! Fail-closed policy gate for AL subsystems.
//! A gate is authorized only by an approved, signed Rulebook containing at least
//! one approved, in-scope rule with explicit control identifiers.

use serde_json::Value;
use crate::authority::{verify_signed_bundle, AuthorityError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gate {
    Compiler, Runtime, Ai, Training, Evaluation, Scientific, Release,
}
impl Gate {
    pub const ALL: [Gate; 7] = [
        Gate::Compiler, Gate::Runtime, Gate::Ai, Gate::Training,
        Gate::Evaluation, Gate::Scientific, Gate::Release,
    ];
    pub fn as_str(self) -> &'static str {
        match self {
            Gate::Compiler => "compiler", Gate::Runtime => "runtime", Gate::Ai => "ai",
            Gate::Training => "training", Gate::Evaluation => "evaluation",
            Gate::Scientific => "scientific", Gate::Release => "release",
        }
    }
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|gate| gate.as_str() == value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatePermit {
    pub gate: Gate,
    pub rule_ids: Vec<String>,
    pub rulebook_version: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateDenied(pub String);
impl std::fmt::Display for GateDenied {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) }
}
impl std::error::Error for GateDenied {}

/// Verify the human authority signature first, then require approved, explicit
/// controls for this subsystem. Missing policy is denial, never implicit allow.
pub fn authorize_gate(
    envelope: &Value,
    verified_registry: &Value,
    gate: Gate,
) -> Result<GatePermit, GateDenied> {
    verify_signed_bundle(envelope, verified_registry)
        .map_err(|e: AuthorityError| GateDenied(format!("Rulebook authority verification failed: {e}")))?;
    let payload = envelope.get("payload")
        .ok_or_else(|| GateDenied("signed Rulebook payload is missing".into()))?;
    let version = payload.get("rulebook_version").and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| GateDenied("Rulebook version is missing".into()))?;
    let rules = payload.get("rules").and_then(Value::as_array)
        .ok_or_else(|| GateDenied("Rulebook has no rules array; fail-closed".into()))?;

    let mut rule_ids = Vec::new();
    for rule in rules {
        let status = rule.get("status").and_then(Value::as_str);
        let in_scope = rule.get("scope").and_then(Value::as_array)
            .is_some_and(|scopes| scopes.iter().any(|s| s.as_str() == Some(gate.as_str())));
        let controls = rule.get("control_ids").and_then(Value::as_array);
        let has_controls = controls.is_some_and(|ids| !ids.is_empty()
            && ids.iter().all(|id| id.as_str().is_some_and(|s| !s.trim().is_empty())));
        if status == Some("approved") && in_scope && has_controls {
            if let Some(id) = rule.get("rule_id").and_then(Value::as_str).filter(|s| !s.trim().is_empty()) {
                rule_ids.push(id.to_owned());
            }
        }
    }
    rule_ids.sort();
    rule_ids.dedup();
    if rule_ids.is_empty() {
        return Err(GateDenied(format!(
            "DENIED: no approved rule with explicit controls covers the '{}' gate",
            gate.as_str()
        )));
    }
    Ok(GatePermit { gate, rule_ids, rulebook_version: version.to_owned() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use ed25519_dalek::{Signer, SigningKey};
    use serde_json::json;

    fn signed_policy() -> (Value, Value) {
        let key = SigningKey::from_bytes(&[29u8; 32]);
        let rules: Vec<Value> = Gate::ALL.iter().enumerate().map(|(i, gate)| json!({
            "rule_id": format!("AL-GATE-{i}"), "status":"approved",
            "title": format!("Approved {} gate policy", gate.as_str()),
            "scope":[gate.as_str()], "control_ids":[format!("CTRL-{i}")]
        })).collect();
        let payload = json!({
            "bundle_id":"al-gate-test", "rulebook_version":"0.2.6", "status":"approved",
            "approval":{"state":"approved","approver":"owner-1","approved_at":"2026-11-27T00:00:00Z"},
            "rules":rules
        });
        let signature = key.sign(&serde_json::to_vec(&payload).unwrap());
        let envelope = json!({"algorithm":"Ed25519","key_id":"owner-key","payload":payload,
            "signature_base64":STANDARD.encode(signature.to_bytes())});
        let registry = json!({"registry_version":"1.0.0","keys":[{
            "key_id":"owner-key","principal_id":"owner-1","role":"rulebook_authority","status":"active",
            "public_key_base64":STANDARD.encode(key.verifying_key().as_bytes())
        }]});
        (envelope, registry)
    }

    #[test]
    fn all_seven_subsystem_gates_accept_approved_controls() {
        let (envelope, registry) = signed_policy();
        for gate in Gate::ALL {
            let permit = authorize_gate(&envelope, &registry, gate).unwrap();
            assert_eq!(permit.gate, gate);
            assert_eq!(permit.rule_ids.len(), 1);
            assert_eq!(permit.rulebook_version, "0.2.6");
        }
    }
    #[test]
    fn missing_rulebook_fails_closed() {
        let (mut envelope, registry) = signed_policy();
        envelope["payload"].as_object_mut().unwrap().remove("rules");
        // Tampering is rejected before policy evaluation.
        assert!(authorize_gate(&envelope, &registry, Gate::Compiler).unwrap_err().0.contains("authority verification"));
    }
    #[test]
    fn absent_gate_rule_fails_closed() {
        let (mut envelope, registry) = signed_policy();
        envelope["payload"]["rules"].as_array_mut().unwrap().retain(|r| r["scope"][0] != "runtime");
        let key = SigningKey::from_bytes(&[29u8; 32]);
        let sig = key.sign(&serde_json::to_vec(&envelope["payload"]).unwrap());
        envelope["signature_base64"] = Value::String(STANDARD.encode(sig.to_bytes()));
        assert!(authorize_gate(&envelope, &registry, Gate::Runtime).unwrap_err().0.contains("no approved rule"));
    }
    #[test]
    fn proposed_rule_cannot_authorize_gate() {
        let (mut envelope, registry) = signed_policy();
        envelope["payload"]["rules"][0]["status"] = Value::String("proposed".into());
        let key = SigningKey::from_bytes(&[29u8; 32]);
        let sig = key.sign(&serde_json::to_vec(&envelope["payload"]).unwrap());
        envelope["signature_base64"] = Value::String(STANDARD.encode(sig.to_bytes()));
        assert!(authorize_gate(&envelope, &registry, Gate::Compiler).unwrap_err().0.contains("no approved rule"));
    }
    #[test]
    fn rule_without_control_ids_cannot_authorize_gate() {
        let (mut envelope, registry) = signed_policy();
        envelope["payload"]["rules"][0]["control_ids"] = json!([]);
        let key = SigningKey::from_bytes(&[29u8; 32]);
        let sig = key.sign(&serde_json::to_vec(&envelope["payload"]).unwrap());
        envelope["signature_base64"] = Value::String(STANDARD.encode(sig.to_bytes()));
        assert!(authorize_gate(&envelope, &registry, Gate::Compiler).unwrap_err().0.contains("no approved rule"));
    }
    #[test]
    fn unknown_gate_name_is_not_parseable() {
        assert!(Gate::parse("everything") .is_none());
    }
}
