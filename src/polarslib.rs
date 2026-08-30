use std::{collections::HashMap, f32::consts::E};
use std::fs::File;
use polars::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use crate::types::{Env, Error, Expr, ExprValue::{self, PolarsExpr}};
use crate::plotlarslib;
use crate::interpreter::{eval_string, State};
use polars::prelude::RollingOptionsFixedWindow;
use crate::corelib::*;
use polars::prelude::QuantileMethod;

pub fn new_polars_expr(value: polars::prelude::Expr) -> Expr {
    Expr { value: ExprValue::PolarsExpr(value), line: None }
}

pub fn new_lazy_frame(value: LazyFrame) -> Expr {
    Expr { value: ExprValue::LazyFrame(value), line: None }
}

// --- Addition / Column Addition ---
// Examples in Lisp:
//   (+ 1 2 3)
//   (+ $age 5)
//   (+ $base_salary $bonus)
pub fn plus(args: Vec<Expr>) -> Result<Expr, Error> {
    let has_polars_expr_arg = args.iter().any(|arg| matches!(arg.value, ExprValue::PolarsExpr(_)));
    let has_float_arg = args.iter().any(|arg| matches!(arg.value, ExprValue::Float(_)));

    if has_polars_expr_arg {
        let mut expr = args[0].clone().expect_polars_expr()?;
        for arg in args.iter().skip(1) {
            expr = expr + arg.clone().expect_polars_expr()?;
        }
        Ok(new_polars_expr(expr))
    } else if has_float_arg {
        let mut sum = 0.0;
        for arg in args {
            sum += arg.expect_number()?.to_float();
        }
        Ok(new_float(sum))
    } else {
        let mut sum = 0;
        for arg in args {
            sum += arg.expect_number()?.to_int();
        }
        Ok(new_int(sum))
    }
}

// --- Subtraction / Column Subtraction ---
// Examples in Lisp:
//   (- 20 5 3)
//   (- $age 2)
//   (- $gross_income $tax)
pub fn minus(args: Vec<Expr>) -> Result<Expr, Error> {
    let has_polars_expr_arg = args.iter().any(|arg| matches!(arg.value, ExprValue::PolarsExpr(_)));
    let has_float_arg = args.iter().any(|arg| matches!(arg.value, ExprValue::Float(_)));

    if has_polars_expr_arg {
        let mut expr = args[0].clone().expect_polars_expr()?;
        for arg in args.iter().skip(1) {
            expr = expr - arg.clone().expect_polars_expr()?;
        }
        Ok(new_polars_expr(expr))
    } else if has_float_arg {
        let mut value = args[0].clone().expect_number()?.to_float();
        for arg in args.iter().skip(1) {
            value -= arg.clone().expect_number()?.to_float();
        }
        Ok(new_float(value))
    } else {
        let mut value = args[0].clone().expect_number()?.to_int();
        for arg in args.iter().skip(1) {
            value -= arg.clone().expect_number()?.to_int();
        }
        Ok(new_int(value))
    }
}

// --- Multiplication / Column Multiplication ---
// Examples in Lisp:
//   (* 2 3 4)
//   (* $price $quantity)
//   (* $age 2)
pub fn mult(args: Vec<Expr>) -> Result<Expr, Error> {
    let has_polars_expr_arg = args.iter().any(|arg| matches!(arg.value, ExprValue::PolarsExpr(_)));
    let has_float_arg = args.iter().any(|arg| matches!(arg.value, ExprValue::Float(_)));

    if has_polars_expr_arg {
        let mut expr = args[0].clone().expect_polars_expr()?;
        for arg in args.iter().skip(1) {
            expr = expr * arg.clone().expect_polars_expr()?;
        }
        Ok(new_polars_expr(expr))
    } else if has_float_arg {
        let mut value = args[0].clone().expect_number()?.to_float();
        for arg in args.iter().skip(1) {
            value *= arg.clone().expect_number()?.to_float();
        }
        Ok(new_float(value))
    } else {
        let mut value = args[0].clone().expect_number()?.to_int();
        for arg in args.iter().skip(1) {
            value *= arg.clone().expect_number()?.to_int();
        }
        Ok(new_int(value))
    }
}

// --- Division / Column Division ---
// Examples in Lisp:
//   (/ 100 2 5)
//   (/ $total_cost $quantity)
//   (/ $age 2)
pub fn divide(args: Vec<Expr>) -> Result<Expr, Error> {
    let has_polars_expr_arg = args.iter().any(|arg| matches!(arg.value, ExprValue::PolarsExpr(_)));

    if has_polars_expr_arg {
        let mut expr = args[0]
            .clone()
            .expect_polars_expr()?
            .cast(DataType::Float64);

        for arg in args.iter().skip(1) {
            let next_expr = arg.clone().expect_polars_expr()?;
            expr = expr / next_expr;
        }
        Ok(new_polars_expr(expr))
    } else {
        let mut value = args[0].clone().expect_number()?.to_float();
        for arg in args.iter().skip(1) {
            value /= arg.clone().expect_number()?.to_float();
        }
        Ok(new_float(value))
    }
}

// --- Read CSV File ---
// Examples in Lisp:
//   (read-csv "data.csv")
//   (filter (read-csv "users.csv") (= $country "US"))
pub fn read_csv(args: Vec<Expr>) -> Result<Expr, Error> {
    let path = args[0].clone().expect_string()?;
    match LazyCsvReader::new(path.as_str().into())
        .with_has_header(true)
        .finish() {
            Ok(lf) => Ok(new_lazy_frame(lf)),
            Err(err) => Err(Error::PolarsError(err)),
        }
}

// --- Write CSV File ---
// Examples in Lisp:
//   (write-csv df "output.csv")
//   (write-csv (filter df (> $age 30)) "filtered.csv")
pub fn write_csv(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let path = args[1].clone().expect_string()?;

    let mut df = lf.collect().map_err(Error::PolarsError)?;
    let mut file = File::create(&path).map_err(|err| {
        Error::PolarsError(PolarsError::ComputeError(
            format!("Failed to create file '{}': {}", path, err).into(),
        ))
    })?;

    CsvWriter::new(&mut file)
        .include_header(true)
        .finish(&mut df)
        .map_err(Error::PolarsError)?;

    Ok(nil()) 
}

