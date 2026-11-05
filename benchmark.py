import argparse
import csv
import hashlib
import inspect
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
import textwrap
import pyarrow as pa
import pyarrow.parquet as pq
import duckdb
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd
import polars as pl
import polars.testing as pl_testing

# Аналіз прибутковості продажів за місяцем, регіоном і товаром.
# Запускати з кореня Rust-проєкту, де є target/release/polaris:
#   python benchmark.py --mode verify
#   python benchmark.py --mode in-process --rows 1000000 10000000 --runs 10
# R-пакети: arrow, dplyr, lubridate. Python-залежності: як у вихідному скрипті.
# Кожне вимірювання виконує ОДИН складний запит; повторювального циклу немає.
# Кожен запит читає ОБИДВІ таблиці; генерація даних не входить у час.
SCENARIO = "retail_profitability_v1"
ROW_SIZES = [1_000, 10_000, 100_000, 1_000_000, 10_000_000, 100_000_000]
WARMUP_RUNS = 1
BENCHMARK_RUNS = 10
THREADS = max(1, len(os.sched_getaffinity(0)) if hasattr(os, "sched_getaffinity") else (os.cpu_count() or 1))
DATASET_CHUNK_ROWS = 250_000
PRODUCT_COUNT = 20_000
REGION_COUNT = 12
PRODUCT_FILE_NAME = "temp_retail_v1_products.parquet"
GROUP_KEYS = ["month", "region_id", "product_id", "category_id"]
MIN_TRANSACTIONS = 2
VERIFY_COLUMNS = GROUP_KEYS + ["transaction_count", "units_sold", "revenue", "profit", "avg_order_value", "unique_customers"]
EXACT_COLUMNS = GROUP_KEYS + ["transaction_count", "units_sold", "unique_customers"]
FLOAT_COLUMNS = ["revenue", "profit", "avg_order_value"]
SORT_COLUMNS = ["profit"] + GROUP_KEYS

POLARIS_BIN = "./target/release/polaris"
POLARIS_SCRIPT = "bench_query.lsp"
POLARS_SCRIPT = "bench_polars.py"
DUCKDB_SCRIPT = "bench_duckdb.py"
PANDAS_SCRIPT = "bench_pandas.py"
R_SCRIPT = "bench_r.R"
R_INPROCESS_SCRIPT = "bench_r_inprocess.R"


def get_data_filepath(num_rows: int) -> str:
    # New names prevent reuse of the old category/col_a/col_b dataset.
    return f"temp_retail_v1_transactions_{num_rows}.parquet"


def product_path_for(data_file):
    return str(Path(data_file).resolve().with_name(PRODUCT_FILE_NAME))


def ensure_products(data_file):
    path = Path(product_path_for(data_file))
    if path.exists():
        expected = {"product_id", "category_id", "unit_cost"}
        if set(pq.read_schema(path).names) != expected or pq.ParquetFile(path).metadata.num_rows != PRODUCT_COUNT:
            raise ValueError(f"Несумісний довідник товарів: {path}")
        return str(path)
    ids = np.arange(1, PRODUCT_COUNT + 1, dtype=np.int64)
    table = pa.table({"product_id": ids, "category_id": (ids - 1) % 100 + 1,
                      "unit_cost": 5.0 + (ids % 500) / 10.0})
    temporary = str(path) + ".partial"
    try:
        pq.write_table(table, temporary, compression="zstd")
        os.replace(temporary, path)
    finally:
        Path(temporary).unlink(missing_ok=True)
    return str(path)


