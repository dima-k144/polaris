use crate::types::*;
use std::process::Command;

pub fn nil() -> Expr { Expr{ value: ExprValue::Nil, line: None } }

pub fn true_() -> Expr { Expr { value: ExprValue::Bool(true), line: None } }

pub fn false_() -> Expr { Expr { value: ExprValue::Bool(false), line: None } }

pub fn new_int(value: i64) -> Expr {
    Expr { value: ExprValue::Int(value), line: None }
}

pub fn new_float(value: f64) -> Expr {
    Expr { value: ExprValue::Float(value), line: None }
}

pub fn new_string(value: String) -> Expr {
    Expr { value: ExprValue::String(value), line: None }
}

pub fn new_list(value: Vec<Expr>) -> Expr {
    Expr { value: ExprValue::List(value), line: None }
}

pub fn new_bool(value: bool) -> Expr {
    Expr { value: ExprValue::Bool(value), line: None }
}

pub fn new_symbol(value: String) -> Expr {
    Expr { value: ExprValue::Symbol(value), line: None }
}

pub fn print(args: Vec<Expr>) -> Result<Expr, Error> {
    for arg in args {
        println!("{}", arg.to_string()?);
    }
    Ok(nil())
}

pub fn list(args: Vec<Expr>) -> Result<Expr, Error> {
    Ok(new_list(args))
}

fn apply_unary_math<F>(args: Vec<Expr>, f_op: F) -> Result<Expr, Error>
where
    F: Fn(f64) -> f64,
{
    let val = args[0].clone().expect_number()?.to_float();
    Ok(new_float(f_op(val)))
}

// --- Math Functions ---

fn has_polars_arg(args: &[Expr]) -> bool {
    args.iter().any(|arg| matches!(arg.value, ExprValue::PolarsExpr(_)))
}

pub fn abs(args: Vec<Expr>) -> Result<Expr, Error> {
    if has_polars_arg(&args) {
        return crate::polarslib::abs_polars(args);
    }
    match args[0].clone().expect_number()? {
        Number::Int(i) => Ok(new_int(i.abs())),
        Number::Float(f) => Ok(new_float(f.abs())),
    }
}

pub fn sqrt(args: Vec<Expr>) -> Result<Expr, Error> {
    if has_polars_arg(&args) {
        return crate::polarslib::sqrt_polars(args);
    }
    apply_unary_math(args, |f| f.sqrt())
}

pub fn exp(args: Vec<Expr>) -> Result<Expr, Error> {
    if has_polars_arg(&args) {
        return crate::polarslib::exp_polars(args);
    }
    apply_unary_math(args, |f| f.exp())
}

pub fn log(args: Vec<Expr>) -> Result<Expr, Error> {
    if has_polars_arg(&args) {
        return crate::polarslib::log_polars(args);
    }
    let val = args[0].clone().expect_number()?.to_float();
    if args.len() > 1 {
        let base = args[1].clone().expect_number()?.to_float();
        Ok(new_float(val.log(base)))
    } else {
        Ok(new_float(val.ln()))
    }
}

pub fn log10(args: Vec<Expr>) -> Result<Expr, Error> {
    if has_polars_arg(&args) {
        return crate::polarslib::log10_polars(args);
    }
    apply_unary_math(args, |f| f.log10())
}

pub fn pow(args: Vec<Expr>) -> Result<Expr, Error> {
    if has_polars_arg(&args) {
        return crate::polarslib::pow_polars(args);
    }
    let base = args[0].clone().expect_number()?;
    let exponent = args[1].clone().expect_number()?;

    match (base, exponent) {
        (Number::Int(b), Number::Int(e)) if e >= 0 && e <= u32::MAX as i64 => {
            Ok(new_int(b.pow(e as u32)))
        }
        (b, e) => Ok(new_float(b.to_float().powf(e.to_float()))),
    }
}

