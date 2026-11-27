mod diagnostic;
mod lexer;
mod source;
mod token;

use std::{env, fs, process::ExitCode};

use diagnostic::Diagnostic;
use lexer::Lexer;
use source::SourceMap;
use token::Span;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(path) = args.next() else {
        eprintln!("AL Compiler 0.1.0");
        eprintln!("Usage: al <source.al>");
        return ExitCode::from(2);
    };
    if args.next().is_some() {
        eprintln!("Usage: al <source.al>");
        return ExitCode::from(2);
    }

    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("Could not read {path}: {error}");
            return ExitCode::FAILURE;
        }
    };

    let source_map = SourceMap::new(&source);
    match Lexer::new(&source).tokenize() {
        Ok(tokens) => {
            println!("AL Compiler 0.1.0 — lexical analysis");
            for token in tokens {
                println!("{:>5}..{:<5} {}", token.span.start, token.span.end, token.kind);
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            let diagnostic = Diagnostic::error(
                "AL1001",
                error.message,
                Some(Span::new(error.position, error.position)),
            );
            eprintln!("{}", diagnostic.render(&source_map));
            ExitCode::FAILURE
        }
    }
}
