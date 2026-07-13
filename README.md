# storymesh

Storybook の story ファイル作成漏れを検出し、ストーリーの充足率を計測・レポートする Rust 製 CLI です。現在は React に対応しています。

## 必要なもの

- [mise](https://mise.jdx.dev/)
- `jq`（ハーネス検査と Codex のトークン利用量集計に使用）

## はじめかた

```sh
mise install
mise run verify
mise exec -- cargo run -- --help
```

`mise` をシェルに有効化していない場合も、上記のように `mise` コマンド経由で実行できます。普段使うシェルで有効化するには、次を `~/.zshrc` に一度だけ追加してください。

```sh
eval "$(mise activate zsh)"
```

## 開発タスク

```sh
mise run format # フォーマットを検査
mise run lint   # Clippy を実行
mise run test   # テストを実行
mise run check  # 型検査
mise run quick  # 編集中の高速フィードバック
mise run handoff # 差分に応じた最小の最終ゲートと検証記録
mise run verify # 完全な品質ゲート（Codexハーネスを含む）
```

## Codex で開発する

このリポジトリには、推論量をタスクの難易度に合わせる起動ラッパー、短い
永続指示、品質ゲート、トークン計測を含む開発ハーネスがあります。

```sh
scripts/codex-task fast       # 文書・機械的変更
scripts/codex-task standard   # 通常の実装（既定）
scripts/codex-task deep       # 設計・セキュリティ・難しい正しさ
mise run harness              # ハーネス自体を検査
mise run handoff              # 差分に適した最終ゲートを実行・記録
```

運用方法と計測指標は [docs/codex-harness.md](docs/codex-harness.md) を参照して
ください。初回または hook 更新後は Codex CLI の `/hooks` で project hook を
確認して信頼してください。

## CLI の方針

- `storymesh check [PATH]`: story ファイルのないコンポーネントを一覧表示。漏れがあれば終了コード `1`、走査エラーは `2`
- `storymesh coverage [PATH]`: ストーリー充足率を算出
- `storymesh report [PATH]`: 充足率と story のないコンポーネントを表示

```sh
# カレントディレクトリを検査
mise exec -- cargo run -- check

# React コンポーネントのあるディレクトリを指定
mise exec -- cargo run -- check src/components --framework react
```

### React の検出規則

- `.tsx` / `.jsx` ファイルをコンポーネントとして扱います。
- `.js` / `.ts` は PascalCase のファイル名（例: `Button.js`）をコンポーネントとして扱います。
- `*.test.*`、`*.spec.*`、`*.d.ts`、story ファイル、および `node_modules`、`dist`、`build` などの生成ディレクトリは除外します。
- `Button.tsx` には、同じディレクトリの `Button.stories.tsx` のような `Button.stories.{js,jsx,mjs,cjs,ts,tsx}` を対応付けます。
- `__stories__` / `stories` サブディレクトリ、および `Button/index.tsx` と `Button/Button.stories.tsx` の構成にも対応します。

Vue や Angular などは、フレームワークごとの検出規則を追加できる構造にした上で今後対応します。
