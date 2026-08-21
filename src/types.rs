use std::collections::HashMap;
use std::fmt;
use std::cell::RefCell;
use std::rc::Rc;
use polars::prelude::*;

#[derive(Clone)]
pub enum Expr {
    Symbol(String),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<Expr>),
    NativeFunction{ body: fn(Vec<Expr>) -> Result<Expr, Error>, min_args: u8, max_args: u8},
    UserFunction{ body: Vec<Expr>, params: Vec<String>, name: Option<String> },
    LazyFrame(LazyFrame),
    PolarsExpr(polars::prelude::Expr),
    SourceInfo(Box<Expr>, usize)
}

pub enum Number {
    Int(i64),
    Float(f64)
}

impl Expr {
    pub fn expect_symbol(self) -> Result<String, Error> {
        match self {
            Expr::Symbol(s) => Ok(s),
            other => Err(Error::WrongType { expected: String::from("symbol"), got: other }),
        }
    }

    pub fn expect_number(self) -> Result<Number, Error> {
        match self {
            Expr::Int(n) => Ok(Number::Int(n)),
            Expr::Float(n) => Ok(Number::Float(n)),
            other => Err(Error::WrongType { expected: String::from("number"), got: other }),
        }
    }

    pub fn expect_int(self) -> Result<i64, Error> {
        match self {
            Expr::Int(i) => Ok(i),
            other => Err(Error::WrongType { expected: String::from("int"), got: other }),
        }
    }

    pub fn expect_float(self) -> Result<f64, Error> {
        match self {
            Expr::Float(f) => Ok(f),
            other => Err(Error::WrongType { expected: String::from("float"), got: other }),
        }
    }


    pub fn expect_string(self) -> Result<String, Error> {
        match self {
            Expr::String(s) => Ok(s),
            other => Err(Error::WrongType { expected: String::from("string"), got: other }),
        }
    }
    
    pub fn expect_list(self) -> Result<Vec<Expr>, Error> {
        match self {
            Expr::List(l) => Ok(l),
            other => Err(Error::WrongType { expected: String::from("list"), got: other }),
        }
    }
    
    pub fn expect_lazy_frame(self) -> Result<LazyFrame, Error> {
        match self {
            Expr::LazyFrame(lf) => Ok(lf),
            other => Err(Error::WrongType { expected: String::from("lazy frame"), got: other }),
        }
    } 

    pub fn expect_polars_expr(self) -> Result<polars::prelude::Expr, Error> {
        match self {
            Expr::PolarsExpr(pe) => Ok(pe),
            Expr::Int(i) => Ok(polars::prelude::lit(i)),
            Expr::Float(f) => Ok(polars::prelude::lit(f)),
            Expr::String(str) => Ok(polars::prelude::lit(str)),
            other => Err(Error::WrongType { expected: String::from("polars expr"), got: other }),
        }
    }

    pub fn type_to_str(self) -> &'static str {
        match self {
            Expr::Symbol(_) => "symbol",
            Expr::Int(_) => "int",
            Expr::Float(_) => "float",
            Expr::String(_) => "string",
            Expr::List(_) => "list",
            Expr::NativeFunction { body, min_args, max_args } => "native function",
            Expr::UserFunction { body, params, name } => "user function",
            Expr::LazyFrame(_) => "lazy frame",
            Expr::PolarsExpr(_) => "polars expr",
            Expr::SourceInfo(_, _) => "source info",
        }
    }

    pub fn to_string(self) -> Result<String, Error> {
        match self {
            Expr::LazyFrame(lf) => {
                let df = lf.clone().collect().map_err(Error::PolarsError)?;
                Ok(format!("{}", df))
            }
            // For others, fall back to standard Display or format manually
            other => Ok(format!("{}", other)),
        }
    }
}

impl Number {
    pub fn to_int(self) -> i64 {
        match self {
            Number::Int(i) => i,
            Number::Float(f) => f as i64,
        }
    }

