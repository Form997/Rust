//! Local verification of evidence manifests and tamper-evident audit chains.
//! CI metadata is corroborating data only; this module does not authenticate GitHub API responses.
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs, fs::OpenOptions, io::Write, path::Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceError(pub String);
impl std::fmt::Display for EvidenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) }
}
impl std::error::Error for EvidenceError {}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn str_field<'a>(v: &'a Value, key: &str) -> Result<&'a str, EvidenceError> {
    v.get(key).and_then(Value::as_str).filter(|s| !s.trim().is_empty())
        .ok_or_else(|| EvidenceError(format!("missing or empty '{key}'")))
}
fn valid_hex(s: &str, len: usize) -> bool {
    s.len() == len && s.bytes().all(|b| b.is_ascii_hexdigit())
}
fn valid_utc(s: &str) -> bool {
    // Deliberately narrow, timezone-unambiguous profile: YYYY-MM-DDTHH:MM:SSZ.
    let b = s.as_bytes();
    b.len() == 20 && b[4] == b'-' && b[7] == b'-' && b[10] == b'T' &&
    b[13] == b':' && b[16] == b':' && b[19] == b'Z' &&
    [0,1,2,3,5,6,8,9,11,12,14,15,17,18].iter().all(|i| b[*i].is_ascii_digit()) &&
    s[5..7].parse::<u8>().is_ok_and(|m| (1..=12).contains(&m)) &&
    s[8..10].parse::<u8>().is_ok_and(|d| (1..=31).contains(&d)) &&
    s[11..13].parse::<u8>().is_ok_and(|h| h <= 23) &&
    s[14..16].parse::<u8>().is_ok_and(|m| m <= 59) &&
    s[17..19].parse::<u8>().is_ok_and(|sec| sec <= 60)
}

/// Verify artifact bytes and consistency of a GitHub Actions run/job evidence snapshot.
/// The snapshot is not a cryptographic GitHub attestation; callers must obtain it from a trusted channel.
pub fn verify_evidence(manifest: &Value, artifact_path: &Path, expected_repo: &str, now_utc: &str) -> Result<String, EvidenceError> {
    let id = str_field(manifest, "evidence_id")?;
    let artifact = manifest.get("artifact").ok_or_else(|| EvidenceError("missing artifact object".into()))?;
    let expected_hash = str_field(artifact, "sha256")?.to_ascii_lowercase();
    if !valid_hex(&expected_hash, 64) { return Err(EvidenceError("artifact.sha256 must be 64 hexadecimal characters".into())); }
    let bytes = fs::read(artifact_path).map_err(|e| EvidenceError(format!("cannot read artifact: {e}")))?;
    let actual_hash = sha256_hex(&bytes);
    if actual_hash != expected_hash { return Err(EvidenceError(format!("artifact digest mismatch: expected {expected_hash}, got {actual_hash}"))); }

    let ci = manifest.get("ci").ok_or_else(|| EvidenceError("missing ci object".into()))?;
    let repo = str_field(ci, "repository")?;
    if repo != expected_repo { return Err(EvidenceError("CI repository does not match expected repository".into())); }
    let commit = str_field(ci, "commit_sha")?;
    if !valid_hex(commit, 40) { return Err(EvidenceError("ci.commit_sha must be a 40-character hexadecimal SHA".into())); }
    let run_id = ci.get("run_id").and_then(Value::as_u64).filter(|n| *n > 0)
        .ok_or_else(|| EvidenceError("ci.run_id must be a positive integer".into()))?;
    let job_id = ci.get("job_id").and_then(Value::as_u64).filter(|n| *n > 0)
        .ok_or_else(|| EvidenceError("ci.job_id must be a positive integer".into()))?;
    let conclusion = str_field(ci, "conclusion")?;
    if conclusion != "success" { return Err(EvidenceError("CI job conclusion is not success".into())); }
    let url = str_field(ci, "run_url")?;
    let expected_url = format!("https://github.com/{repo}/actions/runs/{run_id}");
    if url != expected_url { return Err(EvidenceError("CI run URL does not match repository and run_id".into())); }
    let observed = str_field(ci, "observed_at")?;
    if !valid_utc(observed) || !valid_utc(now_utc) { return Err(EvidenceError("timestamps must use YYYY-MM-DDTHH:MM:SSZ UTC format".into())); }
    let expires = str_field(manifest, "expires_at")?;
    if !valid_utc(expires) { return Err(EvidenceError("expires_at must use YYYY-MM-DDTHH:MM:SSZ UTC format".into())); }
    if now_utc > expires { return Err(EvidenceError("evidence has expired".into())); }
    if let Some(revoked) = manifest.get("revoked_at").and_then(Value::as_str).filter(|s| !s.is_empty()) {
        if !valid_utc(revoked) { return Err(EvidenceError("revoked_at has invalid UTC format".into())); }
        return Err(EvidenceError("evidence has been revoked".into()));
    }
    if manifest.get("revoked_at").is_some_and(|v| !v.is_null() && !v.as_str().is_some_and(str::is_empty)) {
        return Err(EvidenceError("revoked_at must be null or an empty string when not revoked".into()));
    }
    Ok(format!("{}: artifact SHA-256 verified; CI run #{run_id}/job #{job_id} metadata consistent (not cryptographically attested)", id))
}

