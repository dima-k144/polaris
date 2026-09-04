use crate::corelib::*;
use crate::plotlarslib;
use crate::polarslib::*;
use crate::types::{Env, Error, Expr, ExprValue, NativeFunctionData};
use polars::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub static NATIVE_FUNC_TABLE: &[(&str, NativeFunctionData)] = &[
     // Native Functions: (defines, name, function_pointer, min_args, max_args)
    ("print", NativeFunctionData { body: print, min_args: 0, max_args: u8::MAX }),
    ("shell", NativeFunctionData { body: shell, min_args: 1, max_args: u8::MAX }),
    ("type-of", NativeFunctionData { body: crate::corelib::type_of, min_args: 1, max_args: 1 }),

    // Native List Functions
    ("list", NativeFunctionData { body: list, min_args: 0, max_args: u8::MAX }),
    ("car", NativeFunctionData { body: crate::corelib::car, min_args: 1, max_args: 1 }),
    ("first", NativeFunctionData { body: crate::corelib::car, min_args: 1, max_args: 1 }),
    ("cdr", NativeFunctionData { body: crate::corelib::cdr, min_args: 1, max_args: 1 }),
    ("rest", NativeFunctionData { body: crate::corelib::cdr, min_args: 1, max_args: 1 }),
    ("cons", NativeFunctionData { body: crate::corelib::cons, min_args: 2, max_args: 2 }),
    ("len", NativeFunctionData { body: crate::corelib::length, min_args: 1, max_args: 1 }),
    ("nth", NativeFunctionData { body: crate::corelib::nth, min_args: 2, max_args: 2 }),
    ("append", NativeFunctionData { body: crate::corelib::append, min_args: 1, max_args: u8::MAX }),
    ("reverse", NativeFunctionData { body: crate::corelib::reverse, min_args: 1, max_args: 1 }),
    ("empty?", NativeFunctionData { body: crate::corelib::is_empty, min_args: 1, max_args: 1 }),

    // Native String Functions
    ("str-concat", NativeFunctionData { body: crate::corelib::str_concat, min_args: 1, max_args: u8::MAX }),
    ("str-len", NativeFunctionData { body: crate::corelib::str_len, min_args: 1, max_args: 1 }),
    ("substring", NativeFunctionData { body: crate::corelib::substring, min_args: 2, max_args: 3 }),
    ("split", NativeFunctionData { body: crate::corelib::split, min_args: 2, max_args: 2 }),
    ("str-join", NativeFunctionData { body: crate::corelib::str_join, min_args: 1, max_args: 2 }),

    // Native Conversion Functions
    ("to-str", NativeFunctionData { body: crate::corelib::to_str, min_args: 1, max_args: 1 }),
    ("to-string", NativeFunctionData { body: crate::corelib::to_str, min_args: 1, max_args: 1 }),
    ("to-int", NativeFunctionData { body: crate::corelib::to_int, min_args: 1, max_args: 1 }),
    ("to-float", NativeFunctionData { body: crate::corelib::to_float, min_args: 1, max_args: 1 }),
    ("to-symbol", NativeFunctionData { body: crate::corelib::to_symbol, min_args: 1, max_args: 1 }),
    ("to-bool", NativeFunctionData { body: crate::corelib::to_bool, min_args: 1, max_args: 1 }),
    ("to-list", NativeFunctionData { body: crate::corelib::to_list, min_args: 1, max_args: 1 }),

    // Native Higher Order Functions
    ("range", NativeFunctionData { body: crate::corelib::range, min_args: 1, max_args: 3 }),
    ("apply", NativeFunctionData { body: crate::corelib::apply, min_args: 2, max_args: u8::MAX }),
    ("map", NativeFunctionData { body: crate::corelib::map, min_args: 2, max_args: 2 }),
    ("reduce", NativeFunctionData { body: crate::corelib::reduce, min_args: 2, max_args: 3 }),

    // Arithmetic
    ("+", NativeFunctionData { body: plus, min_args: 1, max_args: u8::MAX }),
    ("-", NativeFunctionData { body: minus, min_args: 1, max_args: u8::MAX }),
    ("*", NativeFunctionData { body: mult, min_args: 1, max_args: u8::MAX }),
    ("/", NativeFunctionData { body: divide, min_args: 1, max_args: u8::MAX }),

    // Math Functions
    ("abs", NativeFunctionData { body: crate::corelib::abs, min_args: 1, max_args: 1 }),
    ("sqrt", NativeFunctionData { body: sqrt, min_args: 1, max_args: 1 }),
    ("exp", NativeFunctionData { body: exp, min_args: 1, max_args: 1 }),
    ("log", NativeFunctionData { body: log, min_args: 1, max_args: 2 }),
    ("log10", NativeFunctionData { body: log10, min_args: 1, max_args: 1 }),
    ("pow", NativeFunctionData { body: pow, min_args: 2, max_args: 2 }),
    ("round", NativeFunctionData { body: round, min_args: 1, max_args: 2 }),
    ("floor", NativeFunctionData { body: floor, min_args: 1, max_args: 1 }),
    ("ceil", NativeFunctionData { body: ceil, min_args: 1, max_args: 1 }),
    ("sin", NativeFunctionData { body: sin, min_args: 1, max_args: 1 }),
    ("cos", NativeFunctionData { body: cos, min_args: 1, max_args: 1 }),
    ("tan", NativeFunctionData { body: tan, min_args: 1, max_args: 1 }),

    // IO
    ("read-csv", NativeFunctionData { body: read_csv, min_args: 1, max_args: 1 }),
    ("write-csv", NativeFunctionData { body: write_csv, min_args: 2, max_args: 2 }),
    ("sink", NativeFunctionData { body: sink, min_args: 2, max_args: 2 }),
    ("read-parquet", NativeFunctionData { body: read_parquet, min_args: 1, max_args: 1 }),
    ("write-parquet", NativeFunctionData { body: write_parquet, min_args: 2, max_args: 2 }),

    // Collect
    ("collect", NativeFunctionData { body: collect_, min_args: 1, max_args: 1 }),

    // Inspection
    ("head", NativeFunctionData { body: head, min_args: 1, max_args: 1 }),
    ("tail", NativeFunctionData { body: tail, min_args: 1, max_args: 1 }),
    ("slice", NativeFunctionData { body: slice, min_args: 3, max_args: 3 }),
    ("schema", NativeFunctionData { body: schema, min_args: 1, max_args: 1 }),
    ("shape", NativeFunctionData { body: shape, min_args: 1, max_args: 1 }),
    ("describe", NativeFunctionData { body: crate::polarslib::describe, min_args: 1, max_args: 1 }),

    // Expressions & Literals
    ("col", NativeFunctionData { body: col_, min_args: 1, max_args: 1 }),
    ("lit", NativeFunctionData { body: lit_, min_args: 1, max_args: 1 }),

    // Comparison & Logical
    ("=", NativeFunctionData { body: eq, min_args: 2, max_args: 2 }),
    ("!=", NativeFunctionData { body: neq, min_args: 2, max_args: 2 }),
    (">", NativeFunctionData { body: gt, min_args: 2, max_args: 2 }),
    ("<", NativeFunctionData { body: lt, min_args: 2, max_args: 2 }),
    (">=", NativeFunctionData { body: gt_eq, min_args: 2, max_args: 2 }),
    ("<=", NativeFunctionData { body: lt_eq, min_args: 2, max_args: 2 }),
    ("not", NativeFunctionData { body: crate::polarslib::not, min_args: 1, max_args: u8::MAX }),

    // Aggregations
    ("sum", NativeFunctionData { body: crate::polarslib::sum, min_args: 1, max_args: 1 }),
    ("mean", NativeFunctionData { body: crate::polarslib::mean, min_args: 1, max_args: 1 }),
    ("median", NativeFunctionData { body: crate::polarslib::median, min_args: 1, max_args: 1 }),
    ("count", NativeFunctionData { body: count, min_args: 0, max_args: 1 }),
    ("min", NativeFunctionData { body: crate::polarslib::min, min_args: 1, max_args: 1 }),
    ("max", NativeFunctionData { body: crate::polarslib::max, min_args: 1, max_args: 1 }),
    ("n-unique", NativeFunctionData { body: crate::polarslib::n_unique, min_args: 1, max_args: 1 }),
    ("std", NativeFunctionData { body: crate::polarslib::std_dev, min_args: 1, max_args: 2 }),
    ("var", NativeFunctionData { body: crate::polarslib::variance, min_args: 1, max_args: 2 }),
    ("quantile", NativeFunctionData { body: crate::polarslib::quantile, min_args: 2, max_args: 3 }),
    ("cov", NativeFunctionData { body: crate::polarslib::cov_, min_args: 2, max_args: 3 }),
    ("corr", NativeFunctionData { body: crate::polarslib::corr, min_args: 2, max_args: 2 }),

    // Register Series slicing and indexing
    ("slice-expr", NativeFunctionData { body: expr_slice, min_args: 3, max_args: 3 }),
    ("get", NativeFunctionData { body: expr_get, min_args: 2, max_args: 2 }),

    // LazyFrame Operations
    ("select", NativeFunctionData { body: select, min_args: 1, max_args: u8::MAX }),
    ("filter", NativeFunctionData { body: filter_core, min_args: 2, max_args: u8::MAX }),
    ("with-column", NativeFunctionData { body: with_column, min_args: 3, max_args: u8::MAX }),
    ("sort", NativeFunctionData { body: sort, min_args: 2, max_args: 3 }),
    ("group-by", NativeFunctionData { body: group_by, min_args: 2, max_args: u8::MAX }),
    ("alias", NativeFunctionData { body: alias, min_args: 2, max_args: 2 }),
    ("agg", NativeFunctionData { body: agg, min_args: 1, max_args: u8::MAX }),
    ("join", NativeFunctionData { body: join, min_args: 4, max_args: 5 }),
    ("unique", NativeFunctionData { body: unique_, min_args: 1, max_args: u8::MAX }),
    ("drop", NativeFunctionData { body: drop_, min_args: 2, max_args: u8::MAX }),
    ("rename", NativeFunctionData { body: rename_, min_args: 3, max_args: 3 }),

    // Null Handling & Conditionals
    ("null?", NativeFunctionData { body: is_null_, min_args: 1, max_args: 1 }),
    ("not-null?", NativeFunctionData { body: is_not_null_, min_args: 1, max_args: 1 }),
    ("fill-null", NativeFunctionData { body: fill_null_, min_args: 2, max_args: 2 }),
    ("drop-nulls", NativeFunctionData { body: drop_nulls, min_args: 1, max_args: u8::MAX }),
    ("when", NativeFunctionData { body: crate::polarslib::when, min_args: 2, max_args: u8::MAX }),

    // String
    ("str-contains", NativeFunctionData { body: str_contains, min_args: 2, max_args: 2 }),
    ("str-replace", NativeFunctionData { body: str_replace, min_args: 3, max_args: 3 }),
    ("str-extract", NativeFunctionData { body: str_extract, min_args: 3, max_args: 3 }),
    ("str-split", NativeFunctionData { body: str_split, min_args: 2, max_args: 2 }),
    ("str-strip", NativeFunctionData { body: str_strip, min_args: 1, max_args: 2 }),
    ("str-strip-start", NativeFunctionData { body: str_strip_start, min_args: 1, max_args: 2 }),
    ("str-strip-end", NativeFunctionData { body: str_strip_end, min_args: 1, max_args: 2 }),
    ("str-slice", NativeFunctionData { body: str_slice, min_args: 3, max_args: 3 }),
    ("str-extract-groups", NativeFunctionData { body: str_extract_groups, min_args: 2, max_args: 2 }),

    // Window & Cumulative Functions
    ("over", NativeFunctionData { body: over, min_args: 2, max_args: u8::MAX }),
    ("shift", NativeFunctionData { body: shift, min_args: 2, max_args: 2 }),
    ("cum-sum", NativeFunctionData { body: crate::polarslib::cum_sum, min_args: 1, max_args: 2 }),
    ("cum-min", NativeFunctionData { body: crate::polarslib::cum_min, min_args: 1, max_args: 2 }),
    ("cum-max", NativeFunctionData { body: crate::polarslib::cum_max, min_args: 1, max_args: 2 }),
    ("cum-count", NativeFunctionData { body: crate::polarslib::cum_count, min_args: 1, max_args: 2 }),
    ("rank", NativeFunctionData { body: rank, min_args: 1, max_args: 2 }),

    // Date and time
    ("to-date", NativeFunctionData { body: to_date, min_args: 1, max_args: 2 }),
    ("to-datetime", NativeFunctionData { body: to_datetime, min_args: 1, max_args: 2 }),
    ("dt-year", NativeFunctionData { body: dt_year, min_args: 1, max_args: 1 }),
    ("dt-month", NativeFunctionData { body: dt_month, min_args: 1, max_args: 1 }),
    ("dt-day", NativeFunctionData { body: dt_day, min_args: 1, max_args: 1 }),
    ("dt-hour", NativeFunctionData { body: dt_hour, min_args: 1, max_args: 1 }),
    ("dt-format", NativeFunctionData { body: dt_format, min_args: 2, max_args: 2 }),

    // Reshaping & Set Operations
    ("pivot", NativeFunctionData { body: pivot_, min_args: 4, max_args: 5 }),
    ("melt", NativeFunctionData { body: melt_, min_args: 3, max_args: 5 }),
    ("unpivot", NativeFunctionData { body: melt_, min_args: 3, max_args: 5 }),
    ("concat", NativeFunctionData { body: concat_, min_args: 1, max_args: u8::MAX }),

    // Rolling / Window aggregations
    ("rolling-mean", NativeFunctionData { body: rolling_mean, min_args: 2, max_args: 4 }),
    ("rolling-sum", NativeFunctionData { body: rolling_sum, min_args: 2, max_args: 4 }),
    ("rolling-min", NativeFunctionData { body: rolling_min, min_args: 2, max_args: 4 }),
    ("rolling-max", NativeFunctionData { body: rolling_max, min_args: 2, max_args: 4 }),
    ("rolling-median", NativeFunctionData { body: rolling_median, min_args: 2, max_args: 4 }),
    ("rolling-std", NativeFunctionData { body: rolling_std, min_args: 2, max_args: 5 }),

    // Casting
    ("cast", NativeFunctionData { body: cast_, min_args: 2, max_args: 2 }),

    // Plotlars
    ("scatter-plot", NativeFunctionData { body: plotlarslib::scatter_plot, min_args: 1, max_args: u8::MAX }),
    ("bar-plot", NativeFunctionData { body: plotlarslib::bar_plot, min_args: 1, max_args: u8::MAX }),
    ("pie-plot", NativeFunctionData { body: plotlarslib::pie_plot, min_args: 1, max_args: u8::MAX }),
    ("histogram", NativeFunctionData { body: plotlarslib::histogram, min_args: 1, max_args: u8::MAX }),
    ("box-plot", NativeFunctionData { body: plotlarslib::box_plot, min_args: 1, max_args: u8::MAX }),
    ("time-series-plot", NativeFunctionData { body: plotlarslib::time_series_plot, min_args: 1, max_args: u8::MAX }),
    ("line-plot", NativeFunctionData { body: plotlarslib::line_plot, min_args: 1, max_args: u8::MAX }),

    // Polars to lisp
    ("scalar", NativeFunctionData { body: scalar, min_args: 1, max_args: 1 }),
];

pub fn std_env() -> Rc<RefCell<Env>> {
    let mut defines = HashMap::new();

    // Constant symbol
    defines.insert("nil".to_string(), nil());
    defines.insert("true".to_string(), true_());
    defines.insert("false".to_string(), false_());

    // Math Constants
    defines.insert("*PI*".to_string(), new_float(std::f64::consts::PI));
    defines.insert("*E*".to_string(), new_float(std::f64::consts::E));

    let args: Vec<Expr> = std::env::args().map(|arg| new_string(arg)).collect();
    defines.insert("*args*".to_string(), new_list(args));

    Rc::new(RefCell::new(Env {
        defines,
        outer: None,
    }))
}
