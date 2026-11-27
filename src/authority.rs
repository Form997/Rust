//! AL Rulebook trust-chain and update verification.
//! Private signing keys are never created, stored, or accessed by this module.

use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde_json::Value;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorityError(pub String);
impl std::fmt::Display for AuthorityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) }
}
impl std::error::Error for AuthorityError {}

fn required_text<'a>(value: &'a Value, field: &str) -> Result<&'a str, AuthorityError> {
    value.get(field).and_then(Value::as_str).filter(|s| !s.trim().is_empty())
        .ok_or_else(|| AuthorityError(format!("missing or invalid field: {field}")))
}
fn decode_key(value: &Value, field: &str) -> Result<VerifyingKey, AuthorityError> {
    let bytes: [u8; 32] = STANDARD.decode(required_text(value, field)?)
        .map_err(|_| AuthorityError(format!("{field} is not valid base64")))?
        .try_into().map_err(|_| AuthorityError(format!("{field} must decode to 32 bytes")))?;
    VerifyingKey::from_bytes(&bytes).map_err(|_| AuthorityError(format!("{field} is not a valid Ed25519 key")))
}
fn verify_payload_signature(key: &VerifyingKey, payload: &Value, signature_text: &str) -> Result<(), AuthorityError> {
    let bytes = STANDARD.decode(signature_text).map_err(|_| AuthorityError("signature is not valid base64".into()))?;
    let signature = Signature::from_slice(&bytes).map_err(|_| AuthorityError("Ed25519 signature must be 64 bytes".into()))?;
    let canonical = serde_json::to_vec(payload).map_err(|e| AuthorityError(format!("cannot canonicalize signed payload: {e}")))?;
    key.verify(&canonical, &signature).map_err(|_| AuthorityError("signature verification failed".into()))
}

/// Verify the offline root-signed registry against a SHA-256 public-key fingerprint
/// supplied by a separately protected deployment configuration.
pub fn verify_signed_registry(envelope: &Value, pinned_fingerprint_hex: &str) -> Result<Value, AuthorityError> {
    if required_text(envelope, "algorithm")? != "Ed25519" {
        return Err(AuthorityError("registry algorithm must be Ed25519".into()));
    }
    let root_key = decode_key(envelope, "root_public_key_base64")?;
    let actual = hex_lower(&Sha256::digest(root_key.as_bytes()));
    let expected = pinned_fingerprint_hex.trim().to_ascii_lowercase();
    if expected.len() != 64 || !expected.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(AuthorityError("pinned root fingerprint must be 64 hexadecimal characters".into()));
    }
    if actual != expected {
        return Err(AuthorityError("root public-key fingerprint does not match the out-of-band pin".into()));
    }
    let payload = envelope.get("payload").ok_or_else(|| AuthorityError("signed registry is missing payload".into()))?;
    let signature = required_text(envelope, "signature_base64")?;
    verify_payload_signature(&root_key, payload, signature)?;

    let keys = payload.get("keys").and_then(Value::as_array)
        .ok_or_else(|| AuthorityError("registry payload must contain a keys array".into()))?;
    let mut seen = std::collections::HashSet::new();
    for item in keys {
        let id = required_text(item, "key_id")?;
        if !seen.insert(id.to_owned()) {
            return Err(AuthorityError(format!("duplicate key_id in trust registry: {id}")));
        }
        if required_text(item, "role")? != "rulebook_authority" {
            return Err(AuthorityError(format!("unsupported signer role for key {id}")));
        }
        if !matches!(required_text(item, "status")?, "active" | "revoked") {
            return Err(AuthorityError(format!("invalid signer status for key {id}")));
        }
        let _ = decode_key(item, "public_key_base64")?;
        required_text(item, "principal_id")?;
    }
    Ok(payload.clone())
}

pub fn verify_signed_bundle(envelope: &Value, trusted_registry: &Value) -> Result<(), AuthorityError> {
    if required_text(envelope, "algorithm")? != "Ed25519" {
        return Err(AuthorityError("bundle algorithm must be Ed25519".into()));
    }
    let key_id = required_text(envelope, "key_id")?;
    let payload = envelope.get("payload").ok_or_else(|| AuthorityError("missing payload".into()))?;
    let keys = trusted_registry.get("keys").and_then(Value::as_array)
        .ok_or_else(|| AuthorityError("trusted registry must contain a keys array".into()))?;
    let key = keys.iter().find(|item| item.get("key_id").and_then(Value::as_str) == Some(key_id))
        .ok_or_else(|| AuthorityError(format!("signing key is not trusted: {key_id}")))?;
    if required_text(key, "status")? != "active" {
        return Err(AuthorityError(format!("signing key is not active: {key_id}")));
    }
    if required_text(key, "role")? != "rulebook_authority" {
        return Err(AuthorityError("signing key lacks rulebook_authority role".into()));
    }
    let principal = required_text(key, "principal_id")?;
    if required_text(payload, "status")? != "approved" {
        return Err(AuthorityError("only an approved Rulebook payload may be activated".into()));
    }
    let approval = payload.get("approval").ok_or_else(|| AuthorityError("payload is missing approval record".into()))?;
    if required_text(approval, "state")? != "approved" {
        return Err(AuthorityError("payload approval state is not approved".into()));
    }
    if required_text(approval, "approver")? != principal {
        return Err(AuthorityError("approval identity does not match trusted signing principal".into()));
    }
    required_text(approval, "approved_at")?;
    let key = decode_key(key, "public_key_base64")?;
    verify_payload_signature(&key, payload, required_text(envelope, "signature_base64")?)
}