def ensure_dataset(num_rows: int) -> str:
    """Reproducible transactional data in bounded batches; no nulls or orphan keys."""
    file_path = get_data_filepath(num_rows)
    ensure_products(file_path)
    schema = pa.schema([
        ("sale_date", pa.date32()), ("region_id", pa.int64()),
        ("product_id", pa.int64()), ("customer_id", pa.int64()),
        ("quantity", pa.int64()), ("unit_price", pa.float64()), ("discount", pa.float64()),
    ])
    if os.path.exists(file_path):
        metadata = pq.ParquetFile(file_path)
        if not metadata.schema_arrow.equals(schema, check_metadata=False) or metadata.metadata.num_rows != num_rows:
            raise ValueError(f"Несумісний набір транзакцій: {file_path}")
        print(f"  Файл {file_path} вже існує. Генерацію пропущено.")
        return file_path
    print(f"  Генерація {num_rows:,} транзакцій і довідника з {PRODUCT_COUNT:,} товарів...")
    # Floating uniform draws, one independent RNG per column: invariant to chunk size.
    rngs = [np.random.default_rng(seed) for seed in range(42, 49)]
    temporary_path = file_path + ".partial"
    start_date = np.datetime64("2024-01-01", "D")
    try:
        with pq.ParquetWriter(temporary_path, schema, compression="zstd") as writer:
            for offset in range(0, num_rows, DATASET_CHUNK_ROWS):
                size = min(DATASET_CHUNK_ROWS, num_rows - offset)
                day = (rngs[0].random(size) * 731).astype(np.int64)
                region = (rngs[1].random(size) * REGION_COUNT).astype(np.int64) + 1
                product = (rngs[2].random(size) * PRODUCT_COUNT).astype(np.int64) + 1
                customer = (rngs[3].random(size) * 1_000_000).astype(np.int64) + 1
                quantity = (rngs[4].random(size) * 11).astype(np.int64)  # 0..10
                price = (rngs[5].random(size) * 20_000).astype(np.int64) / 100.0 + 10.0
                discount = (rngs[6].random(size) * 41).astype(np.int64) / 100.0  # 0..40%
                # Small deterministic fixture also makes the 1000-row control nonempty.
                # Three repeated profitable groups, including two different years.
                absolute = np.arange(offset, offset + size)
                fixture = absolute < 24
                group = absolute[fixture] // 8
                day[fixture] = np.choose(group, [0, 366, 397])
                region[fixture] = group + 1
                product[fixture] = group + 1
                customer[fixture] = absolute[fixture] % 4 + 1
                quantity[fixture] = 2
                price[fixture] = 100.0
                discount[fixture] = 0.1
                # Boundary/exclusion cases: quantity=0, discount=0.30, zero/negative profit.
                for idx, values in {
                    24: (0, 0.1, 100.0), 25: (2, 0.30, 100.0),
                    26: (2, 0.1, 1.0), 27: (2, 0.1, 1.0),
                    28: (2, 0.0, 5.6), 29: (2, 0.0, 5.6),
                }.items():
                    if offset <= idx < offset + size:
                        local = idx - offset
                        day[local], region[local], product[local], customer[local] = 0, 12, 6, 1
                        quantity[local], discount[local], price[local] = values
                dates = start_date + day.astype("timedelta64[D]")
                table = pa.Table.from_arrays([
                    pa.array(dates, type=pa.date32()), pa.array(region), pa.array(product),
                    pa.array(customer), pa.array(quantity), pa.array(price), pa.array(discount),
                ], schema=schema)
                writer.write_table(table, row_group_size=DATASET_CHUNK_ROWS)
        os.replace(temporary_path, file_path)
    finally:
        Path(temporary_path).unlink(missing_ok=True)
    return file_path


# Identical logical work in all stacks: filter -> join -> derived columns ->
# grouped aggregates -> group filter -> deterministic full sort -> materialize.
def polars_query(data_file):
    return (
        pl.scan_parquet(data_file)
        .filter((pl.col("quantity") > 0) & (pl.col("discount") < 0.30))
        .join(pl.scan_parquet(product_path_for(data_file)), on="product_id", how="inner")
        .with_columns(
            (pl.col("sale_date").dt.year() * 100 + pl.col("sale_date").dt.month()).alias("month"),
            (pl.col("quantity") * pl.col("unit_price") * (1.0 - pl.col("discount"))).alias("net_revenue"),
        )
        .with_columns((pl.col("net_revenue") - pl.col("quantity") * pl.col("unit_cost")).alias("net_profit"))
        .group_by(GROUP_KEYS)
        .agg(pl.len().alias("transaction_count"), pl.col("quantity").sum().alias("units_sold"),
             pl.col("net_revenue").sum().alias("revenue"), pl.col("net_profit").sum().alias("profit"),
             pl.col("net_revenue").mean().alias("avg_order_value"),
             pl.col("customer_id").n_unique().alias("unique_customers"))
        .filter((pl.col("transaction_count") >= MIN_TRANSACTIONS) & (pl.col("profit") > 0.0))
        .select(VERIFY_COLUMNS)
        .sort(SORT_COLUMNS, descending=[True] + [False] * len(GROUP_KEYS))
    )


