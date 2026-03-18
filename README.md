# 計算モデルと視覚化に関するメモと実験コード

このリポジトリは計算モデルについてのメモです。
個人的な学習メモや考えたこと、実際に実装してみたコードなどを置いています。
また、視覚的に理解できるように動くモデルを（ markdown に）埋め込んであります。

- 計算モデルについて（数学＋α）を書く。
- あまりがっちりと証明は書かない。
- インタプリタとかコンパイラを動かす。

## 内容とディレクトリ構成
- `rust` 言語で各計算モデルの実装をしてます
    - `rust` 言語からコンパイルした `wasm` と `javascript` を用いてブラウザで動いているのを見れるようにしました。   
    - `WIT + component` と `jco` を使って、 `rust` から `wasm component` を作り、 `js` から利用するためのコードを生成します。
- ブラウザで md が編集までできて、 `wasm` とかモデルを動かすことができます。

### 計算モデルについて
- `utils/` は計算モデル共通で使うライブラリと、計算モデルのインターフェースが書いてあります。また、 `wasm` 化用のマクロもあります。
- `models/` は計算モデルをたくさんおいています。
- `docs/` は計算モデルについての文章をおいています。
- `cli/` は component 化された wasm を元に動きます。詳しくは [cli](./cli/README.md) を見ること。
### その他について
- `workspace_fs/` を使って内容を serve します。ブラウザからアクセスして編集もできます。詳しくは [workspace](./workspace_fs/README.md) を見ること。
- `.repo/` は `workspace_fs/` の設定用です。
- `plugin/` は `workspace_fs/` のプラグインです。
    - `plugin/md_preview/` は markdown を html にします。
        1. markdown の AST 化
        2. 独自の node を追加（ inline math など。）
        3. 独自の node の処理（ inline math を katex を呼び出して html node にする。）
        4. 最終的な html を得る
    - `plugin/mount_model/` は各計算モデルを
        1. `cargo build {model} --target wasm32-unknown-unknown` で wasm にビルドする
        2. `wasm-tools` で wasm の component 化
        3. `jco` で component のグルーコードを作る（ブラウザがまだ対応してない）
        4. できた成果物を `script.js`, `renderer.js`, `style.css` とともに plugin の指定場所に入る
- `viewer/` は markdown の preview と editor を配信しています。
    editor は `workspace_fs` の性質上、ディレクトリを API 経由で書き換えることができます。

## 必要なもの
- `rust` 周り
    - `cargo`
    - wasm 向けの target を追加すること
- `wasm-tools` で wasm の component 化をしています。
- `npm` を使って `plugin/*/` を管理しています。 `npm install` をそれぞれすること。
