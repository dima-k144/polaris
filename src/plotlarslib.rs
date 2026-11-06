use crate::corelib::*;
use crate::polarslib::new_lazy_frame;
use crate::types::{Error, Expr, ExprValue, Env};
use plotlars::{
    BarPlot, BoxPlot, Histogram, LinePlot, PieChart, Plot, ScatterPlot, Text, TimeSeriesPlot,
};
use polars::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

fn kw(args: &[Expr], name: &str) -> Option<Expr> {
    for (i, arg) in args.iter().enumerate() {
        if let ExprValue::Symbol(symbol) = arg.value.clone()
            && symbol == name
        {
            return args.get(i + 1).cloned();
        }
    }
    None
}

fn require_kw(args: &[Expr], name: &str) -> Result<Expr, Error> {
    match kw(args, name) {
        Some(expr) => Ok(expr),
        None => Err(Error::MissingRequiredKWArg {
            name: name.to_string(),
        }),
    }
}

fn has_flag(args: &[Expr], name: &str) -> bool {
    args.iter()
        .any(|arg| matches!(arg.value.clone(), ExprValue::Symbol(s) if s == name))
}

// Shared by every plot function. Sizes apply to titles and, by default, tick labels.
// Usage: :font-size 24 (defaults to 20).
fn plot_font_size(args: &[Expr]) -> Result<usize, Error> {
    match kw(args, ":font-size") {
        Some(value) => {
            let n = value.expect_int()?;
            usize::try_from(n)
                .ok()
                .filter(|&n| n > 0)
                .ok_or_else(|| {
                    Error::Custom(format!(
                        "`:font-size` must be a positive integer, got {}", n
                    ))
                })
        }
        None => Ok(20),
    }
}

// Plotlars exposes the tick font family but not its size. Set the actual
// Plotly layout before rendering so category/numeric labels and image exports
// use the requested size. Tick labels inherit :font-size unless overridden.
fn show_plot(plot: &impl Plot, args: &[Expr]) -> Result<(), Error> {
    let font_size = plot_font_size(args)?;
    let size = match kw(args, ":tick-font-size") {
        Some(value) => {
            let n = value.expect_int()?;
            usize::try_from(n)
                .ok()
                .filter(|&n| n > 0)
                .ok_or_else(|| {
                    Error::Custom(format!(
                        "`:tick-font-size` must be a positive integer, got {}", n
                    ))
                })?
        }
        None => plot_font_size(args)?,
    };

    let html = plot.to_html();
    // Replace the rendering call, rather than applying CSS after rendering:
    // Plotly then knows the text dimensions and includes them in exports.
    let render = format!(
        r#"(function(id, data, layout, config) {{
            // plotly.rs uses newPlot(id, {{data, layout, config}}).
            // Also support the separate data/layout argument form.
            var figure = data && !Array.isArray(data) && typeof data === "object"
                ? data : null;
            layout = figure ? (figure.layout || {{}}) : (layout || {{}});
            function resizeTitle(title) {{
                if (typeof title === "string") title = {{text: title}};
                if (title && typeof title === "object") {{
                    title.font = Object.assign({{}}, title.font, {{size: {font_size}}});
                }}
                return title;
            }}
            layout.font = Object.assign({{}}, layout.font, {{size: {font_size}}});
            if (layout.title != null) layout.title = resizeTitle(layout.title);
            if (layout.legend) {{
                layout.legend.font = Object.assign({{}}, layout.legend.font, {{size: {font_size}}});
                if (layout.legend.title != null) {{
                    layout.legend.title = resizeTitle(layout.legend.title);
                }}
            }}
            (layout.annotations || []).forEach(function(annotation) {{
                annotation.font = Object.assign({{}}, annotation.font, {{size: {font_size}}});
            }});
            var axes = Object.keys(layout).filter(function(key) {{
                return /^[xy]axis\d*$/.test(key);
            }});
            ["xaxis", "yaxis"].forEach(function(key) {{
                if (axes.indexOf(key) === -1) axes.push(key);
            }});
            axes.forEach(function(key) {{
                var axis = layout[key] || (layout[key] = {{}});
                if (axis.title != null) axis.title = resizeTitle(axis.title);
                axis.tickfont = Object.assign({{}}, axis.tickfont, {{size: {size}}});
                axis.automargin = true;
            }});
            if (figure) {{
                figure.layout = layout;
                return Plotly.newPlot(id, figure);
            }}
            return Plotly.newPlot(id, data, layout, config);
        }})("#
    );
    if !html.contains("Plotly.newPlot(") {
        return Err(Error::Custom(
            "Cannot apply :tick-font-size: unsupported Plotlars HTML output".into(),
        ));
    }
    // The final occurrence is the chart initialization, after bundled Plotly JS.
    let (before, after) = html.rsplit_once("Plotly.newPlot(").unwrap();
    let html = format!("{before}{render}{after}");

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| Error::Custom(format!("Cannot name chart file: {e}")))?
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "polaris-plot-{}-{timestamp}.html", std::process::id()
    ));
    std::fs::write(&path, html)
        .map_err(|e| Error::Custom(format!("Cannot write chart HTML: {e}")))?;

    // Keep the file available while the browser reads/reloads it.
    #[cfg(target_os = "windows")]
    let status = std::process::Command::new("cmd")
        .args(["/C", "start", ""])
        .arg(&path)
        .status();
    #[cfg(target_os = "macos")]
    let status = std::process::Command::new("open").arg(&path).status();
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let status = std::process::Command::new("xdg-open").arg(&path).status();

    match status {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(Error::Custom(format!(
            "Cannot open chart (exit {status}); HTML is saved at {}", path.display()
        ))),
        Err(e) => Err(Error::Custom(format!(
            "Cannot open chart: {e}; HTML is saved at {}", path.display()
        ))),
    }
}

