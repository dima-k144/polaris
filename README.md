# Polaris

**A Lisp-like language for data analysis, written in Rust.**

Polaris combines indentation-based syntax with traditional Lisp expressions. It uses Polars for dataframe queries and Plotlars for visualization, so you can filter, transform, aggregate, and plot data in the same language.

```lisp
-> (read-csv "sales.csv")
    with-column $revenue {$quantity * $price}
    group-by $category
        alias (sum $revenue) "total_revenue"
    sort $total_revenue :desc
```

## Features

- Indentation-based syntax inspired by Wisp, with support for parenthesized Lisp expressions.
- Inline infix expressions, column shorthand, and a thread-first pipeline form.
- Integers, floating-point numbers, strings, booleans, symbols, lists, and functions.
- Named and anonymous functions, local bindings, conditionals, loops, and higher-order functions.
- Polars lazy queries with CSV and Parquet input/output.
- Grouped aggregations, joins, window functions, rolling statistics, string processing, and date/time operations.
- Scatter, line, bar, pie, histogram, box, and time-series plots through Plotlars.
- An interactive REPL with command history, plus `.lsp` script execution.

## Installation

Download a prebuilt executable from this repository's **Releases** page (available in the repository's right sidebar). Open the release and expand **Assets**, then download the archive for your operating system. The automatically generated **Source code** archives contain source files, not prebuilt executables.

Rust and Cargo are not required to run the prebuilt executable. Download only builds provided by the release; macOS binaries may not be available.

### Linux (x86-64)

Download `polaris-linux-x86_64.tar.gz`, open a terminal in the download directory, and run:

```bash
tar -xzf polaris-linux-x86_64.tar.gz
chmod +x polaris
./polaris
```

To run Polaris from any directory, install it for your user and add its directory to your current shell's `PATH`:

```bash
mkdir -p "$HOME/.local/bin"
install -m 755 polaris "$HOME/.local/bin/polaris"
export PATH="$HOME/.local/bin:$PATH"
```

To keep this `PATH` setting in future Bash sessions, add the `export` line to `~/.bashrc` (or the corresponding configuration file for your shell).

Run a script from the directory containing it:

```bash
polaris analysis.lsp
```

Linux builds use system libraries, so compatibility with every distribution or older Linux version is not guaranteed.

### Windows (x86-64)

Download `polaris-windows-x86_64.zip` and extract the entire archive. Keep any included DLLs alongside `polaris.exe`.

Open PowerShell in the extracted directory and start the REPL:

```powershell
.\polaris.exe
```

To run a script, place it in that directory and use:

```powershell
.\polaris.exe analysis.lsp
```

Alternatively, pass the script's full path in quotes. If Windows reports a missing DLL, check that you extracted all files supplied with the release.

## Build from source

You need Rust, Cargo, and a native build toolchain. The enabled Polars `nightly` feature requires a nightly Rust toolchain. Use the repository's Rust toolchain configuration if one is provided. Dependency versions and enabled Polars features are defined in `Cargo.toml`.

From the repository root:

```bash
cargo build --release
```

The executable is written to `target/release/polaris` on Linux/macOS or `target/release/polaris.exe` on Windows.

Start the REPL or run a script through Cargo:

```bash
cargo run --release
cargo run --release -- analysis.lsp
```

## Usage

The interpreter recognizes script arguments ending in `.lsp`. Multiple scripts can be passed in one command; each runs in a fresh environment. Use `(load "helpers.lsp")` to evaluate another file in the current environment.

The REPL prints the result of each submitted expression. Script execution prints the final result; use `print` for intermediate output. REPL history is stored in `~/.polaris_history`. Press **Ctrl+D** to exit.

For multiline REPL input, use explicit parentheses, brackets, or braces. Indentation alone does not keep the prompt open for more lines.

## Quick start: analyze a CSV file

Save this as `sales.csv`:

```csv
category,product,quantity,price
Food,Bread,10,25
Food,Milk,5,40
Stationery,Notebook,3,60
Stationery,Pen,10,15
```

Save the following as `analysis.lsp` in the same directory:

```lisp
define sales : read-csv "sales.csv"

define summary
    -> sales
        with-column $revenue {$quantity * $price}
        group-by $category
            alias (sum $revenue) "total_revenue"
        sort $total_revenue :desc

print summary
write-csv summary "summary.csv"
```

Run it from that directory using the installed Linux executable:

```bash
polaris analysis.lsp
```

