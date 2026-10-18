# 計算モデル

実装済みモデルの `main.ml` が Machine を定義します。モデル群の `dune` でライブラリを設定します。
Web と CLI の公開名は `registry/registry.ml` に登録します。

## ファイルの分け方

ひとまとまりの処理なら、1ファイルが500行程度あっても構いません。
50行前後の処理は、他モデルと構成を揃える、複数箇所から共有する、
依存や抽象型の境界を設けるなどの理由がなければ、関連する処理と同じファイルに置きます。
行数だけを理由に分割せず、読むときに行き来するファイルの数も考慮します。

## モデルの契約

`Utils_runtime.Machine.S` はコード、入力、出力、状態の型を抽象化した module signature です。
モデルの `.mli` では `include Utils_runtime.Machine.S with ...` を使い、
共通の関数宣言を再利用します。既存の状態型は `with type state := state`、
codec の型は `with type Code.t = code` などで結びます。
固有の機能を公開する codec module は別途宣言し、`with module Code := Code`
などで結びます。モデル固有の型や関数もそのまま公開できます。

```ocaml
type ('state, 'routput, 'foutput) step_result =
  | Continue of { next : 'state; output : 'routput }
  | Halt of 'foutput
```

`make` は初期状態を生成し、`step` は一段階の遷移を返します。
`Code` / `Ainput` / `Rinput` の `parse` は外部のテキストをモデルの型に変換します。
`Utils_runtime.Session.Make` functor が共通のセッション管理を実装します。
エラーでは元の状態を保持し、停止ではセッションを終了します。
セッションは `Continued of string` / `Halted of string` を返し、
`Utils_runtime.Session_json` が CLI / Web の既存 JSON 形式へ変換します。

snapshot は `Yojson.Safe.t` に保存し、モデルごとに復元します。
`ppx_deriving_yojson` が多くの型の JSON エンコーダー・デコーダーを生成します。
`Utils_runtime.Compiler.S` は Source / Target の module を持つ型付きのコンパイラで、
`Utils_runtime.Compiler_text_api.api` が CLI / Web に公開するテキストの境界です。
Source / Target に必要なのは codec を定義する `Utils_runtime.Language.S` で、
状態遷移や描画への依存はありません。`Compiler.Compose` で型付きコンパイラを合成できます。
PCF の式から CEK への変換は共通 AST の恒等変換で、CEK の初期化・遷移はモデル側にあります。
式から SECD 命令列への変換本体は `compilers/pcf/pcf_expr_to_pcf_secd/` にあります。

## 描画

`render` の戻り値は既存の `assets/renderer.js` が扱う JSON block の配列です。
`Utils_runtime.Render` の抽象型 `block` を通して text / table / container を構築し、
`Render.state` で JSON に変換します。向きは `Vertical` / `Horizontal`、
配置は `Inline` / `Block` で指定します。

- text: `{kind: "text", text: string}`
- table: `{kind: "table", columns: [block], rows: [{cells: [block]}]}`
- container: `{kind: "container", children: [block], orientation: "vertical" | "horizontal", display: "inline" | "block"}`

`title` / `className` は任意です。JSON の描画は OCaml の型に依存せず、
保存済み snapshot の描画はアクティブな状態を変更しません。

## テキストの解析

`Utils_parsing.Lexer` はコメント・空白を除いて token と byte 位置を作ります。
`Utils_parsing.Parser.create` はそれを一度読み、対応する `()` / `[]` / `{}` を
`Group` にまとめた token tree を内部に持ちます。`Parser.tree` で参照できます。
文字列・文字リテラルやコメントの中の括弧は対応付けの対象になりません。
閉じ忘れ・対応の不一致は初期化時に検出します。

モデルは共通の token tree を読み、その言語の AST を組み立てます。
演算子の一覧・優先順位・AST の構築方法は各モデル側で指定し、
優先順位に従った読み取りは共通の `Parser.expression` が行います。
キーワードの意味・型検査は各モデル側で定義します。

```ocaml
module P = Utils_parsing.Parser

let parse_numbers =
  P.parse (fun p ->
      P.expect_keyword p "numbers";
      P.group p ~opening:'[' (fun inside ->
          P.separated_list inside ~separator:',' P.natural))
(* 自然数の表示には Utils_base.Number.to_string を使う。 *)
```

`group` は対応済みの括弧の中だけを読む parser を渡します。その parser は
閉じ括弧の位置で `Eof` を返し、中身を読み切ると親が閉じ括弧の先へ進みます。
中身の読み取りが失敗した場合は親の位置を進めません。
`bump_group_if` は指定の開き括弧がなければ `None` を返します。

