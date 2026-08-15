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
        let mut result = None;
        for expr in exprs {
            result = Some(crate::interpreter::eval(expr, &mut env));
        }
        if let Some(res) = result {
            println!("{res}");
        }
    }
}

fn main() {
    repl();
}
