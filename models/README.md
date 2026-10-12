# 計算モデル

実装済みモデルの `main.ml` が Machine を定義します。モデル群の `dune` でライブラリとテストを設定します。
Web と CLI の公開名は `registry/registry.ml` に登録します。

## モデルの契約

`Utils.Machine.S` はコード、入力、出力、状態の型を抽象化した module signature です。

```ocaml
type ('state, 'routput, 'foutput) step_result =
  | Continue of { next : 'state; output : 'routput }
  | Halt of 'foutput
```

`make` は初期状態を生成し、`step` は一段階の遷移を返します。
`Code` / `Ainput` / `Rinput` の `parse` は外部のテキストをモデルの型に変換します。
`Utils.Session.Make` functor が共通のセッション管理を実装します。
エラーでは元の状態を保持し、停止ではセッションを終了します。

snapshot は `Yojson.Safe.t` に保存し、モデルごとに復元します。
`ppx_deriving_yojson` が多くの型の JSON エンコーダー・デコーダーを生成します。
`Utils.Compiler.S` は Source / Target の module を持つ型付きのコンパイラで、
`Utils.Compiler.api` が CLI / Web に公開するテキストの境界です。

## 描画

`render` の戻り値は既存の `assets/renderer.js` が扱う JSON block の配列です。
`Utils.Render` で text / table / container を構築できます。

- text: `{kind: "text", text: string}`
- table: `{kind: "table", columns: [block], rows: [{cells: [block]}]}`
- container: `{kind: "container", children: [block], orientation: "vertical" | "horizontal", display: "inline" | "block"}`

`title` / `className` は任意です。JSON の描画は OCaml の型に依存せず、
保存済み snapshot の描画はアクティブな状態を変更しません。

## TODO
プログラミング言語は基本的に、
物事をよりよく記述できるように拡張していくが、
逆に「機能を極限まで減らしつつもチューリング完全性を失っていないような小さい言語」を作るのも
（個人の趣味として）よくある。
こういうのは Turing tarpit とか esolang と呼ばれ、結構いろんな人がいろんなことをやっている。
それと、論文に出てくるような言語だと、議論したい機能に絞った言語を定義しているため、これも小さい。

そういうものを集めたい。
- 抽象機械
    - [] ZINC
    - [] CAM
    - [] spinless tagless G machine
- 表示が難しそう
    - [] Wang tiles
    - [] セルオートマトン
- その他
    - [] brainfuck
    - [] SKI 計算
    - [] tag system
    - [] FRACTRAN
    - [] Grass
    - [] pure lisp
    - [] rustbelt の lambda rust