pub fn round(args: Vec<Expr>) -> Result<Expr, Error> {
    if has_polars_arg(&args) {
        return crate::polarslib::round_polars(args);
    }
    let val = args[0].clone().expect_number()?.to_float();
    let decimals = if args.len() > 1 {
        args[1].clone().expect_int()? as i32
    } else {
        0
    };

    let factor = 10f64.powi(decimals);
    let rounded = (val * factor).round() / factor;

    if decimals == 0 {
        Ok(new_int(rounded as i64))
    } else {
        Ok(new_float(rounded))
    }
}

pub fn floor(args: Vec<Expr>) -> Result<Expr, Error> {
    if has_polars_arg(&args) {
        return crate::polarslib::floor_polars(args);
    }
    let val = args[0].clone().expect_number()?;
    match val {
        Number::Int(i) => Ok(new_int(i)),
        Number::Float(f) => Ok(new_int(f.floor() as i64)),
    }
}

pub fn ceil(args: Vec<Expr>) -> Result<Expr, Error> {
    if has_polars_arg(&args) {
        return crate::polarslib::ceil_polars(args);
    }
    let val = args[0].clone().expect_number()?;
    match val {
        Number::Int(i) => Ok(new_int(i)),
        Number::Float(f) => Ok(new_int(f.ceil() as i64)),
    }
}

pub fn sin(args: Vec<Expr>) -> Result<Expr, Error> {
    if has_polars_arg(&args) {
        return crate::polarslib::sin_polars(args);
    }
    apply_unary_math(args, |f| f.sin())
}

pub fn cos(args: Vec<Expr>) -> Result<Expr, Error> {
    if has_polars_arg(&args) {
        return crate::polarslib::cos_polars(args);
    }
    apply_unary_math(args, |f| f.cos())
}

pub fn tan(args: Vec<Expr>) -> Result<Expr, Error> {
    if has_polars_arg(&args) {
        return crate::polarslib::tan_polars(args);
    }
    apply_unary_math(args, |f| f.tan())
}

pub fn shell(args: Vec<Expr>) -> Result<Expr, Error> {
    let command = args[0].clone().expect_string()?;
    let command_args: Vec<String> = args[1..]
        .iter()
        .map(|arg| arg.clone().expect_string())
        .collect::<Result<Vec<String>, Error>>()?;

    let output = Command::new(command)
        .args(command_args)
        .output().map_err(|e| Error::Custom(e.to_string()))?;
    
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(new_string(stdout.to_string()))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(new_string(stderr.to_string()))
    }
}

// (car list) or (first list) -> returns first element
pub fn car(args: Vec<Expr>) -> Result<Expr, Error> {
    let list = args[0].clone().expect_list()?;
    if list.is_empty() {
        Ok(nil())
    } else {
        Ok(list[0].clone())
    }
}

// (cdr list) or (rest list) -> returns elements after the first
pub fn cdr(args: Vec<Expr>) -> Result<Expr, Error> {
    let list = args[0].clone().expect_list()?;
    if list.is_empty() {
        Ok(new_list(vec![]))
    } else {
        Ok(new_list(list[1..].to_vec()))
    }
}

// (cons elem list) -> prepends elem to list
pub fn cons(args: Vec<Expr>) -> Result<Expr, Error> {
    let elem = args[0].clone();
    let mut list = args[1].clone().expect_list()?;
    list.insert(0, elem);
    Ok(new_list(list))
}

// (length list) -> returns number of elements
pub fn length(args: Vec<Expr>) -> Result<Expr, Error> {
    let list = args[0].clone().expect_list()?;
    Ok(new_int(list.len() as i64))
}

// (nth list idx) -> returns item at 0-based index
pub fn nth(args: Vec<Expr>) -> Result<Expr, Error> {
    let list = args[0].clone().expect_list()?;
    let idx = args[1].clone().expect_int()?;

    if idx < 0 || idx as usize >= list.len() {
        return Err(Error::Custom(format!(
            "Index {} out of bounds for list of length {}",
            idx,
            list.len()
        )));
    }

    Ok(list[idx as usize].clone())
}

// (append list1 list2 ...) -> concatenates lists
pub fn append(args: Vec<Expr>) -> Result<Expr, Error> {
    let mut combined = Vec::new();
    for arg in args {
        let items = arg.expect_list()?;
        combined.extend(items);
    }
    Ok(new_list(combined))
}

