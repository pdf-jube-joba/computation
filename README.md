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

## OCaml の構成

共通機能は Dune のライブラリ境界でも分けています。新しいコードは必要なライブラリを直接参照します。
`Utils` は既存の利用者向けに主要モジュールをまとめた入口です。

| ディレクトリ | ライブラリ | 責務 |
| --- | --- | --- |
| `utils/base/` | `utils_base` | 自然数、汎用操作、result のエラー境界 |
| `utils/names/` | `utils_names` | 検証済み識別子、型付き ID、intern、名前生成、束縛 |
| `utils/parsing/` | `utils_parsing` | 字句解析、token tree、parser |
| `utils/codec/` | `utils_codec` | JSON、テキスト、環境の行形式 |
| `utils/runtime/` | `utils_runtime` | 言語・機械・コンパイラの契約、セッション、描画 |

`names` は `base`、`parsing` は `base` と `names`、`codec` はこれらの下位層、
`runtime` は `codec` に依存します。下位層はモデルや実行基盤に依存しません。
Dune の暗黙の推移依存は無効です。直接使うライブラリは各 `dune` の `libraries` に追加します。

モデル内では構文、実行時の型、名前解決、ストア、描画を役割ごとに分けます。
`main.ml` はそれらを `Machine.S` として接続します。ソースとターゲットのコード表現を
変換する処理は `compilers/` に置き、機械の初期状態の生成と遷移はモデル側に置きます。
公開する操作・型の境界は `.mli` に記述します。

`dune runtest` は登録済みの全モデルと全コンパイラを実行し、保存状態の往復、描画、
停止、エラー時の状態保持も検査します。Wasm 側も同じ fixture で検査できます。

```sh
node tools/build_models.mjs --out-dir /tmp/computation-wasm-check
node tests/wasm_smoke.cjs /tmp/computation-wasm-check
```
