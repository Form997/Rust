//! Build/release adapter for the shared Rulebook gate.
//!
//! The adapter hashes the exact artifact bytes and requests a scoped RELEASE
//! decision. It does not itself publish or execute the artifact.

use std::{fs, path::Path};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::shared_gate::{evaluate_gate, GateDecision, GateRequest, VerifiedRegistry};

fn is_release_operation(operation: &str) -> bool {
    matches!(operation, "build" | "package" | "publish_release")
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

fn artifact_sha256(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path)
        .map_err(|e| format!("cannot read release artifact {}: {e}", path.display()))?;
    Ok(sha256_hex(&bytes))
}

pub fn evaluate_release_artifact(
    operation: &str,
    artifact_path: &Path,
    evaluated_at: &str,
    decision_id: &str,
    verifier_version: &str,
    evidence_refs: Vec<String>,
    envelope: &Value,
    registry: &VerifiedRegistry,
) -> Result<GateDecision, String> {
    if !is_release_operation(operation) {
        return Err(format!("unsupported release operation: {operation}"));
    }
    let subject_sha256 = artifact_sha256(artifact_path)?;
    let request = GateRequest {
        decision_id: decision_id.to_owned(),
        subsystem: "release".to_owned(),
        operation: operation.to_owned(),
        subject_type: "release_artifact".to_owned(),
        subject_sha256,
        evaluated_at: evaluated_at.to_owned(),
        verifier_version: verifier_version.to_owned(),
        evidence_refs,
    };
    evaluate_gate(&request, envelope, registry).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_operations_are_explicitly_allowlisted() {
        assert!(is_release_operation("build"));
        assert!(is_release_operation("package"));
        assert!(is_release_operation("publish_release"));
        assert!(!is_release_operation("deploy_arbitrary"));
        assert!(!is_release_operation("train"));
        assert!(!is_release_operation(""));
    }

    #[test]
    fn artifact_digest_uses_exact_bytes() {
        // SHA-256 test vector from the standard "abc" byte sequence.
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_ne!(sha256_hex(b"abc"), sha256_hex(b"abc\n"));
    }

    #[test]
    fn artifact_digest_is_lowercase_and_fixed_width() {
        let digest = sha256_hex(b"AL release artifact");
        assert_eq!(digest.len(), 64);
        assert!(digest.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()));
    }

    #[test]
    fn adapter_hashes_the_bytes_read_from_the_artifact_file() {
        let path = std::env::temp_dir().join(format!("al-release-gate-{}.bin", std::process::id()));
        let bytes = b"artifact bytes are the subject";
        fs::write(&path, bytes).expect("write test artifact");
        let expected = sha256_hex(bytes);
        assert_eq!(artifact_sha256(&path).expect("hash test artifact"), expected);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn missing_artifact_is_rejected_before_authorization() {
        let path = std::env::temp_dir().join(format!("al-release-gate-missing-{}.bin", std::process::id()));
        let _ = fs::remove_file(&path);
        assert!(artifact_sha256(&path).is_err());
    }
}


#[cfg(test)]
mod authority_integration_tests {
    use super::*;
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use ed25519_dalek::{Signer, SigningKey};
    use serde_json::{json, Value};
    use sha2::{Digest, Sha256};
    use crate::shared_gate::VerifiedRegistry;

