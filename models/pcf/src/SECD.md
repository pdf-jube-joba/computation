[[Model]]
[[Compiler]]

## definition
CEK よりももっと低級に行う。具体的な値を stack に積んでから呼び出す。
Code も instruction として、スタックマシン方式になる。

> [!note]
> - [youtube の大堀淳の講義動画](https://www.youtube.com/watch?v=JJ5EQ6ldniE)
> - [Wikipedia](https://ja.wikipedia.org/wiki/SECD%E3%83%9E%E3%82%B7%E3%83%B3)
> Wiki の方だと、変数はスタックの上から何番目みたいな指定で、 sel に対する join は return とは異なる扱いになっている。

- \(F\) := \(\text{Fun}(x, C, E) + \text{Rec}(f, x, C, E)\) ... いわゆる closure
- \(V\) := \(\N + \T{\#true} + \T{\#false} + \T{\#unit} + F\)
    - こっちの \(V\) では return 時に closure は評価する際に使った環境をつかむ必要があるので、 \(E\) との recursive な定義になる。
- \(E\) := \(\text{list}(\NT{var}, V)\)
- \(S\) := \(\text{list} V\)
- \(I\) :=
  - \(\text{push} (\N | \T{\#true} | \T{\#false} | \T{\#unit})\)
  - \(\text{print}(n)\)
  - \(\text{acc} (\NT{var})\)
  - \(\text{mkcls}(x, C) \)
  - \(\text{mkrec}(f, x, C)\)
  - \(\text{binop}(\NT{binop})\)
  - \(\text{unop}(\NT{unop})\)
  - \(\text{ap}\)
  - \(\text{ret}\)
  - \(\text{if}(C, C)\)
- \(C\) := \(\text{list} I\)
- \(D\) := \(\text{list}(E, C)\)

状態の遷移：
\[
\begin{aligned}
(S, E, \text{push}(c)::C, D) &\to (c::S, E, C, D) \\
(S, E, \text{print}(n)::C, D) &\to (S, E, C, D) \\
(S, E, \text{acc}(x)::C, D) &\to (v::S, E, C, D) && v = \text{lookup}(E, x) \\
(S, E, \text{mkcls}(x, C)::C_2, D) &\to (\text{Fun}(x, C, E)::S, E, C_2, D) \\
(S, E, \text{mkrec}(f, x, C)::C_2, D) &\to (\text{Rec}(f, x, C, E)::S, E, C_2, D) \\
(v_1::v_2::S, E, \text{binop}(b)::C, D) &\to (v'::S, E, C, D) && v' = v \mathrel{b} v' \\
(v::S, E, \text{unop}(b)::C, D) &\to (v'::S, E, C, D) && v' = \mathop{b} v \\
(v::\text{Fun}(x, C_f, E_f)::S, E, \text{ap}::C, D) &\to (S, (x, v)::E_f, C_f, (E, C)::D) \\
(v::\text{Rec}(f, x, C_f, E_f)::S, E, \text{ap}::C, D) &\to (S, (x, v)::(f, \text{Rec}(f, x, C_f, E_f))::E_f, C_f, (E, C)::D) \\
(S, E, \text{ret}::C, (E_0, C_0)::D) &\to (S, E_0, C_0, D) \\
(\T{\#true}::S, E, \text{if}(C_t, C_f)::C, D) &\to (S, E, C_t +C, D) \\
(\T{\#false}::S, E, \text{if}(C_t, C_f)::C, D) &\to (S, E, C_f +C, D) \\
\end{aligned}
\]

## compile
[expr](./expr.md) からのコンパイルを考える。
- \(\NT{expr}\) の定義に対して、終わった後にそれを評価した結果の \(V\) がスタックに積まれているといい。
- それぞれを命令列 \(C\) に落とし込む。

\[
  \begin{aligned}
  \LLB c = \N | \T{\#true} | \T{\#false} | \T{\#unit} \RRB &= [\text{push}(c)] \\
  \LLB \text{print} n \RRB &= \text{print}(n) \\
  \LLB x \in \NT{var} \RRB &= \text{acc}(x) \\
  \LLB \T{fun} x \T{=>} M \RRB &= \text{mkcls}(x, \LLB M \RRB) \\
  \LLB \T{rec} f x \T{=>} M \RRB &= \text{mkrec}(f, x, \LLB M \RRB) \\
  \LLB e_1 (b \in \NT{binop}) e_2 \RRB &= \LLB e_1 \RRB + \LLB e_2 \RRB + \text{binop}(b) \\
  \LLB (u \in \NT{unop} e) \RRB &= \LLB e \RRB + \text{unop}(u) \\
  \LLB e_1 (e_2) \RRB &= \LLB e_1 \RRB + \LLB e_2 \RRB + \text{ap} + \text{ret} \\
  \LLB \T{if} L M N \RRB &= \LLB L \RRB + \text{if}(\LLB M \RRB, \LLB N \RRB)
  \end{aligned}
\]