def pandas_result(data_file):
    df = pd.read_parquet(data_file, engine="pyarrow", dtype_backend="pyarrow",
                         filters=[("quantity", ">", 0), ("discount", "<", 0.30)])
    products = pd.read_parquet(product_path_for(data_file), engine="pyarrow", dtype_backend="pyarrow")
    df = df.merge(products, on="product_id", how="inner")
    df["month"] = df["sale_date"].dt.year * 100 + df["sale_date"].dt.month
    df["net_revenue"] = df["quantity"] * df["unit_price"] * (1.0 - df["discount"])
    df["net_profit"] = df["net_revenue"] - df["quantity"] * df["unit_cost"]
    result = df.groupby(GROUP_KEYS, as_index=False).agg(
        transaction_count=("quantity", "size"), units_sold=("quantity", "sum"),
        revenue=("net_revenue", "sum"), profit=("net_profit", "sum"),
        avg_order_value=("net_revenue", "mean"), unique_customers=("customer_id", "nunique"),
    )
    result = result[(result["transaction_count"] >= MIN_TRANSACTIONS) & (result["profit"] > 0.0)]
    return result[VERIFY_COLUMNS].sort_values(SORT_COLUMNS, ascending=[False] + [True] * len(GROUP_KEYS))


def duckdb_query(data_file):
    def sql_string(value):
        return "'" + str(value).replace("'", "''") + "'"
    return f"""
WITH enriched AS (
    SELECT CAST(year(t.sale_date) * 100 + month(t.sale_date) AS BIGINT) AS month,
           t.region_id, t.product_id, p.category_id, t.customer_id, t.quantity,
           t.quantity * t.unit_price * (1.0 - t.discount) AS net_revenue,
           t.quantity * t.unit_price * (1.0 - t.discount) - t.quantity * p.unit_cost AS net_profit
    FROM read_parquet({sql_string(data_file)}) t
    INNER JOIN read_parquet({sql_string(product_path_for(data_file))}) p USING (product_id)
    WHERE t.quantity > 0 AND t.discount < 0.30
)
SELECT month, region_id, product_id, category_id, COUNT(*) AS transaction_count,
       CAST(SUM(quantity) AS BIGINT) AS units_sold, SUM(net_revenue) AS revenue, SUM(net_profit) AS profit,
       AVG(net_revenue) AS avg_order_value, COUNT(DISTINCT customer_id) AS unique_customers
FROM enriched
GROUP BY month, region_id, product_id, category_id
HAVING COUNT(*) >= {MIN_TRANSACTIONS} AND SUM(net_profit) > 0.0
ORDER BY profit DESC, month, region_id, product_id, category_id
"""


def polaris_query(data_file, terminal="collect :streaming"):
    path = json.dumps(str(Path(data_file).resolve()), ensure_ascii=False)
    products = json.dumps(product_path_for(data_file), ensure_ascii=False)
    return f"""-> (read-parquet {path})
    filter (> $quantity 0) (< $discount 0.30)
    join (read-parquet {products}) $product_id $product_id :inner
    with-column $month (+ (* (dt-year $sale_date) 100) (dt-month $sale_date))
    with-column $net_revenue (* $quantity $unit_price (- 1.0 $discount))
    with-column $net_profit (- $net_revenue (* $quantity $unit_cost))
    group-by (list $month $region_id $product_id $category_id)
        alias (count) "transaction_count"
        alias (sum $quantity) "units_sold"
        alias (sum $net_revenue) "revenue"
        alias (sum $net_profit) "profit"
        alias (mean $net_revenue) "avg_order_value"
        alias (n-unique $customer_id) "unique_customers"
    filter (>= $transaction_count {MIN_TRANSACTIONS}) (> $profit 0.0)
    select $month $region_id $product_id $category_id $transaction_count $units_sold $revenue $profit $avg_order_value $unique_customers
    sort (list $profit $month $region_id $product_id $category_id) (list :desc :asc :asc :asc :asc)
    {terminal}
"""