    fn fingerprint(key: &SigningKey) -> String {
        Sha256::digest(key.verifying_key().as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
    }

    fn signed_authority(signer_status: &str) -> (Value, Value, String) {
        // Deterministic test-only keys. These keys are never production trust material.
        let root = SigningKey::from_bytes(&[71u8; 32]);
        let owner = SigningKey::from_bytes(&[72u8; 32]);
        let registry_payload = json!({
            "registry_version":"test-1",
            "keys":[{
                "key_id":"test-owner",
                "principal_id":"test-owner-principal",
                "role":"rulebook_authority",
                "status":signer_status,
                "public_key_base64":STANDARD.encode(owner.verifying_key().as_bytes())
            }]
        });
        let registry_signature = root.sign(&serde_json::to_vec(&registry_payload).unwrap());
        let registry = json!({
            "algorithm":"Ed25519",
            "root_public_key_base64":STANDARD.encode(root.verifying_key().as_bytes()),
            "payload":registry_payload,
            "signature_base64":STANDARD.encode(registry_signature.to_bytes())
        });
        let control_id = "CTRL-RELEASE-TEST";
        let payload = json!({
            "bundle_id":"al-release-gate-test",
            "rulebook_version":"0.2.10.3-test",
            "status":"approved",
            "approval":{
                "state":"approved",
                "approver":"test-owner-principal",
                "approved_at":"2026-11-28T00:00:00Z"
            },
            "rules":[{
                "rule_id":"AL-RELEASE-TEST",
                "status":"approved",
                "title":"Test-only release gate authorization",
                "scope":["release"],
                "operations":["build","package","publish_release"],
                "control_ids":[control_id]
            }],
            "control_evidence":[{
                "control_id":control_id,
                "gate":"release",
                "status":"implemented",
                "verification_status":"verified",
                "owner":"test-only",
                "scope":"release adapter integration test",
                "last_verified_at":"2026-11-28T00:00:00Z",
                "test_ids":["R01-POSITIVE-AUTH"],
                "evidence_ids":["R01-TEST-EVIDENCE"]
            }]
        });
        let bundle_signature = owner.sign(&serde_json::to_vec(&payload).unwrap());
        let envelope = json!({
            "algorithm":"Ed25519",
            "key_id":"test-owner",
            "payload":payload,
            "signature_base64":STANDARD.encode(bundle_signature.to_bytes())
        });
        (registry, envelope, fingerprint(&root))
    }

    fn temp_artifact(label: &str, bytes: &[u8]) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "al-r01-{label}-{}.bin", std::process::id()
        ));
        fs::write(&path, bytes).unwrap();
        path
    }

    #[test]
    fn r01_positive_allow_requires_independently_pinned_root_and_active_owner_signature() {
        let (registry, envelope, pin) = signed_authority("active");
        let verified_registry = VerifiedRegistry::from_signed(&registry, &pin).unwrap();
        let artifact = temp_artifact("allow", b"R01 exact release bytes");
        let decision = evaluate_release_artifact(
            "publish_release", &artifact, "2026-11-28T10:00:00Z",
            "R01-ALLOW-001", "0.2.10.3-r01", vec!["R01-TEST-EVIDENCE".into()],
            &envelope, &verified_registry
        ).unwrap();
        assert_eq!(decision.decision.as_str(), "ALLOW");
        assert_eq!(decision.operation, "publish_release");
        assert_eq!(decision.subject_sha256, sha256_hex(b"R01 exact release bytes"));
        assert_eq!(decision.rulebook_version.as_deref(), Some("0.2.10.3-test"));
        let _ = fs::remove_file(artifact);
    }

    #[test]
    fn r01_wrong_out_of_band_root_pin_is_rejected() {
        let (registry, _, _) = signed_authority("active");
        assert!(VerifiedRegistry::from_signed(&registry, &"00".repeat(32)).is_err());
    }

    #[test]
    fn r01_revoked_owner_key_cannot_authorize_release() {
        let (registry, envelope, pin) = signed_authority("revoked");
        let verified_registry = VerifiedRegistry::from_signed(&registry, &pin).unwrap();
        let artifact = temp_artifact("revoked", b"release bytes");
        let decision = evaluate_release_artifact(
            "package", &artifact, "2026-11-28T10:00:00Z",
            "R01-REVOKED-001", "0.2.10.3-r01", vec!["R01-TEST-EVIDENCE".into()],
            &envelope, &verified_registry
        ).unwrap();
        assert_eq!(decision.decision.as_str(), "DENY");
        assert!(decision.reason_codes.iter().any(|c| c == "RULEBOOK_AUTHORITY_VERIFICATION_FAILED"));
        let _ = fs::remove_file(artifact);
    }

    #[test]
    fn r01_tampered_signed_rulebook_cannot_authorize_release() {
        let (registry, mut envelope, pin) = signed_authority("active");
        let verified_registry = VerifiedRegistry::from_signed(&registry, &pin).unwrap();
        envelope["payload"]["rulebook_version"] = Value::String("9.9.9".into());
        let artifact = temp_artifact("tampered", b"release bytes");
        let decision = evaluate_release_artifact(
            "build", &artifact, "2026-11-28T10:00:00Z",
            "R01-TAMPER-001", "0.2.10.3-r01", vec!["R01-TEST-EVIDENCE".into()],
            &envelope, &verified_registry
        ).unwrap();
        assert_eq!(decision.decision.as_str(), "DENY");
        let _ = fs::remove_file(artifact);
    }

    #[test]
    fn r01_missing_evidence_cannot_produce_allow() {
        let (registry, envelope, pin) = signed_authority("active");
        let verified_registry = VerifiedRegistry::from_signed(&registry, &pin).unwrap();
        let artifact = temp_artifact("no-evidence", b"release bytes");
        let decision = evaluate_release_artifact(
            "build", &artifact, "2026-11-28T10:00:00Z",
            "R01-EVIDENCE-001", "0.2.10.3-r01", vec![],
            &envelope, &verified_registry
        ).unwrap();
        assert_eq!(decision.decision.as_str(), "REVIEW");
        let _ = fs::remove_file(artifact);
    }
}