pub fn scatter_plot(args: Vec<Expr>, env: &Rc<RefCell<Env>>) -> Result<Expr, Error> {
    let lf = args
        .first()
        .ok_or_else(|| Error::MissingRequiredKWArg {
            name: "dataframe".to_string(),
        })?
        .clone()
        .expect_lazy_frame()?;

    let df = lf.clone().collect().map_err(Error::PolarsError)?;

    // Required arguments
    let x = require_kw(&args, ":x")?.expect_string()?;
    let y = require_kw(&args, ":y")?.expect_string()?;

    // Optional arguments
    let font_size = plot_font_size(&args)?;
    let title = kw(&args, ":title").map(|k| k.expect_string()).transpose()?;
    let x_title = kw(&args, ":x-title")
        .map(|k| k.expect_string())
        .transpose()?;
    let y_title = kw(&args, ":y-title")
        .map(|k| k.expect_string())
        .transpose()?;
    let group = kw(&args, ":group").map(|k| k.expect_string()).transpose()?;
    let opacity = kw(&args, ":opacity")
        .map(|k| k.expect_float())
        .transpose()?;
    let size = kw(&args, ":size")
        .map(|k| {
            let val = k.expect_int()?;
            usize::try_from(val).map_err(|_| {
                Error::Custom(format!("`:size` must be a positive integer, got {}", val))
            })
        })
        .transpose()?;

    let plot = ScatterPlot::builder()
        .data(&df)
        .x(x.as_str())
        .y(y.as_str())
        .maybe_plot_title(title.as_deref().map(|s| Text::from(s).size(font_size)))
        .maybe_x_title(x_title.as_deref().map(|s| Text::from(s).size(font_size)))
        .maybe_y_title(y_title.as_deref().map(|s| Text::from(s).size(font_size)))
        .maybe_group(group.as_deref())
        .maybe_size(size)
        .maybe_opacity(opacity)
        .build();

    show_plot(&plot, &args)?;

    Ok(new_lazy_frame(lf))
}

