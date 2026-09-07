import os
import subprocess
import time
import duckdb
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import polars as pl

# ---------------------------------------------------------
# Configuration
# ---------------------------------------------------------
ROW_SIZES = [1_000, 10_000, 100_000, 1_000_000, 10_000_000, 100_000_000]
WARMUP_RUNS = 1
BENCHMARK_RUNS = 3
DATA_FILE = "temp_benchmark_data.parquet"
QUERY_FILE = "bench_query.lsp"
CHART_OUTPUT = "benchmark_scaling_results.png"


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
# Engine Runners (Return Execution Time in ms)
# ---------------------------------------------------------
def get_mlisp_script(data_path: str) -> str:
    return f"""
(nth (time-expr (-> (read-parquet "{data_path}")
                (filter (> (col "col_a") (lit 100.0)))
                (with-column (col "ratio") (/ (col "col_b") (col "col_c")))
                (group-by (col "category")
                        (alias (mean (col "col_a")) "avg_a")
                        (alias (sum (col "ratio")) "total_ratio")
                        (alias (count) "n_count"))
                (sort (col "avg_a") :desc)
                (collect :streaming)))
    1)
"""


def run_mlisp() -> float:
    proc = subprocess.run(
        ["./target/release/mlisp", QUERY_FILE],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if proc.returncode != 0:
        err_msg = proc.stderr.strip() or proc.stdout.strip()
        raise RuntimeError(f"MLisp execution failed:\n{err_msg}")

    output = proc.stdout.strip().splitlines()
    elapsed_secs = float(output[-1])
    return elapsed_secs * 1000.0


def run_python_polars() -> float:
    start = time.perf_counter()

    q = (
        pl.scan_parquet(DATA_FILE)
        .filter(pl.col("col_a") > 100.0)
        .with_columns(
            (pl.col("col_b") / pl.col("col_c")).alias("ratio")
        )
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


def run_duckdb() -> float:
    query = f"""
    SELECT
        category,
        AVG(col_a) AS avg_a,
        SUM(col_b / col_c) AS total_ratio,
        COUNT(*) AS n_count
    FROM read_parquet('{DATA_FILE}')
    WHERE col_a > 100.0
    GROUP BY category
    ORDER BY avg_a DESC
    """
    start = time.perf_counter()
    _ = duckdb.sql(query).fetchall()
    return (time.perf_counter() - start) * 1000.0


def run_pandas() -> float:
    start = time.perf_counter()
    df = pd.read_parquet(
        DATA_FILE,
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
    return (time.perf_counter() - start) * 1000.0


# ---------------------------------------------------------
# Benchmark Execution & Plotting
# ---------------------------------------------------------
def benchmark_engine(runner_fn):
    for _ in range(WARMUP_RUNS):
        runner_fn()

    durations = []
    for _ in range(BENCHMARK_RUNS):
        # Collect duration directly from engine runner return value
        duration_ms = runner_fn()
        durations.append(duration_ms)

    return np.mean(durations)


def plot_results(results_df: pd.DataFrame):
    # 1x2 side-by-side layout: left = Log scale, right = Linear scale
    fig, (ax_log, ax_lin) = plt.subplots(1, 2, figsize=(16, 6), dpi=300)

    styles = {
        "MLisp": {"color": "#1f77b4", "marker": "o", "linewidth": 2.5},
        "Polars": {"color": "#ff7f0e", "marker": "s", "linewidth": 2},
        "DuckDB": {"color": "#2ca02c", "marker": "^", "linewidth": 2},
        "Pandas (PyArrow)": {
            "color": "#d62728",
            "marker": "x",
            "linewidth": 2,
        },
    }

    # --- Panel 1: Log-Log Scale (Relative Complexity & Small-Scale Detail) ---
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

    ax_log.set_xscale("log")
    ax_log.set_yscale("log")
    ax_log.set_xlabel("Dataset Size (Number of Rows)", fontsize=11, fontweight="bold")
    ax_log.set_ylabel("Execution Time (ms) - Log Scale", fontsize=11, fontweight="bold")
    ax_log.set_title("Scaling Trend (Log-Log Scale)", fontsize=12, fontweight="bold")
    ax_log.grid(True, which="both", linestyle="--", alpha=0.5)
    ax_log.legend(frameon=True)

    # --- Panel 2: Linear Time Scale (Absolute Runtime Differences) ---
    for engine in results_df.columns:
        style = styles.get(engine, {"marker": "o", "linewidth": 2})
        ax_lin.plot(
            results_df.index,
            results_df[engine],
            label=engine,
            marker=style["marker"],
            linewidth=style["linewidth"],
            color=style.get("color"),
        )

    ax_lin.set_xscale("log")  # Keep row counts spaced evenly on the x-axis
    ax_lin.set_xlabel("Dataset Size (Number of Rows)", fontsize=11, fontweight="bold")
    ax_lin.set_ylabel("Execution Time (ms) - Linear Scale", fontsize=11, fontweight="bold")
    ax_lin.set_title("Absolute Runtime (Linear Time Scale)", fontsize=12, fontweight="bold")
    ax_lin.grid(True, which="both", linestyle="--", alpha=0.5)
    ax_lin.legend(frameon=True)

    plt.suptitle("Query Scaling Benchmark: MLisp vs Other Engines", fontsize=14, fontweight="bold")
    plt.tight_layout()

    plt.savefig(CHART_OUTPUT)
    print(f"\nScaling chart successfully saved to: {CHART_OUTPUT}")
    plt.show()


def main():
    engines = [
        ("MLisp", run_mlisp),
        ("Polars", run_python_polars),
        ("DuckDB", run_duckdb),
        ("Pandas (PyArrow)", run_pandas),
    ]

    all_data = {engine_name: [] for engine_name, _ in engines}

    try:
        # Write MLisp script once pointing to temp path
        with open(QUERY_FILE, "w") as f:
            f.write(get_mlisp_script(DATA_FILE))

        for rows in ROW_SIZES:
            print(f"\n--- Testing Dataset Size: {rows:,} rows ---")
            generate_dataset(rows)

            for name, runner_fn in engines:
                print(f"  Benchmarking {name}...")
                try:
                    mean_ms = benchmark_engine(runner_fn)
                    all_data[name].append(mean_ms)
                    print(f"    -> {mean_ms:.2f} ms")
                except Exception as e:
                    print(f"    -> Failed: {e}")
                    all_data[name].append(np.nan)

        results_df = pd.DataFrame(all_data, index=ROW_SIZES)
        results_df.index.name = "Rows"

        print("\n" + "=" * 60)
        print("Final Scaling Summary Table (Mean Execution Time in ms)")
        print("=" * 60)
        print(results_df.to_string())

        plot_results(results_df)

    finally:
        pass
        #for temp_file in [DATA_FILE, QUERY_FILE]:
        #    if os.path.exists(temp_file):
        #        os.remove(temp_file)


if __name__ == "__main__":
    main()
