use crate::types::*;
use std::process::Command;

pub fn nil() -> Expr { return Expr{ value: ExprValue::Nil, line: None } }

pub fn true_() -> Expr { return Expr { value: ExprValue::Bool(true), line: None } }

pub fn false_() -> Expr { return Expr { value: ExprValue::Bool(false), line: None } }

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

pub fn abs(args: Vec<Expr>) -> Result<Expr, Error> {
    match args[0].clone().expect_number()? {
        Number::Int(i) => Ok(new_int(i.abs())),
        Number::Float(f) => Ok(new_float(f.abs())),
    }
}

pub fn sqrt(args: Vec<Expr>) -> Result<Expr, Error> {
    apply_unary_math(args, |f| f.sqrt())
}

pub fn exp(args: Vec<Expr>) -> Result<Expr, Error> {
    apply_unary_math(args, |f| f.exp())
}

pub fn log(args: Vec<Expr>) -> Result<Expr, Error> {
    let val = args[0].clone().expect_number()?.to_float();
    if args.len() > 1 {
        let base = args[1].clone().expect_number()?.to_float();
        Ok(new_float(val.log(base)))
    } else {
        Ok(new_float(val.ln()))
    }
}

pub fn log10(args: Vec<Expr>) -> Result<Expr, Error> {
    apply_unary_math(args, |f| f.log10())
}

pub fn pow(args: Vec<Expr>) -> Result<Expr, Error> {
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
    let val = args[0].clone().expect_number()?;
    match val {
        Number::Int(i) => Ok(new_int(i)),
        Number::Float(f) => Ok(new_int(f.floor() as i64)),
    }
}

pub fn ceil(args: Vec<Expr>) -> Result<Expr, Error> {
    let val = args[0].clone().expect_number()?;
    match val {
        Number::Int(i) => Ok(new_int(i)),
        Number::Float(f) => Ok(new_int(f.ceil() as i64)),
    }
}

pub fn sin(args: Vec<Expr>) -> Result<Expr, Error> {
    apply_unary_math(args, |f| f.sin())
}

pub fn cos(args: Vec<Expr>) -> Result<Expr, Error> {
    apply_unary_math(args, |f| f.cos())
}

pub fn tan(args: Vec<Expr>) -> Result<Expr, Error> {
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
        .output().unwrap();
    
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
