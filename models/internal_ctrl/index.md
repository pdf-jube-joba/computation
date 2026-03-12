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
