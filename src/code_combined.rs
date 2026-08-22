use reqwest::Client;
use serde_json::json;

pub async fn ask_ai(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = String::from("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");
    
    let system_oprompt = r#"
# MLisp: Language Specification & AI Prompt Guide

**MLisp** is an S-expression-based Lisp dialect implemented in Rust that embeds and executes Apache Arrow / Polars DataFrame queries with lazy evaluation.

---

### Core Syntax & Reader Rules

* **S-Expressions:** Code is structured with standard parenthesized lists `(function arg1 arg2 ...)`.


* **Comments:** Single-line comments start with `;` and continue to the end of the line.


* **Column Sugar (`$`):** The prefix `$` expands to `(col ...)` during parsing.


* `$"age"` or `$age` $\rightarrow$ `(col "age")` / `(col age)`



* **Keywords / Symbols:** Symbols starting with a colon `:` evaluate directly to themselves (e.g., `:inner`, `:left`, `:desc`, `:horizontal`, `:dense`, `:sum`).


* **Literals:** Supports 64-bit integers (`42`), 64-bit floats (`3.14`), and double-quoted strings (`"hello"`).



---

### Special Forms & Control Flow

* **Variable Definition:** `(define name value)` binds `value` to `name` in the current lexical environment.


* **Function Definition:** `(defun name (param1 param2 ...) body...)` defines a named user function.


* **Anonymous Functions:** `(lambda (param1 param2 ...) body...)` creates an anonymous closure.


* **Local Bindings:** `(let (k1 v1 k2 v2 ...) body...)` creates a new local scope with sequential key-value binding pairs.


* **Thread-First Pipeline (`->`):** Passes the evaluation of the first expression as the second element (first argument) of subsequent expressions:


```lisp
(-> (read-csv "data.csv")
    (filter (> $"age" 25))
    (select $"name" $"age"))

```



---

### Data Types

* **Primitives:** `Int` (`i64`), `Float` (`f64`), `String`, `Symbol`, `List`.


* **Nil:** `nil` evaluates to a null/void symbol representation.


* **Engine Native Objects:** `LazyFrame` (Polars query plan), `PolarsExpr` (column computation / expression).



---

### Standard Library Reference

**I/O & Sinks**

* `(read-csv path)` / `(read-parquet path)`: Lazily loads a CSV or Parquet dataset into a `LazyFrame`.


* `(write-csv lf path)` / `(write-parquet lf path)`: Materializes the `LazyFrame` and writes it to disk.


* `(sink lf path)`: Streams execution directly into a `.parquet` or `.csv` destination.



**LazyFrame Operations**

* `(select lf expr1 expr2 ...)`: Projects specified column expressions.


* `(filter lf condition-expr)`: Filters rows matching a boolean expression.


* `(with-column lf target-col expr)`: Appends or replaces a column.


* `(sort lf by-expr [:asc | :desc])`: Sorts by expression.


* `(group-by lf (list keys...) / key agg-expr1 agg-expr2 ...)`: Groups and aggregates.


* `(agg lf expr1 expr2 ...)`: Aggregates across the entire frame.


* `(join left right left-on right-on [:inner | :left | :outer | :semi | :anti | :cross])`: Joins two frames.


* `(drop-nulls lf [cols...])`: Drops null entries across the frame or selected columns.


* `(pivot lf on-cols index-cols value-cols [agg-type])`: Reshapes data into wide format (aggs: `:first`, `:sum`, `:mean`, `:min`, `:max`, `:count`).


* `(melt lf id-vars value-vars [var-col-name val-col-name])` / `(unpivot ...)`: Unpivots columns to narrow format.


* `(concat frame1 frame2 ... [:vertical | :horizontal | :diagonal])`: Concatenates frames.


* `(head lf)` / `(tail lf)` / `(slice lf offset len)`: Limits / slices frame rows.


* `(schema lf)` / `(shape lf)`: Inspects frame metadata.



**Column Expressions & Transformations**

* **Arithmetic:** `(+ a b ...)`, `(- a b ...)`, `(* a b ...)`, `(/ a b ...)`

* **Logic & Comparison:** `(= a b)`, `(!= a b)`, `(> a b)`, `(< a b)`, `(>= a b)`, `(<= a b)`, `(and a b)`, `(or a b)`, `(not a)`

* **Conditionals:** `(when condition on-true [on-false])`

* **Null Checking:** `(null? col)`, `(not-null? col)`, `(fill-null col fill-value)`

* **Aggregators:** `(sum col)`, `(mean col)`, `(median col)`, `(min col)`, `(max col)`, `(count [col])`, `(n-unique col)`, `(std col [ddof])`

* **Window & Cumulative:** `(over expr partition-col...)`, `(shift col offset)`, `(cum-sum col [:reverse])`, `(cum-min ...)`, `(cum-max ...)`, `(cum-count ...)`, `(rank col [:dense | :min | :max | :average | :ordinal])`

* **String Manipulation:** `(str-contains col regex)`, `(str-replace col pattern replacement)`, `(str-extract col regex group-index)`

* **Date & Datetime:** `(to-date col [format])`, `(to-datetime col [format])`, `(dt-year col)`, `(dt-month col)`, `(dt-day col)`, `(dt-hour col)`, `(dt-format col format)`

* **Aliasing:** `(alias expr "new_name")`

* **Scalar Extraction:** `(scalar 1x1-lf)` converts a 1x1 LazyFrame back into an MLisp native scalar type.



---

### Example Program

```lisp
;; Define a helper function to calculate age bands
(defun categorize-age (age-col)
    (when (>= age-col 30) "Senior" "Junior"))

;; Data transformation pipeline
(define result
    (-> (read-csv "employees.csv")
        (filter (and (not-null? $"salary") (> $"salary" 50000)))
        (with-column $"role_level" (categorize-age $"age"))
        (with-column $"running_sal" (over (cum-sum $"salary") $"department"))
        (group-by $"department" 
            (alias (mean $"salary") "avg_salary")
            (alias (count) "total_employees"))
        (sort $"avg_salary" :desc)))

(print result)

```
"#;

    let model = "qwen/qwen3.8-27b";
    let response = Client::new()
        .post("https://openrouter.ai/api/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&json!({
            "model": model,
            "messages": [
                {
                    "role": "system",
                    "content": system_oprompt
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        }))
        .send()
        .await?;

    let json: serde_json::Value = response.json().await?;

    Ok(json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string())
}mod tests {
    use crate::interpreter::{eval_string, State};
    use crate::stdlib::std_env;
    use polars::prelude::*;

    #[test]
    fn test_defun_basic() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (defun add (a b) (+ a b))
            (add 10 25)
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval defun basic failed")
            .expect_int()
            .expect("expected number");

        assert_eq!(res, 35);
    }

    #[test]
    fn test_defun_multi_expression_body() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (defun compute (x y)
                (+ x 1)
                (* x y))
            (compute 4 5)
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval defun multi-body failed")
            .expect_int()
            .expect("expected number");

        assert_eq!(res, 20);
    }

    #[test]
    fn test_lambda_bound_with_define() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (define square (lambda (x) (* x x)))
            (square 6)
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval lambda bound failed")
            .expect_int()
            .expect("expected number");

        assert_eq!(res, 36);
    }

    #[test]
    fn test_lambda_multiple_parameters() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (define calc (lambda (a b c) (- (+ a b) c)))
            (calc 10 20 5)
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval multi-param lambda failed")
            .expect_int()
            .expect("expected number");

        assert_eq!(res, 25);
    }

    #[test]
    fn test_defun_scoping_isolation() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (define x 100)
            (defun override_param (x) (+ x 5))
            (override_param 10)
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval defun scope failed")
            .expect_int()
            .expect("expected number");

        assert_eq!(res, 15);
    }

    #[test]
    fn test_let_basic() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (let (a 1 b 2 c 3) (+ a b c))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval let basic failed")
            .expect_int()
            .expect("expected number");

        assert_eq!(res, 6);
    }

    #[test]
    fn test_let_scoping_isolation() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (define x 100)
            (let (x 10 y 20) (+ x y))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval let scope failed")
            .expect_int()
            .expect("expected number");

        assert_eq!(res, 30);

        // Verify outer scope was not mutated
        let outer_res = eval_string("x".to_string(), &env, &mut state)
            .expect("eval outer lookup failed")
            .expect_int()
            .expect("expected number");

        assert_eq!(outer_res, 100);
    }

    #[test]
    fn test_let_multi_expression_body() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (let (a 5 b 10)
                (+ a b)
                (* a b))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval multi-expression let failed")
            .expect_int()
            .expect("expected number");

        assert_eq!(res, 50);
    }

    #[test]
    fn test_let_nested() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (let (a 2 b 3)
                (let (c 4 d 5)
                    (+ (* a b) (* c d))))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval nested let failed")
            .expect_int()
            .expect("expected number");

        assert_eq!(res, 26);
    }

    #[test]
    fn test_let_odd_bindings_error() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (let (a 1 b) (+ a 1))
        "#.to_string();

        let res = eval_string(code, &env, &mut state);
        assert!(res.is_err());
    }
}use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use crate::types::{Env, Expr, Error};

pub struct State {
    pub line: usize
}