def r_pipeline(data_file):
    path = json.dumps(str(Path(data_file).resolve()), ensure_ascii=False)
    products = json.dumps(product_path_for(data_file), ensure_ascii=False)
    return f"""ds <- open_dataset({path}, format = "parquet")
products <- open_dataset({products}, format = "parquet")
res <- ds |>
    filter(quantity > 0, discount < 0.30) |>
    inner_join(products, by = "product_id") |>
    mutate(month = lubridate::year(sale_date) * 100 + lubridate::month(sale_date),
           net_revenue = quantity * unit_price * (1.0 - discount)) |>
    mutate(net_profit = net_revenue - quantity * unit_cost) |>
    group_by(month, region_id, product_id, category_id) |>
    summarise(transaction_count = n(), units_sold = sum(quantity),
              revenue = sum(net_revenue), profit = sum(net_profit),
              avg_order_value = mean(net_revenue), unique_customers = n_distinct(customer_id),
              .groups = "drop") |>
    filter(transaction_count >= {MIN_TRANSACTIONS}, profit > 0.0) |>
    select(month, region_id, product_id, category_id, transaction_count, units_sold,
           revenue, profit, avg_order_value, unique_customers) |>
    arrange(desc(profit), month, region_id, product_id, category_id) |>
    collect()
"""


def r_header():
    return ('suppressPackageStartupMessages({library(arrow); library(dplyr); library(lubridate)})\n'
            f'arrow::set_cpu_count({THREADS})\n')


# ---------------------------------------------------------
# РЕЖИМ 1: In-process: побудова і виконання запиту після прогріву
# ---------------------------------------------------------
def get_polaris_inprocess_script(data_path: str) -> str:
    query = polaris_query(data_path)
    return ((query + "\n") * WARMUP_RUNS + "define result\n    time-expr\n"
            + textwrap.indent(query, "        ") + "\nnth result 1\n")


def write_r_inprocess_script(data_path: str):
    code = (r_header() + "run_pipeline <- function() {\n" + textwrap.indent(r_pipeline(data_path), "    ")
            + "    invisible(res)\n}\n"
            + f"for (i in seq_len({WARMUP_RUNS})) run_pipeline()\n"
            + 'start <- proc.time()[["elapsed"]]\nres <- run_pipeline()\n'
            + 'elapsed <- proc.time()[["elapsed"]] - start\ncat(sprintf("%.9f\\n", elapsed))\n')
    Path(R_INPROCESS_SCRIPT).write_text(code, encoding="utf-8")


def run_internal_script(command):
    proc = subprocess.run(command, capture_output=True, text=True, env=worker_environment())
    if proc.returncode or "Error:" in proc.stdout:
        raise RuntimeError(proc.stderr.strip() or proc.stdout.strip())
    return float(proc.stdout.strip().splitlines()[-1]) * 1000.0


def run_inprocess_polaris(data_file: str) -> float:
    Path(POLARIS_SCRIPT).write_text(get_polaris_inprocess_script(data_file), encoding="utf-8")
    return run_internal_script([POLARIS_BIN, POLARIS_SCRIPT])


def run_inprocess_r(data_file: str) -> float:
    write_r_inprocess_script(data_file)
    return run_internal_script(["Rscript", "--vanilla", R_INPROCESS_SCRIPT])


def run_inprocess_polars(data_file: str) -> float:
    start = time.perf_counter()
    _ = polars_query(data_file).collect(engine="streaming")
    return (time.perf_counter() - start) * 1000.0


def run_inprocess_duckdb(data_file: str) -> float:
    start = time.perf_counter()
    _ = duckdb.sql(duckdb_query(data_file)).fetchall()
    return (time.perf_counter() - start) * 1000.0


def run_inprocess_pandas(data_file: str) -> float:
    start = time.perf_counter()
    _ = pandas_result(data_file)
    return (time.perf_counter() - start) * 1000.0


# ---------------------------------------------------------
# РЕЖИМ 2: повний запуск CLI; ті самі функції запитів
# ---------------------------------------------------------
def python_process_script(engine, data_file):
    imports = {"polars": "import polars as pl", "pandas": "import pandas as pd\nimport pyarrow as pa",
               "duckdb": "import duckdb"}[engine]
    code = imports + "\nfrom pathlib import Path\n"
    for key in ["PRODUCT_FILE_NAME", "GROUP_KEYS", "MIN_TRANSACTIONS", "VERIFY_COLUMNS", "SORT_COLUMNS"]:
        code += f"{key} = {globals()[key]!r}\n"
    code += inspect.getsource(product_path_for) + "\n"
    function, call = {"polars": (polars_query, 'polars_query(DATA_FILE).collect(engine="streaming")'),
                      "pandas": (pandas_result, "pandas_result(DATA_FILE)"),
                      "duckdb": (duckdb_query, "duckdb.sql(duckdb_query(DATA_FILE)).fetchall()")}[engine]
    code += inspect.getsource(function) + "\n"
    if engine == "duckdb":
        code += f'duckdb.sql("SET threads = {THREADS}")\n'
    if engine == "pandas":
        code += f"pa.set_cpu_count({THREADS})\n"
    code += f"DATA_FILE = {str(Path(data_file).resolve())!r}\n_ = {call}\n"
    return code


