CFG について：
ブロックの最後に terminator があって仮想レジスタを使うモデルを拡張したものを考える。
なお、ジャンプは computed value でのジャンプができるが、ジャンプ先としてはブロックの先頭のみ与えられるとする。
スコープについて：
1. 仮想レジスタのスコープをデフォルトでブロック内に限り、残りは引数を用いる。
2. 仮想レジスタのスコープをデフォルトでブロック内に限り、残りは phi を用いる。
    - これは phi/upsilon 方式とかでもいい
3. 仮想レジスタのスコープは全てのブロックとする。
    - 関数ポインタと合わせて、全然最適化ができなくなりそう。

ところで、 CPS 形式の場合は手続き形式のブロックではなくて、こういった CFG 形式になっている。
手続き形式を一切用いずに毎回 jump 先を渡し続けることができれば（"jump 先を渡す"方法についてはさておき）、 return がいらなくなる。
手続き形式は継続をコントロールすることが目的と考えると、 IR のレベルで手続き形式が導入されているのは不思議に感じる。
むしろ継続をコントロールできる表現力がないといけないのに、 build-in で入っているのは、実質マクロみたいなものと思っている？
（マクロとしては、どっかから return 先を持ってきて jump するだけの展開をすればいい。）

`cfg_vreg` は 3. になっているが、いきなり 1. や 2. にするよりも別のやり方を考える。
単純にスコープを区切りつつ CFG をまとめるだけの概念の region を入れる。

## region の導入
仮想レジスタのスコープを切る仕組みを入れる。
それと、この言語ではメモリに直接アクセスできるとちょっと嫌なことが起きそうなので、その点を少し変形する。
変形の内容としては、 place を全メモリと思って addr をとるのをやめて、
最初に宣言した static なところにアクセスしようとしていることを明言する。
そのため、レジスタに入っているのを使ってもラベルを使っても構わないが、
それで割り振られていない"未使用"な部分にこれでアクセスするのは禁止とする。
また、アドレス演算を禁止する。（アドレスというより、参照みたいな扱い？）

- 引数はなし、制御は関数ではなく jump として、異なるスコープの途中へのジャンプは考えないものとする。
- ラベルについてもコード用とデータ用に分ける。
- どこでメモリとのやり取りが起こるかをわかりやすくするために load/store を入れる。

### 構文
\(\begin{aligned}
\NT{data-label}  &\defeq \T{@} \sp \NT{string} \\
\NT{code-label}  &\defeq \T{@} \sp \NT{string} \\
\NT{vreg}   &\defeq \T{\%v} \sp \NT{number} \\

\NT{value-expr}   &\defeq \NT{vreg} \sp | \sp \NT{imm} \sp | \sp \T{ref} \sp \NT{place-expr} \\
\NT{place-expr}   &\defeq \T{deref} \sp \NT{value-expr} \sp | \sp \NT{data-label} \\

\NT{stmt}   &\defeq ( \\
    &| \sp \NT{vreg}        \sp \T{:=} \sp \T{ld} \sp \NT{place-expr} \\
    &| \sp \NT{vreg}        \sp \T{:=} \sp \NT{value-expr} \\
    &| \sp \NT{vreg}        \sp \T{:=} \sp \NT{value-expr} \sp \NT{binop} \sp \NT{value-expr} \\
    &| \sp \NT{place-expr}  \sp \T{:=} \sp \T{st} \sp \NT{value-expr} \\
    ) \T{;} \\

\NT{cond}     &\defeq (\NT{value-expr} \sp \NT{rel} \sp \NT{value-expr})\\
\NT{jump-if}  &\defeq \T{if}   \sp \NT{cond} \sp \T{then} \sp \NT{place-expr} \sp \T{;} \\
\NT{jump}     &\defeq \T{goto} \sp \NT{place-expr} \sp \T{;} \\
\NT{enter}    &\defeq \T{enter} \sp \NT{place-expr} \sp \T{;} \\
\NT{cont}     &\defeq \NT{jump-if}* (\NT{jump} \sp | \sp \NT{enter} \sp | \sp \T{halt}) \\

\NT{block}  &\defeq \NT{code-label} \T{\{} \NT{stmt}* \sp \NT{cont} \T{\}} \\
\NT{region} &\defeq \NT{code-label} \T{\{} \NT{block}+ \T{\}} \\

\NT{static}     &\defeq \NT{data-label} \sp \NT{imm} \\
\NT{program}    &\defeq \NT{static}* \NT{region}+
\end{aligned}\)

