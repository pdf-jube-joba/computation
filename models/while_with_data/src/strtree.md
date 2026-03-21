[[Model]]

文字の入った木構造で while 言語を書く。
文字の集合として有限で同値性判定のある \(S\) を実世界の文字にしておく。
木構造はこんな感じで複数定義できる。

- \(T = 1 + T * S * T\)
  これは配列の自然な定義 \(T = 1 + S * T\) に近い。
- \(T = S + T * T\)
  これは leaf に atom が来るパターン

> ![Note]
> ところで、 while 言語は自然数を扱っていて aexp/bexp の2種類のデータ型だけで構成できる。
> bexp についても判定に使うだけなので、aexp から if を抜いて bexp は 0 かの判定か aexp の同値判定だけできればいい。
> 配列を扱っているときは head をとる操作を配列に閉じるようにすることで似たようなことができたが、
> 木構造だとあまり自然に思えない。
> なので、素直に3種類目のデータとして"文字単体"を入れたほうがいい。

ここでは2種類目を採用する。
なお、普通に意味論上での stuck が発生する。
理由は、空の木が表現できないので、"失敗したとき"があまりうまく表現できないから。

\[
\begin{aligned}
\NT{var} &\defeq \NT{string} \\
\NT{atom-char} &\defeq \T{'} \NT{char} \T{'} \\
\\
\NT{aexp} &\defeq \\
  &| \T{mtoa} \NT{texp} \\
  &| \T{atom-char} \\
\NT{texp} &\defeq \\
  &| \NT{var} \\
  &| \T{atom} \NT{aexp} \\
  &| \T{cons} \NT{texp} \NT{texp} \\
  &| \T{left} \NT{texp} \\
  &| \T{right} \NT{texp} \\
\NT{bexp} &\defeq \\
  &| \NT{is-atom} \NT{texp} \\
  &| \NT{texp} \T{==} \NT{texp} \\
\\
\NT{stmt} &\defeq \\
  &| \T{Nop} \\
  &| \NT{stmt} \T{;} \NT{stmt} \\
  &| \NT{var} \T{:=} \NT{texp} \\
  &| \T{ifeq} \NT{bexp} \T{then} \NT{stmt} \T{end} \\
  &| \T{while} \NT{bexp} \T{\LCB} \NT{stmt} \T{\RCB}
\end{aligned}
\]

\(V\) はそのまま \(V = S + V * V\) になる。
ここで、 **未初期化の変数に入れる default の値がない**ため、環境は \(\NT{var} \pfun V\) にならざるをえない。
短絡評価のようなものは行わず、そのまま全部を eval する。

実装で気を付ける部分（部分関数なので、 `Err` を吐く。）
- \(\T{mtoa} (s \in S) = s, \T{mtoa}(t_1, t_2) = \bot\)
- \(\T{left} (s \in S) = \bot, \T{left}(t_1, t_2) = t_1\)
- \(\T{right} (s \in S) = \bot, \T{right}(t_1, t_2) = t_2\)
- 木の equality は構造的等値でやる。
