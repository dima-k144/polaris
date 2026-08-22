use reqwest::Client;
use serde_json::json;
use std::cell::RefCell;
use std::rc::Rc;
use crate::types::Env;
use crate::types::*;
use polars::prelude::LazyFrame;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use serde::Deserialize;

static LANGUAGE_SPEC: &str = r#"
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

// Helper to strip markdown formatting if the model wraps output in ```lisp ... ```
fn extract_lisp_code(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.starts_with("```") {
        let lines: Vec<&str> = trimmed.lines().collect();
        if lines.len() >= 2 {
            let start = 1;
            let end = if lines.last().map_or(false, |l| l.starts_with("```")) {
                lines.len() - 1
            } else {
                lines.len()
            };
            return lines[start..end].join("\n").trim().to_string();
        }
    }
    trimmed.to_string()
}

pub fn collect_context(env: &Rc<RefCell<Env>>) -> String {
    let mut context = String::from("Here's the context about user's current environment with variables\nUser env: {");
    for (name, value) in env.borrow().defines.iter() {
        match value {
            Expr::LazyFrame(lf) => {
                let lf_info = lf.clone().collect_schema().unwrap();
                context.push_str(&format!("{name}: LazyFrame {{ {:?} }}", lf_info));
            },
            Expr::NativeFunction { body, min_args, max_args } => {

            },
            _ => { 
                if name == "nil" { continue; }
                context.push_str(&format!("{name}: {value}"))
            }
        }
    }
    context
}

pub async fn ask_ai(prompt: &str, env: &Rc<RefCell<Env>>) -> Result<String, Box<dyn std::error::Error>> {
    // 1. Create and configure spinner
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", " "]),
    );
    pb.set_message("AI is thinking...");
    pb.enable_steady_tick(Duration::from_millis(80));

    let api_key = String::from("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");
    let model = "qwen/qwen3.8-27b";
    
    // 2. Perform the async network request
    let response_res = Client::new()
        .post("https://openrouter.ai/api/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&json!({
            "model": model,
            "messages": [
                { "role": "system", "content": LANGUAGE_SPEC },
                { "role": "system", "content": collect_context(env) },
                { "role": "user", "content": prompt }
            ]
        }))
        .send()
        .await;

    let response = match response_res {
        Ok(res) => res,
        Err(err) => {
            pb.finish_and_clear();
            return Err(Box::new(err));
        }
    };

    let json: serde_json::Value = response.json().await?;

    // 3. Clear or finish the spinner once complete
    pb.finish_and_clear();

    Ok(json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string())
}

pub async fn generate_mlisp_code(
    user_query: &str,
    env: &Rc<RefCell<Env>>,
) -> Result<String, Error> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", " "]),
    );
    pb.set_message("Generating MLisp code...");
    pb.enable_steady_tick(Duration::from_millis(80));

    let api_key = std::env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| {
        "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".to_string()
    });

    let system_instructions = r#"
You are the MLisp Query Engine.
Generate ONLY valid MLisp expressions based on the user's environment and request.

RULES:
1. Return ONLY pure MLisp code.
2. DO NOT wrap in markdown backticks (no ```lisp).
3. DO NOT include explanations, comments, or prose.
4. Use thread-first pipelines (-> ...) when chaining transformations.
5. Use column sugar ($"col_name" or $col_name).

Below is language specification
"#;

    let context = collect_context(env);
    let model = "qwen/qwen3.8-27b";

    let client = Client::new();
    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&json!({
            "model": model,
            "messages": [
                { "role": "system", "content": system_instructions },
                { "role": "system", "content": LANGUAGE_SPEC},
                { "role": "system", "content": context },
                { "role": "user", "content": user_query }
            ],
            "temperature": 0.1
        }))
        .send()
        .await
        .map_err(|e| {
            pb.finish_and_clear();
            Error::Custom(format!("AI network error: {e}"))
        })?;

    let json_resp: serde_json::Value = response
        .json()
        .await
        .map_err(|e| {
            pb.finish_and_clear();
            Error::Custom(format!("Failed to parse AI response JSON: {e}"))
        })?;

    pb.finish_and_clear();

    let raw_code = json_resp["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let cleaned_code = extract_lisp_code(&raw_code);
    Ok(cleaned_code)
}

#[derive(Deserialize)]
pub struct FixResponse {
    pub explanation: String,
    pub fixed_code: String,
}

pub async fn suggest_fix(
    error_msg: &str,
    input_code: &str,
    env: &Rc<RefCell<Env>>,
) -> Result<FixResponse, Box<dyn std::error::Error>> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", " "]),
    );
    pb.set_message("AI is analyzing error and generating fix...");
    pb.enable_steady_tick(Duration::from_millis(80));

    let api_key = std::env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| {
        "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".to_string()
    });

    let system_instructions = r#"
You are the MLisp Error Recovery Agent.
Analyze the user's MLisp code and the encountered error.
Provide a clear, brief explanation and the corrected MLisp code.

Respond ONLY with valid JSON matching this schema:
{
  "explanation": "1-2 sentence explanation of the bug and fix",
  "fixed_code": "pure MLisp expression without markdown formatting or code blocks"
}
"#;

    let user_prompt = format!(
        "Encountered Error:\n{}\n\nFailing Code:\n{}",
        error_msg, input_code
    );

    let client = Client::new();
    let response = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&json!({
            "model": "qwen/qwen3.8-27b",
            "messages": [
                { "role": "system", "content": system_instructions },
                { "role": "system", "content": LANGUAGE_SPEC },
                { "role": "system", "content": collect_context(env) },
                { "role": "user", "content": user_prompt }
            ],
            "response_format": { "type": "json_object" },
            "temperature": 0.1
        }))
        .send()
        .await;

    let response = match response {
        Ok(res) => res,
        Err(err) => {
            pb.finish_and_clear();
            return Err(Box::new(err));
        }
    };

    let json_resp: serde_json::Value = response.json().await?;
    pb.finish_and_clear();

    let content = json_resp["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("{}");

    let parsed: FixResponse = serde_json::from_str(content)?;
    Ok(parsed)
}
