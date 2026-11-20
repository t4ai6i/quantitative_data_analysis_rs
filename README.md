![example workflow](https://github.com/t4ai6i/quantitative_data_analysis_rs/actions/workflows/rust.yml/badge.svg)

# quantitative_data_analysis_rs

Quantitative market analysis toolkit written in Rust. It ingests historical stock data plus fundamentals from J-Quants
and Yahoo Finance to produce chart-ready signals, summaries, and SVG/JSON artifacts for reporting.

## Features

- Hexagonal-style architecture (`domain`, `use_case`, `controller`, `presenter`, `infrastructure`, `shared`) keeps
  market logic testable and interchangeable.
- Trend analysis engine detects crossover patterns, MACO signals, candlestick patterns (marubozu, doji), and aggregates
  them into summaries.
- Financial indicator analyzer correlates statements with the latest available prices to generate JSON-friendly metrics.
- CSV/TSV ingestion utilities plus ready-made assets under `assets/` accelerate local experiments.
- Async-first data access via J-Quants REST API and Yahoo Finance scrapers, including helper tooling for token rotation.

## Scoring universe policy

When selecting the universe for scoring from J-Quants master data, we use both `ProdCat` and `Mkt`:

- `ProdCat = "011"` keeps ordinary domestic listed stocks and excludes ETFs, REITs, foreign stocks, and similar products.
- `Mkt = "0111" | "0112" | "0113"` keeps only Prime, Standard, and Growth listings and excludes other market codes such as TOKYO PRO MARKET.

This keeps the scoring candidate set focused on the common Japanese equity universe used by the analyzers.

## Installation

1. Install the Rust toolchain specified in `rust-toolchain.toml` (Rust 1.90.0). `rustup` automatically picks it up when
   you
   enter the repo.
2. Install build essentials for your OS (LLVM/Clang on macOS, build-essential on Linux).
3. Clone the repository and fetch dependencies:

```bash
cargo fetch
```

> This repository intentionally tracks `Cargo.lock` to keep CI and J-Quants connectivity checks reproducible.

## Usage

- **Financial indicators:**

  ```bash
  cargo run --example financial_indicator
  ```

- **Trend analysis (SVG + JSON outputs):**

  ```bash
  cargo run --example trend_analysis
  ```

- **Company master TSV generator:**

  ```bash
  cargo run --example make_companies
  ```

Generated artifacts are written to `examples/` (SVG charts) and compared against the golden files in `assets/` for
regression safety.

## Configuration

### J-Quants financial disclosures

The J-Quants statement repository retrieves every `/v2/fins/summary` page and
returns disclosure rows in ascending disclosure date and time (JST). Rows with
the same timestamp retain their source order; disclosure numbers do not establish
their chronological order. A failed page, invalid response, or malformed nonempty
field is an error, not a partial history. Rate limiting and retries remain the
caller's responsibility.

`FetchScoringData.financial_disclosures` contains the API's FY, quarterly, and
forecast-revision rows. Each entry retains its disclosure identifier and document
type, period and fiscal-year dates, and operating-profit actual and forecast
fields, including the `NC` variants. Amounts are in the units provided by
J-Quants (normally yen). Missing values are `null`; they do not mean unchanged or
withdrawn. `FOP` refers to the current fiscal year, `NxFOP` to the next fiscal
year. Compare both fiscal-year boundaries before associating forecasts.

The API's `OP` is preserved with its original period dates and
`operating_profit_period: "unspecified"`. `ScoreStock.quarterly_actuals` derives
single-quarter sales, operating profit, ordinary profit, and net income from
adjacent cumulative periods only when fiscal-year boundaries, period dates,
document basis, accounting scope, and disclosure-time order match. Q1 uses its
first-quarter cumulative value; Q2 and Q3 subtract the preceding cumulative
actual; Q4 subtracts 3Q cumulative actual from FY actual. Each value includes
both inputs, the result, source disclosures, and any reason it is undetermined.
Tied disclosures at the same timestamp are ambiguous; disclosure numbers are
never used to break ties. Missing prior periods, values, or compatible metadata
remain undetermined. This deliberately does not infer a general cumulative
meaning for disclosures whose periods or document basis do not meet these
checks. The `NC`-less fields may represent consolidated or individual values
depending on document type; neither accounting basis nor consolidation status
is inferred when the document does not establish it.
The `source_fields` map preserves a source JSON `null` as `null` and a blank
string as `""`; neither implies an unchanged value, company-wide non-disclosure,
or withdrawal. The API does not include range forecasts, so their absence is
not evidence that a forecast had no range.

Scoring selects disclosure rows whose JST disclosure timestamp is less than or
equal to the target date's close: 15:00 before 2024-11-05, and 15:30 from
2024-11-05 onward. Rows after this cutoff are excluded. This is timestamp
filtering only: J-Quants does not guarantee when a value became available to
clients or preserve values before later corrections. The output also does not
guarantee that the provider exposed every historical disclosure; detected fetch,
conversion, or serialization failures mark the history incomplete, while
undetected provider omissions cannot be ruled out. J-Quants also does not
guarantee snapshot consistency if data changes during pagination.

`latest_statement` and `full_year_sales` remain FY projections, separate from the
full history. `get_row_statements` returns the full disclosure rows, whereas
`get_statements` remains an FY-only projection. Generated statements TSVs now
include a `disclosure_json` column;
`examples/make_statements.rs` writes every disclosure and stops on fetch failure.
Existing TSV data lacks the history and must be regenerated before using it with
`FetchScoringData`; it is rejected rather than silently producing incomplete
history. The JSON contract adds `financial_disclosures` and source fields; old
JSON must be regenerated. The example uses `assets/statements.tsv`, so
regeneration and deployment of that input must be handled separately.
`ScoreStock` adds `quarterly_actuals` and reason codes including
`no_prior_quarter`, `incompatible_fiscal_periods`,
`incompatible_disclosure_basis`, `unknown_fiscal_period`,
`unknown_accounting_scope`, `missing_value`, and `ambiguous_disclosure`. Score
inputs retain their individual audit reasons and the top-level score remains
absent unless all required score metrics are determined.

```rust,ignore
use chrono::Utc;
use quantitative_data_analysis_rs::domain::models::scoring::model::ValueScorePolicy;
use quantitative_data_analysis_rs::use_case::interactors::score_stock::interactor::ScoreStock;
use quantitative_data_analysis_rs::use_case::interfaces::score_stock::input::ScoreStock as ScoreInput;

// `fetch_data` is a FetchScoringData value from the fetch-scoring stage.
let result = ScoreStock::new()
    .handle(ScoreInput::new(fetch_data, ValueScorePolicy::standard(), Utc::now()))
    .await?;
for quarter in &result.quarterly_actuals {
    let sales = &quarter.values["sales"];
    println!("{:?}: {:?} ({:?})", quarter.quarter, sales.result, sales.reasons);
}
```

### Stock price and CSV input contract

`Stock` requires `adj_open`, `adj_high`, `adj_low`, `adj_close`, and
`adj_volume`. Adjusted prices account for splits/consolidations and supported
corporate actions, but not cash dividends. Adjusted volume uses the corresponding
split/consolidation share basis and is stored as `f64` to preserve fractional
values. Missing or invalid required values are errors, not a reason to silently
drop a row or substitute source values.

- J-Quants maps `AdjO`, `AdjH`, `AdjL`, `AdjC`, and `AdjVo` directly.
  Its supported price adjustments include rights issues; rights issues do not
  adjust volume.
- Yahoo maps its provided OHLC and volume to both the source and adjusted fields.
  The analysis `adj_close` is Yahoo's `close`, **not** its dividend-adjusted
  `adjclose`. No additional split adjustment is applied.
- Source fields (`open`, `high`, `low`, `close`, `volume`) preserve provider
  values and are **not guaranteed to be unadjusted**.
- Yahoo's split-adjusted OHLC and volume were corroborated for AAPL's 2020
  4-for-1 split. This is not a verified guarantee for all instruments,
  consolidations, or other corporate actions. Providers may differ in supported
  events and rounding; cross-provider numerical equivalence is not guaranteed.
- Adjusted history may change after subsequent corporate actions. It does not
  guarantee point-in-time reconstruction.

CSV input requires these columns (the shown order is also the headerless order):

```csv
Date,Open,High,Low,Close,Adj Open,Adj High,Adj Low,Adj Close,Volume,Adj Volume
```

All fields are required. Dates use `YYYY-MM-DD`, prices must be finite and
positive, `Volume` is a nonnegative integer, and `Adj Volume` is finite and
nonnegative. Legacy seven-column CSVs are rejected; their `Adj Close` must not be
copied blindly because it may include dividend adjustments. The producer is
responsible for supplying a consistent adjustment basis; the reader cannot infer
it from the numbers.

The stock fixtures `assets/2031.T.csv`, `assets/8473.T.csv`, and
`assets/9223.T.csv` were replaced with J-Quants daily bars fetched on 2026-10-24.
Their periods are respectively 2024-02-15, 2022-09-09 through 2023-09-08, and
2023-12-25 through 2024-02-16. Fixture-based tests require no API access.
Technical analysis uses adjusted OHLC and volume consistently: SMA, MACD, RSI,
Bollinger bands, ATR, volume ratios, price/SMA comparisons, crossover returns,
high/low direction, candlestick patterns, and chart/table values. Crossover
returns are price returns excluding dividends, not total returns. ATR uses
adjusted high/low and previous adjusted close with the existing `ta` EMA
smoothing (`2 / (period + 1)`), not Wilder smoothing. Volume ratios compare the
current adjusted volume against the previous N bars, excluding the current bar.
Analysis-result `volume` and chart-view `volumes()` now use `f64` to avoid
truncating fractional adjusted volume. Financial valuation inputs and stored
source fields are unchanged.

- Provide J-Quants API key via environment variable before executing any example or binary:

  ```bash
  export JQUANTS_API_KEY="your-jquants-api-key"
  ```

- Optional inputs (dates, markets, crossover filters) are passed directly when calling controllers or editing the
  example binaries.

## Contribution

- Read `RUST_GUIDELINES.md` and the referenced Microsoft Pragmatic Rust Guidelines before sending PRs.
- Adhere to the layered architecture; new logic should live in `domain`/`use_case`, while adapters go under
  `infrastructure`/`presenter`.
- Include documentation/comments for non-obvious algorithms and add/adjust tests in `tests/` or under the relevant
  module.

## Testing

Run formatting and the full test suite locally:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

Examples double as integration tests; compare their outputs with the JSON/SVG fixtures in `assets/` when modifying
presenters.

## License

The project has not declared an explicit license yet. Please ask the maintainers before using it in production or
redistributing.

## Acknowledgements

- J-Quants for providing authenticated access to fundamental and price data.
- Yahoo Finance for historical market data access.
- The Rust community and the authors of the crates listed in `Cargo.toml` for the ecosystem support.
