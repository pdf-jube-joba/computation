[[Model]]

これは普通にIR1の関数定義を展開して、 jump はラベルじゃなくて行数にしたもの。

\(\begin{aligned}
\NT{label}    &\defeq \NT{string} \\
\NT{var}      &\defeq \NT{string} \\
\NT{rvalue}   &\defeq \T{@}   \sp | \sp \NT{var} \\
\NT{lvalue}   &\defeq \T{@}   \sp | \sp \NT{var} \sp | S \\

\NT{cond}     &\defeq \T{if} \sp \NT{rvalue} \sp \T{=} \sp \NT{rvalue} \\
\NT{stmt}   &\defeq (\\
  &|\T{LT} |\T{RT}  \\
  &|\NT{lvalue} \sp \T{:=} \sp \NT{rvalue} \\
  &|\T{jump}      \sp \NT{number}  \sp \NT{cond} \\
  ) \T{;} \\

\NT{alphabet-decl} &\defeq \T{alphabet} \sp \T{\LP} \syntaxmacro{comma-separated}{S} \T{\RP} \\
\NT{program}  &\defeq \NT{alphabet-decl} \sp \NT{stmt}*
\end{aligned}\)

これの意味は明らかなので書かない。

<div data-model="rec_tm_ir_jump"></div>