pub fn bar_plot(args: Vec<Expr>, env: &Rc<RefCell<Env>>) -> Result<Expr, Error> {
    let lf = args
        .first()
        .ok_or_else(|| Error::MissingRequiredKWArg {
            name: "dataframe".to_string(),
        })?
        .clone()
        .expect_lazy_frame()?;

    let df = lf.clone().collect().map_err(Error::PolarsError)?;

    // Required arguments (accepts :labels or alias :x, and :values or alias :y)
    let labels = kw(&args, ":labels")
        .or_else(|| kw(&args, ":x"))
        .ok_or_else(|| Error::MissingRequiredKWArg {
            name: ":labels".to_string(),
        })?
        .expect_string()?;

    let values = kw(&args, ":values")
        .or_else(|| kw(&args, ":y"))
        .ok_or_else(|| Error::MissingRequiredKWArg {
            name: ":values".to_string(),
        })?
        .expect_string()?;

    // Optional arguments
    let font_size = plot_font_size(&args)?;
    let title = kw(&args, ":title").map(|k| k.expect_string()).transpose()?;
    let x_title = kw(&args, ":x-title")
        .map(|k| k.expect_string())
        .transpose()?;
    let y_title = kw(&args, ":y-title")
        .map(|k| k.expect_string())
        .transpose()?;
    let group = kw(&args, ":group").map(|k| k.expect_string()).transpose()?;
    let error = kw(&args, ":error").map(|k| k.expect_string()).transpose()?;

    let plot = BarPlot::builder()
        .data(&df)
        .labels(labels.as_str())
        .values(values.as_str())
        .maybe_plot_title(title.as_deref().map(|s| Text::from(s).size(font_size)))
        .maybe_x_title(x_title.as_deref().map(|s| Text::from(s).size(font_size)))
        .maybe_y_title(y_title.as_deref().map(|s| Text::from(s).size(font_size)))
        .maybe_group(group.as_deref())
        .maybe_error(error.as_deref())
        .build();

    show_plot(&plot, &args)?;

    Ok(new_lazy_frame(lf))
}

pub fn pie_plot(args: Vec<Expr>, env: &Rc<RefCell<Env>>) -> Result<Expr, Error> {
    let lf = args
        .first()
        .ok_or_else(|| Error::MissingRequiredKWArg {
            name: "dataframe".to_string(),
        })?
        .clone()
        .expect_lazy_frame()?;

    let df = lf.clone().collect().map_err(Error::PolarsError)?;

    // Required arguments
    let labels = require_kw(&args, ":labels")?.expect_string()?;

    // Optional arguments
    let font_size = plot_font_size(&args)?;
    let title = kw(&args, ":title").map(|k| k.expect_string()).transpose()?;
    let hole = kw(&args, ":hole").map(|k| k.expect_float()).transpose()?;
    let pull = kw(&args, ":pull").map(|k| k.expect_float()).transpose()?;
    let rotation = kw(&args, ":rotation")
        .map(|k| k.expect_float())
        .transpose()?;
    let facet = kw(&args, ":facet").map(|k| k.expect_string()).transpose()?;

    let plot = PieChart::builder()
        .data(&df)
        .labels(labels.as_str())
        .maybe_plot_title(title.as_deref().map(|s| Text::from(s).size(font_size)))
        .maybe_hole(hole)
        .maybe_pull(pull)
        .maybe_rotation(rotation)
        .maybe_facet(facet.as_deref())
        .build();

    show_plot(&plot, &args)?;

    Ok(new_lazy_frame(lf))
}

pub fn histogram(args: Vec<Expr>, env: &Rc<RefCell<Env>>) -> Result<Expr, Error> {
    let lf = args
        .first()
        .ok_or_else(|| Error::MissingRequiredKWArg {
            name: "dataframe".to_string(),
        })?
        .clone()
        .expect_lazy_frame()?;

    let df = lf.clone().collect().map_err(Error::PolarsError)?;

    // Required arguments
    let x = require_kw(&args, ":x")?.expect_string()?;

    // Optional arguments
    let font_size = plot_font_size(&args)?;
    let title = kw(&args, ":title").map(|k| k.expect_string()).transpose()?;
    let x_title = kw(&args, ":x-title")
        .map(|k| k.expect_string())
        .transpose()?;
    let y_title = kw(&args, ":y-title")
        .map(|k| k.expect_string())
        .transpose()?;
    let group = kw(&args, ":group").map(|k| k.expect_string()).transpose()?;
    let opacity = kw(&args, ":opacity")
        .map(|k| k.expect_float())
        .transpose()?;
    let facet = kw(&args, ":facet").map(|k| k.expect_string()).transpose()?;

    let plot = Histogram::builder()
        .data(&df)
        .x(x.as_str())
        .maybe_plot_title(title.as_deref().map(|s| Text::from(s).size(font_size)))
        .maybe_x_title(x_title.as_deref().map(|s| Text::from(s).size(font_size)))
        .maybe_y_title(y_title.as_deref().map(|s| Text::from(s).size(font_size)))
        .maybe_group(group.as_deref())
        .maybe_opacity(opacity)
        .maybe_facet(facet.as_deref())
        .build();

    show_plot(&plot, &args)?;

    Ok(new_lazy_frame(lf))
}