def write_process_scripts(data_file: str, script_directory=None):
    directory = Path(script_directory or ".")
    scripts = {POLARIS_SCRIPT: polaris_query(data_file), R_SCRIPT: r_header() + r_pipeline(data_file),
               POLARS_SCRIPT: python_process_script("polars", data_file),
               PANDAS_SCRIPT: python_process_script("pandas", data_file),
               DUCKDB_SCRIPT: python_process_script("duckdb", data_file)}
    for filename, code in scripts.items():
        (directory / Path(filename).name).write_text(code, encoding="utf-8")


def run_command(cmd: list[str]) -> float:
    start = time.perf_counter()
    proc = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, env=worker_environment())
    elapsed_ms = (time.perf_counter() - start) * 1000.0
    if proc.returncode or "Error:" in proc.stdout:
        raise RuntimeError(proc.stderr.strip() or proc.stdout.strip())
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
    binary = Path(POLARIS_BIN)
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
        "scenario": SCENARIO,
        "query": "Filter transactions; inner join products; compute YYYYMM/revenue/profit; group by month/region/product/category; aggregate count/units/revenue/profit/average/distinct customers; retain count>=2 and profit>0; full sort.",
        "product_count": PRODUCT_COUNT, "region_count": REGION_COUNT,
        "date_range": ["2024-01-01", "2025-12-31"],
        "minimum_transactions_per_group": MIN_TRANSACTIONS,
        "timed_scope": "Reading both Parquet files, query construction and execution; no result-file writes.",
        "platform": platform.platform(), "cpu_model": cpu_model,
        "logical_cpus": os.cpu_count(), "ram_bytes": ram_bytes,
        "python": sys.version, "python_executable": sys.executable,
        "versions": {"polars_python": pl.__version__, "duckdb": duckdb.__version__,
                     "pandas": pd.__version__, "pyarrow": pa.__version__, "numpy": np.__version__},
        "rust_polars_from_cargo_lock": rust_polars,
        "rust_polars_note": "Cargo.lock describes local source dependencies; it does not prove which version built the binary.",
        "rustc": probe_command(["rustc", "--version"]),
        "r": probe_command(["Rscript", "--vanilla", "-e",
            'cat(R.version.string, "\\n"); for (p in c("arrow", "dplyr", "lubridate")) cat(p, as.character(packageVersion(p)), "\\n")']),
        "polaris_binary": str(binary.resolve()), "polaris_binary_sha256": binary_hash,
        "polaris_build_note": args.polaris_build_note or "Not supplied; release path alone does not prove build flags.",
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
        "dataset_generation": "Bounded Arrow batches, independent RNG seeds 42..48, zstd; 24 repeated fixture rows and 6 boundary rows; products cost formula 5+(id%500)/10.",
        "dataset_chunk_rows": DATASET_CHUNK_ROWS,
        "scope": "Specific analytic stacks and one query; not a general language-speed ranking.",
    }


def plot_results(results_df: pd.DataFrame, title: str, output_file: str):
    # CSV stores milliseconds; all chart coordinates use seconds.
    results_df = results_df / 1000.0
    styles = {
        "Polaris": {"color": "#1f77b4", "marker": "o", "linewidth": 2.5},          # Blue
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
        ("log", "Логарифмічний граф часу", "Час виконання (с) — логарифмічна шкала"),
        ("linear", "Лінійний граф часу", "Час виконання (с) — лінійна шкала"),
    ]:
        fig, ax = plt.subplots(figsize=(8, 6), dpi=300)
        for engine in results_df.columns:
            style = styles.get(engine, {"marker": "o", "linewidth": 2})
            ax.plot(results_df.index, results_df[engine], label="Розроблена мова" if engine == "Polaris" else engine, **style)

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
        "Polaris": "#1f77b4", "Polars (Python)": "#ff7f0e", "DuckDB": "#2ca02c",
        "Pandas (PyArrow)": "#d62728", "R (Arrow)": "#9467bd",
    }
    labels = [("Розроблена мова" if name == "Polaris" else name).replace(" (", "\n(")
              for name in engine_names]
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
                float(record["elapsed_ms"]) / 1000.0 for record in records
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
        ax.set_ylabel("Час виконання (с), логарифмічна шкала", fontsize=10)
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



