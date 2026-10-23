[[Model]]

# Spineless Tagless G-machine

STG 言語を小ステップで実行する、push/enter 方式の抽象機械。
公開名は `stg`。`main` のクロージャに入って実行を始め、弱頭正規形（WHNF）で停止する。

Simon Peyton Jones の
[Implementing lazy functional languages on stock hardware: the Spineless Tagless G-machine](https://www.cs.tufts.edu/comp/150FP/archive/simon-peyton-jones/spineless-jfp.pdf)
の STG 言語と操作的意味論を参考にしている。
適用の spine をヒープ上に組み立てず、引数スタックとクロージャへの enter で適用する。
自由変数はクロージャに明示的に格納し、更新可能な thunk は評価結果を共有する。

これは状態遷移を観察するためのインタプリタで、ネイティブコード生成・GC は実装していない。
論文の tagless なコードポインタへの分岐は OCaml の variant による enter の分岐で表す。
データ構築子の名前は `case` のパターン照合に使う。
返却スタックと更新スタックは、順序を保持した一つの継続スタックで表現する。

## 構文

```text
name = \fun [free1, free2] \u [] -> expression;
name = \fun [free1, free2] \n [arg1, arg2] -> expression
```

- `\u` は更新可能な thunk。仮引数は空に限る。
- `\n` は更新しないクロージャ。引数ありなら関数、引数なしなら毎回本体を実行する。
- 最初の `[]` は捕捉する自由変数の一覧。引数と捕捉変数以外のローカル変数は本体から参照できない。
- トップレベルの束縛は相互再帰的で、グローバル変数は捕捉せず参照できる。トップレベルの捕捉リストは空にする。
- ローカル変数がグローバルと同名の場合、そのローカル変数を参照するには明示的に捕捉する。
- `main` は引数なし。各束縛・選択肢は `;` で区切り、末尾の `;` も許す。

| 式 | 意味 |
| --- | --- |
| `f(x, 1)` / `f()` | 変数の enter と適用。引数は変数または自然数だけ |
| `42` | unboxed な自然数 |
| `\con Cons(x, xs)` | 構築子を返す。フィールドを評価しない |
| `\let { bindings } \in e` | 非再帰束縛。右辺の捕捉は束縛前の環境から行う |
| `\letrec { bindings } \in e` | 相互再帰束縛。全アドレスを確保してから捕捉する |
| `\case e \of { alternatives }` | `e` を WHNF まで評価し、選択肢へ戻る |
| `\add(x, y)` / `\sub(x, y)` / `\mul(x, y)` | 自然数の算術。減算は0で飽和 |
| `\eq(x, y)` / `\lt(x, y)` | 自然数の比較。真なら1、偽なら0 |

自然数は桁数を制限しない。算術の引数はすでに unboxed な自然数である必要がある。
thunk のアドレスは `case` で評価してから演算に渡す。演算が暗黙に enter することはない。

`case` の選択肢は次の形式。

```text
Cons(x, xs) -> expression;
0 -> expression;
\default value -> expression
```

`\default -> expression` として束縛を省略してもよい。
一致する構築子または数値の選択肢を優先し、なければ default を選ぶ。
default の変数には WHNF 全体を束縛する。数値は直接、関数や構築子はヒープへの参照として渡す。
構築子の宣言・静的型検査はなく、パターンのフィールド数の不一致などは実行時エラーになる。

## 実行例

部分適用された関数を thunk に保存し、あとから引数を渡す。

```text
add = \fun [] \n [x, y] -> \add(x, y);
inc = \fun [] \u [] -> add(1);
main = \fun [] \u [] -> inc(41)
```

結果は `42`。引数不足なら PAP（部分適用クロージャ）を作り、余分な引数は本体の返す関数に渡す。
`inc` の評価結果はヒープ上で共有される。

次の例では `x` の本体は一度だけ実行される。

```text
main = \fun [] \u [] ->
  \let { x = \fun [] \u [] -> \add(20, 1) } \in
  \case x() \of {
    \default a -> \case x() \of {
      \default b -> \add(a, b)
    }
  }
```

循環したデータも扱える。次の例は `2` を返す。

```text
main = \fun [] \u [] ->
  \letrec { ones = \fun [ones] \n [] -> \con Cons(1, ones) } \in
  \case ones() \of {
    Cons(x, xs) -> \case xs() \of {
      Cons(y, ys) -> \add(x, y)
    }
  }
```

## 遷移と状態

状態はコード、グローバル環境、ヒープ、次のアドレス、制御、引数スタック、継続スタックを持つ。
制御は `Eval` / `Enter` / `Return`。`step` は一つの遷移だけを行う。

- 適用は引数をスタック先頭に積み、対象の値に enter する。
- `\n` の関数は必要な数の引数を取り出し、捕捉環境と合わせて本体を実行する。
- `\u` に enter すると更新フレームに引数を退避し、ヒープを black hole に置き換える。
- WHNF が更新フレームに戻ると、元のアドレスを結果で更新して引数を復元する。関数や PAP の結果も共有する。
- `case` は選択肢・環境・引数を退避し、scrutinee の結果に応じて実行を再開する。
- スタックと引数が空の `Return` から次の `step` で停止する。

評価中の thunk への再入は black hole エラー。
たとえば `main = \fun [] \u [] -> main()` は検出される。
`\n` による無限再帰はステップを進め続ける。
不要な引数や構築子のフィールドは評価しない。

snapshot は循環参照を整数アドレスとして保存する。復元時にはアドレス、捕捉環境、PAP、
black hole と更新フレームの対応を検証する。遷移は不変データを使い、保存済み状態を変更しない。
表示には制御・環境・引数・継続・ヒープを含む。

## 入出力と CLI

`AInput` / `RInput` / `ROutput` は空文字列。
`FOutput` は自然数、`\con Cons(1, @2)`、`\function @2` のいずれか。
`@2` は停止直前の snapshot のヒープアドレスで、フィールドの深い評価は行わない。

```sh
opam exec -- dune exec repl-machine -- model stg \
  --code 'main = \fun [] \u [] -> \add(20, 22)' --ainput ''
```

空行を入力するごとに一段階進む。`:snapshot` で状態を取得できる。

<div data-model="stg"></div>
