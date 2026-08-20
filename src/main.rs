use std::io::{self, Write};
use std::fs;

mod lexer;
mod parser;
mod types;
mod interpreter;
mod stdlib;
mod core_tests;
mod polars_tests;

fn is_complete_input(input: &String) -> bool {
    let open_parens_count = input.chars().filter(|&c| c == '(').count();
    let closed_parens_count = input.chars().filter(|&c| c == ')').count();
    let double_quotes_count = input.chars().filter(|&c| c == '"').count();
    open_parens_count == closed_parens_count && double_quotes_count % 2 == 0
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
        let mut tokens = match lexer::scan(&input) {
            Ok(tokens) => tokens,
            Err(err) => { println!("{}", crate::types::format_error(err)); continue; },
        };
        //println!("{:?}", tokens);
        let exprs = match parser::parse(&mut tokens) {
            Ok(exprs) => exprs,
            Err(err) => { println!("{}", crate::types::format_error(err)); continue; },
        };
        //for (n, expr) in exprs.iter().enumerate() {
        //    println!("{} {:?}", n, expr);
        //}
        let mut result = None;
        let mut state = interpreter::State {
            line: 0,
        };
        for expr in exprs {
            result = Some(crate::interpreter::eval(expr, &env, &mut state));
        }
        if let Some(res) = result {
            match res {
                Ok(val) => {
                    match val.to_string() {
                        Ok(str) => println!("{str}"),
                        Err(err) => println!("{}", crate::types::format_error(err)),
                    }
                },
                Err(err) => println!("{}", crate::types::format_error(err)),
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    for arg in args {
        let env = crate::stdlib::std_env();

        if arg.ends_with(".lsp") {
            let contents = fs::read_to_string(arg).unwrap();
            let mut tokens = match lexer::scan(&contents) {
                Ok(tokens) => tokens,
                Err(err) => { println!("{}", crate::types::format_error(err)); continue; },
            };
            let exprs = match parser::parse(&mut tokens) {
                Ok(exprs) => exprs,
                Err(err) => { println!("{}", crate::types::format_error(err)); continue; },
            };
            let mut result = None;
            let mut state = interpreter::State {
                line: 0,
            };
            for expr in exprs {
                result = Some(crate::interpreter::eval(expr, &env, &mut state));
            }
            if let Some(res) = result {
                match res {
                    Ok(val) => {
                        match val.to_string() {
                            Ok(str) => println!("{str}"),
                            Err(err) => println!("{}", crate::types::format_error(err)),
                        }
                    },
                    Err(err) => println!("{}", crate::types::format_error(err)),
                }
            }
        }
    }

    repl();
}