pub fn lookup(symbol: &str, env: &Rc<RefCell<Env>>) -> Option<Expr> {
    let env_borrow = env.borrow();
    if let Some(val) = env_borrow.defines.get(symbol) {
        Some(val.clone())
    } else if let Some(outer) = &env_borrow.outer {
        lookup(symbol, outer)
    } else {
        None
    }
}

pub fn define(symbol: &String, value: Expr, env: &Rc<RefCell<Env>>) {
    env.borrow_mut().defines.insert(symbol.to_string(), value);
}

pub fn eval(expr: Expr, env: &Rc<RefCell<Env>>, state: &mut State) -> Result<Expr, Error> {
    match expr.clone() {
        Expr::SourceInfo(expr, line) => { state.line = line; eval(*expr, env, state) },
        Expr::Symbol(symbol) => {
            if symbol.starts_with(":") {
                Ok(expr)
            } else {
                match lookup(&symbol, env) {
                    Some(value) => Ok(value),
                    None => Err(Error::LookupNotFound { symbol }),
                }
            }
        }
        Expr::List(list) => {
            let symbol = list[0].clone().expect_symbol()?;

            match symbol.as_str() {
                "define" => { 
                    let name = list[1].clone().expect_symbol()?;
                    let value = eval(list[2].clone(), env, state)?;
                    define(&name, value, env);
                    return Ok(lookup(&String::from("nil"), env).unwrap());
                },
                "lambda" => {
                    let mut params = Vec::<String>::new();
                    for param in list[1].clone().expect_list()? {
                        params.push(param.expect_symbol()?)
                    }
                    let body = list.get(2..).unwrap().to_vec();
                    let func = Expr::UserFunction { body: body, params: params, name: None };
                    return Ok(func)
                },
                "defun" => {
                    let name = list[1].clone().expect_symbol()?;
                    let mut params = Vec::<String>::new();
                    for param in list[2].clone().expect_list()? {
                        params.push(param.expect_symbol()?)
                    }
                    let body = list.get(3..).unwrap().to_vec();
                    let func = Expr::UserFunction { body: body, params: params, name: Some(name.clone()) };
                    define(&name.clone(), func.clone(), env);
                    return Ok(func)
                },
                "let" => {
                    if list.len() < 3 {
                        return Err(Error::WrongNumberOfArguments {
                            expected_min: 2,
                            expected_max: u8::MAX,
                            got: (list.len().saturating_sub(1)) as u8,
                        });
                    }

                    let bindings = list[1].clone().expect_list()?;
                    if bindings.len() % 2 != 0 {
                        return Err(Error::WrongType {
                            expected: "an even number of binding pairs (key value ...)".into(),
                            got: list[1].clone(),
                        });
                    }

                    // Create a new child lexical scope
                    let child_env = Rc::new(RefCell::new(Env::with_parent(Rc::clone(env))));

                    // Evaluate values in the outer/current env, then bind them into child_env
                    // (Use child_env for eval if you prefer `let*` semantics where later bindings see earlier ones)
                    for pair in bindings.chunks(2) {
                        let var_name = pair[0].clone().expect_symbol()?;
                        let var_val = eval(pair[1].clone(), env, state)?;
                        define(&var_name, var_val, &child_env);
                    }

                    // Evaluate all body expressions sequentially and return the result of the last one
                    let body = &list[2..];
                    let mut result = Expr::Symbol("nil".into());
                    for expr in body {
                        result = eval(expr.clone(), &child_env, state)?;
                    }

                    return Ok(result);
                },
                "->" => {
                    let mut var = eval(list[1].clone(), env, state)?;
                    for i in 2..list.len() {
                        let mut expr2 = list[i].clone().expect_list()?;
                        expr2.insert(1, var.clone());
                        var = eval(Expr::List(expr2), env, state)?;
                    }
                    return Ok(var);
                }
                _ => {}
            }

            let func = match lookup(&symbol, env) {
                Some(val) => val,
                None => { return Err(Error::LookupNotFound { symbol }) },
            };

            let mut args = Vec::<Expr>::new();
            for j in 1..list.len() {
                args.push(eval(list[j].clone(), env, state)?);
            }

            match func {
                Expr::NativeFunction{ body, min_args, max_args } => {
                    if args.len() < min_args as usize {
                        return Err(Error::WrongNumberOfArguments { expected_min: min_args, expected_max: max_args, got: args.len() as u8 });
                    }
                    
                    return body(args);
                },
                Expr::UserFunction { body, params, name } => {
                    if args.len() < params.len() {
                        return Err(Error::WrongNumberOfArguments { 
                            expected_min: params.len() as u8, expected_max: params.len() as u8, got: args.len() as u8 
                        });
                    }

                    let child_env = Rc::new(RefCell::new(Env::with_parent(Rc::clone(env))));

                    for (param, arg) in params.iter().zip(args) {
                        define(param, arg, &child_env);
                    }

                    let mut result = Expr::Symbol("nil".into());
                    for expr in body {
                        result = eval(expr, &child_env, state)?;
                    }
                    return Ok(result);
                },
                _ => Err(Error::WrongType { expected: String::from("callable"), got: func })
            }
        },
        _ => Ok(expr)
    }
}

pub fn eval_string(source: String, env: &Rc<RefCell<Env>>, state: &mut State) -> Result<Expr, Error> {
    let mut tokens = crate::lexer::scan(&source)?; 
    let exprs = crate::parser::parse(&mut tokens)?;
    let mut result = None;
    for expr in exprs {
        result = Some(crate::interpreter::eval(expr, env, state));
    }
    result.unwrap()
}
use crate::types::Error;

pub fn scan(source: &String) -> Result<Vec<(String, usize)>, Error> {
    let mut tokens = Vec::<(String, usize)>::new();
    let mut item = String::new();
    let mut line = 0;
    let mut in_string = false;
    let mut in_comment = false;
    for c in source.chars() {
        if in_comment {
            if c == '\n' { line += 1; in_comment = false; }
        } else if in_string {
            match c {
                '"' => {
                    in_string = false;
                    item.push('"');
                    tokens.push((item, line)); item = String::new();
                },
                c => {
                    item.push(c)
                }
            }
        } else {
            match c {
                '(' | ')' | '$' => {
                    if !item.is_empty() {
                        tokens.push((item, line));
                        item = String::new();
                    }
                    tokens.push((c.to_string(), line));
                }
                ' ' | '\t' | '\r' => { 
                    if !item.is_empty() { tokens.push((item, line)); item = String::new(); } 
                },
                '\n' => { 
                    if !item.is_empty() { tokens.push((item, line)); item = String::new(); } 
                    line += 1; 
                },
                '"' => {
                    if !item.is_empty() { tokens.push((item, line)); item = String::new(); } 
                    in_string = true;
                    item.push('"');
                },
                ';' => {
                    in_comment = true;
                }
                c => { item.push(c); }
        }
        }

    }

    if in_string {
        return Err(Error::UnclosedString);
    }

    if !item.is_empty() { tokens.push((item, line)) };
    Ok(tokens)
}
use std::io::{self, Write};
use std::fs;

use crate::ai::ask_ai;
use crate::interpreter::eval_string;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod lexer;
mod parser;
mod types;
mod interpreter;
mod stdlib;
mod core_tests;
mod polars_tests;
mod ai;
mod plotlarslib;

fn is_complete_input(input: &String) -> bool {
    let open_parens_count = input.chars().filter(|&c| c == '(').count();
    let closed_parens_count = input.chars().filter(|&c| c == ')').count();
    let double_quotes_count = input.chars().filter(|&c| c == '"').count();
    open_parens_count == closed_parens_count && double_quotes_count % 2 == 0
}

fn handle_error(error: crate::types::Error, input: String) {
    println!("{}", crate::types::format_error(error.clone()));
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let prompt = format!("User has encountered an error:{}\nUser's code:\n{}\n", crate::types::format_error(error), input);
    let answer = runtime.block_on(crate::ai::ask_ai(&prompt)).unwrap();
    termimad::print_text(&answer);
}

fn repl() {
    let env = crate::stdlib::std_env();

    loop {
        let mut input = String::new();
        print!("mlisp > "); io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut input).unwrap();
        while !is_complete_input(&input) {
            let mut buf = String::new();
            std::io::stdin().read_line(&mut buf).unwrap();
            input.push_str(buf.as_str());
        }
        let mut state = interpreter::State {
            line: 0,
        };
        match eval_string(input.clone(), &env, &mut state) {
            Ok(val) => {
                match val.to_string() {
                    Ok(str) => println!("{str}"),
                    Err(err) => handle_error(err, input),
                }
            },
            Err(err) => handle_error(err, input),
        };
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let lsp_files: Vec<&String> = args.iter().filter(|arg| arg.ends_with(".lsp")).collect();

    if !lsp_files.is_empty() {
        for arg in lsp_files {
            let env = crate::stdlib::std_env();
            let contents = fs::read_to_string(arg).unwrap();
            let mut state = interpreter::State { line: 0 };
            match eval_string(contents.clone(), &env, &mut state) {
                Ok(val) => {
                    match val.to_string() {
                        Ok(str) => println!("{str}"),
                        Err(err) => handle_error(err, contents),
                    }
                },
                Err(err) => handle_error(err, contents),
            };
        }
        return; // Exit without entering REPL
    }

    repl();
}
use crate::types::{Expr, Error};

