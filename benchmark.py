import argparse
import csv
import hashlib
import json
import platform
import random
from pathlib import Path
from datetime import datetime, timezone
import os
import subprocess
import sys
import time
import tempfile
import uuid
import pyarrow as pa
import pyarrow.parquet as pq
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
BENCHMARK_RUNS = 10
THREADS = max(1, len(os.sched_getaffinity(0)) if hasattr(os, "sched_getaffinity") else (os.cpu_count() or 1))
DATASET_CHUNK_ROWS = 250_000

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
    # Separate generators preserve column values regardless of chunk size.
    rng_category, rng_a, rng_b, rng_c = [
        np.random.default_rng(seed) for seed in (42, 43, 44, 45)
    ]
    temporary_path = file_path + ".partial"
    try:
        with pq.ParquetWriter(
            temporary_path,
            pa.schema([("category", pa.string()), ("col_a", pa.float64()),
                       ("col_b", pa.float64()), ("col_c", pa.float64())]),
            compression="zstd",
        ) as writer:
            for offset in range(0, num_rows, DATASET_CHUNK_ROWS):
                size = min(DATASET_CHUNK_ROWS, num_rows - offset)
                table = pa.table({
                    "category": rng_category.choice(["A", "B", "C", "D", "E"], size=size),
                    "col_a": rng_a.uniform(0, 500, size=size),
                    "col_b": rng_b.uniform(10, 1000, size=size),
                    "col_c": rng_c.uniform(1, 50, size=size),
                })
                writer.write_table(table, row_group_size=DATASET_CHUNK_ROWS)
                del table
        os.replace(temporary_path, file_path)
    finally:
        if os.path.exists(temporary_path):
            os.remove(temporary_path)
    return file_path


# ---------------------------------------------------------
# РЕЖИМ 1: In-Process (Чистий час виконання запиту)
# ---------------------------------------------------------
def get_mlisp_inprocess_script(data_path: str) -> str:
    measured_script = f"""
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

    warmup_query = measured_script.split("        ->", 1)[1].split("\nnth result", 1)[0]
    warmup_query = "->" + warmup_query
    # The first line starts at column zero; the remaining lines keep relative indentation.
    warmup_query = "\n".join(
        line[8:] if line.startswith("        ") else line
        for line in warmup_query.splitlines()
    )
    return (warmup_query.strip() + "\n\n") * WARMUP_RUNS + measured_script


def write_r_inprocess_script(data_path: str):
    """Створює R-скрипт, який використовує Acero/Arrow Dataset замість конвертації в data.table."""
    r_code = f"""
suppressPackageStartupMessages({{
    library(arrow)
    library(dplyr)
}})

arrow::set_cpu_count({THREADS})

