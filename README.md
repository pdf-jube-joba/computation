# 計算モデルと視覚化に関するメモと実験コード

このリポジトリは計算モデルについてのメモです。
個人的な学習メモや考えたこと、実際に実装してみたコードなどを置いています。
また、 `mdBook` で視覚的に理解できるように動くモデルを埋め込んであります。

## 内容について
- `rust` 言語で各計算モデルの実装をしてます
- `rust` 言語からコンパイルした `wasm` と `javascript` を用いてブラウザで動いているのを見れるようにしました。   
    - `wacm-pack` を使って、 `rust` から `wasm` へのコンパイル、 `js` から利用する用のコードが生成されます。
    - `SVG.js` というライブラリを使って、 `SVG` で描画します。
- `markdown` で各計算モデルについて書いたものを、 `mdBook` でブラウザで見れるようにしています。
    - 数式を書いたりするために、 `mdbook-katex` を使っています。
- ファイルをコピーしたり、ローカルサーバーを建てるコードもあります。
    - `makefile` の中に、 `rust` のビルド、出力物のコピー、 `mdbook build` の起動が書いてあります。
    - `watchexec` を用いてファイルを監視していて、必要に応じて上のことを行います。
    - ローカルのサーバーは `serve.sh` の中で  `python3 -m http.server` としてプロジェクトのルートディレクトリから配信します。

## 必要なもの
- `rust` 周り
 - `cargo`
 - wasm 向けの target を追加すること
- `wasm-pack`
- `mdbook`
    - `mdbook-katex`
- `watchexec`
- `python3`

## ディレクトリ構成
`Cargo.toml` や `Cargo.lock` は `rust` 言語用のファイルです。

```
.
├── book/                   # mdBook用のファイルのルート
│   ├── book.toml           # mdBookの設定ファイル
│   ├── src/                # mdBook文章
│   └── assets/vendor/      # SVG.jsなどの外部ライブラリ
├── utils/                  # utility ライブラリ
├── models/                 # 各計算モデルの実装など
├── compile/                # 計算モデル間のコンパイラなど
├── docs/                   # リポジトリのドキュメントなど
├── makefile                # ビルドやサーバー起動用Makefile
├── serve.sh                # ローカル開発用のサーバースクリプト
```
