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

pub fn nil() -> Expr { return Expr{ value: ExprValue::String("nil".to_string()), line: None } }

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

pub fn new_polars_expr(value: polars::prelude::Expr) -> Expr {
    Expr { value: ExprValue::PolarsExpr(value), line: None }
}

pub fn new_lazy_frame(value: LazyFrame) -> Expr {
    Expr { value: ExprValue::LazyFrame(value), line: None }
}

pub fn new_symbol(value: String) -> Expr {
    Expr { value: ExprValue::Symbol(value), line: None }
}

fn print(args: Vec<Expr>) -> Result<Expr, Error> {
    for arg in args {
        println!("{}", arg.to_string()?);
    }
    Ok(nil())
}

fn list(args: Vec<Expr>) -> Result<Expr, Error> {
    Ok(new_list(args))
}

fn plus(args: Vec<Expr>) -> Result<Expr, Error> {
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

fn minus(args: Vec<Expr>) -> Result<Expr, Error> {
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

fn mult(args: Vec<Expr>) -> Result<Expr, Error> {
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

fn divide(args: Vec<Expr>) -> Result<Expr, Error> {
    let has_polars_expr_arg = args.iter().any(|arg| matches!(arg.value, ExprValue::PolarsExpr(_)));
    // let has_float_arg = args.iter().any(|arg| matches!(arg, Expr::Float(_)));
    // Always compute real division in f64

    if has_polars_expr_arg {
        // Cast the initial expression to Float64 so division produces f64
        let mut expr = args[0].clone().expect_polars_expr()?;
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

fn read_csv(args: Vec<Expr>) -> Result<Expr, Error> {
    let path = args[0].clone().expect_string()?;
    match LazyCsvReader::new(path.as_str().into())
        .with_has_header(true)
        .finish() {
            Ok(lf) => Ok(new_lazy_frame(lf)),
            Err(err) => Err(Error::PolarsError(err)),
        }
}

fn write_csv(args: Vec<Expr>) -> Result<Expr, Error> {
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
fn sink(args: Vec<Expr>) -> Result<Expr, Error> {
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
fn read_parquet(args: Vec<Expr>) -> Result<Expr, Error> {
    let path = args[0].clone().expect_string()?;
    match LazyFrame::scan_parquet(path.as_str().into(), ScanArgsParquet::default()) {
        Ok(lf) => Ok(new_lazy_frame(lf)),
        Err(err) => Err(Error::PolarsError(err)),
    }
}

// (write-parquet df "path/to/output.parquet") -> nil
fn write_parquet(args: Vec<Expr>) -> Result<Expr, Error> {
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

fn head(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    Ok(new_lazy_frame(lf.limit(10)))
}

fn tail(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    Ok(new_lazy_frame(lf.slice(-10, 10)))
}

fn slice(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let offset = args[1].clone().expect_int()?;
    let len = args[2].clone().expect_int()?;
    Ok(new_lazy_frame(lf.slice(offset, len as u32)))
}

fn schema(args: Vec<Expr>) -> Result<Expr, Error> {
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

fn shape(args: Vec<Expr>) -> Result<Expr, Error> {
    let mut lf = args[0].clone().expect_lazy_frame()?;  
    let shape = lf.collect().unwrap().shape();
    Ok(new_list(vec![new_int(shape.0 as i64), new_int(shape.1 as i64)]))
}

fn col_(args: Vec<Expr>) -> Result<Expr, Error> {
    Ok(new_polars_expr(col(args[0].clone().expect_string()?)))
}

fn lit_(args: Vec<Expr>) -> Result<Expr, Error> {
    match args[0].value.clone() {
        ExprValue::Int(num) => Ok(new_polars_expr(lit(num))),
        ExprValue::Float(num) => Ok(new_polars_expr(lit(num))),
        ExprValue::String(string) => Ok(new_polars_expr(lit(string))),
        _ => Err(Error::WrongType { expected: String::from("number"), got: args[0].clone() })
    }
}

fn eq(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let lit = args[1].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.eq(lit)))
}

fn neq(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let lit = args[1].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.neq(lit)))
}

fn gt(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let lit = args[1].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.gt(lit)))
}

fn lt(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let lit = args[1].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.lt(lit)))
}

fn gt_eq(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let lit = args[1].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.gt_eq(lit)))
}

fn lt_eq(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let lit = args[1].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.lt_eq(lit)))
}

fn not(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.not()))
}

