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
# Engine Runners
# ---------------------------------------------------------
def get_mlisp_script(data_path: str) -> str:
    return f"""
(-> (read-parquet "{data_path}")
    (filter (> (col "col_a") (lit 100.0)))
    (with-column (col "ratio") (/ (col "col_b") (col "col_c")))
    (group-by (col "category")
              (alias (mean (col "col_a")) "avg_a")
              (alias (sum (col "ratio")) "total_ratio")
              (alias (count) "n_count"))
    (sort (col "avg_a") :desc)
    (collect))
"""


def run_mlisp():
    proc = subprocess.run(
        ["./target/release/mlisp", QUERY_FILE],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if proc.returncode != 0:
        raise RuntimeError(f"MLisp execution failed:\n{proc.stderr}")
    return proc.stdout


def run_python_polars():
    q = (
        pl.scan_parquet(DATA_FILE)
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
    return q.collect()


def run_duckdb():
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
    return duckdb.sql(query).fetchall()


def run_pandas():
    df = pd.read_parquet(
        DATA_FILE,
        columns=["category", "col_a", "col_b", "col_c"],
        engine="pyarrow",
    )
    filtered = df[df["col_a"] > 100.0].copy()
    filtered["ratio"] = filtered["col_b"] / filtered["col_c"]

    res = (
        filtered.groupby("category", as_index=False)
        .agg(
            avg_a=("col_a", "mean"),
            total_ratio=("ratio", "sum"),
            n_count=("category", "count"),
        )
        .sort_values(by="avg_a", ascending=False)
    )
    return res


# ---------------------------------------------------------
# Benchmark Execution & Plotting
# ---------------------------------------------------------
def benchmark_engine(runner_fn):
    for _ in range(WARMUP_RUNS):
        runner_fn()

    durations = []
    for _ in range(BENCHMARK_RUNS):
        start = time.perf_counter()
        runner_fn()
        durations.append((time.perf_counter() - start) * 1000)

    return np.mean(durations)


def plot_results(results_df: pd.DataFrame):
    plt.figure(figsize=(10, 6), dpi=300)

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

    for engine in results_df.columns:
        style = styles.get(engine, {"marker": "o", "linewidth": 2})
        plt.plot(
            results_df.index,
            results_df[engine],
            label=engine,
            marker=style["marker"],
            linewidth=style["linewidth"],
            color=style.get("color"),
        )

    plt.xscale("log")
    plt.yscale("log")
    plt.xlabel("Dataset Size (Number of Rows)", fontsize=11, fontweight="bold")
    plt.ylabel("Execution Time (ms) - Log Scale", fontsize=11, fontweight="bold")
    plt.title(
        "Query Scaling Benchmark: MLisp vs Other Engines",
        fontsize=13,
        fontweight="bold",
    )
    plt.grid(True, which="both", linestyle="--", alpha=0.5)
    plt.legend(frameon=True)
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
        # Cleanup temporary files
        for temp_file in [DATA_FILE, QUERY_FILE]:
            if os.path.exists(temp_file):
                os.remove(temp_file)


if __name__ == "__main__":
    main()