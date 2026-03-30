[[Model]]
mini_prog はいろいろ詰め込んでいてよくわからないので、ここでは関数呼び出しとポインタだけの解決を図る。

## 構文

\[
\begin{aligned}
\NT{var}, \NT{fn-name} \\
&\defeq \NT{string} \\
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
  &\T{\LP} \syntaxmacro{comma-separated}{\NT{var}} \T{\RP} &\syntaxname{block local variable} \\
  &\T{\LCB} \syntaxmacro{semicolon-separated}{\NT{stmt}} \T{\RCB} \\ \\
\NT{program} &\defeq  \NT{fn-decl}* \\
\end{aligned}
\]

fn main を実行する。

## 意味論
location ~ pointer みたいに使える。
local の導入方法は関数しかない。

- \(L\): set ... location 用でなんでもいい。（例えば、 frameId * offset とか。）
- \(V\) := \(\N + L + \T{\#null-ptr}\)
- \(E\) := \(\NT{var} \pfun L\)
- \(S\) := \(\texttt{list}(L \to V)\) ... store だが、関数呼び出し前後で push/pop する。（ call stack に対応）
- \(S[l := v]\) := \(l\) が \(S\) に存在する場合に限り \(S[l]\) を \(v\) で上書きしたものとする。

評価はこんな感じ。
value や place の eval は 1step で行えるようになっている。
副作用と環境の変更が行われない。

- \(\text{eval-place}(e: E, s: S, p: \NT{place-expr}): L\) := match p with // ... この時点では valid な location か検査しなくてよい。ただし value-expr を除く。
  - var の場合： \(E(x)\) 
  - dereference の場合：\(\text{eval-value}(v) \in L\) なら \(L\)
- \(\text{eval-value}(e: E, s: S, v: \NT{value-expr}): V\) := match v with
  - \(\N\) ならそのまま
  - binop は \(V\) に評価してそれらに適用
  - ld は \(\text{eval-place}\) で得た \(L\) に対して、 \(S\) から value を取り出す。
  - reference は \(L \subset V\) にする。
- \(K\) := \(K_{\text{inner}} * K_{\text{fn}}\) :=
  - \(K_{\text{inner}}\) := \(\texttt{list}\NT{stmt}\) ... 普通の stmt の継続
  - \(K_{\text{fn}}\) := \(\texttt{list}(E, K_{\text{inner}})\) ... 関数呼び出し用：抜けるときに環境を recover する。
- \(S\) := \(\text{eval}(\NT{stmt}, E, S, K) + \text{next}(E, S, K)\)

\[
\begin{aligned}
  \text{eval}(\T{assign} p e, E, S, K) &\to \text{next}(E, S', K) \\
    & l: L := \text{eval-place}(E, S, p) \\
    & v: V := \text{eval-value}(E, S, e) \\
    & S' := S[l := v], \\
  \text{eval}(\T{ifz} e s_1 s_2, E, S, K_{\text{inner}}, K_{\text{fn}}) &\to \text{next}(E, S, s_1 :: K_{\text{inner}}, K_{\text{fn}}) && \text{eval-value}(E, S, e) = 0 \\
  \text{eval}(\T{ifz} e s_1 s_2, E, S, K_{\text{inner}}, K_{\text{fn}}) &\to \text{next}(E, S, s_2 :: K_{\text{inner}}, K_{\text{fn}}) && \text{eval-value}(E, S, e) \neq 0\\
  \text{eval}(\T{call} n (e_i)_i, E, S, K_{\text{inner}}, K_{\text{fn}}) &\to \text{next}(E', S', (s_i), (E, K_{\text{inner}})::K) \\
    & (x_i)_i , (s_i) := \text{coressponding program of } n \\
    & v_i: V := \text{eval-value}(E, S, e_i) \\
    & l_i: L := \texttt{new}(S) \\
    & S_f: L \pfun V := (l_i \mapsto v_i) \\
    & S' := S_f ::S \\
    & E' := [x_i \mapsto l_i] \\
  \text{eval}(\T{return}, E, S, K_{\text{inner}}, K_{\text{fn}}) &\to \text{next}(E', S', K'_{\text{inner}}, K'_{\text{fn}}) \\
    & S' := \texttt{tail}(S) \\
    & (E', K'_{\text{inner}})::K'_{\text{fn}} := K \\
\\
  \text{next}(E, S, K_{\text{inner}}, K_{\text{fn}}) &\to \text{eval}(s, E, S, K'_{\text{inner}}, K_{\text{fn}}) \\
    & s := \texttt{head}(K_{\text{inner}}) \\
    & K'_{\text{inner}} := \texttt{tail}(K_{\text{inner}}) \\
\end{aligned}
\]


