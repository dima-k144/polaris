import os
import subprocess
import sys
import time
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import polars as pl

# ---------------------------------------------------------
# Configuration
# ---------------------------------------------------------
ROW_SIZES = [1_000, 10_000, 100_000, 1_000_000, 10_000_000]
WARMUP_RUNS = 1
BENCHMARK_RUNS = 3
DATA_FILE = "temp_benchmark_data.parquet"
CHART_OUTPUT = "benchmark_process_scaling_results.png"

# Temporary script filenames
MLISP_BIN = "./target/release/mlisp"
MLISP_SCRIPT = "bench_query.lsp"
POLARS_SCRIPT = "bench_polars.py"
DUCKDB_SCRIPT = "bench_duckdb.py"
PANDAS_SCRIPT = "bench_pandas.py"


# ---------------------------------------------------------
# Synthetic Data Generation
# ---------------------------------------------------------
def generate_dataset(num_rows: int):
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
    df.write_parquet(DATA_FILE)


# ---------------------------------------------------------
# Script Generators
# ---------------------------------------------------------
def write_benchmark_scripts():
    # 1. MLisp Script
    mlisp_code = f"""
(-> (read-parquet "{DATA_FILE}")
    (filter (> (col "col_a") (lit 100.0)))
    (with-column (col "ratio") (/ (col "col_b") (col "col_c")))
    (group-by (col "category")
                (alias (mean (col "col_a")) "avg_a")
                (alias (sum (col "ratio")) "total_ratio")
                (alias (count) "n_count"))
    (sort (col "avg_a") :desc)
    (collect))
"""
    with open(MLISP_SCRIPT, "w") as f:
        f.write(mlisp_code.strip())

    # 2. Polars Python Script
    polars_code = f"""
import polars as pl

q = (
    pl.scan_parquet("{DATA_FILE}")
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
_ = q.collect(engine="in-memory")
"""
    with open(POLARS_SCRIPT, "w") as f:
        f.write(polars_code.strip())

    # 3. DuckDB Python Script
    duckdb_code = f"""
import duckdb

query = '''
SELECT
    category,
    AVG(col_a) AS avg_a,
    SUM(col_b / col_c) AS total_ratio,
    COUNT(*) AS n_count
FROM read_parquet('{DATA_FILE}')
WHERE col_a > 100.0
GROUP BY category
ORDER BY avg_a DESC
'''
_ = duckdb.sql(query).fetchall()
"""
    with open(DUCKDB_SCRIPT, "w") as f:
        f.write(duckdb_code.strip())

    # 4. Pandas (PyArrow) Script
    pandas_code = f"""
import pandas as pd

df = pd.read_parquet(
    "{DATA_FILE}",
    columns=["category", "col_a", "col_b", "col_c"],
    engine="pyarrow",
)
filtered = df[df["col_a"] > 100.0].copy()
filtered["ratio"] = filtered["col_b"] / filtered["col_c"]

_ = (
    filtered.groupby("category", as_index=False)
    .agg(
        avg_a=("col_a", "mean"),
        total_ratio=("ratio", "sum"),
        n_count=("category", "count"),
    )
    .sort_values(by="avg_a", ascending=False)
)
"""
    with open(PANDAS_SCRIPT, "w") as f:
        f.write(pandas_code.strip())


# ---------------------------------------------------------
# Subprocess Process Runners (Measures Process Inception -> Exit)
# ---------------------------------------------------------
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
        raise RuntimeError(f"Command failed ({' '.join(cmd)}):\n{err}")

    return elapsed_ms


def run_mlisp_process() -> float:
    return run_command([MLISP_BIN, MLISP_SCRIPT])


def run_polars_process() -> float:
    return run_command([sys.executable, POLARS_SCRIPT])


def run_duckdb_process() -> float:
    return run_command([sys.executable, DUCKDB_SCRIPT])


def run_pandas_process() -> float:
    return run_command([sys.executable, PANDAS_SCRIPT])


# ---------------------------------------------------------
# Benchmark Harness & Plotting
# ---------------------------------------------------------
def benchmark_runner(runner_fn):
    for _ in range(WARMUP_RUNS):
        runner_fn()

    durations = []
    for _ in range(BENCHMARK_RUNS):
        durations.append(runner_fn())

    return np.mean(durations)