### 意味
- \(V := \N + \text{static-code}(\NT{code-label}) + \text{static-data}(\NT{data-label})\) ... これは仮想レジスタに入れうる値
    - めんどくさくて省いているが、 valid なのは、コード中に存在しているラベルのみ。
    - cfg_vreg の方では \(\NT{label} \to \N\) というアドレスの解決を入れていた（ symbol table ）が、それをなくして直接 key とする。
    - \(\text{static}(\NT{label})\) の方には演算が入らないので、 \(V\) 上の演算は partial なものとして、不正な演算は `Err` で処理する。
    - "最終的には"アドレスでの解決がされるものの、
      その"最終的には"というのは `trait Compiler` 側の話なので、 `trait Machine` の方ではそのようには実装しない。
- \(\text{static-env} := \NT{data-label} \to V\) ... static place に入っている値の環境
- \(\text{place-env} := \NT{static-env}\) ... place に入っている値の環境
- \(\text{vreg-env} := \NT{vreg} \to V\) ... vreg に入っている値の環境
    - default env として \(0_{\text{vreg-env}} = v \mapsto 0\) を考えておく。
    - region を超える場合（ enter による jump ）には、この default value を用いる。
    - block 内での jump だった場合には、同じ reg を用いる。
- \(\text{env} := \text{place-env} + \text{vreg-env}\)
- \(\text{eval-value}(e: \text{env}): \NT{value-expr} \to V\) :=
    - \(v \in \NT{vreg} | \NT{imm}\) なら vreg-env を使う。
    - \(v \in \T{ref} \NT{place-expr}\) なら
      \(\text{eval-place}(e, v) = l \in \NT{code-label}\) となる場合に \(l\) を返し、それ以外は `Err` で処理する。
- \(\text{eval-place}(e: \text{env}): \NT{place-expr} \to V\) :=
    - \(v \in \T{deref} \NT{value-expr}\) なら
      \(\text{eval-value}(e, v) = l \in \NT{data-label}\) となる場合に \(l\) を返し、それ以外は `Err` で処理する。
- \(\text{eval-stmt}, \text{eval-cond}\) これは置いておく。
- 制御周りでは \(\text{place-expr}\) の評価結果が code-label であるならちゃんとジャンプができるとする。
- block 内でのジャンプ時には vreg-env は同じままとするが、 region を超える場合は \(0_{\text{vreg-env}}\) を用いる。

> [!Note]
> この定義の仕方を見ると、 \(\T{deref} \sp \T{ref} \sp p = p\) となるのが対応関係？
> \(\T{ref} \sp \T{deref} \sp (v: \NT{vreg})\) の意味は、 vreg へ代入されたラベルの取り出しになっている。
> また、ラベルをアドレス値として扱うなら value expression の方に置くべきだが、直接 place と思うことにした。

## スタック領域
region をまたがってデータを渡すときに共有される用のメモリ領域を作る。
関数の引数みたいなものに使ったりするので、ただしく使われている回数を数えないと大変なことになるので、
このコードを書く側は頑張らないといけない。
普通のスタックの場合はこんな感じ？
- `push(): V -> ()` と `pop(): () -> V`
- `get(index: Number): () -> V` と `set(index: Number): V -> ()` でアクセス

今回はこんな感じで入れる。
- `push`, `pop` は明らかに値だが、副作用っぽいので stmt の方に入れておく。
- `get`, `set` は place を返すように統合する。値としての \(\N\) をハンドル代わりに受け取る。
- スタックの長さを取得できるようにする： `lget` で長さを値にする。

### 構文
\(\begin{aligned}

\NT{place-expr} &\defeq \ldots \\
  &| \sp \T{sacc} \sp \NT{value-expr} \\

\NT{stmt} &\defeq \ldots \\
  &| \sp \T{pop} \sp \NT{vreg} \\
  &| \sp \T{push} \sp \NT{value-expr} \\
  &| \sp \T{lget} \sp \NT{vreg} \\
\end{aligned}\)

### 意味
スタックはいくつかの方法で定義できる： \(V*\) か \(\N \to (V + \bot)\) で順序を保つ写像。
前者の実装はどうせ後者になりそう。まあどっちでもいいか...
\(S: \text{stack}\) に対して \(S[i: \N]\) で _ちゃんと値が入っているとき_ の場所を表す。

- \(\text{stack-env} := V*\), \(\text{place-env} = \ldots \times \text{stack-env}\) とする。
- \(\text{eval-place}(\T{sacc} \sp (v \in \NT{value-expr}) )\) :=
  \(S[ i ]\) ただし \(\text{eval-value}(v) = i \in \N\) のとき（それ以外はエラーとする）。
- \(\text{eval-stmt}(\T{pop} \sp (v \in \NT{vreg}))\) :=
  pop した値を \(v\) に入れる
