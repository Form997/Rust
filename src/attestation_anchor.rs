//! GitHub artifact-attestation verification and externally attestable audit anchors.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs, path::Path, process::Command};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorError(pub String);
impl std::fmt::Display for AnchorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) }
}
impl std::error::Error for AnchorError {}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Create a compact anchor containing both the verified chain head and full ledger digest.
/// The anchor must then be attested by a separately trusted CI workflow.
pub fn create_audit_anchor(ledger_path: &Path, anchor_path: &Path) -> Result<Value, AnchorError> {
    let bytes = fs::read(ledger_path).map_err(|e| AnchorError(format!("cannot read ledger: {e}")))?;
    let content = std::str::from_utf8(&bytes).map_err(|e| AnchorError(format!("ledger is not UTF-8: {e}")))?;
    let mut previous = "GENESIS".to_owned();
    let mut count = 0u64;
    for (line_no, line) in content.lines().enumerate() {
        if line.trim().is_empty() { return Err(AnchorError(format!("blank ledger line {}", line_no + 1))); }
        let record: Value = serde_json::from_str(line).map_err(|e| AnchorError(format!("invalid JSON at line {}: {e}", line_no + 1)))?;
        let sequence = record.get("sequence").and_then(Value::as_u64)
            .ok_or_else(|| AnchorError(format!("missing sequence at line {}", line_no + 1)))?;
        if sequence != count + 1 { return Err(AnchorError(format!("sequence gap at line {}", line_no + 1))); }
        if record.get("previous_hash").and_then(Value::as_str) != Some(previous.as_str()) {
            return Err(AnchorError(format!("previous hash mismatch at line {}", line_no + 1)));
        }
        let stored = record.get("record_hash").and_then(Value::as_str)
            .ok_or_else(|| AnchorError(format!("missing record hash at line {}", line_no + 1)))?;
        let mut unsigned = record.clone();
        unsigned.as_object_mut().ok_or_else(|| AnchorError("audit record must be an object".into()))?.remove("record_hash");
        let canonical = serde_json::to_vec(&unsigned).map_err(|e| AnchorError(e.to_string()))?;
        if sha256_hex(&canonical) != stored { return Err(AnchorError(format!("record hash mismatch at line {}", line_no + 1))); }
        previous = stored.to_owned();
        count += 1;
    }
    if count == 0 { return Err(AnchorError("cannot anchor an empty audit ledger".into())); }
    let anchor = json!({
        "schema_version":"al-audit-anchor-1",
        "sequence":count,
        "head_record_sha256":previous,
        "ledger_sha256":sha256_hex(&bytes)
    });
    fs::write(anchor_path, serde_json::to_vec_pretty(&anchor).map_err(|e| AnchorError(e.to_string()))?)
        .map_err(|e| AnchorError(format!("cannot write anchor: {e}")))?;
    Ok(anchor)
}

/// Check that a ledger still matches the contents committed by an independently attested anchor.
pub fn verify_audit_anchor(ledger_path: &Path, anchor_path: &Path) -> Result<(), AnchorError> {
    let expected: Value = serde_json::from_slice(&fs::read(anchor_path).map_err(|e| AnchorError(e.to_string()))?)
        .map_err(|e| AnchorError(format!("invalid anchor JSON: {e}")))?;
    if expected.get("schema_version").and_then(Value::as_str) != Some("al-audit-anchor-1") {
        return Err(AnchorError("unsupported audit anchor schema".into()));
    }
    let bytes = fs::read(ledger_path).map_err(|e| AnchorError(format!("cannot read ledger: {e}")))?;
    if expected.get("ledger_sha256").and_then(Value::as_str) != Some(sha256_hex(&bytes).as_str()) {
        return Err(AnchorError("ledger digest does not match externally attested anchor".into()));
    }
    let content = std::str::from_utf8(&bytes).map_err(|e| AnchorError(e.to_string()))?;
    let last = content.lines().last().ok_or_else(|| AnchorError("empty ledger".into()))?;
    let record: Value = serde_json::from_str(last).map_err(|e| AnchorError(e.to_string()))?;
    if expected.get("sequence").and_then(Value::as_u64) != record.get("sequence").and_then(Value::as_u64) ||
       expected.get("head_record_sha256").and_then(Value::as_str) != record.get("record_hash").and_then(Value::as_str) {
        return Err(AnchorError("ledger head does not match externally attested anchor".into()));
    }
    Ok(())
}