def scalar_reference(transactions, products):
    """Independent untimed Python calculation for the small verification fixture."""
    import math
    catalog = {row["product_id"]: row for row in products.to_dicts()}
    groups = {}
    for row in transactions.to_dicts():
        if row["quantity"] <= 0 or row["discount"] >= 0.30:
            continue
        product = catalog.get(row["product_id"])
        if product is None:
            continue
        date = row["sale_date"]
        key = (date.year * 100 + date.month, row["region_id"], row["product_id"], product["category_id"])
        group = groups.setdefault(key, {"units": 0, "revenues": [], "profits": [], "customers": set()})
        revenue = row["quantity"] * row["unit_price"] * (1.0 - row["discount"])
        group["units"] += row["quantity"]
        group["revenues"].append(revenue)
        group["profits"].append(revenue - row["quantity"] * product["unit_cost"])
        group["customers"].add(row["customer_id"])
    output = []
    for key, group in groups.items():
        count, profit = len(group["revenues"]), math.fsum(group["profits"])
        if count < MIN_TRANSACTIONS or profit <= 0.0:
            continue
        revenue = math.fsum(group["revenues"])
        output.append(dict(zip(VERIFY_COLUMNS, [*key, count, group["units"], revenue,
                                                 profit, revenue / count, len(group["customers"])])))
    if not output:
        raise AssertionError("Контрольний набір не дав жодної групи.")
    return pl.DataFrame(output).select(VERIFY_COLUMNS).sort(
        SORT_COLUMNS, descending=[True] + [False] * len(GROUP_KEYS))


def compare_verify_tables(expected, actual, rel_tol=1e-8, abs_tol=1e-6):
    """Compare keys/counts exactly, floating aggregates tolerantly; check sorted output."""
    for name, frame in [("reference", expected), ("actual", actual)]:
        if len(frame.columns) != len(VERIFY_COLUMNS) or set(frame.columns) != set(VERIFY_COLUMNS):
            raise AssertionError(f"{name}: unexpected columns {frame.columns}")
        if frame.height == 0 or frame.select(GROUP_KEYS).unique().height != frame.height:
            raise AssertionError(f"{name}: empty result or duplicate group keys")
        if any(frame.null_count().row(0)):
            raise AssertionError(f"{name}: unexpected null values")
        for column in VERIFY_COLUMNS:
            if not frame[column].dtype.is_numeric() or not frame[column].cast(pl.Float64).is_finite().all():
                raise AssertionError(f"{name}: invalid numeric values in {column}")
        if frame.filter((pl.col("transaction_count") < MIN_TRANSACTIONS) | (pl.col("profit") <= 0)).height:
            raise AssertionError(f"{name}: group filter was not applied")
        # Sorting is part of the workload; verify it before comparing keyed contents.
        pl_testing.assert_frame_equal(frame, frame.sort(SORT_COLUMNS, descending=[True] + [False] * len(GROUP_KEYS)))
    expected = expected.select(VERIFY_COLUMNS).sort(GROUP_KEYS)
    actual = actual.select(VERIFY_COLUMNS).sort(GROUP_KEYS)
    pl_testing.assert_frame_equal(actual.select(EXACT_COLUMNS), expected.select(EXACT_COLUMNS),
                                  check_dtypes=False, check_exact=True)
    pl_testing.assert_frame_equal(actual.select(FLOAT_COLUMNS), expected.select(FLOAT_COLUMNS),
                                  check_dtypes=False, check_exact=False, rel_tol=rel_tol, abs_tol=abs_tol)