pub fn box_plot(args: Vec<Expr>, env: &Rc<RefCell<Env>>) -> Result<Expr, Error> {
    let lf = args
        .first()
        .ok_or_else(|| Error::MissingRequiredKWArg {
            name: "dataframe".to_string(),
        })?
        .clone()
        .expect_lazy_frame()?;

    let df = lf.clone().collect().map_err(Error::PolarsError)?;

    // Required arguments (accepts :labels / :x, and :values / :y)
    let labels = kw(&args, ":labels")
        .or_else(|| kw(&args, ":x"))
        .ok_or_else(|| Error::MissingRequiredKWArg {
            name: ":labels".to_string(),
        })?
        .expect_string()?;

    let values = kw(&args, ":values")
        .or_else(|| kw(&args, ":y"))
        .ok_or_else(|| Error::MissingRequiredKWArg {
            name: ":values".to_string(),
        })?
        .expect_string()?;

    // Optional arguments
    let font_size = plot_font_size(&args)?;
    let title = kw(&args, ":title").map(|k| k.expect_string()).transpose()?;
    let x_title = kw(&args, ":x-title")
        .map(|k| k.expect_string())
        .transpose()?;
    let y_title = kw(&args, ":y-title")
        .map(|k| k.expect_string())
        .transpose()?;
    let group = kw(&args, ":group").map(|k| k.expect_string()).transpose()?;
    let opacity = kw(&args, ":opacity")
        .map(|k| k.expect_float())
        .transpose()?;
    let facet = kw(&args, ":facet").map(|k| k.expect_string()).transpose()?;
    let jitter = kw(&args, ":jitter").map(|k| k.expect_float()).transpose()?;
    let point_offset = kw(&args, ":point-offset")
        .map(|k| k.expect_float())
        .transpose()?;
    let box_points = if has_flag(&args, ":box-points") {
        Some(true)
    } else {
        None
    };

    let plot = BoxPlot::builder()
        .data(&df)
        .labels(labels.as_str())
        .values(values.as_str())
        .maybe_plot_title(title.as_deref().map(|s| Text::from(s).size(font_size)))
        .maybe_x_title(x_title.as_deref().map(|s| Text::from(s).size(font_size)))
        .maybe_y_title(y_title.as_deref().map(|s| Text::from(s).size(font_size)))
        .maybe_group(group.as_deref())
        .maybe_opacity(opacity)
        .maybe_facet(facet.as_deref())
        .maybe_box_points(box_points)
        .maybe_point_offset(point_offset)
        .maybe_jitter(jitter)
        .build();

    show_plot(&plot, &args)?;

    Ok(new_lazy_frame(lf))
}