/// Delegate Sigstore/GitHub trust-root, certificate, transparency-log and subject-digest
/// verification to the official GitHub CLI. Missing gh, missing auth or failed verification
/// are all hard failures. Never accept a user-provided 'verified' boolean as a substitute.
pub fn verify_github_attestation(
    subject_path: &Path,
    repository: &str,
    signer_workflow: &str,
    predicate_type: &str
) -> Result<String, AnchorError> {
    if !subject_path.is_file() { return Err(AnchorError("attestation subject file does not exist".into())); }
    if !repository.contains('/') || repository.starts_with('/') || repository.ends_with('/') {
        return Err(AnchorError("repository must use owner/repo form".into()));
    }
    if signer_workflow.trim().is_empty() || predicate_type.trim().is_empty() {
        return Err(AnchorError("signer workflow and predicate type are required".into()));
    }
    let output = Command::new("gh")
        .arg("attestation").arg("verify").arg(subject_path)
        .arg("--repo").arg(repository)
        .arg("--signer-workflow").arg(signer_workflow)
        .arg("--predicate-type").arg(predicate_type)
        .arg("--deny-self-hosted-runners")
        .output()
        .map_err(|e| AnchorError(format!("cannot execute GitHub CLI 'gh': {e}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AnchorError(format!("GitHub attestation verification failed: {}", stderr.trim())));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(format!("GitHub attestation verified for {} using signer workflow {}. {}", subject_path.display(), signer_workflow, stdout.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    fn temp(name:&str)->std::path::PathBuf {
        std::env::temp_dir().join(format!("al-{name}-{}-{}.tmp",std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()))
    }
    fn record(sequence:u64, previous:&str, event:Value)->Value {
        let mut v=json!({"sequence":sequence,"previous_hash":previous,"event":event});
        let unsigned=serde_json::to_vec(&v).unwrap();
        v["record_hash"]=json!(sha256_hex(&unsigned));
        v
    }
    #[test] fn creates_and_verifies_external_anchor() {
        let ledger=temp("anchor-ledger"); let anchor=temp("anchor-file");
        let one=record(1,"GENESIS",json!({"action":"start"}));
        let two=record(2,one["record_hash"].as_str().unwrap(),json!({"action":"finish"}));
        fs::write(&ledger,format!("{}\n{}",one,two)).unwrap();
        create_audit_anchor(&ledger,&anchor).unwrap();
        assert!(verify_audit_anchor(&ledger,&anchor).is_ok());
        let _=fs::remove_file(ledger); let _=fs::remove_file(anchor);
    }
    #[test] fn detects_ledger_rewrite_after_anchor() {
        let ledger=temp("tampered-ledger"); let anchor=temp("tampered-anchor");
        let one=record(1,"GENESIS",json!({"action":"original"}));
        fs::write(&ledger,one.to_string()).unwrap();
        create_audit_anchor(&ledger,&anchor).unwrap();
        let changed=record(1,"GENESIS",json!({"action":"rewritten"}));
        fs::write(&ledger,changed.to_string()).unwrap();
        assert!(verify_audit_anchor(&ledger,&anchor).unwrap_err().0.contains("digest"));
        let _=fs::remove_file(ledger); let _=fs::remove_file(anchor);
    }
    #[test] fn refuses_empty_ledger() {
        let ledger=temp("empty-ledger"); let anchor=temp("empty-anchor");
        fs::write(&ledger,"").unwrap();
        assert!(create_audit_anchor(&ledger,&anchor).is_err());
        let _=fs::remove_file(ledger); let _=fs::remove_file(anchor);
    }
}
