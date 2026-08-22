use plotlars::{Plot, ScatterPlot, BarPlot, PieChart, Histogram};
use crate::types::{Expr, Error};
use crate::stdlib;
use polars::prelude::*;

fn kw(args: &[Expr], name: &str) -> Option<Expr> {
    for (i, arg) in args.iter().enumerate() {
        if let Expr::Symbol(symbol) = arg {
            if symbol == name {
                return args.get(i + 1).cloned();
            }
        }
    }
    None
}

fn require_kw(args: &[Expr], name: &str) -> Result<Expr, Error> {
    match kw(args, name) {
        Some(expr) => Ok(expr),
        None => Err(Error::MissingRequiredKWArg { name: name.to_string() }),
    }
}

fn has_flag(args: &[Expr], name: &str) -> bool {
    args.iter().any(|arg| matches!(arg, Expr::Symbol(s) if s == name))
}

pub fn scatter_plot(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args.first()
        .ok_or_else(|| Error::MissingRequiredKWArg { name: "dataframe".to_string() })?
        .clone()
        .expect_lazy_frame()?;

    let df = lf.collect().map_err(Error::PolarsError)?;

    // Required arguments
    let x = require_kw(&args, ":x")?.expect_string()?;
    let y = require_kw(&args, ":y")?.expect_string()?;

    // Optional arguments
    let title = kw(&args, ":title").map(|k| k.expect_string()).transpose()?;
    let x_title = kw(&args, ":x-title").map(|k| k.expect_string()).transpose()?;
    let y_title = kw(&args, ":y-title").map(|k| k.expect_string()).transpose()?;
    let group = kw(&args, ":group").map(|k| k.expect_string()).transpose()?;
    let opacity = kw(&args, ":opacity").map(|k| k.expect_float()).transpose()?;
    let size = kw(&args, ":size")
    .map(|k| {
        let val = k.expect_int()?;
        usize::try_from(val).map_err(|_| Error::Custom(format!("`:size` must be a positive integer, got {}", val)))
    })
    .transpose()?;

    let plot = ScatterPlot::builder()
        .data(&df)
        .x(x.as_str())
        .y(y.as_str())
        .maybe_plot_title(title.as_deref())
        .maybe_x_title(x_title.as_deref())
        .maybe_y_title(y_title.as_deref())
        .maybe_group(group.as_deref())
        .maybe_size(size)
        .maybe_opacity(opacity)
        .build();

    plot.plot();

    Ok(stdlib::nil())
}

pub fn bar_plot(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args.first()
        .ok_or_else(|| Error::MissingRequiredKWArg { name: "dataframe".to_string() })?
        .clone()
        .expect_lazy_frame()?;

    let df = lf.collect().map_err(Error::PolarsError)?;

    // Required arguments (accepts :labels or alias :x, and :values or alias :y)
    let labels = kw(&args, ":labels")
        .or_else(|| kw(&args, ":x"))
        .ok_or_else(|| Error::MissingRequiredKWArg { name: ":labels".to_string() })?
        .expect_string()?;

    let values = kw(&args, ":values")
        .or_else(|| kw(&args, ":y"))
        .ok_or_else(|| Error::MissingRequiredKWArg { name: ":values".to_string() })?
        .expect_string()?;

    // Optional arguments
    let title = kw(&args, ":title").map(|k| k.expect_string()).transpose()?;
    let x_title = kw(&args, ":x-title").map(|k| k.expect_string()).transpose()?;
    let y_title = kw(&args, ":y-title").map(|k| k.expect_string()).transpose()?;
    let group = kw(&args, ":group").map(|k| k.expect_string()).transpose()?;
    let error = kw(&args, ":error").map(|k| k.expect_string()).transpose()?;

    let plot = BarPlot::builder()
        .data(&df)
        .labels(labels.as_str())
        .values(values.as_str())
        .maybe_plot_title(title.as_deref())
        .maybe_x_title(x_title.as_deref())
        .maybe_y_title(y_title.as_deref())
        .maybe_group(group.as_deref())
        .maybe_error(error.as_deref())
        .build();

    plot.plot();

    Ok(stdlib::nil())
}

pub fn pie_plot(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args.first()
        .ok_or_else(|| Error::MissingRequiredKWArg { name: "dataframe".to_string() })?
        .clone()
        .expect_lazy_frame()?;

    let df = lf.collect().map_err(Error::PolarsError)?;

    // Required arguments
    let labels = require_kw(&args, ":labels")?.expect_string()?;

    // Optional arguments
    let title = kw(&args, ":title").map(|k| k.expect_string()).transpose()?;
    let hole = kw(&args, ":hole").map(|k| k.expect_float()).transpose()?;
    let pull = kw(&args, ":pull").map(|k| k.expect_float()).transpose()?;
    let rotation = kw(&args, ":rotation").map(|k| k.expect_float()).transpose()?;
    let facet = kw(&args, ":facet").map(|k| k.expect_string()).transpose()?;

    let plot = PieChart::builder()
        .data(&df)
        .labels(labels.as_str())
        .maybe_plot_title(title.as_deref())
        .maybe_hole(hole)
        .maybe_pull(pull)
        .maybe_rotation(rotation)
        .maybe_facet(facet.as_deref())
        .build();

    plot.plot();

    Ok(stdlib::nil())
}

pub fn histogram(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args.first()
        .ok_or_else(|| Error::MissingRequiredKWArg { name: "dataframe".to_string() })?
        .clone()
        .expect_lazy_frame()?;

    let df = lf.collect().map_err(Error::PolarsError)?;

    // Required arguments
    let x = require_kw(&args, ":x")?.expect_string()?;

    // Optional arguments
    let title = kw(&args, ":title").map(|k| k.expect_string()).transpose()?;
    let x_title = kw(&args, ":x-title").map(|k| k.expect_string()).transpose()?;
    let y_title = kw(&args, ":y-title").map(|k| k.expect_string()).transpose()?;
    let group = kw(&args, ":group").map(|k| k.expect_string()).transpose()?;
    let opacity = kw(&args, ":opacity").map(|k| k.expect_float()).transpose()?;
    let facet = kw(&args, ":facet").map(|k| k.expect_string()).transpose()?;

    let plot = Histogram::builder()
        .data(&df)
        .x(x.as_str())
        .maybe_plot_title(title.as_deref())
        .maybe_x_title(x_title.as_deref())
        .maybe_y_title(y_title.as_deref())
        .maybe_group(group.as_deref())
        .maybe_opacity(opacity)
        .maybe_facet(facet.as_deref())
        .build();

    plot.plot();

    Ok(stdlib::nil())
}
