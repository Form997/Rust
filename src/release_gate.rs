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
