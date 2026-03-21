[[Model]] [[IR]]

## [[CFG]] について
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

> [!Note]
> ついでに、 rinput と routput をちょっと使いたいので、
> print と input を入れておく。
> 両方とも、 UTF-8 <-> [u8] として解釈する。

<div data-model="flow_ir"></div>

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
\[
\begin{aligned}
\NT{data-label}  &\defeq \T{@} \NT{string} \\
\NT{code-label}  &\defeq \T{:} \NT{string} \\
\NT{vreg}   &\defeq \T{\% v} \NT{number} \\
\\
\NT{value-expr}   &\defeq \NT{vreg} | \NT{imm} | \T{ref}  \NT{place-expr} | \NT{code-label} \\
\NT{place-expr}   &\defeq \T{deref} \NT{vreg} | \NT{data-label} \\
\\
\NT{stmt}   &\defeq ( \\
    &| \NT{vreg}         \T{:=}  \T{ld}  \NT{place-expr} \\
    &| \NT{vreg}         \T{:=}  \NT{value-expr} \\
    &| \NT{vreg}         \T{:=}  \NT{value-expr}  \NT{binop}  \NT{value-expr} \\
    &| \NT{place-expr}   \T{:=}  \T{st}  \NT{value-expr} \\
    &| \T{print} \NT{vreg} \\
    &| \T{input} \NT{place-expr} \\
    ) \T{;} \\
\\
\NT{cond}     &\defeq (\NT{value-expr}  \NT{rel}  \NT{value-expr})\\
\NT{jump-if}  &\defeq \T{if}    \NT{cond}  \T{then}  \NT{value-expr}  \T{;} \\
\NT{jump}     &\defeq \T{goto}  \NT{value-expr}  \T{;} \\
\NT{enter}    &\defeq \T{enter}  \NT{value-expr}  \T{;} \\
\NT{cont}     &\defeq \NT{jump-if}* (\NT{jump}  |  \NT{enter}  |  \T{halt}) \\
\\
\NT{block}  &\defeq \NT{code-label} \T{\{} \NT{stmt}*  \NT{cont} \T{\}} \\
\NT{region} &\defeq \NT{code-label} \T{\{} \NT{block}+ \T{\}} \\
\\
\NT{static}     &\defeq \NT{data-label}  \NT{imm} \\
\NT{program}    &\defeq \NT{static}* \NT{region}+
\end{aligned}
\]

- `%v` を最初につけたのは、後で実レジスタを `%r` で使えるように。

### 意味
- \(V := \N + \NT{code-label} + K\) ... これは仮想レジスタに入れうる値
- place/store/environment 周りについて
  - \(K := K_{\text{static}}\), \(K_{\text{static}} := \NT{data-label}\) ただし \(\NT{static}\) で述べられたもののみ。
      -  valid なのは、コード中に存在しているラベルのみ。
      - cfg_vreg の方では \(\NT{label} \to \N\) というアドレスの解決を入れていた（ symbol table ）が、それをなくして直接 key とする。
      - \(\NT{label}\) の方には演算が入らないので、 \(V\) 上の演算は partial なものとして、不正な演算は `Err` で処理する。
      - "最終的には"アドレスでの解決がされるものの、
        その"最終的には"というのは `trait Compiler` 側の話なので、 `trait Machine` の方ではそのようには実装しない。
  - \(\text{static-env} := K_{\text{static}} \pfun V\) ... static place に入っている値の環境
  - \(\text{place-env} := \text{static-env}\) ... place に入っている値の環境
  - \(\text{vreg-env} := \NT{vreg} \to V\) ... vreg に入っている値の環境
      - default env として \(0_{\text{vreg-env}} = v \mapsto 0\) を考えておく。
      - region を超える場合（ enter による jump ）には、この default value を用いる。
      - block 内での jump だった場合には、同じ vreg-env を用いる。
  - \(\text{env} := \text{place-env} \times \text{vreg-env}\)
- eval の話
  - \(\text{eval-value}(e: \text{env}, v: \NT{value-expr}): V\) :=
      - \(v \in \NT{imm}\) ならそのまま \(v = i \in \N\) を使う。
      - \(v \in \NT{vreg}\) なら vreg-env から \(V\) を取り出す。
      - \(v \in \T{ref} \NT{place-expr}\) なら
        \(\text{eval-place}(e, v) = k \in K\) となる場合に \(k\) を返し、それ以外は `Err` で処理する。
  - \(\text{eval-place}(e: \text{env}, p: \NT{place-expr}): K\) :=
      - \(p \in K\) ならそのまま \(K\) をとる。
      - \(p \in \T{deref} \NT{vreg}\) ならそこから値を取り出して、 \(k \in K\) だったときにそれを返しそれ以外は `Err` で処理する。
  - \(\text{eval-stmt}\) :=
      - ld は place-expr を評価して出た K で指される場所から V を取り出すだけ。
      - st は place-expr を評価して出た K で指される場所に V を入れるだけ。
      - 残りは普通に演算
