# expression
よくある while 言語を作る。
compiler の話としては、式があるのをどうするかが面白い？

\(\begin{aligned}
\NT{var} &\defeq \T{string} \\

\NT{aexp} &\defeq \NT{var} | \NT{number} | \NT{aexp} \NT{binop} \NT{aexp} | \T{if} \NT{bexp} \T{then} \NT{aexp} \T{else} \NT{aexp} \\
\NT{bexp} &\defeq \NT{aexp} \NT{rel} \NT{aexp} | \NT{bexp} \T{||} \NT{bexp} | \T{!} \NT{bexp} \\

\NT{stmt} &\defeq \\
  &| \T{Nop} \\
  &| \NT{var} := \NT{aexp} \\
  &| \NT{stmt} \T{;} \NT{stmt} \\
  &| \NT{if} \NT{bexp} \NT{stmt} \\
  &| \NT{while} \NT{bexp} \T{\LCB} \NT{stmt} \T{\RCB} \\

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

# procedure
call/return で手続呼び出しを行う言語を考える。
- 引数と返り値あり。
  - いずれも複数でいい。
- 大域変数と局所変数あり。
  - global か自分の local で宣言された変数のみ使用できる。
  - 呼び出し元手続きの変数環境は見えないものとする。
- 値呼びにする。
- どの関数を呼び出すかを動的に決める仕組みがないです。

\(\begin{aligned}

\NT{var}    &\defeq \NT{string} \\
\NT{name}   &\defeq \NT{string} \\
\NT{atom}   &\defeq \NT{var} | \NT{imm} \\

\NT{aexp}   &\defeq \NT{atom} \NT{binop} \NT{atom} \\
\NT{bexp}   &\defeq \NT{atom} \NT{binrel} \NT{atom} \\

\NT{var-cmm} &\defeq \syntaxmacro{comma-separated}{\NT{var}} \\

\NT{stmt} &\defeq \\
  &| \T{Nop}
  &| \NT{stmt} \T{;} \NT{stmt} \\
  &| \NT{var} \T{:=} \NT{aexp} \\
  &| \T{if} \NT{bexp} \T{\LCB} \NT{stmt} \T{\RCB} \\
  &| \T{while} \NT{bexp} \T{\LCB} \NT{stmt} \T{\RCB} \\
  &| \T{call} \NT{name} \T{\LP} \NT{var-cmm} \T{\RP} \T{->} \NT{var-cmm} \\
  &| \T{return} \NT{var-cmm} \\

\NT{proc} &\defeq \NT{string} \T{\LP} \NT{var-cmm} \T{\RP} \T{\LCB} \\
  &\T{local} \NT{var-cmm} \\
  &\NT{stmt} \\
  &\T{\RCB} \\

\NT{program} &\defeq \NT{static} \syntaxmacro{comma-separated}{\NT{var}} \T{;} \\
  \NT{proc}+ \\
\end{aligned}\)

- \(\T{;}\) は右結合
- stmt は return で終わらなかったら \(\T{return}\) が入っているものとして扱う。
- static は code の方で宣言だけしておいて、 `AInput` と名前があっているか確認する。

## 意味論（ small-step ）
call-stack が必要になるぐらいで、あとは `while_lang` と同じ。
引数位置に var が来て値渡しなので評価順序も引数評価用の環境もいらない。
`AInput` も `FOutput` も global env と考える。
early return ができる。\(s_1; s_2\) がすでに継続の形をしていることに気が付いた。

- 名前の解決は終わっていることとする。（ `x = 10;` みたいなものの羅列。）
  - global と local と parameter の名前が被ったら `Err` とする。
- main と名前のついている proc から実行する。これの引数はないものとする？
  - 初期状態は全ての variable が \(0\) に行っているものとする。
- proc から push するときに、 stmt の最後には return を入れておく。

以下です。

- 定義
  - \(V \defeq \N\) ... これは値、そんな大げさなことをしないので \(\N\) でいい。
  - \(E_{\text{global}}, E_{\text{local}} \defeq \NT{var} \pfun V\) ... 変数の環境
  - \(\llbracket x \in \NT{var} \rrbracket_{E_g \in E_{\text{global}}, E_l \in E_{\text{local}}}: \N \defeq \cdots\) ...これは look up を local, global でやってその値の取り出し。
  - \(K \defeq \emptyset | (\NT{stmt}, E_{\text{local}}, x_{i \in 0 \ldots n} \in \NT{var} )::K \) ... 継続
    - 戻ってきたら実行する stmt と、環境と、 return 時に代入する variable の組
  - \(S \defeq (\NT{stmt}, E_{\text{global}}, E_{\text{local}}, K)\) ...これは実行機械の状態
- 文の実行の仕方
  - \((s_1; s_2, E_g, E_s, K) \to (s_1'; s_2, E_g', E_s', K')\) if \((s_1, E_g, E_s, K) \to (s_1', E_g', E_s', K')\) ... seq rule 意味の与え方が木構造になった。
  - \((\T{Nop}; s, E, K) \to (s, E, K)\)
  - \((\T{return} r_i; s, E_g, E, K) \to (\T{return} r_i, E_g, E, K)\) ... return を見たら後ろを捨てる。
  - \((\T{return}, E_g, E, \emptyset) \to E_g\) ... これがプログラム全体を通した `AInput` から `FOutput` への写像。
  - \((\T{return} r_{i \in 0 \ldots m'}, E_g, E, (s, E', w_{i \in 0 \ldots m}):: K) \to (s, E_g', E'', K)\)
    - \(m \neq m'\) なら `Err` として、そうじゃないなら、
      - ローカルへの代入： \(w_i \in E'\) に対しては \(E'' := E'[w_i := \llbracket r_i \rrbracket_{E_g, E} ]\) で更新する。
      - グローバルへの代入： \(w_i \in E_g\) に対しては \(E_g' := E'[w_i := \llbracket r_i \rrbracket_{E_g, E}]\) で更新する。
  - \((\T{call} \mathit{name} v_{i \in 0 \ldots n} \T{->} w_{i \in 0 \ldots m}; s, E_g, E_l, K) \to (s'; \T{return}, E_g, E', (s, E_l, w_{i \in 0 \ldots m} ):: K)\)
    - \(\mathit{name}\) で宣言されている手続きが \(\mathit{name} p_{i \in 0 \ldots n'} s'\) とする。
    - \(n \neq n'\) なら `Err` でいい、そうじゃないなら、 \(p_i := \llbracket v_i \rrbracket_{E_g, E_s}\) として**値渡し**
    - local variable は \(\NT{proc}\) のときに宣言されているものを \(0\) で fill して渡して、上と合わせて \(E'\) とする。
  - 他 3 つは straight forward でしょう。

> [!Note]
> AI 談
> - structural operational semantics (Plotkin style?) ... \(s_1; s_2 \to s_1'; s_2\), \(\T{Nop}; s \to s\) にするやりかた
> - abstract machine (SECD, CEK) に近いのは、 \(s_1; s_2\) をみたら \(s_2\) は継続に積むらしい
> あとで本当か調べよう。
> 他に気が付いたこと：
> call/return がきれいに呼び出しの継続を扱っている。
> 例えば、 `(call, E, K) -> (s, E', (Nop, E)::K)` みたいにすると seq rule から `(call; s', E, K) -> (s; s', E', (Nop, E)::K)` となる。
> これは `s` での return がうまくハンドルできていない。（本当は `s` に戻ってほしいのに、 `abort` みたいに、全部の継続が捨てられることになる。）
> なので、 `call; s` という形自体に対する変形ルールが必要になる。**`call` は常に継続をとる！**
> `call` 側がちゃんとしているので、 `return` は文レベルでの継続を全部捨てる形にしていい。

## 意味論 ( program counter 方式 )
どの手続きのどの行を実行しているかを追いかければいい。
あまり述べることがない...
これは実装しなくていい。（実質 `flow_ir` へのコンパイルだから。）

## flow_ir へのコンパイル方法
`Err` になるようなプログラムは考えなくていいので、引数の個数や返り値の個数があっているかは気にせずにやっていい。
**間違えたプログラムを書いた人が悪いので、コンパイル時にはそれを考えない。（未定義動作みたいなもの？）**
\(E_{\text{global}}\) はそのまま static にする。
残りは call-stack に置けばいい。
proc_lang 側でスタックに触れないので、 push/pop は call/local variable/return のときにしか触らないことが確定しているから、
手続きの内部で引数や局所変数にアクセスするときは、固定 index と考えていい。

- 変数・引数については、名前の解決の段階で global/parameter/local がわかるので次のようにする。
  - global: 最初に static に割り振る。
  - parameter \(p_{i \in 0 \ldots n}\): 最初にスタックの長さを \(\T{\%v_fp}\) として手に入れておく。（これは実質 frame pointer になっている。）
    \(\T{\%} p_i := \T{\%v_fp} - (i + 1)\) とする。 \(T{sacc} \T{\%} p_i\) で Key が手に入る。\(i + 1\) はコンパイル時に計算できる。
    局所変数の初期化前は、一番上は return address が入っている状態なので注意。
  - local \(x_{i \in 0 \ldots n}\): 局所変数初期化後にスタックの長さを \(\T{\%v_sp}\) として手に入れておく。
    \(\T{\%} x_i := \T{\%v_sp} - i\) とする。 \(\T{sacc} \T{\%} x_i\) で Key が手に入る。
- \(\NT{stmt} _{\text{proc lang}} \to \NT{block} _{\text{flow ir}}+\) は次のようにやる。
  - \(\T{Nop}\), \(s_1; s_2\), \(x := a\), if/while は while language のときと同じようにやる。
  - call: ブロックを2つにする。
    - 呼び出しブロック：
      - 引数を**右から** ld して push ... 逆順にすると \(- i\) と合う。（これは好み。）
      - 戻ってきたときブロックのラベルを push ... return 先
    - 戻ってきたときブロック：
      - pop して st を各 \(w_i\) ごとにやる。 \(i\) の小さい方から。（caller は \(w_1\) に入れるべき値が top に来ていることを期待。）
  - return: ブロック1つでいい。
    - return に使う variable から値を引き出して `\%v_ri` に入れておく（局所変数を使っているとこの後消えるので注意。）
    - 自分が使った local 分を pop （局所変数は最初に宣言...コンパイル時に確定している。）
    - return add を pop して `\%v_ret` に入れる
    - 返り値 `\%v_ri` を全部 push する。 \(i\) の大きい方から。（ callee は caller に合わせて、 \(w_m\) に入れるべき値から順に push する。）
    - `\%v_ret` に飛ぶ。
- 各手続きの最初：
  - \(\T{lget} \T{\%v_fp}\) で長さを取得。 
  - local variable を push 0 で確保する。
  - \(\T{lget} \T{\%v_sp}\) で長さを取得。
  - これで call stack は次のような状況になっている。スタックは上から入れるような書き方。
    | 入っている値 | 位置 |
    | --- | --- |
    | 局所変数 \(x_0\) | \(v_sp\) |
    | 局所変数 \(x_1\) | \(v_sp - 1\) | 
    | ... | ... |
    | 局所変数 \(x_n\) | \(v_sp - n = v_fp + 1\) |
    | return addr | \(v_fp\) |
    | 引数 \(p_0\) | \(v_fp - 1\) |
    | 引数 \(p_1\) | \(v_fp - 2\) |
  - このスタックの長さは最終的に関数を抜ける return 以外では変化しない。
    他の手続きを call しても、ちゃんと処理が終わった後はこの長さになっている。
    なので、相対位置を手続きの初めに確定したらあとは計算する必要がない。
- main だけ戻る先がないので特別扱いする必要がある。コンパイル先の flow_ir で `:halt {halt; }` だけ書かれたものを用意して、
  return addr を一番最初に `:halt` に向けて push しておく。
  これで、 main も implicit/explicit return で扱える。

# internal control
手続きの**中**での制御の実装。
nested labelled loop + break value/continue を付ける。

\(\begin{aligned}
\NT{var}    &\defeq \NT{string} \\
\NT{atom}   &\defeq \NT{var} | \NT{imm} \\

\NT{aexp}   &\defeq \NT{atom} \NT{binop} \NT{atom} \\
\NT{bexp}   &\defeq \NT{atom} \NT{binrel} \NT{atom} \\

\NT{label}  &\defeq \T{:} \NT{string} \\

\NT{stmt} &\defeq \\
  &| \T{Nop}
  &| \NT{stmt} \T{;} \NT{stmt} \\
  &| \NT{var} \T{:=} \NT{aexp} \\
  &| \T{if} \NT{bexp} \T{\LCB} \NT{stmt} \T{\RCB} \\
  &| \T{break} \NT{label} \NT{var} \\
  &| \T{continue} \NT{label} \\
  &| \T{loop} \NT{label} \T{\LP} \NT{stmt} \T{\RP} \T{->} \NT{var} \\

\NT{program} &\defeq \NT{static} \syntaxmacro{comma-separated}{\NT{var}} \T{;} \\
  &\NT{stmt}
\end{aligned}\)

- ラベルの名前も static も変な被り方はしていないものとする。

## 意味論
Machine としては明らかに大ジャンプをする必要がある。
どういうことをするか：nesting があるので、
今どの階層にいるかを手続き呼び出しスタックと同じように管理する。
結果についても Normal な終了以外に break, continue がある。
try catch と同じように外部に一気に飛ぶので、ここら辺は exception と同じ。
\(V := \N + \text{Break}(L, V) + \text{Continue}(V)\) と定義すると control 自体を扱えるようになっていておかしい。
これができたら面白いけど、ここではやらない。
big step としては上の \(Ef := \N + \text{Break}(L, V) + \text{Continue}(V)\) みたいに recursive にならないようにする。
`AInput` と `FOutput` は static の書き換えと考えておく。
big step の方が自然に見えるが、 small step にした。

> [!Note]
> 文の評価結果としては \(() + \NT{Break}(L, V) + \text{Continue}(V)\) と考えたほうがいい。
> 文指向言語だがこの点で評価結果を入れておくところが必要になる。

- 定義
  - \(V := \N\) ... ちゃんとした値
  - \(E := \NT{var} \to V\) ... 環境（ local が一切ない。）
  - \(F := \text{Loop}(\NT{label}, \NT{stmt}, \NT{var}) + \text{Seq}(\NT{stmt})\) ... stack に積む frame
    - \(\text{Loop}(L, s, x)\) ... loop の frame, これがあると、今はこの loop にいることがわかる。
    - \(\text{Seq}(\NT{stmt})\) ... stmt の frame, これは次に実行する文。
  - \(K := \text{List} \langle F \rangle\) ... 現在の context
  - \(R := () + \text{Break}(L, V) + \text{Continue}(L)\) ... 文の評価結果
    - 書いていたら、 \(\T{Nop}\) と \(()\) が同じような役割になっていた。
  - \(S := (\NT{stmt} + R, E, K)\)
    - stmt を入れるところに文の評価結果としての \(R\) まで入れているのでややこしいです。

- 動き方
  - \(R = ()\) case
    - \(((), E, \emptyset) \to E\) ... `FOutput` の出力
    - \(((), E, \text{Loop}(L, s, x)::K) \to (s, E, \text{Loop}(L, s, x)::K)\) ... Nop でループが終わったので、もう一回
    - \(((), E, \text{Seq}(s):: K) \to (s, E, K)\) ... \(K\) から文を pop する。
  - \(R \neq ()\) case
      - \((R, E, \emptyset)\) ... ループにいないのに break/continue しているので、 `Err`
      - \((R, E, \text{Seq}::K) \to (R, E, K)\) ... この文は実行しない。
      - \((R, E, \text{Loop}(L, s, x)::K) \to (R, E, K)\) if \(R\) のラベルが \(L\) じゃない場合 ... このループじゃないので脱出
      - \((\text{Break}(L, v), E, \text{Loop}(L, s, x)::K) \to ((), E', K)\) where \(E'[x := v]\)
        - 対応するループをちょうど抜ける。結果を入れておくところは \(()\) に戻す。
      - \((\text{Continue}(L), E, \text{Loop}(L, s, x)::K) \to ((), E, \text{Loop}(L, s, x)::K)\)
        - 対応するループをちょうど抜ける。結果を入れておくところは \(()\) に戻す。
  - seq ... \((s_1; s_2, E, K) \to (s_1, E, \text{Seq}(s_2)::K)\) だけでいい。
  - Nop
    - \((\T{Nop}, E, K) \to ((), E, K)\) ... 結果は `()`
  - if/assign （control effect なし）
    - \((\T{if}(b, s), E, K) \to (s, E, K)\) if \(\llbracket b \rrbracket\)
    - \((\T{if}(b, s), E, K) \to ((), E, K)\) if \(\neg \llbracket b \rrbracket\)
    - \((x := a, E, K) \to ((), E', K)\) where \(E' := E[x := \llbracket a \rrbracket_E]\)
  - break/continue （control effect をスタックに積む。）
    - \((\T{break}(L, x), E, K) \to (\text{Break}(L, v), E, K)\) where \(v := \llbracket x \rrbracket_E\)
    - \((\T{Continue}(L), E, K) \to (\text{Continue}(L), E, K)\)
  - loop
    - \((\T{loop}(L, s, x), E, K) \to ((), E, \text{Loop}(L, s, x)::K)\) ... loop 用のをスタックに積む。

あとは `AInput` としての \(E\) を受け取って \((s, E, \emptyset)\) で動かせばいい。

## flow_ir へのコンパイル
正直あまり述べることがない...
むしろ small step による定義をするほうが tricky で理論過ぎて、書くのが大変だった。
ここでも前方参照の問題が立ちはだかる以外は素直に書ける。
これは変数を static に割り当てる。

loop は entry と exit の 2 つのブロックに分ける。

> [!Note]
> break の代入先は（今はラベルを first class に扱えないので、）静的に全体を解析すれば一応わかる。
> ただし解析は局所的にはできない。こんな案がある。
> - 静的に全体を解析して、代入先の変数を手に入れておく。
> - 仮想レジスタの名前を一致させておくような方法をとる。
> - stack に push して loop の exit で pop から代入する。

# data
直和と直積と配列のある言語を作る。
一応注意点として、**メモリの扱い方** としての言語を考えるので、
値を渡さないでメモリへの代入をわかりやすくするというコンセプトで行く。
また、変数については最初から場所としておく。
（仮想レジスタも式もないので、一時変数には毎回メモリを割り当てていることになるが、
まあここでやりたいのはそれを回避することではないので。）

\(\begin{aligned}
\NT{var}    &\defeq \NT{string} \\

\NT{prim-value}    &\defeq \NT{imm} | \T{\#true} | \T{\#false} \\
\NT{place-expr}   &\defeq \NT{var} | \NT{place-expr} \T{.l} | \NT{place-expr} \T{.r} | \NT{place-expr} \T{\LSB} \NT{value-expr} \T{\RSB} \\
\NT{value-expr}   &\defeq \NT{prim-value} | \T{ld} \NT{place-expr} | \T{inl} \NT{value-expr} | \T{inr} \NT{value-expr} \\

\NT{stmt}   &\defeq \\
  &| \NT{place-expr} \T{:=} \NT{value-expr} \\
  &| \T{inl} \NT{place-expr} \T{=>} \NT{var} \T{\LCB} \NT{stmts} \T{\RCB} \\
  &| \T{inr} \NT{place-expr} \T{=>} \NT{var} \T{\LCB} \NT{stmts} \T{\RCB} \\
  &| \T{if} \NT{value-expr} \T{\LCB} \NT{stmts} \T{\RCB} \\
  &| \T{while} \NT{value-expr} \T{LCB} \NT{stmts} \T{\RCB} \\

\NT{stmts} &\defeq \syntaxmacro{semicolon-separated}{\NT{stmt}} \\

\NT{type} &\defeq \T{Num} | \T{Bool}\\
  &| \T{pair} \NT{type} \NT{type} | \T{sum} \NT{type} \NT{type} \\
  &| \T{arr} \NT{type} \NT{imm} \\
  &| \T{\LP} \NT{type} \T{\RP} \\

\NT{program} &\defeq \NT{static} \syntaxmacro{comma-separated}{\NT{type} \NT{var}} \T{;} \NT{stmts}
\end{aligned}\)

inl と inr は place を受け取った後に in-place に"中身へのアクセス"を変数経由で受け取るとする。
なので、 `p := inl(1); inl p => x { x := 2 };` を書くと `p := inl(2)` になっている。

> [!warning]
> 中に入った後では `p` への書き換えは `x` への書き換えになっていることに注意する。
> `p := inl(1); inp => x { p := inr(2); x := 1 }` は明らかに不正。
> なので、これを入れた時点で alias の問題が発生している？
> これを考えると、 `alias <var> := <place-expr>` とかを入れてもやらなければいけないことは変わらない。
> 変数使えなくなったのではなくて、変数は Key に結びついていて、 Valid な Key の全体が変化したと考えたほうが楽？
> オブジェクト指向でも、親が書き変わって子オブジェクトへの名前が使えなくなるとかはあるし。
> その点で、意味論側ではよくない操作をしたらただちに `Err` を吐くようにする必要がある。

コンパイルのことを考えると、変数がどれぐらいセルを使うかを計算するために、実行前にサイズがわかってないといけない。
例えば、 `x := inl(1)` に対して、 `x := inr(?)` が一度も実行されなかったときにはどうなっているのか？
それを確定させておくためにも型の anotation が必要になる。

## 意味論

とりあえず値はこんな感じ。
- \(V_p := \N + \T{\#true} + \T{\#false}\)
- \(V := V_p + (V * V) + [V; \N] + (V + V)\)

`ld` を連結した領域に対して行えるので、 \(V_p\) じゃなくて \(V\) の転送ができる。
`x.l := 1; x.r := 2; y := ld x;`

`inl p => x { x := 1;} ` が書けるようになった時点で、 variable は他の variable と別の領域を確保しているとは限らない。
そのため、 variable がそのまま location にはならないから、別に location を用意する必要がある。
variable -> location が言語の意味論上の束縛を表していて、実際のメモリは location -> value と思えばいい。
（ location の中に値が入っているというよりも、 location という Key をもとにした key value store と思う。）

- \(L := \NT{var} + \text{Left}(L) + \text{Right}(L) + \text{Idx}(L, \N) + \text{Inl}(L) + \text{Inr}(L)\) ... key
  - `(x.l).r` は \(\text{Right}(\text{Left}(x))\) に解釈されるので内部と外部っぽいものが入れ替わっていることに注意。

後はメモリは \(L \pfun V\) と思えばいい。
- 全ての Key が valid ではないこれは \(\text{Inl}, \text{Inr}\) を入れなくても同じ）。
  なので Key 自体には valid を考えずに Key から value を引いてくるときに、存在しない場合は `Err` になる、という風に思った方がいい。
- \(L(k) \in V * V\) だったとき、これをメモリの移動の単位として扱えるようにしつつ、分解ができないといけない。
  例えば、 `x := y` で `M[x]` も `M[y]` も \(V * V\) に入っている状況なら、
  \(M[y]\) という複数の"セル"の内容を \(M[x]\) に移すという形で、そのまま扱えてほしい。
  一方で、 `y.l := x` みたいな状況では \(M[\text{Left}(y)]\) への書き込みなので \(M[y]\) の内容にも変更があるべき。

これを考えると value は \(e: L \pfun V\) であって次のような条件を満たしているものと思える。
（写像全体はそれ自体が自由性を持っているので、書き込みについての条件を課すのはここではない。）
- \(\text{Left}(k)\) か \(\text{Right}(k)\) のどちらかが domain に入ればもう片方も domain に入り、かつ \(k\) 自身も domain に入り：
  - \(e(k) = ( e(\text{Left}(k)), e(\text{Right}(k)) )\)
- \(\text{Idx}(k, 0)\) が domain に入るならある \(n\) が存在して \(\text{Idx}(k, 0 \leq i < n)\) がちょうど \(i\) が domain に入る全体になっていて：
  - \(e(k) = [ e(\text{Idx}(k, 0) \ldots e(\text{Idx}(k, n))) ]\)
- \(\text{Inl}(k)\) が domain に入るなら \(\text{Inr}(k)\) は domain に入らず、
  - \(e(k) = (0, e(\text{Inl}(k)))\)

書き込み条件：\(L\) には親子関係で（全ではない）順序が入るが、
順序関係のない \(k, k'\) に対しては \(k\) への書き込みは \(k'\) のメモリに影響を与えないようにしたい。
そのためには \(e[k := v]\) を頑張って定義すればいい。（関係ある部分全部を書き換えて、それ以外を書き換えなければいい。）
注意点としてどんな代入も許されるわけではないし、 \(\text{Inl}\) の問題を考えると \(e\) 自体が型の情報を持っておかないといけない。
（ \(e[\text{Inl}(k) := 1]\), \(e[\text{Inr}(k) := 1]\), \(e[\text{Inl}(k) := \#true]\) をはじく必要がある。）

実装上は \(e: L \pfun V\) なんて使っていられないので、 \(L\) の木の構造みたいなものを使う。
- \(M\) :=
  - \(\text{cell-n}(\N) + \text{cell-b}(\mathbb{B}) + \) ... これは cell
  - \(\text{prod}(\NT{type}, \NT{type}, M, M) +\) ... left type, right type, left mem, right mem
  - \(\text{pair}(\NT{type}, \NT{type}, \{0, 1\}, M) +\) ... left type, right type, tag(0 left or 1 right), mem
  - \(\text{arr}(\NT{type}, \N, M) \) ... type of elem, max idx
- \(M_{\text{top}} := \NT{var} \to M\)

> [!Tip]
> 最終的な結論としては、メモリは "意味論としては" \(L \pfun V\) で条件を満たすもの
> 実装上は `HashMap` とかは使わないで \(M\) のようにして実装しないと辛い。

ところで、 `inl p => x {...}` の導入で必然的にスコープが生まれるので、変数環境はこれを保っておく必要がある。
そのため、 stmt の評価の時点で状態機械が（残りの文を含めた）スタックを持つことになる。
なので、 `loop` と同じようにして loop 内の継続と残りの文の継続を分ける必要がある。

最終的には、こんな感じでの定義？
- \(L\) := 上で定義したようにする ... location の key
- \(M\) := abstract には、 \(L \pfun V\) でよくて、実装は上で書いた木構造 ... location ごとに value の定まっている（木構造的な）メモリ
  - \(\text{get}(m: M, l: L) \pfun V\) と \(\text{set}(m: M, l: L, v: V) \pfun M\) の interface があって、制約条件を満たすデータ構造ならなんでもよい。
  - \((m, l, v) \in \dom {\text{set}}\) は \((m, l) \in \dom {\text{get}}\) かつ \(v\) が \(l\) と木構造上同じである場合にのみ成り立つ。
  - \(\text{get}(\text{set}(m, l, v), m, l) = v\) ... get/set の整合性
  - \((\text{get}(m, \text{left}(k)), \text{get}(m, \text{right}(k))) = \text{get}(m, k) \) みたいに上で述べた、関係性に関する条件
  - \(k < k'\) について影響を及ぼさないと述べた、無関係性に関する条件
- \(B\) := \((\NT{var}, L\)\) のスタック ... 変数がどの location に束縛されているかを表すスタック（スコープ用に束縛状況を保存しておく）。
- \(K\) := \(\text{scope} + \text{seq}(\NT{stmt})\) ... ループ内の目印と残りの文の継続
  - ループ内なら全部使った時点でスコープを pop して、そうじゃなかったら変えない。
- \(\text{eval-place}(s: B): \NT{place-expr} \pfun L\) :=
  - \(\NT{var}\) なら \(B\) から引いてくるか、 static にあるならそれを使う。
  - それ以外は recursive に呼び出して \(L\) にする。
- \(\text{eval-value}(s: S): \NT{value-xpr} \pfun V\) :=
  - ld のときだけ、 place-expr を評価してメモリから load 値を取り出す。
- \(\text{eval-stmt}: S \to S\) := \(K\) が \(\text{scope}\) ならそれを \(K\) から削除して \(B\) を pop する。そうじゃない場合は \(K = \text{seq}(s)\) を pop して次のように分岐
  - \(\NT{place-expr} \T{:=} \NT{value-expr}\) なら \(M\) の書き換え
  - \(\T{if} b [s]\) なら \(b\) を評価して
    - true なら \(K := [s] ++ K\) のように extend する。
    - false なら何もしなくていい。
    - それ以外は `Err`
  - \(\T{while} b [s]\) は \(b\) を評価して
    - true なら \(K := [s] ++ [\T{while} b [s]] ++ K\) みたいにする。
    - false なら何もしなくていい。
    - それ以外は `Err`
  - \(\T{inl} p x [s]\) の場合には、 \((x, \text{eval-place}(p))\) を \(B\) に積み、 \(K := [s] ++ \text{scope} ++ K\) みたいにする。

> [!warning]
> メモリは、 **実装上は...で書かれた木構造を使って実装すること*