On Windows, with `polaris.exe` in the same directory, use `.\polaris.exe analysis.lsp` in PowerShell. When building from source, use `cargo run --release -- analysis.lsp` from the repository root, with the example files in that directory.

The script displays revenue by category and writes `summary.csv`:

| category | total_revenue |
| --- | ---: |
| Food | 450 |
| Stationery | 330 |

## Syntax

### Calls and indentation

Traditional Lisp calls and indentation-based calls are both supported:

```lisp
(print (+ 2 3))
```

```lisp
print
    + 2 3
```

Both print `5`. Each ordinary indentation-based line becomes a call, and indented expressions become arguments of the parent expression. Use consistent indentation; the examples here use four spaces.

A standalone `:` surrounded by whitespace nests the rest of the line:

```lisp
print : + 2 3
```

This is equivalent to `(print (+ 2 3))`. Keywords such as `:desc` remain symbols and do not introduce nesting.

A leading `.` on an indented line supplies arguments directly to the enclosing expression instead of wrapping them in another call:

```lisp
if (> 5 3)
    . "yes"
    . "no"
```

This is equivalent to `(if (> 5 3) "yes" "no")`.

Comments start with `;` and continue to the end of the line.

### Values and lists

```lisp
(define amount 42)
(define rate 0.25)
(define message "Hello, Polaris!")
(define enabled true)
(define missing nil)
(define numbers '(1 2 3 4))

(first numbers)                 ; 1
(rest numbers)                  ; (2 3 4)
(len numbers)                   ; 4
(list 1 (+ 1 1) 3)              ; (1 2 3)
```

Quotation (`'`) keeps an expression as data. Square brackets are an alternative to parentheses; they do not automatically quote a list.

### Infix expressions

```lisp
(print {2 + 3})                 ; 5
(print {1 + 2 * 3})             ; 9
(print {1 + {2 * 3}})           ; 7
```

Operators inside braces evaluate **left to right**, without conventional arithmetic precedence. Use nested braces or prefix calls to control grouping.

### Functions and control flow

```lisp
(defun square (x)
    (* x x))

(square 5)                      ; 25
(map #square '(1 2 3))          ; (1 4 9)
(map (lambda (x) (+ x 10)) '(1 2 3))
(filter (lambda (x) (> x 2)) '(1 2 3 4))
(reduce #+ '(1 2 3 4))          ; 10

(let (x 10 y 20)
    (+ x y))                    ; 30

(if (> 5 3) "yes" "no")

(define total 0)
(for i 0 5
    (set total (+ total i)))
(print total)                   ; 10

(for-each item '("a" "b" "c")
    (print item))
```

`#name` looks up a function or another binding, making function references explicit in calls such as `map` and `reduce`. Functions can contain multiple body expressions and return the last result.

`let` takes a flat list of name/value pairs. Binding values are evaluated in the surrounding environment. `if` requires a boolean condition. `for` excludes its end value and accepts an optional integer step after the bounds.

### Pipelines

`->` inserts the previous result as the first argument of each subsequent call:

```lisp
-> (range 5)
    reverse
    first
```

This is equivalent to `(first (reverse (range 5)))`, which returns `4`.

## Data analysis

`$name` is shorthand for `(col "name")`. Column expressions describe computations on data; they are distinct from ordinary scalar values.

```lisp
define people : read-csv "people.csv"

-> people
    filter : and (>= $age 18) (= $city "Vinnytsia")
    with-column $age_next_year {$age + 1}
    select $name $age_next_year
    sort $age_next_year :desc
```

`group-by` accepts a grouping expression followed by aggregation expressions:

```lisp
-> people
    group-by $city
        alias (mean $age) "average_age"
        alias (count) "people_count"
```

Use `list` for multiple evaluated grouping columns, for example `(group-by people (list $city $department) (mean $salary))` when those columns exist.

### Lazy execution

Dataframe transformations generally build a Polars lazy query plan. Displaying a LazyFrame, writing a file, plotting, or converting a result with `scalar` executes the query. Some operations may also execute data internally.

```lisp
(explain people)                        ; inspect the query plan
(collect people)                        ; execute and return nil
(scalar (select people (mean $age)))     ; extract a single value
(write-parquet people "people.parquet")
```

`collect` currently executes a query and returns `nil`; it does not return a materialized dataframe. `scalar` requires a result containing exactly one row and one column. Displaying or writing the same LazyFrame again may execute its plan again.

### Available operations

