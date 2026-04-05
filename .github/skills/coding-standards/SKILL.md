---
name: coding-standards
description: Rust の設計原則と品質ゲートを一貫適用し、ヘキサゴナル境界を守った堅牢な変更を支援する。
---

# coding-standards スキルガイド

## 目的

- Rust コードを追加・変更するときに、Microsoft Pragmatic Rust Guidelines と本リポジトリの方針を一貫して適用する。
- レビュー負担を減らし、ヘキサゴナルアーキテクチャを崩さない堅牢な変更を下支えする。

## 適用タイミング

- `src/` や `tests/` 配下の Rust コードを作成・編集・削除するとき。
- CLI／Presenter 層も含め、ビルドに影響する設定を触るとき。

## 主要ルール

1. **設計指針**: ロジックは `domain`/`use_case`、外部適合は `infrastructure`/`presenter` に限定する。
2. **エラー処理**: `Result` とカスタムエラー型を優先。`unwrap`/`expect` はテストや初期化コードのみで使用可。
3. **API 設計**: Public API には意味のある型名、`&[T]`/`&str` のような参照を使う。破壊的変更はインタフェース層で吸収。
4. **性能と相互運用性**: 不要なアロケーションを避け、`Iterator`/`Cow` など標準トレイトを活用。FFI 境界では `unsafe` を局所化。
5. **ドキュメント**: 意図が読み取りづらい変換やアルゴリズムには短いコメントを残す。README/マニュアルとの齟齬を作らない。

## 作業手順

1. 変更前に `~/.rust/rust-guidelines.txt` を参照し、該当セクション（エラー処理・API 設計・性能）を確認。
2. 実装ではレイヤリングと依存方向を常に意識し、必要ならユースケース→インフラの順で改修する。
3. フォーマットと静的解析をローカルで実行:
   ```bash
   cargo fmt
   cargo clippy --all-targets --all-features
   cargo test
   ```
   追加でセキュリティ確認が必要な場合 `cargo audit` を走らせる。
4. テストや CLI 出力の差異は `
``path``
` 表記と再現コマンドをセットで共有する。

## チェックリスト

- [ ] エラー経路は Result/thiserror などで明示されているか。
- [ ] 新規 API の命名・可視性はドメインモデルと整合しているか。
- [ ] 不要な `clone`・`String` 化を避け、参照を渡しているか。
- [ ] `cargo fmt`, `clippy`, `cargo test` を成功させたか (ログを確認したか)。

## 参考資料

- Microsoft Pragmatic Rust Guidelines: `~/.rust/rust-guidelines.txt`
- リポジトリ共通方針: `.github/copilot-instructions.md`
