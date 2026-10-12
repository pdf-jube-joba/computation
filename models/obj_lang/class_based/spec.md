# class based object
AI の出した構文をちょっと変えた。

\[
\begin{aligned}

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

\end{aligned}
\]


