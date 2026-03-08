手続きの中での制御の実装。
多分しょうもない気がする。
nested labelled loop + break value/continue を付ける。

\(\begin{aligned}
\NT{var}    &\defeq \NT{string} \\
\NT{name}   &\defeq \NT{string} \\
\NT{atom}   &\defeq \NT{var} | \NT{imm} \\

\NT{aexp}   &\defeq \NT{atom} \NT{binop} \NT{atom} \\
\NT{bexp}   &\defeq \NT{atom} \NT{binrel} \NT{atom} \\

\NT{stmt} &\defeq \\
  &| \T{Nop}
  &| \NT{stmt} \T{;} \NT{stmt} \\
  &| \NT{var} \T{:=} \NT{aexp} \\
  &| \T{if} \NT{bexp} \T{\LSB} \NT{stmt} \T{\RSB} \\
  &| \T{break} \T{:} \NT{string} (\NT{var})? \\
  &| \T{continue} \T{:} \NT{string} \\
  &| \T{:} \NT{string} \T{loop} \T{\LP} \NT{stmt} \T{\RP} (\T{->} \NT{var})? \\

\NT{program} &\defeq \NT{static} \syntaxmacro{comma-separated}{\NT{var}} \T{;} \\
  &\NT{stmt}
\end{aligned}\)

## 意味論
Machine としては明らかに大ジャンプをする必要がある。