// (sink df "path/to/output.parquet") or (sink df "path/to/output.csv")
pub fn sink(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let path = args[1].clone().expect_string()?;

    let format = if path.ends_with(".parquet") {
        FileWriteFormat::Parquet(Arc::new(ParquetWriteOptions::default()))
    } else if path.ends_with(".csv") {
        FileWriteFormat::Csv(CsvWriterOptions {
            include_header: true,
            ..Default::default()
        })
    } else {
        return Err(Error::PolarsError(PolarsError::ComputeError(
            format!("Unsupported sink file format for '{}'", path).into(),
        )));
    };

    let destination = SinkDestination::File {
        target: SinkTarget::Path(path.as_str().into()),
    };
    let sink_args = UnifiedSinkArgs::default();

    // lf.sink(...) returns a LazyFrame representing the sink task.
    // .collect() triggers the actual streaming execution to disk.
    lf.sink(destination, format, sink_args)
        .map_err(Error::PolarsError)?
        .collect()
        .map_err(|err| {
            Error::PolarsError(PolarsError::ComputeError(
                format!("sink failed for '{}': {}", path, err).into(),
            ))
        })?;

    Ok(nil())
}

// (read-parquet "path/to/data.parquet") -> LazyFrame
pub fn read_parquet(args: Vec<Expr>) -> Result<Expr, Error> {
    let path = args[0].clone().expect_string()?;
    match LazyFrame::scan_parquet(path.as_str().into(), ScanArgsParquet::default()) {
        Ok(lf) => Ok(new_lazy_frame(lf)),
        Err(err) => Err(Error::PolarsError(err)),
    }
}

// (write-parquet df "path/to/output.parquet") -> nil
pub fn write_parquet(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let path = args[1].clone().expect_string()?;

    let mut df = lf.collect().map_err(Error::PolarsError)?;
    let file = File::create(&path).map_err(|err| {
        Error::PolarsError(PolarsError::ComputeError(
            format!("Failed to create file '{}': {}", path, err).into(),
        ))
    })?;

    ParquetWriter::new(file)
        .finish(&mut df)
        .map_err(Error::PolarsError)?;

    Ok(nil())
}

// --- Get First 10 Rows ---
// Examples in Lisp:
//   (head df)
//   (head (sort df $salary :desc))
pub fn head(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    Ok(new_lazy_frame(lf.limit(10)))
}

// --- Get Last 10 Rows ---
// Examples in Lisp:
//   (tail df)
//   (tail (read-csv "logs.csv"))
pub fn tail(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    Ok(new_lazy_frame(lf.slice(-10, 10)))
}

// --- Slice LazyFrame Rows ---
// Examples in Lisp:
//   (slice df 0 5)   ; offset 0, length 5
//   (slice df 10 20) ; skip 10, take 20
pub fn slice(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let offset = args[1].clone().expect_int()?;
    let len = args[2].clone().expect_int()?;
    Ok(new_lazy_frame(lf.slice(offset, len as u32)))
}

// --- Inspect Schema ---
// Examples in Lisp:
//   (schema df)
//   (schema (read-csv "data.csv"))
pub fn schema(args: Vec<Expr>) -> Result<Expr, Error> {
    let mut lf = args[0].clone().expect_lazy_frame()?; 
    let schema = lf.collect_schema().unwrap();
    let mut result = Vec::<Expr>::new();
    let mut fields = Vec::<Expr>::new();
    for field in schema.iter_fields() {
        
        fields.push(new_list(vec![new_string(field.name.into_string()), new_string(field.dtype.to_string())]));
    }
    result.push(new_list(fields));
    Ok(new_list(result))
}

// --- Inspect Shape ---
// Examples in Lisp:
//   (shape df) -> (rows cols)
pub fn shape(args: Vec<Expr>) -> Result<Expr, Error> {
    let mut lf = args[0].clone().expect_lazy_frame()?;  
    let shape = lf.collect().unwrap().shape();
    Ok(new_list(vec![new_int(shape.0 as i64), new_int(shape.1 as i64)]))
}

// --- Column Reference ---
// Examples in Lisp:
//   $age
//   (col "age")
pub fn col_(args: Vec<Expr>) -> Result<Expr, Error> {
    Ok(new_polars_expr(col(args[0].clone().expect_string()?)))
}

// --- Literal Value ---
// Examples in Lisp:
//   (lit 42)
//   (lit "Alice")
//   (lit 3.14)
pub fn lit_(args: Vec<Expr>) -> Result<Expr, Error> {
    match args[0].value.clone() {
        ExprValue::Int(num) => Ok(new_polars_expr(lit(num))),
        ExprValue::Float(num) => Ok(new_polars_expr(lit(num))),
        ExprValue::String(string) => Ok(new_polars_expr(lit(string))),
        _ => Err(Error::WrongType { expected: String::from("number"), got: args[0].clone() })
    }
}

pub fn eq_core(args: Vec<Expr>) -> Result<Expr, Error> {
    if args[0] == args[1] {
        Ok(new_bool(true))
    } else {
        Ok(new_bool(false))
    }
}

pub fn eq_polars(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let lit = args[1].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.eq(lit)))
}

// --- Equality Comparison (=) ---
// Examples in Lisp:
//   (= 1 1)
//   (= $city "London")
//   (filter df (= $id 10))
pub fn eq(args: Vec<Expr>) -> Result<Expr, Error> {
    if args.iter().any(|arg| matches!(arg.value, ExprValue::PolarsExpr(_))) {
        eq_polars(args)
    } else {
        eq_core(args)
    }
}

pub fn neq_polars(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let lit = args[1].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.neq(lit)))
}

pub fn neq_core(args: Vec<Expr>) -> Result<Expr, Error> {
    if args[0] != args[1] {
        Ok(new_bool(true))
    } else {
        Ok(new_bool(false))
    }
}

// --- Inequality Comparison (!=) ---
// Examples in Lisp:
//   (!= 1 2)
//   (!= $status "inactive")
//   (filter df (!= $age 30))
pub fn neq(args: Vec<Expr>) -> Result<Expr, Error> {
    if args.iter().any(|arg| matches!(arg.value, ExprValue::PolarsExpr(_))) {
        neq_polars(args)
    } else {
        neq_core(args)
    }
}

