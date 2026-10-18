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
- [utils/](./utils/): モデルで共通に使うコード。`base/` は基本データ型・汎用操作、`names/` は識別子・ID・束縛、`data/` はテープなどのデータ構造とメモリ操作、`parsing/` は字句・構文解析、`codec/` は入出力変換、`runtime/` はモデルの共通契約・実行・表示。
- [web/](./web/): ブラウザ用コード。`ui/` は画面と描画、`runtime/` はランタイムの読み込みと API 変換、`browser.ml` は OCaml 側の入口。

## コードを触るとき

```sh
opam exec -- dune build @all
opam exec -- dune runtest
```

ターミナルからモデルを動かすには、[CLI の使い方](./cli/README.md)を参照してください。

## OCaml の構成

共通機能は Dune のライブラリ境界でも分けています。新しいコードは必要なライブラリを直接参照します。
モデルや CLI は必要なライブラリを直接参照します。再公開だけの集約モジュールは置きません。

| ディレクトリ | ライブラリ | 責務 |
| --- | --- | --- |
| `utils/base/` | `utils_base` | 自然数、汎用操作、result のエラー境界 |
| `utils/names/` | `utils_names` | 検証済み識別子、型付き ID、intern、名前生成、束縛 |
| `utils/data/` | `utils_data` | テープ・レジスタ画像・文字の木、配列メモリの操作 |
| `utils/parsing/` | `utils_parsing` | 字句解析、token tree、parser |
| `utils/codec/` | `utils_codec` | JSON、テキスト、環境の行形式 |
| `utils/runtime/` | `utils_runtime` | 言語・機械・コンパイラの契約、セッション、描画 |

`names` は `base`、`parsing` は `base` と `names`、`codec` はこれらと `data`、
`runtime` は `codec` に依存します。`data` はほかの共通ライブラリに依存しません。
下位層はモデルや実行基盤に依存しません。
Dune の暗黙の推移依存は無効です。直接使うライブラリは各 `dune` の `libraries` に追加します。

文字リストは `string list`、メモリは `'a array`、環境は `(Identifier.t * 'a) list` を直接使います。
専用の構造が必要なテープ・レジスタ画像・木の型と、配列メモリの操作は `utils/data/`、
テキスト形式は `utils/codec/` に置きます。`'a Utils_data.Tape.t` はテープの型です。
テープの空白セルやメモリの未確保領域の値は操作の `~default` 引数で指定します。
データ本体は要素の codec を要求せず、JSON 変換は要素の変換関数を受け取ります。
`Utils_codec.Tape_text.Make` は default・等値比較・テキストと JSON の codec を持つ
要素モジュールから具体的なテープを作ります。識別子と空白のテープは
`Tape_text.Identifier` で使えます。`Memory_image_text.Make` は要素のテキスト codec、
`parse ~register_count` はモデルのレジスタ数を受け取ります。自然数の画像には
`Memory_image_text.Natural` を使います。文字のリスト・木の入力もモデルの式 parser を経由せず、
共通 codec で読み取ります。モデル固有の構文や値、入力項目の意味・検査はモデル側に置きます。

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