def run_verify(output_directory="benchmark_retail_verify_results"):
    # Fixed control sample; independent of --rows used for timing runs.
    data_file = str(Path(ensure_dataset(VERIFY_ROWS)).resolve())
    control = pl.read_parquet(data_file)
    if control.height != VERIFY_ROWS:
        raise ValueError("Контрольний набір має містити 1000 транзакцій.")
    products = pl.read_parquet(product_path_for(data_file))
    if products["product_id"].n_unique() != PRODUCT_COUNT:
        raise ValueError("Ключі довідника товарів мають бути унікальними.")
    filtered = control.filter((pl.col("quantity") > 0) & (pl.col("discount") < 0.30))
    if not 0 < filtered.height < VERIFY_ROWS:
        raise ValueError("Контрольний набір має перевіряти відкидання транзакцій.")
    # An independent scalar reference checks the query, not just agreement of engines.
    expected_control = scalar_reference(control, products)
    output_dir = Path(output_directory).resolve()
    output_dir.mkdir(parents=True, exist_ok=True)
    engines = [
        ("Polaris", "polaris", POLARIS_SCRIPT),
        ("Polars (Python)", "polars", POLARS_SCRIPT),
        ("DuckDB", "duckdb", DUCKDB_SCRIPT),
        ("Pandas (PyArrow)", "pandas", PANDAS_SCRIPT),
        ("R (Arrow)", "r", R_SCRIPT),
    ]
    report = {"rows": VERIFY_ROWS, "data_file": data_file, "reference": "Independent Python scalar calculation",
              "rel_tol": 1e-8, "abs_tol": 1e-6, "scenario": SCENARIO,
              "products_file": product_path_for(data_file),
              "comparison": "Full grouped contents and output sorting; keys/counts exact; floats tolerant; independent scalar reference.",
              "polaris_note": "Polaris collect returns nil; its verification uses sink with the same preceding pipeline.",
              "results": []}
    print(f"\n--- [VERIFY] Порівняння всіх систем: {VERIFY_ROWS} рядків ---")
    with tempfile.TemporaryDirectory(prefix="benchmark_verify_scripts_") as scripts_dir:
        write_process_scripts(data_file, scripts_dir)
        for name, key, filename in engines:
            output_file = output_dir / f"{key}_result.parquet"
            output_file.unlink(missing_ok=True)  # Never compare stale output after a failed launch.
            script_file = Path(scripts_dir) / Path(filename).name
            code = script_file.read_text()
            if key == "polaris":
                if code.count("collect :streaming") != 1:
                    raise ValueError("Не вдалося знайти завершення Polaris-запиту для перевірки.")
                code = code.replace("collect :streaming", f"sink {json.dumps(output_file.as_posix(), ensure_ascii=False)}")
                command = [str(Path(POLARIS_BIN).resolve()), str(script_file)]
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
                if proc.returncode or "Error:" in proc.stdout:
                    raise RuntimeError(proc.stderr.strip() or proc.stdout.strip() or f"Exit code {proc.returncode}")
                if not output_file.is_file():
                    raise RuntimeError("Система не створила вихідний файл.")
                pl.read_parquet(output_file)  # Ensure output is readable before comparing.
                print(f"  {name}: результат записано")
            except Exception as exc:
                result.update(status="execution_error", error=str(exc))
                print(f"  {name}: помилка запуску — {exc}")
            report["results"].append(result)

    for result in report["results"]:
        if result["status"] == "execution_error":
            continue
        try:
            compare_verify_tables(expected_control, pl.read_parquet(result["file"]),
                                  rel_tol=report["rel_tol"], abs_tol=report["abs_tol"])
            result["status"] = "ok"
            print(f"  {result['engine']}: значення збігаються ✅")
        except Exception as exc:
            result.update(status="mismatch", error=str(exc))
            print(f"  {result['engine']}: відмінність даних — {exc}")
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
    parser.add_argument("--results-dir", default="benchmark_results",
                        help="Батьківський каталог для нової папки кожного запуску.")
    parser.add_argument("--csv-output", default="benchmark_retail_all_runs.csv",
                        help="Ім’я CSV або відносний шлях усередині папки поточного запуску.")
    parser.add_argument("--runs", type=int, default=BENCHMARK_RUNS)
    parser.add_argument("--warmup-runs", type=int, default=WARMUP_RUNS)
    parser.add_argument("--threads", type=int, default=THREADS)
    parser.add_argument("--rows", type=int, nargs="+", default=ROW_SIZES)
    parser.add_argument("--chunk-rows", type=int, default=DATASET_CHUNK_ROWS)
    parser.add_argument("--verify-output-dir", default="benchmark_retail_verify_results", help="Відносний підкаталог перевірки всередині папки поточного запуску.")
    parser.add_argument("--seed", type=int, default=42, help="Відтворюване перемішування порядку систем.")
    parser.add_argument("--polaris-build-note", default="", help="Версія Rust Polars, команда збірки, RUSTFLAGS тощо.")
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

    # Workers report a single timing to the parent; only the parent creates a folder.
    for option, value in [("--csv-output", args.csv_output),
                          ("--verify-output-dir", args.verify_output_dir)]:
        path = Path(value)
        if path.is_absolute() or ".." in path.parts:
            parser.error(f"{option}: потрібен відносний шлях усередині папки запуску; "
                         "для іншого місця використайте --results-dir.")
    started_at = datetime.now().astimezone()
    results_directory = Path(args.results_dir).expanduser().resolve() / started_at.strftime("%Y-%m-%d_%H-%M-%S_%f")
    results_directory.mkdir(parents=True, exist_ok=False)
    args.csv_output = str(results_directory / args.csv_output)
    args.verify_output_dir = str(results_directory / args.verify_output_dir)
    print(f"Папка результатів: {results_directory}", flush=True)

    if args.mode == "verify":
        run_verify(args.verify_output_dir)
        return

    if args.mode == "in-process":
        chart_output = "benchmark_retail_inprocess_results.png"
        title = "Час запиту для аналітичних систем після прогріву"
        engines = [
            ("Polaris", run_inprocess_polaris),
            ("Polars (Python)", lambda f: run_python_trial("polars", f)),
            ("DuckDB", lambda f: run_python_trial("duckdb", f)),
            ("Pandas (PyArrow)", lambda f: run_python_trial("pandas", f)),
            ("R (Arrow)", run_inprocess_r),
        ]
    else:
        chart_output = "benchmark_retail_process_results.png"
        title = "Час повного запуску аналітичного скрипту"
        engines = [
            ("Polaris", lambda _: run_command([POLARIS_BIN, POLARIS_SCRIPT])),
            ("Polars (Python)", lambda _: run_command([sys.executable, POLARS_SCRIPT])),
            ("DuckDB", lambda _: run_command([sys.executable, DUCKDB_SCRIPT])),
            ("Pandas (PyArrow)", lambda _: run_command([sys.executable, PANDAS_SCRIPT])),
            ("R (Arrow)", lambda _: run_command(["Rscript", "--vanilla", R_SCRIPT])),
        ]

    chart_output = str(results_directory / chart_output)

    session_id = f"{started_at.astimezone(timezone.utc).isoformat()}_{uuid.uuid4().hex[:8]}"
    csv_columns = ["session_id", "scenario", "mode", "rows", "engine", "phase", "run", "elapsed_ms", "status", "error"]
    output_path = Path(args.csv_output)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    session_suffix = session_id.replace(":", "-")
    metadata_path = output_path.with_name(f"{output_path.stem}_{session_suffix}_metadata.json")
    summary_path = output_path.with_name(f"{output_path.stem}_{session_suffix}_summary.csv")
    rng = random.Random(args.seed)
    records = []
    metadata = environment_metadata(args)
    metadata.update(session_id=session_id, mode=args.mode, datasets=[],
                    results_directory=str(results_directory), started_at_local=started_at.isoformat())
    metadata_path.write_text(json.dumps(metadata, indent=2, ensure_ascii=False), encoding="utf-8")
    print("Сценарій: прибутковість товарів за місяцями й регіонами; join та 6 агрегатів.")
    print("Порівнюються конкретні програмні стеки; версії й умови записано в JSON.")
    print("Файловий кеш прогрівається перед кожним запуском; холодне читання не вимірюється.")
    print("Для Polaris версію Rust Polars і прапорці збірки слід підтвердити через --polaris-build-note.")

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
                product_file = Path(product_path_for(current_data_file))
                metadata["datasets"][-1]["products"] = {
                    "path": str(product_file), "rows": PRODUCT_COUNT,
                    "size_bytes": product_file.stat().st_size, "mtime_ns": product_file.stat().st_mtime_ns,
                }
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
                            record = dict(session_id=session_id, scenario=SCENARIO, mode=args.mode, rows=rows, engine=name,
                                          phase=phase, run=run_number, elapsed_ms="", status="ok", error="")
                            try:
                                warm_file_cache(current_data_file)
                                warm_file_cache(product_path_for(current_data_file))
                                elapsed = runner_fn(current_data_file)
                                if not np.isfinite(elapsed) or elapsed <= 0:
                                    raise ValueError(f"Некоректний час: {elapsed}; можливо, роздільності таймера недостатньо.")
                                record["elapsed_ms"] = elapsed
                                print(f"  {name}: {phase} {run_number}/{count} — {elapsed / 1000.0:.6f} с")
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
        for f in [POLARIS_SCRIPT, POLARS_SCRIPT, DUCKDB_SCRIPT, PANDAS_SCRIPT, R_SCRIPT, R_INPROCESS_SCRIPT]:
            if os.path.exists(f):
                os.remove(f)


if __name__ == "__main__":
    main()
