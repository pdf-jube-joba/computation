[[Model]] [[手続き型言語]]

- 手続き
- ラベル付きループ（入れ子）
- 式
- データ構造（直積・直和・配列）
- ポインタ（関数ポインタもメモリも）
- ヒープも使えないと、他の言語を書くほどのことができない。

これが全部入ってて意味論が簡単な言語が欲しい...mini と書いたが、あまりにも大きい。

ポインタ周りを後置にしてみる。
環境は default value で全部初期化済みにしておく。
recursive なデータがほしいが、型定義でそれを書くのは辛くはないが話が長くなる。
それよりも cast/uncast みたいなことができればもっと色々使える。

## 構文

\[
\begin{aligned}
\NT{var}, \NT{label} \\
\NT{fn-name} \\
&\defeq \NT{string} \\
\\
\NT{type} &\defeq \\
  &|\T{\#num} | \T{\#u8} | \T{\#bool} | \T{\#unit} \\
  &| \T{\#ptr}| \T{\#fn} \\
  &| \NT{type} (\T{*} \NT{type})+ | \NT{type} (\T{+} \NT{type})+ | \T{\LSB} \NT{type} \T{;} \N \T{\RCB} \\
\\
\NT{place-expr} &\defeq \\
  &| \T{static} \NT{var} | \T{local} \NT{var} \\
  &| \NT{place-expr} \T{.} \N &\syntaxname{field access} \\
  &| \NT{place-expr} \T{?} \N &\syntaxname{tag access} \\
  &| \NT{place-expr} \T{\LSB} \NT{value-expr} \T{\RSB} &\syntaxname{index access} \\
  &| \NT{value-expr} \T{\#loc} &\syntaxname{dereference} \\
\\
\NT{value-expr} &\defeq \\
  &| \N | \T{'} \NT{char} \T{'} | \T{\#true} | \T{\#false} | \T{\#unit} &\syntaxname{primitive value} \\
  &| \T{\#null-ptr} | \T{\#null-fn} \\
  &| \NT{value-expr} \NT{value-expr} \NT{binop} | \NT{value-expr} \NT{unop}  &\syntaxname{binop, unop} \\
  &| \NT{value-expr} (\T{,} \NT{value-expr})+ \T{pair} \T{\LP} \N \T{\RP} &\syntaxname{pair} \\
  &| \NT{value-expr} \T{tag} \T{\LP} \N \T{\RP} &\syntaxname{tagged sum} \\
  &| \T{ld} \NT{place-expr} &\syntaxname{load from location} \\
  &| \NT{place-expr} \T{\#addr} &\syntaxname{reference} \\
  &| \T{fn} \NT{fn-name} &\syntaxname{function pointer} \\
\\
\NT{stmt} &\defeq \\
  &| \T{assign} \NT{place-expr} \T{:=} \NT{value-expr} &\syntaxname{assign} \\
  &| \T{if} \NT{value-expr} \NT{stmt} \\
  &| \T{case} \N \T{of} \NT{value-expr} \NT{stmt} \\
  &| \T{halloc} \NT{type} \T{->} \NT{place-expr} &\syntaxname{alloc returning pointer} \\
  &| \T{hfree} \NT{value-expr} \\
  &| \T{loop} \NT{label} \T{:} \NT{stmt} \\
  &| \T{break} \NT{label} \\
  &| \T{continue} \NT{label} \\
  &| \T{call} \NT{value-expr} \T{\LP} \syntaxmacro{comma-separated}{\NT{value-expr}} \T{\RP} \T{->} \syntaxmacro{comma-separated}{\NT{place-expr}} &\syntaxname{function call} \\
  &| \T{return} \syntaxmacro{comma-separated}{\NT{value-expr}} \\
  &| \T{block} \NT{block} \\
\NT{block} &\defeq \\
  &\T{\LP} \syntaxmacro{comma-separated}{\NT{var} \T{:} \NT{type}} \T{\RP} &\syntaxname{block local variable} \\
  &\T{\LCB} \syntaxmacro{semicolon-separated}{\NT{stmt}} \T{\RCB} \\
\\
\NT{fn-decl} &\defeq \\
  &\T{fn} \NT{fn-name} \NT{stmt} \\
\NT{static-decl} &\defeq \T{static} \NT{var} \T{:} \NT{type} \\
\NT{program} &\defeq \NT{static-decl}* \NT{fn-decl}* \\
\end{aligned}
\]

fn main を実行する。

## 意味論
動的型付け言語みたいになる。
location 自体を first class にもってくることでそのまま pointer みたいに使える。
引数も返り値も1つでいいのは、 pair があるから。
引数は local の一種として考えておく。

- \(L\) := \(L_{\text{key}} + L.\N + L?\N + L[\N]\)
  - \(L_{\text{key}}\) := \(L_{\text{heap}} + L_{\text{static}} + L_{\text{local}}\)
  - \(L_{\text{heap}}\): set ... heap 用の key (top level) 適当な集合でいい
  - \(L_{\text{static}}\) := \(\NT{var}\)
  - \(L_{\text{local}}\) := \(\NT{var}\)
- \(V\) := sum of
  - \(V_p\) := sum of (メモリセル一つ分)
    - \(\N + \T{'} \NT{char} \T{'} + \T{\#true} + \T{\#false} + \T{\#unit}\)
    - \(\T{\#null-ptr} + L\) ... データメモリへの"ポインタ"
    - \(\T{\#null-fn} + \NT{fn-name}\) ... 関数ラベルへの"ポインタ"
  - \(V * V + V * V * V +\) ...pair と array
    - 一緒にしない方がいい？一応、型の方で防ぐことができる。
  - \(\N \times V\) ... sum
- \(\text{type-match}(v: V, t: \NT{type}): \Bool\) := match (v, t) で以下の場合のみ true
  - \((n \in \N, t = \T{\#Number})\), \((c \in \NT{char}, \T{\#char})\), bool, unit も同じく。
  - \((\T{\#null-ptr} | L, \T{\#ptr})\), \((\T{\#null-fn} | \NT{fn-name}, T{\#fn})\)
  - \(((v_0, \ldots, v_n), (t_0 * \cdots * t_m))\) で \(n = m\), \(\text{type-match}(v_i, t_i)\)
  - \(((v_0, \ldots, v_n), [t; m])\) で \(\text{type-match}(v_i, t)\), \(n = m\)
  - \(((k \in \N, v), t_0 + \cdots t_n)\) で \(k \leq n\), \(\text{type-match}(v, t)\)
- \(E\) := \(E_{\text{heap}} * E_{\text{static}} * E_{\text{locals}}\) とする。
  - \(E_{\text{static}}\) := \(L_{\text{static}} \pfun (\NT{type}, V)\)
  - \(E_{\text{heap}}\) := \(L_{\text{heap}} \pfun (\NT{type}, V)\)
  - \(E_{\text{locals}}\) := \(\text{list}(E_{\text{scope}})\) where \(E_{\text{scope}}\) := \(L_{\text{local}} \pfun (\NT{type}, V)\)
    - alias 用として、 block を抜けたら alias 達を pop する。

まずはこんな感じで定義しておく。次に補助
- \(\text{default-value}(t: \NT{type}): V\) := \(0 \in \N \T{'\textbackslash 0'} | \T{\#false} | \T{\#null-ptr} | \T{\#null-fn}\) を型に合わせて作る。
- \(E[l := v]\) は
  1. \(l\) が \(E\) に存在し \(\text{type-match}(v, E[l].0)\) のときに限り
  2. \(E[l].1\) を \(v\) で上書きしたものとする。

評価はこんな感じ。
value や place の eval は 1step で行えるようになっている。
副作用と環境の変更が行われない。

- \(\text{eval-place}(e: E, p: \NT{place-expr}): L\) := match p with // ... この時点では valid な location か検査しなくてよい。ただし value-expr を除く。
  - static の場合：それぞれ \(L_*\) としてそのまま解釈できる。
  - local var の場合： \(E_{\text{locals}}\) にあるそれぞれの \(E_{\text{scope}}\) を頭から順に探す。
    - scope の外にある局所変数でもアクセスできるべき。 `block x: num { block y: num { assign y := ld local x } }`
  - pair の場合：\(\text{eval-place}(p).n\)
  - sum の場合： \(\text{eval-place}(p)?n\)
  - index access の場合：\(\text{eval-place}(p)[\text{eval-value}( e )]\)
  - dereference の場合：\(\text{eval-value}(v) \in L\) なら \(L\)
- \(\text{eval-value}(e: E, v: \NT{value-expr}): V\) := match v with
  - primitive value と null ptr/ null fn はそのまま \(V\) に
  - binop, unop, equality は \(V\) に評価してそれらに適用
  - pair は \((v_1, \ldots, v_n)\) に変換する。
  - tag は \((n, v)\) に変換する
  - ld は \(\text{eval-place}\) で得た \(L\) に対して、 \(E\) から value を取り出す。
  - reference は \(L \subset V\) にする。
  - fn はそのまま \(\NT{fn-name} \subset V\) にする。

ここまでは \(E\) が全然変化しないし、あまり制御部分を気にする必要もない。これ以降は stmt に関数呼び出しもラベル付きループもあるので、それだけ気を付ける。
closure がないので、環境がそのまま引数スタックみたいになっている。
明確な返り値がないときは unit が返ってくるとする。
てか無理やりやらなくても、 program counter 方式で十分な気がする。
関数内の制御と関数間の制御を分けている。

- \(R\) :=
  - \(\text{next}\)
  - \(\text{break} \NT{label}\)
  - \(\text{continue} \NT{label}\)
- \(K\) := \(K_{\text{inner}} * K_{\text{fn}}\) :=
  - \(K_{\text{inner}}\) :=
    - \(\text{loop}(\NT{label}, \NT{stmt})\) ... loop 用
      - スタックを再度拡張する。
    - \(\text{scope}\) ... block 用
    - \(\NT{stmt}\) ... 普通の stmt の継続
  - \(K_{\text{fn}}\) :=
    - \(\text{fn-call}(E_{\text{local}}, (l_i: L))\) ... 関数呼び出し用：抜けるときに環境を recover しつつ、値を代入する。
- \(S\) := \(\text{eval}(\NT{stmt}, E, K) + \text{ctrl}(R, E, K) + \text{ret}((v_i), E, K)\)

\[
\begin{aligned}
  \text{eval}(\T{assign} p e, E, K) &\to \text{ctrl}(\text{next}, E', K) \\
    & l: L := \text{eval-place}(p, E) \\
    & v: V := \text{eval-value}(e, E) \\
    & E' := E[l := v], \\
  \text{eval}(\T{if} e s, E, K) &\to \text{ctrl}(\text{next}, E, K) &&\text{eval-value}(e, E) = \T{\#false} \\
  \text{eval}(\T{if} e s, E, K) &\to \text{eval}(s, E, K) && \text{eval-value}(e, E) = \T{\#true}\\
  \text{eval}(\T{case} n e s, E, K) &\to \text{ctrl}(\text{next}, E, K) &&\text{eval-value}(e, E) = (m \neq n, v) \\
  \text{eval}(\T{case} n e s, E, K) &\to \text{eval}(s, E, K) && \text{eval-value}(e, E) = (n, v)\\
  \text{eval}(\T{halloc} t p, E_{\text{heap}}, E_{\text{static}}, E_{\text{local}}, K) &\to \text{ctrl}(\text{next}, E'_{\text{heap}}, E_{\text{static}}, E_{\text{local}}, K) \\
    & k: L = \text{new}(E_{\text{heap}}) \\
    & l: L = \text{eval-place}(p, E) \\
    & E'_{\text{heap}} := E_{\text{heap}}[k := (t, \text{default-value}(t)][l := k] \\
  \text{eval}(\T{hfree} e, E_{\text{heap}}, E_{\text{static}}, E_{\text{local}}, K) &\to \text{ctrl}(\text{next}, E'_{\text{heap}}, E_{\text{static}}, E_{\text{local}}, K) \\
    & l: L := \text{eval-value}(e, E) \\
    & E'_{\text{heap}} := E_{\text{heap}} \backslash l \\
  \text{eval}(\T{loop} n s, E, K) &\to \text{ctrl}(\text{next}, E, \text{loop}(n, s)::K) \\
  \text{eval}(\T{break} n, E, K) &\to \text{ctrl}(\text{break}(n), E, K) \\
  \text{eval}(\T{continue} n, E, K) &\to \text{ctrl}(\text{continue}(n), E, K) \\
  \text{eval}(\T{call} e (e_i)_i (p_i)_i, E_{\text{heap}}, E_{\text{static}}, E_{\text{local}}, K_{\text{inner}}, K_{\text{fn}}) &\to \text{eval}(s, E_{\text{heap}}, E_{\text{static}}, [E'_{\text{arg}}], K') \\
    & \T{fn} n := \text{eval-value}(e, E) \\
    & (x_i: t_i), s := \text{coressponding program of } n \\
    & v_i: V := \text{eval-value}(e_i) \\
    & l_i: L := \text{eval-place}(p_i, E) \\
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

