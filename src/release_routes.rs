//! Fail-closed assessment of release route inventory completeness.
//! A repository fixture cannot prove that every external deployment channel
//! has been inventoried; only a separately reviewed and attested inventory can.
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteInventoryDecision {
    Review(String),
    Deny(String),
}

pub fn assess_release_route_inventory(value: &Value) -> RouteInventoryDecision {
    let Some(object) = value.as_object() else {
        return RouteInventoryDecision::Deny("inventory must be a JSON object".into());
    };
    if object.get("schema_version").and_then(Value::as_str) != Some("0.1.0") {
        return RouteInventoryDecision::Deny("unsupported inventory schema version".into());
    }
    let status = object.get("inventory_status").and_then(Value::as_str);
    let Some(routes) = object.get("routes").and_then(Value::as_array) else {
        return RouteInventoryDecision::Deny("routes must be an array".into());
    };
    if status == Some("NOT_SET") {
        return RouteInventoryDecision::Review(
            "release route inventory is NOT_SET; complete external channel discovery before claiming coverage".into()
        );
    }
    if status != Some("verified") {
        return RouteInventoryDecision::Review(
            "release route inventory is not verified".into()
        );
    }
    if routes.is_empty() {
        return RouteInventoryDecision::Review("verified inventory cannot be empty".into());
    }
    for route in routes {
        if route.get("mandatory_gate").and_then(Value::as_bool) != Some(true) {
            return RouteInventoryDecision::Deny("a release route does not require the Rulebook gate".into());
        }
        if route.get("bypass_test_status").and_then(Value::as_str) != Some("passed") {
            return RouteInventoryDecision::Review("a release route lacks a passing bypass test".into());
        }
        if route.get("gate_adapter").and_then(Value::as_str).is_none_or(|s| s.trim().is_empty()) {
            return RouteInventoryDecision::Deny("a release route has no gate adapter".into());
        }
        if route.get("evidence_ids").and_then(Value::as_array).is_none_or(Vec::is_empty) {
            return RouteInventoryDecision::Review("a release route has no evidence references".into());
        }
    }
    // Even a structurally complete JSON inventory is only an assertion. The
    // cryptographic attestation and independent completeness review are not
    // implemented in H0.2.10.3, so this code deliberately never returns ALLOW.
    RouteInventoryDecision::Review(
        "inventory structure passes, but independent completeness attestation is not implemented".into()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn r04_not_set_inventory_is_review_not_allow() {
        let inventory = json!({"schema_version":"0.1.0","inventory_status":"NOT_SET",
            "owner":"NOT_SET","reviewed_at":null,"routes":[],"coverage_evidence":[]});
        assert!(matches!(assess_release_route_inventory(&inventory), RouteInventoryDecision::Review(_)));
    }
    #[test]
    fn r04_route_without_mandatory_gate_is_denied() {
        let inventory = json!({"schema_version":"0.1.0","inventory_status":"verified",
            "owner":"release-owner","reviewed_at":"2026-11-28T00:00:00Z",
            "routes":[{"route_id":"R-1","mandatory_gate":false,"bypass_test_status":"passed",
                "gate_adapter":"al rulebook release-gate","evidence_ids":["E-1"]}],
            "coverage_evidence":["E-COVERAGE"]});
        assert!(matches!(assess_release_route_inventory(&inventory), RouteInventoryDecision::Deny(_)));
    }
    #[test]
    fn r04_structural_inventory_never_claims_independent_completeness() {
        let inventory = json!({"schema_version":"0.1.0","inventory_status":"verified",
            "owner":"release-owner","reviewed_at":"2026-11-28T00:00:00Z",
            "routes":[{"route_id":"R-1","mandatory_gate":true,"bypass_test_status":"passed",
                "gate_adapter":"al rulebook release-gate","evidence_ids":["E-1"]}],
            "coverage_evidence":["E-COVERAGE"]});
        let result = assess_release_route_inventory(&inventory);
        assert!(matches!(result, RouteInventoryDecision::Review(_)));
    }
}