pub fn canonical_sha256(value: &Value) -> Result<String, AuthorityError> {
    let bytes = serde_json::to_vec(value).map_err(|e| AuthorityError(format!("cannot canonicalize JSON: {e}")))?;
    Ok(hex_lower(&Sha256::digest(bytes)))
}
fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes { out.push(HEX[(byte >> 4) as usize] as char); out.push(HEX[(byte & 0x0f) as usize] as char); }
    out
}
fn semver_tuple(value: &str) -> Result<(u64,u64,u64), AuthorityError> {
    let core = value.split(['-', '+']).next().unwrap_or("");
    let parts: Vec<_> = core.split('.').collect();
    if parts.len() != 3 { return Err(AuthorityError(format!("Rulebook version must be strict MAJOR.MINOR.PATCH: {value}"))); }
    let parsed: Result<Vec<u64>,_> = parts.iter().map(|p| p.parse::<u64>()).collect();
    let v = parsed.map_err(|_| AuthorityError(format!("invalid semantic version: {value}")))?;
    Ok((v[0],v[1],v[2]))
}

/// Verify chain of custody, human signature, parent hash and monotonic version
/// before an update is eligible for a separate activation step.
pub fn verify_update(current_payload: &Value, candidate_envelope: &Value, registry: &Value) -> Result<(), AuthorityError> {
    verify_signed_bundle(candidate_envelope, registry)?;
    let candidate = candidate_envelope.get("payload").ok_or_else(|| AuthorityError("candidate payload missing".into()))?;
    let expected_parent = canonical_sha256(current_payload)?;
    if required_text(candidate, "parent_bundle_sha256")? != expected_parent {
        return Err(AuthorityError("candidate parent hash does not match current active Rulebook".into()));
    }
    let current_version = semver_tuple(required_text(current_payload, "rulebook_version")?)?;
    let next_version = semver_tuple(required_text(candidate, "rulebook_version")?)?;
    if next_version <= current_version {
        return Err(AuthorityError("candidate Rulebook version must be greater than current version".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use serde_json::json;

    fn signed_registry_and_bundle() -> (Value, Value, Value, String, SigningKey) {
        let root = SigningKey::from_bytes(&[11u8;32]);
        let maintainer = SigningKey::from_bytes(&[7u8;32]);
        let registry_payload = json!({"registry_version":"1.0.0","keys":[{
            "key_id":"maintainer-1","principal_id":"human-1","role":"rulebook_authority","status":"active",
            "public_key_base64":STANDARD.encode(maintainer.verifying_key().as_bytes())
        }]});
        let root_sig = root.sign(&serde_json::to_vec(&registry_payload).unwrap());
        let registry_envelope = json!({"algorithm":"Ed25519","root_public_key_base64":STANDARD.encode(root.verifying_key().as_bytes()),
            "payload":registry_payload,"signature_base64":STANDARD.encode(root_sig.to_bytes())});
        let pin = hex_lower(&Sha256::digest(root.verifying_key().as_bytes()));
        let current = json!({"rulebook_version":"0.1.0","status":"approved"});
        let candidate_payload = json!({"rulebook_version":"0.2.0","status":"approved",
            "parent_bundle_sha256":canonical_sha256(&current).unwrap(),
            "approval":{"state":"approved","approver":"human-1","approved_at":"2026-11-27T00:00:00Z"}});
        let sig = maintainer.sign(&serde_json::to_vec(&candidate_payload).unwrap());
        let candidate = json!({"algorithm":"Ed25519","key_id":"maintainer-1","payload":candidate_payload,
            "signature_base64":STANDARD.encode(sig.to_bytes())});
        (registry_envelope,current,candidate,pin,root)
    }

    #[test] fn accepts_root_signed_registry_with_correct_pin() {
        let (registry,_,_,pin,_) = signed_registry_and_bundle();
        assert!(verify_signed_registry(&registry,&pin).is_ok());
    }
    #[test] fn rejects_registry_with_wrong_root_pin() {
        let (registry,_,_,_,_) = signed_registry_and_bundle();
        assert!(verify_signed_registry(&registry,&"00".repeat(32)).unwrap_err().0.contains("fingerprint"));
    }
    #[test] fn rejects_tampered_root_signed_registry() {
        let (mut registry,_,_,pin,_) = signed_registry_and_bundle();
        registry["payload"]["keys"][0]["status"] = Value::String("revoked".into());
        assert!(verify_signed_registry(&registry,&pin).unwrap_err().0.contains("signature verification failed"));
    }
    #[test] fn accepts_update_with_valid_chain_and_higher_version() {
        let (registry,current,candidate,pin,_) = signed_registry_and_bundle();
        let verified = verify_signed_registry(&registry,&pin).unwrap();
        assert!(verify_update(&current,&candidate,&verified).is_ok());
    }
    #[test] fn rejects_update_with_wrong_parent_hash() {
        let (registry,current,mut candidate,pin,_) = signed_registry_and_bundle();
        candidate["payload"]["parent_bundle_sha256"] = Value::String("f".repeat(64));
        let verified = verify_signed_registry(&registry,&pin).unwrap();
        assert!(verify_update(&current,&candidate,&verified).unwrap_err().0.contains("parent hash"));
    }
    #[test] fn rejects_update_with_non_increasing_version() {
        let (registry,current,mut candidate,pin,key) = signed_registry_and_bundle();
        candidate["payload"]["rulebook_version"] = Value::String("0.1.0".into());
        let payload = &candidate["payload"];
        let sig = key.sign(&serde_json::to_vec(payload).unwrap());
        candidate["signature_base64"] = Value::String(STANDARD.encode(sig.to_bytes()));
        let verified = verify_signed_registry(&registry,&pin).unwrap();
        assert!(verify_update(&current,&candidate,&verified).unwrap_err().0.contains("greater than current"));
    }
}
