直和と直積と配列のある言語を作る。
一応注意点として、**メモリの扱い方** としての言語を考えるので、
値を渡さないでメモリへの代入をわかりやすくするというコンセプトで行く。
また、変数については最初から場所としておく。
（仮想レジスタも式もないので、一時変数には毎回メモリを割り当てていることになるが、
まあここでやりたいのはそれを回避することではないので。）

\(\begin{aligned}
\NT{var}    &\defeq \NT{string} \\

\NT{prim-value}    &\defeq \NT{imm} | \T{\#true} | \T{\#false} \\
\NT{place-expr}   &\defeq \NT{var} | \NT{place-expr} \T{.l} | \NT{place-expr} \T{.r} | \NT{place-expr} \T{\LSB} \NT{value-expr} \T{\RSB} \\
\NT{value-expr}   &\defeq \NT{prim-value} | \T{ld} \NT{place-expr} | \T{inl} \NT{value-expr} | \T{inr} \NT{value-expr} \\

\NT{stmt}   &\defeq \\
  &| \NT{place-expr} \T{:=} \NT{value-expr} \\
  &| \T{inl} \NT{place-expr} \T{=>} \NT{var} \T{\LCB} \NT{stmts} \T{\RCB} \\
  &| \T{inr} \NT{place-expr} \T{=>} \NT{var} \T{\LCB} \NT{stmts} \T{\RCB} \\
  &| \T{if} \NT{value-expr} \T{\LCB} \NT{stmts} \T{\RCB} \\
  &| \T{while} \NT{value-expr} \T{LCB} \NT{stmts} \T{\RCB} \\

\NT{stmts} &\defeq \syntaxmacro{semicolon-separated}{\NT{stmt}} \\

\NT{type} &\defeq \T{Num} | \T{Bool}\\
  &| \T{pair} \NT{type} \NT{type} | \T{sum} \NT{type} \NT{type} \\
  &| \T{arr} \NT{type} \NT{imm} \\
  &| \T{\LP} \NT{type} \T{\RP} \\

\NT{program} &\defeq \NT{static} \syntaxmacro{comma-separated}{\NT{type} \NT{var}} \T{;} \NT{stmts}
\end{aligned}\)

inl と inr は place を受け取った後に in-place に"中身へのアクセス"を変数経由で受け取るとする。
なので、 `p := inl(1); inl p => x { x := 2 };` を書くと `p := inl(2)` になっている。

> [!warning]
> 中に入った後では `p` への書き換えは `x` への書き換えになっていることに注意する。
> `p := inl(1); inp => x { p := inr(2); x := 1 }` は明らかに不正。
> なので、これを入れた時点で alias の問題が発生している？
> これを考えると、 `alias <var> := <place-expr>` とかを入れてもやらなければいけないことは変わらない。
> 変数使えなくなったのではなくて、変数は Key に結びついていて、 Valid な Key の全体が変化したと考えたほうが楽？
> オブジェクト指向でも、親が書き変わって子オブジェクトへの名前が使えなくなるとかはあるし。
> その点で、意味論側ではよくない操作をしたらただちに `Err` を吐くようにする必要がある。

コンパイルのことを考えると、変数がどれぐらいセルを使うかを計算するために、実行前にサイズがわかってないといけない。
例えば、 `x := inl(1)` に対して、 `x := inr(?)` が一度も実行されなかったときにはどうなっているのか？
それを確定させておくためにも型の anotation が必要になる。

## 意味論

とりあえず値はこんな感じ。
- \(V_p := \N + \T{\#true} + \T{\#false}\)
- \(V := V_p + (V * V) + (V + V) + [V; \N]\)

`ld` を連結した領域に対して行えるので、 \(V_p\) じゃなくて \(V\) の転送ができる。
`x.l := 1; x.r := 2; y := ld x;`

`inl p => x { x := 1;} ` が書けるようになった時点で、 variable は他の variable と別の領域を確保しているとは限らない。
そのため、 variable がそのまま location にはならないから、別に location を用意する必要がある。
variable -> location が言語の意味論上の束縛を表していて、実際のメモリは location -> value と思えばいい。
（ location の中に値が入っているというよりも、 location という Key をもとにした key value store と思う。）

- \(L := \NT{var} + \text{Left}(L) + \text{Right}(L) + \text{Idx}(L, \N) + \text{Inl}(L) + \text{Inr}(L)\) ... key
  - `(x.l).r` は \(\text{Right}(\text{Left}(x))\) に解釈されるので内部と外部っぽいものが入れ替わっていることに注意。

後はメモリは \(L \pfun V\) と思えばいい。
- 全ての Key が valid ではないこれは \(\text{Inl}, \text{Inr}\) を入れなくても同じ）。
  なので Key 自体には valid を考えずに Key から value を引いてくるときに、存在しない場合は `Err` になる、という風に思った方がいい。
- \(L(k) \in V * V\) だったとき、これをメモリの移動の単位として扱えるようにしつつ、分解ができないといけない。
  例えば、 `x := y` で `M[x]` も `M[y]` も \(V * V\) に入っている状況なら、
  \(M[y]\) という複数の"セル"の内容を \(M[x]\) に移すという形で、そのまま扱えてほしい。
  一方で、 `y.l := x` みたいな状況では \(M[\text{Left}(y)]\) への書き込みなので \(M[y]\) の内容にも変更があるべき。

これを考えると value は \(e: L \pfun V\) であって次のような条件を満たしているものと思える。
（写像全体はそれ自体が自由性を持っているので、書き込みについての条件を課すのはここではない。）
- \(\text{Left}(k)\) か \(\text{Right}(k)\) のどちらかが domain に入ればもう片方も domain に入り、かつ \(k\) 自身も domain に入り：
  - \(e(k) = ( e(\text{Left}(k)), e(\text{Right}(k)) )\)
- \(\text{Idx}(k, 0)\) が domain に入るならある \(n\) が存在して \(\text{Idx}(k, 0 \leq i < n)\) がちょうど \(i\) が domain に入る全体になっていて：
  - \(e(k) = [ e(\text{Idx}(k, 0) \ldots e(\text{Idx}(k, n))) ]\)
- \(\text{Inl}(k)\) が domain に入るなら \(\text{Inr}(k)\) は domain に入らず、
  - \(e(k) = (0, e(\text{Inl}(k)))\)

書き込み条件：\(L\) には親子関係で（全ではない）順序が入るが、
順序関係のない \(k, k'\) に対しては \(k\) への書き込みは \(k'\) のメモリに影響を与えないようにしたい。
そのためには \(e[k := v]\) を頑張って定義すればいい。（関係ある部分全部を書き換えて、それ以外を書き換えなければいい。）
注意点としてどんな代入も許されるわけではないし、 \(\text{Inl}\) の問題を考えると \(e\) 自体が型の情報を持っておかないといけない。
（ \(e[\text{Inl}(k) := 1]\), \(e[\text{Inr}(k) := 1]\), \(e[\text{Inl}(k) := \#true]\) をはじく必要がある。）

実装上は \(e: L \pfun V\) なんて使っていられないので、 \(L\) の木の構造みたいなものを使う。
- \(M\) :=
  - \(\text{cell-n}(\N) + \text{cell-b}(\mathbb{B}) + \) ... これは cell
  - \(\text{prod}(\NT{type}, \NT{type}, M, M) +\) ... left type, right type, left mem, right mem
  - \(\text{pair}(\NT{type}, \NT{type}, \{0, 1\}, M) +\) ... left type, right type, tag(0 left or 1 right), mem
  - \(\text{arr}(\NT{type}, \N, M) \) ... type of elem, max idx
- \(M_{\text{top}} := \NT{var} \to M\)
