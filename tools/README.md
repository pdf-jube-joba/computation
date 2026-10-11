# Model bundle build

`node tools/build_models.mjs` はルートの Dune プロジェクトをビルドし、
JS / Wasm の処理系と registry に載った model / compiler のアダプターを配信用ディレクトリへ配置します。
`WORKSPACE_FS_OUTPUT_DIRECTORY` があればそのディレクトリを使い、なければ `wasm_bundle/` を使います。

オプション: `--clean`、`--release`、`--out-dir PATH`、`--model NAME`、`--package LIBRARY`。
モデルと library の指定は複数回使えます。一覧は `models.json` です。
外部 npm パッケージは不要です。OCaml / Dune / opam / Node.js を使います。

## ソースと配信先

- `web/browser.ml`: OCaml のモデルとコンパイラをブラウザに公開する入口。Dune で JS / Wasm にコンパイルします。
- `web/runtime/`: ブラウザでランタイムを読み込むコードと、UI 向けの API 変換。ビルド時に出力先へコピーします。
- `web/ui/`: Playground、モデル操作 UI、描画、Markdown への埋め込み。wfs がソースを直接 `/assets/` で配信します。
- `wasm_bundle/`: ビルドスクリプトの既定の出力先。wfs が `/wasm_bundle/` で配信します。

`.repo/config.toml` の `build-wasm` がこのスクリプトを実行します。
`web-ui` は `web/ui/` を配信するための静的なマウントで、ビルド処理はありません。
`task build` は従来どおりモデルのビルドと Markdown プレビューの生成を実行します。
