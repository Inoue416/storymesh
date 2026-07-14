# storymesh

`storymesh` は、コンポーネントに対応する Storybook の story ファイルがあるかを検査し、story coverage を報告する Rust 製 CLI です。React、Vue、Angular に対応しています。

次の用途を想定しています。

- story がないコンポーネントを一覧表示する
- Storybook coverage を件数とパーセントで確認する
- CI で story の追加漏れを検知する

## 対応フレームワーク

| フレームワーク | 主なコンポーネントファイル | `--framework` |
| --- | --- | --- |
| React | `.tsx`、`.jsx`、PascalCase の `.ts` / `.js` | `react` |
| Vue | `.vue` | `vue` |
| Angular | `*.component.ts`、`@Component(...)` を持つ `.ts` | `angular` |

## インストール

[mise](https://mise.jdx.dev/) をインストールし、このリポジトリを取得したディレクトリでビルドします。

```sh
mise install
mise exec -- cargo build --release
./target/release/storymesh --help
```

以降の例ではビルド済みの `./target/release/storymesh` を使用します。開発中に直接実行する場合は、代わりに `mise exec -- cargo run --` を使用できます。

## クイックスタート

React プロジェクトの `src/components` を検査する例です。

```sh
./target/release/storymesh check src/components --framework react
```

story がないコンポーネントがある場合は、対象ファイルを表示して終了コード `1` を返します。

```text
Missing stories for 1 React component(s):
Card.tsx
```

すべてのコンポーネントに story がある場合は終了コード `0` です。

```text
All 3 React components have stories.
```

## コマンド

### `check`

story がないコンポーネントを一覧表示します。CI で追加漏れを検知する場合に使用します。

```sh
./target/release/storymesh check [PATH] [--framework react|vue|angular]
```

### `coverage`

coverage のパーセントと件数を表示します。

```sh
./target/release/storymesh coverage src/components --framework vue
```

```text
Vue Storybook coverage: 83.3% (5/6 components)
```

### `report`

coverage と、story がないコンポーネントの両方を表示します。

```sh
./target/release/storymesh report src/app --framework angular
```

```text
Angular Storybook coverage: 83.3% (5/6 components)
Missing: 1
profile.ts
```

`PATH` を省略するとカレントディレクトリを検査します。`--framework` を省略した場合は `react` です。

## 終了コード

| 終了コード | 意味 |
| --- | --- |
| `0` | 検査またはレポートが正常に完了した。`check` では missing がない |
| `1` | `check` が story のないコンポーネントを検出した |
| `2` | パスの読み取りや出力などでエラーが発生した |

`coverage` と `report` は missing があっても正常終了します。missing を CI の失敗として扱う場合は `check` を使用してください。

## 検出規則

### 共通

- story はコンポーネントと同じディレクトリ、または直下の `stories` / `__stories__` ディレクトリから検索します。
- story の拡張子は `.js`、`.jsx`、`.mjs`、`.cjs`、`.ts`、`.tsx` に対応します。
- `.git`、`.next`、`.storybook`、`build`、`coverage`、`dist`、`node_modules`、`target` ディレクトリは走査しません。
- `*.test.*`、`*.spec.*`、story 自身はコンポーネント数に含めません。

### React

- `.tsx` / `.jsx` をコンポーネントとして扱います。小文字の `main.tsx` / `main.jsx` はエントリポイントとして除外します。
- `.js` / `.ts` は PascalCase のファイル名（例: `Button.js`）をコンポーネントとして扱います。
- `*.d.ts` は除外します。
- `Button.tsx` には `Button.stories.tsx` のような同名の story を対応付けます。
- `Button/index.tsx` と `Button/Button.stories.tsx` の構成にも対応します。

### Vue

- `.vue` をコンポーネントとして扱います。
- `Button.vue` には `Button.stories.ts` のような同名の story を対応付けます。
- `Button/index.vue` と `Button/Button.stories.ts` の構成にも対応します。

### Angular

- `*.component.ts` をコンポーネントとして扱います。
- Angular の新しい命名規則で生成される `app.ts` などは、コメントと文字列を除いたコード上の `@Component(...)` デコレータから検出します。
- `button.component.ts` には `button.stories.ts` または `button.component.stories.ts` を対応付けます。
- suffix-less component の `profile.ts` には `profile.stories.ts` を対応付けます。

## 既知の制約

`storymesh` はパスとファイル名を中心に判定し、Storybook の CSF や各フレームワークの AST を完全には解析しません。

- コンポーネントと異なる名前の story は対応付けません。
- MDX ドキュメントは coverage に数えません。
- React の非コンポーネント `.jsx` / `.tsx` や、Vue の画面・レイアウトもコンポーネントとして数える場合があります。
- Angular の `Component` を別名 import した suffix-less component は検出しません。

実アプリでの検証結果と詳細な未対応ケースは [実アプリ検証メモ](docs/real-app-validation.md) を参照してください。

## 開発

開発用コマンドは `mise` 経由で実行します。ハーネス検査には `jq` も必要です。

```sh
mise run quick    # rustfmt + tests
mise run handoff  # 差分に応じた最終ゲート
mise run verify   # harness + rustfmt + Clippy + tests
mise run format
mise run lint
mise run test
mise run check
```

Codex 開発ハーネスの運用方法は [docs/codex-harness.md](docs/codex-harness.md) を参照してください。