pub fn time_series_plot(args: Vec<Expr>, env: &Rc<RefCell<Env>>) -> Result<Expr, Error> {
    let lf = args
        .first()
        .ok_or_else(|| Error::MissingRequiredKWArg {
            name: "dataframe".to_string(),
        })?
        .clone()
        .expect_lazy_frame()?;

    let mut df = lf.clone().collect().map_err(Error::PolarsError)?;

    // Required arguments
    let x = require_kw(&args, ":x")?.expect_string()?;
    let y = require_kw(&args, ":y")?.expect_string()?;

    // Cast Date / Datetime to ISO-8601 String for Plotly rendering
    if let Ok(col) = df.column(&x)
        && matches!(col.dtype(), DataType::Date | DataType::Datetime(_, _))
    {
        let str_col = col.cast(&DataType::String).map_err(Error::PolarsError)?;
        df.with_column(str_col).map_err(Error::PolarsError)?;
    }

    // Optional arguments
    let font_size = plot_font_size(&args)?;
    let title = kw(&args, ":title").map(|k| k.expect_string()).transpose()?;
    let x_title = kw(&args, ":x-title")
        .map(|k| k.expect_string())
        .transpose()?;
    let y_title = kw(&args, ":y-title")
        .map(|k| k.expect_string())
        .transpose()?;
    let y2_title = kw(&args, ":y2-title")
        .map(|k| k.expect_string())
        .transpose()?;
    let facet = kw(&args, ":facet").map(|k| k.expect_string()).transpose()?;
    let width = kw(&args, ":width").map(|k| k.expect_float()).transpose()?;
    let size = kw(&args, ":size")
        .map(|k| {
            let val = k.expect_int()?;
            usize::try_from(val).map_err(|_| {
                Error::Custom(format!("`:size` must be a positive integer, got {}", val))
            })
        })
        .transpose()?;

    // Optional additional series (expects a list of strings or single string)
    let additional_series = match kw(&args, ":additional-series") {
        Some(some) => match some.value.clone() {
            ExprValue::List(list) => {
                let mut series_vec = Vec::new();
                for item in list {
                    series_vec.push(item.expect_string()?);
                }
                Some(series_vec)
            }
            ExprValue::String(s) => Some(vec![s]),
            _other => {
                return Err(Error::WrongType {
                    expected: "list or string".into(),
                    got: some.clone(),
                });
            }
        },
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
        .maybe_plot_title(title.as_deref().map(|s| Text::from(s).size(font_size)))
        .maybe_x_title(x_title.as_deref().map(|s| Text::from(s).size(font_size)))
        .maybe_y_title(y_title.as_deref().map(|s| Text::from(s).size(font_size)))
        .maybe_y2_title(y2_title.as_deref().map(|s| Text::from(s).size(font_size)))
        .maybe_facet(facet.as_deref())
        .maybe_width(width)
        .maybe_size(size)
        .build();

    show_plot(&plot, &args)?;

    Ok(new_lazy_frame(lf))
}

pub fn line_plot(args: Vec<Expr>, env: &Rc<RefCell<Env>>) -> Result<Expr, Error> {
    let lf = args
        .first()
        .ok_or_else(|| Error::MissingRequiredKWArg {
            name: "dataframe".to_string(),
        })?
        .clone()
        .expect_lazy_frame()?;

    let df = lf.clone().collect().map_err(Error::PolarsError)?;

    // Required arguments
    let x = require_kw(&args, ":x")?.expect_string()?;
    let y = require_kw(&args, ":y")?.expect_string()?;

    // Optional arguments
    let font_size = plot_font_size(&args)?;
    let title = kw(&args, ":title").map(|k| k.expect_string()).transpose()?;
    let x_title = kw(&args, ":x-title")
        .map(|k| k.expect_string())
        .transpose()?;
    let y_title = kw(&args, ":y-title")
        .map(|k| k.expect_string())
        .transpose()?;
    let y2_title = kw(&args, ":y2-title")
        .map(|k| k.expect_string())
        .transpose()?;
    let facet = kw(&args, ":facet").map(|k| k.expect_string()).transpose()?;
    let width = kw(&args, ":width").map(|k| k.expect_float()).transpose()?;
    let size = kw(&args, ":size")
        .map(|k| {
            let val = k.expect_int()?;
            usize::try_from(val).map_err(|_| {
                Error::Custom(format!("`:size` must be a positive integer, got {}", val))
            })
        })
        .transpose()?;

    // Optional additional lines (expects a list of strings or a single string)
    let additional_lines = match kw(&args, ":additional-lines") {
        Some(some) => match some.value.clone() {
            ExprValue::List(list) => {
                let mut lines_vec = Vec::new();
                for item in list {
                    lines_vec.push(item.expect_string()?);
                }
                Some(lines_vec)
            }
            ExprValue::String(s) => Some(vec![s]),
            _other => {
                return Err(Error::WrongType {
                    expected: "list or string".into(),
                    got: some,
                });
            }
        },
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
        .maybe_plot_title(title.as_deref().map(|s| Text::from(s).size(font_size)))
        .maybe_x_title(x_title.as_deref().map(|s| Text::from(s).size(font_size)))
        .maybe_y_title(y_title.as_deref().map(|s| Text::from(s).size(font_size)))
        .maybe_y2_title(y2_title.as_deref().map(|s| Text::from(s).size(font_size)))
        .maybe_facet(facet.as_deref())
        .maybe_width(width)
        .maybe_size(size)
        .build();

    show_plot(&plot, &args)?;

    Ok(new_lazy_frame(lf))
}
