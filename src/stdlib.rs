use crate::corelib::*;
use crate::plotlarslib;
use crate::polarslib::*;
use crate::types::{Env, Error, Expr, ExprValue, NativeFunctionData};
use polars::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub fn insert_native_function(
    defines: &mut HashMap<String, Expr>,
    name: &str,
    body: fn(Vec<Expr>) -> Result<Expr, Error>,
    min_args: u8,
    max_args: u8,
) {
    let func_data = NativeFunctionData {
        body,
        min_args,
        max_args,
    };

    defines.insert(
        name.to_string(),
        Expr {
            value: ExprValue::NativeFunction(Box::new(func_data)),
            line: None,
        },
    );
}

pub fn std_env() -> Rc<RefCell<Env>> {
    let mut defines = HashMap::new();

    // Constant symbol
    defines.insert("nil".to_string(), nil());
    defines.insert("true".to_string(), true_());
    defines.insert("false".to_string(), false_());

    // Math Constants
    defines.insert("PI".to_string(), new_float(std::f64::consts::PI));
    defines.insert("E".to_string(), new_float(std::f64::consts::E));

    // Native Functions: (defines, name, function_pointer, min_args, max_args)
    insert_native_function(&mut defines, "print", print, 0, u8::MAX);
    insert_native_function(&mut defines, "shell", shell, 1, u8::MAX);
    insert_native_function(&mut defines, "type-of", crate::corelib::type_of, 1, 1);

    // Native List Functions
    insert_native_function(&mut defines, "list", list, 0, u8::MAX);
    insert_native_function(&mut defines, "car", crate::corelib::car, 1, 1);
    insert_native_function(&mut defines, "first", crate::corelib::car, 1, 1);
    insert_native_function(&mut defines, "cdr", crate::corelib::cdr, 1, 1);
    insert_native_function(&mut defines, "rest", crate::corelib::cdr, 1, 1);
    insert_native_function(&mut defines, "cons", crate::corelib::cons, 2, 2);
    insert_native_function(&mut defines, "len", crate::corelib::length, 1, 1);
    insert_native_function(&mut defines, "nth", crate::corelib::nth, 2, 2);
    insert_native_function(&mut defines, "append", crate::corelib::append, 1, u8::MAX);
    insert_native_function(&mut defines, "reverse", crate::corelib::reverse, 1, 1);
    insert_native_function(&mut defines, "empty?", crate::corelib::is_empty, 1, 1);

    // Native String Functions
    insert_native_function(
        &mut defines,
        "str-concat",
        crate::corelib::str_concat,
        1,
        u8::MAX,
    );
    insert_native_function(
        &mut defines,
        "str-concat",
        crate::corelib::str_concat,
        1,
        u8::MAX,
    );
    insert_native_function(&mut defines, "str-len", crate::corelib::str_len, 1, 1);
    insert_native_function(&mut defines, "substring", crate::corelib::substring, 2, 3);
    insert_native_function(&mut defines, "split", crate::corelib::split, 2, 2);
    insert_native_function(&mut defines, "str-join", crate::corelib::str_join, 1, 2);

    // Native Conversion Functions
    insert_native_function(&mut defines, "to-str", crate::corelib::to_str, 1, 1);
    insert_native_function(&mut defines, "to-string", crate::corelib::to_str, 1, 1);
    insert_native_function(&mut defines, "to-int", crate::corelib::to_int, 1, 1);
    insert_native_function(&mut defines, "to-float", crate::corelib::to_float, 1, 1);
    insert_native_function(&mut defines, "to-symbol", crate::corelib::to_symbol, 1, 1);
    insert_native_function(&mut defines, "to-bool", crate::corelib::to_bool, 1, 1);
    insert_native_function(&mut defines, "to-list", crate::corelib::to_list, 1, 1);

    // Native Higher Order Functions
    insert_native_function(&mut defines, "range", crate::corelib::range, 1, 3);
    insert_native_function(&mut defines, "apply", crate::corelib::apply, 2, u8::MAX);
    insert_native_function(&mut defines, "map", crate::corelib::map, 2, 2);
    insert_native_function(&mut defines, "reduce", crate::corelib::reduce, 2, 3);

    // Arithmetic
    insert_native_function(&mut defines, "+", plus, 1, u8::MAX);
    insert_native_function(&mut defines, "-", minus, 1, u8::MAX);
    insert_native_function(&mut defines, "*", mult, 1, u8::MAX);
    insert_native_function(&mut defines, "/", divide, 1, u8::MAX);

    // Math Functions
    insert_native_function(&mut defines, "abs", crate::corelib::abs, 1, 1);
    insert_native_function(&mut defines, "sqrt", sqrt, 1, 1);
    insert_native_function(&mut defines, "exp", exp, 1, 1);
    insert_native_function(&mut defines, "log", log, 1, 2);
    insert_native_function(&mut defines, "log10", log10, 1, 1);
    insert_native_function(&mut defines, "pow", pow, 2, 2);
    insert_native_function(&mut defines, "round", round, 1, 2);
    insert_native_function(&mut defines, "floor", floor, 1, 1);
    insert_native_function(&mut defines, "ceil", ceil, 1, 1);
    insert_native_function(&mut defines, "sin", sin, 1, 1);
    insert_native_function(&mut defines, "cos", cos, 1, 1);
    insert_native_function(&mut defines, "tan", tan, 1, 1);

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
    insert_native_function(&mut defines, "describe", crate::polarslib::describe, 1, 1);

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
    insert_native_function(&mut defines, "not", crate::polarslib::not, 1, u8::MAX);

    // Aggregations
    insert_native_function(&mut defines, "sum", crate::polarslib::sum, 1, 1);
    insert_native_function(&mut defines, "mean", crate::polarslib::mean, 1, 1);
    insert_native_function(&mut defines, "median", crate::polarslib::median, 1, 1);
    insert_native_function(&mut defines, "count", count, 0, 1);
    insert_native_function(&mut defines, "min", crate::polarslib::min, 1, 1);
    insert_native_function(&mut defines, "max", crate::polarslib::max, 1, 1);
    insert_native_function(&mut defines, "n-unique", crate::polarslib::n_unique, 1, 1);
    insert_native_function(&mut defines, "std", crate::polarslib::std_dev, 1, 2);
    insert_native_function(&mut defines, "var", crate::polarslib::variance, 1, 2);
    insert_native_function(&mut defines, "quantile", crate::polarslib::quantile, 2, 3);
    insert_native_function(&mut defines, "cov", crate::polarslib::cov_, 2, 3);
    insert_native_function(&mut defines, "corr", crate::polarslib::corr, 2, 2);

    // Register Series slicing and indexing
    insert_native_function(&mut defines, "slice-expr", expr_slice, 3, 3);
    insert_native_function(&mut defines, "get", expr_get, 2, 2);

    // LazyFrame Operations
    insert_native_function(&mut defines, "select", select, 1, u8::MAX);
    insert_native_function(&mut defines, "filter", filter_core, 2, u8::MAX);
    insert_native_function(&mut defines, "with-column", with_column, 3, u8::MAX);
    insert_native_function(&mut defines, "sort", sort, 2, 3);
    insert_native_function(&mut defines, "group-by", group_by, 2, u8::MAX);
    insert_native_function(&mut defines, "alias", alias, 2, 2);
    insert_native_function(&mut defines, "agg", agg, 1, u8::MAX);
    insert_native_function(&mut defines, "join", join, 4, 5);
    insert_native_function(&mut defines, "unique", unique_, 1, u8::MAX);
    insert_native_function(&mut defines, "drop", drop_, 2, u8::MAX);
    insert_native_function(&mut defines, "rename", rename_, 3, 3);

    // Null Handling & Conditionals
    insert_native_function(&mut defines, "null?", is_null_, 1, 1);
    insert_native_function(&mut defines, "not-null?", is_not_null_, 1, 1);
    insert_native_function(&mut defines, "fill-null", fill_null_, 2, 2);
    insert_native_function(&mut defines, "drop-nulls", drop_nulls, 1, u8::MAX);
    insert_native_function(&mut defines, "when", crate::polarslib::when, 2, u8::MAX);

    // String
    insert_native_function(&mut defines, "str-contains", str_contains, 2, 2);
    insert_native_function(&mut defines, "str-replace", str_replace, 3, 3);
    insert_native_function(&mut defines, "str-extract", str_extract, 3, 3);
    insert_native_function(&mut defines, "str-split", str_split, 2, 2);
    insert_native_function(&mut defines, "str-strip", str_strip, 1, 2);
    insert_native_function(&mut defines, "str-strip-start", str_strip_start, 1, 2);
    insert_native_function(&mut defines, "str-strip-end", str_strip_end, 1, 2);
    insert_native_function(&mut defines, "str-slice", str_slice, 3, 3);
    insert_native_function(&mut defines, "str-extract-groups", str_extract_groups, 2, 2);

    // Window & Cumulative Functions
    insert_native_function(&mut defines, "over", over, 2, u8::MAX);
    insert_native_function(&mut defines, "shift", shift, 2, 2);
    insert_native_function(&mut defines, "cum-sum", crate::polarslib::cum_sum, 1, 2);
    insert_native_function(&mut defines, "cum-min", crate::polarslib::cum_min, 1, 2);
    insert_native_function(&mut defines, "cum-max", crate::polarslib::cum_max, 1, 2);
    insert_native_function(&mut defines, "cum-count", crate::polarslib::cum_count, 1, 2);
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
    insert_native_function(
        &mut defines,
        "scatter-plot",
        plotlarslib::scatter_plot,
        1,
        u8::MAX,
    );
    insert_native_function(&mut defines, "bar-plot", plotlarslib::bar_plot, 1, u8::MAX);
    insert_native_function(&mut defines, "pie-plot", plotlarslib::pie_plot, 1, u8::MAX);
    insert_native_function(
        &mut defines,
        "histogram",
        plotlarslib::histogram,
        1,
        u8::MAX,
    );
    insert_native_function(&mut defines, "box-plot", plotlarslib::box_plot, 1, u8::MAX);
    insert_native_function(
        &mut defines,
        "time-series-plot",
        plotlarslib::time_series_plot,
        1,
        u8::MAX,
    );
    insert_native_function(
        &mut defines,
        "line-plot",
        plotlarslib::line_plot,
        1,
        u8::MAX,
    );

    // Polars to lisp
    insert_native_function(&mut defines, "scalar", scalar, 1, 1);

    Rc::new(RefCell::new(Env {
        defines,
        outer: None,
    }))
}
