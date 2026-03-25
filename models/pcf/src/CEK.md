[[Model]]
[[Compiler]]

## definition

[expr](expr.md) の言語を実行するための抽象機械として、 CEK を定義する。
control, environment, kontinuation(continuation) の略らしい。
ここでも stuck を入れる。

- \(F\) := \(\text{Fun}(x, M, E) + \text{Rec}(f, x, M, E)\) ... いわゆる closure
- \(V\) := \(\N + \T{\#true} + \T{\#false} + \T{\#unit} + F\)
    - こっちの \(V\) では return 時に closure は評価する際に使った環境をつかむ必要があるので、 \(E\) との recursive な定義になる。
- \(C\) := \(\NT{expr}\)
- \(E\) := \(\text{list}(\NT{var}, V)\)
- \(K\) := \(\text{list}F\) where F := 
    - \(([] \NT{binop} M, E)\)
    - \((V  \NT{binop} [], E)\)
    - \((\NT{unop} [], E)\)
    - \(([] (N), E)\)
    - \((f \in F)[]\)
    - \((\T{if} [] M N, E)\)
    - \(\text{init}\)
- \(S\) := \(\text{eval}(C, E, K) + \text{return}(V, K)\)

closure だけ環境が値の中に入っているので、 paring の方法が異なる。
また、値としての closure と引数評価中の関数呼び出しの継続を覚えておくための部分で被りがある。

これらに対する遷移：

\[
\begin{aligned}
\text{eval}(v, E, K) &\to \text{return}(v, K) && v = n \in \N | \T{\#true} | \T{\#true} | \T{\#unit} \\
\text{eval}(\T{fun} x \T{=>} M, E, K) &\to \text{return}(\text{Fun}(x, M, E), K) \\
\text{eval}(\T{rec} f x \T{:=} M, E, K) &\to \text{return}(\text{Rec}(f, x, M, E), K) \\
\\
\text{eval}(\T{print} n, E, K) &\to \text{return}(\T{\#unit}, K) \\
\\
\text{eval}(x, E, K) &\to \text{return}(\text{lookup}(E, x), K) \\
\text{eval}(M \NT{binop} N, E, K) &\to \text{eval}(M, E, ([] \NT{binop} N, E) ::K) \\
\text{eval}(\NT{unop} M, E, K) &\to \text{eval}(M, E, (\NT{unop} []) ::K ) \\
\text{eval}(M (N), E, K) &\to \text{eval}(M, E, ([] (N), E) ::K) \\
\text{eval}(\T{if} L M N, E, K) &\to \text{eval}(L, E, (\T{if} [] M N, E) ::K) \\
\\
\text{return}(v, ([] \NT{binop} M, E) ::K) &\to \text{eval}(M, E, (v \NT{binop} [], E) :: K) \\
\text{return}(v, (v' \NT{binop} [], E) ::K) &\to \text{return}(v'', K) && v'' = v \NT{binop} v' \\
\text{return}(v, (\NT{unop} [], E) ::K) &\to \text{return}(v', K) && v' = \NT{unop} v \\
\\
\text{return}(f \in F, ([] (N), E_a)::K) &\to \text{eval}(N, E_a, (f [])::K) \\
\text{return}(v, (\text{Fun}(x, M, E_f) [])::K) &\to \text{eval}(M, (x, v):: E_f, K) \\
\text{return}(v, (\text{Rec}(f, x, M, E_f) [])::K) &\to \text{eval}(N, (x, v)::(f, \text{Rec}(f, x, M, E_f))::E_f ,K) \\
\\
\text{return}(\T{\#true}, (\T{if} [] M N, E):: K) &\to \text{eval}(M, E, K) \\
\text{return}(\T{\#false}, (\T{if} [] M N, E):: K) &\to \text{eval}(N, E, K) \\
\end{aligned}
\]

これで、 \(\text{return}(v, \text{init})\) になったら \(v\) が結果と思えばいい。

## compile
[expr](./expr.md) からのコンパイルを考えるとして、やることはほとんどない。
\(c\) が与えられたら、そのまま \((c, [], \text{init})\) で動かせばいい。