- \(\text{eval-stmt}(\T{push} \sp (v \in \NT{value-expr}))\) :=
  \(\text{eval-value}(v) = v'\) の結果を push する
- \(\text{eval-stmt}(\T{lget} \sp (v \in \NT{vreg}))\) :=
  スタックの長さを \(v\) に入れる。

> [!Note]
> 当初考えていたタイプの region にスコープを持つ局所変数についてはここでは扱わないことにした。
> コンパイルするときに、共有されるスタックとの一本化が難しい。完全に独立にすると、 region で制御が戻ってこないので...
> 共有スタックの一部に名前でアクセスする機能としての局所変数も考えられる
> （スコープを抜けると名前がなくなるだけでスタック上からは消えないタイプ）
> が、これはこれで、あまりやる意味を感じなかった。

## ヒープ領域
これはスコープを飛び越えて共有される、ハンドル**値**をもとにアクセスするタイプのヒープを用意する。
特に、ちゃんと解放するためにハンドル値が失われないように制御しないといけない。

- `halloc(size: Number): () -> Handle` でハンドル値を返す...これを vreg 相当のところやスタックに入れるかは自由にする。
- `hfree(handle: Handle): ()` でその領域を解放する
- `hget(handle: Handle, index: Number)` は（存在するなら） place を返すようにする。
    - この"存在するなら"は、 handle が割り振られていること、確保されたサイズの中の index が指し示されていること。

> [!Note]
> 普通にヒープがない世界は主流らしい。
> ヒープがあるこの言語は高級言語だが、あまりコンパイル先として選ぶときに悩みたくないので入れる。

### 構文
\(\begin{aligned}
\NT{place-expr} &\defeq \ldots \\
  &| \sp \T{hacc} \sp \T{\LP} \sp \NT{value-expr} \sp \T{\RP} \sp \T{\LSB} \sp \NT{value-expr} \sp \T{\RSB} \\

\NT{stmt} &\defeq \ldots \\
  &| \sp \T{halloc} \sp \T{\LP} \sp \NT{value-expr} \sp \T{\RP} \sp \NT{vreg} \\
  &| \sp \T{hfree} \sp \NT{value-expr} \\
\end{aligned}\)

### 意味
初めに、 Handle の集合として \(H\) を入れておく。運用上はこれは `usize` とか `Number` でいい。
要は、既に使われている集合があったらそことかぶらないようにとれればいい。

- \(V = \cdots + H\) こうやって handle 値自体も値の集合に直和で入れておく。
- \(\text{heap-env} = H \pfun (n \in \N, 0 ldots n \to \N)\), \(\text{place-env} = \cdots \times \text{heap-env}\)
  \(H\) がちゃんと domain に入っている場合が handle が割り振られている場合で、
  そのときに size \(n\) と各 index \(i \in 0 \ldots n\) ごとに値が格納された領域があるとする。
- \(\text{eval-place}(\T{hacc} \sp \T{\LP} \sp v_1 \in \NT{value-expr} \sp \T{\RP} \sp \T{\LSB} \sp v_2 \in \NT{value-expr} \sp \T{\RSB}) \) :=
  \(\text{eval-value}(v_1) = h \in H\) で \(\text{eval-value}(v_2) = i \in \N\) に評価されるときに、
  \((e, e_s, e_h \in \text{heap-env}): \text{env}\) を用いて \(e_h (h) = (n, f)\) とちゃんと handle が存在して、 \(i < n\) の場合に、
  \(f(i)\) という場所を指すとする。
- \(\text{eval-stmt}(\T{halloc} \sp \T{\LP} \sp (v_1 \in \NT{value-expr}) \sp \T{\RP} \sp (v \in \NT{vreg}) )\) :=
  \(\text{eval-value}(v_1) = n \in \N\) のときに、 \(e_h\) の Domain には含まれてない新しい handle を取り出して、
  それと \(0\) fill されたサイズが \(n\) の領域へのマップを作る。
- \(\text{eval-stmt}(\T{hfree} \sp (h \in \NT{value-expr}) )\) :=
  \(\text{eval-value}(v) = h \in H\) に評価されて、 \(e_h\) の Domain に含まれているときに、それを Domain から取り除く。

> [!Note]
> 一度 stmt にある place からの load/store について考えると、 rust ではいずれにせよ place-expr 自体を `&mut` とかで実装すればいいが、
> 改めて place とは何かを問われると難しい。
> ここまでスタックとヒープを定義した後に各 place-expr ごとに左辺や右辺に来た時にどうするかを定義するなら楽だが、
> 一般の place 環境に対する操作を上げろと言われたら、それは代入？になりそう。
> つまり、 \(K\) をカギとする \(V\) 上の環境というものを partial function \(K \pfun V\) としたときに、
> static は \(K\) がラベル、スタックは \(K = \N\) でヒープは \(K = H\) みたいになっていて、
> いずれにせよ \(\NT{vreg} \T{:=} \T{ld} (p \in \NT{place}) \) は \(p\) に対応する値を取り出して、 st は \(K \pfun V\) の上書き操作をしている。
> だから、これの仕組みさえあればもっと変なメモリ領域を考えても統一的に書けるはず。