run_pipeline <- function() {{
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
    invisible(res)
}}
for (i in seq_len({WARMUP_RUNS})) run_pipeline()
start <- Sys.time()
res <- run_pipeline()
elapsed <- as.numeric(difftime(Sys.time(), start, units = "secs"))

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
        env=worker_environment(),
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
        env=worker_environment(),
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
        env=worker_environment(),
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
def write_process_scripts(data_file: str, script_directory=None):
    def script_path(filename):
        if script_directory is None:
            return filename
        return str(Path(script_directory) / Path(filename).name)

    with open(script_path(MLISP_SCRIPT), "w") as f:
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

    with open(script_path(POLARS_SCRIPT), "w") as f:
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

    with open(script_path(DUCKDB_SCRIPT), "w") as f:
        f.write(f"""
import duckdb
duckdb.sql("SET threads = {THREADS}")
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

    with open(script_path(PANDAS_SCRIPT), "w") as f:
        f.write(f"""
import pandas as pd
import pyarrow as pa
pa.set_cpu_count({THREADS})
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

    with open(script_path(R_SCRIPT), "w") as f:
        f.write(f"""
suppressPackageStartupMessages({{
    library(arrow)
    library(dplyr)
}})
arrow::set_cpu_count({THREADS})
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

    with open(script_path(JULIA_SCRIPT), "w") as f:
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
        env=worker_environment(),
    )
    elapsed_ms = (time.perf_counter() - start) * 1000.0

    if proc.returncode != 0:
        err = proc.stderr.strip() or proc.stdout.strip()
        raise RuntimeError(f"Помилка виконання команди ({' '.join(cmd)}):\n{err}")

    return elapsed_ms


# ---------------------------------------------------------
# Допоміжні функції виконання та побудови графіків
# ---------------------------------------------------------
def worker_environment():
    env = os.environ.copy()
    for key in ("POLARS_MAX_THREADS", "OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS",
                "MKL_NUM_THREADS", "NUMEXPR_NUM_THREADS", "ARROW_NUM_THREADS"):
        env[key] = str(THREADS)
    return env


def warm_file_cache(data_file):
    """Best-effort OS-cache warm-up outside the timed section, with bounded memory."""
    with open(data_file, "rb") as source:
        while source.read(8 * 1024 * 1024):
            pass


def run_python_trial(engine, data_file):
    proc = subprocess.run(
        [sys.executable, str(Path(__file__).resolve()), "--worker", engine,
         "--data-file", str(Path(data_file).resolve()), "--warmup-runs", str(WARMUP_RUNS),
         "--threads", str(THREADS)],
        capture_output=True, text=True, env=worker_environment(),
    )
    if proc.returncode:
        raise RuntimeError(proc.stderr.strip() or proc.stdout.strip())
    return float(proc.stdout.strip().splitlines()[-1])


def summarize_runs(records):
    measured = pd.DataFrame([r for r in records if r["phase"] == "benchmark"])
    grouped = measured.groupby(["mode", "rows", "engine"], sort=False, dropna=False)
    summaries = []
    for (mode, rows, engine), group in grouped:
        values = pd.to_numeric(group.loc[group.status == "ok", "elapsed_ms"])
        summaries.append({
            "mode": mode, "rows": rows, "engine": engine,
            "successful_runs": len(values), "failed_runs": int((group.status == "error").sum()),
            "mean_ms": values.mean(), "median_ms": values.median(),
            "std_ms": values.std(ddof=1), "min_ms": values.min(), "max_ms": values.max(),
            "q25_ms": values.quantile(0.25), "q75_ms": values.quantile(0.75),
        })
    return pd.DataFrame(summaries)


def probe_command(command):
    try:
        proc = subprocess.run(command, capture_output=True, text=True, timeout=30,
                              env=worker_environment())
        return {"returncode": proc.returncode,
                "output": (proc.stdout.strip() or proc.stderr.strip())[:6000]}
    except (OSError, subprocess.TimeoutExpired) as exc:
        return {"error": str(exc)}


def environment_metadata(args):
    cpu_model = platform.processor()
    try:
        for line in Path("/proc/cpuinfo").read_text().splitlines():
            if line.startswith("model name"):
                cpu_model = line.split(":", 1)[1].strip()
                break
    except OSError:
        pass
    try:
        ram_bytes = os.sysconf("SC_PAGE_SIZE") * os.sysconf("SC_PHYS_PAGES")
    except (ValueError, OSError, AttributeError):
        ram_bytes = None
    binary = Path(MLISP_BIN)
    rust_polars = []
    cargo_lock = Path("Cargo.lock")
    if cargo_lock.exists():
        try:
            import tomllib
            packages = tomllib.loads(cargo_lock.read_text())["package"]
            rust_polars = [{"name": item["name"], "version": item["version"]}
                           for item in packages if item["name"].startswith("polars")]
        except (ImportError, ValueError, KeyError) as exc:
            rust_polars = [{"error": str(exc)}]
    binary_hash = None
    if binary.exists():
        with binary.open("rb") as stream:
            digest = hashlib.sha256()
            while chunk := stream.read(1024 * 1024):
                digest.update(chunk)
            binary_hash = digest.hexdigest()
    return {
        "platform": platform.platform(), "cpu_model": cpu_model,
        "logical_cpus": os.cpu_count(), "ram_bytes": ram_bytes,
        "python": sys.version, "python_executable": sys.executable,
        "versions": {"polars_python": pl.__version__, "duckdb": duckdb.__version__,
                     "pandas": pd.__version__, "pyarrow": pa.__version__, "numpy": np.__version__},
        "rust_polars_from_cargo_lock": rust_polars,
        "rust_polars_note": "Cargo.lock describes local source dependencies; it does not prove which version built the binary.",
        "rustc": probe_command(["rustc", "--version"]),
        "r": probe_command(["Rscript", "--vanilla", "-e",
            'cat(R.version.string, "\\n"); for (p in c("arrow", "dplyr")) cat(p, as.character(packageVersion(p)), "\\n")']),
        "mlisp_binary": str(binary.resolve()), "mlisp_binary_sha256": binary_hash,
        "mlisp_build_note": args.mlisp_build_note or "Not supplied; release path alone does not prove build flags.",
        "threads_requested": THREADS, "worker_thread_environment": {
            k: v for k, v in worker_environment().items() if k in {
                "POLARS_MAX_THREADS", "OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS",
                "NUMEXPR_NUM_THREADS", "ARROW_NUM_THREADS"}},
        "thread_note": "Thread limits are configured; actual parallelism depends on the engine and operation.",
        "benchmark_runs": BENCHMARK_RUNS, "warmup_runs": WARMUP_RUNS,
        "row_sizes": ROW_SIZES, "shuffle_seed": args.seed,
        "cache_policy": "Sequential file pre-read before each trial; best-effort warm OS cache, not guaranteed RAM residency.",
        "inprocess_policy": "Fresh process per sample; warm-ups and the measured query share that process; startup excluded.",
        "process_policy": "Fresh CLI process per sample; startup, imports and query included.",
        "dataset_generation": "Bounded Arrow batches; new files use independent RNG seeds 42/43/44/45 and zstd.",
        "dataset_chunk_rows": DATASET_CHUNK_ROWS,
        "scope": "Specific analytic stacks and one query; not a general language-speed ranking.",
    }


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
        fig.suptitle(title + "\nМедіанний час виконання", fontsize=12, fontweight="bold")
        fig.tight_layout()
        image_path = f"{output_stem}_{scale}{output_ext}"
        fig.savefig(image_path)
        figures.append(fig)
        print(f"\nГрафік успішно збережено до: {image_path}")

    plt.show()
    for fig in figures:
        plt.close(fig)


def plot_run_spread(records, engine_names, title, output_file):
    """Show raw measured samples, medians and min–max ranges, excluding warm-ups/errors."""
    colors = {
        "MLisp": "#1f77b4", "Polars (Python)": "#ff7f0e", "DuckDB": "#2ca02c",
        "Pandas (PyArrow)": "#d62728", "R (Arrow)": "#9467bd",
    }
    labels = [name.replace(" (", "\n(") for name in engine_names]
    sizes = list(dict.fromkeys(record["rows"] for record in records if record["phase"] == "benchmark"))
    if not sizes:
        return
    columns = min(3, len(sizes))
    panel_rows = (len(sizes) + columns - 1) // columns
    fig, axes = plt.subplots(panel_rows, columns, figsize=(6 * columns, 4.5 * panel_rows),
                             dpi=300, squeeze=False)
    for ax, size in zip(axes.flat, sizes):
        has_samples = False
        for position, engine in enumerate(engine_names, start=1):
            values = np.asarray([
                float(record["elapsed_ms"]) for record in records
                if record["phase"] == "benchmark" and record["status"] == "ok"
                and record["rows"] == size and record["engine"] == engine
            ])
            values = values[np.isfinite(values) & (values > 0)]
            if not len(values):
                continue
            has_samples = True
            color = colors.get(engine, "#7f7f7f")
            # Small horizontal offsets keep individual samples visible; x has no time meaning.
            offsets = np.linspace(-0.12, 0.12, len(values)) if len(values) > 1 else np.zeros(1)
            ax.scatter(position + offsets, values, color=color, s=22, alpha=0.8, zorder=3)
            median = float(np.median(values))
            ax.errorbar(position, median,
                        yerr=[[median - float(values.min())], [float(values.max()) - median]],
                        fmt="none", ecolor=color, capsize=6, linewidth=1.5, zorder=2)
            ax.plot(position, median, marker="_", color="black", markersize=16,
                    markeredgewidth=2, linestyle="none", zorder=4)
        if has_samples:
            ax.set_yscale("log")
        else:
            ax.text(0.5, 0.5, "Немає успішних вимірювань", ha="center", va="center", transform=ax.transAxes)
        ax.set_title(f"{size:,} рядків", fontsize=12, fontweight="bold")
        ax.set_ylabel("Час виконання (мс), логарифмічна шкала", fontsize=10)
        ax.set_xticks(range(1, len(engine_names) + 1), labels)
        ax.tick_params(axis="x", labelsize=9)
        ax.set_xlim(0.5, len(engine_names) + 0.5)
        ax.grid(True, axis="y", which="both", linestyle="--", alpha=0.4)
    for ax in list(axes.flat)[len(sizes):]:
        ax.set_visible(False)
    fig.suptitle(title + " — розкид результатів\nТочки — окремі запуски; чорна риска — медіана\n"
                 "Кольоровий відрізок — від мінімуму до максимуму", fontsize=12, fontweight="bold")
    fig.tight_layout(rect=(0, 0, 1, 0.88 if panel_rows == 1 else 0.92))
    fig.savefig(output_file)
    print(f"\nГрафік розкиду збережено до: {output_file}")
    plt.show()
    plt.close(fig)


VERIFY_ROWS = 1000
VERIFY_COLUMNS = ["category", "avg_a", "total_ratio", "n_count"]


def compare_verify_tables(expected, actual, rel_tol=1e-5, abs_tol=1e-5):
    """Compare integer/category values exactly and floating aggregates with tolerance."""
    for name, frame in [("reference", expected), ("actual", actual)]:
        if len(frame.columns) != len(VERIFY_COLUMNS) or set(frame.columns) != set(VERIFY_COLUMNS):
            raise AssertionError(f"{name}: unexpected columns {frame.columns}")
        if frame.height != 5 or frame["category"].n_unique() != 5:
            raise AssertionError(f"{name}: expected exactly five unique category rows")
        if frame.null_count().row(0) != (0, 0, 0, 0):
            raise AssertionError(f"{name}: unexpected null values")
        for column in ("avg_a", "total_ratio", "n_count"):
            if not frame[column].dtype.is_numeric():
                raise AssertionError(f"{name}: {column} must be numeric")
            if not np.isfinite(frame[column].to_numpy()).all():
                raise AssertionError(f"{name}: non-finite values in {column}")
    expected = expected.select(VERIFY_COLUMNS).sort("category")
    actual = actual.select(VERIFY_COLUMNS).sort("category")
    for column in ("category", "n_count"):
        pl_testing.assert_series_equal(actual[column], expected[column],
                                       check_dtypes=False, check_exact=True)
    pl_testing.assert_frame_equal(
        actual.select("avg_a", "total_ratio"), expected.select("avg_a", "total_ratio"),
        check_dtypes=False, check_exact=False, rel_tol=rel_tol, abs_tol=abs_tol,
    )


def run_verify(output_directory="benchmark_verify_results"):
    # Fixed control sample; independent of --rows used for timing runs.
    data_file = str(Path(ensure_dataset(VERIFY_ROWS)).resolve())
    control = pl.read_parquet(data_file)
    if control.height != VERIFY_ROWS or set(control["category"].unique().to_list()) != set("ABCDE"):
        raise ValueError("Контрольний набір має містити 1000 рядків і всі категорії A–E.")
    filtered_count = control.filter(pl.col("col_a") > 100.0).height
    if not 0 < filtered_count < VERIFY_ROWS:
        raise ValueError("Контрольний набір має містити рядки, які проходять і не проходять фільтр.")
    if control.filter(pl.col("col_a") > 100.0)["category"].n_unique() != 5:
        raise ValueError("Після фільтрації мають залишитися всі п'ять категорій.")
    output_dir = Path(output_directory).resolve()
    output_dir.mkdir(parents=True, exist_ok=True)
    engines = [
        ("MLisp", "mlisp", MLISP_SCRIPT),
        ("Polars (Python)", "polars", POLARS_SCRIPT),
        ("DuckDB", "duckdb", DUCKDB_SCRIPT),
        ("Pandas (PyArrow)", "pandas", PANDAS_SCRIPT),
        ("R (Arrow)", "r", R_SCRIPT),
    ]
    report = {"rows": VERIFY_ROWS, "data_file": data_file, "reference": "Polars (Python)",
              "rel_tol": 1e-5, "abs_tol": 1e-5,
              "comparison": "Table contents, sorted by category; category/count exact; floating aggregates tolerant.",
              "mlisp_note": "MLisp collect returns nil; its verification uses sink with the same preceding pipeline.",
              "results": []}
    print(f"\n--- [VERIFY] Порівняння всіх систем: {VERIFY_ROWS} рядків ---")
    with tempfile.TemporaryDirectory(prefix="benchmark_verify_scripts_") as scripts_dir:
        write_process_scripts(data_file, scripts_dir)
        for name, key, filename in engines:
            output_file = output_dir / f"{key}_result.parquet"
            output_file.unlink(missing_ok=True)  # Never compare stale output after a failed launch.
            script_file = Path(scripts_dir) / Path(filename).name
            code = script_file.read_text()
            if key == "mlisp":
                if code.count("collect :streaming") != 1:
                    raise ValueError("Не вдалося знайти завершення MLisp-запиту для перевірки.")
                code = code.replace("collect :streaming", f"sink {json.dumps(output_file.as_posix(), ensure_ascii=False)}")
                command = [str(Path(MLISP_BIN).resolve()), str(script_file)]
            elif key == "polars":
                code += f"\n_.write_parquet({str(output_file)!r})\n"
                command = [sys.executable, str(script_file)]
            elif key == "pandas":
                code += f"\n_.to_parquet({str(output_file)!r}, engine='pyarrow', index=False)\n"
                command = [sys.executable, str(script_file)]
            elif key == "duckdb":
                code += ("\nimport pyarrow as pa\nimport pyarrow.parquet as pq\n"
                         f"result = pa.Table.from_pylist([dict(zip({VERIFY_COLUMNS!r}, row)) for row in _])\n"
                         f"pq.write_table(result, {str(output_file)!r})\n")
                command = [sys.executable, str(script_file)]
            else:
                code += f"\narrow::write_parquet(res, {json.dumps(output_file.as_posix(), ensure_ascii=False)})\n"
                command = ["Rscript", "--vanilla", str(script_file)]
            script_file.write_text(code, encoding="utf-8")
            result = {"engine": name, "file": str(output_file), "status": "not_compared", "error": ""}
            try:
                proc = subprocess.run(command, capture_output=True, text=True, env=worker_environment())
                if proc.returncode:
                    raise RuntimeError(proc.stderr.strip() or proc.stdout.strip() or f"Exit code {proc.returncode}")
                if not output_file.is_file():
                    raise RuntimeError("Система не створила вихідний файл.")
                pl.read_parquet(output_file)  # Ensure output is readable before comparing.
                print(f"  {name}: результат записано")
            except Exception as exc:
                result.update(status="execution_error", error=str(exc))
                print(f"  {name}: помилка запуску — {exc}")
            report["results"].append(result)

    reference_result = next(r for r in report["results"] if r["engine"] == report["reference"])
    if reference_result["status"] != "execution_error":
        expected = pl.read_parquet(reference_result["file"])
        for result in report["results"]:
            if result["status"] == "execution_error":
                continue
            try:
                compare_verify_tables(expected, pl.read_parquet(result["file"]),
                                      rel_tol=report["rel_tol"], abs_tol=report["abs_tol"])
                result["status"] = "ok"
                print(f"  {result['engine']}: значення збігаються ✅")
            except Exception as exc:
                result.update(status="mismatch", error=str(exc))
                print(f"  {result['engine']}: відмінність даних — {exc}")
    else:
        print("  Еталон Polars недоступний: порівняння виконати неможливо.")
    report["all_match"] = all(result["status"] == "ok" for result in report["results"])
    report_file = output_dir / "verify_report.json"
    report_file.write_text(json.dumps(report, indent=2, ensure_ascii=False), encoding="utf-8")
    print(f"\nВихідні таблиці й звіт: {output_dir}")
    if not report["all_match"]:
        print("Перевірка НЕ пройдена: є відмінності або системи, які не вдалося перевірити.")
        raise SystemExit(1)
    print("Перевірка пройдена: результати всіх п'яти систем збігаються.")
    return report

# ---------------------------------------------------------
# Головна точка входу
# ---------------------------------------------------------
def main():
    global WARMUP_RUNS, BENCHMARK_RUNS, THREADS, ROW_SIZES, DATASET_CHUNK_ROWS
    parser = argparse.ArgumentParser(description="Бенчмарк конкретного запиту для аналітичних систем.")
    parser.add_argument("--mode", choices=["in-process", "process", "verify"], default="in-process")
    parser.add_argument("--csv-output", default="benchmark_all_runs.csv",
                        help="Єдиний CSV запусків; нові сесії додаються до наявного файлу.")
    parser.add_argument("--runs", type=int, default=BENCHMARK_RUNS)
    parser.add_argument("--warmup-runs", type=int, default=WARMUP_RUNS)
    parser.add_argument("--threads", type=int, default=THREADS)
    parser.add_argument("--rows", type=int, nargs="+", default=ROW_SIZES)
    parser.add_argument("--chunk-rows", type=int, default=DATASET_CHUNK_ROWS)
    parser.add_argument("--verify-output-dir", default="benchmark_verify_results", help="Каталог результатів перевірки.")
    parser.add_argument("--seed", type=int, default=42, help="Відтворюване перемішування порядку систем.")
    parser.add_argument("--mlisp-build-note", default="", help="Версія Rust Polars, команда збірки, RUSTFLAGS тощо.")
    parser.add_argument("--worker", choices=["polars", "duckdb", "pandas"], help=argparse.SUPPRESS)
    parser.add_argument("--data-file", help=argparse.SUPPRESS)
    args = parser.parse_args()
    if args.runs < 2 or args.warmup_runs < 1 or args.threads < 1 or args.chunk_rows < 1:
        parser.error("Потрібно runs >= 2, warmup-runs >= 1, threads >= 1, chunk-rows >= 1.")
    if not args.rows or any(rows < 1 for rows in args.rows) or len(set(args.rows)) != len(args.rows):
        parser.error("Розміри датасетів мають бути додатними та без повторів.")
    WARMUP_RUNS, BENCHMARK_RUNS, THREADS = args.warmup_runs, args.runs, args.threads
    ROW_SIZES, DATASET_CHUNK_ROWS = args.rows, args.chunk_rows

    if args.worker:
        if not args.data_file:
            parser.error("Worker requires --data-file")
        pa.set_cpu_count(THREADS)
        duckdb.sql(f"SET threads = {THREADS}")
        runner = {"polars": run_inprocess_polars, "duckdb": run_inprocess_duckdb,
                  "pandas": run_inprocess_pandas}[args.worker]
        for _ in range(WARMUP_RUNS):
            runner(args.data_file)
        print(runner(args.data_file))
        return

    if args.mode == "verify":
        run_verify(args.verify_output_dir)
        return

    if args.mode == "in-process":
        chart_output = "benchmark_inprocess_results.png"
        title = "Час запиту для аналітичних систем після прогріву"
        engines = [
            ("MLisp", run_inprocess_mlisp),
            ("Polars (Python)", lambda f: run_python_trial("polars", f)),
            ("DuckDB", lambda f: run_python_trial("duckdb", f)),
            ("Pandas (PyArrow)", lambda f: run_python_trial("pandas", f)),
            ("R (Arrow)", run_inprocess_r),
        ]
    else:
        chart_output = "benchmark_process_scaling_results.png"
        title = "Час повного запуску аналітичного скрипту"
        engines = [
            ("MLisp", lambda _: run_command([MLISP_BIN, MLISP_SCRIPT])),
            ("Polars (Python)", lambda _: run_command([sys.executable, POLARS_SCRIPT])),
            ("DuckDB", lambda _: run_command([sys.executable, DUCKDB_SCRIPT])),
            ("Pandas (PyArrow)", lambda _: run_command([sys.executable, PANDAS_SCRIPT])),
            ("R (Arrow)", lambda _: run_command(["Rscript", "--vanilla", R_SCRIPT])),
        ]

    session_id = f"{datetime.now(timezone.utc).isoformat()}_{uuid.uuid4().hex[:8]}"
    csv_columns = ["session_id", "mode", "rows", "engine", "phase", "run", "elapsed_ms", "status", "error"]
    output_path = Path(args.csv_output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    session_suffix = session_id.replace(":", "-")
    metadata_path = output_path.with_name(f"{output_path.stem}_{session_suffix}_metadata.json")
    summary_path = output_path.with_name(f"{output_path.stem}_{session_suffix}_summary.csv")
    rng = random.Random(args.seed)
    records = []
    metadata = environment_metadata(args)
    metadata.update(session_id=session_id, mode=args.mode, datasets=[])
    metadata_path.write_text(json.dumps(metadata, indent=2, ensure_ascii=False), encoding="utf-8")
    print("Порівнюються конкретні програмні стеки; версії й умови записано в JSON.")
    print("Файловий кеш прогрівається перед кожним запуском; холодне читання не вимірюється.")
    print("Для MLisp версію Rust Polars і прапорці збірки слід підтвердити через --mlisp-build-note.")

    try:
        if output_path.exists() and output_path.stat().st_size:
            with output_path.open(newline="", encoding="utf-8") as existing:
                if next(csv.reader(existing), None) != csv_columns:
                    raise ValueError(f"Несумісний заголовок CSV: {output_path}")
        with output_path.open("a", newline="", encoding="utf-8") as csv_file:
            writer = csv.DictWriter(csv_file, fieldnames=csv_columns)
            if csv_file.tell() == 0:
                writer.writeheader()
                csv_file.flush()
            for rows in ROW_SIZES:
                print(f"\n--- [{args.mode.upper()}] {rows:,} рядків ---")
                current_data_file = ensure_dataset(rows)
                file_stat = Path(current_data_file).stat()
                metadata["datasets"].append({"rows": rows, "path": str(Path(current_data_file).resolve()),
                                             "size_bytes": file_stat.st_size, "mtime_ns": file_stat.st_mtime_ns})
                metadata_path.write_text(json.dumps(metadata, indent=2, ensure_ascii=False), encoding="utf-8")
                if args.mode == "process":
                    write_process_scripts(current_data_file)
                # CLI warm-ups warm OS caches, but never pretend to reuse the process.
                # In-process warm-ups run INSIDE each trial's process and are excluded
                # from reported sample timings (their count is recorded in metadata).
                phases = [("warmup", WARMUP_RUNS), ("benchmark", BENCHMARK_RUNS)] if args.mode == "process" else [("benchmark", BENCHMARK_RUNS)]
                for phase, count in phases:
                    for run_number in range(1, count + 1):
                        order = engines.copy()
                        rng.shuffle(order)
                        for name, runner_fn in order:
                            record = dict(session_id=session_id, mode=args.mode, rows=rows, engine=name,
                                          phase=phase, run=run_number, elapsed_ms="", status="ok", error="")
                            try:
                                warm_file_cache(current_data_file)
                                elapsed = runner_fn(current_data_file)
                                if not np.isfinite(elapsed) or elapsed <= 0:
                                    raise ValueError(f"Некоректний час: {elapsed}; можливо, роздільності таймера недостатньо.")
                                record["elapsed_ms"] = elapsed
                                print(f"  {name}: {phase} {run_number}/{count} — {elapsed:.3f} мс")
                            except Exception as exc:
                                record.update(status="error", error=str(exc))
                                print(f"  {name}: {phase} {run_number}/{count} — Помилка: {exc}")
                            writer.writerow(record)
                            csv_file.flush()
                            records.append(record)
        summary = summarize_runs(records)
        summary.to_csv(summary_path, index=False)
        print("\nСтатистика часу виконання (мс):")
        print(summary.to_string(index=False))
        def pivot(column):
            return summary.pivot(index="rows", columns="engine", values=column).reindex(
                index=ROW_SIZES, columns=[name for name, _ in engines])
        plot_results(pivot("median_ms"), title, chart_output)
        output_stem, output_ext = os.path.splitext(chart_output)
        plot_run_spread(records, [name for name, _ in engines], title,
                        f"{output_stem}_spread{output_ext or '.png'}")
        print(f"\nЗапуски: {output_path}\nСтатистика: {summary_path}\nУмови: {metadata_path}")
    finally:
        for f in [MLISP_SCRIPT, POLARS_SCRIPT, DUCKDB_SCRIPT, PANDAS_SCRIPT, R_SCRIPT, R_INPROCESS_SCRIPT]:
            if os.path.exists(f):
                os.remove(f)


if __name__ == "__main__":
    main()
