# 計算モデルと視覚化に関するメモと実験コード

このリポジトリは計算モデルについてのメモです。
個人的な学習メモや考えたこと、実際に実装してみたコードなどを置いています。
また、 `mkDocs` で視覚的に理解できるように動くモデルを埋め込んであります。

## 内容について
- `rust` 言語で各計算モデルの実装をしてます
- `rust` 言語からコンパイルした `wasm` と `javascript` を用いてブラウザで動いているのを見れるようにしました。
  - `wasm-bindgen-cli` を用いています。
- `markdown` で各計算モデルについて書いたものを、 `mkDocs` でブラウザで見れるようにしています。
  - 数式を書いたりするために、 `assets/vendor/` 以下に katex 関連のファイルを置く必要があります。
    - `katex.min.css`, `katex.min.js`, `contrib/auto-render.min.js`, `fonts/*` ... これは katex の zip から持ってきた。

## 必要なもの
- `rust` 周り
  - `cargo`
  - wasm 向けの target を追加すること
  - `wasm-bindgen-cli` をインストールすること
- `mkDocs` 周り
  - `mkdocs-material`: これはなくても見れそう。
  - `mkdocs-github-admonitions-plugin`: github flavored な admonition の処理
  - `pymdown-extension`
  - `awesome-pages`
  - こんなコードでいいらしい
    ```
    pipx install mkdocs
    pipx inject mkdocs mkdocs-github-admonitions-plugin
    pipx inject mkdocs mkdocs-material
    pipx inject mkdocs pymdown-extensions
    pipx inject mkdocs mkdocs-awesome-pages-plugin
    ```
