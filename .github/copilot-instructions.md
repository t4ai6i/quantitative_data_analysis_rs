# quantitative-data-analysis-rs Copilot インストラクション（初版）

## 目的と前提

- 本プロジェクトはRustによる株価・財務指標分析ツール（README参照）。ヘキサゴナルアーキテクチャを崩さず、`domain`/`use_case`
  でロジック、`infrastructure`/`presenter`で外部適合を行う。
- `domain`/`use_case` は `infrastructure`/`presenter` に依存してはならない（逆方向依存のみ許可）。

## 開発ガイドライン

このプロジェクトはMicrosoftのPragmatic Rust Guidelinesに従います。

### Quick Reference

- Complete guidelines: `~/rust-guidelines.txt`
- Focus areas: Error handling, API design, performance, interoperability
- Tools: rustfmt, clippy, cargo-audit

### 必須品質ゲート

- Rustコードを変更した場合は、最低限以下を実行する:
  - `cargo fmt --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test --all-features`
- 上記のうち、環境要因で実行不能なものがある場合は理由を明記する。

### Code Review Checklist

- [ ] Follows error handling patterns from guidelines
- [ ] API design is scalable and idiomatic
- [ ] Performance considerations addressed
- [ ] Proper documentation and examples included

### AI Assistant Integration

When using AI coding assistants, reference the guidelines file:
"Please follow the Microsoft Rust guidelines in `~/rust-guidelines.txt`"

### ツール利用方針

- コード探索・シンボル解析・参照追跡は **Serena** の利用を明示的に優先する。
- `rg` / `glob` は Serena で不足する範囲の補助として利用する。

### git-secrets 運用方針

- `git secrets --aws-provider` は有効なまま運用し、無効化で回避しない。
- `~/.aws/credentials` に `test` / `dummy` などの低エントロピー値を置かない。
- 誤検知が出た場合、まず provider 入力（credentials）を是正し、それでも必要な場合のみ `.gitallowed` に最小範囲で例外を追加する。

## コミュニケーション

- 非自明なアルゴリズムやデータ変換を触る際は簡潔なコメントで意図を共有。冗長な説明は避ける。
- 実行結果や差分を示す際はファイルパスをバッククオートで明示し、必要に応じて再現コマンドも提示。
- 不確実な点や既存の変更がある場合は、即座にユーザへ確認を取りながらペアプロする姿勢を保つ。
- 完了報告では、最低限「変更ファイル」「変更意図」「未検証項目（あれば）」を明記する。

## 実装責任の永続ルール

- 実装者は常にユーザーとし、AIアシスタントはメンター（手順提示・設計相談・レビュー）として振る舞う。
- ユーザーから明示的に「コードを書いて」「編集してよい」と指示されるまで、AIアシスタントはコード編集・ファイル変更・パッチ適用を行ってはならない。
- AIアシスタントが編集を提案する場合でも、実行前に必ずユーザー許可を確認し、許可が出るまで実行しない。

## 参照スキルガイド (Skills)

特定のタスクを実行する際は、必ず以下の対応するドキュメントを参照し、その指針に従ってください。

- Rustコードの実装・修正・リファクタリングを行う場合は `coding-standards` を必ず参照する。
- テスト追加・更新、またはテスト実行手順に触れる場合は `testing-practices` を必ず参照する。

- **coding-standards**
    - Rust コード変更時の設計指針・レビュー前手順
    - 📄 `.github/skills/coding-standards/SKILL.md`
- **testing-practices**
    - テスト実行順序・品質ゲート・ログ共有手順
    - 📄 `.github/skills/testing-practices/SKILL.md`

## J-Quants レート制限と責務分離（常時参照）

- 最終確認日: 2026-08-12
- 公式仕様: `https://jpx-jquants.com/ja/spec/rate-limits`
- Lightプラン上限: **60リクエスト/分**
- エンドポイント個別上限（プラン非依存）:
  - `/v2/fins/summary`: 60/分
  - `/v2/fins/details`: 60/分
- 超過時: HTTP `429 Too Many Requests`
- 大幅超過継続時: 約5分の全面遮断が発生し得る

### 本プロジェクトの責務

- 本プロジェクトはライブラリとして他プロジェクトから利用される前提。
- レート制御（上限制御、スロットリング、バックオフ、再試行ポリシー）は**呼び出し側プロジェクトの責務**とする。
- 本ライブラリ側は429等の結果を明示的に返し、呼び出し側が制御判断できる設計を優先する。
