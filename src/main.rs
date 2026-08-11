use std::io::{self, Write};

mod lexer;

fn repl() {
    let mut input = String::new();
    loop {
        print!("mlisp > "); io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut input).unwrap();
        let tokens = lexer::scan(&input);
        println!("{:?}", tokens)
    }
}

fn main() {
    repl();
}
