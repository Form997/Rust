mod lexer;
mod token;

use std::{env, fs, process::ExitCode};

use lexer::Lexer;

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

    let mut lexer = Lexer::new(&source);

    match lexer.tokenize() {
        Ok(tokens) => {
            println!("AL Compiler 0.1.0 — lexical analysis");
            for token in tokens {
                println!("{:>5}..{:<5} {}", token.span.start, token.span.end, token.kind);
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
