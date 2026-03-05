よくある while 言語を作る。

\(\begin{aligned}
\NT{var} &\defeq \T{string} \\

\NT{aexp} &\defeq \NT{var} | \NT{number} | \NT{aexp} \NT{binop} \NT{aexp} | \T{if} \NT{bexp} \T{then} \NT{aexp} \T{else} \NT{aexp} \\
\NT{bexp} &\defeq \NT{aexp} \NT{rel} \NT{aexp} | \NT{bexp} \T{||} \NT{bexp} | \T{!} \NT{bexp} \\

\NT{stmt} &\defeq \\
  &| \T{Nop} \\
  &| \NT{var} := \NT{aexp} \\
  &| \NT{stmt} \T{;} \NT{stmt} \\
  &| \NT{if} \NT{bexp} \NT{stmt} \\
  &| \NT{while} \NT{bexp} \T{\LSB} \NT{stmt} \T{\RSB} \\

\end{aligned}\)

## 意味論（small-step）
- 状態は \((s, \sigma)\)（`stmt` と変数環境）で表し、1ステップで \((s, \sigma) \to (s', \sigma')\) に遷移する。
- 代入は右辺式を環境で評価して環境を更新し、`Nop` に進む。
- 逐次実行 `s1 ; s2` は、`s1` が `Nop` になるまで左側を進め、`Nop ; s2` は `s2` に簡約する。
- `if b s` は `b` を評価し、真なら `s`、偽なら `Nop` に進む。
- `while b [s]` は `if b (s ; while b [s])` に展開して実行する（`b` が偽なら停止）。

## コンパイル方針
目的は、`while` の small-step を `flow_ir` の region ベース制御へ写すこと。
かなり愚直に変換する。
重要な点として、 while における式の評価時点では環境に対して副作用が起きない。
なので、短絡評価できなくてもあまり問題ないが、ここでは短絡評価になっている。

- 各文ごとに region を（場合によっては複数）割り当てていき、適宜仮想レジスタのスコープを区切る。
  - 文の内部制御は `goto`（同一 region 内）で行う。
  - 文から次の文への遷移は `enter`（region 遷移）で行う。
- 変数は static 領域に固定割当てする（`@x`, `@y`, ...）。
  - `enter` で vreg 環境がリセットされることを前提に、
    文をまたいで必要な値は static に必ず保存する。
- 算術式 `aexp` は vreg の名前（ここでは `%t`）と終わった後の goto 先の名前（ここでは `:end` ）を受け取って、次のように複数のblockにする。
  - `x`: `:this { %t := ld @x; goto :end }` だけ。 `:this` は適当に生成して入口にする。
  - `n`: `:this { %t := #n; goto :end }` だけ。 `:this` は適当に生成して入口にする。
  - `a1 + a2`:
    1. `%l, %r` と `:this` を適当に生成する。
    2. `a1/a2` を `%l, %r` と `:this` で生成する。
    3. `:this {%t := %l binop %r; goto :end}` で生成する。
    4. 入口は `%l` で生成された部分
  - `if b then a1 else a2`:
    2. `a1/a2` を vreg は `%t` 、継続は `:end` で生成する: 名前は `:a1/:a2` になったとする。
    2. `b` を `:a1` と `:a2` を渡して生成する。
    5. 入口は `b` で生成したブロックとする。
- 真偽式 `bexp` は条件が true だった場合と false だった場合に飛ぶblock（ここでは `:t` と `:f` ）を受け取って、次のように複数の block にする。
  - `a1 rel a2`:
    1. left/right で vreg の名前を作り（ここでは `%l/%r` ）、自分の名前も生成する（ここでは `:this` ）。
    2. `a1/a2` で `%l/%r` と継続 `:this` で生成する。
    3. `:this { if %l rel %r goto :t; goto :f }` を加える。
    4. 入口は left で生成したところ。
  - `b1 || b2`: 短絡評価に注意
    1. `b2` 評価用の block を `:t` と `:f` で生成する。名前は `:b2` になったとする。
    2. `b1` 評価用の block を、 `:t` と `:b2` で生成する。
    3. 入口は `b1` にする。
  - `!b`: true/false 遷移先を交換して作るだけ。
- 文ごとの変換規則（region ベース）：継続用の region ラベル（ここでは `:end` ）を受け取って次のように複数の region にする。
  - `Nop`: `:this { enter :end }` 入口は `:this` （生成したもの）。
  - `s1 ; s2`:
    1. `s2` に対して `:end` を継続に与えて region を作る。名前は `:s2` になったとする。
    2. `s1` に対して `:s2` を継続に加えて region を入れる。
    3. 入口は `s1` で生成したもの
  - `x := a`:
    1. レジスタの名前（ `%r` ）と最後の block の名前（ `:asn_end` ）を生成する。
    2. 最後の block を生成する: `:asn_end { @x := st %r; enter :end}`
    3. `a` 部分の block を追加する。入口はこことする。
    4. 1つの region にまとめる、名前は適当に生成する。
  - `if b s`:
    1. まずは `s` を `:end` で生成する。名前は `:s` になったとする。
    2. `b` に対応する block を `:t` と `:f` で作って、次のような region にまとめる： `:br { ... :t {enter :s; }  :f {enter :end; }}`
    3. 入口は `:br` とする。
  - `while b [s]`:
    1. 名前を生成する： `:t`, `:f`, `:br`, `:head`
    2. `s` に対応する region を継続は `:head` で作る： `:s { ... }` になったとする。
    3. `b` に対応する block を `:t` と `:f` で作って `:br { ... :t {enter :s}  :f {enter :end} }` のように並べる。
    4. `:head { :inner {enter :br} }` と region を生成する。
- label / vreg 名は生成器で一意に払い出し、衝突を避ける。
- `Compiler` trait の対応：
  - `compile`: `WhileCode -> FlowIrCode`
  - `encode_ainput`: `Var -> N` を static map へ写像
  - `decode_foutput`: static map から `Var -> N` を復元
  - `encode_rinput` / `decode_routput` は `()` 同士の恒等写像
- 正しさ確認は、`while` の最終環境と `flow_ir` 実行後 decode 結果の一致で行う。
