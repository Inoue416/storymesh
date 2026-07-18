# 実アプリ検証メモ

## 検証環境

2026-07-14 に、公式 CLI で次のアプリを `.storymesh-test-apps/` 以下へ作成し、Storybook 10.5.0 と `storymesh` を導入して検証した。このディレクトリはリポジトリの `.gitignore` に登録してあり、アプリ本体、依存パッケージ、ビルド生成物は Git の追跡対象外である。

| アプリ | 構成 | 意図した missing | 実測 |
| --- | --- | --- | --- |
| React | Vite、React 19、標準 story、同一 `stories` 配置、`__stories__` + `index.tsx`、test 除外 | `MissingCard.tsx` | 5/6、83.3% |
| Vue | Vite、Vue 3、標準 story、同一 `stories` 配置、`__stories__` + `index.vue`、spec 除外 | `HelloWorld.vue` | 5/6、83.3% |
| Angular | Angular CLI 20、標準 story、`*.component.ts`、suffix-less `@Component`、spec 除外 | `profile.ts` | 5/6、83.3% |

各アプリ全体への `check` は意図した 1 ファイルを表示して終了コード 1、Storybook 標準サンプルだけへの `check` は全件 covered で終了コード 0 になった。3 アプリのプロダクションビルドと Storybook 静的ビルドも成功した。

各アプリの `package.json` には、リポジトリ本体をそのアプリの `src` に対して実行する `pnpm run storymesh` も登録した。3 アプリすべてでこのスクリプトを実行し、表の実測値を確認した。

### React のスケルトン生成を手元でデバッグする

検証用 React アプリが `.storymesh-test-apps/react-app` にある環境では、次の手順で missing の再現から生成後の Storybook 表示まで確認できる。

```sh
cd .storymesh-test-apps/react-app
pnpm storymesh:reset
pnpm storymesh:check       # missing を表示し、終了コード 1
pnpm storymesh:generate    # 対応する *.stories.tsx を生成し、終了コード 0
pnpm storymesh:check       # 全件 covered、終了コード 0
pnpm build
pnpm build-storybook
pnpm storybook             # http://localhost:6006/
```

`MissingCard.tsx` は named export、`Sample2.tsx` は export のない空ファイルである。これにより、named import の生成と、import 可能な export がない場合のプレースホルダー生成を確認できる。詳しい期待結果は同アプリの `DEBUGGING.md` に記載している。

## 実アプリで見つけて修正した問題

- `stories/Button.tsx` と `stories/Button.stories.ts` のように component と story が同じ `stories` ディレクトリにある Storybook 標準配置を 0% と誤判定していた。実パスと、別置き story 用の正規化パスを両方照合するよう修正した。
- Angular 20 が生成する `app.ts` のような suffix-less component を検出できなかった。コメントと文字列を除いたコード上の `@Component(...)` を検出するよう修正した。
- Vite React の `main.tsx` を component と誤判定していた。小文字の `main.jsx` / `main.tsx` をエントリポイントとして除外した。
- 並列単体テストの一時ディレクトリ名が衝突し得た。原子的な連番を加えて安定化した。

## 破綻ケース・未対応範囲

検出は Storybook や各フレームワークの AST を完全には解析せず、パスと限定的なソース検査を使う。このため次は正しく対応付けられない。

- component と異なる名前の story。たとえば `Button.tsx` を import する `Checkout.stories.tsx` は `Button` の story として数えない。
- CSF の `.js` / `.jsx` / `.mjs` / `.cjs` / `.ts` / `.tsx` 以外だけで記述された story。MDX ドキュメントは coverage に数えない。
- React の小文字 `main` 以外の非 component `.jsx` / `.tsx` ファイルは component として数える場合がある。
- Vue の `.vue` は用途を解析しないため、画面、レイアウトなども component として数える。
- Angular の `Component` を別名 import して `@NgComponent(...)` のように使う suffix-less `.ts` は検出しない。`*.component.ts` はデコレータの有無にかかわらず命名規則を優先する。

これらを解消するには import、re-export、デコレータ、CSF meta を含む言語別 AST 解析が必要で、現在のファイル名ベースの対応規則を越える変更になる。

## セットアップ時の外部ツール制約

- 検証時点の Angular CLI 最新版は Node.js 22.22.3 以上を要求したため、Node.js 22.14.0 と互換な Angular CLI 20 を使用した。
- Storybook が React サンプルへ生成した未使用の `React` import は TypeScript 6 の `noUnusedLocals` で失敗したため、追跡外検証アプリ内だけで削除した。
- Angular ビルドは sandbox 内で SIGABRT になったが、同じコマンドを sandbox 外で実行すると成功した。
