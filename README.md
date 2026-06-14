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

## Installation

1. Install the Rust toolchain specified in `rust-toolchain` (Rust 1.90.0). `rustup` automatically picks it up when you
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
cargo fmt --all
cargo clippy --all-targets --all-features
cargo test
```

Examples double as integration tests; compare their outputs with the JSON/SVG fixtures in `assets/` when modifying
presenters.

## License

The project has not declared an explicit license yet. Please ask the maintainers before using it in production or
redistributing.

## Acknowledgements

- J-Quants for providing authenticated access to fundamental and price data.
- Yahoo Finance for historical CSV feeds used in examples.
- The Rust community and the authors of the crates listed in `Cargo.toml` for the ecosystem support.
