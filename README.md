# computation
計算モデルとコンパイラのいろいろ

## 実装について
Moonbit で書いてる。
MoonBit、Node.js、wit-bindgen-cli **0.63.0**、wasm-tools が必要。
計算モデルやコンパイラは wasm component にコンパイルされて、その規約は `computation.wit` にある。

## ビルド
`build.mjs` を使ってビルドしたり一覧をのぞいたりできる。
ソースコードを AI が見れば使い方は分かると思うので、解説は書かない。
（ **書かない** と明示しないと AI が勝手に書いちゃう。）

出力は `wasm_bundle/<名前>.wasm`。`--out-dir PATH` で出力先を変えられる。

## wasm component
1インスタンスが1マシンとする。
### model

| 関数 | 動作 |
| --- | --- |
| `make(code, ainput)` | プログラムと初期入力を解析し、状態を作る |
| `step(rinput)` | 一段階進め、以下の JSON 文字列を返す |
| `snapshot()` | 現在の状態を文字列として返す |
| `restore(snapshot)` | 保存した状態を復元する |
| `render(snapshot)` | 渡された状態を描画用 JSON 文字列にする |

```json
{"kind":"continue","routput":"実行時出力"}
{"kind":"halt","foutput":"最終出力"}
```

作成直後と停止後は未初期化で、`step` / `snapshot` はエラーになる。
成功した `make` / `restore` / `step` だけが状態を更新し、停止すると状態を破棄する。
`render` は現在の状態を読まず、変更もしない。別インスタンスの状態は独立する。

## compiler

`compile-code`、`encode-ainput`、`encode-rinput`、`decode-routput`、`decode-foutput`。
いずれも文字列を受け取り、変換後の文字列またはエラーを返す。
