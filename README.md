# computation

MoonBit で書いた計算モデルとコンパイラ。各処理系を独立した Wasm Component にビルドする。
実装は `models/` と `compilers/`、ビルド対象は [registry/components.json](registry/components.json)。

## ビルド

MoonBit、Node.js、wit-bindgen-cli **0.63.0**、wasm-tools が必要。

```sh
node tools/build_components.mjs --list
node tools/build_components.mjs --model goto_lang
node tools/build_components.mjs  # 全モデル・コンパイラ
```

出力は `wasm_bundle/<名前>.wasm`。`--out-dir PATH` で出力先を変えられる。
`MOON` / `WIT_BINDGEN` / `WASM_TOOLS` で実行ファイルを指定できる。
`moon` は `~/.moon/bin/moon`、wit-bindgen は `../build/tools/bin/wit-bindgen` も探索する。

## コードの入口

- `models/*/main.mbt` の `model()` がモデルを作る。
- `compilers/*/main.mbt` の `compiler()` が変換器を作る。
- [utils/runtime/session.mbt](utils/runtime/session.mbt) が状態の保持と文字列 API を担当する。
- [utils/wit/web.wit](utils/wit/web.wit) が公開する関数の型、[動作規約](utils/wit/README.md) がその意味を定める。
- `tools/build_components.mjs` が WIT アダプターを生成して Component にまとめる。

モデルの状態は代数直和と不変の値で表し、失敗した操作では元の状態を保つ。
MoonBit の `wasm` ターゲットは線形メモリと参照カウントを使う。ヒープ内の循環は ID で表す。

型検査は `moon check --target wasm`。Component の接続確認は makeAI 側の共通テストで行う。
モデルの意味論や構文のメモは各モデルの README / spec、読書メモは `docs/` にある。
