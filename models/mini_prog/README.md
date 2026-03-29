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

> [!note]
> 再帰的なデータ型がほしいが、型定義でそれを書くのは辛くはないが話が長くなる。
> それよりも cast/uncast みたいなことができればもっと色々使える。

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
\NT{fn-decl} &\defeq \T{fn} \NT{fn-name} \NT{block} \\
\NT{static-decl} &\defeq \T{static} \NT{var} \T{:} \NT{type} \\
\NT{program} &\defeq \NT{static-decl}* \NT{fn-decl}* \\
\end{aligned}
\]

- binop は `+`, `-`, `==`, `<`, `&&`, `||`
- unop は `!`

## 意味論
動的型付け言語みたいになる。
location 自体を first class にもってくることでそのまま pointer みたいに使える。
引数も返り値も1つでいいのは、 pair があるから。
引数は local の一種として考えておく。

- \(L\) := \(L_{\text{key}} + L.\N + L?\N + L[\N]\)
  - \(L_{\text{key}}\) := \(L_{\text{heap}} + L_{\text{static}} + L_{\text{local}}\)
  - \(L_{\text{heap}}\): set ... heap 用の key (top level) 適当な集合でいい
  - \(L_{\text{static}}\) := \(\NT{var}\)
  - \(L_{\text{local}}\): set ...stack 用の key で適当な集合でいい。直接 var と結び付けず、間に挟む。
    - 関数呼び出しと block 内局所変数の2つと offset みたいな実装になる？
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
- \(S\) := \(S_{\text{heap}} * S_{\text{static}} * S_{\text{locals}}\) とする。
  - \(S_{\text{static}}\) := \(L_{\text{static}} \pfun (\NT{type}, V)\)
  - \(S_{\text{heap}}\) := \(L_{\text{heap}} \pfun (\NT{type}, V)\)
  - \(S_{\text{locals}}\) := \(\texttt{list}(\texttt{list}(S_{\text{scope}}))\) where \(S_{\text{scope}}\) := \(L_{\text{local}} \pfun (\NT{type}, V)\)
    - list of list になっているのは、関数ごとが list で、ブロックごとが list in list になるから。
      だから、 block の入出では head の list を操作して、関数の入出ではそれ自身を pop/push する。
- \(E\) := \(\texttt{list}(E_{\text{scope}})\) where \(E_{\text{scope}}\) := \(\NT{var} \to L_{\text{local}}\)
  - 変数と location の結びつきは関数間では共有されないので、これは scope だけ管理するため list でいい。

