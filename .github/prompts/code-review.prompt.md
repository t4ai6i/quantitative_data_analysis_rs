---
mode: agent
description: PRレビュー時に概要把握と内容確認の見逃しを防ぐための、チェックリスト連動プロンプト
---

# PR コードレビュー

レビュー対象の差分（または変更ファイル一覧）を提示してください。
以下の手順に沿ってレビューを実施し、所定テンプレートで結果を出力します。

## レビュー手順

### Step 1 — 変更概要の把握

- 変更の目的・背景をコミットメッセージや PR 説明から読み取る。
- 変更規模（追加/削除行数、影響モジュール）を確認する。
- ヘキサゴナル境界（`domain`/`use_case`/`infrastructure`/`presenter`）のどの層が触れられているかを特定する。

### Step 2 — チェックリストの実施

`.github/instructions/code-review.instructions.md` の「必須チェック項目」を順に確認する。

- [ ] **正しさ/回帰**: 境界値・null相当・空配列・0件データ・例外経路
- [ ] **アーキテクチャ整合**: ヘキサゴナル境界、レイヤ分離
- [ ] **Rust品質**: `unwrap`/`expect` の乱用、`Result` とエラー文脈、不要な `clone`
- [ ] **テスト妥当性**: 対応するユニット/統合テストの有無、失敗ケースの網羅
- [ ] **可読性/保守性**: 命名の意図明確さ、複雑処理への補助コメント

### Step 3 — 結果の出力

下記テンプレートに沿って出力する。

---

## Findings

<!-- 重大度順（Critical → High → Medium → Low）で列挙。指摘がなければ「なし」と記載。 -->
<!-- 書式: [Severity] `path/to/file.rs:行番号` — 問題点 / 影響 / 修正案 -->

## Open Questions / Assumptions

<!-- 仕様不明点、動作確認が取れていない箇所、前提として置いた仮定を列挙。 -->

## Final Note

<!-- 全体評価を1〜3文で。重大な指摘なしの場合はその旨を明記。残留リスクや未実施テストがあれば記載。 -->

---

## 参照ドキュメント

- `.github/instructions/code-review.instructions.md`
- `.github/skills/coding-standards/SKILL.md`
- `.github/skills/testing-practices/SKILL.md`
- `~/.rust/rust-guidelines.txt`

