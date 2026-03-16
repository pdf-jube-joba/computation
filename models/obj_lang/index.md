# object identity
object identity を入れるなら、 `new` が欲しい。

\(\begin{aligned}

\NT{x} &\defeq \NT{string} \\
\NT{f} &\defeq \NT{string} \\

\NT{expr} &\defeq \\
    &| \T{this} \\
    &| \NT{object-decl} \\
    &| \N | \T{\#true} | \T{\#false} | \T{null} \\
    &| \NT{x} | \NT{x} \T{.} \NT{f} \\
    &| \NT{expr} \NT{expr} \\
    &| \T{fun} \NT{x} \T{=>} \NT{expr} \\
    &| \NT{expr} \NT{binop} \NT{expr} \\
    &| \NT{expr} \T{=} \NT{expr} \\
    &| \NT{expr} \T{===} \NT{expr} \\
    &| \T{begin} \NT{stmt}* \T{end} \\

\NT{stmt} &\defeq \\
    &| \NT{x} \T{:=} \NT{expr} \\
    &| \NT{x} \T{.} \NT{f} \T{:=} \NT{expr} \\
    &| \T{if} \NT{expr} \T{then} \NT{block} \\
    &| \T{while} \NT{expr} \T{then} \NT{block} \\
    &| \T{return} \NT{expr} \\

\NT{field-decl} &\defeq \NT{x} \T{:} \NT{expr} \T{;} \\
\NT{object-decl} &\defeq \T{new} \T{\LSB} \NT{field-decl}* \T{\RSB} \\

\NT{program} &\defeq \NT{object-decl}+ \\

\end{aligned}\)

# message passing and actor model
ここでは、 object が生成されることはない。
（ややこしいから。）

各 stmt はループしないので停止する。（分岐はすることに注意。）

\(\begin{aligned}

\NT{bit} &\defeq 0 | 1 \\
\NT{message} &\defeq \NT{string} \NT{bit}* \\

\NT{expr} &\defeq \NT{bit} \\
    &| 
    &| \T{head} \NT{expr} \\
    &| \T{tail} \NT{expr} \\

\NT{stmt} &\defeq \\
    &| \T{this} \T{.} \NT{var} \T{+=}   &\syntaxname{self modify} \\
    &| \T{} \\
    &| \T{send} \NT{string} \NT{bit}*            &\syntaxname{send message} \\

\NT{object} &\defeq \T{\LSB} \\
    &\syntaxmacro{comma-separated}{\NT{var}} \T{;} \\
\T{\RSB} \\

\end{aligned}\)

# class based object
AI の出した構文をちょっと変えた。

\(\begin{aligned}

\NT{x} &\defeq \NT{string} \\
\NT{f} &\defeq \NT{string} \\
\NT{m} &\defeq \NT{string} \\

\NT{name} &\defeq \NT{string} \\

\NT{expr} &\defeq \\
    &| \T{this} \\
    &| \N | \T{\#true} | \T{\#false} | \T{null} \\
    &| \NT{x} | \NT{x} \T{.} \NT{f} \\
    &| \NT{expr} \NT{binop} \NT{expr} \\
    &| \NT{expr} \T{=} \NT{expr} \\
    &| \NT{expr} \T{===} \NT{expr} \\

\NT{stmt} &\defeq \\
    &| \NT{x} \T{:=} \NT{expr} \\
    &| \NT{x} \T{.} \NT{f} \T{:=} \NT{expr} \\
    &| \T{if} \NT{expr} \T{then} \NT{block} \\
    &| \T{while} \NT{expr} \T{then} \NT{block} \\
    &| \NT{x} \T{:=} \T{new} \NT{name} \\
    &| \NT{x} \T{:=} \NT{x} \T{.} \NT{m} \T{\LP} \syntaxmacro{comma-separated}{\NT{expr}} \T{\RP} \\
    &| \T{return} \NT{expr} \\

\NT{field-decl} &\defeq \T{var} \NT{x} \\
\NT{method-decl} &\defeq \T{proc} \NT{m} \T{\LP} \syntaxmacro{comma-separated}{\NT{x}} \T{\RP} \NT{block} \\
\NT{block} &\defeq \T{begin} \NT{stmt}* \T{end} \\

\NT{object-decl} &\defeq \NT{name} \T{\LSB}  \NT{field-decl}* \NT{method-decl}* \T{\RSB} \\

\NT{program} &\defeq \NT{object-decl}+ \\

\end{aligned}\)


