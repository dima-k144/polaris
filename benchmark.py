import os
import subprocess
import time
import duckdb
import numpy as np
import pandas as pd
import polars as pl

# ---------------------------------------------------------
# 1. Dataset Configuration
# ---------------------------------------------------------
DATA_PATH = "benchmark_data.parquet"
NUM_ROWS = 50_000_000
WARMUP_RUNS = 1
BENCHMARK_RUNS = 3


def generate_dataset_if_needed():
    if not os.path.exists(DATA_PATH):
        print(f"Generating {NUM_ROWS:,} rows of synthetic test data...")
        np.random.seed(42)
        df = pl.DataFrame(
            {
                "category": np.random.choice(
                    ["A", "B", "C", "D", "E"], size=NUM_ROWS
                ),
                "col_a": np.random.uniform(0, 500, size=NUM_ROWS),
                "col_b": np.random.uniform(10, 1000, size=NUM_ROWS),
                "col_c": np.random.uniform(1, 50, size=NUM_ROWS),
            }
        )
        df.write_parquet(DATA_PATH)
        print(f"Saved {DATA_PATH} ({os.path.getsize(DATA_PATH) / 1e6:.2f} MB)")


# ---------------------------------------------------------
# 2. Query Definition Across Engines
# ---------------------------------------------------------
MLISP_SCRIPT = f"""
(-> (read-parquet "{DATA_PATH}")
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
        ["./target/release/mlisp", "bench_query.lsp"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if proc.returncode != 0:
        raise RuntimeError(f"MLisp execution failed:\n{proc.stderr}")
    return proc.stdout


def run_python_polars():
    q = (
        pl.scan_parquet(DATA_PATH)
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
    FROM read_parquet('{DATA_PATH}')
    WHERE col_a > 100.0
    GROUP BY category
    ORDER BY avg_a DESC
    """
    return duckdb.sql(query).fetchall()


def run_pandas():
    # Pandas requires loading columns into memory first (pyarrow backend recommended)
    df = pd.read_parquet(
        DATA_PATH,
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
# 3. Benchmark Harness
# ---------------------------------------------------------


def benchmark_engine(name, runner_fn):
    for _ in range(WARMUP_RUNS):
        runner_fn()

    durations = []
    for _ in range(BENCHMARK_RUNS):
        start = time.perf_counter()
        runner_fn()
        durations.append((time.perf_counter() - start) * 1000)

    return {
        "engine": name,
        "mean_ms": np.mean(durations),
        "min_ms": np.min(durations),
        "std_ms": np.std(durations),
    }


def main():
    generate_dataset_if_needed()

    with open("bench_query.lsp", "w") as f:
        f.write(MLISP_SCRIPT)

    engines = [
        ("Your Language (MLisp + Rust)", run_mlisp),
        ("Python Polars", run_python_polars),
        ("DuckDB (In-Memory SQL)", run_duckdb),
        ("Python Pandas (PyArrow)", run_pandas),
    ]

    results = []
    print("\nRunning benchmarks...")
    for name, fn in engines:
        print(f"  Benchmarking {name}...")
        try:
            res = benchmark_engine(name, fn)
            results.append(res)
        except Exception as e:
            print(f"  Failed to benchmark {name}: {e}")

    baseline = results[0]["mean_ms"] if results else 1.0

    print("\n" + "=" * 72)
    print(
        f"{'Engine':<32} | {'Mean (ms)':<10} | {'Min (ms)':<10} | {'Rel. Speed':<10}"
    )
    print("-" * 72)
    for r in results:
        speedup = baseline / r["mean_ms"]
        print(
            f"{r['engine']:<32} | {r['mean_ms']:>8.2f}ms | {r['min_ms']:>8.2f}ms | {speedup:>8.2f}x"
        )
    print("=" * 72 + "\n")

    if os.path.exists("bench_query.lsp"):
        os.remove("bench_query.lsp")


if __name__ == "__main__":
    main()