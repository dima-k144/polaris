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
    Bool(bool),
    Nil,
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

    pub fn expect_bool(self) -> Result<bool, Error> {
        let Expr { value, line } = self;
        match value {
            ExprValue::Bool(b) => Ok(b),
            other => Err(Error::WrongType {
                expected: String::from("bool"),
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
            ExprValue::Bool(_) => "bool",
            ExprValue::Nil => "nil",
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
            ExprValue::Bool(value) => {
                match value {
                    true => write!(f, "true"),
                    false => write!(f, "false"),
                }
            },
            ExprValue::Nil => {
                write!(f, "nil")
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
            ExprValue::Bool(b) => f.debug_tuple("Bool").field(b).finish(),
            ExprValue::Nil => f.write_str("Nil"),
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

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialEq for ExprValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ExprValue::Symbol(a), ExprValue::Symbol(b)) => a == b,
            (ExprValue::Int(a), ExprValue::Int(b)) => a == b,
            (ExprValue::Float(a), ExprValue::Float(b)) => {
                // Standard IEEE 754 float equality:
                a == b
                // Or if you want NaN == NaN: (a == b) || (a.is_nan() && b.is_nan())
            }
            (ExprValue::String(a), ExprValue::String(b)) => a == b,
            (ExprValue::List(a), ExprValue::List(b)) => a == b, // Requires Expr: PartialEq
            (ExprValue::Bool(a), ExprValue::Bool(b)) => a == b,
            (ExprValue::Nil, ExprValue::Nil) => true,

            // Function pointers: compare memory addresses
            (
                ExprValue::NativeFunction { body: b1, min_args: min1, max_args: max1 },
                ExprValue::NativeFunction { body: b2, min_args: min2, max_args: max2 },
            ) => {
                (*b1 as *const ()) == (*b2 as *const ()) 
                    && min1 == min2 
                    && max1 == max2
            }

            (
                ExprValue::UserFunction { body: b1, params: p1, name: n1 },
                ExprValue::UserFunction { body: b2, params: p2, name: n2 },
            ) => b1 == b2 && p1 == p2 && n1 == n2,

            // Polars expression comparison (implements PartialEq)
            (ExprValue::PolarsExpr(a), ExprValue::PolarsExpr(b)) => a == b,

            // LazyFrame does NOT implement PartialEq in Polars.
            // Option 1: Compare their logical plans via debug representation
            (ExprValue::LazyFrame(a), ExprValue::LazyFrame(b)) => {
                format!("{:?}", a.logical_plan) == format!("{:?}", b.logical_plan)
            }
            // Option 2 (Alternative): Treat different LazyFrames as always unequal
            // (ExprValue::LazyFrame(_), ExprValue::LazyFrame(_)) => false,

            _ => false,
        }
    }
}
