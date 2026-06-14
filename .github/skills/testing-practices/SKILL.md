---
name: testing-practices
description: Rust 変更を安全に統合するための標準テストフローと品質ゲートを定義する。
---

# testing-practices スキルガイド

## 目的

- Rust コードやビルド設定を触る際に、必要な検証手順を明示して品質を一定に保つ。
- テスト結果の共有方法を統一し、レビューの手戻りを減らす。

## 適用タイミング

- `src/` や `tests/` 配下のコードを変更・追加・削除するとき。
- Cargo 設定や CI スクリプトを更新してビルド/テスト結果に影響が出るとき。

## 推奨コマンド

```bash
cargo fmt
cargo clippy --all-targets --all-features
cargo test
cargo nextest run --all --all-features   # 長時間テストやCI検証が必要な場合
cargo audit                              # 依存パッケージの脆弱性チェック
```

## 実行手順

1. 変更内容に応じて必要なテストデータや API 資格情報を準備する。
2. `cargo fmt` でフォーマッタ差分を解消し、`cargo clippy --all-targets --all-features` で警告が出ないことを確認。
3. `cargo test` でユニット/統合テストを実行し、失敗した場合は該当ターゲットを `-- --nocapture` などで再実行して切り分ける。
4. 長時間処理やCI互換が求められる変更は `cargo nextest run --all --all-features` を追加実行する。
5. 依存を更新した場合やセキュリティリスクを懸念する場合は `cargo audit` を実施し、出力を記録する。
6. レビュー提出時は成功したコマンドとログ要約を共有し、差分が生じたファイルパスはバッククオートで提示する。

## チェックリスト

- [ ] フォーマッタと Clippy を警告ゼロで通過したか。
- [ ] 追加・変更したロジックに対してユニットテスト/統合テストを更新したか。
- [ ] `cargo test`/`cargo nextest run` の結果を確認し、失敗ケースを再現できるログを添付したか。
- [ ] 依存更新時に `cargo audit` を実行し、CVE が残っていないか。

## 失敗時の切り分けメモ

- Clippy が `allow` 無しで回避できない場合は `coding-standards` に立ち返り設計を再検討する。
- 外部 API を叩くテストは `-- --ignored` 等で分離し、副作用がある場合はモックや固定データを使う。
- 長時間テストが CI 限定の場合、ローカルでは対象モジュールに絞り `cargo test path::to::module` を使って最小範囲で再現する。

## 参考資料

- 共通方針: `.github/copilot-instructions.md`
- 設計ルール: `.github/skills/coding-standards/SKILL.md`
