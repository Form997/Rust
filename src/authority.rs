//! Cryptographic authority gate for activating AL Rulebook bundles.
//!
//! The private signing key must remain with the authorized human maintainer.
//! This module only verifies signatures; it never creates or stores private keys.

use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorityError(pub String);

impl std::fmt::Display for AuthorityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for AuthorityError {}

fn required_text<'a>(value: &'a Value, field: &str) -> Result<&'a str, AuthorityError> {
    value.get(field).and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| AuthorityError(format!("missing or invalid field: {field}")))
}

/// Verify a signed Rulebook envelope against an out-of-band trusted public-key registry.
///
/// Signed bytes are the compact UTF-8 JSON serialization of the payload. serde_json's
/// default map representation is key-sorted; arrays retain their order. Producers must
/// use the same canonicalization rule before signing.
pub fn verify_signed_bundle(envelope: &Value, trusted_registry: &Value) -> Result<(), AuthorityError> {
    let algorithm = required_text(envelope, "algorithm")?;
    if algorithm != "Ed25519" {
        return Err(AuthorityError(format!("unsupported signature algorithm: {algorithm}")));
    }

    let key_id = required_text(envelope, "key_id")?;
    let signature_text = required_text(envelope, "signature_base64")?;
    let payload = envelope.get("payload")
        .ok_or_else(|| AuthorityError("missing payload".into()))?;

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
    let principal_id = required_text(key, "principal_id")?;
    if required_text(payload, "status")? != "approved" {
        return Err(AuthorityError("only an approved Rulebook payload may be activated".into()));
    }

    let approval = payload.get("approval")
        .ok_or_else(|| AuthorityError("payload is missing approval record".into()))?;
    if required_text(approval, "state")? != "approved" {
        return Err(AuthorityError("payload approval state is not approved".into()));
    }
    if required_text(approval, "approver")? != principal_id {
        return Err(AuthorityError("approval identity does not match trusted signing principal".into()));
    }
    required_text(approval, "approved_at")?;

    let public_key_bytes: [u8; 32] = STANDARD.decode(required_text(key, "public_key_base64")?)
        .map_err(|_| AuthorityError("trusted public key is not valid base64".into()))?
        .try_into().map_err(|_| AuthorityError("Ed25519 public key must be 32 bytes".into()))?;
    let verifying_key = VerifyingKey::from_bytes(&public_key_bytes)
        .map_err(|_| AuthorityError("trusted Ed25519 public key is invalid".into()))?;

    let signature_bytes = STANDARD.decode(signature_text)
        .map_err(|_| AuthorityError("signature is not valid base64".into()))?;
    let signature = Signature::from_slice(&signature_bytes)
        .map_err(|_| AuthorityError("Ed25519 signature must be 64 bytes".into()))?;
    let canonical_payload = serde_json::to_vec(payload)
        .map_err(|error| AuthorityError(format!("cannot canonicalize payload: {error}")))?;
    verifying_key.verify(&canonical_payload, &signature)
        .map_err(|_| AuthorityError("Rulebook signature verification failed".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use serde_json::json;

    fn signed_fixture() -> (Value, Value) {
        let signing_key = SigningKey::from_bytes(&[7u8; 32]);
        let payload = json!({
            "bundle_id": "al-rulebook-test",
            "rulebook_version": "0.1.0",
            "status": "approved",
            "approval": {
                "state": "approved",
                "approver": "human-maintainer-001",
                "approved_at": "2026-11-27T00:00:00Z"
            }
        });
        let bytes = serde_json::to_vec(&payload).unwrap();
        let signature = signing_key.sign(&bytes);
        let envelope = json!({
            "algorithm": "Ed25519",
            "key_id": "root-maintainer-001",
            "payload": payload,
            "signature_base64": STANDARD.encode(signature.to_bytes())
        });
        let registry = json!({
            "registry_version": "0.1.0",
            "keys": [{
                "key_id": "root-maintainer-001",
                "principal_id": "human-maintainer-001",
                "role": "rulebook_authority",
                "status": "active",
                "public_key_base64": STANDARD.encode(signing_key.verifying_key().as_bytes())
            }]
        });
        (envelope, registry)
    }

    #[test]
    fn accepts_valid_human_authority_signature() {
        let (envelope, registry) = signed_fixture();
        assert!(verify_signed_bundle(&envelope, &registry).is_ok());
    }

    #[test]
    fn rejects_payload_tampering() {
        let (mut envelope, registry) = signed_fixture();
        envelope["payload"]["rulebook_version"] = Value::String("9.9.9".into());
        assert!(verify_signed_bundle(&envelope, &registry).unwrap_err().0.contains("signature verification failed"));
    }

    #[test]
    fn rejects_unknown_signing_key() {
        let (mut envelope, registry) = signed_fixture();
        envelope["key_id"] = Value::String("unknown".into());
        assert!(verify_signed_bundle(&envelope, &registry).unwrap_err().0.contains("not trusted"));
    }

    #[test]
    fn rejects_revoked_signing_key() {
        let (envelope, mut registry) = signed_fixture();
        registry["keys"][0]["status"] = Value::String("revoked".into());
        assert!(verify_signed_bundle(&envelope, &registry).unwrap_err().0.contains("not active"));
    }

    #[test]
    fn rejects_approval_by_different_person() {
        let (mut envelope, registry) = signed_fixture();
        envelope["payload"]["approval"]["approver"] = Value::String("other-person".into());
        assert!(verify_signed_bundle(&envelope, &registry).unwrap_err().0.contains("identity does not match"));
    }

    #[test]
    fn rejects_non_approved_payload() {
        let (mut envelope, registry) = signed_fixture();
        envelope["payload"]["status"] = Value::String("proposed".into());
        assert!(verify_signed_bundle(&envelope, &registry).unwrap_err().0.contains("only an approved"));
    }
}
