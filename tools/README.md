# Model bundle build

`node tools/build_models.mjs` はルートの Dune プロジェクトをビルドし、
Wasm の処理系、共通 runtime、registry に載った model / compiler の一覧を配信用ディレクトリへ配置します。
`WORKSPACE_FS_OUTPUT_DIRECTORY` があればそのディレクトリを使い、なければ `wasm_bundle/` を使います。

オプション: `--clean`、`--release`、`--out-dir PATH`、`--model NAME`、`--package LIBRARY`。
モデルと library の指定は複数回使えます（`--model` は compiler 名にも使えます）。
一覧は `manifest.json` で、各 entry の `kind` と `name` に種類と名前が入ります。
トップレベルの `runtime` に共通 runtime の相対パスが入ります。
model / compiler ごとの `.js` は生成せず、共通 runtime の生成関数に名前を渡して利用します。
外部 npm パッケージは不要です。OCaml / Dune / opam / Node.js を使います。

## ソースと配信先

- `web/browser.ml`: OCaml のモデルとコンパイラをブラウザに公開する入口。Dune で Wasm とその JS ローダーを生成します。
- `web/runtime/`: ブラウザでランタイムを読み込むコードと、UI 向けの API 変換。ビルド時に出力先へコピーします。
- `web/ui/`: Playground、モデル操作 UI、描画、Markdown への埋め込み。wfs がソースを直接 `/assets/` で配信します。
- `wasm_bundle/`: ビルドスクリプトの既定の出力先。wfs が `/wasm_bundle/` で配信します。
  - `manifest.json`、`runtime.mjs`、`model_adapter.mjs`、`browser.bc.wasm.js`、`browser.bc.wasm.assets/`: 共通の一覧とランタイム。

再ビルドでは一覧、共通 runtime、Wasm assets を置き換えます。
旧形式の一覧に載った個別 `.js`（`models/`・`compiles/` 内および平坦なモジュール）、
`models.json`、旧 JS ランタイムも移行時に削除します。空になった `models/`・`compiles/` は削除します。
絞り込み指定時の一覧は選んだ entry のみになり、前回の選択結果は残しません。
`--out-dir` と環境変数は、共通 runtime と一覧を配置するディレクトリを指定します。

ブラウザから直接利用する場合:

```js
import {createModel, createCompiler} from '/wasm_bundle/runtime.mjs';

const model = await createModel('lambda_calculus');
const compiler = await createCompiler('recursive_function-goto_lang');
```

Wasm runtime は共有し、`createModel()` を呼ぶたびに独立した実行状態を作ります。

Playground は `/assets/playground_model.html` と `/assets/playground_compiler.html` に分かれます。
Playground と Markdown の `data-model` 埋め込みは `manifest.json` の種類と名前から生成関数を呼ぶため、
既存の model / compiler 名をそのまま使えます。

`.repo/config.toml` の `build-wasm` がこのスクリプトを実行します。
`web-ui` は `web/ui/` を配信するための静的なマウントで、ビルド処理はありません。
`task build` は従来どおりモデルのビルドと Markdown プレビューの生成を実行します。
