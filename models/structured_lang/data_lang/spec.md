[[Model]] [[構造化言語]]

直和と直積と配列のある言語を作る。
一応注意点として、**メモリの扱い方** としての言語を考えるので、
値を渡さないでメモリへの代入をわかりやすくするというコンセプトで行く。
また、変数については最初から場所としておく。
（仮想レジスタも式もないので、一時変数には毎回メモリを割り当てていることになるが、
まあここでやりたいのはそれを回避することではないので。）

\[
\begin{aligned}
\NT{var}    &\defeq \NT{string} \\
\\
\NT{prim-value}    &\defeq \N | \T{\#true} | \T{\#false} \\
\NT{place-expr}   &\defeq \NT{var} | \NT{place-expr} \T{.left} | \NT{place-expr} \T{.right} | \NT{place-expr} \T{\LSB} \NT{value-expr} \T{\RSB} \\
\NT{value-expr}   &\defeq \NT{prim-value} | \T{ld} \NT{place-expr} | \T{inl} \NT{value-expr} | \T{inr} \NT{value-expr} \\
\\
\NT{stmt}   &\defeq \\
  &| \NT{place-expr} \T{:=} \NT{value-expr} \\
  &| \T{inl} \NT{place-expr} \T{=>} \NT{var} \T{\LCB} \NT{stmts} \T{\RCB} \\
  &| \T{inr} \NT{place-expr} \T{=>} \NT{var} \T{\LCB} \NT{stmts} \T{\RCB} \\
  &| \T{if} \NT{value-expr} \T{\LCB} \NT{stmts} \T{\RCB} \\
  &| \T{while} \NT{value-expr} \T{LCB} \NT{stmts} \T{\RCB} \\
\\
\NT{stmts} &\defeq \syntaxmacro{semicolon-separated}{\NT{stmt}} \\
\\
\NT{type} &\defeq \T{Num} | \T{Bool}\\
  &| \T{pair} \NT{type} \NT{type} | \T{sum} \NT{type} \NT{type} \\
  &| \T{arr} \NT{type} \N_{>0} \\
  &| \T{\LP} \NT{type} \T{\RP} \\
\\
\NT{program} &\defeq \NT{static} \syntaxmacro{comma-separated}{\NT{type} \NT{var}} \T{;} \NT{stmts}
\end{aligned}
\]

inl と inr は place を受け取った後に in-place に"中身へのアクセス"を変数経由で受け取るとする。
なので、 `p := inl(1); inl p => x { x := 2 };` を書くと `p := inl(2)` になっている。

> [!warning]
> 中に入った後では `p` への書き換えは `x` への書き換えになっていることに注意する。
> `p := inl(1); inp => x { p := inr(2); x := 1 }` は明らかに不正。
> なので、これを入れた時点で alias の問題が発生している？
> これを考えると、 `alias <var> := <place-expr>` とかを入れてもやらなければいけないことは変わらない。
> 変数が使えなくなったのではなくて、変数は Key に結びついていて、 Valid な Key の全体が変化したと考えたほうが楽？
> オブジェクト指向でも、親が書き変わって子オブジェクトへの名前が使えなくなるとかはあるし。
> その点で、意味論側ではよくない操作をしたらただちに `Err` を吐くようにする必要がある。

コンパイルのことを考えると、変数がどれぐらいセルを使うかを計算するために、実行前にサイズがわかってないといけない。
例えば、 `x := inl(1)` に対して、 `x := inr(?)` が一度も実行されなかったときにはどうなっているのか？
それを確定させておくためにも型の anotation が必要になる。

## 意味論

とりあえず値はこんな感じ。
- \(V_p := \N + \T{\#true} + \T{\#false}\)
- \(V := V_p + (V * V) + [V; \N] + (V + V)\)
  - \(V + V\) 部分は left が \((0, v)\) になり right が \((1, v)\) になっていた。

`ld` を連結した領域に対して行えるので、 \(V_p\) じゃなくて \(V\) の転送ができる。
`x.left := 1; x.right := 2; y := ld x;`

> [!note]
>  `inl p => x { x := 1;} ` が書けるようになった時点で、 variable は他の variable と別の領域を確保しているとは限らない。
> そのため、 variable がそのまま location にはならないから、別に location を用意する必要がある。
> variable -> location が言語の意味論上の変数と位置の束縛を表していて、メモリは location -> value という位置と値の束縛と思えばいい。
> （ location の中に値が入っているというよりも、 location という Key をもとにした key value store と思う。）

> [!note]
> location とメモリの実装について：
> value から \(V_p\) 以外を外して、 location は \(\N\) にするのが一番簡単なモデルだが、
> これだけだと変数側でどれぐらいの連結した領域に言及していたのかを覚えておく必要がある。
> （ ld をするときにどれだけのセルをコピーするのかがわからないから。）
> location を \((\N, \N)\) にして長さも入れるのが一番楽。
> ただこの場合は、 inl/inr をうまく表現できていない。（使われていない部分には何が入っていてもいいとすれば、うまく収まるが。）
>
> ここで定義を変えて、 value から \(V_p\) 以外を外さないで、ある location への書き換えが他の location にも影響すると思うといい。
> 例えば、 location には \(L = \NT{var} + L.\text{left} + L.\text{left}\) みたいなものが入っているとする。
> これで、 \(l \in L\) へ \(v = (1, 0)\) と書き込むと \(l.\text{left}\) は valid な location となって \(v.\text{left} = 1\) が入ると思うことにする。
> - location と型を守るために、メモリ側で validity を検査する必要がある。
>   可能な location は木構造として生成して、実際にアクセス可能かどうかを location -> value 側で制御しておく。
> - inl/inr だけが操作の前後で location のアクセス可能性を変更することになる。
>   `p := inl(expr)` と書くと `inr p => x { }` の中で `x` を経由する path として書かれていた全ての location が無効になる。

- \(L\) := \(\NT{var} + L.\text{left} + L.\text{right} + L[\N] + L.\text{inl} + L.\text{inr}\)
- \(M\) := \(e: L \pfun V\) s.t.
  - \(l.\text{left}\) か \(l.\text{right}\) のどちらかが domain に入ればもう片方も domain に入り、かつ \(l\) 自身も domain に入り：
    - \(e(l) = ( e(l.\text{left}), e(l.\text{right}) )\)
  - \(l[0]\) が domain に入るならある \(n \in \N\) が存在して \(l[i]\) がちょうど \(0 <= i < n\) に対して domain に入る全体になっていて：
    - \(e(l) = [ e(l[0]) \ldots e(l[n - 1]) ]\)
  - \(l.\text{inl}\) と \(l.\text{inr}\) のどちらか一方だけが domain に入り
    - \(e(l) = (0, e(l.\text{inl}))\) if inl が domain に入る
    - \(e(l) = (1, e(l.\text{inr}))\) if inl が domain に入る

> [!warning]
> 写像全体はそれ自体が自由性を持っているので、書き込みについての条件を課さなくてよい。
> 書き込みがどうなるかについては、 \(M[l := v]\) の定義をどのようにするかという話になる。
> \(L\) には親子関係で（全ではない）順序が入るが、
> 順序関係のない \(k, k'\) に対しては \(k\) への書き込みは \(k'\) のメモリに影響を与えないようにして、
> 関係あるメモリについては recursive に書き換えを定義していけばいい。
> ただし、書き込みの話のために、メモリの型を覚えておく必要がある。
> 例えば、 `x := inl(1); x := inr(true); x := inl(false)` は inl に入っている型が変わっているので不正だが、
> 今のメモリに入っている値だけから型は復元できない。

- \(T_M\) := \(t: L \pfun \NT{type}\) s.t.
  - \(l.\text{left}\) か \(l.\text{right}\) のどちらかが domain に入ればもう片方も domain に入り、かつ \(l\) 自身も domain に入り：
    - \(t(l) = \text{pair}  t(l.\text{left}) t(l.\text{right}) \)
  - \(l[0]\) が domain に入るならある \(n \in \N_{> 0}\) が存在して：
    - \(t(l) = \text{arr} t(l[0]) n\)
  - \(l.\text{inl}\) と \(l.\text{inr}\) のどちらかが domain に入ればもう片方も domain に入り、かつ \(l\) 自身も domain に入り：
    \(t(l) = \text{sum}  t(l.\text{left}) t(l.\text{right}) \)
- \(t: T_M\) と \(m: M\) が compatible \(\Leftirighrarrow\) ... _いい感じの関係式_

代入は次のように型を使ってはじくことにする。
- \(\text{assign}(t: T_M, m: \{M \mid \text{compatible with} t\}, l: L, v: V)\) :=
  - _対応する \(l\) の部分が \(v\) の型と合っていたら代入する_
  - _\(l' < l\) な location に対しても \(v[l - l']\) みたいな prefix 除いた部分を使って再帰的に代入する_

気持ちをちゃんと書くのが大変。

### メモリ以外の部分
ところで、 `inl p => x {...}` の導入で必然的にスコープが生まれるので、変数環境はこれを保っておく必要がある。
そのため、 stmt の評価の時点で状態機械が（残りの文を含めた）スタックを持つことになる。
なので、 `loop` と同じようにして loop 内の継続と残りの文の継続を分ける必要がある。

- \(B\) := \(\text{list}(\NT{var}, L)\) ... 変数がどの location に束縛されているかを表すスタック（スコープ用に束縛状況を保存しておく）。
- \(K\) := \(\text{scope} + \text{seq}(\NT{stmt})\) ... ループ内の目印と残りの文の継続
  - ループ内なら全部使った時点でスコープを pop して、そうじゃなかったら変えない。
- \(\text{eval-place}(s: B): \NT{place-expr} \pfun L\) :=
  - \(\NT{var}\) なら \(B\) から引いてくるか、 static にあるならそれを使う。
  - それ以外は recursive に呼び出して \(L\) にする。
- \(\text{eval-value}(s: S): \NT{value-expr} \pfun V\) :=
  - ld のときだけ、 place-expr を評価してメモリから load 値を取り出す。
  - それ以外は recursive に呼び出して \(L\) にする。
- \(\text{eval-stmt}: S \to S\) := \(K\) が \(\text{scope}\) ならそれを \(K\) から削除して \(B\) を pop する。そうじゃない場合は \(K = \text{seq}(s)\) を pop して次のように分岐
  - \(\NT{place-expr} \T{:=} \NT{value-expr}\) なら \(M\) の書き換え
  - \(\T{if} b [s]\) なら \(b\) を評価して
    - true なら \(K := [s] ++ K\) のように extend する。
    - false なら何もしなくていい。
    - それ以外は `Err`
  - \(\T{while} b [s]\) は \(b\) を評価して
    - true なら \(K := [s] ++ [\T{while} b [s]] ++ K\) みたいにする。
    - false なら何もしなくていい。
    - それ以外は `Err`
  - \(\T{inl} p x [s]\) の場合には、 \((x, \text{eval-place}(p))\) を \(B\) に積み、 \(K := [s] ++ \text{scope} ++ K\) みたいにする。

### 実装上のメモリの定義
> [!warning]
> 実装上は \(e: L \pfun V\) なんて使っていられない。
> toplevel の部分だけ覚えておけば、あとは location からそのまま書き換えをすればいい。
> 型も固定できる。

というわけで、
- \(M\) := \(\NT{var} \to (V, \NT{type})\) があればいい。以下のインターフェースだけあれば上記で書いたメモリの話の意味論が置き換えられる。
  - \(\text{get}(m: M, l: L): V\) := _対応する部分を読み取る、型が違ったら error_
  - \(\text{set}(m: M, l: L, v: V)\) :=  _対応する部分を読み取る、型が違ったら error_

これってそのままメモリのちゃんとした定義に使える...やらなくてよかった？
