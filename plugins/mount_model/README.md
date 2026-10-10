# Model bundle build

`node plugins/mount_model/build.mjs` はルートの Dune プロジェクトをビルドし、
JS / Wasm の処理系と registry に載った model / compiler のアダプターを配信用ディレクトリへ配置します。
`WORKSPACE_FS_OUTPUT_DIRECTORY` があればそのディレクトリを使い、なければ `wasm_bundle/` を使います。

オプション: `--clean`、`--release`、`--out-dir PATH`、`--model NAME`、`--package LIBRARY`。
モデルと library の指定は複数回使えます。一覧は `models.json` です。
外部 npm パッケージは不要です。OCaml / Dune / opam / Node.js を使います。
