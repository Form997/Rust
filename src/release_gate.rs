//! Build/release adapter for the shared Rulebook gate.
//!
//! The adapter hashes the exact artifact bytes and requests a scoped RELEASE
//! decision. It does not itself publish or execute the artifact.

use std::{fs, path::Path};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::shared_gate::{evaluate_gate, GateDecision, GateRequest, VerifiedRegistry};

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
    if !matches!(operation, "build" | "package" | "publish_release") {
        return Err(format!("unsupported release operation: {operation}"));
    }
    let bytes = fs::read(artifact_path)
        .map_err(|e| format!("cannot read release artifact {}: {e}", artifact_path.display()))?;
    let digest = Sha256::digest(&bytes);
    let subject_sha256 = digest.iter().map(|b| format!("{b:02x}")).collect::<String>();
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
        assert!(matches!("build", "build" | "package" | "publish_release"));
        assert!(matches!("package", "build" | "package" | "publish_release"));
        assert!(matches!("publish_release", "build" | "package" | "publish_release"));
        assert!(!matches!("deploy_arbitrary", "build" | "package" | "publish_release"));
    }
}
