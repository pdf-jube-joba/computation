# 計算モデルと視覚化に関するメモと実験コード

このリポジトリは計算モデルについてのメモです。
個人的な学習メモや考えたこと、実際に実装してみたコードなどを置いています。
また、 `mdBook` で視覚的に理解できるように動くモデルを埋め込んであります。

> [!warning]
> ただし、 SUMMARY.md を自動生成するために makefile 経由での起動を想定しています。
> 本というよりも、記事の集合としてとらえたほうがいいです。

- 計算モデルについて（数学＋α）を書く。
- あまりがっちりと証明は書かない。
- インタプリタとかコンパイラを動かす。

## 内容について
- `rust` 言語で各計算モデルの実装をしてます
    - `rust` 言語からコンパイルした `wasm` と `javascript` を用いてブラウザで動いているのを見れるようにしました。   
    - `WIT + component` と `jco` を使って、 `rust` から `wasm component` を作り、 `js` から利用するためのコードを生成します。
- `mdBook` を使って `markdown` で各計算モデルについて書いたものをブラウザで見れるようにしています。
    - 数式を書いたりするために、 `mdbook-katex` を使っています。
- `make serve` をすると、ブラウザの `localhost:3000` から見ることができます。

## 必要なもの
- `rust` 周り
    - `cargo`
    - wasm 向けの target を追加すること
- `wasm-tools`
- `npx` と `jco`
- `mdbook`
    - `mdbook-katex` も。
- `python3`

## ディレクトリ構成
`Cargo.toml` や `Cargo.lock` は `rust` 言語用のファイルです。

```
.
├── utils/                  # utility ライブラリ
├── models/                 # 各計算モデルとコンパイラの実装など
├── docs/                   # リポジトリのドキュメントなど
```

## CLI の使い方
`utils` には `wasm_bundle/*.component.wasm` を使うための CLI が2つあります。

### `repl-machine` (対話)

```bash
cargo run -p cli --bin repl-machine -- model <NAME> [--code <CODE>] [--ainput <AINPUT>]
```

- `--code` を省略すると、起動後に複数行の `code` を入力します。入力確定は `Ctrl-D` です。
- `--ainput` を省略すると、起動後に `ainput` を1行入力します（空なら Enter）。
- 起動後は `rinput> ` に1行ずつ入力して `step` と `snapshot` を確認できます。
- `:begin` / `:end` で複数行入力、 `:q` / `:quit` / `:exit` で終了します。

`compiler` は1機能ずつ実行します。

```bash
cargo run -p cli --bin repl-machine -- compiler compile-code <NAME> --code <TEXT>
cargo run -p cli --bin repl-machine -- compiler compile-ainput <NAME> --ainput <TEXT>
cargo run -p cli --bin repl-machine -- compiler compile-rinput <NAME> --rinput <TEXT>
cargo run -p cli --bin repl-machine -- compiler decode-routput <NAME> --routput <TEXT>
cargo run -p cli --bin repl-machine -- compiler decode-foutput <NAME> --foutput <TEXT>
```

### `pipe-machine` (one-shot / pipe)

`model create` が初期 snapshot JSON を返し、 `model step` は stdin から snapshot JSON を受け取って次の snapshot JSON を返します。

```bash
cargo run -p cli --bin pipe-machine -- model example_counter create --code 0 --ainput ""
cargo run -p cli --bin pipe-machine -- model example_counter create --code 0 --ainput "" \
  | cargo run -p utils --bin pipe-machine -- model example_counter step --rinput inc
```

## ビルド

wasm 生成と book 生成は役割を分けています。

```bash
make wasm-build  # build.py: wasm_bundle/ を生成
make generate    # generate.py: docs/ + wasm_bundle/ -> dist/
make md-build    # generate + mdbook build dist
make serve       # wasm-build + md-build + mdbook serve dist
```
