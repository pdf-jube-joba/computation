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

### `repl` (対話)

`repl model` は `machine-daemon` 経由で状態を保持するため、先に daemon を起動してください。

```bash
cargo run -p utils --bin machine -- daemon start
cargo run -p utils --bin repl -- model <NAME> --code <CODE> [--ainput <AINPUT>]
```

- 起動後は `rinput> ` に1行ずつ入力して `step` と `snapshot` を確認できます。
- `:begin` / `:end` で複数行入力、 `:q` / `:quit` / `:exit` で終了します。

`compiler` は1機能ずつ実行します。

```bash
cargo run -p utils --bin repl -- compiler compile-code <NAME> --code <TEXT>
cargo run -p utils --bin repl -- compiler compile-ainput <NAME> --ainput <TEXT>
cargo run -p utils --bin repl -- compiler compile-rinput <NAME> --rinput <TEXT>
cargo run -p utils --bin repl -- compiler decode-routput <NAME> --routput <TEXT>
cargo run -p utils --bin repl -- compiler decode-foutput <NAME> --foutput <TEXT>
```

### `machine` + `machine-daemon` (one-shot)

daemon を先に起動し、 `machine` を one-shot で重ねて使います。

```bash
cargo run -p utils --bin machine -- daemon start
cargo run -p utils --bin machine -- model example_counter
cargo run -p utils --bin machine -- create --code 0 --ainput ""
cargo run -p utils --bin machine -- step --rinput inc
cargo run -p utils --bin machine -- current
cargo run -p utils --bin machine -- daemon kill
```

- デフォルトの socket は `/tmp/computation-machine.sock`
- デフォルトの pidfile は `/tmp/computation-machine.pid`
- `machine daemon status` で daemon の状態確認ができます。

## ビルド

wasm 生成と book 生成は役割を分けています。

```bash
make wasm-build  # build.py: wasm_bundle/ を生成
make generate    # generate.py: docs/ + wasm_bundle/ -> dist/
make md-build    # generate + mdbook build dist
make serve       # wasm-build + md-build + mdbook serve dist
```
