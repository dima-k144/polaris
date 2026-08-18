use std::collections::HashMap;
use polars::prelude::*;
use crate::types::{Env, Expr, Error};

fn nil() -> Expr { Expr::Symbol(String::from("nil")) }

fn print(args: Vec<Expr>) -> Result<Expr, Error> {
    for arg in args {
        println!("{}", arg);
    }
    Ok(nil())
}

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
    match args[0] {
        Expr::Number(num) => Ok(Expr::PolarsExpr(lit(num))),
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

pub fn std_env() -> Env {
    let mut defines = HashMap::new();

    defines.insert("nil".to_string(), Expr::Symbol(String::from("nil")));

    defines.insert("print".to_string(), Expr::NativeFunction(print));

    defines.insert("+".to_string(), Expr::NativeFunction(plus));

    defines.insert("read-csv".to_string(), Expr::NativeFunction(read_csv));
    defines.insert("head".to_string(), Expr::NativeFunction(head));
    defines.insert("tail".to_string(), Expr::NativeFunction(tail));
    defines.insert("slice".to_string(), Expr::NativeFunction(slice));
    defines.insert("schema".to_string(), Expr::NativeFunction(schema));
    defines.insert("shape".to_string(), Expr::NativeFunction(shape));
    defines.insert("col".to_string(), Expr::NativeFunction(col_));
    defines.insert("lit".to_string(), Expr::NativeFunction(lit_));
    defines.insert("=".to_string(), Expr::NativeFunction(eq));
    defines.insert("!=".to_string(), Expr::NativeFunction(neq));
    defines.insert(">".to_string(), Expr::NativeFunction(gt));
    defines.insert("<".to_string(), Expr::NativeFunction(lt));
    defines.insert(">=".to_string(), Expr::NativeFunction(gt_eq));
    defines.insert("<=".to_string(), Expr::NativeFunction(lt_eq));
    defines.insert("select".to_string(), Expr::NativeFunction(select));
    defines.insert("filter".to_string(), Expr::NativeFunction(filter));

    Env {
        defines: defines,
        outer: None,
    }
}
