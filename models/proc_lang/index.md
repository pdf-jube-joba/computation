call/return で手続呼び出しを行う言語を考える。
- 引数と返り値あり。
  - いずれも複数でいい。
- 大域変数と局所変数あり。
  - global か自分の local で宣言された変数のみ使用できる。
  - 呼び出し元手続きの変数環境は見えないものとする。
- 値呼びにする。

\(\begin{aligned}

\NT{var}    &\defeq \NT{string} \\
\NT{name}   &\defeq \NT{string} \\
\NT{atom}   &\defeq \NT{var} | \NT{imm} \\

\NT{aexp}   &\defeq \NT{atom} \NT{binop} \NT{atom} \\
\NT{bexp}   &\defeq \NT{atom} \NT{binrel} \NT{atom} \\

\NT{var-cmm} &\defeq \syntaxmacro{comma-separated}{\NT{var}} \\

\NT{stmt} &\defeq \\
  &| \T{Nop}
  &| \NT{string} \T{:=} \NT{aexp} \\
  &| \NT{stmt} \T{;} \NT{stmt} \\
  &| \T{if} \NT{bexp} \T{\LSB} \NT{stmt} \T{\RSB} \\
  &| \NT{while} \NT{bexp} \T{\LSB} \NT{stmt} \T{\RSB} \\
  &| \T{call} \NT{name} \T{\LP}  \T{\RP} \T{->} \NT{var-cmm} \\

\NT{proc} &\defeq \NT{string} \T{LP} \NT{var-cmm} \T{\RP} \T{\LSB} \\
  &\T{local} \NT{var-cmm} \\
  &\NT{stmt} \\
  &\T{return} \NT{var-cmm} \\
  &\T{\RSB} \\

\NT{program} &\defeq \\
  &\T{global} \NT{var-cmm} \\
  &\NT{proc}+ \\
\end{aligned}\)

## 意味論（ small-step ）
call-stack が必要になるぐらいで、あとは `while_lang` と同じ。
引数位置に var が来て値渡しなので評価順序も引数評価用のフレームもいらない。
cont の方に呼び出し元の環境も入れておくんだっけ？