fn and(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr1 = args[0].clone().expect_polars_expr()?;
    let expr2 = args[1].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr1.and(expr2)))
}

fn or(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr1 = args[0].clone().expect_polars_expr()?;
    let expr2 = args[1].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr1.or(expr2)))
}

fn sum(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.sum()))
}

fn mean(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.mean()))
}

fn median(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.median()))
}

fn count(args: Vec<Expr>) -> Result<Expr, Error> {
    if args.is_empty() {
        // (count) without arguments returns the total number of rows
        Ok(new_polars_expr(polars::prelude::len()))
    } else {
        // (count $"col") returns non-null count for that column
        let expr = args[0].clone().expect_polars_expr()?;
        Ok(new_polars_expr(expr.count()))
    }
}

fn min(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.min()))
}

fn max(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.max()))
}

fn n_unique(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(expr.n_unique()))
}

fn std_dev(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    // Default to sample standard deviation (ddof = 1); allow overriding via second argument
    let ddof = if args.len() > 1 {
        args[1].clone().expect_int()? as u8
    } else {
        1
    };
    Ok(new_polars_expr(expr.std(ddof)))
}

fn select(args: Vec<Expr>) -> Result<Expr, Error> {
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

fn filter(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let expr = args[1].clone().expect_polars_expr()?;

    // Eagerly validate that all columns in `expr` exist in `lf`
    lf.clone()
        .select(vec![expr.clone()])
        .collect_schema()
        .map_err(Error::PolarsError)?;

    Ok(new_lazy_frame(lf.filter(expr)))
}

fn with_column(args: Vec<Expr>) -> Result<Expr, Error> {
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

fn sort(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let by_expr = args[1].clone().expect_polars_expr()?;
    
    let descending = if args.len() > 2 {
        match args[2].clone().expect_symbol()?.as_str() {
            ":desc" => true,
            _ => false,
        }
    } else {
        false
    };

    let options = SortMultipleOptions::new().with_order_descending(descending);
    Ok(new_lazy_frame(lf.sort_by_exprs(vec![by_expr], options)))
}

fn group_by(args: Vec<Expr>) -> Result<Expr, Error> {
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

fn alias(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let name = args[1].clone().expect_string()?;
    Ok(new_polars_expr(expr.alias(&name)))
}

fn agg(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let mut exprs = Vec::<polars::prelude::Expr>::new();
    
    // Collect all expression arguments passed to agg
    for arg in args.iter().skip(1) {
        exprs.push(arg.clone().expect_polars_expr()?);
    }
    
    // Polars aggregates entire lazy frames via select
    Ok(new_lazy_frame(lf.select(exprs)))
}

fn join(args: Vec<Expr>) -> Result<Expr, Error> {
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

fn is_null_(args: Vec<Expr>) -> Result<Expr, Error> {
    let col = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(is_null(col)))
}

fn is_not_null_(args: Vec<Expr>) -> Result<Expr, Error> {
    let col = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(is_not_null(col)))
}

fn fill_null_(args: Vec<Expr>) -> Result<Expr, Error> {
    let col = args[0].clone().expect_polars_expr()?;
    let value = args[1].clone().expect_polars_expr()?;
    Ok(new_polars_expr(col.fill_null(value)))
}

fn drop_nulls(args: Vec<Expr>) -> Result<Expr, Error> {
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

fn when(args: Vec<Expr>) -> Result<Expr, Error> {
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

// (str-contains $"name" "^A.*")
fn str_contains(args: Vec<Expr>) -> Result<Expr, Error> {
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

// (str-replace $"city" "^Lon" "New-Lon")
fn str_replace(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let pat_expr = args[1].clone().expect_polars_expr()?;
    let rep_expr = args[2].clone().expect_polars_expr()?;

    // false = interpret pattern as regex, not literal
    Ok(new_polars_expr(col_expr.str().replace(pat_expr, rep_expr, false)))
}

// (str-extract $"code" "([A-Z]+)-(\\d+)" 1)
fn str_extract(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let pat_expr = args[1].clone().expect_polars_expr()?;
    let group_idx = args[2].clone().expect_int()? as usize;

    Ok(new_polars_expr(col_expr.str().extract(pat_expr, group_idx)))
}

// (over <expr> <partition_col1> ...) or (over <expr> (<col1> <col2>))
fn over(args: Vec<Expr>) -> Result<Expr, Error> {
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

// (shift $"col" 1) or (shift $"col" -1)
fn shift(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let offset = args[1].clone().expect_int()? as i64;
    Ok(new_polars_expr(expr.shift(lit(offset))))
}

// (cum-sum $"col")
fn cum_sum(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let reverse = if args.len() > 1 {
        matches!(args[1].clone().expect_symbol()?.as_str(), ":desc" | ":reverse")
    } else {
        false
    };
    
    Ok(new_polars_expr(expr.cum_sum(reverse)))
}

// (cum-min $"col")
fn cum_min(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let reverse = if args.len() > 1 {
        matches!(args[1].clone().expect_symbol()?.as_str(), ":desc" | ":reverse")
    } else {
        false
    };
    Ok(new_polars_expr(expr.cum_min(reverse)))
}

// (cum-max $"col")
fn cum_max(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let reverse = if args.len() > 1 {
        matches!(args[1].clone().expect_symbol()?.as_str(), ":desc" | ":reverse")
    } else {
        false
    };
    Ok(new_polars_expr(expr.cum_max(reverse)))
}

// (cum-count $"col")
fn cum_count(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let reverse = if args.len() > 1 {
        matches!(args[1].clone().expect_symbol()?.as_str(), ":desc" | ":reverse")
    } else {
        false
    };
    Ok(new_polars_expr(expr.cum_count(reverse)))
}

// (rank $"col") or (rank $"col" :dense)
fn rank(args: Vec<Expr>) -> Result<Expr, Error> {
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

// (to-date $"col" "%Y-%m-%d") or (to-date $"col") -> ISO-8601 default
fn to_date(args: Vec<Expr>) -> Result<Expr, Error> {
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

// (to-datetime $"col" "%Y-%m-%d %H:%M:%S")
fn to_datetime(args: Vec<Expr>) -> Result<Expr, Error> {
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

// (dt-year $"date_col")
fn dt_year(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(col_expr.dt().year()))
}

// (dt-month $"date_col")
fn dt_month(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(col_expr.dt().month()))
}

// (dt-day $"date_col")
fn dt_day(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(col_expr.dt().day()))
}

// (dt-hour $"datetime_col")
fn dt_hour(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    Ok(new_polars_expr(col_expr.dt().hour()))
}

// (dt-format $"date_col" "%B %d, %Y")
fn dt_format(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let format = args[1].clone().expect_string()?;
    Ok(new_polars_expr(col_expr.dt().to_string(&format)))
}

// Helper to extract a column name string from String, Symbol, or $col Expr
fn extract_col_name(expr: &Expr) -> Result<String, Error> {
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
fn extract_col_names(arg: &Expr) -> Result<Vec<String>, Error> {
    match arg.value.clone() {
        ExprValue::List(list) => list.iter().map(extract_col_name).collect(),
        _ => Ok(vec![extract_col_name(arg)?]),
    }
}

fn pivot_(args: Vec<Expr>) -> Result<Expr, Error> {
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

fn melt_(args: Vec<Expr>) -> Result<Expr, Error> {
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

fn concat_(args: Vec<Expr>) -> Result<Expr, Error> {
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

fn cast_(args: Vec<Expr>) -> Result<Expr, Error> {
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
fn parse_rolling_options(args: &[Expr]) -> Result<(polars::prelude::Expr, RollingOptionsFixedWindow), Error> {
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

// (rolling-mean $"col" 5) or (rolling-mean $"col" 5 1 :center)
fn rolling_mean(args: Vec<Expr>) -> Result<Expr, Error> {
    let (expr, opts) = parse_rolling_options(&args)?;
    Ok(new_polars_expr(expr.rolling_mean(opts)))
}

// (rolling-sum $"col" 5)
fn rolling_sum(args: Vec<Expr>) -> Result<Expr, Error> {
    let (expr, opts) = parse_rolling_options(&args)?;
    Ok(new_polars_expr(expr.rolling_sum(opts)))
}

// (rolling-min $"col" 5)
fn rolling_min(args: Vec<Expr>) -> Result<Expr, Error> {
    let (expr, opts) = parse_rolling_options(&args)?;
    Ok(new_polars_expr(expr.rolling_min(opts)))
}

// (rolling-max $"col" 5)
fn rolling_max(args: Vec<Expr>) -> Result<Expr, Error> {
    let (expr, opts) = parse_rolling_options(&args)?;
    Ok(new_polars_expr(expr.rolling_max(opts)))
}

// (rolling-std $"col" 5)
fn rolling_std(args: Vec<Expr>) -> Result<Expr, Error> {
    let (expr, opts) = parse_rolling_options(&args)?;
    Ok(new_polars_expr(expr.rolling_std(opts)))
}

// (rolling-median $"col" 5)
fn rolling_median(args: Vec<Expr>) -> Result<Expr, Error> {
    let (expr, opts) = parse_rolling_options(&args)?;
    Ok(new_polars_expr(expr.rolling_median(opts)))
}

// (scalar expr) — materialize a 1x1 result into a Lisp scalar
fn scalar(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let df = lf.collect().map_err(Error::PolarsError)?;
    if df.shape() != (1, 1) { return Err(Error::ScalarShapeError { shape: df.shape() }) }
    let row = df.get(0).unwrap();
    match row[0].clone() {
        AnyValue::Null => Ok(nil()),
        AnyValue::Boolean(b) => if b { Ok(new_symbol("true".to_string())) } else { Ok(new_symbol("false".to_string())) },
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

fn collect_(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    lf.collect_with_engine(polars::prelude::Engine::Streaming).unwrap();
    Ok(nil())
}

pub fn insert_native_function(defines: &mut HashMap<String, Expr>, name: &str, body: fn(Vec<Expr>) -> Result<Expr, Error>,
    min_args: u8, max_args: u8) {
        defines.insert(name.to_string(), Expr {value: ExprValue::NativeFunction { body, min_args, max_args }, line: None });
}

pub fn std_env() -> Rc<RefCell<Env>> {
    let mut defines = HashMap::new();

    // Constant symbol
    defines.insert("nil".to_string(), nil());

    // Native Functions: (defines, name, function_pointer, min_args, max_args)
    insert_native_function(&mut defines, "print", print, 0, u8::MAX);
    insert_native_function(&mut defines, "list", list, 0, u8::MAX);

    // Arithmetic
    insert_native_function(&mut defines, "+", plus, 1, u8::MAX);
    insert_native_function(&mut defines, "-", minus, 1, u8::MAX);
    insert_native_function(&mut defines, "*", mult, 1, u8::MAX);
    insert_native_function(&mut defines, "/", divide, 1, u8::MAX);

    // IO
    insert_native_function(&mut defines, "read-csv", read_csv, 1, 1);
    insert_native_function(&mut defines, "write-csv", write_csv, 2, 2);
    insert_native_function(&mut defines, "sink", sink, 2, 2);
    insert_native_function(&mut defines, "read-parquet", read_parquet, 1, 1);
    insert_native_function(&mut defines, "write-parquet", write_parquet, 2, 2);

    // Collect
    insert_native_function(&mut defines, "collect", collect_, 1, 1);

    // Inspection
    insert_native_function(&mut defines, "head", head, 1, 1);
    insert_native_function(&mut defines, "tail", tail, 1, 1);
    insert_native_function(&mut defines, "slice", slice, 3, 3);
    insert_native_function(&mut defines, "schema", schema, 1, 1);
    insert_native_function(&mut defines, "shape", shape, 1, 1);

    // Expressions & Literals
    insert_native_function(&mut defines, "col", col_, 1, 1);
    insert_native_function(&mut defines, "lit", lit_, 1, 1);

    // Comparison & Logical
    insert_native_function(&mut defines, "=", eq, 2, 2);
    insert_native_function(&mut defines, "!=", neq, 2, 2);
    insert_native_function(&mut defines, ">", gt, 2, 2);
    insert_native_function(&mut defines, "<", lt, 2, 2);
    insert_native_function(&mut defines, ">=", gt_eq, 2, 2);
    insert_native_function(&mut defines, "<=", lt_eq, 2, 2);
    insert_native_function(&mut defines, "not", not, 1, 1);
    insert_native_function(&mut defines, "and", and, 2, 2);
    insert_native_function(&mut defines, "or", or, 2, 2);

    // Aggregations
    insert_native_function(&mut defines, "sum", sum, 1, 1);
    insert_native_function(&mut defines, "mean", mean, 1, 1);
    insert_native_function(&mut defines, "median", median, 1, 1);
    insert_native_function(&mut defines, "count", count, 0, 1);
    insert_native_function(&mut defines, "min", min, 1, 1);
    insert_native_function(&mut defines, "max", max, 1, 1);
    insert_native_function(&mut defines, "n-unique", n_unique, 1, 1);
    insert_native_function(&mut defines, "std", std_dev, 1, 2);

    // LazyFrame Operations
    insert_native_function(&mut defines, "select", select, 1, u8::MAX);
    insert_native_function(&mut defines, "filter", filter, 2, 2);
    insert_native_function(&mut defines, "with-column", with_column, 3, 3);
    insert_native_function(&mut defines, "sort", sort, 2, 3);
    insert_native_function(&mut defines, "group-by", group_by, 2, u8::MAX);
    insert_native_function(&mut defines, "alias", alias, 2, 2);
    insert_native_function(&mut defines, "agg", agg, 1, u8::MAX);
    insert_native_function(&mut defines, "join", join, 4, 5);

    // Null Handling & Conditionals
    insert_native_function(&mut defines, "null?", is_null_, 1, 1);
    insert_native_function(&mut defines, "not-null?", is_not_null_, 1, 1);
    insert_native_function(&mut defines, "fill-null", fill_null_, 2, 2);
    insert_native_function(&mut defines, "drop-nulls", drop_nulls, 1, u8::MAX);
    insert_native_function(&mut defines, "when", when, 2, 3);

    // String
    insert_native_function(&mut defines, "str-contains", str_contains, 2, 2);
    insert_native_function(&mut defines, "str-replace", str_replace, 3, 3);
    insert_native_function(&mut defines, "str-extract", str_extract, 3, 3);

    // Window & Cumulative Functions
    insert_native_function(&mut defines, "over", over, 2, u8::MAX);
    insert_native_function(&mut defines, "shift", shift, 2, 2);
    insert_native_function(&mut defines, "cum-sum", cum_sum, 1, 2);
    insert_native_function(&mut defines, "cum-min", cum_min, 1, 2);
    insert_native_function(&mut defines, "cum-max", cum_max, 1, 2);
    insert_native_function(&mut defines, "cum-count", cum_count, 1, 2);
    insert_native_function(&mut defines, "rank", rank, 1, 2);

    // Date and time
    insert_native_function(&mut defines, "to-date", to_date, 1, 2);
    insert_native_function(&mut defines, "to-datetime", to_datetime, 1, 2);
    insert_native_function(&mut defines, "dt-year", dt_year, 1, 1);
    insert_native_function(&mut defines, "dt-month", dt_month, 1, 1);
    insert_native_function(&mut defines, "dt-day", dt_day, 1, 1);
    insert_native_function(&mut defines, "dt-hour", dt_hour, 1, 1);
    insert_native_function(&mut defines, "dt-format", dt_format, 2, 2);

    // Reshaping & Set Operations
    insert_native_function(&mut defines, "pivot", pivot_, 4, 5);
    insert_native_function(&mut defines, "melt", melt_, 3, 5);
    insert_native_function(&mut defines, "unpivot", melt_, 3, 5);
    insert_native_function(&mut defines, "concat", concat_, 1, u8::MAX);

    // Rolling / Window aggregations
    insert_native_function(&mut defines, "rolling-mean", rolling_mean, 2, 4);
    insert_native_function(&mut defines, "rolling-sum", rolling_sum, 2, 4);
    insert_native_function(&mut defines, "rolling-min", rolling_min, 2, 4);
    insert_native_function(&mut defines, "rolling-max", rolling_max, 2, 4);
    insert_native_function(&mut defines, "rolling-median", rolling_median, 2, 4);
    insert_native_function(&mut defines, "rolling-std", rolling_std, 2, 5);

    // Casting
    insert_native_function(&mut defines, "cast", cast_, 2, 2);

    // Plotlars
    insert_native_function(&mut defines, "scatter-plot", plotlarslib::scatter_plot, 1, u8::MAX);
    insert_native_function(&mut defines, "bar-plot", plotlarslib::bar_plot, 1, u8::MAX);
    insert_native_function(&mut defines, "pie-plot", plotlarslib::pie_plot, 1, u8::MAX);
    insert_native_function(&mut defines, "histogram", plotlarslib::histogram, 1, u8::MAX);
    insert_native_function(&mut defines, "box-plot", plotlarslib::box_plot, 1, u8::MAX);
    insert_native_function(&mut defines, "time-series-plot", plotlarslib::time_series_plot, 1, u8::MAX);
    insert_native_function(&mut defines, "line-plot", plotlarslib::line_plot, 1, u8::MAX);

    // Polars to lisp
    insert_native_function(&mut defines, "scalar", scalar, 1, 1);

    Rc::new(RefCell::new(Env {
        defines,
        outer: None,
    }))
}