- 制御の話
    - \(\text{value-expr}\) の評価結果が code-label であるならちゃんとジャンプができるとする。
    - ジャンプ先の名前の解決は、 1. 同じ region 内のブロックか？ 2. 異なる region の名前か？ でやる。
- block 内でのジャンプ時には vreg-env は同じままとするが、 region を超える場合は \(0_{\text{vreg-env}}\) を用いる。

> [!Note]
> この定義の仕方を見ると、 \(\T{deref}  \T{ref}  p = p\) となるのが対応関係？
> \(\T{ref}  \T{deref}  (v: \NT{vreg})\) の意味は、 vreg へ代入されたラベルの取り出しになっている。
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
  - このハンドルについては、"下からの長さ"でとることとする。例： `get(0); push(10); get(0)` で `get(0)` は同じ場所になる。
- スタックの長さを取得できるようにする： `lget` で長さを値にする。

### 構文
\[
\begin{aligned}
\\
\NT{place-expr} &\defeq \ldots \\
  &|  \T{sacc}  \NT{value-expr} \\
\\
\NT{stmt} &\defeq \ldots \\
  &|  \T{pop}  \NT{vreg} \\
  &|  \T{push}  \NT{value-expr} \\
  &|  \T{lget}  \NT{vreg} \\
\end{aligned}
\]

### 意味
\(S: \text{stack}\) に対して \(S[i: \N]\) で _ちゃんと値が入っているとき_ の場所を表す。

- \(K_{\text{stack}} = \N\), \(K := \cdots + K_{\text{stack}}\)
- \(\text{stack-env} := K_{\text{stack}} \pfun V\), \(\text{place-env} = \ldots \times \text{stack-env}\) とする。
- \(\text{eval-place}(\T{sacc}  (v \in \NT{value-expr}) )\) :=
  \(i \in K_{\text{stack}}\) ただし \(\text{eval-value}(v) = i \in \N\) のとき（それ以外はエラーとする）。
- \(\text{eval-stmt}(\T{pop}  (v \in \NT{vreg}))\) :=
  pop した値を \(v\) に入れる
- \(\text{eval-stmt}(\T{push}  (v \in \NT{value-expr}))\) :=
  \(\text{eval-value}(v) = v'\) の結果を push する
- \(\text{eval-stmt}(\T{lget}  (v \in \NT{vreg}))\) :=
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
\[
\begin{aligned}
\NT{place-expr} &\defeq \ldots \\
  &|  \T{hacc}  \T{\LP}  \NT{value-expr}  \T{\RP}  \T{\LCB}  \NT{value-expr}  \T{\RCB} \\
\\
\NT{stmt} &\defeq \ldots \\
  &|  \T{halloc}  \T{\LP}  \NT{value-expr}  \T{\RP}  \NT{vreg} \\
  &|  \T{hfree}  \NT{value-expr} \\
\end{aligned}
\]

### 意味
初めに、 Handle の集合として \(H\) を入れておく。運用上はこれは `usize` とか `Number` でいい。
要は、既に使われている集合があったらそことかぶらないようにとれればいい。

- \(K_{\text{heap}} = H \times \N\), \(K = \cdots + K_{\text{heap}}\)
- \(\text{heap-env} = H \pfun (n \in \N, 0 \ldots n \to \N)\), \(\text{place-env} = \cdots \times \text{heap-env}\)
  \(H\) がちゃんと domain に入っている場合が handle が割り振られている場合で、
  そのときに size \(n\) と各 index \(i \in 0 \ldots n\) ごとに値が格納された領域があるとする。
  halloc で確保されて hfree されていないような handle と、その長さの中に入っている index を合わせた \(K\) のみが有効とする。
- \(\text{eval-place}(\T{hacc}  \T{\LP}  v_1 \in \NT{value-expr}  \T{\RP}  \T{\LCB}  v_2 \in \NT{value-expr}  \T{\RCB}) \) :=
  \(\text{eval-value}(v_1) = h \in H\) で \(\text{eval-value}(v_2) = i \in \N\) に評価されるときに、 \((h, i) \in K_{\text{heap}}\) とする。
- \(\text{eval-stmt}(\T{halloc}  \T{\LP}  (v_1 \in \NT{value-expr})  \T{\RP}  (v \in \NT{vreg}) )\) :=
  \(\text{eval-value}(v_1) = n \in \N\) のときに、 \(e_h\) の Domain には含まれてない新しい handle を取り出して、
  それと \(0\) fill されたサイズが \(n\) の領域へのマップを作る。
- \(\text{eval-stmt}(\T{hfree}  (h \in \NT{value-expr}) )\) :=
  \(\text{eval-value}(v) = h \in H\) に評価されて、 \(e_h\) の Domain に含まれているときに、それを Domain から取り除く。