// (reverse list) -> reverses list
pub fn reverse(args: Vec<Expr>) -> Result<Expr, Error> {
    let mut list = args[0].clone().expect_list()?;
    list.reverse();
    Ok(new_list(list))
}

// (empty? list) -> returns boolean
pub fn is_empty(args: Vec<Expr>) -> Result<Expr, Error> {
    let list = args[0].clone().expect_list()?;
    Ok(new_bool(list.is_empty()))
}

// (to-str val) -> converts number, symbol, bool, nil, or list to String
pub fn to_str(args: Vec<Expr>) -> Result<Expr, Error> {
    let arg = &args[0];
    let s = match &arg.value {
        ExprValue::String(s) => s.clone(),
        ExprValue::Symbol(s) => s.clone(),
        ExprValue::Int(i) => i.to_string(),
        ExprValue::Float(f) => f.to_string(),
        ExprValue::Bool(b) => b.to_string(),
        ExprValue::Nil => "nil".to_string(),
        ExprValue::List(_) => format!("{}", arg),
        _ => return Err(Error::WrongType {
            expected: "convertible value to string".into(),
            got: arg.clone(),
        }),
    };
    Ok(new_string(s))
}

// (to-int val) -> converts float, int, or parseable string/bool to Int
pub fn to_int(args: Vec<Expr>) -> Result<Expr, Error> {
    let arg = &args[0];
    match &arg.value {
        ExprValue::Int(i) => Ok(new_int(*i)),
        ExprValue::Float(f) => Ok(new_int(*f as i64)),
        ExprValue::Bool(b) => Ok(new_int(if *b { 1 } else { 0 })),
        ExprValue::String(s) => {
            s.parse::<i64>()
                .map(new_int)
                .map_err(|_| Error::FailedToParseNumber { number: s.clone() })
        }
        _ => Err(Error::WrongType {
            expected: "convertible value to int".into(),
            got: arg.clone(),
        }),
    }
}

// (to-float val) -> converts int, float, or parseable string to Float
pub fn to_float(args: Vec<Expr>) -> Result<Expr, Error> {
    let arg = &args[0];
    match &arg.value {
        ExprValue::Float(f) => Ok(new_float(*f)),
        ExprValue::Int(i) => Ok(new_float(*i as f64)),
        ExprValue::Bool(b) => Ok(new_float(if *b { 1.0 } else { 0.0 })),
        ExprValue::String(s) => {
            s.parse::<f64>()
                .map(new_float)
                .map_err(|_| Error::FailedToParseNumber { number: s.clone() })
        }
        _ => Err(Error::WrongType {
            expected: "convertible value to float".into(),
            got: arg.clone(),
        }),
    }
}

// (to-symbol val) -> converts string or symbol to Symbol
pub fn to_symbol(args: Vec<Expr>) -> Result<Expr, Error> {
    let arg = &args[0];
    match &arg.value {
        ExprValue::Symbol(s) | ExprValue::String(s) => Ok(new_symbol(s.clone())),
        _ => Err(Error::WrongType {
            expected: "string or symbol".into(),
            got: arg.clone(),
        }),
    }
}

// (to-bool val) -> converts value to Bool (nil & false -> false; 0 -> false; others -> true)
pub fn to_bool(args: Vec<Expr>) -> Result<Expr, Error> {
    let arg = &args[0];
    match &arg.value {
        ExprValue::Bool(b) => Ok(new_bool(*b)),
        ExprValue::Nil => Ok(new_bool(false)),
        ExprValue::Int(i) => Ok(new_bool(*i != 0)),
        ExprValue::Float(f) => Ok(new_bool(*f != 0.0)),
        ExprValue::String(s) => Ok(new_bool(!s.is_empty())),
        ExprValue::List(l) => Ok(new_bool(!l.is_empty())),
        _ => Ok(new_bool(true)),
    }
}