fn record_hash(record: &Value) -> Result<String, EvidenceError> {
    let mut unsigned = record.clone();
    let obj = unsigned.as_object_mut().ok_or_else(|| EvidenceError("audit record must be an object".into()))?;
    obj.remove("record_hash");
    let bytes = serde_json::to_vec(&unsigned).map_err(|e| EvidenceError(e.to_string()))?;
    Ok(sha256_hex(&bytes))
}

/// Verify a JSONL hash chain. A separately stored trusted head hash is still required to
/// prevent an attacker with write access from rewriting and re-hashing the whole ledger.
pub fn verify_audit_chain(path: &Path) -> Result<usize, EvidenceError> {
    let content = fs::read_to_string(path).map_err(|e| EvidenceError(format!("cannot read audit ledger: {e}")))?;
    let mut previous = "GENESIS".to_owned();
    let mut count = 0usize;
    for (line_no, line) in content.lines().enumerate() {
        if line.trim().is_empty() { return Err(EvidenceError(format!("blank audit line at {}", line_no + 1))); }
        let record: Value = serde_json::from_str(line).map_err(|e| EvidenceError(format!("invalid JSON at audit line {}: {e}", line_no + 1)))?;
        let seq = record.get("sequence").and_then(Value::as_u64).ok_or_else(|| EvidenceError("audit sequence missing".into()))?;
        if seq != (count + 1) as u64 { return Err(EvidenceError(format!("audit sequence gap at line {}", line_no + 1))); }
        if record.get("previous_hash").and_then(Value::as_str) != Some(previous.as_str()) {
            return Err(EvidenceError(format!("previous_hash mismatch at line {}", line_no + 1)));
        }
        let stored = str_field(&record, "record_hash")?;
        let actual = record_hash(&record)?;
        if stored != actual { return Err(EvidenceError(format!("record hash mismatch at line {}", line_no + 1))); }
        previous = stored.to_owned();
        count += 1;
    }
    Ok(count)
}

