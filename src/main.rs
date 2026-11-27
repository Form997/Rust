mod authority;
mod lexer;
mod policy_gate;
mod rulebook;
mod token;

use std::{env, fs, process::ExitCode};
use authority::{verify_signed_bundle, verify_signed_registry, verify_update};
use lexer::Lexer;
use policy_gate::{Gate, authorize_gate};
use rulebook::{RecordKind, validate};
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
    eprintln!("  al rulebook gate <compiler|runtime|ai|training|evaluation|scientific|release> <signed-envelope.json> <signed-registry.json> <pinned-root-fingerprint-hex>");
    eprintln!("The root fingerprint must come from separately protected deployment configuration.");
}
fn read_json(path: &str) -> Result<Value,String> {
    let text = fs::read_to_string(path).map_err(|e| format!("Could not read {path}: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("Invalid JSON in {path}: {e}"))
}
fn run_rulebook(args: &[String]) -> ExitCode {
    match args.get(1).map(String::as_str) {
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
        Some("gate") if args.len()==6 => {
            let Some(gate)=Gate::parse(&args[2]) else {
                eprintln!("DENIED: unknown subsystem gate '{}'",args[2]);
                return ExitCode::from(2);
            };
            let envelope=match read_json(&args[3]) {Ok(v)=>v,Err(e)=>{eprintln!("{e}");return ExitCode::FAILURE;}};
            let signed_registry=match read_json(&args[4]) {Ok(v)=>v,Err(e)=>{eprintln!("{e}");return ExitCode::FAILURE;}};
            let registry=match verify_signed_registry(&signed_registry,&args[5]) {
                Ok(v)=>v,Err(e)=>{eprintln!("DENIED: trust registry: {e}");return ExitCode::FAILURE;}
            };
            match authorize_gate(&envelope,&registry,gate) {
                Ok(permit)=>{
                    println!("PERMITTED: gate={} rulebook={} rules={}",permit.gate.as_str(),permit.rulebook_version,permit.rule_ids.join(","));
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