// (to-list val) -> converts a string (into chars) or wraps a single element in a list
pub fn to_list(args: Vec<Expr>) -> Result<Expr, Error> {
    let arg = &args[0];
    match &arg.value {
        ExprValue::List(_) => Ok(arg.clone()),
        ExprValue::String(s) => {
            let chars = s
                .chars()
                .map(|c| new_string(c.to_string()))
                .collect::<Vec<Expr>>();
            Ok(new_list(chars))
        }
        ExprValue::Nil => Ok(new_list(vec![])),
        _ => Ok(new_list(vec![arg.clone()])),
    }
}

// (type-of val) -> returns the type name as a string (e.g., "int", "float", "string", "symbol", "list", "bool", "nil")
pub fn type_of(args: Vec<Expr>) -> Result<Expr, Error> {
    let type_name = args[0].clone().type_to_str();
    Ok(new_string(type_name.to_string()))
}

// (str-concat "a" "b" "c") -> "abc"
pub fn str_concat(args: Vec<Expr>) -> Result<Expr, Error> {
    let mut out = String::new();
    for arg in args {
        match arg.value {
            ExprValue::String(s) | ExprValue::Symbol(s) => out.push_str(&s),
            ExprValue::Int(i) => out.push_str(&i.to_string()),
            ExprValue::Float(f) => out.push_str(&f.to_string()),
            _ => out.push_str(&to_str(vec![arg])?.expect_string()?),
        }
    }
    Ok(new_string(out))
}

// Helper to execute any callable Expr (NativeFunction or UserFunction)
pub fn invoke_callable(func: &Expr, args: Vec<Expr>) -> Result<Expr, Error> {
    match &func.value {
        ExprValue::NativeFunction(data) => {
            crate::interpreter::expect_args_count(args.len(), data.min_args as usize, data.max_args as usize)?;
            (data.body)(args)
        }
        ExprValue::UserFunction(data) => {
            crate::interpreter::expect_args_count(args.len(), data.params.len(), data.params.len())?;
            
            // Re-use std_env as base so standard functions/macros are visible inside closures
            let env = crate::stdlib::std_env();
            let child_env = std::rc::Rc::new(std::cell::RefCell::new(crate::types::Env::with_parent(env)));

            for (param, arg) in data.params.iter().zip(args) {
                crate::interpreter::define(param, arg, &child_env);
            }

            let mut state = crate::interpreter::State { line: 0, file: None };
            let mut result = nil();
            for expr in &data.body {
                result = crate::interpreter::eval(expr.clone(), &child_env, &mut state)?;
            }
            Ok(result)
        }
        _ => Err(Error::WrongType {
            expected: "callable function".into(),
            got: func.clone(),
        }),
    }
}

// (range end)
// (range start end [step])
pub fn range(args: Vec<Expr>) -> Result<Expr, Error> {
    let (start, end, step) = match args.len() {
        1 => (0, args[0].clone().expect_int()?, 1),
        2 => (args[0].clone().expect_int()?, args[1].clone().expect_int()?, 1),
        3 => {
            let step = args[2].clone().expect_int()?;
            if step == 0 {
                return Err(Error::Custom("range step cannot be 0".into()));
            }
            (args[0].clone().expect_int()?, args[1].clone().expect_int()?, step)
        }
        _ => {
            return Err(Error::WrongNumberOfArguments {
                expected_min: 1,
                expected_max: 3,
                got: args.len() as u8,
            });
        }
    };

    let mut values = Vec::new();
    let mut curr = start;

    if step > 0 {
        while curr < end {
            values.push(new_int(curr));
            curr += step;
        }
    } else {
        while curr > end {
            values.push(new_int(curr));
            curr += step;
        }
    }

    Ok(new_list(values))
}

// (apply func [arg1 arg2 ...] list)
pub fn apply(args: Vec<Expr>) -> Result<Expr, Error> {
    if args.len() < 2 {
        return Err(Error::WrongNumberOfArguments {
            expected_min: 2,
            expected_max: u8::MAX,
            got: args.len() as u8,
        });
    }

    let func = &args[0];
    let mut call_args = Vec::new();

    for arg in &args[1..args.len() - 1] {
        call_args.push(arg.clone());
    }

    let final_list = args.last().unwrap().clone().expect_list()?;
    call_args.extend(final_list);

    invoke_callable(func, call_args)
}

