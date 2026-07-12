# storymesh

Storybook の story ファイル作成漏れを検出し、将来的にはストーリーの充足率を計測・レポートする Rust 製 CLI です。

## 必要なもの

- [mise](https://mise.jdx.dev/)

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
mise run verify # 上記をすべて実行
```

## CLI の方針

- `storymesh check [PATH]`: story ファイルのないコンポーネントを検出
- `storymesh coverage [PATH]`: ストーリー充足率を算出
- `storymesh report [PATH]`: 集計結果をレポート

各サブコマンドは雛形のみです。次に対象となるコンポーネント・story ファイルの命名規則と、対応付けのルールを実装します。
