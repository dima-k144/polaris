use std::collections::HashMap;
use polars::prelude::*;
use crate::types::{Env, Expr, Error};

fn plus(args: Vec<Expr>) -> Result<Expr, Error> {
    let mut sum = 0.0;
    for arg in args {
        sum += arg.expect_number()?;
    }
    Ok(Expr::Number(sum))
}

fn read_csv(args: Vec<Expr>) -> Result<Expr, Error> {
    let path = args[0].clone().expect_string()?;
    let lf = LazyCsvReader::new(path.as_str().into())
        .with_has_header(true)
        .finish().unwrap();
    Ok(Expr::LazyFrame(lf))
}

fn col_(args: Vec<Expr>) -> Result<Expr, Error> {
    Ok(Expr::PolarsExpr(col(args[0].clone().expect_string()?)))
}

fn select(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let mut exprs = Vec::<polars::prelude::Expr>::new();
    for arg in args.iter().skip(1) {
        exprs.push(arg.clone().expect_polars_expr()?);
    }
    Ok(Expr::LazyFrame(lf.select(exprs)))
}

pub fn std_env() -> Env {
    let mut defines = HashMap::new();

    defines.insert("nil".to_string(), Expr::Symbol(String::from("nil")));

    defines.insert("+".to_string(), Expr::NativeFunction(plus));

    defines.insert("read-csv".to_string(), Expr::NativeFunction(read_csv));
    defines.insert("col".to_string(), Expr::NativeFunction(col_));
    defines.insert("select".to_string(), Expr::NativeFunction(select));

    Env {
        defines: defines,
        outer: None,
    }
}
