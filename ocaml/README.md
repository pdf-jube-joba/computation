# OCaml の計算モデル実験

Rust 実装とは独立した Dune プロジェクト。Wasm Component / WIT は使わず、
同じ評価器をネイティブ・JavaScript・WebAssembly 向けにビルドします。

## ビルド

OCaml と opam が導入済みの環境で、以下を実行します。

```sh
opam install 'dune>=3.17' yojson js_of_ocaml js_of_ocaml-ppx js_of_ocaml-compiler wasm_of_ocaml-compiler
cd ocaml
opam exec -- dune build
opam exec -- dune runtest
```

テストはネイティブの意味論と、生成した JS / Wasm 両方の公開 API を確認します。
API テストには Node.js が必要です（Node.js 24 で動作確認）。

`wasm_of_ocaml-compiler` は Binaryen も必要とします。opam のシステム依存関係の案内に従ってください。
個別のターゲットは以下です。

```sh
opam exec -- dune build bin/repl.exe
opam exec -- dune build web/browser.bc.js
opam exec -- dune build web/browser.bc.wasm.js
```

## CLI

```sh
opam exec -- dune exec bin/repl.exe -- counter 0
opam exec -- dune exec bin/repl.exe -- lambda '(\x. x) (\y. y)'
opam exec -- dune exec bin/repl.exe -- lambda '\x. x' '\y. y'
```

標準入力に counter は `inc` / `dec`、lambda は redex の番号を入力します。
`:snapshot` は状態を JSON で出力、`:quit` は終了です。
終了時の出力は `{"kind":"halt","output":...}`。エラーでは状態を保持します。

## ブラウザ

リポジトリのルートで、配信用の bundle をビルドして HTTP で配信します。

```sh
node plugins/mount_ocaml/build.mjs
python3 -m http.server 8080
```

`http://localhost:8080/ocaml/web/` で Wasm 版、`http://localhost:8080/ocaml/web/?backend=js`
で JS 版を試せます。Wasm 版は WasmGC 対応ブラウザが必要です。
生成された JS ローダーと Wasm のアセットを一緒に配信してください。
UI は既存の `assets/script.js` と `assets/renderer.js` を使います。
Create → Step で実行し、snapshot の保存・復元は API から利用できます。

公開アダプターの API は既存のモデルと同じ形です。
異なる `instance` で import すると独立したセッションになります。

```js
const model = await import('/ocaml_bundle/ocaml_lambda_calculus.js?instance=example');
model.make('(\\x. x) (\\y. y)', '');
const saved = model.snapshot();
JSON.parse(model.step('0')); // {kind: 'halt', foutput: '\\y. y'}
JSON.parse(model.render(saved)); // render はセッションを変更しない
model.restore(saved);
```

`make` / `restore` は値を返さず、`step` / `snapshot` / `render` は JSON 文字列を返し、
失敗は例外になります。継続時は `routput`、停止時は `foutput` を返します。
counter の描画は table block、lambda は text block です。
Rust 版の snapshot 形式との互換性はありません。

## _wfs と Markdown 埋め込み

`.repo/config.toml` に `build-ocaml` プラグインを登録し、`/ocaml_bundle/` に配信します。
Rust の `/wasm_bundle/` とは別の出力先なので、それぞれ独立してビルドできます。
クライアントをリポジトリルートから起動して、REPL でタスクを実行します。

```sh
cargo run --manifest-path _wfs/Cargo.toml -- .
```

```text
task build-ocaml
```

このタスクは OCaml のビルドと md-preview の生成を実行します。
`http://localhost:3000/md/md_preview.html?path=ocaml/demo.md` で埋め込み例を見られます。
既にクライアントが起動している場合は、設定を再読み込みするため再起動してください。

```html
<div data-model="ocaml_example_counter" data-bundle-base-url="/ocaml_bundle/">
<template data-default-code>3</template>
<template data-default-rinput>inc</template>
</div>
```

`data-bundle-base-url` は `assets/header.js` が読む任意の属性で、省略時は従来どおり
`/wasm_bundle/` を使います。OCaml のモデル名は `ocaml_example_counter` と
`ocaml_lambda_calculus`、JS 版はそれぞれ末尾に `_js` を付けます。
Wasm と JS の処理系はページ内で各一回ロードし、各埋め込みは独立したセッションを持ちます。
`_wfs/` 本体の変更は不要です。

起動中の _wfs に対する任意の統合テストは、リポジトリルートで実行します。

```sh
node --experimental-vm-modules ocaml/web/integration.cjs http://localhost:3000
```

実際の HTTP 配信と Markdown 変換を通し、ブラウザ用の JS / Wasm ローダーを
Node.js の VM で動かします。bundle の読み込み、セッションの独立性、描画データ、
snapshot を検証します。実ブラウザの DOM 操作を検証するテストではありません。

## 実装

- `lib/machine.ml`: 関連型に相当する抽象型を持つ module signature。
- `lib/example_counter.ml`: 0 未満を拒否し、increment 後に 10 以上なら終了。
- `lib/lambda_calculus.ml`: 再帰型、手書き parser、置換、redex 選択、JSON snapshot。
- `lib/session.ml`: モデル共通の状態管理 functor。
- `bin/repl.ml` / `web/browser.ml`: ネイティブ / JS 向けアダプター。
- `web/browser.ml`: OCaml の関数を JS オブジェクトとして公開する薄い FFI 層。
  内部 API の成功・失敗は `{ok: value}` / `{error: message}` の JSON 文字列です。
  バックエンドごとに `computationOcamlJs` / `computationOcamlWasm` として公開します。
- `web/runtime.mjs` / `web/model_adapter.mjs`: 処理系のロードと、既存の mountModel API への変換。
- `web/smoke.cjs`: Node.js で生成済み JS / Wasm を実行し、公開 API とアダプターを検証するテスト。
  `.cjs` は CommonJS の指定で、Wasm ローダーの Node.js 実行にも対応します。
- `plugins/mount_ocaml/build.mjs`（リポジトリルート）: _wfs の出力先へ配信用ファイルをコピーするビルドプラグイン。

ラムダ項は `\x. body`、`\x y. body`、括弧、左結合の適用を扱います。
変数名は ASCII 英字・underscore で始まり、数字・hyphen・apostrophe も使えます。
AInput はコンマ区切りの項。redex は 0 始まり、左から深さ優先の preorder で数え、
ラムダ抽象の内部も簡約します。Rust 版のコメント構文等は未対応です。

変数の同一性には表示名と別の ID を使います。自由変数の同じ名前は一つの parse 内で
同じ ID にまとめます（Rust 版は自由変数の出現ごとに別の ID）。独立して parse した
AInput の ID と、beta 簡約する抽象の束縛変数の ID を freshen して、変数捕獲を防ぎます。
印字時も別 ID に別名を割り当て、見た目の捕獲を避けます。
snapshot は ID を保存するので、印字・再パースより正確に状態を復元できます。

構文木は不変で部分木を共有し、メモリ管理を GC に任せます。
数値は OCaml の `int` の範囲です。評価は一段階ずつで、発散する項も操作を返しますが、
極端に深い構文木の走査にはスタックの制限があります。