/// Append one event after validating the existing chain. File permissions and external
/// anchoring of the final hash must be enforced by deployment infrastructure.
pub fn append_audit_event(path: &Path, event: &Value) -> Result<String, EvidenceError> {
    if !event.is_object() { return Err(EvidenceError("audit event must be a JSON object".into())); }
    let (count, previous) = if path.exists() {
        let n = verify_audit_chain(path)?;
        let content = fs::read_to_string(path).map_err(|e| EvidenceError(e.to_string()))?;
        let last = content.lines().last().ok_or_else(|| EvidenceError("unexpected empty ledger".into()))?;
        let v: Value = serde_json::from_str(last).map_err(|e| EvidenceError(e.to_string()))?;
        (n, str_field(&v, "record_hash")?.to_owned())
    } else { (0, "GENESIS".to_owned()) };
    let mut record = json!({"sequence":count + 1,"previous_hash":previous,"event":event});
    let hash = record_hash(&record)?;
    record["record_hash"] = Value::String(hash.clone());
    let mut file = OpenOptions::new().create(true).append(true).open(path)
        .map_err(|e| EvidenceError(format!("cannot open audit ledger: {e}")))?;
    if count > 0 || path.metadata().map(|m| m.len() > 0).unwrap_or(false) { file.write_all(b"\n").map_err(|e| EvidenceError(e.to_string()))?; }
    file.write_all(serde_json::to_string(&record).map_err(|e| EvidenceError(e.to_string()))?.as_bytes())
        .map_err(|e| EvidenceError(e.to_string()))?;
    file.flush().map_err(|e| EvidenceError(e.to_string()))?;
    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};
    fn temp(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("al-{name}-{}-{}.jsonl", std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()))
    }
    fn manifest(bytes: &[u8]) -> Value {
        json!({"evidence_id":"EV-1","artifact":{"sha256":sha256_hex(bytes)},
          "ci":{"repository":"Form997/Rust","commit_sha":"0123456789012345678901234567890123456789",
            "run_id":123,"job_id":456,"conclusion":"success",
            "run_url":"https://github.com/Form997/Rust/actions/runs/123",
            "observed_at":"2026-11-27T10:00:00Z"},
          "expires_at":"2026-12-01T00:00:00Z","revoked_at":null})
    }
    #[test] fn verifies_digest_and_ci_consistency() {
        let p = temp("artifact"); fs::write(&p,b"verified artifact").unwrap();
        assert!(verify_evidence(&manifest(b"verified artifact"),&p,"Form997/Rust","2026-11-27T11:00:00Z").is_ok());
        let _ = fs::remove_file(p);
    }
    #[test] fn rejects_digest_mismatch() {
        let p = temp("bad-digest"); fs::write(&p,b"changed").unwrap();
        assert!(verify_evidence(&manifest(b"original"),&p,"Form997/Rust","2026-11-27T11:00:00Z").unwrap_err().0.contains("digest mismatch"));
        let _ = fs::remove_file(p);
    }
    #[test] fn rejects_wrong_repository() {
        let p = temp("wrong-repo"); fs::write(&p,b"x").unwrap();
        assert!(verify_evidence(&manifest(b"x"),&p,"other/repo","2026-11-27T11:00:00Z").unwrap_err().0.contains("repository"));
        let _ = fs::remove_file(p);
    }
    #[test] fn rejects_expired_or_revoked_evidence() {
        let p = temp("expired"); fs::write(&p,b"x").unwrap();
        assert!(verify_evidence(&manifest(b"x"),&p,"Form997/Rust","2027-01-01T00:00:00Z").unwrap_err().0.contains("expired"));
        let mut m = manifest(b"x"); m["revoked_at"] = json!("2026-11-27T10:30:00Z");
        assert!(verify_evidence(&m,&p,"Form997/Rust","2026-11-27T11:00:00Z").unwrap_err().0.contains("revoked"));
        let _ = fs::remove_file(p);
    }
    #[test] fn detects_audit_chain_tampering() {
        let p = temp("ledger");
        append_audit_event(&p,&json!({"action":"evidence_verified","id":"EV-1"})).unwrap();
        append_audit_event(&p,&json!({"action":"evidence_revoked","id":"EV-1"})).unwrap();
        assert_eq!(verify_audit_chain(&p).unwrap(),2);
        let mut s = fs::read_to_string(&p).unwrap(); s = s.replace("evidence_verified","evidence_forged"); fs::write(&p,s).unwrap();
        assert!(verify_audit_chain(&p).unwrap_err().0.contains("hash mismatch"));
        let _ = fs::remove_file(p);
    }
}