// Helper function to compare two numbers natively
pub fn cmp_numbers(a: &Expr, b: &Expr) -> Result<std::cmp::Ordering, Error> {
    let num_a = a.clone().expect_number()?;
    let num_b = b.clone().expect_number()?;

    match (a.value.clone(), b.value.clone()) {
        (ExprValue::Int(i1), ExprValue::Int(i2)) => Ok(i1.cmp(&i2)),
        _ => {
            let f1 = num_a.to_float();
            let f2 = num_b.to_float();
            f1.partial_cmp(&f2).ok_or_else(|| {
                Error::Custom("Cannot compare NaN values".to_string())
            })
        }
    }
}

// Greater than (>)
pub fn gt_polars(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let lit = args[1].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.gt(lit)))
}

pub fn gt_core(args: Vec<Expr>) -> Result<Expr, Error> {
    let ord = cmp_numbers(&args[0], &args[1])?;
    Ok(new_bool(ord == std::cmp::Ordering::Greater))
}

// --- Greater Than (>) ---
// Examples in Lisp:
//   (> 5 3)
//   (> $age 25)
//   (filter df (> $salary 80000))
pub fn gt(args: Vec<Expr>) -> Result<Expr, Error> {
    if args.iter().any(|arg| matches!(arg.value, ExprValue::PolarsExpr(_))) {
        gt_polars(args)
    } else {
        gt_core(args)
    }
}

// Less than (<)
pub fn lt_polars(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let lit = args[1].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.lt(lit)))
}

pub fn lt_core(args: Vec<Expr>) -> Result<Expr, Error> {
    let ord = cmp_numbers(&args[0], &args[1])?;
    Ok(new_bool(ord == std::cmp::Ordering::Less))
}

// --- Less Than (<) ---
// Examples in Lisp:
//   (< 2 8)
//   (< $age 30)
//   (filter df (< $price 50.0))
pub fn lt(args: Vec<Expr>) -> Result<Expr, Error> {
    if args.iter().any(|arg| matches!(arg.value, ExprValue::PolarsExpr(_))) {
        lt_polars(args)
    } else {
        lt_core(args)
    }
}

// Greater than or equal (>=)
pub fn gt_eq_polars(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let lit = args[1].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.gt_eq(lit)))
}

pub fn gt_eq_core(args: Vec<Expr>) -> Result<Expr, Error> {
    let ord = cmp_numbers(&args[0], &args[1])?;
    Ok(new_bool(ord == std::cmp::Ordering::Greater || ord == std::cmp::Ordering::Equal))
}

// --- Greater Than or Equal (>=) ---
// Examples in Lisp:
//   (>= 10 10)
//   (>= $age 18)
//   (filter df (>= $score 75))
pub fn gt_eq(args: Vec<Expr>) -> Result<Expr, Error> {
    if args.iter().any(|arg| matches!(arg.value, ExprValue::PolarsExpr(_))) {
        gt_eq_polars(args)
    } else {
        gt_eq_core(args)
    }
}

// Less than or equal (<=)
pub fn lt_eq_polars(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let lit = args[1].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.lt_eq(lit)))
}

pub fn lt_eq_core(args: Vec<Expr>) -> Result<Expr, Error> {
    let ord = cmp_numbers(&args[0], &args[1])?;
    Ok(new_bool(ord == std::cmp::Ordering::Less || ord == std::cmp::Ordering::Equal))
}

// --- Less Than or Equal (<=) ---
// Examples in Lisp:
//   (<= 5 5)
//   (<= $age 65)
//   (filter df (<= $rank 3))
pub fn lt_eq(args: Vec<Expr>) -> Result<Expr, Error> {
    if args.iter().any(|arg| matches!(arg.value, ExprValue::PolarsExpr(_))) {
        lt_eq_polars(args)
    } else {
        lt_eq_core(args)
    }
}

pub fn not_polars(args: Vec<Expr>) -> Result<Expr, Error> {
    let mut combined = args[0].clone().expect_polars_expr()?;
    for arg in args.iter().skip(1) {
        let next_expr = arg.clone().expect_polars_expr()?;
        combined = combined.and(next_expr);
    }
    Ok(new_polars_expr(combined.not()))
}

pub fn not_core(args: Vec<Expr>) -> Result<Expr, Error> {
    let mut combined = true;
    for arg in args {
        combined = combined && arg.expect_bool()?;
    }
    Ok(new_bool(!combined))
}

// --- Logical NOT ---
// Examples in Lisp:
//   (not true)
//   (not true false)                 ; returns true because (true AND false) is false
//   (not (> $age 25))
//   (not (> $age 25) (= $city "Paris")) ; negates the AND of all conditions
//   (filter df (not (= $city "London") (> $age 30)))
pub fn not(args: Vec<Expr>) -> Result<Expr, Error> {
    if args.iter().any(|arg| matches!(arg.value, ExprValue::PolarsExpr(_))) {
        not_polars(args)
    } else {
        not_core(args)
    }
}

pub fn and_polars(args: Vec<Expr>) -> Result<Expr, Error> {
    let mut combined = args[0].clone().expect_polars_expr()?;
    for arg in args.iter().skip(1) {
        let next_expr = arg.clone().expect_polars_expr()?;
        combined = combined.and(next_expr);
    }
    Ok(new_polars_expr(combined))
}

pub fn and_core(args: Vec<Expr>) -> Result<Expr, Error> {
    let mut result = true;
    for arg in args {
        result = result && arg.expect_bool()?;
        if !result {
            return Ok(new_bool(false));
        }
    }
    Ok(new_bool(result))
}

// --- Logical AND ---
// Examples in Lisp:
//   (and true false)
//   (and true true true)
//   (and (> $age 20) (< $age 40) (= $department "Engineering"))
//   (filter df (and (= $department "HR") (> $salary 50000) (< $age 30)))
pub fn and(args: Vec<Expr>) -> Result<Expr, Error> {
    if args.iter().any(|arg| matches!(arg.value, ExprValue::PolarsExpr(_))) {
        and_polars(args)
    } else {
        and_core(args)
    }
}

pub fn or_polars(args: Vec<Expr>) -> Result<Expr, Error> {
    let mut combined = args[0].clone().expect_polars_expr()?;
    for arg in args.iter().skip(1) {
        let next_expr = arg.clone().expect_polars_expr()?;
        combined = combined.or(next_expr);
    }
    Ok(new_polars_expr(combined))
}

pub fn or_core(args: Vec<Expr>) -> Result<Expr, Error> {
    let mut result = false;
    for arg in args {
        result = result || arg.expect_bool()?;
        if result {
            return Ok(new_bool(true));
        }
    }
    Ok(new_bool(result))
}