pub fn parse1(tokens: &mut Vec<(String, usize)>) -> Result<Expr, Error> {
    let token = tokens.remove(0);
    let mut chars = token.0.chars();
    let first_char = chars.next().unwrap();
    let line = token.1;
    if first_char.is_numeric() || (token.0.len() > 1 && first_char == '-' && chars.nth(0).unwrap().is_numeric()) {
        if token.0.contains('.') {
            match token.0.parse::<f64>() {
                Ok(value) => Ok(Expr::SourceInfo(Box::new(Expr::Float(value)), line)),
                Err(err) => Err(Error::FailedToParseNumber { number: token.0 }),
            }
        } else {
            match token.0.parse::<i64>() {
                Ok(value) => Ok(Expr::SourceInfo(Box::new(Expr::Int(value)), line)),
                Err(err) => Err(Error::FailedToParseNumber { number: token.0 }),
            }
        }
    } else if token.0 == "(" {
        let mut items = Vec::<Expr>::new();
        while !tokens.is_empty() && tokens[0].0 != ")" {
            items.push(parse1(tokens)?);
        }

        if tokens.is_empty() {
            return Err(Error::UnclosedListExpr)
        }

        tokens.remove(0); // remove )
        Ok(Expr::List(items))
    } else if first_char == '"' {
        Ok(Expr::String(token.0.chars().skip(1).take(token.0.chars().count() - 2).collect()))
    } else if token.0 == "$" {
        Ok(Expr::List(vec![Expr::Symbol(String::from("col")), parse1(tokens)?]))
    } else {
        Ok(Expr::Symbol(token.0))
    }
}

pub fn parse(tokens: &mut Vec<(String, usize)>) -> Result<Vec<Expr>, Error> {
    let mut exprs = Vec::<Expr>::new();
    while !tokens.is_empty() {
        exprs.push(parse1(tokens)?);
    }
    Ok(exprs)
}use plotlars::{Plot, ScatterPlot, BarPlot, PieChart, Histogram, BoxPlot, TimeSeriesPlot, LinePlot};
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
#[cfg(test)]
mod tests {
    use crate::interpreter::{eval_string, State};
    use crate::types::Error;
    use crate::stdlib::std_env;
    use polars::prelude::*;
    use std::fs::File;

    fn generate_data1_parquet() -> Result<(), PolarsError> {
        let mut df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "department" => &["Engineering", "HR", "Engineering", "Marketing", "HR"],
            "salary" => &[90000i64, 60000, 95000, 70000, 65000]
        )?;

