mod authority;
mod lexer;
mod rulebook;
mod token;

use std::{env, fs, process::ExitCode};
use lexer::Lexer;
use rulebook::{RecordKind, validate};
use authority::verify_signed_bundle;

fn print_usage() {
    eprintln!("AL Compiler 0.1.0");
    eprintln!("Usage:");
    eprintln!("  al <source.al>");
    eprintln!("  al rulebook validate <bundle|metric-card|source-record|decision-record|training-run-manifest> <file.json>");
    eprintln!("  al rulebook verify-signed <envelope.json> <trusted-keys.json>");
}

fn run_rulebook(args: &[String]) -> ExitCode {
    if args.len()==4 && args[1]=="verify-signed" {\n        let envelope_text=match fs::read_to_string(&args[2]) { Ok(s)=>s, Err(e)=>{eprintln!("Could not read {}: {e}",args[2]);return ExitCode::FAILURE;} };\n        let registry_text=match fs::read_to_string(&args[3]) { Ok(s)=>s, Err(e)=>{eprintln!("Could not read {}: {e}",args[3]);return ExitCode::FAILURE;} };\n        let envelope:serde_json::Value=match serde_json::from_str(&envelope_text) { Ok(v)=>v, Err(e)=>{eprintln!("Invalid envelope JSON: {e}");return ExitCode::FAILURE;} };\n        let registry:serde_json::Value=match serde_json::from_str(&registry_text) { Ok(v)=>v, Err(e)=>{eprintln!("Invalid trusted-key JSON: {e}");return ExitCode::FAILURE;} };\n        return match verify_signed_bundle(&envelope,&registry) { Ok(())=>{println!("AUTHORIZED: signature and human approval verified");ExitCode::SUCCESS}, Err(e)=>{eprintln!("DENIED: {e}");ExitCode::FAILURE} };\n    }\n    if args.len()!=4 || args[1]!="validate" { print_usage(); return ExitCode::from(2); }
    let Some(kind)=RecordKind::parse(&args[2]) else {
        eprintln!("Unknown Rulebook record type: {}",args[2]); print_usage(); return ExitCode::from(2);
    };
    let path=&args[3];
    let source=match fs::read_to_string(path) {
        Ok(source)=>source,
        Err(error)=>{eprintln!("Could not read {path}: {error}");return ExitCode::FAILURE;}
    };
    let value:serde_json::Value=match serde_json::from_str(&source) {
        Ok(value)=>value,
        Err(error)=>{eprintln!("Invalid JSON in {path}: {error}");return ExitCode::FAILURE;}
    };
    let report=validate(kind,&value);
    for warning in &report.warnings { eprintln!("WARNING: {warning}"); }
    if report.is_valid() {
        println!("VALID: {path}");
        println!("Warnings: {}",report.warnings.len());
        ExitCode::SUCCESS
    } else {
        for error in &report.errors { eprintln!("ERROR: {error}"); }
        eprintln!("INVALID: {} error(s), {} warning(s)",report.errors.len(),report.warnings.len());
        ExitCode::FAILURE
    }
}

fn main()->ExitCode {
    let args:Vec<String>=env::args().skip(1).collect();
    if args.first().is_some_and(|arg|arg=="rulebook") { return run_rulebook(&args); }
    if args.len()!=1 { print_usage(); return ExitCode::from(2); }
    let path=&args[0];
    let source=match fs::read_to_string(path) {
        Ok(source)=>source,
        Err(error)=>{eprintln!("Could not read {path}: {error}");return ExitCode::FAILURE;}
    };
    let mut lexer=Lexer::new(&source);
    match lexer.tokenize() {
        Ok(tokens)=>{
            println!("AL Compiler 0.1.0 — lexical analysis");
            for token in tokens { println!("{:>5}..{:<5} {}",token.span.start,token.span.end,token.kind); }
            ExitCode::SUCCESS
        }
        Err(error)=>{eprintln!("{error}");ExitCode::FAILURE}
    }
}
