use crate::types::Error;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
use std::fs;

use crate::interpreter::eval_string;

use tikv_jemallocator::Jemalloc;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

mod core_tests;
mod corelib;
mod extra_tests;
mod interpreter;
mod lexer;
mod parser;
mod plotlarslib;
mod polars_tests;
mod polarslib;
mod preprocessor_test;
mod stdlib;
mod types;
mod wisp;

fn is_complete_input(input: &str) -> bool {
    let mut in_string = false;
    let mut in_comment = false;
    let mut escape = false;

    let mut paren_depth: usize = 0;
    let mut bracket_depth: usize = 0;
    let mut brace_depth: usize = 0;

    for c in input.chars() {
        if in_comment {
            if c == '\n' {
                in_comment = false;
            }
            continue;
        }

        if in_string {
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }

        match c {
            '"' => in_string = true,
            ';' => in_comment = true,
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            _ => {}
        }
    }

    !in_string && paren_depth == 0 && bracket_depth == 0 && brace_depth == 0
}

pub fn format_error_message(error: &Error) -> String {
    match error {
        Error::UnclosedString => "Error: unclosed string".to_string(),
        Error::UnclosedListExpr => "Error: unclosed list expression".to_string(),
        Error::FailedToParseNumber { number } => {
            format!("Error: failed to parse number '{number}'")
        }
        Error::UnexpectedClosedParen => "Error: unexpected ')'".to_string(),
        Error::WrongType { expected, got } => format!(
            "Error: expected {0}, got {1}",
            expected,
            got.clone().type_to_str()
        ),
        Error::LookupNotFound { symbol } => format!("Error: symbol '{symbol}' is not defined"),
        Error::WrongNumberOfArguments {
            expected_min,
            expected_max,
            got,
        } => format!("Error: expected {expected_min}-{expected_max} number of argumens, got {got}"),
        Error::ScalarShapeError { shape } => format!(
            "Error: failed to convert to lisp expr, shape is {} by {}",
            shape.0, shape.1
        ),
        Error::ScalarUnsupportedError { name } => {
            format!("Error: unsupported scalar conversion from type '{name}'")
        }
        Error::MissingRequiredKWArg { name } => {
            format!("Error: missing required keyword argument '{name}'")
        }
        Error::Custom(message) => format!("Error: {message}"),
        Error::PolarsError(perror) => format!("Error: {}", perror),
        Error::MixedWithinLine(line) => format!(
            "Error: both tabs and spaces on the same line, on line {}",
            line
        ),
        Error::MixedAcrossFile(line) => format!(
            "Error: inconsistent style compared to previous lines, on line {}",
            line
        ),
        Error::AmbiguousSpaces(line) => format!(
            "Error: spaces found, but not a multiple of 2 or 4, on line {}",
            line
        ),
        Error::EmptyListEval => "Error: attempted to evaluate empty list".to_string(),
    }
}

pub fn format_error_line_info(_error: &Error, state: &interpreter::State) -> Option<String> {
    match state.file.clone() {
        Some(file) => {
            let contents = fs::read_to_string(file).ok()?;
            let lines: Vec<&str> = contents.lines().collect();
            if lines.is_empty() {
                return None;
            }

            let mut result = String::new();
            result.push_str(&format!(" on line {}\n", state.line));

            let start_line = state.line.saturating_sub(2);
            let end_line = (state.line + 2).min(lines.len().saturating_sub(1));

            for idx in start_line..=end_line {
                if idx == state.line {
                    result.push_str(&format!("{} | {} <- error here\n", idx, lines[idx]));
                } else {
                    result.push_str(&format!("{} | {}\n", idx, lines[idx]));
                }
            }

            Some(result)
        }
        None => None,
    }
}

fn handle_error(error: &Error, state: &interpreter::State) {
    let error_message = format_error_message(error);

    match format_error_line_info(error, state) {
        Some(line_info) => println!("{} {}", error_message, line_info),
        None => println!("{}", error_message),
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
                let mut state = interpreter::State {
                    line: 0,
                    file: None,
                };
                match eval_string(input.clone(), &env, &mut state) {
                    Ok(val) => match val.to_string() {
                        Ok(str) => println!("{str}"),
                        Err(err) => handle_error(&err, &state),
                    },
                    Err(err) => handle_error(&err, &state),
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
            let mut state = interpreter::State {
                line: 0,
                file: Some(arg.to_string()),
            };
            let contents = match fs::read_to_string(arg).map_err(|e| Error::Custom(e.to_string())) {
                Ok(text) => text,
                Err(_err) => {
                    handle_error(
                        &Error::Custom(format!("Failed to open {}", arg).to_string()),
                        &state,
                    );
                    continue;
                }
            };
            match eval_string(contents.clone(), &env, &mut state) {
                Ok(val) => match val.to_string() {
                    Ok(str) => println!("{str}"),
                    Err(err) => handle_error(&err, &state),
                },
                Err(err) => handle_error(&err, &state),
            };
        }
        return; // Exit without entering REPL
    }

    repl();
}