def plot_results(results_df: pd.DataFrame):
    fig, (ax_log, ax_lin) = plt.subplots(1, 2, figsize=(16, 6), dpi=300)

    styles = {
        "MLisp": {"color": "#1f77b4", "marker": "o", "linewidth": 2.5},
        "Polars": {"color": "#ff7f0e", "marker": "s", "linewidth": 2},
        "DuckDB": {"color": "#2ca02c", "marker": "^", "linewidth": 2},
        "Pandas (PyArrow)": {"color": "#d62728", "marker": "x", "linewidth": 2},
    }

    for engine in results_df.columns:
        style = styles.get(engine, {"marker": "o", "linewidth": 2})
        ax_log.plot(
            results_df.index,
            results_df[engine],
            label=engine,
            marker=style["marker"],
            linewidth=style["linewidth"],
            color=style.get("color"),
        )
        ax_lin.plot(
            results_df.index,
            results_df[engine],
            label=engine,
            marker=style["marker"],
            linewidth=style["linewidth"],
            color=style.get("color"),
        )

    # Panel 1: Log-Log Scale
    ax_log.set_xscale("log")
    ax_log.set_yscale("log")
    ax_log.set_xlabel("Dataset Size (Number of Rows)", fontsize=11, fontweight="bold")
    ax_log.set_ylabel("Total Process Time (ms) - Log Scale", fontsize=11, fontweight="bold")
    ax_log.set_title("Scaling Trend (Log-Log Scale)", fontsize=12, fontweight="bold")
    ax_log.grid(True, which="both", linestyle="--", alpha=0.5)
    ax_log.legend(frameon=True)

    # Panel 2: Linear Time Scale
    ax_lin.set_xscale("log")
    ax_lin.set_xlabel("Dataset Size (Number of Rows)", fontsize=11, fontweight="bold")
    ax_lin.set_ylabel("Total Process Time (ms) - Linear Scale", fontsize=11, fontweight="bold")
    ax_lin.set_title("Absolute Runtime (Process Start -> Exit)", fontsize=12, fontweight="bold")
    ax_lin.grid(True, which="both", linestyle="--", alpha=0.5)
    ax_lin.legend(frameon=True)

    plt.suptitle("CLI Process Execution Benchmark: Start to Exit", fontsize=14, fontweight="bold")
    plt.tight_layout()
    plt.savefig(CHART_OUTPUT)
    print(f"\nScaling chart successfully saved to: {CHART_OUTPUT}")
    plt.show()


def main():
    engines = [
        ("MLisp", run_mlisp_process),
        ("Polars", run_polars_process),
        ("DuckDB", run_duckdb_process),
        ("Pandas (PyArrow)", run_pandas_process),
    ]

    all_data = {engine_name: [] for engine_name, _ in engines}

    try:
        write_benchmark_scripts()

        for rows in ROW_SIZES:
            print(f"\n--- Testing Dataset Size: {rows:,} rows ---")
            generate_dataset(rows)

            for name, runner_fn in engines:
                print(f"  Benchmarking {name} process...")
                try:
                    mean_ms = benchmark_runner(runner_fn)
                    all_data[name].append(mean_ms)
                    print(f"    -> {mean_ms:.2f} ms")
                except Exception as e:
                    print(f"    -> Failed: {e}")
                    all_data[name].append(np.nan)

        results_df = pd.DataFrame(all_data, index=ROW_SIZES)
        results_df.index.name = "Rows"

        print("\n" + "=" * 65)
        print("Final CLI Process Execution Summary (Mean Wall Time in ms)")
        print("=" * 65)
        print(results_df.to_string())

        plot_results(results_df)

    finally:
        # Cleanup temporary files
        for f in [
            DATA_FILE,
            MLISP_SCRIPT,
            POLARS_SCRIPT,
            DUCKDB_SCRIPT,
            PANDAS_SCRIPT,
        ]:
            if os.path.exists(f):
                os.remove(f)


if __name__ == "__main__":
    main()