// --- Logical OR ---
// Examples in Lisp:
//   (or true false)
//   (or false false true)
//   (or (= $city "Paris") (= $city "Berlin") (= $city "London"))
//   (filter df (or (< $age 25) (>= $age 60) (= $department "Executive")))
pub fn or(args: Vec<Expr>) -> Result<Expr, Error> {
    if args.iter().any(|arg| matches!(arg.value, ExprValue::PolarsExpr(_))) {
        or_polars(args)
    } else {
        or_core(args)
    }
}

pub fn sum(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.sum()))
}

pub fn mean(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.mean()))
}

pub fn median(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.median()))
}

pub fn count(args: Vec<Expr>) -> Result<Expr, Error> {
    if args.is_empty() {
        // (count) without arguments returns the total number of rows
        Ok(new_polars_expr(polars::prelude::len()))
    } else {
        // (count $col) returns non-null count for that column
        let expr = args[0].clone().expect_polars_expr()?;
        Ok(new_polars_expr(expr.count()))
    }
}

pub fn min(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.min()))
}

pub fn max(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.max()))
}

pub fn n_unique(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.n_unique()))
}

pub fn std_dev(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    // Default to sample standard deviation (ddof = 1); allow overriding via second argument
    let ddof = if args.len() > 1 {
        args[1].clone().expect_int()? as u8
    } else {
        1
    };
    Ok(new_polars_expr(expr.std(ddof)))
}

pub fn select(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let mut exprs = Vec::<polars::prelude::Expr>::new();
    for arg in args.iter().skip(1) {
        exprs.push(arg.clone().expect_polars_expr()?);
    }

    // Eagerly validate all expressions against the current schema
    lf.clone()
        .select(exprs.clone())
        .collect_schema()
        .map_err(Error::PolarsError)?;

    Ok(new_lazy_frame(lf.select(exprs)))
}

pub fn filter(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let expr = args[1].clone().expect_polars_expr()?;

    // Eagerly validate that all columns in `expr` exist in `lf`
    lf.clone()
        .select(vec![expr.clone()])
        .collect_schema()
        .map_err(Error::PolarsError)?;

    Ok(new_lazy_frame(lf.filter(expr)))
}

pub fn with_column(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let alias_expr = args[1].clone().expect_polars_expr()?;
    let value = args[2].clone().expect_polars_expr()?;
    let alias = match alias_expr {
        polars::prelude::Expr::Column(name) => name.to_string(),
        _ => {
            panic!("Shouldnt happen");
        }
    };

    // Eagerly validate `value`
    lf.clone()
        .select(vec![value.clone()])
        .collect_schema()
        .map_err(Error::PolarsError)?;

    Ok(new_lazy_frame(lf.with_column(value.alias(&alias))))
}

// --- 1. Multi-column Sorting with Mixed Orders ---
// Examples in Lisp:
//   (sort df $a)
//   (sort df $a :desc)
//   (sort df ($a $b) (:desc :asc))
//   (sort df ($a $b) :desc) ; sets descending for all
pub fn sort(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;

    // 1. Extract by expressions
    let by_exprs: Vec<polars::prelude::Expr> = match &args[1].value {
        ExprValue::List(items) => {
            let mut exprs = Vec::with_capacity(items.len());
            for item in items {
                exprs.push(item.clone().expect_polars_expr()?);
            }
            exprs
        }
        _ => vec![args[1].clone().expect_polars_expr()?],
    };

    // 2. Extract descending flags
    let descending: Vec<bool> = if args.len() > 2 {
        match &args[2].value {
            ExprValue::List(items) => {
                let mut desc = Vec::with_capacity(items.len());
                for item in items {
                    let sym = item.clone().expect_symbol()?;
                    desc.push(matches!(sym.as_str(), ":desc" | ":descending" | "true"));
                }
                desc
            }
            _ => {
                let sym = args[2].clone().expect_symbol()?;
                let is_desc = matches!(sym.as_str(), ":desc" | ":descending" | "true");
                vec![is_desc; by_exprs.len()]
            }
        }
    } else {
        vec![false; by_exprs.len()]
    };

    if !descending.is_empty() && descending.len() != by_exprs.len() {
        return Err(Error::Custom(format!(
            "Number of sort orders ({}) does not match number of columns ({})",
            descending.len(),
            by_exprs.len()
        )));
    }

    let options = SortMultipleOptions::new().with_order_descending_multi(descending);
    Ok(new_lazy_frame(lf.sort_by_exprs(by_exprs, options)))
}

// (slice-expr $col offset length)
pub fn expr_slice(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let offset = args[1].clone().expect_int()?;
    let length = args[2].clone().expect_int()?;

    Ok(new_polars_expr(expr.slice(lit(offset), lit(length as u32))))
}

// (get $col 0) -> first element
// (get $col -1) -> last element
pub fn expr_get(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let index = args[1].clone().expect_int()?;

    // Set null_on_oob to true (returns null if out of bounds) or false (raises an error)
    Ok(new_polars_expr(expr.get(lit(index), true)))
}

pub fn group_by(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;

    // Support either a single grouping column or a list of grouping columns
    let group_keys = match &args[1].value {
        ExprValue::List(cols) => {
            let mut exprs = Vec::new();
            for col_expr in cols {
                exprs.push(col_expr.clone().expect_polars_expr()?);
            }
            exprs
        }
        _ => vec![args[1].clone().expect_polars_expr()?],
    };

    // Collect all trailing expressions as aggregations
    let mut agg_exprs = Vec::<polars::prelude::Expr>::new();
    for arg in args.iter().skip(2) {
        agg_exprs.push(arg.clone().expect_polars_expr()?);
    }

    Ok(new_lazy_frame(lf.group_by(group_keys).agg(agg_exprs)))
}

pub fn alias(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let name = args[1].clone().expect_string()?;
    Ok(new_polars_expr(expr.alias(&name)))
}

pub fn agg(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let mut exprs = Vec::<polars::prelude::Expr>::new();
    
    // Collect all expression arguments passed to agg
    for arg in args.iter().skip(1) {
        exprs.push(arg.clone().expect_polars_expr()?);
    }
    
    // Polars aggregates entire lazy frames via select
    Ok(new_lazy_frame(lf.select(exprs)))
}