The standard library exposes a subset of Polars through language-level functions.

| Area | Functions |
| --- | --- |
| Files | `read-csv`, `write-csv`, `read-parquet`, `write-parquet`, `sink` |
| Inspection | `head`, `tail`, `slice`, `schema`, `shape`, `describe`, `explain` |
| Transformations | `select`, `filter`, `with-column`, `sort`, `drop`, `rename`, `unique`, `cast` |
| Grouping and joins | `group-by`, `agg`, `alias`, `join` |
| Statistics | `sum`, `mean`, `median`, `count`, `min`, `max`, `n-unique`, `std`, `var`, `quantile`, `cov`, `corr` |
| Nulls and conditions | `null?`, `not-null?`, `fill-null`, `drop-nulls`, `when` |
| Column strings | `str-contains`, `str-replace`, `str-extract`, `str-split`, `str-strip`, `str-strip-start`, `str-strip-end`, `str-slice`, `str-extract-groups` |
| Windows | `over`, `shift`, `rank`, `cum-sum`, `cum-min`, `cum-max`, `cum-count` |
| Rolling statistics | `rolling-mean`, `rolling-sum`, `rolling-min`, `rolling-max`, `rolling-median`, `rolling-std` |
| Dates and times | `to-date`, `to-datetime`, `dt-year`, `dt-month`, `dt-day`, `dt-hour`, `dt-format` |
| Reshaping | `pivot`, `melt`, `unpivot`, `concat` |
| Expressions and execution | `col`, `lit`, `slice-expr`, `get`, `apply-udf`, `collect`, `scalar` |

For example, `with-column` takes a frame followed by column-name/expression pairs, while `head` and `tail` each take only a frame. Use `slice` for an explicit offset and row count.

## Visualization

Plot functions accept a LazyFrame and keyword arguments. Column names in plot options are strings:

```lisp
bar-plot summary :x "category" :y "total_revenue" :title "Revenue by category"
```

Available functions: `scatter-plot`, `line-plot`, `bar-plot`, `pie-plot`, `histogram`, `box-plot`, and `time-series-plot`.

## Core library

| Area | Functions and forms |
| --- | --- |
| Bindings and functions | `define`, `set`, `let`, `lambda`, `defun`, `lookup`, `quote` |
| Control flow | `if`, `and`, `or`, `not`, `for`, `for-each`, `->` |
| Lists and functional programming | `list`, `car` / `first`, `cdr` / `rest`, `cons`, `len`, `nth`, `append`, `reverse`, `empty?`, `range`, `apply`, `map`, `filter`, `reduce` |
| Scalar strings | `str-concat`, `str-len`, `substring`, `split`, `str-join` |
| Conversion | `type-of`, `to-str` / `to-string`, `to-int`, `to-float`, `to-symbol`, `to-bool`, `to-list` |
| Arithmetic and comparison | `+`, `-`, `*`, `/`, `=`, `!=`, `<`, `>`, `<=`, `>=` |
| Mathematics | `abs`, `sqrt`, `exp`, `log`, `log10`, `pow`, `round`, `floor`, `ceil`, `sin`, `cos`, `tan` |
| Utilities | `print`, `shell`, `load`, `time-expr` |

Predefined bindings include `true`, `false`, `nil`, `*PI*`, `*E*`, and `*args*` (the process arguments).

## Implementation

The interpreter processes source through an indentation preprocessor, lexer, parser, and tree-walk evaluator. Dataframe values hold Polars LazyFrames, while column computations hold Polars expressions.

| Module | Purpose |
| --- | --- |
| `main.rs` | Command-line entry point, REPL, and error display |
| `wisp.rs` | Indentation and inline-grouping preprocessing |
| `lexer.rs` | Tokenization with source line information |
| `parser.rs` | Expression parsing and syntax shorthand |
| `interpreter.rs` | Evaluation, environments, and special forms |
| `types.rs` | Runtime values, environments, and errors |
| `corelib.rs` | Scalar, list, string, and functional operations |
| `polarslib.rs` | Dataframe and column-expression operations |
| `plotlarslib.rs` | Plotting functions |
| `stdlib.rs` | Native function registration and predefined bindings |

## Development

Run the Rust tests from the repository root:

```bash
cargo test
```

The test modules cover preprocessing, core language behavior, Polars operations, and indentation-based data pipelines. Dataframe tests reference fixture files such as `data1.csv` and `data2.csv`; keep the required fixtures available in the working directory when running them.
