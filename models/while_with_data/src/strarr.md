[[Model]]

文字列で while 言語
- 空文字列・定数文字列の宣言
- 文字列結合・分解（ head と tail ）
- 等値性の判定、空文字列かどうか

文字の集合として有限で同値性判定のある \(S\) を実世界の文字にしておく。

\[
\begin{aligned}
\NT{var} &\defeq \NT{string} \\
\NT{const} &\defeq \T{'} S \T{'} \\
\\
\NT{aexp} &\defeq \\
  &| \T{\LSB} \syntaxmacro{comma-separated}{\NT{const}} \T{\RSB} \\
  &| \NT{var} \\
  &| \T{\LR} \NT{aexp} \T{\RP} \\
  &| \NT{aexp} \T{++} \NT{aexp} \\
  &| \T{head} \NT{aexp} \\
  &| \T{tail} \NT{aexp} \\
\NT{bexp} &\defeq \\
  &| \T{is-empty} \NT{aexp} \\
  &| \NT{aexp} \T{==} \NT{aexp} \\
\\
\NT{stmt} &\defeq \\
  &| \T{Nop} \\
  &| \NT{stmt} \T{;} \NT{stmt} \\
  &| \NT{var} \T{:=} \NT{aexp} \\
  &| \T{if} \NT{bexp} \T{then} \NT{stmt} \T{end} \\
  &| \T{while} \NT{bexp} \T{\LCB} \NT{stmt} \T{\RCB}

\end{aligned}
\]
- 定数文字列の宣言については空の配列を許す。

\(V\) を \(S\) の列とすればそのまま素直に意味論が実装できる。
環境としては、空の文字列が `Default` にできるので、
初期環境を \(\NT{var} \mapsto ([] \in V)\) という全域関数と思える。

実装で気を付ける部分
- \(\T{head} [] = [], \T{tail} [] = []\)
- \(\T{head} a::v = [a], \T{tail} a::v = v\)

とする。

> [!Note]
> pattern match だけ入れても強くないが、 pattern match 自体はあると書きやすそう。

> [!Note]
> 自然数で言う mu 再帰みたいな定義を入れるのも面白くて、一般に有限集合 S 上のリストを扱う言語を考えたとして、
> まず S 自体に計算できる equality はほしい、順序があれば辞書順が定義できて、
> f(xs) = [] になる最小の xs とかが定義できる。
> ただし、（これは自然数の場合でも同じだが）非決定性を許すなら mu 再帰っぽいもので"最小"条件を外して、
> `choose xs f` みたいな operator （非決定的に `f(xs) = []` を満たす `xs` を取り出す。）とかを加えてもいい。
> なお、 S は一元集合でもいい。（リストの長さで十分に強いから。）
