mod lexer;
mod rulebook;
mod token;

use std::{env, fs, process::ExitCode};
use lexer::Lexer;
use rulebook::{RecordKind, validate};

fn print_usage() {
    eprintln!("AL Compiler 0.1.0");
    eprintln!("Usage:");
    eprintln!("  al <source.al>");
    eprintln!("  al rulebook validate <bundle|metric-card|source-record|decision-record|training-run-manifest> <file.json>");
}

fn run_rulebook(args: &[String]) -> ExitCode {
    if args.len()!=4 || args[1]!="validate" { print_usage(); return ExitCode::from(2); }
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
