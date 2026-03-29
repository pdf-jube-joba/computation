[[Model]]

- `new` があって object identity の話ができる。
- `this` 付き

## 構文
なんかラムダ計算みたいなのがある。
method と variable を区別してない。
primitive value はオブジェクトじゃない。

\[
\begin{aligned}
\NT{var} &\defeq \NT{string} \\
\NT{f} &\defeq \NT{string} \\
\\
\NT{expr} &\defeq \\
    &| \T{\#this} | \T{\#new} \\
    &| \N | \T{\#true} | \T{\#false} | \T{\#unit} \\
    &| \NT{var} | \NT{expr} \T{.} \NT{f} \\
    &| \NT{expr} \NT{binop} \NT{expr} \\
    &| \NT{expr} \T{===} \NT{expr} & \syntaxname{equality of identity} \\
    &| \T{fun} \NT{var} \T{=>} \NT{expr} \\
    &| \NT{expr} \T{\LP} \NT{expr} \T{\RP} \\
    &| \T{set-this} \NT{expr} \T{in} \T{\LCB} \NT{stmt} \T{;} \NT{expr} \T{\RCB} \\
\\
\NT{stmt} &\defeq \\
    &| \T{skip} \\
    &| \NT{stmt} \T{;} \NT{stmt} \\
    &| \T{let} \NT{var} \T{:=} \NT{expr} &\syntaxname{syntax suger of fun} \\
    &| \NT{expr} \T{.} \NT{f} \T{:=} \NT{expr} \\
    &| \T{if} \NT{expr} \T{then} \NT{stmt} \T{end} \\
    &| \T{while} \NT{expr} \T{do} \NT{stmt} \T{end} \\
\\
\NT{program} &\defeq \NT{expr} \\
\end{aligned}
\]

## 意味論

ふんわり考える
- object store というものがあってこれは KV である。
  - object identity は何かしらの key 値と思える。
- 代入関連は、右辺が object なら K を入れて、そうじゃないならそのまま primitive を入れる
- それをもとにすると、 V は primitive + K + で書ける。
- 難しいのが closure の評価と this の評価。
  - stmt や expr を実行している"主体"みたいなものがあるはずなので、
    状態として object key をスタックに積んで置き、現在の focus しているオブジェクトとして考える。
- ある意味で、 environment が object になる。ただし、 identity 付き
> [!note]
> **古い記録**
> - object-decl 中の変数と "this" の評価をどうするかが難しい。
>  - `{ x: 1, m: new { y: x, z: this } }`
>  - `{ x: 1, y: this.x }`
> - stmt と object-decl を消して、 `M.f := N` と `new` だけあればいいようにした。これで、 `new {this.x := 1; this.y := 2; }` が書けて、代わりになる。
>  なお、 `N` の評価結果が return されることにする。なので、 `new {this}` で新たに生成されたオブジェクトが返る。
> - new の中で環境は同じものを使うが、戻った時には回復する `let x := 1 in new { this.y = x; let x := 2 in this}; x` は `this.x` に `1` を入れつつ `new` の外に戻ってきたら `x = 1` になっていてほしい。
> - equality については、ラムダ計算とは違うので、 closure が入ったら比較を拒否することにする。
> AI と議論していて気が付いたこと： object identity を含むと参照透過性が壊れる。
> `let x = e in (x === x)` と `e === e` は異なる結果になりうる。
> Ocaml でも似たようなことは起きていて、 ref 周りがそうっぽい。

やっぱり stmt は別にした方がわかりやすい。

以降はちゃんと練る
- \(I\) : set ... object identity 用で、なんでもいい
- \(V_p\) := \(\N + \mathbb{B} + \T{\#null}\)
- \(V\) := \(V_p + \text{cls}(x, M, E) + I\)
- \(E\) := \(\typeparam{list}{(\NT{var}, V)}\)
- \(S\) := \(I \pfun \typeparam{list}{(\NT{f}, V)}\)
- \(K\) := \(\text{list}K_f\) where K_f :=
    - \([] \NT{binop} N, E\)
    - \(V \NT{binop} [], E\)
    - \([] N, E\)
    - \(\text{cls}(x, M, E) []\)
    - \([] === N, E\)
    - \(V === []\)
    - \(([]; s, E)\)
    - \([] . f := M, E\)
    - \(v. f := []\)
    - \(\T{if} [] M N, E\)
- \(K\) := \(\typeparam{list}{F}\)

\[
\begin{aligned}
\text{eval-expr}(\T{this}, i, E, S, K) &\to \text{return}(i, i, S, K) \\
\text{eval-expr}(\T{new} e, i, E, S, K) &\to \text{eval-expr}(e, i', E, S', \text{new}(i)::K) \\
    & i': I := \text{new}(S), S' := S[i' := []] \\
\text{eval-expr}(v \in V_p, i, E, S, K) &\to \text{return}(v, i, S, K) \\
\text{eval-expr}(x \in \NT{var}, i, E, S, K) &\to \text{return}(v, i, S, K) \\
    & v = \text{lookup}(E, x) \\
\text{eval-expr}(e.f, i, E, S, K) &\to \text{eval-expr}(e, i, E, S, (\text{Acc(f)})::K) \\
\text{eval-expr}(M (N), i, E, S, K) &\to \text{eval-expr}(M, i, E, S, ([] N, E)::K) \\
\text{eval-expr}(\T{fun} x \T{=>} M, i, E, S, K) &\to \text{return}(\text{mkcls}(x, M, E), i, S, K) \\
\text{eval-expr}(M (b \in \NT{binop}) N, i, E, S, K) &\to \text{eval-expr}(M, i, E, S, ([] b N, E)::K) \\
\text{eval-expr}(M === N, i, E, S, K) &\to \text{eval-expr}(M, i, E, S, ([] === N, E)::K) \\
\\
\text{eval-expr}(e_1 \T{;} e_2, i, E, S, K) &\to \text{eval-expr}(e_1, i, E, S, ([]; e_2, E)::K) \\
\text{eval-expr}(e_1. f := e_2, i, E, S, K) &\to \text{eval-expr}(e_1, i, E, S, ([]. f := e_2, E)::K) \\
\text{eval-expr}(\T{if} e M N, i, E, S, K) &\to \text{eval-expr}(e, i, E, S, (\T{if} [] M N, E)::K) \\
\text{eval-expr}(\T{while} e M, i, E, S, K) &\to \text{eval-expr}(\T{if} e (M; \T{while} e M) \T{\#null}, i, E, S, K) \\
\\
\text{return}(v, i, S, \text{new}(i'):: K) &\to \text{return}(v, i', S, K) \\
\text{return}((i \in I), i', S, \text{Acc}(f)::K) &\to \text{return}(v, i', S, K) \\
    & v = \text{lookup}(S(i), f) \\
\text{return}(\text{cls}(x, M, E), i, S, ([] N, E')::K) &\to \text{eval-expr}(N, i, E', S, (\text{cls}(x, M, E)[])::K) \\
\text{return}(v, i, S, \text{cls}(x, M, E)[]::K) &\to \text{eval-expr}(M, i, E[x := v], S, K) \\
\text{return}(v, i, S, ([] \mathrel{b} N, E)::K) &\to \text{eval-expr}(N, i, E, S,(v \mathrel{b} [])::K) \\
\text{return}(v, i, S, (v' \mathrel{b} [])::K) &\to \text{return}(v'', i, S, K) \\
  &v' = v \mathrel{b} v' \\
\text{return}(v, i, S, ([] === N, E)::K) &\to \text{eval-expr}(N, i, E, S, (v === [])::K) \\
\text{return}(v, i, S, (v' === [])::K) &\to \text{return}(t, i, S, K) \\
  &t = (v \in I) == (v' \in in I) \\
\text{return}(v, i, S, ([]; s, E)::K) &\to \text{eval-expr}(s, i, E, S, K) \\
\text{return}(v, i, S, ([]. f := M, E)::K) &\to \text{eval-expr}(M, i, E, S, (v.f := [])::K) \\
\text{return}(v, i, S, (v'.f := [])::K) &\to \text{return}(\T{\#null}, i, S', K) \\
  &S' := S[v' \mapsto E'], E' := S(v')[f := v] \\
\text{return}(\T{\#true}, i, S, (\T{if}[] M N, E)::K) &\to \text{eval-expr}(M, i, E, S, K) \\
\text{return}(\T{\#false}, i, S, (\T{if}[] M N, E)::K) &\to \text{eval-expr}(N, i, E, S, K) \\
\\
\end{aligned}
\]

それ以外の場合には stuck する。
