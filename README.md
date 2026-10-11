# 計算モデルと視覚化に関するメモと実験コード

計算モデルについての学習メモと、インタプリタ・コンパイラの実験です。
OCaml で計算モデルを実装し、Markdown に埋め込んでブラウザから一段階ずつ実行できます。

## セットアップと検証

```sh
opam install dune yojson ppx_deriving_yojson js_of_ocaml js_of_ocaml-compiler js_of_ocaml-ppx wasm_of_ocaml-compiler
```

リポジトリルート（`dune-project` があるディレクトリ）で実行します。
`opam exec --` は、選択中の opam switch の環境で OCaml と Dune を実行します。

```sh
# 全体をビルド
opam exec -- dune build @all

# モデルと CLI のテスト
opam exec -- dune runtest

# ネイティブ CLI をビルドして実行
opam exec -- dune exec computation -- list
opam exec -- dune exec pipe-machine -- model example_counter create --code 5

# ブラウザ向け成果物を wasm_bundle/ に配置
node plugins/mount_model/build.mjs --clean
```

`dune build` の成果物は `_build/` に置かれます。
`node plugins/mount_model/build.mjs` がブラウザ向けの成果物を `wasm_bundle/` に配置します。
`_wfs` の `task build-wasm` / `task build` も、このスクリプトを実行します。

特定の成果物だけなら `opam exec -- dune build web/browser.bc.wasm.js` のように
対象を指定します。`opam exec -- dune clean` は `_build/` を削除し、次回は再ビルドします。

WebAssembly のビルドには Binaryen も必要です。opam の依存関係の案内に従ってください。
CLI のテストと配信用ビルドには Node.js を使います。

## ブラウザと Markdown

単に HTTP で配信する場合は、リポジトリルートで `python3 -m http.server 8080` を起動し、
`http://localhost:8080/assets/playground.html` を開きます。

Markdown のプレビューと編集には独立したサブモジュール `_wfs/` を使います。
初回はサブモジュールを取得し、プレビュー用の依存パッケージをインストールします。

```sh
git submodule update --init --recursive
npm --prefix _wfs/default_plugins/md_preview install
```

`plugins/mount_model/` は外部 npm パッケージを使わないため、`npm install` は不要です。
Rust / Cargo を用意し、リポジトリルートから起動します。

```sh
cargo run --manifest-path _wfs/Cargo.toml -- .
```

起動後の REPL で `task build` を実行すると、モデルの JS / Wasm と md-preview を生成します。
`http://localhost:3000/md/md_preview.html?path=models/_example_counter/README.md` が動作例です。
モデルだけなら `task build-wasm`、Markdown 側だけなら `task md` を使えます。
`_wfs/` は別の Rust リポジトリで、モデルのビルドや CLI には必要ありません。

- [README のプレビュー](http://localhost:3000/md/md_preview.html?path=README.md)
- [example counter（動いている見本）](./models/_example_counter/README.md)
- [Model 一覧](./md/directory_view.html?link=Model)
- [Compiler 一覧](./md/directory_view.html?link=Compiler)
- [Playground](./assets/playground.html)

既存の `data-model` 名でモデルやコンパイラを埋め込めます。

```html
<div data-model="example_counter">
<template data-default-code>3</template>
<template data-default-rinput>\inc</template>
</div>
```

JS 版はモデル名の末尾に `_js` を付けます。各埋め込みは独立したセッションを持ちます。
`assets/script.js` が操作 UI、`assets/renderer.js` が JSON の描画を担当します。

## ディレクトリとライブラリ

- `utils/`: モデルの共通インターフェース、字句解析、任意精度自然数、JSON と描画。
- `models/`: 計算モデルとモデル間コンパイラ。一つの従来の crate を基本的に一つの Dune library に対応させています。
- `registry/`: Web と CLI が共有するモデル名・コンパイラ名の一覧。
- `cli/src/`: ネイティブの対話実行と、snapshot を渡す一回実行の CLI。[使用例](cli/README.md)。
- `assets/`: 操作 UI と描画。
- `web/`: OCaml と JS の接続、ブラウザ用アダプター。
- `plugins/mount_model/`: Dune の成果物を `wasm_bundle/` に配置する `_wfs` プラグイン。
- `docs/`: 計算モデルについての文章。

ルートの `dune-project` は Cargo workspace に相当するビルドの単位です。
各 library の `dune` に依存ライブラリを書きます。たとえばラムダ計算は以下です。

```lisp
(library
 (name lambda_calculus)
 (public_name computation.lambda_calculus)
 (libraries utils yojson)
 (preprocess (pps ppx_deriving_yojson)))
```

同じプロジェクト内では `(libraries lambda_calculus)` と書くだけでリンクされ、
コードからは `Lambda_calculus` の名前空間で利用できます。複数ファイルのライブラリでは、
たとえば `Pcf.Syntax` のように library 名の下に各ファイルの module が並びます。
公開する型と関数は `.mli` で指定でき、実装を隠した抽象型も作れます。
モデルの契約は [Utils.Machine.S](utils/machine.ml)、コンパイラの契約は
[Utils.Compiler.S](utils/compiler.ml) です。Rust の関連型は signature 内の抽象型で表します。

opam package は配布・外部依存の単位で、Dune library とは別です。
今はすべての library を一つの `computation` package にまとめています。
各モデルを個別に配布したくなった時点で package も分けられます。
`public_name` はインストール後の library 名で、外部プロジェクトからは
`(libraries computation.lambda_calculus)` のように利用します。
`opam exec -- dune build -p computation` と `opam exec -- dune install` で、
ライブラリとネイティブ CLI をインストールできます。

## Web のビルド経路

`web/browser.ml` は OCaml のモデルを JavaScript から呼び出すための入口です。
`web/runtime.mjs` は処理系を読み込み、`web/model_adapter.mjs` は
`assets/script.js` に合わせた API を提供します。

`registry/registry.ml` は公開名と実装 module の対応表です。
たとえば `lambda_calculus` という名前を `Lambda_calculus` に対応させ、
CLI のモデル選択、Markdown の `data-model`、配信用ファイルの一覧で共有します。

OCaml → bytecode → `js_of_ocaml` / `wasm_of_ocaml` → JS ローダーと Wasm アセット、
という構成です。Wasm Component・WIT・jco は使いません。
全モデルの処理系をバックエンドごとに一回読み込み、各モデルは別のセッションを作ります。
公開するファイル名は `wasm_bundle/models.json` に載り、Playground もこの一覧を使います。

```js
const model = await import('/wasm_bundle/lambda_calculus.js?instance=example');
model.make('(\\x. x) (\\y. y)', '');
const saved = model.snapshot();
JSON.parse(model.step('0')); // {kind: 'halt', foutput: '\\y. y'}
JSON.parse(model.render(saved));
model.restore(saved);
```

snapshot はモデルごとの JSON です。以前の Rust snapshot 形式との互換性はありません。
停止したセッションでも保存済み snapshot を描画・復元できます。
