[[Model]] [[IR]]

この言語の特徴
- 書き込み先はテープと変数の2種類
  - `@` という変数はテープの指しているセルを指す。
  - 変数もテープも事前に与えられた有限集合 \(S\) の値しかとらないものとする。
- 関数は再帰はできない。呼び出しグラフは有向非巡回である。衛生的なマクロ展開に近い。
  - 変数は関数呼び出し前後で共有されない。
  - テープは関数呼び出し前後で共有される。
- `main` と名前の付いた関数を呼び出して始まる。存在しないならエラーになる。

\[
\begin{aligned}
\NT{label}    &\defeq \NT{string} \\
\NT{var}      &\defeq \NT{string} \\
\NT{rvalue}   &\defeq \T{@}   \sp | \sp \NT{var} \\
\NT{lvalue}   &\defeq \T{@}   \sp | \sp \NT{var} \sp | \sp S \\

\NT{cond}     &\defeq \T{if} \sp \NT{rvalue} \sp \T{=} \sp \NT{rvalue} \\
\NT{stmt}   &\defeq (\\
  &|\T{LT} |\T{RT}  \\
  &|\NT{;value} \sp \T{:=} \sp \NT{rvalue} \\
  &|\T{jump}      \sp \NT{label}  \sp \NT{cond} \\
  &|\T{break}                     \sp \NT{cond} \\
  &|\T{continue}                  \sp \NT{cond} \\
  &|\T{call}      \sp \NT{name} \\
  &|\T{return}                    \sp \NT{cond} \\
  ) \T{;} \\
\NT{block}    &\defeq \NT{label} \sp \T{:} \NT{stmt}* \\
\NT{function} &\defeq \NT{name} \sp \T{\{} \NT{block} \T{\}} \\

\NT{alphabet-decl} &\defeq \T{alphabet} \sp \T{\LP} \syntaxmacro{comma-separated}{S} \T{\RP} \\
\NT{program}    &\defeq \NT{alphabet-decl} \sp \NT{function}+
\end{aligned}
\]

プログラム状態
: 実行状態は (tape, head, env) とする。
  - tape: \(\Z \to S\), 有限を除いて \(\mathbb{B}\) に移る
  - head: \(\Z\)
  - env: \(V \to S\), 有限を除いて \(\mathbb{B}\) に移る

基本命令
: 
  - LT: head := head - 1
  - RT: head := head + 1
  - @ := w: tape[head] への w の代入
  - v := w: env[v] への env[w] の代入
  - v := @: env[v] への tape[head] の代入

制御
:
  - jump: 対応するブロックに飛ぶ
  - continue 及び break: 今あるブロックの先頭に飛ぶ、次のブロックの先頭に飛ぶ
  - call/return: 別の関数を呼び出す、今の関数を抜ける

<div data-model="rec_tm_ir"></div>
