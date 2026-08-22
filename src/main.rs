use std::io::{self, Write};
use std::fs;
use std::cell::RefCell;
use std::rc::Rc;
use crate::types::Env;

use crate::ai::ask_ai;
use crate::interpreter::eval_string;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod lexer;
mod parser;
mod types;
mod interpreter;
mod stdlib;
mod core_tests;
mod polars_tests;
mod ai;
mod plotlarslib;

fn is_complete_input(input: &String) -> bool {
    let open_parens_count = input.chars().filter(|&c| c == '(').count();
    let closed_parens_count = input.chars().filter(|&c| c == ')').count();
    let double_quotes_count = input.chars().filter(|&c| c == '"').count();
    open_parens_count == closed_parens_count && double_quotes_count % 2 == 0
}

fn handle_error(error: crate::types::Error, input: String, env: &Rc<RefCell<Env>>) {
    println!("{}", crate::types::format_error(error.clone()));
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let prompt = format!("User has encountered an error:{}\nUser's code:\n{}\n", crate::types::format_error(error), input);
    let answer = runtime.block_on(crate::ai::ask_ai(&prompt, env)).unwrap();
    termimad::print_text(&answer);
}

fn repl() {
    let env = crate::stdlib::std_env();

    loop {
        let mut input = String::new();
        print!("mlisp > "); io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut input).unwrap();
        while !is_complete_input(&input) {
            let mut buf = String::new();
            std::io::stdin().read_line(&mut buf).unwrap();
            input.push_str(buf.as_str());
        }
        let mut state = interpreter::State {
            line: 0,
        };
        match eval_string(input.clone(), &env, &mut state) {
            Ok(val) => {
                match val.to_string() {
                    Ok(str) => println!("{str}"),
                    Err(err) => handle_error(err, input, &env),
                }
            },
            Err(err) => handle_error(err, input, &env),
        };
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let lsp_files: Vec<&String> = args.iter().filter(|arg| arg.ends_with(".lsp")).collect();

    if !lsp_files.is_empty() {
        for arg in lsp_files {
            let env = crate::stdlib::std_env();
            let contents = fs::read_to_string(arg).unwrap();
            let mut state = interpreter::State { line: 0 };
            match eval_string(contents.clone(), &env, &mut state) {
                Ok(val) => {
                    match val.to_string() {
                        Ok(str) => println!("{str}"),
                        Err(err) => handle_error(err, contents, &env),
                    }
                },
                Err(err) => handle_error(err, contents, &env),
            };
        }
        return; // Exit without entering REPL
    }

    repl();
}