pub fn join(args: Vec<Expr>) -> Result<Expr, Error> {
    let left_lf = args[0].clone().expect_lazy_frame()?;
    let right_lf = args[1].clone().expect_lazy_frame()?;
    
    // Parse left key(s)
    let left_on = match &args[2].value {
        ExprValue::List(cols) => {
            let mut exprs = Vec::new();
            for c in cols {
                exprs.push(c.clone().expect_polars_expr()?);
            }
            exprs
        }
        _ => vec![args[2].clone().expect_polars_expr()?],
    };

    // Parse right key(s)
    let right_on = match &args[3].value {
        ExprValue::List(cols) => {
            let mut exprs = Vec::new();
            for c in cols {
                exprs.push(c.clone().expect_polars_expr()?);
            }
            exprs
        }
        _ => vec![args[3].clone().expect_polars_expr()?],
    };

    // Determine join type (defaults to Inner)
    let how = if args.len() > 4 {
        match args[4].clone().expect_symbol()?.as_str() {
            ":left" => JoinType::Left,
            ":outer" | ":full" => JoinType::Full,
            ":cross" => JoinType::Cross,
            ":semi" => JoinType::Semi,
            ":anti" => JoinType::Anti,
            _ => JoinType::Inner,
        }
    } else {
        JoinType::Inner
    };

    let joined = left_lf.join(
        right_lf,
        left_on,
        right_on,
        JoinArgs::new(how),
    );

    Ok(new_lazy_frame(joined))
}

pub fn is_null_(args: Vec<Expr>) -> Result<Expr, Error> {
    let col = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(is_null(col)))
}

pub fn is_not_null_(args: Vec<Expr>) -> Result<Expr, Error> {
    let col = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(is_not_null(col)))
}

pub fn fill_null_(args: Vec<Expr>) -> Result<Expr, Error> {
    let col = args[0].clone().expect_polars_expr()?;
    let value = args[1].clone().expect_polars_expr()?;
    Ok(new_polars_expr(col.fill_null(value)))
}

pub fn drop_nulls(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    if args.len() == 1 {
        Ok(new_lazy_frame(lf.drop_nulls(None)))
    } else {
        let mut col_names = Vec::<String>::new();

        let mut extract_name = |expr: &Expr| -> Result<(), Error> {
            match expr.value.clone() {
                ExprValue::String(s) | ExprValue::Symbol(s) => {
                    col_names.push(s.clone());
                    Ok(())
                }
                ExprValue::PolarsExpr(polars::prelude::Expr::Column(name)) => {
                    col_names.push(name.to_string());
                    Ok(())
                }
                other => Err(Error::WrongType {
                    expected: String::from("column name or $col"),
                    got: expr.clone(),
                }),
            }
        };

        for arg in args.iter().skip(1) {
            match arg.value.clone() {
                ExprValue::List(list) => {
                    for item in list {
                        extract_name(&item)?;
                    }
                }
                _ => extract_name(arg)?,
            }
        }

        Ok(new_lazy_frame(lf.drop_nulls(Some(cols(col_names).into()))))
    }
}

pub fn when(args: Vec<Expr>) -> Result<Expr, Error> {
    let condition = args[0].clone().expect_polars_expr()?;
    let on_true = args[1].clone().expect_polars_expr()?;
    match args.len() {
        2 => Ok(new_polars_expr(polars::prelude::when(condition).then(on_true).otherwise(polars::prelude::lit(polars::prelude::NULL)))),
        _ => {
            let on_false = args[2].clone().expect_polars_expr()?;
            Ok(new_polars_expr(polars::prelude::when(condition).then(on_true).otherwise(on_false)))
        }
    }
}

// (str-contains $name "^A.*")
pub fn str_contains(args: Vec<Expr>) -> Result<Expr, Error> {
    if args.len() < 2 {
        return Err(Error::WrongType {
            expected: "2 arguments (column, pattern)".into(),
            got: args[0].clone(),
        });
    }
    let col_expr = args[0].clone().expect_polars_expr()?;
    let pat_expr = args[1].clone().expect_polars_expr()?;
    
    // Polars: expr.str().contains(pattern, strict)
    Ok(new_polars_expr(col_expr.str().contains(pat_expr, true)))
}

// (str-replace $city "^Lon" "New-Lon")
pub fn str_replace(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let pat_expr = args[1].clone().expect_polars_expr()?;
    let rep_expr = args[2].clone().expect_polars_expr()?;

    // false = interpret pattern as regex, not literal
    Ok(new_polars_expr(col_expr.str().replace(pat_expr, rep_expr, false)))
}

// (str-extract $code "([A-Z]+)-(\\d+)" 1)
pub fn str_extract(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let pat_expr = args[1].clone().expect_polars_expr()?;
    let group_idx = args[2].clone().expect_int()? as usize;

    Ok(new_polars_expr(col_expr.str().extract(pat_expr, group_idx)))
}

// (over <expr> <partition_col1> ...) or (over <expr> (<col1> <col2>))
pub fn over(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;

    let partition_by = match &args[1].value {
        ExprValue::List(cols) => {
            let mut exprs = Vec::new();
            for col_expr in cols {
                exprs.push(col_expr.clone().expect_polars_expr()?);
            }
            exprs
        }
        _ => {
            let mut exprs = Vec::new();
            for arg in args.iter().skip(1) {
                exprs.push(arg.clone().expect_polars_expr()?);
            }
            exprs
        }
    };

    match expr.over(partition_by) {
        Ok(result) => Ok(new_polars_expr(result)),
        Err(err) => Err(Error::PolarsError(err)),
    } 
}

// (shift $col 1) or (shift $col -1)
pub fn shift(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let offset = args[1].clone().expect_int()? as i64;
    Ok(new_polars_expr(expr.shift(lit(offset))))
}

// (cum-sum $col)
pub fn cum_sum(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let reverse = if args.len() > 1 {
        matches!(args[1].clone().expect_symbol()?.as_str(), ":desc" | ":reverse")
    } else {
        false
    };
    
    Ok(new_polars_expr(expr.cum_sum(reverse)))
}

// (cum-min $col)
pub fn cum_min(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let reverse = if args.len() > 1 {
        matches!(args[1].clone().expect_symbol()?.as_str(), ":desc" | ":reverse")
    } else {
        false
    };
    Ok(new_polars_expr(expr.cum_min(reverse)))
}

