use std::io::{self, Write};
use std::fs;
use std::cell::RefCell;
use std::rc::Rc;
use crate::types::Env;

use crate::ai::ask_ai;
use crate::interpreter::eval_string;
use crate::ai::suggest_fix;

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
    let err_str = crate::types::format_error(error);
    println!("{err_str}");

    // If cancelled by user, return to the prompt without asking for AI fix
    if err_str.contains("cancelled by user") {
        return;
    }

    let runtime = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("Runtime error: {e}");
            return;
        }
    };

    match runtime.block_on(suggest_fix(&err_str, &input, env)) {
        Ok(fix) => {
            termimad::print_text(&format!("**AI Diagnosis:** {}", fix.explanation));
            println!("\nSuggested Fix:\n  {}", fix.fixed_code);
            print!("\nApply suggested fix? (y/N): ");
            io::stdout().flush().unwrap();

            let mut choice = String::new();
            if io::stdin().read_line(&mut choice).is_ok() {
                let trimmed = choice.trim().to_lowercase();
                if trimmed == "y" || trimmed == "yes" {
                    println!("\nExecuting fix...");
                    let mut state = interpreter::State { line: 0 };
                    match eval_string(fix.fixed_code.clone(), env, &mut state) {
                        Ok(val) => match val.to_string() {
                            Ok(str) => println!("{str}"),
                            Err(e) => handle_error(e, fix.fixed_code, env),
                        },
                        Err(e) => handle_error(e, fix.fixed_code, env),
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to generate AI auto-fix: {e}");
        }
    }
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
