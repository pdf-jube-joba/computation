とりあえずオブジェクト指向言語を提案してもらった。
\(\begin{aligned}

\NT{x} &\defeq \NT{string} \\
\NT{f} &\defeq \NT{string} \\
\NT{m} &\defeq \NT{string} \\

\NT{expr} &\defeq \\
    &| \N | \T{\#true} | \T{\#false} | \T{null} \\
    &| \NT{x} | \T{this} | \NT{x} \T{.} \NT{f} \\
    &| \NT{expr} \NT{binop} \NT{expr} \\
    &| \NT{expr} \T{=} \NT{expr} \\
    &| \NT{expr} \T{===} \NT{expr} \\

\NT{stmt} &\defeq \\
    &| \NT{x} \T{:=} \NT{expr} \\
    &| \NT{x} \T{.} \NT{f} \T{:=} \NT{expr} \\

\NT{field-decl} &\defeq \T{var} \NT{x} \\
\NT{method-decl} &\defeq \T{proc} \NT{m} \T{\LP} \syntaxmacro{comma-separated}{\NT{x}} \T{\RP} \NT{block} \\
\NT{block} &\defeq \T{begin} \NT{stmt}* \T{end} \\

\end{aligned}\)

program    ::= object_decl*

object_decl ::= object Name {
                  field_decl*
                  method_decl*
                }

field_decl ::= var x;
method_decl ::= proc m(params) block

block      ::= begin stmt* end

stmt       ::= x := expr;
             | x.f := expr;
             | if expr then block else block
             | while expr do block
             | x := new Name();
             | x := y.m(args);
             | y.m(args);
             | return expr;
             | skip;

expr       ::= n | true | false | null
             | x
             | this
             | x.f
             | expr + expr
             | expr = expr
             | expr == expr