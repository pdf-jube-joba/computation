[[Model]] [[Actor Model]]

# message passing and actor model
各 message 

各 stmt はループしないので停止する。（分岐はすることに注意。）

\[
\begin{aligned}
\\
\NT{bit} &\defeq 0 | 1 \\
\NT{message} &\defeq \NT{string} \NT{bit}* \\
\\
\NT{expr} &\defeq \NT{bit} \\
    &| \T{head} \NT{expr} \\
    &| \T{tail} \NT{expr} \\
\\
\NT{stmt} &\defeq \\
    &| \T{this} \T{.} \NT{var} \T{+=}   &\syntaxname{self modify} \\
    &| \T{send} \NT{string} \NT{bit}*            &\syntaxname{send message} \\
\\
\NT{object} &\defeq \T{\LSB} \\
    &\syntaxmacro{comma-separated}{\NT{var}} \T{;} \\
\T{\RSB} \\
\\
\end{aligned}
\]