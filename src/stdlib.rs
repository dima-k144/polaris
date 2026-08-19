use std::collections::HashMap;
use std::fs::File;
use polars::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use crate::types::{Env, Error, Expr::{self, PolarsExpr}};

fn nil() -> Expr { Expr::Symbol(String::from("nil")) }

fn print(args: Vec<Expr>) -> Result<Expr, Error> {
    for arg in args {
        println!("{}", arg);
    }
    Ok(nil())
}

fn plus(args: Vec<Expr>) -> Result<Expr, Error> {
    if args.iter().any(|arg| matches!(arg, Expr::PolarsExpr(_))) {
        let mut expr = args[0].clone().expect_polars_expr()?;
        for arg in args.iter().skip(1) {
            expr = expr + arg.clone().expect_polars_expr()?;
        }
        Ok(Expr::PolarsExpr(expr))
    } else {
        let mut sum = 0.0;
        for arg in args {
            sum += arg.expect_number()?;
        }
        Ok(Expr::Number(sum))
    }
}

fn minus(args: Vec<Expr>) -> Result<Expr, Error> {
    if args.iter().any(|arg| matches!(arg, Expr::PolarsExpr(_))) {
        let mut expr = args[0].clone().expect_polars_expr()?;
        for arg in args.iter().skip(1) {
            expr = expr - arg.clone().expect_polars_expr()?;
        }
        Ok(Expr::PolarsExpr(expr))
    } else {
        let mut value = args[0].clone().expect_number()?;
        for arg in args.iter().skip(1) {
            value -= arg.clone().expect_number()?;
        }
        Ok(Expr::Number(value))
    }
}

fn mult(args: Vec<Expr>) -> Result<Expr, Error> {
    if args.iter().any(|arg| matches!(arg, Expr::PolarsExpr(_))) {
        let mut expr = args[0].clone().expect_polars_expr()?;
        for arg in args.iter().skip(1) {
            expr = expr * arg.clone().expect_polars_expr()?;
        }
        Ok(Expr::PolarsExpr(expr))
    } else {
        let mut value = args[0].clone().expect_number()?;
        for arg in args.iter().skip(1) {
            value *= arg.clone().expect_number()?;
        }
        Ok(Expr::Number(value))
    }
}

fn divide(args: Vec<Expr>) -> Result<Expr, Error> {
    if args.iter().any(|arg| matches!(arg, Expr::PolarsExpr(_))) {
        let mut expr = args[0].clone().expect_polars_expr()?;
        for arg in args.iter().skip(1) {
            expr = expr / arg.clone().expect_polars_expr()?;
        }
        Ok(Expr::PolarsExpr(expr))
    } else {
        let mut value = args[0].clone().expect_number()?;
        for arg in args.iter().skip(1) {
            value /= arg.clone().expect_number()?;
        }
        Ok(Expr::Number(value))
    }
}

