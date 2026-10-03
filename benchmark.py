import argparse
import csv
from datetime import datetime, timezone
import os
import subprocess
import sys
import time
import uuid
import duckdb
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import polars as pl
import polars.testing as pl_testing

# ---------------------------------------------------------
# Загальна конфігурація
# ---------------------------------------------------------
ROW_SIZES = [1_000, 10_000, 100_000, 1_000_000, 10_000_000, 100_000_000]
WARMUP_RUNS = 1
BENCHMARK_RUNS = 3

# Файли скриптів
MLISP_BIN = "./target/release/mlisp"
MLISP_SCRIPT = "bench_query.lsp"
POLARS_SCRIPT = "bench_polars.py"
DUCKDB_SCRIPT = "bench_duckdb.py"
PANDAS_SCRIPT = "bench_pandas.py"
R_SCRIPT = "bench_r.R"
R_INPROCESS_SCRIPT = "bench_r_inprocess.R"
JULIA_SCRIPT = "bench_julia.jl"
JULIA_INPROCESS_SCRIPT = "bench_julia_inprocess.jl"


def get_data_filepath(num_rows: int) -> str:
    """Генерує назву файлу залежно від кількості рядків."""
    return f"temp_benchmark_data_{num_rows}.parquet"


# ---------------------------------------------------------
# Генерація синтетичних даних
# ---------------------------------------------------------
def ensure_dataset(num_rows: int) -> str:
    """Генерує файл даних лише за умови, що він ще не існує."""
    file_path = get_data_filepath(num_rows)

    if os.path.exists(file_path):
        print(f"  Файл {file_path} вже існує. Генерацію пропущено.")
        return file_path

    print(f"  Генерація нового датасету: {file_path}...")
    np.random.seed(42)
    df = pl.DataFrame(
        {
            "category": np.random.choice(
                ["A", "B", "C", "D", "E"], size=num_rows
            ),
            "col_a": np.random.uniform(0, 500, size=num_rows),
            "col_b": np.random.uniform(10, 1000, size=num_rows),
            "col_c": np.random.uniform(1, 50, size=num_rows),
        }
    )
    df.write_parquet(file_path)
    return file_path


# ---------------------------------------------------------
# РЕЖИМ 1: In-Process (Чистий час виконання запиту)
# ---------------------------------------------------------
def get_mlisp_inprocess_script(data_path: str) -> str:
    return f"""
define result
    time-expr
        -> (read-parquet "{data_path}")
            filter : > $col_a 100.0
            with-column $ratio : / $col_b $col_c
            group-by $category
                alias (mean $col_a) "avg_a"
                alias (sum $ratio) "total_ratio"
                alias (count) "n_count"
            sort $avg_a :desc
            collect :streaming

nth result 1
"""


def write_r_inprocess_script(data_path: str):
    """Створює R-скрипт, який використовує Acero/Arrow Dataset замість конвертації в data.table."""
    r_code = f"""
suppressPackageStartupMessages({{
    library(arrow)
    library(dplyr)
}})

elapsed <- system.time({{
    ds <- open_dataset("{data_path}", format = "parquet")
    res <- ds |>
        select(category, col_a, col_b, col_c) |>
        filter(col_a > 100.0) |>
        mutate(ratio = col_b / col_c) |>
        group_by(category) |>
        summarise(
            avg_a = mean(col_a),
            total_ratio = sum(ratio),
            n_count = n()
        ) |>
        arrange(desc(avg_a)) |>
        collect()
}})[["elapsed"]]

cat(sprintf("%.6f\\n", elapsed))
"""
    with open(R_INPROCESS_SCRIPT, "w") as f:
        f.write(r_code.strip())


