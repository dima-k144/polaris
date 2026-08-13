use std::collections::HashMap;
use std::fmt;
use polars::prelude::*;

#[derive(Clone)]
pub enum Expr {
    Symbol(String),
    Number(f64),
    String(String),
    FlatList(usize),
    NativeFunction(fn(Vec<Expr>) -> Expr),
    LazyFrame(LazyFrame)
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

    pub fn expect_string(self) -> String {
        match self {
            Expr::String(s) => s,
            other => panic!("expected string, found {other:?}")
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Symbol(s) => write!(f, "{s}"),
            Expr::Number(n) => write!(f, "{n}"),
            Expr::String(s) => write!(f, "{s}"),
            Expr::FlatList(id) => write!(f, "<FlatList {id}>"),
            Expr::NativeFunction(_) => write!(f, "<NativeFunction>"),
            Expr::LazyFrame(lf) => write!(f, "{}", lf.clone().collect().unwrap())
        }
    }
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Symbol(s) => f.debug_tuple("Symbol").field(s).finish(),
            Expr::Number(n) => f.debug_tuple("Number").field(n).finish(),
            Expr::String(s) => f.debug_tuple("String").field(s).finish(),
            Expr::FlatList(n) => f.debug_tuple("FlatList").field(n).finish(),
            Expr::NativeFunction(_) => {
                f.write_str("NativeFunction")
            }
            Expr::LazyFrame(_) => {
                f.write_str("LazyFrame")
            }
        }
    }
}

pub struct Env {
    pub defines: HashMap<String, Expr>,
    pub outer: Option<Box<Env>>
}
