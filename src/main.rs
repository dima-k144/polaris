use std::io::{self, Write};
use std::fs;
use std::cell::RefCell;
use std::rc::Rc;
use crate::types::Env;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use crate::ai::ask_ai;
use crate::interpreter::eval_string;
use crate::ai::suggest_fix;

use tikv_jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

mod lexer;
mod parser;
mod types;
mod interpreter;
mod stdlib;
mod core_tests;
mod polars_tests;
mod extra_tests;
mod preprocessor_test;
mod ai;
mod plotlarslib;
mod wisp;

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
    
    // Initialize the line editor
    let mut rl = match DefaultEditor::new() {
        Ok(editor) => editor,
        Err(err) => {
            eprintln!("Failed to initialize line reader: {err}");
            return;
        }
    };

    // Optional: Load history from a persistent file
    let history_path = dirs::home_dir().map(|p| p.join(".mlisp_history"));
    if let Some(ref path) = history_path {
        let _ = rl.load_history(path);
    }

    loop {
        // Read input with prompt (Up/Down arrows work here)
        let readline = rl.readline("mlisp > ");
        match readline {
            Ok(line) => {
                let mut input = line;
                
                // Handle multiline incomplete parenthesis input
                while !is_complete_input(&input) {
                    match rl.readline("... ") {
                        Ok(more) => {
                            input.push('\n');
                            input.push_str(&more);
                        }
                        Err(_) => break,
                    }
                }

                if input.trim().is_empty() {
                    continue;
                }

                // Add to active session history
                let _ = rl.add_history_entry(input.as_str());

                // Evaluate MLisp expression
                let mut state = interpreter::State { line: 0 };
                match eval_string(input.clone(), &env, &mut state) {
                    Ok(val) => match val.to_string() {
                        Ok(str) => println!("{str}"),
                        Err(err) => handle_error(err, input, &env),
                    },
                    Err(err) => handle_error(err, input, &env),
                };
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
            }
            Err(ReadlineError::Eof) => {
                println!("Exiting...");
                break;
            }
            Err(err) => {
                eprintln!("REPL error: {err}");
                break;
            }
        }
    }

    // Optional: Save session history to disk on exit
    if let Some(ref path) = history_path {
        let _ = rl.save_history(path);
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
