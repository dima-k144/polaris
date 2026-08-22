use plotlars::{Plot, ScatterPlot, BarPlot, PieChart, Histogram, BoxPlot, TimeSeriesPlot, LinePlot};
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

pub fn box_plot(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args.first()
        .ok_or_else(|| Error::MissingRequiredKWArg { name: "dataframe".to_string() })?
        .clone()
        .expect_lazy_frame()?;

    let df = lf.collect().map_err(Error::PolarsError)?;

    // Required arguments (accepts :labels / :x, and :values / :y)
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
    let opacity = kw(&args, ":opacity").map(|k| k.expect_float()).transpose()?;
    let facet = kw(&args, ":facet").map(|k| k.expect_string()).transpose()?;
    let jitter = kw(&args, ":jitter").map(|k| k.expect_float()).transpose()?;
    let point_offset = kw(&args, ":point-offset").map(|k| k.expect_float()).transpose()?;
    let box_points = if has_flag(&args, ":box-points") {
        Some(true)
    } else {
        None
    };

    let plot = BoxPlot::builder()
        .data(&df)
        .labels(labels.as_str())
        .values(values.as_str())
        .maybe_plot_title(title.as_deref())
        .maybe_x_title(x_title.as_deref())
        .maybe_y_title(y_title.as_deref())
        .maybe_group(group.as_deref())
        .maybe_opacity(opacity)
        .maybe_facet(facet.as_deref())
        .maybe_box_points(box_points)
        .maybe_point_offset(point_offset)
        .maybe_jitter(jitter)
        .build();

    plot.plot();

    Ok(stdlib::nil())
}

pub fn time_series_plot(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args.first()
        .ok_or_else(|| Error::MissingRequiredKWArg { name: "dataframe".to_string() })?
        .clone()
        .expect_lazy_frame()?;

    let mut df = lf.collect().map_err(Error::PolarsError)?;

    // Required arguments
    let x = require_kw(&args, ":x")?.expect_string()?;
    let y = require_kw(&args, ":y")?.expect_string()?;

    // Cast Date / Datetime to ISO-8601 String for Plotly rendering
    if let Ok(col) = df.column(&x) {
        if matches!(col.dtype(), DataType::Date | DataType::Datetime(_, _)) {
            let str_col = col.cast(&DataType::String).map_err(Error::PolarsError)?;
            df.with_column(str_col).map_err(Error::PolarsError)?;
        }
    }

    // Optional arguments
    let title = kw(&args, ":title").map(|k| k.expect_string()).transpose()?;
    let x_title = kw(&args, ":x-title").map(|k| k.expect_string()).transpose()?;
    let y_title = kw(&args, ":y-title").map(|k| k.expect_string()).transpose()?;
    let y2_title = kw(&args, ":y2-title").map(|k| k.expect_string()).transpose()?;
    let facet = kw(&args, ":facet").map(|k| k.expect_string()).transpose()?;
    let width = kw(&args, ":width").map(|k| k.expect_float()).transpose()?;
    let size = kw(&args, ":size")
        .map(|k| {
            let val = k.expect_int()?;
            usize::try_from(val).map_err(|_| Error::Custom(format!("`:size` must be a positive integer, got {}", val)))
        })
        .transpose()?;

    // Optional additional series (expects a list of strings or single string)
    let additional_series = match kw(&args, ":additional-series") {
        Some(Expr::List(list)) => {
            let mut series_vec = Vec::new();
            for item in list {
                series_vec.push(item.expect_string()?);
            }
            Some(series_vec)
        }
        Some(Expr::String(s)) => Some(vec![s]),
        Some(other) => return Err(Error::WrongType { expected: "list or string".into(), got: other }),
        None => None,
    };

    let additional_series_slices: Option<Vec<&str>> = additional_series
        .as_ref()
        .map(|v| v.iter().map(|s| s.as_str()).collect());

    let plot = TimeSeriesPlot::builder()
        .data(&df)
        .x(x.as_str())
        .y(y.as_str())
        .maybe_additional_series(additional_series_slices)
        .maybe_plot_title(title.as_deref())
        .maybe_x_title(x_title.as_deref())
        .maybe_y_title(y_title.as_deref())
        .maybe_y2_title(y2_title.as_deref())
        .maybe_facet(facet.as_deref())
        .maybe_width(width)
        .maybe_size(size)
        .build();

    plot.plot();

    Ok(stdlib::nil())
}

pub fn line_plot(args: Vec<Expr>) -> Result<Expr, Error> {
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
    let y2_title = kw(&args, ":y2-title").map(|k| k.expect_string()).transpose()?;
    let facet = kw(&args, ":facet").map(|k| k.expect_string()).transpose()?;
    let width = kw(&args, ":width").map(|k| k.expect_float()).transpose()?;
    let size = kw(&args, ":size")
        .map(|k| {
            let val = k.expect_int()?;
            usize::try_from(val).map_err(|_| Error::Custom(format!("`:size` must be a positive integer, got {}", val)))
        })
        .transpose()?;

    // Optional additional lines (expects a list of strings or a single string)
    let additional_lines = match kw(&args, ":additional-lines") {
        Some(Expr::List(list)) => {
            let mut lines_vec = Vec::new();
            for item in list {
                lines_vec.push(item.expect_string()?);
            }
            Some(lines_vec)
        }
        Some(Expr::String(s)) => Some(vec![s]),
        Some(other) => return Err(Error::WrongType { expected: "list or string".into(), got: other }),
        None => None,
    };

    let additional_lines_slices: Option<Vec<&str>> = additional_lines
        .as_ref()
        .map(|v| v.iter().map(|s| s.as_str()).collect());

    let plot = LinePlot::builder()
        .data(&df)
        .x(x.as_str())
        .y(y.as_str())
        .maybe_additional_lines(additional_lines_slices)
        .maybe_plot_title(title.as_deref())
        .maybe_x_title(x_title.as_deref())
        .maybe_y_title(y_title.as_deref())
        .maybe_y2_title(y2_title.as_deref())
        .maybe_facet(facet.as_deref())
        .maybe_width(width)
        .maybe_size(size)
        .build();

    plot.plot();

    Ok(stdlib::nil())
}
