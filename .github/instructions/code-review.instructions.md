---
applyTo: "**/*"
excludeAgent: "coding-agent"
---

# code-review instructions

## 目的

- 本リポジトリのコードレビューで、バグ・回帰・設計崩れを早期に検知する。
- 指摘は再現性と修正方針が分かる形で提示し、手戻りを減らす。

## レビュー時の基本姿勢

- まず重大度順に指摘を列挙する（Critical/High/Medium/Low）。
- 概要説明は最後に短くまとめる。指摘がない場合は明示する。
- 推測で断定せず、根拠（コード位置、想定実行経路、テスト観点）を添える。

## 必須チェック項目

1. **正しさ/回帰**
   - 境界値、null相当、空配列、0件データ、例外経路を扱えているか。
   - 既存の振る舞いを壊す変更がないか（互換性、入出力形式、公開API）。
2. **アーキテクチャ整合**
   - ヘキサゴナル境界を維持できているか。
   - ロジックが `domain`/`use_case`、外部適合が `infrastructure`/`presenter` に分離されているか。
3. **Rust品質**
   - `unwrap`/`expect` の乱用がないか（テスト・初期化を除く）。
   - `Result` とエラー型で失敗が明示され、エラー文脈が不足していないか。
   - 不要な `clone` や過剰アロケーションがないか。
4. **テスト妥当性**
   - 変更に対応するユニット/統合テストが追加・更新されているか。
   - 失敗ケース・境界ケースを検証しているか。
5. **可読性/保守性**
   - 命名が意図を表しているか。
   - 複雑な処理に最小限の補助コメントがあるか。

## レビューコメントの書き方

- 1指摘につき以下を含める:
  - 重大度
  - 対象ファイルと行（例: `src/use_case/interactors/...:42`）
  - 何が問題か
  - なぜ問題か（実害/将来リスク）
  - 修正案（短く具体的に）

## 出力テンプレート

- Findings
  - `[Severity] path:line` 問題点、影響、修正案
- Open Questions / Assumptions
  - 仕様不明点や確認事項
- Final Note
  - 指摘なしの場合は「重大な指摘なし」。
  - 残留リスクや未実施テストがあれば明記。

## 参照ドキュメント

- `.github/copilot-instructions.md`
- `.github/skills/coding-standards/SKILL.md`
- `.github/skills/testing-practices/SKILL.md`
- `~/.rust/rust-guidelines.txt`
