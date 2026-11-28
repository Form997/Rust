mod authority;
mod attestation_anchor;
mod control_evidence;
mod evidence_audit;
mod lexer;
mod policy_gate;
mod rulebook;
mod release_gate;
mod release_routes;
mod shared_gate;
mod token;

use std::{env, fs, path::Path, process::ExitCode};
use authority::{verify_signed_bundle, verify_signed_registry, verify_update};
use attestation_anchor::{create_audit_anchor, verify_audit_anchor, verify_github_attestation};
use control_evidence::verify_control_evidence;
use evidence_audit::{append_audit_event, verify_audit_chain, verify_evidence};
use lexer::Lexer;
use policy_gate::{Gate, authorize_gate};
use rulebook::{RecordKind, validate};
use release_gate::{evaluate_release_artifact, evaluate_release_artifact_audited};
use release_routes::{assess_release_route_inventory, RouteInventoryDecision};
use shared_gate::{evaluate_gate, GateRequest, VerifiedRegistry};
use serde_json::Value;

fn print_usage() {
    eprintln!("AL Compiler 0.1.0");
    eprintln!("Usage:");
    eprintln!("  al <source.al>");
    eprintln!("  al rulebook validate <record-kind> <file.json>");
    eprintln!("  al rulebook verify-signed <envelope.json> <verified-registry.json>");
    eprintln!("  al rulebook verify-registry <signed-registry.json> <pinned-root-fingerprint-hex>");
    eprintln!("  al rulebook verify-update <current-payload.json> <candidate-envelope.json> <signed-registry.json> <pinned-root-fingerprint-hex>");
    eprintln!("  al rulebook compare <current-payload.json> <candidate-payload.json>");
    eprintln!("  al rulebook gate <gate> <signed-envelope.json> <signed-registry.json> <pinned-root-fingerprint-hex>");
    eprintln!("  al rulebook gate-evaluate <subsystem> <operation> <subject-type> <subject-sha256> <evaluated-at-utc> <decision-id> <verifier-version> <evidence-refs.json> <signed-envelope.json> <signed-registry.json> <pinned-root-fingerprint-hex>");
    eprintln!("  al rulebook release-gate <build|package|publish_release> <artifact-file> <evaluated-at-utc> <decision-id> <verifier-version> <evidence-refs.json> <signed-envelope.json> <signed-registry.json> <pinned-root-fingerprint-hex>");
    eprintln!("  al rulebook release-gate-audited <build|package|publish_release> <artifact-file> <evaluated-at-utc> <decision-id> <verifier-version> <evidence-refs.json> <signed-envelope.json> <signed-registry.json> <pinned-root-fingerprint-hex> <audit-ledger.jsonl>");
    eprintln!("  al rulebook release-routes-verify <route-inventory.json>");
    eprintln!("  al rulebook evidence-verify <manifest.json> <artifact-file> <owner/repo> <now-utc>");
    eprintln!("  al rulebook audit-append <ledger.jsonl> <event.json>");
    eprintln!("  al rulebook audit-verify <ledger.jsonl>");
    eprintln!("  al rulebook audit-anchor-create <ledger.jsonl> <anchor.json>");
    eprintln!("  al rulebook audit-anchor-verify <ledger.jsonl> <anchor.json>");
    eprintln!("  al rulebook attestation-verify <subject-file> <owner/repo> <signer-workflow>");
    eprintln!("The root fingerprint must come from separately protected deployment configuration.");
}
fn read_json(path: &str) -> Result<Value,String> {
    let text = fs::read_to_string(path).map_err(|e| format!("Could not read {path}: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("Invalid JSON in {path}: {e}"))
}
fn run_rulebook(args: &[String]) -> ExitCode {
    match args.get(1).map(String::as_str) {
        Some("evidence-verify") if args.len()==6 => {
            let manifest=match read_json(&args[2]) {Ok(v)=>v,Err(e)=>{eprintln!("{e}");return ExitCode::FAILURE;}};
            match verify_evidence(&manifest,Path::new(&args[3]),&args[4],&args[5]) {
                Ok(message)=>{println!("EVIDENCE VERIFIED: {message}");ExitCode::SUCCESS},
                Err(e)=>{eprintln!("EVIDENCE DENIED: {e}");ExitCode::FAILURE}
            }
        }
        Some("audit-anchor-create") if args.len()==4 => {
            match create_audit_anchor(Path::new(&args[2]),Path::new(&args[3])) {
                Ok(anchor)=>{println!("AUDIT ANCHOR CREATED: {}",anchor);ExitCode::SUCCESS},
                Err(e)=>{eprintln!("ANCHOR DENIED: {e}");ExitCode::FAILURE}
            }
        }
        Some("audit-anchor-verify") if args.len()==4 => {
            match verify_audit_anchor(Path::new(&args[2]),Path::new(&args[3])) {
                Ok(())=>{println!("AUDIT ANCHOR VERIFIED");ExitCode::SUCCESS},
                Err(e)=>{eprintln!("ANCHOR INVALID: {e}");ExitCode::FAILURE}
            }
        }
        Some("attestation-verify") if args.len()==5 => {
            match verify_github_attestation(Path::new(&args[2]),&args[3],&args[4],"https://slsa.dev/provenance/v1") {
                Ok(message)=>{println!("ATTESTATION VERIFIED: {message}");ExitCode::SUCCESS},
                Err(e)=>{eprintln!("ATTESTATION DENIED: {e}");ExitCode::FAILURE}
            }
        }
        Some("audit-append") if args.len()==4 => {
            let event=match read_json(&args[3]) {Ok(v)=>v,Err(e)=>{eprintln!("{e}");return ExitCode::FAILURE;}};
            match append_audit_event(Path::new(&args[2]),&event) {
                Ok(hash)=>{println!("AUDIT APPENDED: record_hash={hash}");ExitCode::SUCCESS},
                Err(e)=>{eprintln!("AUDIT DENIED: {e}");ExitCode::FAILURE}
            }
        }
        Some("audit-verify") if args.len()==3 => {
            match verify_audit_chain(Path::new(&args[2])) {
                Ok(count)=>{println!("AUDIT VALID: {count} record(s)");ExitCode::SUCCESS},
                Err(e)=>{eprintln!("AUDIT INVALID: {e}");ExitCode::FAILURE}
            }
        }
        Some("verify-registry") if args.len()==4 => {
            let envelope=match read_json(&args[2]) {Ok(v)=>v,Err(e)=>{eprintln!("{e}");return ExitCode::FAILURE;}};
            match verify_signed_registry(&envelope,&args[3]) {
                Ok(registry)=>{println!("TRUSTED REGISTRY: version {}",registry["registry_version"].as_str().unwrap_or("unknown"));ExitCode::SUCCESS},
                Err(e)=>{eprintln!("DENIED: {e}");ExitCode::FAILURE}
            }
        }
        Some("verify-signed") if args.len()==4 => {
            let envelope=match read_json(&args[2]) {Ok(v)=>v,Err(e)=>{eprintln!("{e}");return ExitCode::FAILURE;}};
            let registry=match read_json(&args[3]) {Ok(v)=>v,Err(e)=>{eprintln!("{e}");return ExitCode::FAILURE;}};
            match verify_signed_bundle(&envelope,&registry) {
                Ok(())=>{println!("AUTHORIZED: signature and human approval verified");ExitCode::SUCCESS},
                Err(e)=>{eprintln!("DENIED: {e}");ExitCode::FAILURE}
            }
        }
        Some("verify-update") if args.len()==6 => {
            let current=match read_json(&args[2]) {Ok(v)=>v,Err(e)=>{eprintln!("{e}");return ExitCode::FAILURE;}};
            let candidate=match read_json(&args[3]) {Ok(v)=>v,Err(e)=>{eprintln!("{e}");return ExitCode::FAILURE;}};
            let signed_registry=match read_json(&args[4]) {Ok(v)=>v,Err(e)=>{eprintln!("{e}");return ExitCode::FAILURE;}};
            let registry=match verify_signed_registry(&signed_registry,&args[5]) {Ok(v)=>v,Err(e)=>{eprintln!("DENIED: trust registry: {e}");return ExitCode::FAILURE;}};
            match verify_update(&current,&candidate,&registry) {
                Ok(())=>{println!("UPDATE VERIFIED: signature, authority, parent hash and version progression passed");ExitCode::SUCCESS},
                Err(e)=>{eprintln!("UPDATE DENIED: {e}");ExitCode::FAILURE}
            }
        }
        Some("release-gate") if args.len()==11 => {
            let evidence_value=match read_json(&args[7]) {Ok(v)=>v,Err(e)=>{eprintln!("{e}");return ExitCode::FAILURE;}};
            let Some(items)=evidence_value.as_array() else {eprintln!("evidence-refs file must contain a JSON array of strings");return ExitCode::FAILURE;};
            let mut evidence_refs=Vec::new();
            for item in items {
                let Some(id)=item.as_str() else {eprintln!("evidence-refs array must contain only strings");return ExitCode::FAILURE;};
                evidence_refs.push(id.to_owned());
            }
            let envelope=match read_json(&args[8]) {Ok(v)=>v,Err(e)=>{eprintln!("{e}");return ExitCode::FAILURE;}};
            let signed_registry=match read_json(&args[9]) {Ok(v)=>v,Err(e)=>{eprintln!("{e}");return ExitCode::FAILURE;}};
            let registry=match VerifiedRegistry::from_signed(&signed_registry,&args[10]) {
                Ok(v)=>v,Err(e)=>{eprintln!("RELEASE DENIED: trust registry: {e}");return ExitCode::FAILURE;}
            };
            match evaluate_release_artifact(&args[2],Path::new(&args[3]),&args[4],&args[5],&args[6],evidence_refs,&envelope,&registry) {
                Ok(decision)=>{
                    let rendered=serde_json::to_string_pretty(&decision).unwrap_or_else(|_| String::from("SERIALIZATION_FAILURE"));
                    println!("{rendered}");
                    if decision.decision.as_str()=="ALLOW" {ExitCode::SUCCESS} else {ExitCode::FAILURE}
                },
                Err(e)=>{eprintln!("RELEASE GATE DENIED: {e}");ExitCode::from(2)}
            }
        }
        Some("release-routes-verify") if args.len()==3 => {
            let inventory=match read_json(&args[2]) {Ok(v)=>v,Err(e)=>{eprintln!("{e}");return ExitCode::FAILURE;}};
            match assess_release_route_inventory(&inventory) {
                RouteInventoryDecision::Review(reason)=>{eprintln!("REVIEW: {reason}");ExitCode::from(2)},
                RouteInventoryDecision::Deny(reason)=>{eprintln!("DENY: {reason}");ExitCode::FAILURE}
            }
        }
        Some("release-gate-audited") if args.len()==12 => {
            let evidence_value=match read_json(&args[7]) {Ok(v)=>v,Err(e)=>{eprintln!("{e}");return ExitCode::FAILURE;}};
            let Some(items)=evidence_value.as_array() else {eprintln!("evidence-refs file must contain a JSON array of strings");return ExitCode::FAILURE;};
            let mut evidence_refs=Vec::new();
            for item in items {
                let Some(id)=item.as_str() else {eprintln!("evidence-refs array must contain only strings");return ExitCode::FAILURE;};
                evidence_refs.push(id.to_owned());
            }
            let envelope=match read_json(&args[8]) {Ok(v)=>v,Err(e)=>{eprintln!("{e}");return ExitCode::FAILURE;}};
            let signed_registry=match read_json(&args[9]) {Ok(v)=>v,Err(e)=>{eprintln!("{e}");return ExitCode::FAILURE;}};
            let registry=match VerifiedRegistry::from_signed(&signed_registry,&args[10]) {
                Ok(v)=>v,Err(e)=>{eprintln!("RELEASE DENIED: trust registry: {e}");return ExitCode::FAILURE;}
            };
            match evaluate_release_artifact_audited(
                &args[2],Path::new(&args[3]),&args[4],&args[5],&args[6],
                evidence_refs,&envelope,&registry,Path::new(&args[11])
            ) {
                Ok(decision)=>{
                    let rendered=serde_json::to_string_pretty(&decision).unwrap_or_else(|_| String::from("SERIALIZATION_FAILURE"));
                    println!("{rendered}");
                    if decision.decision.as_str()=="ALLOW" {ExitCode::SUCCESS} else {ExitCode::FAILURE}
                },
                Err(e)=>{eprintln!("RELEASE DENIED: {e}");ExitCode::from(2)}
            }
        }
        Some("gate-evaluate") if args.len()==13 => {
            let evidence_value=match read_json(&args[9]) {Ok(v)=>v,Err(e)=>{eprintln!("{e}");return ExitCode::FAILURE;}};
            let Some(evidence_items)=evidence_value.as_array() else {eprintln!("evidence-refs file must contain a JSON array of strings");return ExitCode::FAILURE;};
            let mut evidence_refs=Vec::new();
            for item in evidence_items {
                let Some(id)=item.as_str() else {eprintln!("evidence-refs array must contain only strings");return ExitCode::FAILURE;};
                evidence_refs.push(id.to_owned());
            }
            let envelope=match read_json(&args[10]) {Ok(v)=>v,Err(e)=>{eprintln!("{e}");return ExitCode::FAILURE;}};
            let signed_registry=match read_json(&args[11]) {Ok(v)=>v,Err(e)=>{eprintln!("{e}");return ExitCode::FAILURE;}};
            let registry=match VerifiedRegistry::from_signed(&signed_registry,&args[12]) {
                Ok(v)=>v,Err(e)=>{eprintln!("DENIED: {e}");return ExitCode::FAILURE;}
            };
            let request=GateRequest {
                subsystem:args[2].clone(), operation:args[3].clone(), subject_type:args[4].clone(),
                subject_sha256:args[5].clone(), evaluated_at:args[6].clone(), decision_id:args[7].clone(),
                verifier_version:args[8].clone(), evidence_refs,
            };
            match evaluate_gate(&request,&envelope,&registry) {
                Ok(decision)=>{
                    let rendered=serde_json::to_string_pretty(&decision).unwrap_or_else(|_| String::from("SERIALIZATION_FAILURE"));
                    println!("{rendered}");
                    if decision.decision.as_str()=="ALLOW" {ExitCode::SUCCESS} else {ExitCode::FAILURE}
                },
                Err(e)=>{eprintln!("GATE REQUEST INVALID: {e}");ExitCode::from(2)}
            }
        }
        Some("gate") if args.len()==6 => {
            let Some(gate)=Gate::parse(&args[2]) else {eprintln!("DENIED: unknown subsystem gate '{}'",args[2]);return ExitCode::from(2);};
            let envelope=match read_json(&args[3]) {Ok(v)=>v,Err(e)=>{eprintln!("{e}");return ExitCode::FAILURE;}};
            let signed_registry=match read_json(&args[4]) {Ok(v)=>v,Err(e)=>{eprintln!("{e}");return ExitCode::FAILURE;}};
            let registry=match verify_signed_registry(&signed_registry,&args[5]) {Ok(v)=>v,Err(e)=>{eprintln!("DENIED: trust registry: {e}");return ExitCode::FAILURE;}};
            match authorize_gate(&envelope,&registry,gate) {
                Ok(permit)=>{
                    let payload=match envelope.get("payload"){Some(v)=>v,None=>{eprintln!("DENIED: signed Rulebook payload is missing");return ExitCode::FAILURE;}};
                    let controls=match verify_control_evidence(payload,gate){Ok(ids)=>ids,Err(e)=>{eprintln!("DENIED: control evidence: {e}");return ExitCode::FAILURE;}};
                    println!("PERMITTED: gate={} rulebook={} rules={} controls={}",permit.gate.as_str(),permit.rulebook_version,permit.rule_ids.join(","),controls.join(","));
                    ExitCode::SUCCESS
                },
                Err(e)=>{eprintln!("DENIED: {}",e);ExitCode::FAILURE}
            }
        }
        Some("compare") if args.len()==4 => {
            let current=match read_json(&args[2]) {Ok(v)=>v,Err(e)=>{eprintln!("{e}");return ExitCode::FAILURE;}};
            let candidate=match read_json(&args[3]) {Ok(v)=>v,Err(e)=>{eprintln!("{e}");return ExitCode::FAILURE;}};
            let old=current.as_object(); let new=candidate.as_object();
            let (Some(old),Some(new))=(old,new) else {eprintln!("Both inputs must be JSON objects");return ExitCode::FAILURE;};
            let keys:std::collections::BTreeSet<_>=old.keys().chain(new.keys()).collect();
            let mut changes=0usize;
            for key in keys { if old.get(key)!=new.get(key) { changes+=1; println!("CHANGED {key}: {} -> {}",old.get(key).unwrap_or(&Value::Null),new.get(key).unwrap_or(&Value::Null)); } }
            println!("SUMMARY: {changes} top-level field(s) changed. This is a review aid, not approval.");
            ExitCode::SUCCESS
        }
        Some("validate") if args.len()==4 => {
            let Some(kind)=RecordKind::parse(&args[2]) else {eprintln!("Unknown record kind: {}",args[2]);return ExitCode::from(2);};
            let value=match read_json(&args[3]) {Ok(v)=>v,Err(e)=>{eprintln!("{e}");return ExitCode::FAILURE;}};
            let report=validate(kind,&value);
            for warning in &report.warnings {eprintln!("WARNING: {warning}");}
            if report.is_valid(){println!("VALID: {} ({} warning(s))",args[3],report.warnings.len());ExitCode::SUCCESS}
            else {for error in &report.errors {eprintln!("ERROR: {error}");}ExitCode::FAILURE}
        }
        _=>{print_usage();ExitCode::from(2)}
    }
}
fn main()->ExitCode {
    let args:Vec<String>=env::args().skip(1).collect();
    if args.first().is_some_and(|a|a=="rulebook"){return run_rulebook(&args);}
    if args.len()!=1 {print_usage();return ExitCode::from(2);}
    let path=&args[0];
    let source=match fs::read_to_string(path){Ok(s)=>s,Err(e)=>{eprintln!("Could not read {path}: {e}");return ExitCode::FAILURE;}};
    let mut lexer=Lexer::new(&source);
    match lexer.tokenize(){Ok(tokens)=>{println!("AL Compiler 0.1.0 — lexical analysis");for token in tokens{println!("{:>5}..{:<5} {}",token.span.start,token.span.end,token.kind);}ExitCode::SUCCESS},Err(e)=>{eprintln!("{e}");ExitCode::FAILURE}}
}
