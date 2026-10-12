# 計算モデルと視覚化に関するメモと実験コード

計算モデルについての個人的な学習メモと、インタプリタ・コンパイラの実験コードを置いています。
Markdown に埋め込んだモデルを動かしながら、計算の様子を見られます。

## 動かし方

OCaml / opam、Node.js、Rust / Cargo を用意し、リポジトリのルートで実行します。

```sh
git submodule update --init --recursive
opam install . --deps-only
npm --prefix _wfs/default_plugins/md_preview install
cargo run --manifest-path _wfs/Cargo.toml -- .
```

起動後に `task build` を実行し、[README のプレビュー](http://localhost:3000/md/md_preview.html?path=README.md)を開きます。

- [動く見本：カウンタ](models/_example_counter/README.md)
- [Model 一覧](./md/directory_view.html?link=Model)
- [Compiler 一覧](./md/directory_view.html?link=Compiler)
- [Model Playground](./assets/playground_model.html)
- [Compiler Playground](./assets/playground_compiler.html)

## 内容

- [models/](./models/): 計算モデル。
- [compilers/](./compilers/): モデル間のコンパイラ。
- [docs/](./docs/): 計算モデルについての文章。
- [utils/](./utils/): モデルで共通に使うコード。`base/` は基本データ型・汎用操作、`names/` は識別子・ID・束縛、`parsing/` は字句・構文解析、`codec/` は入出力変換、`runtime/` はモデルの共通契約・実行・表示。
- [web/](./web/): ブラウザ用コード。`ui/` は画面と描画、`runtime/` はランタイムの読み込みと API 変換、`browser.ml` は OCaml 側の入口。

## コードを触るとき

```sh
opam exec -- dune build @all
opam exec -- dune runtest
```

ターミナルからモデルを動かすには、[CLI の使い方](./cli/README.md)を参照してください。