// (map func list)
pub fn map(args: Vec<Expr>) -> Result<Expr, Error> {
    let func = &args[0];
    let items = args[1].clone().expect_list()?;

    let mut out = Vec::with_capacity(items.len());
    for item in items {
        out.push(invoke_callable(func, vec![item])?);
    }
    Ok(new_list(out))
}

// (filter predicate list) - with Polars DataFrame fallback
pub fn filter_core(args: Vec<Expr>) -> Result<Expr, Error> {
    if matches!(args[0].value, ExprValue::LazyFrame(_)) {
        return crate::polarslib::filter(args);
    }

    if args.len() != 2 {
        return Err(Error::WrongNumberOfArguments {
            expected_min: 2,
            expected_max: 2,
            got: args.len() as u8,
        });
    }

    let func = &args[0];
    let items = args[1].clone().expect_list()?;

    let mut out = Vec::new();
    for item in items {
        let keep = invoke_callable(func, vec![item.clone()])?.expect_bool()?;
        if keep {
            out.push(item);
        }
    }
    Ok(new_list(out))
}

// (reduce func list) OR (reduce func init list)
pub fn reduce(args: Vec<Expr>) -> Result<Expr, Error> {
    let func = &args[0];

    if args.len() == 2 {
        let items = args[1].clone().expect_list()?;
        if items.is_empty() {
            return Err(Error::Custom("reduce over empty list without initial value".into()));
        }
        let mut acc = items[0].clone();
        for item in &items[1..] {
            acc = invoke_callable(func, vec![acc, item.clone()])?;
        }
        Ok(acc)
    } else if args.len() == 3 {
        let mut acc = args[1].clone();
        let items = args[2].clone().expect_list()?;
        for item in items {
            acc = invoke_callable(func, vec![acc, item])?;
        }
        Ok(acc)
    } else {
        Err(Error::WrongNumberOfArguments {
            expected_min: 2,
            expected_max: 3,
            got: args.len() as u8,
        })
    }
}

// (str-len "hello") -> 5
pub fn str_len(args: Vec<Expr>) -> Result<Expr, Error> {
    let s = args[0].clone().expect_string()?;
    Ok(new_int(s.chars().count() as i64))
}

// (substring "hello" 1 3) -> "el" (start index, length)
// (substring "hello" 1)   -> "ello" (to the end)
pub fn substring(args: Vec<Expr>) -> Result<Expr, Error> {
    let s = args[0].clone().expect_string()?;
    let start = args[1].clone().expect_int()?;

    if start < 0 {
        return Err(Error::Custom(format!("substring start index cannot be negative: {}", start)));
    }

    let char_count = s.chars().count();
    let start_idx = (start as usize).min(char_count);

    let sub: String = if args.len() > 2 {
        let len = args[2].clone().expect_int()?;
        if len < 0 {
            return Err(Error::Custom(format!("substring length cannot be negative: {}", len)));
        }
        s.chars().skip(start_idx).take(len as usize).collect()
    } else {
        s.chars().skip(start_idx).collect()
    };

    Ok(new_string(sub))
}

// (split "a,b,c" ",") -> '("a" "b" "c")
pub fn split(args: Vec<Expr>) -> Result<Expr, Error> {
    let s = args[0].clone().expect_string()?;
    let delim = args[1].clone().expect_string()?;

    let parts: Vec<Expr> = s
        .split(&delim)
        .map(|part| new_string(part.to_string()))
        .collect();

    Ok(new_list(parts))
}

// (str-join '("a" "b" "c") ", ") -> "a, b, c"
// (str-join '("a" "b" "c"))      -> "abc" (default empty delimiter)
pub fn str_join(args: Vec<Expr>) -> Result<Expr, Error> {
    let items = args[0].clone().expect_list()?;
    let delim = if args.len() > 1 {
        args[1].clone().expect_string()?
    } else {
        String::new()
    };

    let mut string_parts = Vec::with_capacity(items.len());
    for item in items {
        string_parts.push(to_str(vec![item])?.expect_string()?);
    }

    Ok(new_string(string_parts.join(&delim)))
}