def run_inprocess_mlisp(data_file: str) -> float:
    with open(MLISP_SCRIPT, "w") as f:
        f.write(get_mlisp_inprocess_script(data_file))

    proc = subprocess.run(
        [MLISP_BIN, MLISP_SCRIPT],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if proc.returncode != 0:
        err_msg = proc.stderr.strip() or proc.stdout.strip()
        raise RuntimeError(f"Помилка виконання MLisp:\n{err_msg}")

    output = proc.stdout.strip().splitlines()
    elapsed_secs = float(output[-1])
    return elapsed_secs * 1000.0


def run_inprocess_polars(data_file: str) -> float:
    start = time.perf_counter()
    q = (
        pl.scan_parquet(data_file)
        .filter(pl.col("col_a") > 100.0)
        .with_columns((pl.col("col_b") / pl.col("col_c")).alias("ratio"))
        .group_by("category")
        .agg(
            pl.col("col_a").mean().alias("avg_a"),
            pl.col("ratio").sum().alias("total_ratio"),
            pl.len().alias("n_count"),
        )
        .sort("avg_a", descending=True)
    )
    _ = q.collect(engine="streaming")
    return (time.perf_counter() - start) * 1000.0


def run_inprocess_duckdb(data_file: str) -> float:
    query = f"""
    SELECT
        category,
        AVG(col_a) AS avg_a,
        SUM(col_b / col_c) AS total_ratio,
        COUNT(*) AS n_count
    FROM read_parquet('{data_file}')
    WHERE col_a > 100.0
    GROUP BY category
    ORDER BY avg_a DESC
    """
    start = time.perf_counter()
    _ = duckdb.sql(query).fetchall()
    return (time.perf_counter() - start) * 1000.0


def run_inprocess_pandas(data_file: str) -> float:
    start = time.perf_counter()
    df = pd.read_parquet(
        data_file,
        columns=["category", "col_a", "col_b", "col_c"],
        filters=[("col_a", ">", 100.0)],
        engine="pyarrow",
        dtype_backend="pyarrow",
    )
    df["ratio"] = df["col_b"] / df["col_c"]

    _ = (
        df.groupby("category", as_index=False)
        .agg(
            avg_a=("col_a", "mean"),
            total_ratio=("ratio", "sum"),
            n_count=("category", "count"),
        )
        .sort_values(by="avg_a", ascending=False)
    )
    return (time.perf_counter() - start) * 1000.0


def run_inprocess_r(data_file: str) -> float:
    write_r_inprocess_script(data_file)
    proc = subprocess.run(
        ["Rscript", "--vanilla", R_INPROCESS_SCRIPT],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if proc.returncode != 0:
        err_msg = proc.stderr.strip() or proc.stdout.strip()
        raise RuntimeError(f"Помилка виконання R:\n{err_msg}")

    output = proc.stdout.strip().splitlines()
    elapsed_secs = float(output[-1])
    return elapsed_secs * 1000.0

def write_julia_inprocess_script(data_path: str):
    """Writes an optimized Julia script using views and in-place operations."""
    jl_code = f"""
using Parquet2
using DataFrames
using Statistics

function run_pipeline(path::String)
    # copycols=false avoids duplicating arrays during construction
    df = DataFrame(Parquet2.readfile(path); copycols=false)

    # subset with view=true avoids copying filtered rows into a new DataFrame
    vdf = subset(df, :col_a => ByRow(>(100.0)); view=true)

    # Compute sum(b ./ c) directly without creating an intermediate column
    res = combine(
        groupby(vdf, :category),
        :col_a => mean => :avg_a,
        [:col_b, :col_c] => ((b, c) -> sum(b ./ c)) => :total_ratio,
        nrow => :n_count
    )

    sort!(res, :avg_a, rev=true)
    return res
end

# Warm-up pass to compile functions and schema
_ = run_pipeline("{data_path}")

# Timed execution pass
t = @elapsed run_pipeline("{data_path}")
println(t)
"""
    with open(JULIA_INPROCESS_SCRIPT, "w") as f:
        f.write(jl_code.strip())


def run_inprocess_julia(data_file: str) -> float:
    write_julia_inprocess_script(data_file)
    proc = subprocess.run(
        ["julia", "--threads=auto", JULIA_INPROCESS_SCRIPT],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if proc.returncode != 0:
        err_msg = proc.stderr.strip() or proc.stdout.strip()
        raise RuntimeError(f"Помилка виконання Julia:\n{err_msg}")

    output = proc.stdout.strip().splitlines()
    elapsed_secs = float(output[-1])
    return elapsed_secs * 1000.0


# ---------------------------------------------------------
# РЕЖИМ 2: Process (Запуск процесу CLI: старт -> завершення)
# ---------------------------------------------------------
def write_process_scripts(data_file: str):
    with open(MLISP_SCRIPT, "w") as f:
        f.write(f"""
-> (read-parquet "{data_file}")
    filter : > $col_a 100.0
    with-column $ratio : / $col_b $col_c
    group-by $category
        alias (mean $col_a) "avg_a"
        alias (sum $ratio) "total_ratio"
        alias (count) "n_count"
    sort $avg_a :desc
    collect :streaming
""".strip())

    with open(POLARS_SCRIPT, "w") as f:
        f.write(f"""
import polars as pl
q = (
    pl.scan_parquet("{data_file}")
    .filter(pl.col("col_a") > 100.0)
    .with_columns((pl.col("col_b") / pl.col("col_c")).alias("ratio"))
    .group_by("category")
    .agg(
        pl.col("col_a").mean().alias("avg_a"),
        pl.col("ratio").sum().alias("total_ratio"),
        pl.len().alias("n_count"),
    )
    .sort("avg_a", descending=True)
)
_ = q.collect(engine="streaming")
""".strip())

    with open(DUCKDB_SCRIPT, "w") as f:
        f.write(f"""
import duckdb
query = '''
SELECT
    category,
    AVG(col_a) AS avg_a,
    SUM(col_b / col_c) AS total_ratio,
    COUNT(*) AS n_count
FROM read_parquet('{data_file}')
WHERE col_a > 100.0
GROUP BY category
ORDER BY avg_a DESC
'''
_ = duckdb.sql(query).fetchall()
""".strip())

    with open(PANDAS_SCRIPT, "w") as f:
        f.write(f"""
import pandas as pd
df = pd.read_parquet(
    "{data_file}",
    columns=["category", "col_a", "col_b", "col_c"],
    filters=[("col_a", ">", 100.0)],
    engine="pyarrow",
    dtype_backend="pyarrow",
)
df["ratio"] = df["col_b"] / df["col_c"]
_ = (
    df.groupby("category", as_index=False)
    .agg(avg_a=("col_a", "mean"), total_ratio=("ratio", "sum"), n_count=("category", "count"))
    .sort_values(by="avg_a", ascending=False)
)
""".strip())

    with open(R_SCRIPT, "w") as f:
        f.write(f"""
suppressPackageStartupMessages({{
    library(arrow)
    library(dplyr)
}})
ds <- open_dataset("{data_file}", format = "parquet")
res <- ds |>
    select(category, col_a, col_b, col_c) |>
    filter(col_a > 100.0) |>
    mutate(ratio = col_b / col_c) |>
    group_by(category) |>
    summarise(
        avg_a = mean(col_a),
        total_ratio = sum(ratio),
        n_count = n()
    ) |>
    arrange(desc(avg_a)) |>
    collect()
""".strip())

    with open(JULIA_SCRIPT, "w") as f:
        f.write(f"""
using Parquet2
using DataFrames
using Statistics

df = DataFrame(Parquet2.readfile("{data_file}"); copycols=false)
vdf = subset(df, :col_a => ByRow(>(100.0)); view=true)
res = combine(
    groupby(vdf, :category),
    :col_a => mean => :avg_a,
    [:col_b, :col_c] => ((b, c) -> sum(b ./ c)) => :total_ratio,
    nrow => :n_count
)
sort!(res, :avg_a, rev=true)
""".strip())


def run_command(cmd: list[str]) -> float:
    start = time.perf_counter()
    proc = subprocess.run(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    elapsed_ms = (time.perf_counter() - start) * 1000.0

    if proc.returncode != 0:
        err = proc.stderr.strip() or proc.stdout.strip()
        raise RuntimeError(f"Помилка виконання команди ({' '.join(cmd)}):\n{err}")

    return elapsed_ms


# ---------------------------------------------------------
# Допоміжні функції виконання та побудови графіків
# ---------------------------------------------------------
def benchmark_runner(runner_fn, *args, record_run=None):
    durations = []
    for phase, run_count in [("warmup", WARMUP_RUNS), ("benchmark", BENCHMARK_RUNS)]:
        for run_number in range(1, run_count + 1):
            try:
                elapsed_ms = runner_fn(*args)
            except Exception as exc:
                if record_run is not None:
                    record_run(phase, run_number, "", "error", str(exc))
                raise
            if record_run is not None:
                record_run(phase, run_number, elapsed_ms, "ok", "")
            if phase == "benchmark":
                durations.append(elapsed_ms)

    return np.mean(durations)


def plot_results(results_df: pd.DataFrame, title: str, output_file: str):
    styles = {
        "MLisp": {"color": "#1f77b4", "marker": "o", "linewidth": 2.5},          # Blue
        "Polars (Python)": {"color": "#ff7f0e", "marker": "s", "linewidth": 2}, # Orange
        "DuckDB": {"color": "#2ca02c", "marker": "^", "linewidth": 2},          # Green
        "Pandas (PyArrow)": {"color": "#d62728", "marker": "x", "linewidth": 2},# Red
        "R (Arrow)": {"color": "#9467bd", "marker": "D", "linewidth": 2},       # Purple
        #"Julia (DataFrames)": {"color": "#17becf", "marker": "v", "linewidth": 2}, # Cyan / Teal
    }

    output_stem, output_ext = os.path.splitext(output_file)
    output_ext = output_ext or ".png"
    figures = []
    for scale, subtitle, ylabel in [
        ("log", "Логарифмічний граф часу", "Час виконання (мс) — логарифмічна шкала"),
        ("linear", "Лінійний граф часу", "Час виконання (мс) — лінійна шкала"),
    ]:
        fig, ax = plt.subplots(figsize=(8, 6), dpi=300)
        for engine in results_df.columns:
            style = styles.get(engine, {"marker": "o", "linewidth": 2})
            ax.plot(results_df.index, results_df[engine], label=engine, **style)

        ax.set_xscale("log")
        ax.set_yscale(scale)
        ax.set_xlabel("Розмір датасету (кількість рядків)", fontsize=11, fontweight="bold")
        ax.set_ylabel(ylabel, fontsize=11, fontweight="bold")
        ax.set_title(subtitle, fontsize=12, fontweight="bold")
        ax.grid(True, which="both", linestyle="--", alpha=0.5)
        ax.legend(frameon=True)
        fig.suptitle(title, fontsize=14, fontweight="bold")
        fig.tight_layout()
        image_path = f"{output_stem}_{scale}{output_ext}"
        fig.savefig(image_path)
        figures.append(fig)
        print(f"\nГрафік успішно збережено до: {image_path}")

    plt.show()
    for fig in figures:
        plt.close(fig)


VERIFY_OUTPUT = "temp_mlisp_verify_result.parquet"

def run_verify():
    test_size = ROW_SIZES[0]
    data_file = ensure_dataset(test_size)
    print(f"\n--- [VERIFY] Перевірка результатів запиту на датасеті: {test_size:,} рядків ---")

    # 1. Еталонне виконання через Polars
    expected_df = (
        pl.scan_parquet(data_file)
        .filter(pl.col("col_a") > 100.0)
        .with_columns((pl.col("col_b") / pl.col("col_c")).alias("ratio"))
        .group_by("category")
        .agg(
            pl.col("col_a").mean().alias("avg_a"),
            pl.col("ratio").sum().alias("total_ratio"),
            pl.len().alias("n_count"),
        )
        .sort("avg_a", descending=True)
        .collect()
    )

    # 2. Перевірка 1: Режим write_process_scripts
    print("  Тестування коду з write_process_scripts...")
    write_process_scripts(data_file)
    with open(MLISP_SCRIPT, "r") as f:
        process_code = f.read()

    verify_process_code = process_code.rstrip()
    verify_process_code = verify_process_code.replace("collect :streaming", f"sink \"{VERIFY_OUTPUT}\"")

    with open(MLISP_SCRIPT, "w") as f:
        f.write(verify_process_code)

    if os.path.exists(VERIFY_OUTPUT):
        os.remove(VERIFY_OUTPUT)

    proc = subprocess.run([MLISP_BIN, MLISP_SCRIPT], capture_output=True, text=True)
    if proc.returncode != 0:
        raise RuntimeError(f"Помилка в process скрипті:\n{proc.stderr or proc.stdout}")

    actual_process_df = pl.read_parquet(VERIFY_OUTPUT)
    pl_testing.assert_frame_equal(
        actual_process_df,
        expected_df,
        check_exact=False,
        rel_tol=1e-5,
        abs_tol=1e-5,
        check_dtypes=False,
    )
    print("    -> write_process_scripts: результати збігаються! ✅")

# ---------------------------------------------------------
# Головна точка входу
# ---------------------------------------------------------
def main():
    parser = argparse.ArgumentParser(description="Бенчмарк масштабування аналітичних запитів.")
    parser.add_argument(
        "--mode",
        choices=["in-process", "process", "verify"],
        default="in-process",
        help="Режим бенчмарку: 'in-process' (чистий час запиту) або 'process' (повний запуск процесу CLI).",
    )
    parser.add_argument(
        "--csv-output",
        default="benchmark_all_runs.csv",
        help="CSV для всіх запусків; нові сесії додаються до наявного файлу.",
    )
    args = parser.parse_args()

    if args.mode == "in-process":
        chart_output = "benchmark_inprocess_results.png"
        title = "Бенчмарк часу виконання запиту"
        engines = [
            ("MLisp", lambda f: run_inprocess_mlisp(f)),
            ("Polars (Python)", lambda f: run_inprocess_polars(f)),
            ("DuckDB", lambda f: run_inprocess_duckdb(f)),
            ("Pandas (PyArrow)", lambda f: run_inprocess_pandas(f)),
            ("R (Arrow)", lambda f: run_inprocess_r(f)),
            #("Julia (DataFrames)", lambda f: run_inprocess_julia(f)),
        ]
    elif args.mode == "process":
        chart_output = "benchmark_process_scaling_results.png"
        title = "Бенчмарк запуску скрипту з запитом"
        engines = [
            ("MLisp", lambda _: run_command([MLISP_BIN, MLISP_SCRIPT])),
            ("Polars (Python)", lambda _: run_command([sys.executable, POLARS_SCRIPT])),
            ("DuckDB", lambda _: run_command([sys.executable, DUCKDB_SCRIPT])),
            ("Pandas (PyArrow)", lambda _: run_command([sys.executable, PANDAS_SCRIPT])),
            ("R (Arrow)", lambda _: run_command(["Rscript", "--vanilla", R_SCRIPT])),
            #("Julia (DataFrames)", lambda _: run_command(["julia", "--threads=auto", JULIA_SCRIPT])),
        ]
    elif args.mode == "verify":
        try:
            run_verify()
        finally:
            for f in [MLISP_SCRIPT, VERIFY_OUTPUT]:
                if os.path.exists(f):
                    os.remove(f)
        return
    else:
        raise Exception()

    all_data = {engine_name: [] for engine_name, _ in engines}

    session_id = f"{datetime.now(timezone.utc).isoformat()}_{uuid.uuid4().hex[:8]}"
    csv_columns = ["session_id", "mode", "rows", "engine", "phase", "run", "elapsed_ms", "status", "error"]
    csv_file = None
    try:
        # Append sessions to one CSV, preserving results from previous invocations.
        if os.path.exists(args.csv_output) and os.path.getsize(args.csv_output) > 0:
            with open(args.csv_output, newline="", encoding="utf-8") as existing:
                if next(csv.reader(existing), None) != csv_columns:
                    raise ValueError(f"Несумісний заголовок CSV: {args.csv_output}")
        csv_file = open(args.csv_output, "a", newline="", encoding="utf-8")
        writer = csv.writer(csv_file)
        if csv_file.tell() == 0:
            writer.writerow(csv_columns)
            csv_file.flush()

        for rows in ROW_SIZES:
            print(f"\n--- [{args.mode.upper()}] Тестування на датасеті: {rows:,} рядків ---")
            current_data_file = ensure_dataset(rows)

            if args.mode == "process":
                write_process_scripts(current_data_file)

            for name, runner_fn in engines:
                print(f"  Тестування {name}...")
                try:
                    def record_run(phase, run_number, elapsed_ms, status, error):
                        writer.writerow([session_id, args.mode, rows, name, phase,
                                         run_number, elapsed_ms, status, error])
                        csv_file.flush()

                    mean_ms = benchmark_runner(
                        runner_fn, current_data_file, record_run=record_run
                    )
                    all_data[name].append(mean_ms)
                    print(f"    -> {mean_ms:.2f} мс")
                except Exception as e:
                    print(f"    -> Помилка: {e}")
                    all_data[name].append(np.nan)

        results_df = pd.DataFrame(all_data, index=ROW_SIZES)
        results_df.index.name = "Рядки"

        print("\n" + "=" * 65)
        print(f"Підсумковий звіт ({args.mode}) — середній час у мс")
        print("=" * 65)
        print(results_df.to_string())

        print(f"\nРезультати всіх запусків збережено до: {args.csv_output}")
        plot_results(results_df, title, chart_output)

    finally:
        if csv_file is not None:
            csv_file.close()
        for f in [
            MLISP_SCRIPT,
            POLARS_SCRIPT,
            DUCKDB_SCRIPT,
            PANDAS_SCRIPT,
            R_SCRIPT,
            R_INPROCESS_SCRIPT,
        ]:
            if os.path.exists(f):
                os.remove(f)


if __name__ == "__main__":
    main()
