# Component Model の公開インターフェース

[web.wit](web.wit) は OCaml 全面移行の直前に使われていた WIT を、内容を変更せず復元したものです。
この旧 WIT を、実装言語に依存しない Component Model の公開インターフェースとして採用します。
現在の OCaml ビルドからこの WIT に対応した Component が生成されるわけではありません。

## 採用方針

処理系の `model` world は **1コンポーネントインスタンス = 1マシン** とします。

- 各インスタンスは、現在の実行状態を最大一つ保持します。
- 成功した `make` は、そのインスタンスの現在状態を新しい初期状態に置き換えます。
- `step` はその現在状態を進めます。状態を識別するハンドルは引数に取りません。
- 独立したマシンを同時に動かす場合は、コンポーネントを別々にインスタンス化します。
- それぞれのマシンの実行状態は独立し、一方の操作は他方の状態を変更しません。

この公開 API に `resource machine` は導入しません。
旧 WIT の関数構成を維持し、`step` の成功値も従来の JSON 文字列を使います。
`snapshot` / `restore` / `render` と `compiler` world も、旧 WIT の構成を採用します。

makeAI が依存するのは、この WIT と動作規約に適合した Component の生成物です。
computation の実装言語やソース構成は makeAI の接続条件に含めません。

## 来歴

以下のコミットは computation リポジトリ内のものです。

| コミット | 日付 | 変更 |
| --- | --- | --- |
| `7955a42080d1a6d8afcb1eb4e163b54d2e10869c` | 2026-03-06 | `utils/wit/web.wit` を追加。world 名は `web-model` / `web-compiler` |
| `07bc7aee501e5d1a3ac076b1b77ca604e50f4175` | 2026-03-07 | 現在復元した `model` / `compiler` とその関数へ改訂 |
| `e48621e099c2541aa51c1afeb3ca0e0408ecb5fd` | 2026-10-10 | 「ocaml 全面移行」で WIT と Rust 実装を削除 |

復元元は `e48621e^:utils/wit/web.wit` です。関連実装も Git 履歴から確認できます。
以下は computation のルートで実行します。

```sh
git show e48621e^:utils/wit/web.wit
git show e48621e^:utils/src/wasm_util.rs
git show e48621e^:cli/README.md
git show e48621e^:plugins/mount_model/build.mjs
```

`utils/src/wasm_util.rs` は `wit_bindgen::generate!` と公開用マクロを定義していました。
旧ビルドは Rust の Core Wasm を `wasm-tools` で Component 化し、ブラウザー向けに `jco` の生成物も作っていました。
旧 CLI は Component を読み込んで実行する構成でした。

## 復元した公開仕様と実装の振る舞い

`package computation:web` に2つの world があります。バージョン指定はありません。

### `model`

| 関数 | 旧実装の動作 |
| --- | --- |
| `make(code, ainput)` | プログラムと初期入力を解析し、現在の状態を作る。成功時は値を返さない |
| `step(rinput)` | 現在の状態を一段階進め、継続または停止を表す JSON 文字列を返す |
| `snapshot()` | 現在の状態を JSON 文字列として取り出す |
| `restore(snapshot)` | JSON 文字列から現在の状態を復元する |
| `render(snapshot)` | 渡された snapshot を描画用 JSON 文字列に変換する |

すべての関数は WIT の `result<..., string>` でエラーを返します。
`resource` や状態ハンドルは定義されていません。旧 Rust マクロは
`thread_local!` の `RefCell<Option<Machine>>` に一つの現在状態を保持していました。
この状態の単位を、採用する公開仕様でもコンポーネントインスタンスに対応させます。

`step` の成功値には次のいずれかの JSON が入ります。

```json
{"kind":"continue","routput":"実行時出力の文字列表現"}
{"kind":"halt","foutput":"停止時出力の文字列表現"}
```

停止すると現在状態は破棄されます。`render(snapshot)` は現在状態を使わず、引数の snapshot を描画します。

### `compiler`

`compile-code`、`encode-ainput`、`encode-rinput`、`decode-routput`、`decode-foutput` を公開します。
それぞれ文字列を受け取り、変換後の文字列またはエラーを返します。

これらの機能は現在の OCaml の `Machine.S` / `Session` と
`registry/compiler_text_api.ml` にも対応する操作があります。
現在のブラウザー用アダプターは `step` の出力を上記の JSON 形式へ変換しています。
一方、`wasms/runner.ml` による makeAI の WASI 接続は、予算付きの実行を JSON-lines で要求する別の規約です。

## 動作規約の補足として残る事項

WIT の関数構成を変更せず、次の事項を明確にする必要があります。

- 将来の仕様変更時における互換性とバージョン管理の方針。
- エラー後の状態保持や、停止後に呼び出せる操作の規定。
- 各関数の動作と JSON 内部の形式を検証する適合テスト。

エラー後の状態保持には既存実装間の差があります。旧 Rust の `step_machine_impl` は
`Option::take()` で状態を外した後に入力解析・実行をするため、その途中のエラーで状態を失います。
現在の OCaml の `Session.step` は、成功するまでは保存した状態を置き換えません。
移行時にどちらの振る舞いを採用するか明示する必要があります。

WIT だけでは各関数の意味や JSON 内部の形式は定まりません。
makeAI が依存するものを公開仕様に限定するため、これらの動作規約と適合テストも必要です。

## 構文確認

```sh
wasm-tools component wit utils/wit/web.wit
```
