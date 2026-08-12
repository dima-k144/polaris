use std::io::{self, Write};

mod lexer;
mod parser;
mod types;
mod interpreter;
mod stdlib;

fn repl() {
    let mut env = crate::stdlib::std_env();

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
        let mut i = 0;
        let mut result = None;
        while i < exprs.len() {
            result = Some(crate::interpreter::eval_next(&exprs, &mut i, &mut env));
        }
        if let Some(res) = result {
            println!("{res}");
        }
    }
}

fn main() {
    repl();
}
