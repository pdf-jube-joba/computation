# Model bundle build

`node tools/build_models.mjs` はルートの Dune プロジェクトをビルドし、
Wasm の処理系と registry に載った model / compiler のアダプターを配信用ディレクトリへ配置します。
`WORKSPACE_FS_OUTPUT_DIRECTORY` があればそのディレクトリを使い、なければ `wasm_bundle/` を使います。

オプション: `--clean`、`--release`、`--out-dir PATH`、`--model NAME`、`--package LIBRARY`。
モデルと library の指定は複数回使えます（`--model` は compiler 名にも使えます）。
一覧は `manifest.json` で、各 entry の `kind` と `module` に種類とモジュールの相対パスが入ります。
各 entry は Wasm 版を一つ生成し、別の `*_js` 版は生成しません。
外部 npm パッケージは不要です。OCaml / Dune / opam / Node.js を使います。

## ソースと配信先

- `web/browser.ml`: OCaml のモデルとコンパイラをブラウザに公開する入口。Dune で Wasm とその JS ローダーを生成します。
- `web/runtime/`: ブラウザでランタイムを読み込むコードと、UI 向けの API 変換。ビルド時に出力先へコピーします。
- `web/ui/`: Playground、モデル操作 UI、描画、Markdown への埋め込み。wfs がソースを直接 `/assets/` で配信します。
- `wasm_bundle/`: ビルドスクリプトの既定の出力先。wfs が `/wasm_bundle/` で配信します。
  - `models/`: model のアダプター。
  - `compiles/`: compiler のアダプター。
  - `manifest.json`、`runtime.mjs`、`model_adapter.mjs`、`browser.bc.wasm.js`、`browser.bc.wasm.assets/`: 共通の一覧とランタイム。

再ビルドでは前回の一覧に載ったモジュールと Wasm assets を置き換えます。
旧形式の `models.json`、平坦なモジュール、JS ランタイムも移行時に削除します。
絞り込み指定時の一覧は選んだ entry のみになり、前回の選択結果は残しません。
`--out-dir` と環境変数は、引き続き `models/`・`compiles/` の親ディレクトリを指定します。

Playground は `/assets/playground_model.html` と `/assets/playground_compiler.html` に分かれます。
Markdown の `data-model` 埋め込みは `manifest.json` で読み込み先を解決するため、
既存の model / compiler 名をそのまま使えます。

`.repo/config.toml` の `build-wasm` がこのスクリプトを実行します。
`web-ui` は `web/ui/` を配信するための静的なマウントで、ビルド処理はありません。
`task build` は従来どおりモデルのビルドと Markdown プレビューの生成を実行します。
