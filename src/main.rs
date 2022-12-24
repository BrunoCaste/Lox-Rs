#![feature(is_some_and)]

use std::{
    env,
    fs::read_to_string,
    io::{stdin, stdout, Write},
    process::ExitCode,
};

use lexer::Lexer;
use parser::{Parser, RecursiveDescent};

mod expr;
mod lexer;
mod parser;

fn usage(prog: String) -> ExitCode {
    eprintln!("Usage: {prog} [script]");
    ExitCode::from(64)
}

fn run_file(path: &str) -> ExitCode {
    let src = match read_to_string(path) {
        Ok(src) => src,
        Err(_) => {
            println!("ERROR: unable to open file: {path}");
            return ExitCode::from(74);
        }
    };
    run(&src).unwrap_or(ExitCode::from(0))
}

fn repl() -> ExitCode {
    let stdin = stdin();
    let mut input = String::with_capacity(64);
    loop {
        input.clear();
        print!("> ");
        stdout().flush().expect("Error flushing stdout");
        stdin
            .read_line(&mut input)
            .expect("Error reading from stdin");

        if let Some(e) = run(&input) {
            return e;
        }
    }
}

fn run(src: &str) -> Option<ExitCode> {
    let mut lexer = Lexer::new(src.chars()).peekable();
    let expr = match RecursiveDescent::parse(&mut lexer) {
        Ok(e) => e,
        e => {
            println!("syntax error\t{e:?}");
            return Some(ExitCode::from(1));
        }
    };

    match expr.eval() {
        Ok(v) => {
            println!("{v}");
            None
        }
        e => {
            println!("runtime error\t{e:?}");
            Some(ExitCode::from(1))
        }
    }
}

fn main() -> ExitCode {
    let mut args = env::args();
    let prog = args.next().expect("Program name must always be present");
    let args: Vec<_> = args.collect();
    match &args[..] {
        [] => repl(),
        [script] => run_file(script),
        _ => usage(prog),
    }
}
