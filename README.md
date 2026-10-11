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

起動後に `task build` を実行し、[README のプレビュー](http://localhost:3001/md/md_preview.html?path=README.md)を開きます。

- [動く見本：カウンタ](./models/_example_counter/README.md)
- [Model 一覧](./md/directory_view.html?link=Model)
- [Compiler 一覧](./md/directory_view.html?link=Compiler)
- [Playground](./assets/playground.html)

## 内容

- [models/](./models/): 計算モデルとモデル間のコンパイラ。
- [docs/](./docs/): 計算モデルについての文章。
- [utils/](./utils/): モデルで共通に使うコード。

## コードを触るとき

```sh
opam exec -- dune build @all
opam exec -- dune runtest
```

ターミナルからモデルを動かすには、[CLI の使い方](./cli/README.md)を参照してください。
