use std::collections::HashMap;
use std::fmt;
use std::cell::RefCell;
use std::rc::Rc;
use polars::prelude::*;

#[derive(Clone)]
pub struct Expr {
    pub value: ExprValue,
    pub line: Option<usize>
}

#[derive(Clone)]
pub enum ExprValue {
    Symbol(String),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<Expr>),
    NativeFunction{ body: fn(Vec<Expr>) -> Result<Expr, Error>, min_args: u8, max_args: u8},
    UserFunction{ body: Vec<Expr>, params: Vec<String>, name: Option<String> },
    LazyFrame(LazyFrame),
    PolarsExpr(polars::prelude::Expr),
}

pub enum Number {
    Int(i64),
    Float(f64)
}

impl Expr {
     pub fn expect_symbol(self) -> Result<String, Error> {
        let Expr { value, line } = self;
        match value {
            ExprValue::Symbol(s) => Ok(s),
            other => Err(Error::WrongType {
                expected: String::from("symbol"),
                got: Expr { value: other, line },
            }),
        }
    }

    pub fn expect_number(self) -> Result<Number, Error> {
        let Expr { value, line } = self;
        match value {
            ExprValue::Int(n) => Ok(Number::Int(n)),
            ExprValue::Float(n) => Ok(Number::Float(n)),
            other => Err(Error::WrongType {
                expected: String::from("number"),
                got: Expr { value: other, line },
            }),
        }
    }

    pub fn expect_int(self) -> Result<i64, Error> {
        let Expr { value, line } = self;
        match value {
            ExprValue::Int(i) => Ok(i),
            other => Err(Error::WrongType {
                expected: String::from("int"),
                got: Expr { value: other, line },
            }),
        }
    }

    pub fn expect_float(self) -> Result<f64, Error> {
        let Expr { value, line } = self;
        match value {
            ExprValue::Float(f) => Ok(f),
            other => Err(Error::WrongType {
                expected: String::from("float"),
                got: Expr { value: other, line },
            }),
        }
    }

    pub fn expect_string(self) -> Result<String, Error> {
        let Expr { value, line } = self;
        match value {
            ExprValue::String(s) => Ok(s),
            other => Err(Error::WrongType {
                expected: String::from("string"),
                got: Expr { value: other, line },
            }),
        }
    }

    pub fn expect_list(self) -> Result<Vec<Expr>, Error> {
        let Expr { value, line } = self;
        match value {
            ExprValue::List(l) => Ok(l),
            other => Err(Error::WrongType {
                expected: String::from("list"),
                got: Expr { value: other, line },
            }),
        }
    }

    pub fn expect_lazy_frame(self) -> Result<LazyFrame, Error> {
        let Expr { value, line } = self;
        match value {
            ExprValue::LazyFrame(lf) => Ok(lf),
            other => Err(Error::WrongType {
                expected: String::from("lazy frame"),
                got: Expr { value: other, line },
            }),
        }
    }

    pub fn expect_polars_expr(self) -> Result<polars::prelude::Expr, Error> {
        let Expr { value, line } = self;
        match value {
            ExprValue::PolarsExpr(pe) => Ok(pe),
            ExprValue::Int(i) => Ok(polars::prelude::lit(i)),
            ExprValue::Float(f) => Ok(polars::prelude::lit(f)),
            ExprValue::String(str) => Ok(polars::prelude::lit(str)),
            other => Err(Error::WrongType {
                expected: String::from("polars expr"),
                got: Expr { value: other, line },
            }),
        }
    }

    pub fn type_to_str(self) -> &'static str {
        match self.value {
            ExprValue::Symbol(_) => "symbol",
            ExprValue::Int(_) => "int",
            ExprValue::Float(_) => "float",
            ExprValue::String(_) => "string",
            ExprValue::List(_) => "list",
            ExprValue::NativeFunction { body, min_args, max_args } => "native function",
            ExprValue::UserFunction { body, params, name } => "user function",
            ExprValue::LazyFrame(_) => "lazy frame",
            ExprValue::PolarsExpr(_) => "polars expr",
        }
    }

    pub fn to_string(self) -> Result<String, Error> {
        match self.value {
            ExprValue::LazyFrame(lf) => {
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
        self.value.fmt(f)
    }
}

impl fmt::Display for ExprValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExprValue::Symbol(s) => write!(f, "{s}"),
            ExprValue::Int(n) => write!(f, "{n}"),
            ExprValue::Float(n) => write!(f, "{n}"),
            ExprValue::String(s) => write!(f, "{s}"),
            ExprValue::List(items) => {
                write!(f, "(")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", item.value)?;
                }
                write!(f, ")")
            },
            ExprValue::NativeFunction{ body, min_args, max_args } => write!(f, "<NativeFunction>"),
            ExprValue::UserFunction { body, params, name } => {
                match name {
                    Some(name) => write!(f, "<{name}>"),
                    None => write!(f, "<lambda>"),
                }
            }
            ExprValue::LazyFrame(lf) => write!(f, "<LazyFrame>"),
            ExprValue::PolarsExpr(pe) => write!(f, "{}", pe),
        }
    }
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Expr")
            .field("value", &self.value)
            .field("line", &self.line)
            .finish()
    }
}

impl fmt::Debug for ExprValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExprValue::Symbol(s) => f.debug_tuple("Symbol").field(s).finish(),
            ExprValue::Int(n) => f.debug_tuple("Number").field(n).finish(),
            ExprValue::Float(n) => f.debug_tuple("Number").field(n).finish(),
            ExprValue::String(s) => f.debug_tuple("String").field(s).finish(),
            ExprValue::List(items) => f.debug_tuple("List").field(items).finish(),
            ExprValue::NativeFunction{ body, min_args, max_args } => { f.write_str("NativeFunction") },
            ExprValue::UserFunction { body, params, name } => { f.write_str("UserFunction") },
            ExprValue::LazyFrame(_) => { f.write_str("LazyFrame") },
            ExprValue::PolarsExpr(_) => { f.write_str("PolarsExpr") },
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
    MissingRequiredKWArg{
        name: String
    },
    Custom(String),
    PolarsError(polars::prelude::PolarsError)
}