fn read_csv(args: Vec<Expr>) -> Result<Expr, Error> {
    let path = args[0].clone().expect_string()?;
    match LazyCsvReader::new(path.as_str().into())
        .with_has_header(true)
        .finish() {
            Ok(lf) => Ok(Expr::LazyFrame(lf)),
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

fn head(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    Ok(Expr::LazyFrame(lf.limit(10)))
}

fn tail(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    Ok(Expr::LazyFrame(lf.slice(-10, 10)))
}

fn slice(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let offset = args[1].clone().expect_number()?;
    let len = args[2].clone().expect_number()?;
    Ok(Expr::LazyFrame(lf.slice(offset as i64, len as u32)))
}

fn schema(args: Vec<Expr>) -> Result<Expr, Error> {
    let mut lf = args[0].clone().expect_lazy_frame()?; 
    let schema = lf.collect_schema().unwrap();
    let mut result = Vec::<Expr>::new();
    let mut fields = Vec::<Expr>::new();
    for field in schema.iter_fields() {
        
        fields.push(Expr::List(vec![Expr::String(field.name.into_string()), Expr::String(field.dtype.to_string())]));
    }
    result.push(Expr::List(fields));
    Ok(Expr::List(result))
}

fn shape(args: Vec<Expr>) -> Result<Expr, Error> {
    let mut lf = args[0].clone().expect_lazy_frame()?;  
    let shape = lf.collect().unwrap().shape();
    Ok(Expr::List(vec![Expr::Number(shape.0 as f64), Expr::Number(shape.1 as f64)]))
}

fn col_(args: Vec<Expr>) -> Result<Expr, Error> {
    Ok(Expr::PolarsExpr(col(args[0].clone().expect_string()?)))
}

fn lit_(args: Vec<Expr>) -> Result<Expr, Error> {
    match args[0].clone() {
        Expr::Number(num) => Ok(Expr::PolarsExpr(lit(num))),
        Expr::String(string) => Ok(Expr::PolarsExpr(lit(string))),
        _ => Err(Error::WrongType { expected: String::from("number"), got: args[0].clone() })
    }
}

fn eq(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let lit = args[1].clone().expect_polars_expr()?;
    Ok(Expr::PolarsExpr(expr.eq(lit)))
}

fn neq(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let lit = args[1].clone().expect_polars_expr()?;
    Ok(Expr::PolarsExpr(expr.neq(lit)))
}

fn gt(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let lit = args[1].clone().expect_polars_expr()?;
    Ok(Expr::PolarsExpr(expr.gt(lit)))
}

fn lt(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let lit = args[1].clone().expect_polars_expr()?;
    Ok(Expr::PolarsExpr(expr.lt(lit)))
}

fn gt_eq(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let lit = args[1].clone().expect_polars_expr()?;
    Ok(Expr::PolarsExpr(expr.gt_eq(lit)))
}

fn lt_eq(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let lit = args[1].clone().expect_polars_expr()?;
    Ok(Expr::PolarsExpr(expr.lt_eq(lit)))
}

fn not(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    Ok(Expr::PolarsExpr(expr.not()))
}

fn and(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr1 = args[0].clone().expect_polars_expr()?;
    let expr2 = args[1].clone().expect_polars_expr()?;
    Ok(Expr::PolarsExpr(expr1.and(expr2)))
}

fn or(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr1 = args[0].clone().expect_polars_expr()?;
    let expr2 = args[1].clone().expect_polars_expr()?;
    Ok(Expr::PolarsExpr(expr1.or(expr2)))
}

fn sum(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    Ok(Expr::PolarsExpr(expr.sum()))
}

fn mean(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    Ok(Expr::PolarsExpr(expr.mean()))
}

fn median(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    Ok(Expr::PolarsExpr(expr.median()))
}

fn count(args: Vec<Expr>) -> Result<Expr, Error> {
    if args.is_empty() {
        // (count) without arguments returns the total number of rows
        Ok(Expr::PolarsExpr(polars::prelude::len()))
    } else {
        // (count $"col") returns non-null count for that column
        let expr = args[0].clone().expect_polars_expr()?;
        Ok(Expr::PolarsExpr(expr.count()))
    }
}

fn min(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    Ok(Expr::PolarsExpr(expr.min()))
}

fn max(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    Ok(Expr::PolarsExpr(expr.max()))
}

fn n_unique(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    Ok(Expr::PolarsExpr(expr.n_unique()))
}

fn std_dev(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    // Default to sample standard deviation (ddof = 1); allow overriding via second argument
    let ddof = if args.len() > 1 {
        args[1].clone().expect_number()? as u8
    } else {
        1
    };
    Ok(Expr::PolarsExpr(expr.std(ddof)))
}

fn select(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let mut exprs = Vec::<polars::prelude::Expr>::new();
    for arg in args.iter().skip(1) {
        exprs.push(arg.clone().expect_polars_expr()?);
    }
    Ok(Expr::LazyFrame(lf.select(exprs)))
}

fn filter(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let expr = args[1].clone().expect_polars_expr()?;
    Ok(Expr::LazyFrame(lf.filter(expr)))
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
    Ok(Expr::LazyFrame(lf.with_column(value.alias(&alias))))
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
    Ok(Expr::LazyFrame(lf.sort_by_exprs(vec![by_expr], options)))
}

fn group_by(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;

    // Support either a single grouping column or a list of grouping columns
    let group_keys = match &args[1] {
        Expr::List(cols) => {
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

    Ok(Expr::LazyFrame(lf.group_by(group_keys).agg(agg_exprs)))
}

fn alias(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let name = args[1].clone().expect_string()?;
    Ok(Expr::PolarsExpr(expr.alias(&name)))
}

fn agg(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let mut exprs = Vec::<polars::prelude::Expr>::new();
    
    // Collect all expression arguments passed to agg
    for arg in args.iter().skip(1) {
        exprs.push(arg.clone().expect_polars_expr()?);
    }
    
    // Polars aggregates entire lazy frames via select
    Ok(Expr::LazyFrame(lf.select(exprs)))
}

fn join(args: Vec<Expr>) -> Result<Expr, Error> {
    let left_lf = args[0].clone().expect_lazy_frame()?;
    let right_lf = args[1].clone().expect_lazy_frame()?;
    
    // Parse left key(s)
    let left_on = match &args[2] {
        Expr::List(cols) => {
            let mut exprs = Vec::new();
            for c in cols {
                exprs.push(c.clone().expect_polars_expr()?);
            }
            exprs
        }
        _ => vec![args[2].clone().expect_polars_expr()?],
    };

    // Parse right key(s)
    let right_on = match &args[3] {
        Expr::List(cols) => {
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

    Ok(Expr::LazyFrame(joined))
}

fn is_null_(args: Vec<Expr>) -> Result<Expr, Error> {
    let col = args[0].clone().expect_polars_expr()?;
    Ok(Expr::PolarsExpr(is_null(col)))
}

fn is_not_null_(args: Vec<Expr>) -> Result<Expr, Error> {
    let col = args[0].clone().expect_polars_expr()?;
    Ok(Expr::PolarsExpr(is_not_null(col)))
}

fn fill_null_(args: Vec<Expr>) -> Result<Expr, Error> {
    let col = args[0].clone().expect_polars_expr()?;
    let value = args[1].clone().expect_polars_expr()?;
    Ok(Expr::PolarsExpr(col.fill_null(value)))
}

fn drop_nulls(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    if args.len() == 1 {
        Ok(Expr::LazyFrame(lf.drop_nulls(None)))
    } else {
        let mut col_names = Vec::<String>::new();

        let mut extract_name = |expr: &Expr| -> Result<(), Error> {
            match expr {
                Expr::String(s) | Expr::Symbol(s) => {
                    col_names.push(s.clone());
                    Ok(())
                }
                Expr::PolarsExpr(polars::prelude::Expr::Column(name)) => {
                    col_names.push(name.to_string());
                    Ok(())
                }
                other => Err(Error::WrongType {
                    expected: String::from("column name or $col"),
                    got: other.clone(),
                }),
            }
        };

        for arg in args.iter().skip(1) {
            match arg {
                Expr::List(list) => {
                    for item in list {
                        extract_name(item)?;
                    }
                }
                _ => extract_name(arg)?,
            }
        }

        Ok(Expr::LazyFrame(lf.drop_nulls(Some(cols(col_names).into()))))
    }
}

fn when(args: Vec<Expr>) -> Result<Expr, Error> {
    let condition = args[0].clone().expect_polars_expr()?;
    let on_true = args[1].clone().expect_polars_expr()?;
    match args.len() {
        2 => Ok(PolarsExpr(polars::prelude::when(condition).then(on_true).otherwise(polars::prelude::lit(polars::prelude::NULL)))),
        _ => {
            let on_false = args[2].clone().expect_polars_expr()?;
            Ok(PolarsExpr(polars::prelude::when(condition).then(on_true).otherwise(on_false)))
        }
    }
}

pub fn insert_native_function(defines: &mut HashMap<String, Expr>, name: &str, body: fn(Vec<Expr>) -> Result<Expr, Error>,
    min_args: u8, max_args: u8) {
        defines.insert(name.to_string(), Expr::NativeFunction { body, min_args, max_args });
    }

pub fn std_env() -> Rc<RefCell<Env>> {
    let mut defines = HashMap::new();

    // Constant symbol
    defines.insert("nil".to_string(), nil());

    // Native Functions: (defines, name, function_pointer, min_args, max_args)
    insert_native_function(&mut defines, "print", print, 0, u8::MAX);

    // Arithmetic
    insert_native_function(&mut defines, "+", plus, 1, u8::MAX);
    insert_native_function(&mut defines, "-", minus, 1, u8::MAX);
    insert_native_function(&mut defines, "*", mult, 1, u8::MAX);
    insert_native_function(&mut defines, "/", divide, 1, u8::MAX);

    // IO & Inspection
    insert_native_function(&mut defines, "read-csv", read_csv, 1, 1);
    insert_native_function(&mut defines, "write-csv", write_csv, 2, 2);
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

    Rc::new(RefCell::new(Env {
        defines,
        outer: None,
    }))
}
