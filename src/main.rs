use std::io::{self, Write};

mod lexer;
mod parser;
mod types;

fn repl() {
    loop {
        let mut input = String::new();
        print!("mlisp > "); io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut input).unwrap();
        let mut tokens = lexer::scan(&input);
        println!("{:?}", tokens);
        let exprs = parser::parse(&mut tokens);
        for (n, expr) in exprs.iter().enumerate() {
            println!("{} {:?}", n, expr);
        }
    }
}

fn main() {
    repl();
}