    pub fn to_float(self) -> f64 {
        match self {
            Number::Int(i) => i as f64,
            Number::Float(f) => f,
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Symbol(s) => write!(f, "{s}"),
            Expr::Int(n) => write!(f, "{n}"),
            Expr::Float(n) => write!(f, "{n}"),
            Expr::String(s) => Ok(termimad::print_text(s)),
            Expr::List(items) => {
                write!(f, "(")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, ")")
            },
            Expr::NativeFunction{ body, min_args, max_args } => write!(f, "<NativeFunction>"),
            Expr::UserFunction { body, params, name } => {
                match name {
                    Some(name) => write!(f, "<{name}>"),
                    None => write!(f, "<lambda>"),
                }
            }
            Expr::LazyFrame(lf) => match lf.clone().collect() {
                Ok(df) => write!(f, "{df}"),
                Err(err) => write!(f, "<Failed to collect LazyFrame: {err}>"),
            },
            Expr::PolarsExpr(pe) => write!(f, "{}", pe),
            Expr::SourceInfo(expr, line) => write!(f, "<SourceInfo>")
        }
    }
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Symbol(s) => f.debug_tuple("Symbol").field(s).finish(),
            Expr::Int(n) => f.debug_tuple("Number").field(n).finish(),
            Expr::Float(n) => f.debug_tuple("Number").field(n).finish(),
            Expr::String(s) => f.debug_tuple("String").field(s).finish(),
            Expr::List(items) => f.debug_tuple("List").field(items).finish(),
            Expr::NativeFunction{ body, min_args, max_args } => { f.write_str("NativeFunction") },
            Expr::UserFunction { body, params, name } => { f.write_str("UserFunction") },
            Expr::LazyFrame(_) => { f.write_str("LazyFrame") },
            Expr::PolarsExpr(_) => { f.write_str("PolarsExpr") },
            Expr::SourceInfo(expr, line) => { f.write_str("SourceInfo") }
        }
    }
}

pub struct Env {
    pub defines: HashMap<String, Expr>,
    pub outer: Option<Rc<RefCell<Env>>>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            defines: HashMap::new(),
            outer: None,
        }
    }

    pub fn with_parent(outer: Rc<RefCell<Env>>) -> Self {
        Self {
            defines: HashMap::new(),
            outer: Some(outer),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Error {
    UnclosedString,
    UnclosedListExpr,
    FailedToParseNumber {
        number: String
    },
    WrongType {
        expected: String,
        got: Expr
    },
    LookupNotFound {
        symbol: String
    },
    WrongNumberOfArguments {
        expected_min: u8,
        expected_max: u8,
        got: u8
    },
    ScalarShapeError{
        shape: (usize, usize)
    },
    ScalarUnsupportedError{
        name: String
    },
    PolarsError(polars::prelude::PolarsError)
}

pub fn format_error(error: Error) -> String {
    match error {
        Error::UnclosedString => format!("Error: unclosed string"),
        Error::UnclosedListExpr => format!("Error: unclosed list expression"),
        Error::FailedToParseNumber { number } => format!("Error: failed to parse number '{number}'"),
        Error::WrongType { expected, got } => format!("Error: expected {0}, got {1}", expected, got.type_to_str()),
        Error::LookupNotFound { symbol } => format!("Error: symbol '{symbol}' is not defined"),
        Error::WrongNumberOfArguments { 
            expected_min, expected_max, got 
        } => format!("Error: expected {expected_min}-{expected_max} number of argumens, got {got}"),
        Error::ScalarShapeError { shape } => format!("Error: failed to convert to lisp expr, shape is {} by {}", shape.0, shape.1),
        Error::ScalarUnsupportedError { name } => format!("Error: unsupported scalar conversion from type '{name}'"),
        Error::PolarsError(perror) => format!("Error: {}", perror)
    }
}
