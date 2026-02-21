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
    - `wasm-bindgen` を使って、 `rust` から `wasm` へのコンパイル、 `js` から利用する用のコードが生成されます。
- `mdBook` を使って `markdown` で各計算モデルについて書いたものをブラウザで見れるようにしています。
    - 数式を書いたりするために、 `mdbook-katex` を使っています。
- `make serve` をすると、ブラウザの `localhost:3000` から見ることができます。

## 必要なもの
- `rust` 周り
    - `cargo`
    - wasm 向けの target を追加すること
- `wasm-bindgen-cli`
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
`Machine` 実装を CLI で動かすときは、モデル側の bin に `utils::model_entry!(...)` を書きます。
native ターゲットでは同じ bin が CLI として動きます。

実行形式:

```bash
cargo run -p <model_crate> --bin <bin_name> -- <code> <ainput> [OPTIONS]
```

- `<code>` と `<ainput>` はファイルパス、または `-`
- `--code-text TEXT` / `--ainput-text TEXT` で直接文字列も指定可能
- `--snapshot` で各 step 後の `SnapShot` を JSON 出力

`<code>` と `<ainput>` の両方に `-` を指定した場合は、`--split DELIM` が必須です。
stdin 全体を `DELIM` で 3 セクションに分け、`code / ainput / rinput` として扱います。

```text
<code>
DELIM
<ainput>
DELIM
<rinput lines...>
```

`rinput` は 1 行ずつ `step` に渡され、`Output` が `Some(...)` になった時点で終了します。
