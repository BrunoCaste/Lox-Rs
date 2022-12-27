#![feature(is_some_and)]

use std::{
    env,
    fs::read_to_string,
    io::{stdin, stdout, Write},
    process::ExitCode,
};

use lexer::Lexer;
use parser::{Parser, RecursiveDescent};
use prog::Scope;

mod expr;
mod lexer;
mod parser;
mod prog;
mod stmt;

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
    run(&src, &mut Scope::new()).unwrap_or(ExitCode::from(0))
}

fn repl() -> ExitCode {
    let stdin = stdin();
    let mut input = String::with_capacity(64);

    let mut env = Scope::new();

    loop {
        input.clear();
        print!("> ");
        stdout().flush().expect("Error flushing stdout");
        stdin
            .read_line(&mut input)
            .expect("Error reading from stdin");

        if let Some(e) = run(&input, &mut env) {
            return e;
        }
    }
}

fn run(src: &str, env: &mut Scope) -> Option<ExitCode> {
    let mut lexer = Lexer::new(src.chars()).peekable();
    let prog = match RecursiveDescent::<prog::Prog>::parse(&mut lexer) {
        Ok(p) => p,
        Err(e) => {
            println!("syntax error\t{e:?}");
            return Some(ExitCode::from(1));
        }
    };

    match prog.exec(env) {
        Ok(_) => None,
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
