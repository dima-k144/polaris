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
}