/// Result of the two-sided build boundary: an authorized build plan followed by
/// an authorized digest of the produced artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardedBuildResult {
    pub preflight: GateDecision,
    pub postflight: GateDecision,
}

/// Run a build callback only after a signed Rulebook ALLOW for the exact build
/// plan, then require a second ALLOW for the exact produced artifact.
///
/// This is an application-level adapter contract. It cannot prevent callers
/// from bypassing this API and invoking external build tools directly; operating
/// system and CI policy must make this path mandatory in production.
pub fn run_guarded_build<F>(
    build_plan_path: &Path,
    artifact_path: &Path,
    evaluated_at: &str,
    decision_prefix: &str,
    verifier_version: &str,
    evidence_refs: Vec<String>,
    envelope: &Value,
    registry: &VerifiedRegistry,
    build: F,
) -> Result<GuardedBuildResult, String>
where
    F: FnOnce() -> Result<(), String>,
{
    let plan_bytes = fs::read(build_plan_path)
        .map_err(|e| format!("cannot read build plan {}: {e}", build_plan_path.display()))?;
    let preflight_request = GateRequest {
        decision_id: format!("{decision_prefix}-PREFLIGHT"),
        subsystem: "release".to_owned(),
        operation: "build".to_owned(),
        subject_type: "build_plan".to_owned(),
        subject_sha256: sha256_hex(&plan_bytes),
        evaluated_at: evaluated_at.to_owned(),
        verifier_version: verifier_version.to_owned(),
        evidence_refs: evidence_refs.clone(),
    };
    let preflight = evaluate_gate(&preflight_request, envelope, registry)
        .map_err(|e| format!("build preflight request invalid: {e}"))?;
    if preflight.decision.as_str() != "ALLOW" {
        return Err(format!("build preflight blocked: {}", preflight.explanation));
    }

    build().map_err(|e| format!("build callback failed: {e}"))?;

    let postflight = evaluate_release_artifact(
        "build", artifact_path, evaluated_at,
        &format!("{decision_prefix}-POSTFLIGHT"), verifier_version,
        evidence_refs, envelope, registry
    )?;
    if postflight.decision.as_str() != "ALLOW" {
        return Err(format!("build artifact postflight blocked: {}", postflight.explanation));
    }
    Ok(GuardedBuildResult { preflight, postflight })
}