// (cum-max $col)
pub fn cum_max(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let reverse = if args.len() > 1 {
        matches!(args[1].clone().expect_symbol()?.as_str(), ":desc" | ":reverse")
    } else {
        false
    };
    Ok(new_polars_expr(expr.cum_max(reverse)))
}

// (cum-count $col)
pub fn cum_count(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let reverse = if args.len() > 1 {
        matches!(args[1].clone().expect_symbol()?.as_str(), ":desc" | ":reverse")
    } else {
        false
    };
    Ok(new_polars_expr(expr.cum_count(reverse)))
}

// (rank $col) or (rank $col :dense)
pub fn rank(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let method = if args.len() > 1 {
        match args[1].clone().expect_symbol()?.as_str() {
            ":dense" => RankMethod::Dense,
            ":min" => RankMethod::Min,
            ":max" => RankMethod::Max,
            ":average" => RankMethod::Average,
            ":ordinal" | ":first" => RankMethod::Ordinal,
            _ => RankMethod::Average,
        }
    } else {
        RankMethod::Average
    };

    let options = RankOptions {
        method,
        descending: false,
    };

    Ok(new_polars_expr(expr.rank(options, None)))
}

// (to-date $col "%Y-%m-%d") or (to-date $col) -> ISO-8601 default
pub fn to_date(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let format = if args.len() > 1 {
        Some(args[1].clone().expect_string()?)
    } else {
        None
    };

    let options = StrptimeOptions {
        format: format.map(|s| PlSmallStr::from(s.as_str())),
        strict: true,
        exact: true,
        cache: true,
    };

    Ok(new_polars_expr(col_expr.str().to_date(options)))
} 

// (to-datetime $col "%Y-%m-%d %H:%M:%S")
pub fn to_datetime(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let format = if args.len() > 1 {
        Some(args[1].clone().expect_string()?)
    } else {
        None
    };

    let options = StrptimeOptions {
        format: format.map(|s| PlSmallStr::from(s.as_str())),
        strict: true,
        exact: true,
        cache: true,
    };

    // Defaults to Microseconds timeunit, no timezone
    Ok(new_polars_expr(col_expr.str().to_datetime(
        Some(TimeUnit::Microseconds),
        None,
        options,
        polars::prelude::lit("raise"),
    )))
}

// (dt-year $date_col)
pub fn dt_year(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(col_expr.dt().year()))
}

// (dt-month $date_col)
pub fn dt_month(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(col_expr.dt().month()))
}

// (dt-day $date_col)
pub fn dt_day(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(col_expr.dt().day()))
}

// (dt-hour $datetime_col)
pub fn dt_hour(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(col_expr.dt().hour()))
}

// (dt-format $date_col "%B %d, %Y")
pub fn dt_format(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let format = args[1].clone().expect_string()?;
    Ok(new_polars_expr(col_expr.dt().to_string(&format)))
}

// Helper to extract a column name string from String, Symbol, or $col Expr
pub fn extract_col_name(expr: &Expr) -> Result<String, Error> {
    match expr.value.clone() {
        ExprValue::String(s) | ExprValue::Symbol(s) => Ok(s.clone()),
        ExprValue::PolarsExpr(polars::prelude::Expr::Column(name)) => Ok(name.to_string()),
        other => Err(Error::WrongType {
            expected: String::from("column name or $col"),
            got: expr.clone(),
        }),
    }
}

// Helper to extract a Vec<String> from either a single item or Expr::List
pub fn extract_col_names(arg: &Expr) -> Result<Vec<String>, Error> {
    match arg.value.clone() {
        ExprValue::List(list) => list.iter().map(extract_col_name).collect(),
        _ => Ok(vec![extract_col_name(arg)?]),
    }
}

pub fn pivot_(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let on_cols = extract_col_names(&args[1])?;
    let index_cols = extract_col_names(&args[2])?;
    let values_cols = extract_col_names(&args[3])?;

    let agg_sym = if args.len() > 4 {
        args[4].clone().expect_symbol()?
    } else {
        ":first".to_string()
    };

    let agg_expr = match agg_sym.as_str() {
        ":sum" => polars::prelude::element().sum(),
        ":min" => polars::prelude::element().min(),
        ":max" => polars::prelude::element().max(),
        ":mean" | ":avg" => polars::prelude::element().mean(),
        ":median" => polars::prelude::element().median(),
        ":count" | ":len" => polars::prelude::len(),
        ":last" => polars::prelude::element().last(),
        ":first" | _ => polars::prelude::element().first(),
    };

    // 1. Select distinct on_cols and SORT them for deterministic column ordering
    let sort_options = SortMultipleOptions::new().with_order_descending(false);
    let pivot_values = Arc::new(
        lf.clone()
            .select([cols(on_cols.clone()).as_expr()])
            .unique(None, UniqueKeepStrategy::Any)
            .sort_by_exprs(vec![cols(on_cols.clone()).as_expr()], sort_options)
            .collect()
            .map_err(Error::PolarsError)?,
    );

    // 2. Pivot LazyFrame
    let pivoted = lf.pivot(
        cols(on_cols),
        pivot_values,
        cols(index_cols),
        cols(values_cols),
        agg_expr,
        false,
        PlSmallStr::from_static("_"),
        Default::default(),
    );

    Ok(new_lazy_frame(pivoted))
}

pub fn melt_(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let id_vars = extract_col_names(&args[1])?;
    let value_vars = extract_col_names(&args[2])?;

    let var_name = if args.len() > 3 {
        Some(PlSmallStr::from(args[3].clone().expect_string()?.as_str()))
    } else {
        None
    };

    let val_name = if args.len() > 4 {
        Some(PlSmallStr::from(args[4].clone().expect_string()?.as_str()))
    } else {
        None
    };

    let unpivot_args = UnpivotArgsDSL {
        index: cols(id_vars).into(),
        on: cols(value_vars).into(),
        variable_name: var_name,
        value_name: val_name,
    };

    Ok(new_lazy_frame(lf.unpivot(unpivot_args)))
}

