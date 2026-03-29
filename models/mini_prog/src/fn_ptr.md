[[Model]]
mini_prog はいろいろ詰め込んでいてよくわからないので、ここでは関数呼び出しとポインタだけの解決を図る。

## 構文

\[
\begin{aligned}
\NT{var}, \NT{fn-name} \\
&\defeq \NT{string} \\
\\
\NT{type} &\defeq \\
  &|\T{\#num} | \T{\#ptr} \\
\\
\NT{place-expr} &\defeq \\
  &| \NT{var} \\
  &| \NT{value-expr} \T{\#loc}
    &\syntaxname{dereference} \\
\\
\NT{value-expr} &\defeq \\
  &| \N | \T{\#null-ptr} \\
  &| \NT{value-expr} \NT{value-expr} \NT{binop} \\
  &| \T{ld} \NT{place-expr}
    &\syntaxname{load from location} \\
  &| \NT{place-expr} \T{\#addr}
    &\syntaxname{reference} \\
\\
\NT{stmt} &\defeq \\
  &| \T{assign} \NT{place-expr} \T{:=} \NT{value-expr} &\syntaxname{assign} \\
  &| \T{ifz} \NT{value-expr} \T{then} \NT{stmt} \T{else} \NT{stmt} \T{end} \\
  &| \T{call} \NT{fn-name} \T{\LP} \syntaxmacro{comma-separated}{\NT{value-expr}} \T{\RP}  
    &\syntaxname{function call} \\
  &| \T{return}\\
\\
\NT{fn-decl} &\defeq \\
  &\T{fn} \NT{fn-name}  \\
  &\T{\LP} \syntaxmacro{comma-separated}{\NT{var} \T{:} \NT{type}} \T{\RP} &\syntaxname{block local variable} \\
  &\T{\LCB} \syntaxmacro{semicolon-separated}{\NT{stmt}} \T{\RCB} \\ \\
\NT{program} &\defeq  \NT{fn-decl}* \\
\end{aligned}
\]

fn main を実行する。

## 意味論
動的型付け言語みたいになる。
location 自体を first class にもってくることでそのまま pointer みたいに使える。
local の導入方法は関数しかない。

- \(L\): set ... location 用でなんでもいい。
- \(V\) := \(\N + L\)
- \(E\) := \(\NT{var} \pfun L\)
- \(S\) := \(\texttt{list}(L \to V)\) ... store だが、関数呼び出し前後で push/pop する。
- \(S[l := v]\) := \(l\) が \(E\) に存在する場合に限り \(S[l]\) を \(v\) で上書きしたものとする。

評価はこんな感じ。
value や place の eval は 1step で行えるようになっている。
副作用と環境の変更が行われない。

- \(\text{eval-place}(e: E, p: \NT{place-expr}): L\) := match p with // ... この時点では valid な location か検査しなくてよい。ただし value-expr を除く。
  - var の場合： \(E(x)\) 
  - dereference の場合：\(\text{eval-value}(v) \in L\) なら \(L\)
- \(\text{eval-value}(e: E, v: \NT{value-expr}): V\) := match v with
  - \(\N\) ならそのまま
  - binop は \(V\) に評価してそれらに適用
  - ld は \(\text{eval-place}\) で得た \(L\) に対して、 \(E\) から value を取り出す。
  - reference は \(L \subset V\) にする。
- \(K\) := \(K_{\text{inner}} * K_{\text{fn}}\) :=
  - \(K_{\text{inner}}\) := \(\texttt{list}\NT{stmt}\) ... 普通の stmt の継続
  - \(K_{\text{fn}}\) := \(\texttt{list}(E, K_{\text{inner}})\) ... 関数呼び出し用：抜けるときに環境を recover しつつ、値を代入する。
- \(S\) := \(\text{eval}(\NT{stmt}, E, S, K) + \text{next}(E, S, K) + \text{ret}(S, K)\)

\[
\begin{aligned}
  \text{eval}(\T{assign} p e, E, S, K) &\to \text{ctrl}(\text{next}, E, S', K) \\
    & l: L := \text{eval-place}(p, E) \\
    & v: V := \text{eval-value}(e, E) \\
    & S' := S[l := v], \\
  \text{eval}(\T{ifz} e s_1 s_2, E, S, K) &\to \text{eval}(s_1, E, S, K) &&\text{eval-value}(e, E) = 0 \\
  \text{eval}(\T{ifz} e s_1 s_2, E, S, K) &\to \text{eval}(s_2, E, S, K) && \text{eval-value}(e, E) \neq 0\\
  \text{eval}(\T{call} n (e_i)_i, E, S, K) &\to \text{eval}(s, E_{\text{heap}}, E_{\text{static}}, [E'_{\text{arg}}], K') \\
    & (x_i: t_i), s := \text{coressponding program of } n \\
    & v_i: V := \text{eval-value}(e_i) \\
    & l_i: L := \texttt{new}(S) \\
    & S' := (l_i \mapsto v_i)::S \\
    & E'_{\text{arg}} := [x_i \mapsto (t_i, v_i)] \\
    & K' := \text{fn-call}(E_{\text{local}}, (l_i)_i)::K \\
  \text{eval}(\T{return} (e_i)_i, E, K) &\to \text{ret}((v_i)_i, E, K) \\
    & v_i := \text{eval-value}(e_i, E) \\
  \text{eval}(\text{block} (x_i: t_i)_i (s_i)_i, E_{\text{heap}}, E_{\text{static}}, E_{\text{local}}, K) &\to \text{ctrl}(\text{next}, E_{\text{heap}}, E_{\text{static}}, E'_{\text{local}}, K') \\
    & E_f := [x_i \mapsto (t_i, \text{default-value}(t_i))] \\
    & E'_{\text{local}} := E_f :: E_{\text{local}} \\
    & K' := (s_i) ++ \text{scope}::K \\
\\
  \text{ctrl}(r, E, \text{scope}::K) &\to \text{ctrl}(r, E', K) \\
    & F :: E'_{\text{local}} := E_{\text{local}} \\
    & E' := (E_{\text{heap}}, E_{\text{static}}, E'_{\text{local}}) \\
  \text{ctrl}(\text{next}, E, \NT{stmt}::K) &\to \text{eval}(s, E, K) \\
  \text{ctrl}(r, E, \NT{stmt}::K) &\to \text{ctrl}(r, E, K) && r = \text{ret} | \text{break} | \text{continue} \\
  \text{ctrl}(\text{next}, E, \text{loop}(n, s)::K) &\to \text{eval}(s, E, \text{loop}(n, s)::K) \\
  \text{ctrl}(\text{break}(n), E, \text{loop}(n, s)::K) & \to \text{ctrl}(\text{next}, E, K) \\
  \text{ctrl}(\text{break}(n), E, \text{loop}(n', s)::K) & \to \text{ctrl}(\text{break}(n), E, K) & n \neq n' \\
  \text{ctrl}(\text{continue}(n), E, \text{loop}(n, s)::K) & \to \text{ctrl}(\text{next}, E, \text{loop}(n, s)::K) \\
  \text{ctrl}(\text{continue}(n), E, \text{loop}(n', s)::K) & \to \text{ctrl}(\text{break}(n), E, K) & n \neq n' \\
  \text{ctrl}(\text{ret}, E, \text{loop}(n, s)::K) &\to \text{ctrl}(\text{ret}, E, K) \\
  \text{ctrl}(\text{ret} (v_i)_i, E, \text{fn-call}(E', l_i)::K) &\to \text{ctrl}(\text{next}, E', K) \\
    & E' := (E_{\text{heap}}, E_{\text{static}}, E \\
\end{aligned}
\]


