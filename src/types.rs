use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Expr {
    Symbol(String),
    Number(f64),
    FlatList(usize),
    NativeFunction(fn(Vec<Expr>) -> Expr)
}

impl Expr {
    pub fn expect_symbol(self) -> String {
        match self {
            Expr::Symbol(s) => s,
            other => panic!("expected symbol, found {other:?}"),
        }
    }

    pub fn expect_number(self) -> f64 {
        match self {
            Expr::Number(n) => n,
            other => panic!("expected number, found {other:?}")
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Symbol(s) => write!(f, "{s}"),
            Expr::Number(n) => write!(f, "{n}"),
            Expr::FlatList(id) => write!(f, "<FlatList {id}>"),
            Expr::NativeFunction(_) => write!(f, "<NativeFunction>"),
        }
    }
}

pub struct Env {
    pub defines: HashMap<String, Expr>,
    pub outer: Option<Box<Env>>
}