`peek` は次の token の確認、`bump` は一つの消費、`bump_if` は一致した
token 列の消費、`expect` は必須の token 列の消費です。
`bump_if` は不一致なら位置を戻します。既存の文法では括弧を個別の
`expect_symbol` で読むこともできます。後戻りには `checkpoint` / `restore` を使います。
失敗時の復元まで含めた試行には `attempt` を使います。
`parse` は読み残しを検査して `(値, string) result` にまとめます。

`natural` は桁数を制限しない `Utils_base.Number.t` に正規化し、`integer` は
OCaml の非負 `int` に収まるか検査します。`string_literal` は JSON 形式の
エスケープを解釈した文字列を返します。`character_literal` を使う文法は
`~character_literals:true` を指定します。
`many` は括弧内などのスコープ末尾まで読み、`separated_list` は区切り必須の
リストを読みます。区切りの省略・重複は `many ~skip_separator`、
末尾の区切りは `separated_list ~allow_trailing:true` で明示的に許可できます。
自然数リストの入力形式は `separated_list ~skip_empty:true` で空の項目を無視し、
`natural_with_plus` で数字に直接付いた `+` を許可します。

`expression` は前置・中置演算子の表を受け取り、優先順位と結合順序を扱います。
中置演算子の項目は `(token 列, 左の優先度, 右の優先度, AST 構築関数)` です。
数が大きいほど強く結合し、右を左より 1 大きくすると左結合、同じにすると右結合です。
最小の優先度は 0 です。前置演算子は `(token 列, 右の優先度, AST 構築関数)` です。
`atom` には再帰的な読み取り関数が渡され、`group` と組み合わせて括弧付きの式を読めます。
coroutine、`expr_lang`、PCF、`fn_ptr`、文字列配列モデルがこの共通処理を使います。
PCF の `+`・`-`・`&&` のように、モデル固有の同じ優先順位もそのまま指定できます。

比較などの二項演算を一つだけ読む文法は `binary` を使います。
`chain_left` は文などを区切り記号で読み、左から AST を組み立てます。
procedure 言語の文は `separated_list` で読み、モデル側で右結合の AST に組み立てます。

ほかに残している専用の解析は、通常の中置式とは異なる以下の文法です。

| 場所 | 専用の処理 |
| --- | --- |
| `structured_lang/mini_prog` | 後置記法でスタックから値を組み立てる処理、型の積・和を混在させない規則 |
| `recursive_function` | `COMP` の括弧付き引数列と続く引数を結合する規則 |
| `lambda_calculus`・`lambda_continuations` | λの束縛・適用、モデルごとのキーワード |
| `flow_ir`・`logic_circuit` | 命令の順序、終端命令、ピン接続などの構造 |
| `tiny_isa/symbolic_asm` | 改行単位の命令・ディレクティブ、セクションの切り替え |
| `tiny_isa/cfg_vreg` | ブロック内の命令順序と終端命令の検査。括弧・数値・比較・オペランドの読み取りは共通 parser |
| `turing_machine` | テープの区切り、行・CSV 形式の遷移表 |
| `Utils_codec.Text`・入力 codec | 単独の数値・素の文字列・行形式の環境など、既存の入力形式の解釈。自然数リストは共通 parser |

CFG、`fn_ptr` の `loc(...)` 入力、自然数リストも `group` で括弧を読みます。
λ計算の入力リストと論理回路の信号リストも `separated_list` を使います。
symbolic assembly や CSV 形式の行単位 parser は、`String_ext`・`Number`・`Text` の
共通関数を使いながら専用の読み取りを行います。
環境の行形式は `Utils_codec.Environment_text`、実行時の汎用例外変換は
`Utils_base.Error.protect` が受け持ちます。

`Utils_names.Identifier.t` と `Utils_base.Number.t` は抽象型です。
識別子は `Identifier.make`、自然数は `Number.of_string` / `Number.of_int` で作り、
文字列が必要な境界で `to_string` を使います。Flow IR の構築と名前解決も
`Id` / `Intern` / `Naming` を共通の土台とし、IR 固有の規則だけをモデル側に置きます。

## 入出力のデータ

テープ・メモリ・レジスタ画像・環境・文字のリストと木は `Utils_data` に定義します。
テキスト変換は `Utils_codec.Tape_text`、`Memory_image_text`、`Natural_lines`、
`Environment_text`、`Character_list_text`、`Character_tree_text` を使います。
テープ・メモリ・レジスタ画像は要素型に関して polymorphic です。操作には `~default` を渡し、
テープの codec は `Tape_text.Make`、画像の codec は `Memory_image_text.Make` で具体化します。
テープの共通描画 `Utils_runtime.Data_view.tape` には要素を表示する関数を渡します。
新しいモデルからデータを共有するときは、他モデルの `Main` を経由せずこれらを参照します。
環境入力の並び順、予約語、使用できる記号、レジスタ数などモデル固有の制約は各モデルで扱います。

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