まずはこんな感じで定義しておく。次に補助
- \(\text{default-value}(t: \NT{type}): V\) := \(0 \in \N \T{'\textbackslash 0'} | \T{\#false} | \T{\#null-ptr} | \T{\#null-fn}\) を型に合わせて作る。
- \(S[l := v]\) は \(l\) の指す場所を上書きする（ persistent update ）
  1. \(l\) が \(S\) に存在し \(\text{type-match}(v, S[l].0)\) のときに限り
  2. \(S[l].1\) を \(v\) で上書きしたものとする。
- \(\text{push-scope}(E, S, (x_i: \NT{var}, t_i: \NT{type}, v_i: V)_i): (E, S)\) := ... scope を新たに作り出す。
  - まず、 \(\text{type-match}(v_i, t_i)\) かを確認する。
  - \(l_i: L\) := \(\texttt{new}(S_{\text{locals}})\) で新しい frame 用の locations を用意する。
  - \(S_f\): \(L \to V\) := \(l_i \mapsto (t_i, v_i)\) ... scope を新たに用意する。
  - \(S_{\text{local}}\):= \(\texttt{head}(S_{\text{locals}})\)
  - \(S'_{\text{local}}\) := \(S_f::S_{\text{local}}\)
  - \(E_f\): \(\NT{var} \to L\) := \(x_i \mapsto l_i\)
  - 結果は \(( E_f::E, (S_{\text{heap}}, S_{\text{static}}, S'_{\text{local}}::\texttt{tail}(S_{\text{locals}})) )\)
- \(\text{pop-scope}(E, S): (E, S)\) :=
  - \(E\) から pop
  - \(S_{\text{local}}\) の head の list から pop

評価はこんな感じ。
value や place の eval は 1step で行えるようになっている。
副作用と環境の変更が行われない。

- \(\text{eval-place}(e: E, s: S, p: \NT{place-expr}): L\) := match p with // ... この時点では valid な location か検査しなくてよい。ただし value-expr を除く。
  - static の場合：\(L_{\text{static}}\) としてそのまま解釈できる。
  - local var の場合： \(E(x)\) とする。
    - scope の外にある局所変数でもアクセスできるべき。例えば：
      ```
      block (x: num) { block (y: num) { assign y := ld local x } }
      ```
      このため、 \(E\) という list の全部を head から順に存在するか確認すること。
  - pair の場合：\(\text{eval-place}(p).n\)
  - sum の場合： \(\text{eval-place}(p) = L?n\)
  - index access の場合：\(\text{eval-place}(p)[\text{eval-value}( e )]\)
  - dereference の場合：\(\text{eval-value}(v) \in L\) なら \(L\)
- \(\text{eval-value}(e: E, s: S, v: \NT{value-expr}): V\) := match v with
  - primitive value と null ptr / null fn はそのまま \(V\) に
  - binop, unop, equality は \(V\) に評価してそれらに適用
  - pair は \((v_1, \ldots, v_n)\) に変換する。
  - tag は \((n, v)\) に変換する
  - ld は \(\text{eval-place}\) で得た \(L\) に対して、 \(S\) から value を取り出す。
  - reference は \(L \subset V\) にする。
  - fn はそのまま \(\NT{fn-name} \subset V\) にする。

ここまでは \(E\) が全然変化しないし、あまり制御部分を気にする必要もない。これ以降は stmt に関数呼び出しもラベル付きループもあるので、それだけ気を付ける。
closure がないので、環境がそのまま引数スタックみたいになっている。
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
    - \((E_{\text{local}}, (l_i: L)_i, K_{\text{inner}})\) ... 関数呼び出し用：抜けるときに環境を recover しつつ、値を代入する。
- \(\text{State}\) := \(\text{eval}(\NT{stmt}, E, S, K) + \text{ctrl}(R, E, S, K) + \text{ret}((v_i), E, S, K)\)

制御以外は簡単に next に移る。
\[
\begin{aligned}
  \text{eval}(\T{assign} p e, E, S, K) &\to \text{ctrl}(\text{next}, E', S, K) \\
    & l: L := \text{eval-place}(p, E, S) \\
    & v: V := \text{eval-value}(e, E, S) \\
    & E' := E[l := v], \\
  \text{eval}(\T{if} e s, E, S, K) &\to \text{ctrl}(\text{next}, E, S, K) &&\text{eval-value}(e, E) = \T{\#false} \\
  \text{eval}(\T{if} e s, E, S, K) &\to \text{eval}(s, E, S, K) && \text{eval-value}(e, E) = \T{\#true}\\
  \text{eval}(\T{case} n e s, E, S, K) &\to \text{ctrl}(\text{next}, E, S, K) &&\text{eval-value}(e, E) = (m \neq n, v) \\
  \text{eval}(\T{case} n e s, E, S, K) &\to \text{eval}(s, E, S, K) && \text{eval-value}(e, E) = (n, v)\\
\\
  \text{eval}(\T{halloc} t p, E, S_{\text{heap}}, S_{\text{static}}, S_{\text{locals}}, K) &\to \text{ctrl}(\text{next}, E,S'_{\text{heap}}, S_{\text{static}}, S_{\text{locals}}, K) \\
    & k: L_{\text{heap}} := \text{new}(S_{\text{heap}}) \\
    & l: L := \text{eval-place}(p, E, S) \\
    & S'_{\text{heap}} := (k \mapsto (t, \text{default-value}(t)))::S_{\text{heap}} \\
    & S_1 := (S'_{\text{heap}}, S_{\text{static}}, S_{\text{locals}}) \\
    & S' := S_1[l := k] \\
  \text{eval}(\T{hfree} e, E, S_{\text{heap}}, S_{\text{static}}, S_{\text{locals}}, K) &\to \text{ctrl}(\text{next}, E, S'_{\text{heap}}, S_{\text{static}}, S_{\text{local}}, K) \\
    & l: L_{\text{heap}} := \text{eval-value}(e, E, S) \\
    & S'_{\text{heap}} := S_{\text{heap}} \backslash l \\
\end{aligned}
\]

inner な制御の部分
\[
\begin{aligned}
  \text{eval}(\T{loop} n s, E, S, K_{\text{inner}}, K_{\text{fn}}) &\to \text{ctrl}(\text{next}, E, S, \text{loop}(n, s):: K_{\text{inner}}, K_{\text{fn}}) \\
  \text{eval}(\T{break} n, E, S, K) &\to \text{ctrl}(\text{break}(n), E, S, K) \\
  \text{eval}(\T{continue} n, E, S, K) &\to \text{ctrl}(\text{continue}(n), E, S, K) \\
\\
  \text{eval}(\T{block} (x_i: t_i)_i (s_i)_i, E, S, K_{\text{inner}}, K_{\text{fn}}) &\to \text{ctrl}(\text{next}, E, S', K'_{\text{inner}}, K_{\text{fn}}) \\
    & v_i: V := \text{default-value}(t_i) \\
    & (E', S') := \text{push-scope}(E, S, (x_i, t_i, v_i)) \\
    &K'_{\text{inner}} := (s_i)_i {++} (\text{scope} ::K_{\text{inner}}) \\
\\
  \text{ctrl}(r, E, S, \text{scope}::K_{\text{inner}}, K_{\text{fn}}) &\to \text{ctrl}(r, E', S', K_{\text{inner}}, K_{\text{fn}}) \\
    & E' := \texttt{tail}(E) \\
    & S' := \texttt{tail}(\texttt{head}(S)):: \texttt{tail}(S) \\
  \text{ctrl}(\text{next}, E, S, (s \in \NT{stmt})::K_{\text{inner}}, K_{\text{fn}}) &\to \text{eval}(s, E, S, K_{\text{fn}}) \\
  \text{ctrl}(r, E, S, \NT{stmt}::K) &\to \text{ctrl}(r, E, S, K) && r = \text{break} | \text{continue} \\
  \text{ctrl}(\text{next}, E, S, \text{loop}(n, s)::K, K') &\to \text{eval}(s, E, S, \text{loop}(n, s)::K, K') \\
  \text{ctrl}(\text{break}(n), E, S, \text{loop}(n, s)::K, K') & \to \text{ctrl}(\text{next}, E, S, K, K') \\
  \text{ctrl}(\text{break}(n), E, S, \text{loop}(n', s)::K, K') & \to \text{ctrl}(\text{break}(n), E, S, K, K') && n \neq n' \\
  \text{ctrl}(\text{continue}(n), E, S, \text{loop}(n, s)::K, K') & \to \text{ctrl}(\text{next}, E, S, \text{loop}(n, s)::K, K') \\
  \text{ctrl}(\text{continue}(n), E, S, \text{loop}(n', s)::K, K') & \to \text{ctrl}(\text{break}(n), E, S, K, K') && n \neq n' \\
\end{aligned}
\]

関数呼び出しの場合
\[
\begin{aligned}
\\
  \text{eval}(\T{call} e (e_i)_i (p_i)_i, E, S, K) &\to \text{ctrl}(\text{next}, E', S', K') \\
    & r_i: L := \text{eval-place}(p_i, E) \\
    & K'_{\text{fn}} := \text{fn-call}(E, (r_i)_i, K_{\text{inner}})::K_{\text{fn}} \\
    & \T{fn} n := \text{eval-value}(e, E, S) \\
    & (x_i: t_i), (s_i)_i := \text{coressponding program of } n \\
    & v_i: V := \text{eval-value}(e_i) \\
    & (E', S') := \text{push-scope}([], []::S, (x_i, t_i, v_i)_i) \\
    & K'_{\text{local}} := (s_i)_i, K' := (K'_{\text{local}}, K'_{\text{fn}}) \\
  \text{eval}(\T{return} (e_i)_i, E, S, K) &\to \text{ret}((v_i)_i, E, S, K) \\
    & v_i := \text{eval-value}(e_i, E) \\
\\
  \text{ret}((v_i)_i, E, S, K_{\text{inner}},  K_{\text{fn}}) &\to \text{ctrl}(\text{next}, E', S', K'_{\text{inner}}, K'_{\text{fn}}) \\
    & S_1 := \texttt{tail}(S) \\
    & (E', l_i, K'_{\text{inner}}) = \texttt{head}(K_{\text{fn}}) \\
    &S' := S_1[l_i := v_i] \\
    & K'_{\text{fn}} := \texttt{tail}(K_{\text{fn}}) \\
\end{aligned}
\]

このもとで、
- `AInput`, `FOutpu` は \(\N\) の tuple とする。
- static は t の default-value で初期化する。
- `main` の名前のついている関数を `AInput` を引数に呼び出す。ただし、 \(K_{\text{fn}}\) にはなにも積まずに \([]\) で呼び出す。
- 最終的に \(\text{ret}(n \in \N, E, S, [])\) となったとき （ `main` の return 結果）
- 以上の部分で"存在しない"とみなせる部分は全部 `Err` とする。
  - 遷移状態に列挙されていないものは `Err` になる。
    - 明確な返り値がないときは `Err` が返ってくるなど。
  - Store にない location を使った場合は `Err`
  - if は bool のみ、 case は tag のみ、それ以外が来たら `Err`
