use std::io::{self, Write};
use std::fs;

mod lexer;
mod parser;
mod types;
mod interpreter;
mod stdlib;
mod core_tests;
mod polars_tests;

fn repl() {
    let env = crate::stdlib::std_env();

    loop {
        let mut input = String::new();
        print!("mlisp > "); io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut input).unwrap();
        let mut tokens = lexer::scan(&input);
        //println!("{:?}", tokens);
        let exprs = parser::parse(&mut tokens);
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
            let mut tokens = lexer::scan(&contents); 
            let exprs = parser::parse(&mut tokens);
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