pub fn concat_(args: Vec<Expr>) -> Result<Expr, Error> {
    let mut frames = Vec::<LazyFrame>::new();
    let mut how_str: Option<String> = None;

    let items = match &args[0].value {
        ExprValue::List(list) => list.as_slice(),
        _ => &args[..],
    };

    for item in items {
        match item.value.clone() {
            ExprValue::LazyFrame(lf) => frames.push(lf.clone()),
            ExprValue::Symbol(s) if s.starts_with(':') => how_str = Some(s.clone()),
            _ => {
                if let Ok(lf) = item.clone().expect_lazy_frame() {
                    frames.push(lf);
                }
            }
        }
    }

    if args.len() > 1 && how_str.is_none() {
        if let Ok(sym) = args.last().unwrap().clone().expect_symbol() {
            if sym.starts_with(':') {
                how_str = Some(sym);
            }
        }
    }

    let concatted = match how_str.as_deref() {
        Some(":horizontal") | Some(":horiz") => {
            concat_lf_horizontal(frames, Default::default()).map_err(Error::PolarsError)?
        }
        Some(":diagonal") | Some(":diag") => {
            let mut union_args = UnionArgs::default();
            union_args.diagonal = true;
            polars::prelude::concat(frames, union_args).map_err(Error::PolarsError)?
        }
        _ => {
            // Default: Vertical concatenation
            polars::prelude::concat(frames, UnionArgs::default()).map_err(Error::PolarsError)?
        }
    };

    Ok(new_lazy_frame(concatted))
}

pub fn cast_(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let target_type = match args[1].clone().expect_symbol()?.as_str() {
        ":str" | ":string" => DataType::String,
        ":int" | ":i64" => DataType::Int64,
        ":float" | ":f64" => DataType::Float64,
        ":date" => DataType::Date,
        ":datetime" => DataType::Datetime(TimeUnit::Microseconds, None),
        ":bool" => DataType::Boolean,
        other => return Err(Error::Custom(format!("Unknown cast target: {}", other))),
    };
    Ok(new_polars_expr(col_expr.cast(target_type)))
}

// Helper to construct window options: (expr, window_size, [min_periods], [center])
pub fn parse_rolling_options(args: &[Expr]) -> Result<(polars::prelude::Expr, RollingOptionsFixedWindow), Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let window_size = args[1].clone().expect_int()? as usize;

    let min_periods = if args.len() > 2 {
        args[2].clone().expect_int()? as usize
    } else {
        1
    };

    let center = if args.len() > 3 {
        match args[3].clone().expect_symbol()?.as_str() {
            ":center" | "true" => true,
            _ => false,
        }
    } else {
        false
    };

    let options = RollingOptionsFixedWindow {
        window_size,
        min_periods,
        weights: None,
        center,
        fn_params: None,
    };

    Ok((expr, options))
}

// (rolling-mean $col 5) or (rolling-mean $col 5 1 :center)
pub fn rolling_mean(args: Vec<Expr>) -> Result<Expr, Error> {
    let (expr, opts) = parse_rolling_options(&args)?;
    Ok(new_polars_expr(expr.rolling_mean(opts)))
}

// (rolling-sum $col 5)
pub fn rolling_sum(args: Vec<Expr>) -> Result<Expr, Error> {
    let (expr, opts) = parse_rolling_options(&args)?;
    Ok(new_polars_expr(expr.rolling_sum(opts)))
}

// (rolling-min $col 5)
pub fn rolling_min(args: Vec<Expr>) -> Result<Expr, Error> {
    let (expr, opts) = parse_rolling_options(&args)?;
    Ok(new_polars_expr(expr.rolling_min(opts)))
}

// (rolling-max $col 5)
pub fn rolling_max(args: Vec<Expr>) -> Result<Expr, Error> {
    let (expr, opts) = parse_rolling_options(&args)?;
    Ok(new_polars_expr(expr.rolling_max(opts)))
}

// (rolling-std $col 5)
pub fn rolling_std(args: Vec<Expr>) -> Result<Expr, Error> {
    let (expr, opts) = parse_rolling_options(&args)?;
    Ok(new_polars_expr(expr.rolling_std(opts)))
}

// (rolling-median $col 5)
pub fn rolling_median(args: Vec<Expr>) -> Result<Expr, Error> {
    let (expr, opts) = parse_rolling_options(&args)?;
    Ok(new_polars_expr(expr.rolling_median(opts)))
}

// (str-split $col ",")
pub fn str_split(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let pat_expr = args[1].clone().expect_polars_expr()?;
    Ok(new_polars_expr(col_expr.str().split(pat_expr)))
}

// (str-strip $col [chars])
pub fn str_strip(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let expr = if args.len() > 1 {
        args[1].clone().expect_polars_expr()?
    } else {
        polars::prelude::lit(polars::prelude::NULL)
    };
    Ok(new_polars_expr(col_expr.str().strip_chars(expr)))
}

// (str-strip-start $col [chars])
pub fn str_strip_start(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let expr = if args.len() > 1 {
        args[1].clone().expect_polars_expr()?
    } else {
        polars::prelude::lit(polars::prelude::NULL)
    };
    Ok(new_polars_expr(col_expr.str().strip_chars_start(expr)))
}

// (str-strip-end $col [chars])
pub fn str_strip_end(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let expr = if args.len() > 1 {
        args[1].clone().expect_polars_expr()?
    } else {
        polars::prelude::lit(polars::prelude::NULL)
    };
    Ok(new_polars_expr(col_expr.str().strip_chars_end(expr)))
}

// (str-slice $col offset length)
pub fn str_slice(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let offset = args[1].clone().expect_int()?;
    let length = args[2].clone().expect_int()?;
    Ok(new_polars_expr(col_expr.str().slice(lit(offset), lit(length))))
}

// (str-extract-groups $col "([A-Z]+)-(\\d+)")
pub fn str_extract_groups(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let pat = args[1].clone().expect_string()?;
    
    let res = col_expr.str().extract_groups(&pat).map_err(Error::PolarsError)?;
    Ok(new_polars_expr(res))
}

