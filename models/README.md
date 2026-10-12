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

## テキストの解析

`Utils.Lexer` はコメント・空白を除いて token と byte 位置を作ります。
`Utils.Parser.create` はそれを一度読み、対応する `()` / `[]` / `{}` を
`Group` にまとめた token tree を内部に持ちます。`Parser.tree` で参照できます。
文字列・文字リテラルやコメントの中の括弧は対応付けの対象になりません。
閉じ忘れ・対応の不一致は初期化時に検出します。

モデルは共通の token tree を読み、その言語の AST を組み立てます。
演算子の優先順位・キーワードの意味・型検査は各モデル側で定義します。

```ocaml
module P = Utils.Parser

let parse_numbers =
  P.parse (fun p ->
      P.expect_keyword p "numbers";
      P.group p ~opening:'[' (fun inside ->
          P.separated_list inside ~separator:',' P.natural))
(* numbers [001, 2] -> Ok ["1"; "2"] *)
```

`group` は対応済みの括弧の中だけを読む parser を渡します。その parser は
閉じ括弧の位置で `Eof` を返し、中身を読み切ると親が閉じ括弧の先へ進みます。
中身の読み取りが失敗した場合は親の位置を進めません。
`bump_group_if` は指定の開き括弧がなければ `None` を返します。

`peek` は次の token の確認、`bump` は一つの消費、`bump_if` は一致した
token 列の消費、`expect` は必須の token 列の消費です。
`bump_if` は不一致なら位置を戻します。既存の文法では括弧を個別の
`expect_symbol` で読むこともできます。後戻りには `checkpoint` / `restore` を使います。
`parse` は読み残しを検査して `(値, string) result` にまとめます。

`natural` は桁数を制限しない `Utils.Number.t` に正規化し、`integer` は
OCaml の非負 `int` に収まるか検査します。`string_literal` は JSON 形式の
エスケープを解釈した文字列を返します。`character_literal` を使う文法は
`~character_literals:true` を指定します。
`many` は括弧内などのスコープ末尾まで読み、`separated_list` は区切り必須の
リストを読みます。区切りの省略・重複は `many ~skip_separator`、
末尾の区切りは `separated_list ~allow_trailing:true` で明示的に許可できます。

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
