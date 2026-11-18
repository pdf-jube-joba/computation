# Component の契約

[web.wit](web.wit) の `computation:web` に従う、ホスト import のない Component を渡す。
実装言語とソースコードは接続条件に含めない。ビルド手順は [README](../../README.md)。

## model

**1インスタンス = 1マシン**。状態ハンドルや resource は使わない。

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

Code・入力・出力・snapshot の文字列形式は処理系が定める。
ホストはその中身を解釈せず、WIT の `result` によるエラーと Wasm trap を区別する。

## compiler

`compile-code`、`encode-ainput`、`encode-rinput`、`decode-routput`、`decode-foutput`。
いずれも文字列を受け取り、変換後の文字列またはエラーを返す。