// (scalar expr) — materialize a 1x1 result into a Lisp scalar
pub fn scalar(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let df = lf.collect().map_err(Error::PolarsError)?;
    if df.shape() != (1, 1) { return Err(Error::ScalarShapeError { shape: df.shape() }) }
    let row = df.get(0).unwrap();
    match row[0].clone() {
        AnyValue::Null => Ok(nil()),
        AnyValue::Boolean(b) => Ok(new_bool(b)),
        AnyValue::String(s) => Ok(new_string(s.to_string())),
        AnyValue::UInt8(i) => Ok(new_int(i.into())),
        AnyValue::UInt16(i) => Ok(new_int(i.into())),
        AnyValue::UInt32(i) => Ok(new_int(i.into())),
        AnyValue::UInt64(i) => Ok(new_int(i as i64)),
        AnyValue::UInt128(i) => Ok(new_int(i as i64)),
        AnyValue::Int8(i) => Ok(new_int(i as i64)),
        AnyValue::Int16(i) => Ok(new_int(i as i64)),
        AnyValue::Int32(i) => Ok(new_int(i as i64)),
        AnyValue::Int64(i) => Ok(new_int(i as i64)),
        AnyValue::Int128(i) => Ok(new_int(i as i64)),
        AnyValue::Float16(f) => Ok(new_float(f.into())),
        AnyValue::Float32(f) => Ok(new_float(f.into())),
        AnyValue::Float64(f) => Ok(new_float(f.into())),
        AnyValue::Date(_) => Err(Error::ScalarUnsupportedError { name: "Date".to_string() }),
        AnyValue::Datetime(_, time_unit, time_zone) => Err(Error::ScalarUnsupportedError { name: "DateTime".to_string() }),
        AnyValue::DatetimeOwned(_, time_unit, time_zone) => Err(Error::ScalarUnsupportedError { name: "DatetimeOwned".to_string() }),
        AnyValue::Duration(_, time_unit) => Err(Error::ScalarUnsupportedError { name: "Duration".to_string() }),
        AnyValue::Time(_) => Err(Error::ScalarUnsupportedError { name: "Time".to_string() }),
        AnyValue::Categorical(_, categorical_mapping) => Err(Error::ScalarUnsupportedError { name: "Categorical".to_string() }),
        AnyValue::CategoricalOwned(_, categorical_mapping) => Err(Error::ScalarUnsupportedError { name: "CategoricalOwned".to_string() }),
        AnyValue::Enum(_, categorical_mapping) => Err(Error::ScalarUnsupportedError { name: "Enum".to_string() }),
        AnyValue::EnumOwned(_, categorical_mapping) => Err(Error::ScalarUnsupportedError { name: "EnumOwned".to_string() }),
        AnyValue::List(series) => Err(Error::ScalarUnsupportedError { name: "List".to_string() }),
        AnyValue::Struct(_, struct_array, fields) => Err(Error::ScalarUnsupportedError { name: "Struct".to_string() }),
        AnyValue::StructOwned(_) => Err(Error::ScalarUnsupportedError { name: "StructOwned".to_string() }),
        AnyValue::StringOwned(pl_small_str) => Ok(new_string(pl_small_str.to_string())),
        AnyValue::Binary(items) => Err(Error::ScalarUnsupportedError { name: "Binary".to_string() }),
        AnyValue::BinaryOwned(items) => Err(Error::ScalarUnsupportedError { name: "BinaryOwned".to_string() }),
        _ => Err(Error::ScalarUnsupportedError { name: "Unknown".to_string() })
    }
}

pub fn collect_(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    lf.collect_with_engine(polars::prelude::Engine::Streaming).unwrap();
    Ok(nil())
}

// (var $col) or (var $col 1)
pub fn variance(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let ddof = if args.len() > 1 {
        args[1].clone().expect_int()? as u8
    } else {
        1
    };
    Ok(new_polars_expr(expr.var(ddof)))
}

// (quantile $col 0.5) or (quantile $col 0.5 :linear)
pub fn quantile(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let q = args[1].clone().expect_number()?.to_float();
    
    let method = if args.len() > 2 {
        match args[2].clone().expect_symbol()?.as_str() {
            ":linear" => QuantileMethod::Linear,
            ":lower" => QuantileMethod::Lower,
            ":higher" => QuantileMethod::Higher,
            ":midpoint" => QuantileMethod::Midpoint,
            _ => QuantileMethod::Nearest,
        }
    } else {
        QuantileMethod::Nearest
    };

    Ok(new_polars_expr(expr.quantile(polars::prelude::lit(q), method)))
}

// (cov $col_a $col_b) or (cov $col_a $col_b 1)
pub fn cov_(args: Vec<Expr>) -> Result<Expr, Error> {
    let a = args[0].clone().expect_polars_expr()?;
    let b = args[1].clone().expect_polars_expr()?;
    let ddof = if args.len() > 2 {
        args[2].clone().expect_int()? as u8
    } else {
        1
    };

    Ok(new_polars_expr(polars::lazy::dsl::cov(a, b, ddof)))
}

// (corr $col_a $col_b)
pub fn corr(args: Vec<Expr>) -> Result<Expr, Error> {
    let a = args[0].clone().expect_polars_expr()?;
    let b = args[1].clone().expect_polars_expr()?;

    Ok(new_polars_expr(polars::lazy::dsl::pearson_corr(a, b)))
}

// (describe df)
pub fn describe(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let schema = lf.clone().collect_schema().map_err(Error::PolarsError)?;

    // We generate summary statistic rows for all numeric fields
    let mut stats_exprs = Vec::new();

    for field in schema.iter_fields() {
        let name = field.name();
        let c = col(name.clone());

        if field.dtype().is_numeric() {
            stats_exprs.push(c.clone().count().cast(DataType::Float64).alias(&format!("{}_count", name)));
            stats_exprs.push(c.clone().null_count().cast(DataType::Float64).alias(&format!("{}_null_count", name)));
            stats_exprs.push(c.clone().mean().alias(&format!("{}_mean", name)));
            stats_exprs.push(c.clone().std(1).alias(&format!("{}_std", name)));
            stats_exprs.push(c.clone().min().cast(DataType::Float64).alias(&format!("{}_min", name)));
            stats_exprs.push(c.clone().quantile(lit(0.25), QuantileMethod::Linear).cast(DataType::Float64).alias(&format!("{}_25%", name)));
            stats_exprs.push(c.clone().median().cast(DataType::Float64).alias(&format!("{}_50%", name)));
            stats_exprs.push(c.clone().quantile(lit(0.75), QuantileMethod::Linear).cast(DataType::Float64).alias(&format!("{}_75%", name)));
            stats_exprs.push(c.clone().max().cast(DataType::Float64).alias(&format!("{}_max", name)));
        } else {
            stats_exprs.push(c.clone().count().cast(DataType::Float64).alias(&format!("{}_count", name)));
            stats_exprs.push(c.clone().null_count().cast(DataType::Float64).alias(&format!("{}_null_count", name)));
        }
    }

    Ok(new_lazy_frame(lf.select(stats_exprs)))
}
