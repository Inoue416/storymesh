# AI エージェント向けスキルの配布・公開

`storymesh` の AI エージェント向けスキルは
[`skills/storymesh/SKILL.md`](../skills/storymesh/SKILL.md) にあります。公開
GitHub リポジトリ上のこのファイルを `npx skills` が直接取得するため、npm
package への同梱や skills.sh への別途アップロードは不要です。

## 利用者向けの導入

公開済みの `main` から、対話的にスキルを導入します。

```sh
npx skills add Inoue416/storymesh --skill storymesh
```

Codex だけを対象に、確認なしでプロジェクトへ導入する場合は次のとおりです。

```sh
npx skills add Inoue416/storymesh --skill storymesh --agent codex --yes
```

`--global` を追加すると、プロジェクトではなく利用者のグローバルなエージェント
設定へ導入します。特定のコミットやブランチを検証する場合は、リポジトリ URL を
指定できます。

```sh
npx skills add https://github.com/Inoue416/storymesh --skill storymesh
```

## 公開前の確認

1. `npx skills` がこのリポジトリから検出できることを確認します。これはファイルを
   導入せず、見つかったスキルだけを表示します。

   ```sh
   npx skills@latest add . --list
   ```

2. `mise run handoff`、`git diff --check`、最終 diff を確認します。

## 公開手順

1. 検証済みの変更をレビューし、公開状態の GitHub リポジトリの `main` へ
   merge します。
2. GitHub リポジトリの About に `agent-skills` topic を追加します。これは発見性を
   高めるためのもので、`npx skills add Inoue416/storymesh --skill storymesh` の利用に
   は必須ではありません。
3. `main` を取得して、利用者向けの導入コマンドを実行し、`storymesh` が一覧に
   表示されることを確認します。

   ```sh
   npx skills@latest add Inoue416/storymesh --list
   ```

4. スキルの更新を GitHub Release で告知する場合は、`skills-vX.Y.Z` のように
   `v` で始まらない tag を使います。このリポジトリでは `v*` tag が npm の
   リリース workflow を起動するためです。`npx skills` での導入自体は Release を
   必要とせず、公開済み `main` の内容を利用します。

skills.sh の利用数表示は `npx skills` の匿名テレメトリをもとに更新されます。公開者が
別の registry へ手動登録する手順はありません。
