# quantitative-data-analysis-rs Copilot インストラクション（初版）

## 目的と前提

- 本プロジェクトはRustによる株価・財務指標分析ツール（README参照）。ヘキサゴナルアーキテクチャを崩さず、`domain`/`use_case`
  でロジック、`infrastructure`/`presenter`で外部適合を行う。

## 開発ガイドライン

このプロジェクトはMicrosoftのPragmatic Rust Guidelinesに従います。

### Quick Reference

- Complete guidelines: `~/.rust/rust-guidelines.txt`
- `~/rust-toolchain` for Rust toolchain management
- Focus areas: Error handling, API design, performance, interoperability
- Tools: rustfmt, clippy, cargo-audit

### Code Review Checklist

- [ ] Follows error handling patterns from guidelines
- [ ] API design is scalable and idiomatic
- [ ] Performance considerations addressed
- [ ] Proper documentation and examples included

### AI Assistant Integration

When using AI coding assistants, reference the guidelines file:
"Please follow the Microsoft Rust guidelines in `~/.rust/rust-guidelines.txt`"

## コミュニケーション

- 非自明なアルゴリズムやデータ変換を触る際は簡潔なコメントで意図を共有。冗長な説明は避ける。
- 実行結果や差分を示す際はファイルパスをバッククオートで明示し、必要に応じて再現コマンドも提示。
- 不確実な点や既存の変更がある場合は、即座にユーザへ確認を取りながらペアプロする姿勢を保つ。

## 参照スキルガイド (Skills)

特定のタスクを実行する際は、必ず以下の対応するドキュメントを参照し、その指針に従ってください。

- **coding-standards**
    - Rust コード変更時の設計指針・レビュー前手順
    - 📄 `.github/skills/coding-standards/SKILL.md`
- **testing-practices**
    - テスト実行順序・品質ゲート・ログ共有手順
    - 📄 `.github/skills/testing-practices/SKILL.md`
