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

## コミュニケーション

- 非自明なアルゴリズムやデータ変換を触る際は簡潔なコメントで意図を共有。冗長な説明は避ける。
- 実行結果や差分を示す際はファイルパスをバッククオートで明示し、必要に応じて再現コマンドも提示。
- 不確実な点や既存の変更がある場合は、即座にユーザへ確認を取りながらペアプロする姿勢を保つ。
- 完了報告では、最低限「変更ファイル」「変更意図」「未検証項目（あれば）」を明記する。

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
