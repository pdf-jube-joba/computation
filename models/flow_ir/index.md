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
- 制御周りでは \(\text{place-expr}\) の評価結果が code-label であるならちゃんとジャンプができる。
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

### 構文
\(\begin{aligned}

\NT{place-expr} &\defeq \ldots \\
  &| \sp \T{pget} \sp \NT{value-expr} \\

\NT{stmt} &\defeq \ldots \\
  &| \sp \T{pop} \sp \NT{vreg} \\
  &| \sp \T{push} \sp \NT{value-expr}
\end{aligned}\)

### 意味
スタックはいくつかの方法で定義できる： \(V*\) か \(\N \to (V + \bot)\) で順序を保つ写像。
前者の実装はどうせ後者になりそう。まあどっちでもいいか...
\(S: \text{stack}\) に対して \(S[i: \N]\) でちゃんと値が入っているときの場所を表す。

- \(\text{stack-env} := V*\), \(\text{place-env} = \ldots + \text{stack-env}\) とする。
- \(\text{eval-place}(\T{pget} \sp (v \in \NT{}) )\) :=
  \(S[ i ]\) ただし \(\text{eval-value}(v) = i \in \N\) のとき（それ以外はエラーとする）。
- \(\text{eval-stmt}(\T{pop} \sp (v \NT{vreg}))\) :=
    - pop した値を \(v\) に入れる
- \(\text{eval-stmt}(\T{push} \sp (v \NT{value-expr}))\) :=
    - \(\text{eval-value}(v) = v'\) の結果を push する

## region 内での局所領域
region の中で共有される、 offset 付きのメモリ領域を述べる。
- ブロック単位では宣言ができないものとする。
- 仮想レジスタとは異なる"変数"を導入し、メモリ領域はこの変数に結びつけるものとする。
- region の先頭で宣言して確保し、 region を抜けるときに開放する。
- サイズの指定は即値とする。

- `salloc(var: Variable, size: Number): ()` で変数の宣言
- `sget(var: Variable, index: Number): () -> V` と `sset(var: Variable, index: Number): V -> ()` でアクセス
- 変数のスコープが region に限られるので、region を抜けたら自動的に開放する。
  - 実質的には共有される仮想レジスタと同じだが、addr が取れるのが別でいいところかもしれない。
  - そのため、仮想レジスタとは別の「変数」という概念として導入したい。

> [!Warning]
> 関数の呼び出しみたいなことをしたい場合は region を抜けるしかない（あるいは、 region 内でやるしかない）ので、
> 引数を通常のように push/pop をして 引数やら保存したい引数やら戻り先アドレスやらをスタックに push するしかない。
> なので、その目的でこの領域を使うことはできない。

\(\begin{aligned}
\NT{local} &\defeq \T{\$} \sp \NT{string} \\

\NT{place-expr}
\end{aligned}\)

## ヒープ領域
- 自由に呼び出してちゃんと `free` で返す必要のある領域。
  - `halloc(size: Number): () -> Handle` でハンドル値を返す...これを vreg 相当のところやスタックに入れるかは自由にする。
  - `hfree(handle: Handle): ()` でその領域を解放する
  - `hget(handle: Handle, index: Number): () ->  V` と `hset(handle: Handle, index: Number): V -> ()` でアクセス
- これはスコープを飛び越えて共有される