        let file = File::create("data1.parquet").unwrap();
        ParquetWriter::new(file).finish(&mut df)?;
        Ok(())
    }

    #[test]
    fn test_filter_eq() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (filter (read-csv "data1.csv") (= $"age" 30))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with '=' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[2i64],
            "name" => &["Bob"],
            "age" => &[30i64],
            "city" => &["Paris"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_filter_neq() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (filter (read-csv "data1.csv") (!= $"age" 30))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with '!=' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 3, 4, 5],
            "name" => &["Alice", "Charlie", "Diana", "Evan"],
            "age" => &[25i64, 22, 28, 35],
            "city" => &["London", "Berlin", "Rome", "Madrid"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_filter_gt() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (filter (read-csv "data1.csv") (> $"age" 25))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with '>' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[2i64, 4, 5],
            "name" => &["Bob", "Diana", "Evan"],
            "age" => &[30i64, 28, 35],
            "city" => &["Paris", "Rome", "Madrid"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_filter_lt() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (filter (read-csv "data1.csv") (< $"age" 28))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with '<' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 3],
            "name" => &["Alice", "Charlie"],
            "age" => &[25i64, 22],
            "city" => &["London", "Berlin"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_filter_gt_eq() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (filter (read-csv "data1.csv") (>= $"age" 28))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with '>=' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[2i64, 4, 5],
            "name" => &["Bob", "Diana", "Evan"],
            "age" => &[30i64, 28, 35],
            "city" => &["Paris", "Rome", "Madrid"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_filter_lt_eq() {
        let env = std_env();
        let mut state = State { line: 0 };
    
        let code = r#"
            (filter (read-csv "data1.csv") (<= $"age" 28))
        "#.to_string();
    
        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with '<=' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");
    
        let df = res.collect().expect("failed to collect DataFrame");
    
        let expected_df = df!(
            "id" => &[1i64, 3, 4],
            "name" => &["Alice", "Charlie", "Diana"],
            "age" => &[25i64, 22, 28],
            "city" => &["London", "Berlin", "Rome"],
        ).expect("failed to construct expected DataFrame");
    
        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_filter_and() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (filter (read-csv "data1.csv") (and (> $"age" 22) (< $"age" 35)))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with 'and' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 4],
            "name" => &["Alice", "Bob", "Diana"],
            "age" => &[25i64, 30, 28],
            "city" => &["London", "Paris", "Rome"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_filter_or() {
        let env = std_env();
        let mut state = State { line: 0 };
    
        let code = r#"
            (filter (read-csv "data1.csv") (or (< $"age" 25) (>= $"age" 35)))
        "#.to_string();
    
        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with 'or' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");
    
        let df = res.collect().expect("failed to collect DataFrame");
    
        let expected_df = df!(
            "id" => &[3i64, 5],
            "name" => &["Charlie", "Evan"],
            "age" => &[22i64, 35],
            "city" => &["Berlin", "Madrid"],
        ).expect("failed to construct expected DataFrame");
    
        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_filter_not() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (filter (read-csv "data1.csv") (not (> $"age" 25)))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with 'not' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 3],
            "name" => &["Alice", "Charlie"],
            "age" => &[25i64, 22],
            "city" => &["London", "Berlin"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_select_sum() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (select (read-csv "data1.csv") (sum $"age"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval select with 'sum' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "age" => &[140i64],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_select_mean() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (select (read-csv "data1.csv") (mean $"age"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval select with 'mean' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "age" => &[28.0f64],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_select_median() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (select (read-csv "data1.csv") (median $"age"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval select with 'median' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "age" => &[28.0f64],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_with_column_computed() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (with-column (read-csv "data1.csv") $"is_over_25" (> $"age" 25))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval with-column failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "name" => &["Alice", "Bob", "Charlie", "Diana", "Evan"],
            "age" => &[25i64, 30, 22, 28, 35],
            "city" => &["London", "Paris", "Berlin", "Rome", "Madrid"],
            "is_over_25" => &[false, true, false, true, true],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_sort_asc() {
        let env = std_env();
        let mut state = State { line: 0 };
    
        let code = r#"
            (sort (read-csv "data1.csv") $"age")
        "#.to_string();
    
        let res = eval_string(code, &env, &mut state)
            .expect("eval sort ascending failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");
    
        let df = res.collect().expect("failed to collect DataFrame");
    
        let expected_df = df!(
            "id" => &[3i64, 1, 4, 2, 5],
            "name" => &["Charlie", "Alice", "Diana", "Bob", "Evan"],
            "age" => &[22i64, 25, 28, 30, 35],
            "city" => &["Berlin", "London", "Rome", "Paris", "Madrid"],
        ).expect("failed to construct expected DataFrame");
    
        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_group_by() {
        use crate::interpreter::{eval_string, State};
        use crate::stdlib::std_env;
        use polars::prelude::*;
    
        let env = std_env();
        let mut state = State { line: 0 };
    
        // Group by city and compute mean age, then sort by city to ensure deterministic order
        let code = r#"
            (sort (group-by (read-csv "data1.csv") $"city" (mean $"age")) $"city")
        "#.to_string();
    
        let res = eval_string(code, &env, &mut state)
            .expect("eval group-by failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");
    
        let df = res.collect().expect("failed to collect DataFrame");
    
        let expected_df = df!(
            "city" => &["Berlin", "London", "Madrid", "Paris", "Rome"],
            "age" => &[22.0f64, 25.0, 35.0, 30.0, 28.0],
        ).expect("failed to construct expected DataFrame");
    
        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_agg_multiple() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (agg (read-csv "data1.csv") 
                 (alias (sum $"age") "total_age") 
                 (alias (mean $"age") "mean_age"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval agg failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "total_age" => &[140i64],
            "mean_age" => &[28.0f64],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_join_inner() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (sort (join (read-csv "data1.csv") (read-csv "data2.csv") $"id" $"id" :inner) $"id")
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval inner join failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3],
            "name" => &["Alice", "Bob", "Charlie"],
            "age" => &[25i64, 30, 22],
            "city" => &["London", "Paris", "Berlin"],
            "department" => &["Engineering", "Marketing", "Engineering"],
            "salary" => &[75000i64, 65000, 60000],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_arithmetic_plus() {
        let env = std_env();
        let mut state = State { line: 0 };

        let res = eval_string("(+ 1 2 3 4)".to_string(), &env, &mut state)
            .expect("eval failed")
            .expect_int()
            .expect("expected number");

        assert_eq!(res, 10);
    }

    #[test]
    fn test_arithmetic_minus() {
        let env = std_env();
        let mut state = State { line: 0 };

        let res = eval_string("(- 20 5 3 2)".to_string(), &env, &mut state)
            .expect("eval failed")
            .expect_int()
            .expect("expected number");

        assert_eq!(res, 10);
    }

    #[test]
    fn test_arithmetic_mult() {
        let env = std_env();
        let mut state = State { line: 0 };

        let res = eval_string("(* 2 3 4)".to_string(), &env, &mut state)
            .expect("eval failed")
            .expect_int()
            .expect("expected number");

        assert_eq!(res, 24);
    }

    #[test]
    fn test_arithmetic_divide() {
        let env = std_env();
        let mut state = State { line: 0 };

        let res = eval_string("(/ 100 2 5)".to_string(), &env, &mut state)
            .expect("eval failed")
            .expect_float()
            .expect("expected number");

        assert_eq!(res, 10.0);
    }

    #[test]
    fn test_arithmetic_nested() {
        let env = std_env();
        let mut state = State { line: 0 };

        // Evaluates: ((10 + 5) * 2) - (20 / 4) = (15 * 2) - 5 = 30 - 5 = 25
        let code = "(- (* (+ 10 5) 2) (/ 20 4))".to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval failed")
            .expect_float()
            .expect("expected number");

        assert_eq!(res, 25.0);
    }

    #[test]
    fn test_polars_plus() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (with-column (select (read-csv "data1.csv") $"id" $"age") $"age_plus_5" (+ $"age" 5))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval plus failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "age" => &[25i64, 30, 22, 28, 35],
            "age_plus_5" => &[30i64, 35, 27, 33, 40],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_polars_minus() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (with-column (select (read-csv "data1.csv") $"id" $"age") $"age_minus_2" (- $"age" 2))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval minus failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "age" => &[25i64, 30, 22, 28, 35],
            "age_minus_2" => &[23i64, 28, 20, 26, 33],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_polars_mult() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (with-column (select (read-csv "data1.csv") $"id" $"age") $"age_double" (* $"age" 2))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval multiply failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "age" => &[25i64, 30, 22, 28, 35],
            "age_double" => &[50i64, 60, 44, 56, 70],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_polars_divide() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (with-column (select (read-csv "data1.csv") $"id" $"age") $"age_half" (/ $"age" 2))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval divide failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "age" => &[25i64, 30, 22, 28, 35],
            "age_half" => &[12.5f64, 15.0, 11.0, 14.0, 17.5],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_is_null() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (filter (read-csv "data3.csv") (null? $"age"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with 'null?' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[2i64, 5],
            "name" => &[Some("Bob"), Some("Evan")],
            "age" => &[None::<i64>, None::<i64>],
            "city" => &[Some("Paris"), None],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_is_not_null() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (filter (read-csv "data3.csv") (not-null? $"city"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with 'not-null?' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 4],
            "name" => &[Some("Alice"), Some("Bob"), None],
            "age" => &[Some(25i64), None, Some(28)],
            "city" => &[Some("London"), Some("Paris"), Some("Rome")],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_fill_null() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (with-column (read-csv "data3.csv") $"age" (fill-null $"age" 0))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval with-column fill-null failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "name" => &[Some("Alice"), Some("Bob"), Some("Charlie"), None, Some("Evan")],
            "age" => &[25i64, 0, 22, 28, 0],
            "city" => &[Some("London"), Some("Paris"), None, Some("Rome"), None],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_drop_nulls_all() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (drop-nulls (read-csv "data3.csv"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval drop-nulls failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64],
            "name" => &[Some("Alice")],
            "age" => &[Some(25i64)],
            "city" => &[Some("London")],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_drop_nulls_selected_columns() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (drop-nulls (read-csv "data3.csv") $"name" $"age")
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval drop-nulls with columns failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 3],
            "name" => &[Some("Alice"), Some("Charlie")],
            "age" => &[Some(25i64), Some(22)],
            "city" => &[Some("London"), None],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_when_with_otherwise() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (with-column (read-csv "data1.csv") $"category" (when (>= $"age" 30) "senior" "junior"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval when with otherwise failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "name" => &["Alice", "Bob", "Charlie", "Diana", "Evan"],
            "age" => &[25i64, 30, 22, 28, 35],
            "city" => &["London", "Paris", "Berlin", "Rome", "Madrid"],
            "category" => &["junior", "senior", "junior", "junior", "senior"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_when_without_otherwise() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (with-column (read-csv "data1.csv") $"status" (when (>= $"age" 30) "senior"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval 2-arg when failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "name" => &["Alice", "Bob", "Charlie", "Diana", "Evan"],
            "age" => &[25i64, 30, 22, 28, 35],
            "city" => &["London", "Paris", "Berlin", "Rome", "Madrid"],
            "status" => &[Option::<&str>::None, Some("senior"), None, None, Some("senior")],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_aggregations_min_max_count_nunique_std() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (agg (read-csv "data1.csv")
                 (alias (min $"age") "min_age")
                 (alias (max $"age") "max_age")
                 (alias (count $"age") "count_age")
                 (alias (n-unique $"city") "unique_cities"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval aggregations failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "min_age" => &[22i64],
            "max_age" => &[35i64],
            "count_age" => &[5u32],
            "unique_cities" => &[5u32],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_write_csv_roundtrip() {
        let env = std_env();
        let mut state = State { line: 0 };
        let temp_output = "test_output.csv";

        // Filter and write to CSV
        let write_code = format!(
            r#"(write-csv (filter (read-csv "data1.csv") (= $"age" 30)) "{}")"#,
            temp_output
        );

        eval_string(write_code, &env, &mut state)
            .expect("eval write-csv failed");

        // Read back and assert content matches
        let read_code = format!(r#"(read-csv "{}")"#, temp_output);
        let res = eval_string(read_code, &env, &mut state)
            .expect("eval read-csv on written file failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[2i64],
            "name" => &["Bob"],
            "age" => &[30i64],
            "city" => &["Paris"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);

        // Clean up temporary test file
        let _ = std::fs::remove_file(temp_output);
    }

    #[test]
    fn test_str_contains() {
        let env = std_env();
        let mut state = State { line: 0 };

        // Use a single backslash inside r#"..."#
        let code = r#"
            (filter (read-csv "data4.csv") (str-contains $"email" "@example\.com$"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter with 'str-contains' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 4],
            "name" => &["Alice Johnson", "Diana Prince"],
            "email" => &["alice.j@example.com", "diana.p@example.com"],
            "sku" => &["PROD-101-US", "STAGING-404-US"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }
    
    #[test]
    fn test_str_replace() {
        let env = std_env();
        let mut state = State { line: 0 };

        // Mask the numbers in the SKU column with "XXX"
        let code = r#"
            (with-column (read-csv "data4.csv") $"masked_sku" (str-replace $"sku" "[0-9]+" "XXX"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval with-column 'str-replace' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "name" => &["Alice Johnson", "Bob Smith", "Charlie Brown", "Diana Prince", "Evan Wright"],
            "email" => &[
                "alice.j@example.com",
                "bob_smith@work.org",
                "charlie@test.net",
                "diana.p@example.com",
                "evan_w@domain.io",
            ],
            "sku" => &[
                "PROD-101-US",
                "DEV-202-EU",
                "PROD-303-APAC",
                "STAGING-404-US",
                "DEV-505-LATAM",
            ],
            "masked_sku" => &[
                "PROD-XXX-US",
                "DEV-XXX-EU",
                "PROD-XXX-APAC",
                "STAGING-XXX-US",
                "DEV-XXX-LATAM",
            ],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_str_extract() {
        let env = std_env();
        let mut state = State { line: 0 };

        // Extract the numeric portion (capture group 1) from SKU
        let code = r#"
            (with-column (read-csv "data4.csv") $"sku_code" (str-extract $"sku" ".*-([0-9]+)-.*" 1))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval with-column 'str-extract' failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "name" => &["Alice Johnson", "Bob Smith", "Charlie Brown", "Diana Prince", "Evan Wright"],
            "email" => &[
                "alice.j@example.com",
                "bob_smith@work.org",
                "charlie@test.net",
                "diana.p@example.com",
                "evan_w@domain.io",
            ],
            "sku" => &[
                "PROD-101-US",
                "DEV-202-EU",
                "PROD-303-APAC",
                "STAGING-404-US",
                "DEV-505-LATAM",
            ],
            "sku_code" => &[
                Some("101"),
                Some("202"),
                Some("303"),
                Some("404"),
                Some("505"),
            ],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_window_over_mean() {
        let env = std_env();
        let mut state = State { line: 0 };

        // Compute average age per city over the dataset without grouping
        let code = r#"
            (with-column (read-csv "data1.csv") $"city_mean_age" (over (mean $"age") $"city"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval over mean failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "name" => &["Alice", "Bob", "Charlie", "Diana", "Evan"],
            "age" => &[25i64, 30, 22, 28, 35],
            "city" => &["London", "Paris", "Berlin", "Rome", "Madrid"],
            "city_mean_age" => &[25.0f64, 30.0, 22.0, 28.0, 35.0],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_window_shift_and_cumsum() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (-> (read-csv "data1.csv")
                (with-column $"prev_age" (shift $"age" 1))
                (with-column $"running_age" (cum-sum $"age")))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval shift/cum-sum failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "name" => &["Alice", "Bob", "Charlie", "Diana", "Evan"],
            "age" => &[25i64, 30, 22, 28, 35],
            "city" => &["London", "Paris", "Berlin", "Rome", "Madrid"],
            "prev_age" => &[None::<i64>, Some(25), Some(30), Some(22), Some(28)],
            "running_age" => &[25i64, 55, 77, 105, 140],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_to_date_and_components() {
        let env = std_env();
        let mut state = State { line: 0 };

        // Parse date and extract year, month, and day components
        let code = r#"
            (-> (read-csv "data5.csv")
                (with-column $"date" (to-date $"date_str" "%Y-%m-%d"))
                (with-column $"year" (dt-year $"date"))
                (with-column $"month" (dt-month $"date"))
                (with-column $"day" (dt-day $"date"))
                (select $"id" $"year" $"month" $"day"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval to-date and dt components failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3],
            "year" => &[2024i32, 2025, 2026],
            "month" => &[1i8, 6, 11],
            "day" => &[15i8, 20, 5],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_to_datetime_and_hour() {
        let env = std_env();
        let mut state = State { line: 0 };

        // Parse datetime and extract hour component
        let code = r#"
            (-> (read-csv "data5.csv")
                (with-column $"dt" (to-datetime $"datetime_str" "%Y-%m-%d %H:%M:%S"))
                (with-column $"hour" (dt-hour $"dt"))
                (select $"id" $"hour"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval to-datetime and dt-hour failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3],
            "hour" => &[8i8, 14, 22],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_dt_format() {
        let env = std_env();
        let mut state = State { line: 0 };

        // Parse date and reformat to custom display string
        let code = r#"
            (-> (read-csv "data5.csv")
                (with-column $"date" (to-date $"date_str" "%Y-%m-%d"))
                (with-column $"formatted_date" (dt-format $"date" "%d/%m/%Y"))
                (select $"id" $"formatted_date"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval dt-format failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3],
            "formatted_date" => &["15/01/2024", "20/06/2025", "05/11/2026"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_filter_by_date_component() {
        let env = std_env();
        let mut state = State { line: 0 };

        // Filter rows where year is greater than or equal to 2025
        let code = r#"
            (-> (read-csv "data5.csv")
                (with-column $"date" (to-date $"date_str" "%Y-%m-%d"))
                (filter (>= (dt-year $"date") 2025))
                (select $"id" $"date_str"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval filter by date component failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[2i64, 3],
            "date_str" => &["2025-06-20", "2026-11-05"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_write_and_read_parquet_roundtrip() {
        let env = std_env();
        let mut state = State { line: 0 };
        let temp_parquet = "test_output.parquet";
    
        // 1. Filter CSV data and write to Parquet
        let write_code = format!(
            r#"(write-parquet (filter (read-csv "data1.csv") (= $"city" "Paris")) "{}")"#,
            temp_parquet
        );
    
        eval_string(write_code, &env, &mut state)
            .expect("eval write-parquet failed");
    
        // 2. Read back from Parquet and assert contents
        let read_code = format!(r#"(read-parquet "{}")"#, temp_parquet);
        let res = eval_string(read_code, &env, &mut state)
            .expect("eval read-parquet failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");
    
        let df = res.collect().expect("failed to collect DataFrame");
    
        let expected_df = df!(
            "id" => &[2i64],
            "name" => &["Bob"],
            "age" => &[30i64],
            "city" => &["Paris"],
        ).expect("failed to construct expected DataFrame");
    
        assert_eq!(df, expected_df);
    
        // Clean up temporary test file
        let _ = std::fs::remove_file(temp_parquet);
    }
    
    #[test]
    fn test_parquet_query_pipeline() {
        let env = std_env();
        let mut state = State { line: 0 };
        let parquet_path = "data1.parquet";
    
        // Create fixture data1.parquet using polars directly before querying
        let mut df_fixture = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "department" => &["Engineering", "HR", "Engineering", "Marketing", "HR"],
            "salary" => &[90000i64, 60000, 95000, 70000, 65000],
        ).expect("failed to create fixture DataFrame");
    
        let file = std::fs::File::create(parquet_path).expect("failed to create parquet file");
        ParquetWriter::new(file)
            .finish(&mut df_fixture)
            .expect("failed to write parquet fixture");
    
        // Run query over data1.parquet via MLisp
        let code = r#"
            (-> (read-parquet "data1.parquet")
                (filter (= $"department" "Engineering"))
                (select (alias (mean $"salary") "avg_eng_salary")))
        "#.to_string();
    
        let res = eval_string(code, &env, &mut state)
            .expect("eval read-parquet pipeline failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");
    
        let df = res.collect().expect("failed to collect DataFrame");
    
        let expected_df = df!(
            "avg_eng_salary" => &[92500.0f64],
        ).expect("failed to construct expected DataFrame");
    
        assert_eq!(df, expected_df);
    
        // Clean up fixture file
        let _ = std::fs::remove_file(parquet_path);
    }

    #[test]
    fn test_sink_csv_roundtrip() {
        let env = std_env();
        let mut state = State { line: 0 };
        let temp_output = "test_sink_output.csv";

        // Stream filtered lazy plan directly to CSV
        let sink_code = format!(
            r#"(sink (filter (read-csv "data1.csv") (= $"city" "Paris")) "{}")"#,
            temp_output
        );

        eval_string(sink_code, &env, &mut state)
            .expect("eval sink csv failed");

        // Read back and verify
        let read_code = format!(r#"(read-csv "{}")"#, temp_output);
        let res = eval_string(read_code, &env, &mut state)
            .expect("eval read-csv failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[2i64],
            "name" => &["Bob"],
            "age" => &[30i64],
            "city" => &["Paris"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);

        let _ = std::fs::remove_file(temp_output);
    }

    #[test]
    fn test_sink_parquet_roundtrip() {
        let env = std_env();
        let mut state = State { line: 0 };
        let temp_output = "test_sink_output.parquet";

        // Stream filtered lazy plan directly to Parquet
        let sink_code = format!(
            r#"(sink (filter (read-csv "data1.csv") (> $"age" 28)) "{}")"#,
            temp_output
        );

        eval_string(sink_code, &env, &mut state)
            .expect("eval sink parquet failed");

        // Read back from Parquet and verify
        let read_code = format!(r#"(read-parquet "{}")"#, temp_output);
        let res = eval_string(read_code, &env, &mut state)
            .expect("eval read-parquet failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[2i64, 5],
            "name" => &["Bob", "Evan"],
            "age" => &[30i64, 35],
            "city" => &["Paris", "Madrid"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);

        let _ = std::fs::remove_file(temp_output);
    }

    #[test]
    fn test_pivot_basic() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (sort (pivot (read-csv "data1.csv") $"city" $"name" $"age") $"name")
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval pivot failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "name" => &["Alice", "Bob", "Charlie", "Diana", "Evan"],
            "Berlin" => &[None, None, Some(22i64), None, None],
            "London" => &[Some(25i64), None, None, None, None],
            "Madrid" => &[None, None, None, None, Some(35i64)],
            "Paris" => &[None, Some(30i64), None, None, None],
            "Rome" => &[None, None, None, Some(28i64), None],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_pivot_with_explicit_agg_sum() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (sort (pivot (read-csv "data1.csv") $"city" $"name" $"age" :sum) $"name")
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval pivot with :sum failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        // :sum produces 0i64 for non-matching combinations instead of null
        let expected_df = df!(
            "name" => &["Alice", "Bob", "Charlie", "Diana", "Evan"],
            "Berlin" => &[0i64, 0, 22, 0, 0],
            "London" => &[25i64, 0, 0, 0, 0],
            "Madrid" => &[0i64, 0, 0, 0, 35],
            "Paris" => &[0i64, 30, 0, 0, 0],
            "Rome" => &[0i64, 0, 0, 28, 0],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_pivot_string_col_arguments() {
        let env = std_env();
        let mut state = State { line: 0 };

        // Column names can be strings, but the subsequent sort must use a column expression $"name"
        let code = r#"
            (sort (pivot (read-csv "data1.csv") "city" "name" "age" :first) $"name")
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval pivot with string arguments failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "name" => &["Alice", "Bob", "Charlie", "Diana", "Evan"],
            "Berlin" => &[None, None, Some(22i64), None, None],
            "London" => &[Some(25i64), None, None, None, None],
            "Madrid" => &[None, None, None, None, Some(35i64)],
            "Paris" => &[None, Some(30i64), None, None, None],
            "Rome" => &[None, None, None, Some(28i64), None],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_pivot_pipeline_threading() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (-> (read-csv "data1.csv")
                (pivot $"city" $"id" $"age" :max)
                (sort $"id"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval pipeline pivot failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "Berlin" => &[None, None, Some(22i64), None, None],
            "London" => &[Some(25i64), None, None, None, None],
            "Madrid" => &[None, None, None, None, Some(35i64)],
            "Paris" => &[None, Some(30i64), None, None, None],
            "Rome" => &[None, None, None, Some(28i64), None],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_melt_basic() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (melt (read-csv "data1.csv") (list $"id") (list $"name" $"city"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval melt basic failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5, 1, 2, 3, 4, 5],
            "variable" => &[
                "name", "name", "name", "name", "name",
                "city", "city", "city", "city", "city",
            ],
            "value" => &[
                "Alice", "Bob", "Charlie", "Diana", "Evan",
                "London", "Paris", "Berlin", "Rome", "Madrid",
            ],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_melt_custom_variable_and_value_names() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (melt (read-csv "data1.csv") (list $"id" $"name") (list $"age") "metric" "val")
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval melt with custom column names failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "name" => &["Alice", "Bob", "Charlie", "Diana", "Evan"],
            "metric" => &["age", "age", "age", "age", "age"],
            "val" => &[25i64, 30, 22, 28, 35],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_melt_string_column_arguments() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (melt (read-csv "data1.csv") (list "id") (list "name" "city"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval melt with string arguments failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5, 1, 2, 3, 4, 5],
            "variable" => &[
                "name", "name", "name", "name", "name",
                "city", "city", "city", "city", "city",
            ],
            "value" => &[
                "Alice", "Bob", "Charlie", "Diana", "Evan",
                "London", "Paris", "Berlin", "Rome", "Madrid",
            ],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_unpivot_alias_and_pipeline() {
        let env = std_env();
        let mut state = State { line: 0 };

        let code = r#"
            (-> (read-csv "data1.csv")
                (unpivot (list $"id") (list $"name" $"city") "attr" "val")
                (filter (= $"attr" "city"))
                (sort $"id"))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval unpivot pipeline failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "attr" => &["city", "city", "city", "city", "city"],
            "val" => &["London", "Paris", "Berlin", "Rome", "Madrid"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_concat_vertical_args() {
        let env = std_env();
        let mut state = State { line: 0 };

        // Concatenate two filtered LazyFrames vertically via direct arguments
        let code = r#"
            (concat 
                (filter (read-csv "data1.csv") (< $"age" 25))
                (filter (read-csv "data1.csv") (> $"age" 30)))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval concat vertical args failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[3i64, 5],
            "name" => &["Charlie", "Evan"],
            "age" => &[22i64, 35],
            "city" => &["Berlin", "Madrid"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_concat_vertical_list() {
        let env = std_env();
        let mut state = State { line: 0 };

        // Concatenate a list of LazyFrames vertically
        let code = r#"
            (concat (list 
                (filter (read-csv "data1.csv") (= $"id" 1))
                (filter (read-csv "data1.csv") (= $"id" 2))))
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval concat list failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2],
            "name" => &["Alice", "Bob"],
            "age" => &[25i64, 30],
            "city" => &["London", "Paris"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_concat_horizontal() {
        let env = std_env();
        let mut state = State { line: 0 };

        // Concatenate columns horizontally using the :horizontal strategy
        let code = r#"
            (concat 
                (select (read-csv "data1.csv") $"id" $"name")
                (select (read-csv "data1.csv") $"age" $"city")
                :horizontal)
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval concat horizontal failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5],
            "name" => &["Alice", "Bob", "Charlie", "Diana", "Evan"],
            "age" => &[25i64, 30, 22, 28, 35],
            "city" => &["London", "Paris", "Berlin", "Rome", "Madrid"],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_concat_diagonal() {
        let env = std_env();
        let mut state = State { line: 0 };

        // Diagonal concatenation of frames with non-matching schemas
        let code = r#"
            (concat 
                (select (read-csv "data1.csv") $"id" $"name")
                (select (read-csv "data1.csv") $"id" $"age")
                :diagonal)
        "#.to_string();

        let res = eval_string(code, &env, &mut state)
            .expect("eval concat diagonal failed")
            .expect_lazy_frame()
            .expect("expected lazy frame");

        let df = res.collect().expect("failed to collect DataFrame");

        let expected_df = df!(
            "id" => &[1i64, 2, 3, 4, 5, 1, 2, 3, 4, 5],
            "name" => &[
                Some("Alice"), Some("Bob"), Some("Charlie"), Some("Diana"), Some("Evan"),
                None, None, None, None, None
            ],
            "age" => &[
                None, None, None, None, None,
                Some(25i64), Some(30), Some(22), Some(28), Some(35)
            ],
        ).expect("failed to construct expected DataFrame");

        assert_eq!(df, expected_df);
    }

    #[test]
    fn test_scalar_int() {
        let env = std_env();
        let mut state = State { line: 0 };
    
        // Aggregate to a 1x1 LazyFrame containing the sum of ages (140)
        let code = r#"
            (scalar (agg (read-csv "data1.csv") (sum $"age")))
        "#.to_string();
    
        let res = eval_string(code, &env, &mut state)
            .expect("eval scalar int failed")
            .expect_int()
            .expect("expected int result");
    
        assert_eq!(res, 140);
    }
    
    #[test]
    fn test_scalar_float() {
        let env = std_env();
        let mut state = State { line: 0 };
    
        // Aggregate to a 1x1 LazyFrame containing the mean age (28.0)
        let code = r#"
            (scalar (agg (read-csv "data1.csv") (mean $"age")))
        "#.to_string();
    
        let res = eval_string(code, &env, &mut state)
            .expect("eval scalar float failed")
            .expect_float()
            .expect("expected float result");
    
        assert_eq!(res, 28.0);
    }
    
    #[test]
    fn test_scalar_string() {
        let env = std_env();
        let mut state = State { line: 0 };
    
        // Filter to a single row and select 1 column to get a 1x1 String
        let code = r#"
            (scalar (select (filter (read-csv "data1.csv") (= $"id" 1)) $"name"))
        "#.to_string();
    
        let res = eval_string(code, &env, &mut state)
            .expect("eval scalar string failed")
            .expect_string()
            .expect("expected string result");
    
        assert_eq!(res, "Alice");
    }
    
    #[test]
    fn test_scalar_boolean() {
        let env = std_env();
        let mut state = State { line: 0 };
    
        // Materialize a 1x1 boolean expression
        let code = r#"
            (scalar (select (filter (read-csv "data1.csv") (= $"id" 1)) (= $"name" "Alice")))
        "#.to_string();
    
        let res = eval_string(code, &env, &mut state)
            .expect("eval scalar boolean failed")
            .expect_symbol()
            .expect("expected symbol result");
    
        assert_eq!(res, "true");
    }
    
    #[test]
    fn test_scalar_shape_error() {
        let env = std_env();
        let mut state = State { line: 0 };
    
        // Attempting scalar conversion on a non-1x1 LazyFrame should fail with ScalarShapeError
        let code = r#"
            (scalar (read-csv "data1.csv"))
        "#.to_string();
    
        let res = eval_string(code, &env, &mut state);
        assert!(matches!(res, Err(Error::ScalarShapeError { .. })));
    }
}use std::{collections::HashMap, f32::consts::E};
use std::fs::File;
use polars::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use crate::types::{Env, Error, Expr::{self, PolarsExpr}};
use crate::plotlarslib;

pub fn nil() -> Expr { Expr::Symbol(String::from("nil")) }

fn print(args: Vec<Expr>) -> Result<Expr, Error> {
    for arg in args {
        println!("{}", arg);
    }
    Ok(nil())
}

fn list(args: Vec<Expr>) -> Result<Expr, Error> {
    Ok(Expr::List(args))
}

fn plus(args: Vec<Expr>) -> Result<Expr, Error> {
    let has_polars_expr_arg = args.iter().any(|arg| matches!(arg, Expr::PolarsExpr(_)));
    let has_float_arg = args.iter().any(|arg| matches!(arg, Expr::Float(_)));

    if has_polars_expr_arg {
        let mut expr = args[0].clone().expect_polars_expr()?;
        for arg in args.iter().skip(1) {
            expr = expr + arg.clone().expect_polars_expr()?;
        }
        Ok(Expr::PolarsExpr(expr))
    } else if has_float_arg {
        let mut sum = 0.0;
        for arg in args {
            sum += arg.expect_number()?.to_float();
        }
        Ok(Expr::Float(sum))
    } else {
        let mut sum = 0;
        for arg in args {
            sum += arg.expect_number()?.to_int();
        }
        Ok(Expr::Int(sum))
    }
}

fn minus(args: Vec<Expr>) -> Result<Expr, Error> {
    let has_polars_expr_arg = args.iter().any(|arg| matches!(arg, Expr::PolarsExpr(_)));
    let has_float_arg = args.iter().any(|arg| matches!(arg, Expr::Float(_)));

    if has_polars_expr_arg {
        let mut expr = args[0].clone().expect_polars_expr()?;
        for arg in args.iter().skip(1) {
            expr = expr - arg.clone().expect_polars_expr()?;
        }
        Ok(Expr::PolarsExpr(expr))
    } else if has_float_arg {
        let mut value = args[0].clone().expect_number()?.to_float();
        for arg in args.iter().skip(1) {
            value -= arg.clone().expect_number()?.to_float();
        }
        Ok(Expr::Float(value))
    } else {
        let mut value = args[0].clone().expect_number()?.to_int();
        for arg in args.iter().skip(1) {
            value -= arg.clone().expect_number()?.to_int();
        }
        Ok(Expr::Int(value))
    }
}

fn mult(args: Vec<Expr>) -> Result<Expr, Error> {
    let has_polars_expr_arg = args.iter().any(|arg| matches!(arg, Expr::PolarsExpr(_)));
    let has_float_arg = args.iter().any(|arg| matches!(arg, Expr::Float(_)));

    if has_polars_expr_arg {
        let mut expr = args[0].clone().expect_polars_expr()?;
        for arg in args.iter().skip(1) {
            expr = expr * arg.clone().expect_polars_expr()?;
        }
        Ok(Expr::PolarsExpr(expr))
    } else if has_float_arg {
        let mut value = args[0].clone().expect_number()?.to_float();
        for arg in args.iter().skip(1) {
            value *= arg.clone().expect_number()?.to_float();
        }
        Ok(Expr::Float(value))
    } else {
        let mut value = args[0].clone().expect_number()?.to_int();
        for arg in args.iter().skip(1) {
            value *= arg.clone().expect_number()?.to_int();
        }
        Ok(Expr::Int(value))
    }
}

fn divide(args: Vec<Expr>) -> Result<Expr, Error> {
    let has_polars_expr_arg = args.iter().any(|arg| matches!(arg, Expr::PolarsExpr(_)));
    // let has_float_arg = args.iter().any(|arg| matches!(arg, Expr::Float(_)));
    // Always compute real division in f64

    if has_polars_expr_arg {
        // Cast the initial expression to Float64 so division produces f64
        let mut expr = args[0].clone().expect_polars_expr()?.cast(DataType::Float64);
        for arg in args.iter().skip(1) {
            let next_expr = arg.clone().expect_polars_expr()?.cast(DataType::Float64);
            expr = expr / next_expr;
        }
        Ok(Expr::PolarsExpr(expr))
    } else {
        let mut value = args[0].clone().expect_number()?.to_float();
        for arg in args.iter().skip(1) {
            value /= arg.clone().expect_number()?.to_float();
        }
        Ok(Expr::Float(value))
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
        Ok(lf) => Ok(Expr::LazyFrame(lf)),
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
    Ok(Expr::LazyFrame(lf.limit(10)))
}

fn tail(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    Ok(Expr::LazyFrame(lf.slice(-10, 10)))
}

fn slice(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let offset = args[1].clone().expect_int()?;
    let len = args[2].clone().expect_int()?;
    Ok(Expr::LazyFrame(lf.slice(offset, len as u32)))
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
    Ok(Expr::List(vec![Expr::Int(shape.0 as i64), Expr::Int(shape.1 as i64)]))
}

fn col_(args: Vec<Expr>) -> Result<Expr, Error> {
    Ok(Expr::PolarsExpr(col(args[0].clone().expect_string()?)))
}

fn lit_(args: Vec<Expr>) -> Result<Expr, Error> {
    match args[0].clone() {
        Expr::Int(num) => Ok(Expr::PolarsExpr(lit(num))),
        Expr::Float(num) => Ok(Expr::PolarsExpr(lit(num))),
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
        args[1].clone().expect_int()? as u8
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

// (str-contains $"name" "^A.*")
fn str_contains(args: Vec<Expr>) -> Result<Expr, Error> {
    if args.len() < 2 {
        return Err(Error::WrongType {
            expected: "2 arguments (column, pattern)".into(),
            got: Expr::List(args),
        });
    }
    let col_expr = args[0].clone().expect_polars_expr()?;
    let pat_expr = args[1].clone().expect_polars_expr()?;
    
    // Polars: expr.str().contains(pattern, strict)
    Ok(Expr::PolarsExpr(col_expr.str().contains(pat_expr, true)))
}

// (str-replace $"city" "^Lon" "New-Lon")
fn str_replace(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let pat_expr = args[1].clone().expect_polars_expr()?;
    let rep_expr = args[2].clone().expect_polars_expr()?;

    // false = interpret pattern as regex, not literal
    Ok(Expr::PolarsExpr(col_expr.str().replace(pat_expr, rep_expr, false)))
}

// (str-extract $"code" "([A-Z]+)-(\\d+)" 1)
fn str_extract(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let pat_expr = args[1].clone().expect_polars_expr()?;
    let group_idx = args[2].clone().expect_int()? as usize;

    Ok(Expr::PolarsExpr(col_expr.str().extract(pat_expr, group_idx)))
}

// (over <expr> <partition_col1> ...) or (over <expr> (<col1> <col2>))
fn over(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;

    let partition_by = match &args[1] {
        Expr::List(cols) => {
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
        Ok(result) => Ok(Expr::PolarsExpr(result)),
        Err(err) => Err(Error::PolarsError(err)),
    } 
}

// (shift $"col" 1) or (shift $"col" -1)
fn shift(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let offset = args[1].clone().expect_int()? as i64;
    Ok(Expr::PolarsExpr(expr.shift(lit(offset))))
}

// (cum-sum $"col")
fn cum_sum(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let reverse = if args.len() > 1 {
        matches!(args[1].clone().expect_symbol()?.as_str(), ":desc" | ":reverse")
    } else {
        false
    };
    
    Ok(Expr::PolarsExpr(expr.cum_sum(reverse)))
}

// (cum-min $"col")
fn cum_min(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let reverse = if args.len() > 1 {
        matches!(args[1].clone().expect_symbol()?.as_str(), ":desc" | ":reverse")
    } else {
        false
    };
    Ok(Expr::PolarsExpr(expr.cum_min(reverse)))
}

// (cum-max $"col")
fn cum_max(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let reverse = if args.len() > 1 {
        matches!(args[1].clone().expect_symbol()?.as_str(), ":desc" | ":reverse")
    } else {
        false
    };
    Ok(Expr::PolarsExpr(expr.cum_max(reverse)))
}

// (cum-count $"col")
fn cum_count(args: Vec<Expr>) -> Result<Expr, Error> {
    let expr = args[0].clone().expect_polars_expr()?;
    let reverse = if args.len() > 1 {
        matches!(args[1].clone().expect_symbol()?.as_str(), ":desc" | ":reverse")
    } else {
        false
    };
    Ok(Expr::PolarsExpr(expr.cum_count(reverse)))
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

    Ok(Expr::PolarsExpr(expr.rank(options, None)))
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

    Ok(Expr::PolarsExpr(col_expr.str().to_date(options)))
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
    Ok(Expr::PolarsExpr(col_expr.str().to_datetime(
        Some(TimeUnit::Microseconds),
        None,
        options,
        polars::prelude::lit("raise"),
    )))
}

// (dt-year $"date_col")
fn dt_year(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    Ok(Expr::PolarsExpr(col_expr.dt().year()))
}

// (dt-month $"date_col")
fn dt_month(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    Ok(Expr::PolarsExpr(col_expr.dt().month()))
}

// (dt-day $"date_col")
fn dt_day(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    Ok(Expr::PolarsExpr(col_expr.dt().day()))
}

// (dt-hour $"datetime_col")
fn dt_hour(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    Ok(Expr::PolarsExpr(col_expr.dt().hour()))
}

// (dt-format $"date_col" "%B %d, %Y")
fn dt_format(args: Vec<Expr>) -> Result<Expr, Error> {
    let col_expr = args[0].clone().expect_polars_expr()?;
    let format = args[1].clone().expect_string()?;
    Ok(Expr::PolarsExpr(col_expr.dt().to_string(&format)))
}

// Helper to extract a column name string from String, Symbol, or $col Expr
fn extract_col_name(expr: &Expr) -> Result<String, Error> {
    match expr {
        Expr::String(s) | Expr::Symbol(s) => Ok(s.clone()),
        Expr::PolarsExpr(polars::prelude::Expr::Column(name)) => Ok(name.to_string()),
        other => Err(Error::WrongType {
            expected: String::from("column name or $col"),
            got: other.clone(),
        }),
    }
}

// Helper to extract a Vec<String> from either a single item or Expr::List
fn extract_col_names(arg: &Expr) -> Result<Vec<String>, Error> {
    match arg {
        Expr::List(list) => list.iter().map(extract_col_name).collect(),
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

    Ok(Expr::LazyFrame(pivoted))
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

    Ok(Expr::LazyFrame(lf.unpivot(unpivot_args)))
}

fn concat_(args: Vec<Expr>) -> Result<Expr, Error> {
    let mut frames = Vec::<LazyFrame>::new();
    let mut how_str: Option<String> = None;

    let items = match &args[0] {
        Expr::List(list) => list.as_slice(),
        _ => &args[..],
    };

    for item in items {
        match item {
            Expr::LazyFrame(lf) => frames.push(lf.clone()),
            Expr::Symbol(s) if s.starts_with(':') => how_str = Some(s.clone()),
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

    Ok(Expr::LazyFrame(concatted))
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
    Ok(Expr::PolarsExpr(col_expr.cast(target_type)))
}

// (scalar expr) — materialize a 1x1 result into a Lisp scalar
fn scalar(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    let df = lf.collect().map_err(Error::PolarsError)?;
    if df.shape() != (1, 1) { return Err(Error::ScalarShapeError { shape: df.shape() }) }
    let row = df.get(0).unwrap();
    match row[0].clone() {
        AnyValue::Null => Ok(nil()),
        AnyValue::Boolean(b) => if b { Ok(Expr::Symbol("true".to_string())) } else { Ok(Expr::Symbol("false".to_string())) },
        AnyValue::String(s) => Ok(Expr::String(s.to_string())),
        AnyValue::UInt8(i) => Ok(Expr::Int(i.into())),
        AnyValue::UInt16(i) => Ok(Expr::Int(i.into())),
        AnyValue::UInt32(i) => Ok(Expr::Int(i.into())),
        AnyValue::UInt64(i) => Ok(Expr::Int(i as i64)),
        AnyValue::UInt128(i) => Ok(Expr::Int(i as i64)),
        AnyValue::Int8(i) => Ok(Expr::Int(i as i64)),
        AnyValue::Int16(i) => Ok(Expr::Int(i as i64)),
        AnyValue::Int32(i) => Ok(Expr::Int(i as i64)),
        AnyValue::Int64(i) => Ok(Expr::Int(i as i64)),
        AnyValue::Int128(i) => Ok(Expr::Int(i as i64)),
        AnyValue::Float16(f) => Ok(Expr::Float(f.into())),
        AnyValue::Float32(f) => Ok(Expr::Float(f.into())),
        AnyValue::Float64(f) => Ok(Expr::Float(f.into())),
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
        AnyValue::StringOwned(pl_small_str) => Ok(Expr::String(pl_small_str.to_string())),
        AnyValue::Binary(items) => Err(Error::ScalarUnsupportedError { name: "Binary".to_string() }),
        AnyValue::BinaryOwned(items) => Err(Error::ScalarUnsupportedError { name: "BinaryOwned".to_string() }),
        _ => Err(Error::ScalarUnsupportedError { name: "Unknown".to_string() })
    }
}

fn collect_(args: Vec<Expr>) -> Result<Expr, Error> {
    let lf = args[0].clone().expect_lazy_frame()?;
    lf.collect().unwrap();
    Ok(nil())
}

pub fn ask(args: Vec<Expr>) -> Result<Expr, Error> {
    let question = args[0].clone().expect_string()?;
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let answer = runtime.block_on(crate::ai::ask_ai(&question)).unwrap();
    Ok(Expr::String(answer))
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
    insert_native_function(&mut defines, "list", list, 0, u8::MAX);

    // AI
    insert_native_function(&mut defines, "ask", ask, 1, 1);

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
use std::collections::HashMap;
use std::fmt;
use std::cell::RefCell;
use std::rc::Rc;
use polars::prelude::*;

#[derive(Clone)]
pub enum Expr {
    Symbol(String),
    Int(i64),
    Float(f64),
    String(String),
    List(Vec<Expr>),
    NativeFunction{ body: fn(Vec<Expr>) -> Result<Expr, Error>, min_args: u8, max_args: u8},
    UserFunction{ body: Vec<Expr>, params: Vec<String>, name: Option<String> },
    LazyFrame(LazyFrame),
    PolarsExpr(polars::prelude::Expr),
    SourceInfo(Box<Expr>, usize)
}

pub enum Number {
    Int(i64),
    Float(f64)
}

impl Expr {
    pub fn expect_symbol(self) -> Result<String, Error> {
        match self {
            Expr::Symbol(s) => Ok(s),
            other => Err(Error::WrongType { expected: String::from("symbol"), got: other }),
        }
    }

    pub fn expect_number(self) -> Result<Number, Error> {
        match self {
            Expr::Int(n) => Ok(Number::Int(n)),
            Expr::Float(n) => Ok(Number::Float(n)),
            other => Err(Error::WrongType { expected: String::from("number"), got: other }),
        }
    }

    pub fn expect_int(self) -> Result<i64, Error> {
        match self {
            Expr::Int(i) => Ok(i),
            other => Err(Error::WrongType { expected: String::from("int"), got: other }),
        }
    }

    pub fn expect_float(self) -> Result<f64, Error> {
        match self {
            Expr::Float(f) => Ok(f),
            other => Err(Error::WrongType { expected: String::from("float"), got: other }),
        }
    }


    pub fn expect_string(self) -> Result<String, Error> {
        match self {
            Expr::String(s) => Ok(s),
            other => Err(Error::WrongType { expected: String::from("string"), got: other }),
        }
    }
    
    pub fn expect_list(self) -> Result<Vec<Expr>, Error> {
        match self {
            Expr::List(l) => Ok(l),
            other => Err(Error::WrongType { expected: String::from("list"), got: other }),
        }
    }
    
    pub fn expect_lazy_frame(self) -> Result<LazyFrame, Error> {
        match self {
            Expr::LazyFrame(lf) => Ok(lf),
            other => Err(Error::WrongType { expected: String::from("lazy frame"), got: other }),
        }
    } 

    pub fn expect_polars_expr(self) -> Result<polars::prelude::Expr, Error> {
        match self {
            Expr::PolarsExpr(pe) => Ok(pe),
            Expr::Int(i) => Ok(polars::prelude::lit(i)),
            Expr::Float(f) => Ok(polars::prelude::lit(f)),
            Expr::String(str) => Ok(polars::prelude::lit(str)),
            other => Err(Error::WrongType { expected: String::from("polars expr"), got: other }),
        }
    }

    pub fn type_to_str(self) -> &'static str {
        match self {
            Expr::Symbol(_) => "symbol",
            Expr::Int(_) => "int",
            Expr::Float(_) => "float",
            Expr::String(_) => "string",
            Expr::List(_) => "list",
            Expr::NativeFunction { body, min_args, max_args } => "native function",
            Expr::UserFunction { body, params, name } => "user function",
            Expr::LazyFrame(_) => "lazy frame",
            Expr::PolarsExpr(_) => "polars expr",
            Expr::SourceInfo(_, _) => "source info",
        }
    }

    pub fn to_string(self) -> Result<String, Error> {
        match self {
            Expr::LazyFrame(lf) => {
                let df = lf.clone().collect().map_err(Error::PolarsError)?;
                Ok(format!("{}", df))
            }
            // For others, fall back to standard Display or format manually
            other => Ok(format!("{}", other)),
        }
    }
}

impl Number {
    pub fn to_int(self) -> i64 {
        match self {
            Number::Int(i) => i,
            Number::Float(f) => f as i64,
        }
    }

    pub fn to_float(self) -> f64 {
        match self {
            Number::Int(i) => i as f64,
            Number::Float(f) => f,
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Symbol(s) => write!(f, "{s}"),
            Expr::Int(n) => write!(f, "{n}"),
            Expr::Float(n) => write!(f, "{n}"),
            Expr::String(s) => Ok(termimad::print_text(s)),
            Expr::List(items) => {
                write!(f, "(")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, ")")
            },
            Expr::NativeFunction{ body, min_args, max_args } => write!(f, "<NativeFunction>"),
            Expr::UserFunction { body, params, name } => {
                match name {
                    Some(name) => write!(f, "<{name}>"),
                    None => write!(f, "<lambda>"),
                }
            }
            Expr::LazyFrame(lf) => match lf.clone().collect() {
                Ok(df) => write!(f, "{df}"),
                Err(err) => write!(f, "<Failed to collect LazyFrame: {err}>"),
            },
            Expr::PolarsExpr(pe) => write!(f, "{}", pe),
            Expr::SourceInfo(expr, line) => write!(f, "<SourceInfo>")
        }
    }
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Symbol(s) => f.debug_tuple("Symbol").field(s).finish(),
            Expr::Int(n) => f.debug_tuple("Number").field(n).finish(),
            Expr::Float(n) => f.debug_tuple("Number").field(n).finish(),
            Expr::String(s) => f.debug_tuple("String").field(s).finish(),
            Expr::List(items) => f.debug_tuple("List").field(items).finish(),
            Expr::NativeFunction{ body, min_args, max_args } => { f.write_str("NativeFunction") },
            Expr::UserFunction { body, params, name } => { f.write_str("UserFunction") },
            Expr::LazyFrame(_) => { f.write_str("LazyFrame") },
            Expr::PolarsExpr(_) => { f.write_str("PolarsExpr") },
            Expr::SourceInfo(expr, line) => { f.write_str("SourceInfo") }
        }
    }
}

pub struct Env {
    pub defines: HashMap<String, Expr>,
    pub outer: Option<Rc<RefCell<Env>>>,
}

impl Env {
    pub fn new() -> Self {
        Self {
            defines: HashMap::new(),
            outer: None,
        }
    }

    pub fn with_parent(outer: Rc<RefCell<Env>>) -> Self {
        Self {
            defines: HashMap::new(),
            outer: Some(outer),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Error {
    UnclosedString,
    UnclosedListExpr,
    FailedToParseNumber {
        number: String
    },
    WrongType {
        expected: String,
        got: Expr
    },
    LookupNotFound {
        symbol: String
    },
    WrongNumberOfArguments {
        expected_min: u8,
        expected_max: u8,
        got: u8
    },
    ScalarShapeError{
        shape: (usize, usize)
    },
    ScalarUnsupportedError{
        name: String
    },
    MissingRequiredKWArg{
        name: String
    },
    Custom(String),
    PolarsError(polars::prelude::PolarsError)
}

pub fn format_error(error: Error) -> String {
    match error {
        Error::UnclosedString => format!("Error: unclosed string"),
        Error::UnclosedListExpr => format!("Error: unclosed list expression"),
        Error::FailedToParseNumber { number } => format!("Error: failed to parse number '{number}'"),
        Error::WrongType { expected, got } => format!("Error: expected {0}, got {1}", expected, got.type_to_str()),
        Error::LookupNotFound { symbol } => format!("Error: symbol '{symbol}' is not defined"),
        Error::WrongNumberOfArguments { 
            expected_min, expected_max, got 
        } => format!("Error: expected {expected_min}-{expected_max} number of argumens, got {got}"),
        Error::ScalarShapeError { shape } => format!("Error: failed to convert to lisp expr, shape is {} by {}", shape.0, shape.1),
        Error::ScalarUnsupportedError { name } => format!("Error: unsupported scalar conversion from type '{name}'"),
        Error::MissingRequiredKWArg { name } => format!("Error: missing required keyword argument '{name}'"),
        Error::Custom(message) => format!("Error: {message}"),
        Error::PolarsError(perror) => format!("Error: {}", perror)
    }
}
