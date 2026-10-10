# 計算モデルと視覚化に関するメモと実験コード

計算モデルについての学習メモと、インタプリタ・コンパイラの実験です。
OCaml で計算モデルを実装し、Markdown に埋め込んでブラウザから一段階ずつ実行できます。

## セットアップと検証

```sh
opam install dune yojson ppx_deriving_yojson js_of_ocaml js_of_ocaml-compiler js_of_ocaml-ppx wasm_of_ocaml-compiler
opam exec -- dune build @all
opam exec -- dune runtest
node plugins/mount_model/build.mjs --clean
```

WebAssembly のビルドには Binaryen も必要です。opam の依存関係の案内に従ってください。
API のテストと配信用ビルドには Node.js を使います。
`dune runtest` には、モデルの意味論・コンパイラの実行結果・CLI・
Markdown の実行例を JS / Wasm の両方で確認するテストが含まれます。

## ブラウザと Markdown

単に HTTP で配信する場合は、リポジトリルートで `python3 -m http.server 8080` を起動し、
`http://localhost:8080/web/` または `http://localhost:8080/assets/playground.html` を開きます。
`web/?backend=js` では JavaScript 版を使えます。

Markdown のプレビューと編集には独立したサブモジュール `_wfs/` を使います。

```sh
cargo run --manifest-path _wfs/Cargo.toml -- .
```

起動後の REPL で `task build` を実行すると、モデルの JS / Wasm と md-preview を生成します。
`http://localhost:3000/md/md_preview.html?path=web/demo.md` が動作例です。
モデルだけなら `task build-wasm`、Markdown 側だけなら `task md` を使えます。
`_wfs/` は別の Rust リポジトリで、モデルのビルドや CLI には必要ありません。

既存の `data-model` 名でモデルやコンパイラを埋め込めます。

```html
<div data-model="example_counter">
<template data-default-code>3</template>
<template data-default-rinput>inc</template>
</div>
```

JS 版はモデル名の末尾に `_js` を付けます。各埋め込みは独立したセッションを持ちます。
`assets/script.js` が操作 UI、`assets/renderer.js` が JSON の描画を担当します。

## ディレクトリとライブラリ

- `utils/src/`: モデルの共通インターフェース、字句解析、任意精度自然数、JSON と描画。
- `models/`: 計算モデルとモデル間コンパイラ。一つの従来の crate を基本的に一つの Dune library に対応させています。
- `registry/`: Web と CLI が共有するモデル名・コンパイラ名の一覧。
- `cli/src/`: ネイティブの対話実行と、snapshot を渡す一回実行の CLI。[使用例](cli/README.md)。
- `web/`: OCaml と JS の接続、ブラウザ用アダプター、デモと統合テスト。
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
モデルの契約は [Utils.Machine.S](utils/src/machine.ml)、コンパイラの契約は
[Utils.Compiler.S](utils/src/compiler.ml) です。Rust の関連型は signature 内の抽象型で表します。

opam package は配布・外部依存の単位で、Dune library とは別です。
今はすべての library を一つの `computation` package にまとめています。
各モデルを個別に配布したくなった時点で package も分けられます。
`public_name` はインストール後の library 名で、外部プロジェクトからは
`(libraries computation.lambda_calculus)` のように利用します。
`opam exec -- dune build -p computation` と `opam exec -- dune install` で、
ライブラリとネイティブ CLI をインストールできます。

## Web のビルド経路

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

起動中の `_wfs` に対して、配信・Markdown 変換・ブラウザ用ローダーも検証できます。

```sh
node --experimental-vm-modules web/integration.cjs http://localhost:3000
```

このテストは HTTP と Node.js VM を使い、実ブラウザでのクリック操作は検証しません。
