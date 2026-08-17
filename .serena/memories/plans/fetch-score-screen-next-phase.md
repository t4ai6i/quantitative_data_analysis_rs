# 次フェーズ実装計画（quantitative-data-analysis-rsのみ）

## スコープ

- 対象: `quantitative-data-analysis-rs` の3段実行ロジック（interactor層起点）
- 非対象: `aws_lambda_trend_analysis`（run_id/manifest 管理は別プロジェクト）
- 命名固定: `fetch_scoring_data` / `score_stocks` / `screen_stocks`
- 出力先: `assets/` 配下（examples 経由）
- 後方互換: 不要（破壊的変更可）
- 永続化: examples 実行ごとに前回結果削除（初期化ロジック追加）

## 設計方針

- interactor は I/Oなし: `handle(...) -> anyhow::Result<...ResultDTO>`
- DTO は `serde::Serialize` 実装
- JSON ファイル出力は呼び出し側（examples / aws_lambda_trend_analysis）の責務
- run_id / manifest.json: **本プロジェクトでは実装しない**
- schema_version: **不要（削除）**
- 市場区分フィルタ: 呼び出し側がS3会社一覧から事前絞り込み済みcodeを渡す
- statement は TSV DSV 実装（JQUANTS APIではなくS3事前取得データを利用）

## DTO スキーマ（presenter層 output / interactor戻り値DTO）

### FetchScoringData

- `code`: String
- `fetched_at`: DateTime<Utc>
- `status`: FetchStatus (ok | rate_limited | empty_data | failed | skipped)
- `error_type`: Option<FetchErrorType>
- `company`: Option<FetchCompany>
- `price`: Option<FetchPrice>
- `latest_statement`: Option<FetchLatestStatement>
- `full_year_sales`: Vec<FetchFullYearSales>

### ScoreStocks

- `code`: String
- `scored_at`: DateTime<Utc>
- `status`: ScoreStatus (ok | failed | skipped)
- `error_type`: Option<ScoreErrorType>
- `input_ref`: { fetch_file: String }
- `score`: Option<{ total: f64, components: Map<String, ComponentDetail> }>
    - ComponentDetail: { raw: f64, normalized: f64, points: f64 }

### ScreenStocks

- `generated_at`: DateTime<Utc>
- `summary`: { total, ok, failed, skipped }
- `ranking`: Vec<{ code, total_score, rank, scoring_policy: { preset_name }, components: Map }>

## 実装フェーズ

### Phase 1: statement DSV 基盤

- 1-1. `structures/internal.rs` + `structures/internal/tsv.rs` 新規作成
- 1-2. `statement/dsv.rs` 新規作成 + `statement.rs` にモジュール追加
- 1-3. `examples/make_statements.rs` 新規作成（JQUANTS API → `assets/statements.tsv` 保存）

### Phase 2: fetch_scoring_data

- 2-1. `interfaces/fetch_scoring_data.rs` + `input.rs` + `use_case.rs` 新規作成 ✅ DONE
- 2-2. `interactors/fetch_scoring_data.rs` + `interactor.rs` 新規作成 ✅ DONE（ハッピーパス + エラーハンドリング）
    - 依存: stock (JQUANTS API) / company (DSV) / statement (DSV)
- 2-3. `interfaces.rs` + `interactors.rs` + `presenters.rs` にモジュール宣言追加 ✅ DONE
- 2-4. `presenter/presenters/fetch_scoring_data/output.rs` の DTO 整理 ✅ DONE

### Phase 3: score_stocks

- 3-1. `interfaces/score_stocks.rs` + `input.rs` + `use_case.rs` 新規作成
- 3-2. `interactors/score_stocks.rs` + `interactor.rs` 新規作成
    - `presenters.rs` のモジュール宣言が前提
- 3-3. `presenters.rs` + `interactors.rs` にモジュール宣言追加

### Phase 4: screen_stocks

- 4-1. `interfaces/screen_stocks.rs` + `input.rs` + `use_case.rs` 新規作成
- 4-2. `interactors/screen_stocks.rs` + `interactor.rs` 新規作成
    - `presenters.rs` のモジュール宣言が前提
- 4-3. `presenters.rs` + `interactors.rs` にモジュール宣言追加

### Phase 5: examples 3本

- 5-1. `examples/fetch_scoring_data.rs`（sample_companies.tsv + JQUANTS API 実呼び出し・前回結果削除）
- 5-2. `examples/score_stocks.rs`（前回結果削除・status!=ok を stderr 出力）
- 5-3. `examples/screen_stocks.rs`（前回結果削除・top 10 stdout + summary stderr）

### Phase 6: 品質ゲート

- 6-1. `cargo fmt --check` / `cargo clippy -- -D warnings` / `cargo test`
- 6-2. E2E 実行シーケンス確認（make_statements → fetch → score → screen）

## 廃止フェーズ

- `define_run_contract`: 削除（run_id/manifest は aws_lambda_trend_analysis 側）
- `define_data_contract`: 削除（Phase 2-1/3-1/4-1 の interface 定義に吸収）

## 実装状況（2026-08-17 更新）

### Phase 2 の状況

- **DTO定義**: ✅ 完全（presenter層 output に配置、source 削除）
- **ハッピーパス実装**: ✅ 完全（3リポジトリ読込・DTO変換）
- **エラーハンドリング**: ✅ 実装済み
    - company / stock / statement の失敗時は失敗DTOを返却
    - error_type を設定
    - full_year_statements は空配列で続行
- **status**: ✅ 保持中（Phase 3 検証後に再検討）

### DTO フィールド名統一（ドメインモデル準拠）

- `annual_dividend_forecast`: ✅（計画の `dividend` から更新）
- `current_fiscal_year_end_date`: ✅（計画の `fiscal_year_end_date` から更新）

## 完了条件

- 3段が status/error_type/components を正しく保持・伝播
- examples で E2E 実行可能
- cargo fmt/clippy/check/test 合格